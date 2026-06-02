# CVKG Renderer — Complete Implementation Plan (Kvasir Graph)

**Based on:** Surtr-Arch-Review.md (14 sections, 11,343 lines)
**Feasibility:** Kvasir_Graph_Implementation_Plan_1.md (900 lines)
**Starting state:** v0.2.7 on crates.io, 597 tests passing, Phase 0 (presentation fixes) already committed

---

## Architecture Summary

**Current:** Manually orchestrated hardcoded pass sequence in `end_frame()`. Glass samples blur before it's generated. Four independent blur systems. Monolithic 140-byte vertex format. 10+ material modes multiplexed in a single fragment shader via raw `u32` branches. No dependency validation. No resource lifetime tracking.

**Target:** Kvasir — a unified visual computation graph. Every operation is a typed graph node with declared input/output resources. The execution planner derives correct barrier insertion, dead-node elimination, and pass merging automatically. Glass and bloom share a single ImagePyramid. Materials are pipeline-specialized. Accessibility is a native graph stage. Zero hardcoded ordering.

---

## Bug Fixes Required Before Graph Work (Phase 0)

These bugs corrupt output regardless of architecture. Fix first.

| ID | Bug | Fix |
|----|-----|-----|
| P0-1 | `fs_copy` reads Mega-Atlas (`t_diffuse[0]`) instead of scene texture for backdrop copy | Copy pass must bind `scene_texture` bind group, not dummy. |
| P0-2 | `stroke_path` DrawCall uses `compositor_index_cursor` (vertex offset) as index cursor — indices point to wrong vertices | Capture `base_index = self.indices.len() as u32` before tessellation, use it as `index_start`. |
| P0-3 | Parallel rayon Glass/UI passes share `ctx_scene_texture` without barrier — UB in wgpu | Encode sequentially (already fixed in Phase 0 commit). |
| P0-4 | Bloom extract overwrites `blur_texture_a` which holds backdrop blur | Allocate separate `bloom_tex_a` / `bloom_tex_b`. |
| P0-5 | `vs_fullscreen` draws 6 vertices — second triangle is degenerate (verticies 0,2,2) | Change all fullscreen draws from `0..6` to `0..3`. |
| P0-6 | Glyph atlas fallback writes to `(0,0)` on full atlas instead of returning error | Return `Err` early with log when atlas is full. |
| P0-7 | SVG tessellation `.unwrap()` panics on malformed paths | Propagate as `Result`, skip degenerate paths. |
| P0-8 | Clip SDF in shape shader uses `clip_position.xy` as NDC but values are already window-space pixels | Remove the `* 0.5 + 0.5 * resolution` transform. |
| P0-9 | Bifrost glass alpha 1–3% — fresnel alpha calculation too aggressive | Fix fresnel term in mode 7 branch. |
| P0-10 | blur_pyramid.wgsl `@Override` keyword and missing `@` on `group(0)` — shader won't compile | Replace with `@group(0) @binding(0)`. |

---

## Phase 1 — Resource Graph Foundation

**Goal:** Every GPU resource gets a `ResourceId` and `ResourceDescriptor`. The renderer behavior doesn't change — this is a naming/tracking layer.

### 1.1 Resource Type System

New file: `cvkg-render-gpu/src/kvasir/resource.rs`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(u64);

pub enum ResourceKind {
    Image { width: u32, height: u32, format: wgpu::TextureFormat, mip_levels: u32 },
    Geometry { vertex_count: u32, index_count: u32 },
    VectorPath,
    Material,
    Animation,
    Accessibility,
    Scene,
}

pub enum ResourceLifetime {
    Frame,        // allocation + content transient — freed at end of frame
    FrameContent, // allocation persists, content rebuilt (blur/pyramid textures)
    Persistent,   // lives across frames (atlas, scene geometry)
    Streaming,    // loaded async, may not be ready
}

pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub label: &'static str,
    pub lifetime: ResourceLifetime,
}
```

### 1.2 ResourceRegistry

New file: `cvkg-render-gpu/src/kvasir/registry.rs`

```rust
pub struct ResourceRegistry {
    descriptors: HashMap<ResourceId, ResourceDescriptor>,
    gpu_images:  HashMap<ResourceId, wgpu::Texture>,
    gpu_buffers: HashMap<ResourceId, wgpu::Buffer>,
    lru:         LruCache<ResourceId, ()>,
    next_id:     AtomicU64,
}

impl ResourceRegistry {
    pub fn register(&mut self, desc: ResourceDescriptor) -> ResourceId;
    pub fn get_image(&self, id: ResourceId) -> Option<&wgpu::Texture>;
    pub fn get_or_create_image(&mut self, desc: &ResourceDescriptor, device: &wgpu::Device) -> ResourceId;
    pub fn evict_frame_resources(&mut self);
    pub fn reclaim(&mut self, budget_vram: u64);
}
```

### 1.3 Register All Existing Resources

In `forge_internal()` / `register_window()` — name every texture:

```
scene_color      → Image { Rgba16Float, FrameContent }
depth            → Image { Depth32Float, FrameContent }
blur_a           → Image { Rgba16Float, FrameContent }
blur_b           → Image { Rgba16Float, FrameContent }
bloom_a          → Image { Rgba16Float, Frame }
bloom_b          → Image { Rgba16Float, Frame }
mega_atlas       → Image { Rgba8UnormSrgb, Persistent }
```

The renderer still manages these directly for now. The registry tracks names and IDs only.

### 1.4 Replace `reclaim_vram()`

The existing `update_vram_telemetry()` + manual VRAM tracking is replaced by `ResourceRegistry::reclaim()` which uses the LRU cache for eviction ordering.

**Deliverable:** `ResourceRegistry` tracks all GPU resources by name. Renderer behavior unchanged. All P0 bugs fixed. 597+ tests pass.

---

## Phase 2 — Kvasir Graph Core + Execution Planner

**Goal:** Graph data structure → topological sort → barrier insertion → execution. This is the largest engineering piece.

### 2.1 KvasirNode Trait

New file: `cvkg-render-gpu/src/kvasir/node.rs`

```rust
pub trait KvasirNode: Send + Sync {
    fn label(&self) -> &'static str;
    fn inputs(&self)  -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];
    fn execute(
        &self,
        ctx: &mut ExecutionContext,
        registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError>;
    fn execution_hint(&self) -> ExecutionHint { ExecutionHint::Raster }
}

pub enum ExecutionHint { Raster, Compute, Hybrid }

pub enum KvasirError {
    MissingInput(ResourceId),
    ResourceConflict { resource: ResourceId, existing: AccessMode, requested: AccessMode },
    ExecutionFailed { node: &'static str, source: Box<dyn Error + Send + Sync> },
}
```

### 2.2 KvasirGraph

New file: `cvkg-render-gpu/src/kvasir/graph.rs`

```rust
pub struct KvasirGraph {
    nodes:  SlotMap<NodeKey, Box<dyn KvasirNode>>,
    edges:  Vec<Edge>,        // (producer, ResourceId, consumer)
    roots:  Vec<NodeKey>,
    sinks:  Vec<NodeKey>,
}

pub struct ExecutionPlan {
    pub ordered_nodes: Vec<NodeKey>,
    pub barriers:      Vec<BarrierPoint>,
    pub resource_aliases: HashMap<ResourceId, ResourceId>, // transient reuse
}

impl KvasirGraph {
    pub fn add_node(&mut self, node: impl KvasirNode + 'static) -> NodeKey;
    pub fn connect(&mut self, from: NodeKey, resource: ResourceId, to: NodeKey);
    pub fn set_sink(&mut self, key: NodeKey);
    pub fn validate(&self) -> Result<(), Vec<KvasirError>>;  // cycle detection + type check
    pub fn compile(&self, registry: &ResourceRegistry) -> Result<ExecutionPlan, KvasirError>;
}
```

### 2.3 Execution Planner

New file: `cvkg-render-gpu/src/kvasir/planner.rs`

The planner does:
1. **Topological sort** (Kahn's algorithm — error on cycle)
2. **Lifetime analysis** — first write / last read per resource
3. **Transient aliasing** — reuse GPU memory for non-overlapping frame resources
4. **Barrier insertion** — insert `wgpu::TextureBarrier` when a resource transitions from write to read between nodes (this is what prevents the P0-3 parallel encoding UB)
5. **Pass merging** — adjacent raster passes targeting the same texture can share a render pass

```rust
pub struct ExecutionPlanner;

impl ExecutionPlanner {
    pub fn plan(
        graph: &KvasirGraph,
        registry: &ResourceRegistry,
    ) -> Result<ExecutionPlan, KvasirError> {
        let sorted = topological_sort(graph)?;
        let lifetimes = analyze_lifetimes(&sorted, graph);
        let aliases = compute_transient_aliases(&lifetimes);
        let barriers = insert_barriers(&sorted, &lifetimes, graph);
        Ok(ExecutionPlan { ordered_nodes: sorted, barriers, resource_aliases: aliases })
    }
}
```

### 2.4 Convert All Passes to KvasirNodes

New file: `cvkg-render-gpu/src/kvasir/nodes/`

Each existing pass gets a node wrapper with declared resource I/O:

```rust
// background.rs
pub struct BackgroundNode { pub output: ResourceId }
// scene_geometry.rs — renders all opaque draw_calls
pub struct SceneGeometryNode { pub scene_output: ResourceId, pub depth_output: ResourceId, pub material: DrawMaterial }
// backdrop_copy.rs — SceneTexture → BlurTexture (identity fs_copy, ALL pixels)
pub struct BackdropCopyNode { pub input: ResourceId, pub output: ResourceId }
// backdrop_blur.rs — Kawase downsample/upsample on BlurTexture
pub struct BackdropBlurNode { pub input: ResourceId, pub output: ResourceId, pub mip_levels: u32, pub iterations: u32 }
// glass.rs — samples from blur mip chain, draws glass quads
pub struct GlassNode { pub blur_input: ResourceId, pub scene_input: ResourceId, pub scene_output: ResourceId }
// ui.rs — draws TopUI draw_calls
pub struct UINode { pub scene_input: ResourceId, pub scene_output: ResourceId }
// bloom_extract.rs — luminance gate
pub struct BloomExtractNode { pub scene_input: ResourceId, pub bloom_output: ResourceId }
// bloom_blur.rs — Kawase on bloom texture
pub struct BloomBlurNode { pub input: ResourceId, pub output: ResourceId, pub iterations: u32 }
// bloom_composite.rs — additive blend + tonemap
pub struct CompositeNode { pub scene_input: ResourceId, pub bloom_input: ResourceId, pub output: ResourceId }
// accessibility.rs — color blind transform
pub struct AccessibilityNode { pub input: ResourceId, pub output: ResourceId, pub mode: ColorBlindMode }
// present.rs
pub struct PresentNode { pub surface: wgpu::Surface<'static> }
```

### 2.5 Correct Frame Graph (After All Fixes)

The validated execution order the planner derives:

```
BackgroundNode        → clears scene_color + depth
       │
SceneGeometryNode     → draws opaque calls into scene_color
       │
BackdropCopyNode      → copies scene_color → blur_a (identity, ALL pixels)
       │
BackdropBlurNode      → Kawase downsample/upsample on blur_a
       │                    Generates mip pyramid (5 levels)
       │
GlassNode            → draws glass quads, samples blur_a mips for backdrop
       │
UINode               → draws overlay quads
       │
BloomExtractNode     → samples scene_color → bloom_a (luminance > 0.8 gate)
       │
BloomBlurNode        → Kawase downsample/upsample on bloom_a (same pipelines!)
       │
CompositeNode        → blends scene_color + bloom_a → swapchain + ACES tonemap
       │
AccessibilityNode    → reads swapchain → color transform → swapchain
       │
PresentNode          → surface.present()
```

### 2.6 Frame Graph Construction (What end_frame Becomes)

```rust
pub fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
    let mut graph = KvasirGraph::new();
    let has_glass = self.draw_calls.iter().any(|c| matches!(c.material, Glass{..}));
    let has_bloom = self.bloom_enabled;

    // ── Nodes ──
    let bg = graph.add_node(BackgroundNode { output: self.scene_color_id });
    let geo = graph.add_node(SceneGeometryNode {
        scene_output: self.scene_color_id,
        depth_output: self.depth_id,
    });
    graph.connect(bg, self.scene_color_id, geo);

    let mut last_scene_writer = geo;

    if has_glass {
        let copy = graph.add_node(BackdropCopyNode {
            input: self.scene_color_id,
            output: self.blur_a_id,
        });
        let blur = graph.add_node(BackdropBlurNode {
            input: self.blur_a_id,
            output: self.blur_a_id,
            mip_levels: 5,
            iterations: 4,
        });
        let glass = graph.add_node(GlassNode {
            blur_input: self.blur_a_id,
            scene_input: self.scene_color_id,
            scene_output: self.scene_color_id,
        });
        graph.connect(last_scene_writer, self.scene_color_id, copy);
        graph.connect(copy, self.blur_a_id, blur);
        graph.connect(blur, self.blur_a_id, glass);
        graph.connect(last_scene_writer, self.scene_color_id, glass);
        last_scene_writer = glass;
    }

    let ui = graph.add_node(UINode {
        scene_input: self.scene_color_id,
        scene_output: self.scene_color_id,
    });
    graph.connect(last_scene_writer, self.scene_color_id, ui);
    last_scene_writer = ui;

    if has_bloom {
        let extract = graph.add_node(BloomExtractNode {
            scene_input: self.scene_color_id,
            bloom_output: self.bloom_a_id,
        });
        let bloom_blur = graph.add_node(BloomBlurNode {
            input: self.bloom_a_id,
            output: self.bloom_b_id,
            iterations: 2,
        });
        graph.connect(last_scene_writer, self.scene_color_id, extract);
        graph.connect(extract, self.bloom_a_id, bloom_blur);
        graph.connect(bloom_blur, self.bloom_b_id, /* to composite */);
    }

    let composite = graph.add_node(CompositeNode {
        scene_input: self.scene_color_id,
        bloom_input: self.bloom_a_id,
        output: self.swapchain_id,
    });
    graph.connect(last_scene_writer, self.scene_color_id, composite);

    let accessibility = graph.add_node(AccessibilityNode {
        input: self.swapchain_id,
        output: self.swapchain_id,
        mode: self.color_blind_mode,
    });
    graph.connect(composite, self.swapchain_id, accessibility);

    let present = graph.add_node(PresentNode { surface: &self.surface });
    graph.connect(accessibility, self.swapchain_id, present);

    graph.set_sink(present);

    // ── Compile + Execute ──
    plan = graph.validate_and_compile(&self.resource_registry)
        .expect("Render graph validation failed — this is a bug");
    self.execute_plan(&plan, encoder)
        .expect("Render graph execution failed — this is a bug");
}
```

**Deliverable:** `end_frame()` is fully graph-driven. Default execution order is identical to current hardcoded sequence. Dead-node elimination skips glass blur when no glass is present. Conditional bloom. The planner inserts correct barriers. 597+ tests pass. Zero behavioral change.

---

## Phase 3 — Image Pyramid + Shared Blur Infrastructure

**Goal:** One blur pyramid serves glass, bloom, and future effects. Activate the Kawase shader. Delete duplicate blur systems.

### 3.1 ImagePyramid as Persistent Resource

```rust
pub struct ImagePyramid {
    pub mips:      Vec<ResourceId>,  // [full_res, 1/2, 1/4, 1/8, 1/16]
    pub luminance: ResourceId,       // per-mip luminance (for bloom)
}

impl ImagePyramid {
    pub fn build_node(&self) -> ImagePyramidBuildNode;
    pub fn sample_at_radius(&self, radius: f32) -> ResourceId {
        let mip = (radius / 8.0).min(self.mips.len() as f32 - 1.0) as usize;
        self.mips[mip]
    }
}
```

### 3.2 ImagePyramidBuildNode

Replaces `BackdropCopyNode + BackdropBlurNode` with a single node that generates all mip levels via Kawase passes:

```rust
impl KvasirNode for ImagePyramidBuildNode {
    fn execute(&self, ctx: &mut ExecutionContext, registry: &mut ResourceRegistry) -> Result<(), KvasirError> {
        // Downsample chain: full_res → 1/2 → 1/4 → 1/8 → 1/16
        for mip in 1..self.mip_count {
            ctx.begin_render_pass()
                .set_pipeline(&self.down_pipeline)
                .set_bind_group(0, &self.source_mip_views[mip - 1])
                .draw(0..3);
        }
        // Upsample chain: 1/16 → 1/8 → 1/4 → 1/2 → full_res (into full_res output)
        for mip in (1..self.mip_count).rev() {
            ctx.begin_render_pass()
                .set_pipeline(&self.up_pipeline)
                .set_bind_group(0, &self.source_mip_views[mip])
                .set_bind_group(1, &self.source_mip_views[mip - 1]) // for accumulation
                .draw(0..3);
            // wgpu auto-barrier between passes in same encoder
        }
        Ok(())
    }
}
```

### 3.3 Delete Duplicate Blur Systems (Phase B4 from original plan)

After ImagePyramid is active:
- Delete `blur_h_pipeline`, `blur_v_pipeline` from SurtrRenderer
- Delete `bloom.wgsl` `fs_blur_h`, `fs_blur_v` functions
- Delete `backdrop_blur_node` standalone — replaced by `ImagePyramidBuildNode`
- GlassNode reads directly from `pyramid.sample_at_radius(blur_radius)`
- BloomExtractNode reads from `pyramid.luminance`
- **One blur system. One shader. Every consumer shares it.**

### 3.4 Glass Fresnel Fix (P0-9)

Fix the glass alpha calculation in shapes.wgsl mode 7:

```wgsl
// Before (alpha too low — 1-3%):
let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.0) * 0.02;

// After (proper glass-like alpha):
let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 3.0) * 0.15 + 0.02;
```

**Deliverable:** One blur pyramid services all consumers. Kawase shader is active. Gaussian blur deleted. Glass alpha correct. 597+ tests pass.

---

## Phase 4 — Material System + Pipeline Specialization

**Goal:** Replace `mode: u32` branching with typed materials and specialized shaders.

### 4.1 MaterialKind in cvkg-core

```rust
// cvkg-core/src/lib.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialKind {
    Opaque,
    Glass { blur_radius: f32 },
    TopUI,
    Emissive { intensity: f32 },
    Gradient,
    Text,
    Vector,
    PBR,
}

pub struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: cvkg_core::Rect,
    pub index_start: u32,
    pub index_count: u32,
    pub material: MaterialKind,  // Was DrawMaterial
}
```

### 4.2 Split Shapes Shader

Instead of one `shapes.wgsl` with 10+ mode branches:

- `material_opaque.wgsl` — modes 0,1,3,4,20,21 (rectangle, neon, rounded, 9-slice, cube)
- `material_glass.wgsl` — mode 7 only (backdrop blur sampling + fresnel)
- `material_gradient.wgsl` — modes 16,17 (linear/radial)
- `material_pbr.wgsl` — modes 14,18 (raymarch, metallic)

Each includes common.wgsl, defines only its entry point.

Pipeline creation:
```rust
self.opaque_pipeline  = create_pipeline(&device, &shader, "vs_main", "fs_opaque");
self.glass_pipeline   = create_pipeline(&device, &shader, "vs_main", "fs_glass");
self.gradient_pipeline = create_pipeline(&device, &shader, "vs_main", "fs_gradient");
```

### 4.3 Material-Sorted Dispatch

In `render_frame()`:
```rust
// Group draw calls by material
let mut by_material: HashMap<MaterialKind, Vec<&DrawCall>> = HashMap::new();
for call in &self.draw_calls {
    by_material.entry(call.material).or_default().push(call);
}

// Render each group with matching pipeline — zero branch divergence within a group
if let Some(calls) = by_material.get(&MaterialKind::Opaque) {
    p.set_pipeline(&self.opaque_pipeline);
    for call in calls { p.draw_indexed(...); }
}
if let Some(calls) = by_material.get(&MaterialKind::Glass { .. }) {
    p.set_pipeline(&self.glass_pipeline);
    p.set_bind_group(1, &self.blur_mip_bind_group, &[]); // bind pyramid
    for call in calls { p.draw_indexed(...); }
}
// etc.
```

**Deliverable:** Typed materials. Fragment shaders are 1/4 the size. No intra-group branch divergence. 597+ tests pass.

---

## Phase 5 — Accessibility Integration

**Goal:** Color blindness simulation as a graph-native final pass, not a disconnected shader file.

### 5.1 AccessibilityNode

Already wired in Phase 2 frame graph. Now it actually works:

```rust
pub struct AccessibilityNode {
    pub input:  ResourceId,
    pub output: ResourceId,
    pub mode:   ColorBlindMode,
    pub intensity: f32,  // 0.0 = off, 1.0 = full simulation
}

impl KvasirNode for AccessibilityNode {
    fn execute(&self, ctx: &mut ExecutionContext, registry: &mut ResourceRegistry) -> Result<(), KvasirError> {
        if self.intensity < 0.01 { return Ok(()); } // no-op fast path
        let uniforms = ColorBlindUniforms { mode: self.mode, intensity: self.intensity };
        ctx.write_buffer(&self.uniforms_buf, 0, bytemuck::bytes_of(&uniforms));
        ctx.begin_render_pass()
            .set_pipeline(&ctx.color_blind_pipeline)
            .set_bind_group(0, &ctx.swapchain_texture_bind_group)
            .set_bind_group(1, &self.uniforms_buf_bind_group)
            .draw(0..3);
        Ok(())
    }
}
```

### 5.2 Renderer Config

Add to SurtrRenderer:
```rust
pub color_blind_mode: ColorBlindMode,  // default: Normal (no-op)
pub color_blind_intensity: f32,         // default: 1.0
```

**Deliverable:** Accessibility is a graph node that runs after composite, before present. Normal mode is a no-op. 597+ tests pass.

---

## Phase 6 — Performance Optimizations

### 6.1 Instanced Rendering

Replace 4x per-vertex transform duplication with instance buffer:

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    translation: [f32; 2],
    scale:       [f32; 2],
    rotation:    f32,
    tex_index:   u32,
}
```

Vertex buffer: 4 positions for unit quad (reused). Instance buffer: one entry per draw call.
`draw_indexed(indices, instance_range)` instead of per-vertex duplication.

### 6.2 Half-Resolution Bloom

Extract + blur bloom at 1/2 resolution, composite at full:
```rust
let bloom_width = config.width / 2;
let bloom_height = config.height / 2;
// 4x pixel cost reduction for bloom
```

### 6.3 Adaptive Quality

```rust
let blur_iterations = match self.telemetry.current_fps {
    fps if fps < 30.0 => 2,
    fps if fps < 60.0 => 4,
    _ => 6,
};
```

---

## Non-Goals (From Audit Section 13.5)

- **Do NOT rewrite the renderer.** Architecture problems, not implementation problems.
- **Do NOT rewrite shaders.** Shader issues are architectural integration issues.
- **Do NOT replace the atlas.** The atlas is not a bottleneck.
- **Do NOT replace wgpu.** No evidence wgpu is causing issues.
- **Do NOT rewrite glass.** The glass shader is ahead of the renderer architecture — just connect it correctly.

---

## Success Criteria

1. All P0 bugs fixed (glass-blur dependency, fullscreen draws, bloom texture separation, etc.)
2. `end_frame()` is graph-derived, not hardcoded
3. Glass samples blur data that was generated THIS frame, not stale data
4. One blur pyramid shared by all consumers
5. Pipeline specialization eliminates intra-group branch divergence
6. Accessibility runs as a graph-native pass after composite
7. Zero dead code, stubs, TODOs, FIXMEs, placeholders
8. All existing tests pass
9. Render output verified end-to-end via headless tests

# Kvasir Graph Architecture — Feasibility Assessment & Implementation Plan
**Source:** Surtr-Arch-Review.md, Section 14  
**Subject:** Renderer 3.0 — Unified Visual Computation Graph  
**Assessment:** Feasibility analysis followed by phased implementation plan

---

## Feasibility Verdict: Yes — With Significant Caveats

The Kvasir Graph architecture as described is **technically achievable in Rust with wgpu** and represents a well-understood class of systems. Frame graphs, render dependency graphs, and node-based execution models are production-proven in Frostbite (EA), Filament (Google), FrameGraph (Destiny), and Bevy's render graph. The specific combination CVKG is attempting — unified graph covering UI, 2D vector, 3D, image processing, accessibility, and AI-assisted generation — is more ambitious in scope than any of those precedents, but not architecturally impossible.

What makes it achievable is that Rust's type system is unusually well-suited to this architecture. Ownership semantics naturally enforce the resource producer/consumer model. Trait objects (`dyn KvasirNode`) with compile-time dispatch where possible give you the node abstraction without the overhead of a scripting runtime. wgpu's explicit resource model maps cleanly onto a graph-managed resource lifetime system.

**The hard problems are not whether this can be built. They are:**

1. The transition from the current immediate-mode renderer without breaking the existing application surface
2. The execution planner (dependency resolution, scheduling, barrier insertion) is non-trivial compiler-grade work
3. The material graph compiler — turning a subgraph into a WGSL shader — is a mini shader compiler
4. The blur pyramid as shared infrastructure requires every consumer to agree on its format, resolution, and lifetime model
5. The accessibility service layer requires a formal semantic model that doesn't exist in the current codebase

Each of these is a multi-sprint engineering effort. The plan below treats them as first-class work items rather than implementation details.

---

## What Already Exists That Helps

The current codebase has more scaffolding toward this architecture than it may appear:

- `cvkg-flow` (`FlowGraph`, `FlowNode`, `FlowEdge`) is the node/edge data model — it needs to become the foundation of the Kvasir graph topology layer, not a separate UI component
- `cvkg-scene` provides the retained scene graph and AABB hierarchy — the `SceneGraph` becomes one feed into the Kvasir resource model
- `cvkg-vdom` provides the diffing model — Kvasir's dirty-tracking and incremental re-execution builds on this
- `cvkg-anim`'s RK4 spring solver and `cvkg-physics` can become `AnimationNode` and `PhysicsNode` implementations directly
- The existing shader infrastructure (Surtr pipeline, Muspelheim passes) becomes the execution backend that the Kvasir planner emits to — it does not need to be rewritten, it needs to be driven by the graph rather than hardcoded

---

## What Doesn't Exist and Must Be Built from Scratch

- The `KvasirGraph` data structure (DAG with typed resource edges)
- The `ResourceRegistry` (virtual resource space with lifetime tracking)
- The `ExecutionPlanner` (topological sort, barrier insertion, pass merging)
- The `MaterialCompiler` (subgraph → WGSL)
- The `ImagePyramid` shared infrastructure
- The `AccessibilityServiceLayer` (graph-native, not post-process)
- The AI generation API surface (graph input, not draw call input)

---

## Implementation Plan

The plan is structured in five phases. Each phase is independently shippable and leaves the renderer in a better state than it found it. Phases 3, 4, and 5 can be designed and prototyped in parallel with Phase 2, but they cannot ship until Phase 2's graph infrastructure exists — `ImagePyramidBuildNode`, `BifrostMaterialNode`, and `AccessibilityService` all require a working `KvasirGraph` and `ExecutionPlanner` to execute within. Use the Phase 2 window to spec and prototype the later phases, not to build them to completion.

---

## Phase 0 — Fix the Blocking Bugs First (Sprint 1–2)

**Do not begin Kvasir work until these are resolved. They will corrupt any graph built on top of them.**

The shader audit identified critical correctness failures that will produce wrong output regardless of how well the graph is structured:

| Bug | Fix |
|---|---|
| `fs_copy` reads Mega-Atlas instead of scene texture | Change to sample `t_env` |
| `stroke_path` DrawCall uses vertex cursor as index cursor | Capture `base_index = self.indices.len()` before tessellation |
| Parallel rayon passes share `ctx_scene_texture` without barrier | Encode Glass and UI passes sequentially |
| Bloom extract overwrites backdrop blur texture | Allocate separate `bloom_tex_a/b` |
| `vs_fullscreen` draws 6 vertices — second triangle is degenerate | Change all fullscreen draws to `0..3` |
| Glyph atlas fallback writes to `(0,0)` on full atlas | Return early with error log |
| SVG tessellation `.unwrap()` panics on malformed paths | Propagate as `Result`, skip degenerate paths |
| Clip SDF uses `clip_position.xy` as NDC (already window pixels) | Remove the `* 0.5 + 0.5 * resolution` transform |
| Bifrost glass alpha is 1–3% | Fix the fresnel alpha calculation |

These bugs mean the renderer's most visually critical features (glass panels, text, clip rects, bloom) are currently producing incorrect output. The Kvasir graph must be built on a correct foundation.

---

## Phase 1 — Resource Graph Foundation (Sprint 3–6)

**Goal:** Introduce the `ResourceId` / `ResourceRegistry` layer. Make physical GPU resources (textures, buffers, atlases) into named virtual resources. Nothing else changes yet.

### 1.1 Define the Resource Type System

```rust
// cvkg-render-gpu/src/kvasir/resource.rs

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(u64);

pub enum ResourceKind {
    Image { width: u32, height: u32, format: wgpu::TextureFormat },
    Geometry { vertex_count: u32, index_count: u32 },
    VectorPath,
    Material,
    Animation,
    Accessibility,
    Scene,
}

pub struct ResourceDescriptor {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub label: &'static str,
    pub lifetime: ResourceLifetime,
}

pub enum ResourceLifetime {
    Frame,         // allocation AND content are transient — freed at end of frame
    FrameContent,  // allocation persists across frames, content rebuilt each frame
                   // use this for blur/pyramid textures: no realloc cost, fresh pixels
    Persistent,    // lives across frames (atlas, scene geometry, loaded assets)
    Streaming,     // loaded asynchronously, may not be ready this frame
}
```

### 1.2 Build the ResourceRegistry

```rust
pub struct ResourceRegistry {
    descriptors: HashMap<ResourceId, ResourceDescriptor>,
    gpu_images:  HashMap<ResourceId, wgpu::Texture>,
    gpu_buffers: HashMap<ResourceId, wgpu::Buffer>,
    lru:         LruCache<ResourceId, ()>,
    next_id:     AtomicU64,
}

impl ResourceRegistry {
    pub fn register(&mut self, desc: ResourceDescriptor) -> ResourceId
    pub fn get_image(&self, id: ResourceId) -> Option<&wgpu::Texture>
    pub fn get_or_create_image(&mut self, desc: &ResourceDescriptor, device: &wgpu::Device) -> ResourceId
    pub fn evict_frame_resources(&mut self)  // called at end of frame
    pub fn reclaim(&mut self, budget: u64)   // replaces current reclaim_vram()
}
```

### 1.3 Migrate Existing Physical Resources Into the Registry

Name and register every texture the current renderer manages:

```rust
// During forge():
let scene_tex_id    = registry.register(ResourceDescriptor { label: "scene_color", ... });
let blur_tex_a_id   = registry.register(ResourceDescriptor { label: "blur_a", ... });
let blur_tex_b_id   = registry.register(ResourceDescriptor { label: "blur_b", ... });
let bloom_tex_a_id  = registry.register(ResourceDescriptor { label: "bloom_a", ... });
let bloom_tex_b_id  = registry.register(ResourceDescriptor { label: "bloom_b", ... });
let depth_tex_id    = registry.register(ResourceDescriptor { label: "depth", ... });
let mega_atlas_id   = registry.register(ResourceDescriptor { label: "mega_atlas", ... });
```

The renderer still manages these directly for now — the registry is a naming and tracking layer, not yet a lifetime manager. That comes in Phase 2.

### 1.4 Deliverable

A `ResourceRegistry` that names every GPU resource. The renderer behavior is unchanged. `reclaim_vram()` uses the registry's LRU instead of its own internal logic. The glyph atlas fallback bug (Phase 0) is fixed as part of this work.

---

## Phase 2 — Kvasir Graph Core + Execution Planner (Sprint 7–14)

**Goal:** Build the graph data structure, the dependency model, and the execution planner. This is the largest single piece of engineering in the entire plan.

### 2.1 The KvasirNode Trait

```rust
// cvkg-render-gpu/src/kvasir/node.rs

pub trait KvasirNode: Send + Sync {
    fn label(&self) -> &'static str;
    fn inputs(&self)  -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];

    fn execute(
        &self,
        ctx: &mut ExecutionContext,
        registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError>;

    // Optional: hint to the planner about execution preference
    fn execution_hint(&self) -> ExecutionHint {
        ExecutionHint::Raster
    }
}

pub enum ExecutionHint { Raster, Compute, Hybrid }
```

### 2.2 The KvasirGraph Data Structure

```rust
// cvkg-render-gpu/src/kvasir/graph.rs

pub struct KvasirGraph {
    nodes:  SlotMap<NodeKey, Box<dyn KvasirNode>>,
    edges:  Vec<Edge>,        // (producer NodeKey, ResourceId, consumer NodeKey)
    roots:  Vec<NodeKey>,     // nodes with no inputs (scene sources)
    sinks:  Vec<NodeKey>,     // nodes whose output is the final present target
}

impl KvasirGraph {
    pub fn add_node(&mut self, node: impl KvasirNode + 'static) -> NodeKey
    pub fn connect(&mut self, from: NodeKey, resource: ResourceId, to: NodeKey)
    pub fn validate(&self) -> Result<(), Vec<KvasirError>>  // cycle detection, type checking
    pub fn compile(&self) -> Result<ExecutionPlan, KvasirError>
}
```

### 2.3 The Execution Planner

This is the hardest component. It takes the graph and produces an ordered list of GPU operations with correct resource barriers.

```rust
pub struct ExecutionPlanner;

impl ExecutionPlanner {
    pub fn plan(graph: &KvasirGraph, registry: &ResourceRegistry) -> Result<ExecutionPlan, KvasirError> {
        // 1. Topological sort (Kahn's algorithm — panic on cycle)
        // 2. Lifetime analysis — determine when each resource is first written and last read
        // 3. Transient resource allocation — reuse memory for non-overlapping frame resources
        // 4. Barrier insertion — insert wgpu pipeline barriers between passes that share resources
        // 5. Pass merging — identify adjacent raster passes that can share a render pass
        // 6. Emit ExecutionPlan
    }
}

pub struct ExecutionPlan {
    pub ordered_nodes: Vec<NodeKey>,
    pub barriers:      HashMap<(NodeKey, NodeKey), BarrierDescriptor>,
    pub merged_passes: Vec<MergedPassGroup>,
    pub resource_aliases: HashMap<ResourceId, ResourceId>,  // transient aliasing
}
```

**Barrier insertion is where most GPU correctness bugs live.** The current renderer's parallel rayon pass bug (Audit 1.1) is exactly what barriers are designed to prevent. The planner must insert a barrier whenever a resource transitions from being written by one node to being read by the next.

### 2.4 Convert the Existing Passes Into KvasirNodes

Begin the migration by wrapping existing render passes as nodes. The passes do not change internally — they are just given node wrappers that declare their resource I/O:

```rust
pub struct BackgroundNode {
    pub output: ResourceId,  // scene_color
}

pub struct GlassNode {
    pub blur_input:    ResourceId,  // blur_a (backdrop)
    pub scene_input:   ResourceId,  // scene_color
    pub scene_output:  ResourceId,  // scene_color (read-write)
}

pub struct BloomExtractNode {
    pub scene_input:   ResourceId,  // scene_color
    pub bloom_output:  ResourceId,  // bloom_a
}

pub struct GaussianBlurNode {
    pub input:   ResourceId,
    pub output:  ResourceId,
    pub axis:    BlurAxis,
}

pub struct CompositeNode {
    pub scene_input: ResourceId,
    pub bloom_input: ResourceId,
    pub output:      ResourceId,
}
```

Wire them into a graph and run through the planner. The planner emits the same execution order the hardcoded pass sequence produces today — but now it is derived from data, not hardcoded.

### 2.5 Deliverable

The frame rendering loop changes from:

```rust
// Before: hardcoded pass sequence
self.encode_background_pass();
self.encode_blur_passes();
self.encode_glass_pass();
self.encode_ui_pass();
self.encode_bloom_passes();
self.encode_composite_pass();
```

To:

```rust
// After: graph-driven
let plan = self.kvasir_graph.compile()?;
self.execute_plan(&plan, &mut self.resource_registry)?;
```

The output is visually identical. The architecture is now extensible.

### 2.6 Full End-to-End Wiring Example

This is what a complete frame graph construction looks like after Phase 2. Every resource is named. Every dependency is explicit. The planner derives the correct execution order and inserts barriers — nothing is hardcoded.

```rust
fn build_frame_graph(
    registry: &mut ResourceRegistry,
    has_glass: bool,
) -> KvasirGraph {
    let mut graph = KvasirGraph::new();

    // --- Declare frame-lifetime resources ---
    let scene_color = registry.get_or_create_image(&ResourceDescriptor {
        label: "scene_color",
        kind:  ResourceKind::Image { width: 1920, height: 1080, format: wgpu::TextureFormat::Rgba16Float },
        lifetime: ResourceLifetime::Frame,
        ..Default::default()
    });

    // Define the descriptor once — blur_b is identical except the label.
    // FrameContent means the GPU allocation persists across frames; only the pixel
    // data is considered transient. evict_frame_resources() skips FrameContent textures.
    let fullres_desc = ResourceDescriptor {
        label: "blur_a",
        kind:  ResourceKind::Image { width: 1920, height: 1080, format: wgpu::TextureFormat::Rgba16Float },
        lifetime: ResourceLifetime::FrameContent,
        ..Default::default()
    };
    let blur_a = registry.get_or_create_image(&fullres_desc);
    let blur_b = registry.get_or_create_image(&ResourceDescriptor { label: "blur_b", ..fullres_desc });

    let bloom_a = registry.get_or_create_image(&ResourceDescriptor {
        label: "bloom_a",
        kind:  ResourceKind::Image { width: 960, height: 540, format: wgpu::TextureFormat::Rgba16Float },
        lifetime: ResourceLifetime::Frame,
        ..Default::default()
    });

    // --- Add nodes ---
    let bg_node = graph.add_node(BackgroundNode {
        output: scene_color,
    });

    // scene_input and scene_output are the same ResourceId — this is intentional.
    // Declaring both is what tells the planner this node reads-then-writes scene_color,
    // so it inserts a barrier after bg_node before allowing ui_node to execute.
    let ui_node = graph.add_node(UiGeometryNode {
        scene_input:  scene_color,   // reads what bg_node wrote
        scene_output: scene_color,   // overdraw composites UI geometry on top
    });

    // Glass pass is conditional — added only when glass panels exist this frame.
    // The planner only schedules nodes reachable from the sink.
    if has_glass {
        let copy_node = graph.add_node(SceneCopyNode {
            input:  scene_color,
            output: blur_a,
        });

        let blur_h = graph.add_node(GaussianBlurNode {
            input:  blur_a,
            output: blur_b,
            axis:   BlurAxis::Horizontal,
        });

        let blur_v = graph.add_node(GaussianBlurNode {
            input:  blur_b,
            output: blur_a,   // ping-pong back
            axis:   BlurAxis::Vertical,
        });

        let glass_node = graph.add_node(GlassNode {
            blur_input:   blur_a,
            scene_input:  scene_color,
            scene_output: scene_color,
        });

        // Explicit connections drive the planner's dependency analysis.
        // Without connect(), the planner cannot infer ordering from struct fields alone.
        graph.connect(bg_node,   scene_color, ui_node);
        graph.connect(ui_node,   scene_color, copy_node);
        graph.connect(copy_node, blur_a,      blur_h);
        graph.connect(blur_h,    blur_b,      blur_v);
        graph.connect(blur_v,    blur_a,      glass_node);
        graph.connect(ui_node,   scene_color, glass_node);
    } else {
        graph.connect(bg_node, scene_color, ui_node);
    }

    let bloom_extract = graph.add_node(BloomExtractNode {
        scene_input:  scene_color,
        bloom_output: bloom_a,
    });

    let composite = graph.add_node(CompositeNode {
        scene_input: scene_color,
        bloom_input: bloom_a,
        output:      scene_color,   // final output overwrites scene_color for present
    });

    graph.connect(ui_node,       scene_color, bloom_extract);
    graph.connect(bloom_extract, bloom_a,     composite);
    // Connect the last scene-writer to composite, not ui_node directly.
    // When glass ran, glass_node is the last writer. When it didn't, ui_node is.
    // This ensures the planner mandates glass completes before composite reads scene_color.
    if has_glass {
        graph.connect(glass_node,  scene_color, composite);
    } else {
        graph.connect(ui_node,     scene_color, composite);
    }

    graph.set_sink(composite);
    graph
}
```

Notice that `has_glass` makes the blur nodes conditionally absent. The planner performs dead-node elimination — if no glass node exists, the copy and blur nodes are never scheduled even if they were added. This is structurally impossible in the hardcoded pass sequence, where blur passes run every frame regardless.

---

## Phase 3 — Image Pyramid + Shared Blur Infrastructure (Sprint 15–18)

**Goal:** Replace the current ping-pong blur textures with the Kvasir Image Pyramid — a shared, multi-consumer blur and image analysis structure.

### 3.1 ImagePyramid as a Persistent Resource

```rust
pub struct ImagePyramid {
    pub mips:         Vec<ResourceId>,   // downsampled scene at each level
    pub luminance:    ResourceId,        // per-mip luminance buffer
    pub motion:       Option<ResourceId>,// optional motion vectors
    pub depth:        Option<ResourceId>,// optional depth pyramid
    pub focus_data:   Option<ResourceId>,// for focus ring effects
}

impl ImagePyramid {
    pub fn build_node(&self) -> impl KvasirNode  // produces all pyramid levels
    pub fn sample_at_blur_radius(&self, radius: f32) -> ResourceId  // returns appropriate mip
}
```

### 3.2 Activate the Kawase Pyramid Shader

`blur_pyramid.wgsl` exists but is dead code (Shader Audit S-1). It needs:
- The `@Override` line replaced entirely — `@Override` is a WGSL pipeline-override keyword that is syntactically valid but semantically wrong in a binding position. The full line must become `@group(0) @binding(0) var<uniform> blur: BlurUniforms;`
- A Rust pipeline created for `fs_kawase_down` and `fs_kawase_up`
- `KawaseDownNode` and `KawaseUpNode` implementations
- The Gaussian blur in `bloom.wgsl` retained for bloom (narrower kernel needed for bloom extraction) but replaced with Kawase for the backdrop blur (wider, more natural blur)

### 3.3 Wire Glass and Bloom as Pyramid Consumers

```rust
// Glass reads from the pyramid at its configured blur strength
let backdrop_mip = pyramid.sample_at_blur_radius(theme.glass_blur_strength);
let glass_node = GlassNode {
    blur_input:   backdrop_mip,     // from pyramid, not from dedicated blur pass
    scene_input:  scene_color_id,
    scene_output: scene_color_id,
};

// Bloom reads from the pyramid's high-luminance mip
let bloom_node = BloomExtractNode {
    scene_input:  pyramid.luminance,
    bloom_output: bloom_tex_a_id,
};
```

### 3.4 Multi-Consumer Pyramid Example

The critical advantage of the pyramid over dedicated per-effect blur passes is that multiple consumers can read from it simultaneously without paying redundant GPU cost. Here is a concrete example with three simultaneous consumers — glass panels at two different blur strengths and a focus ring effect — none of which trigger additional blur passes:

```rust
fn build_multi_effect_graph(
    registry:   &mut ResourceRegistry,
    pyramid:    &ImagePyramid,
    scene_color: ResourceId,
) -> KvasirGraph {
    let mut graph = KvasirGraph::new();

    // Build the pyramid once per frame. All mips are computed in a single downsampling chain.
    let pyramid_node = graph.add_node(ImagePyramidBuildNode {
        source:  scene_color,
        outputs: pyramid.mips.clone(),
    });

    // Consumer A: Heavy glass panel (modal dialog) — needs strong blur, reads mip 3
    let modal_blur_resource = pyramid.sample_at_blur_radius(24.0);  // mip 3
    let modal_glass = graph.add_node(GlassNode {
        blur_input:   modal_blur_resource,
        scene_input:  scene_color,
        scene_output: scene_color,
        label:        "modal_glass",
    });

    // Consumer B: Subtle glass panel (tooltip) — light blur, reads mip 1
    let tooltip_blur_resource = pyramid.sample_at_blur_radius(4.0);  // mip 1
    let tooltip_glass = graph.add_node(GlassNode {
        blur_input:   tooltip_blur_resource,
        scene_input:  scene_color,
        scene_output: scene_color,
        label:        "tooltip_glass",
    });

    // Consumer C: Focus ring magnification — reads luminance channel from pyramid
    let focus_node = graph.add_node(FocusEnhancementNode {
        luminance_input: pyramid.luminance,
        scene_output:    scene_color,
        target_region:   focused_widget_rect,
    });

    // All three consumers depend on the pyramid but not on each other.
    // The planner can schedule them in any order after pyramid_node completes.
    // On a GPU with async compute, it may overlap them.
    graph.connect(pyramid_node, modal_blur_resource,   modal_glass);
    graph.connect(pyramid_node, tooltip_blur_resource, tooltip_glass);
    graph.connect(pyramid_node, pyramid.luminance,     focus_node);

    // In the hardcoded pass sequence this scenario would require:
    //   3 separate blur pass chains (one per consumer at its own strength)
    //   = 3x the blur GPU cost
    // With the pyramid, all three consumers share one downsampling chain.
    // GPU cost: one pyramid build regardless of consumer count.

    graph
}
```

The contrast with the old model is explicit in the comment: three glass consumers at different blur strengths would have required three independent blur chains in the hardcoded sequence. The pyramid pays the blur cost once and amortizes it across all consumers in the frame.

### 3.5 Deliverable

- The backdrop blur and bloom are driven by the same pyramid rather than competing for the same two ping-pong textures (fixing Audit 1.2 at the architectural level)
- Multiple glass panels at different blur strengths can each read from the appropriate mip without additional blur passes
- The pyramid textures use `ResourceLifetime::FrameContent` — allocations persist across frames, content is rebuilt each frame from the current scene

---

## Phase 4 — Material Graph + Shader Compiler (Sprint 19–28)

**Goal:** Make materials composable subgraphs rather than hardcoded shader mode integers.

### 4.1 Material Graph Data Model

```rust
pub struct MaterialGraph {
    nodes: SlotMap<MatNodeKey, Box<dyn MaterialNode>>,
    edges: Vec<MatEdge>,
    output: MatNodeKey,  // the final color output node
}

pub trait MaterialNode {
    fn inputs(&self)  -> &[MatSocket];
    fn outputs(&self) -> &[MatSocket];
    fn wgsl_function(&self) -> &str;  // the WGSL snippet this node contributes
}
```

Built-in material nodes:

```rust
pub struct NoiseNode     { pub scale: f32, pub octaves: u32 }
pub struct FresnelNode   { pub ior: f32 }
pub struct RefractionNode { pub strength: f32 }
pub struct GradientNode  { pub start: Vec4, pub end: Vec4 }
pub struct BifrostMaterialNode { pub blur_strength: f32 }  // named to avoid collision with render-graph GlassNode
pub struct GlowNode      { pub radius: f32, pub intensity: f32 }
pub struct CompositeNode { pub blend_mode: BlendMode }
```

### 4.2 Material Compiler

The material compiler takes a `MaterialGraph` and emits a WGSL fragment shader function:

```rust
pub struct MaterialCompiler;

impl MaterialCompiler {
    pub fn compile(graph: &MaterialGraph) -> Result<CompiledMaterial, CompileError> {
        // 1. Topological sort of material nodes
        // 2. Emit WGSL function body by visiting nodes in order
        // 3. Connect node outputs to next node's inputs via named WGSL variables
        // 4. Wrap in a standard fragment shader function signature
        // 5. Create wgpu pipeline from the emitted WGSL
        // Returns: CompiledMaterial { pipeline, bind_group_layout, wgsl_source }
    }
}
```

This is the most technically complex component. Precedents exist: Bevy's material system, Godot's visual shader editor, Unreal's Material Blueprint. The WGSL generation is string templating with topological ordering — it is not a full compiler, but it is not trivial either.

### 4.3 Replace Shader Mode Integers with Material Graphs

The current `in.mode` switch in `fs_main` (`shapes.wgsl`) with 20+ branches gets replaced. Each mode becomes a named `MaterialGraph` with built-in nodes:

```rust
// Before:
DrawCall { mode: 3u, ... }  // rounded rect, hardcoded in shader

// After:
// For simple single-node materials, MaterialGraph exposes a convenience compile()
// that calls MaterialCompiler::compile(&self) internally. The explicit form is
// MaterialCompiler::compile(&mat) — both are valid, the latter is used in complex examples.
let mut mat = MaterialGraph::new();
mat.add(RoundedRectNode { radius: 8.0 });
mat.add(ColorNode { color: theme.primary });
mat.connect_to_output(MatSocket::Color);
let compiled = MaterialCompiler::compile(&mat)?;
DrawCall { material: compiled.id(), ... }
```

Built-in materials are pre-compiled at renderer startup. User materials are compiled on first use and cached by hash.

### 4.4 Composing a Novel Effect — Right Way vs. Wrong Way

This example exists specifically to illustrate the difference between using the material graph correctly and misusing it in the way someone might if they cargo-culted the simpler example above.

**Scenario:** A neon-glowing rounded button with animated Fresnel edge shimmer and a noise-based inner turbulence — the kind of compound effect currently requiring a new `mode` integer and 40 lines of new WGSL.

**Wrong approach — treating the graph like a flat list of settings:**

```rust
// This looks like it compiles but produces incorrect output.
// Each .add() overwrites the previous node's color output rather than composing with it.
// The compiler has no way to know you meant "layer these" vs "replace with these."
let mut mat = MaterialGraph::new();
mat.add(RoundedRectNode { radius: 12.0 });
mat.add(NoiseNode { scale: 6.0, octaves: 4 });     // ← registered but not connected
mat.add(FresnelNode { ior: 1.45 });                // ← registered but not connected
mat.add(GlowNode { radius: 8.0, intensity: 1.2 }); // ← registered but not connected
let compiled = MaterialCompiler::compile(&mat)?;   // ← compiles only the last-added node
// Result: only GlowNode output is visible. The rect, noise, and fresnel are discarded.
```

**Right approach — explicit socket connections between nodes:**

```rust
// Each node's output is connected to a named input socket on the next node.
// The compiler follows the connection graph, not insertion order.
let mut mat = MaterialGraph::new();

// Step 1: Shape — establishes the SDF mask that clips everything downstream
let shape   = mat.add(RoundedRectNode { radius: 12.0 });

// Step 2: Base color with noise turbulence layered on top
let noise   = mat.add(NoiseNode { scale: 6.0, octaves: 4 });
let base    = mat.add(ColorNode { color: theme.primary_neon });
let turbulence = mat.add(CompositeNode {
    blend_mode: BlendMode::Add,
    opacity:    0.15,
});
mat.connect(base,  MatSocket::Color,  turbulence, MatSocket::Bottom);
mat.connect(noise, MatSocket::Value,  turbulence, MatSocket::Top);

// Step 3: Fresnel edge shimmer layered over the turbulent base
let fresnel = mat.add(FresnelNode { ior: 1.45 });
let shimmer = mat.add(CompositeNode {
    blend_mode: BlendMode::Screen,
    opacity:    0.4,
});
mat.connect(turbulence, MatSocket::Color, shimmer, MatSocket::Bottom);
mat.connect(fresnel,    MatSocket::Value, shimmer, MatSocket::Top);

// Step 4: Glow applied as an additive layer outside the shape boundary
let glow = mat.add(GlowNode { radius: 8.0, intensity: 1.2 });
mat.connect(shimmer, MatSocket::Color, glow, MatSocket::Input);

// Step 5: Shape mask clips the interior — glow extends outside it intentionally
let output = mat.add(ShapeMaskNode { invert_exterior: false });
mat.connect(shape, MatSocket::Mask,  output, MatSocket::Mask);
mat.connect(glow,  MatSocket::Color, output, MatSocket::Color);

mat.set_output(output);
let compiled = mat.compile()?;

// The compiler emits a single WGSL function that evaluates all five stages in order.
// No new shader files. No new mode integer. No recompile of the renderer.
DrawCall { material: compiled.id(), bounds: button_rect, .. }
```

The key insight is that `.add()` registers nodes but does not wire them. `.connect()` establishes the dataflow. The compiler only sees what is connected to the output node — unconnected nodes are dead code and are pruned before WGSL emission.

### 4.4 Deliverable

- The 20+ branch `if/else` mode switch in the fragment shader is replaced with dynamically composed materials
- New visual effects can be created without modifying `shapes.wgsl`
- The material graph is the foundation for the future material editor in `cvkg-components`
- AI-generated materials can be expressed as material graphs without touching shader code

---

## Phase 5 — Accessibility Service Layer + AI API Surface (Sprint 29–36)

**Goal:** Make accessibility a first-class graph service, and expose the graph as the AI-generation API.

### 5.1 Accessibility as Graph Infrastructure

Current color blindness simulation is an empty file (`color_blind.wgsl` contains only comments). The Daltonization matrices referenced in the README don't exist in the shader or the Rust code.

Build it properly as a graph service:

```rust
pub struct AccessibilityService {
    pub active_transforms: Vec<AccessibilityTransform>,
}

pub enum AccessibilityTransform {
    ColorBlind(ColorBlindType),   // Protanopia, Deuteranopia, Tritanopia
    HighContrast(f32),
    MotionReduction,
    Magnification { region: Rect, scale: f32 },
    FocusEnhancement { target: ResourceId },
}

impl AccessibilityService {
    // Returns a list of KvasirNodes to inject into the graph
    // Called during graph compilation, not as post-process
    pub fn graph_nodes(&self) -> Vec<Box<dyn KvasirNode>>
}
```

The Brettel/Viénot matrices get implemented as a `ColorTransformNode` using a 3x3 matrix applied in linear light space (not gamma-compressed sRGB, which is the common error in daltonization implementations).

### 5.2 AI Generation API Surface

The graph-first architecture enables a clean API for AI-generated content:

```rust
pub trait KvasirGenerator: Send + Sync {
    // Background generator thread produces materials/graphs and delivers them
    // via an internal channel. Returns Ok(()) when generation completed, Err on failure.
    // The channel (not the return value) is how output reaches the renderer thread.
    fn generate(&self) -> Result<(), GeneratorError>;
}

// Example implementations:
pub struct SvgGeneratorNode      { pub prompt: String }     // AI generates SVG → VectorResource
pub struct MaterialGeneratorNode { pub description: String } // AI generates MaterialGraph
pub struct LayoutGeneratorNode   { pub constraints: LayoutSpec }
pub struct SceneGeneratorNode    { pub description: String }
```

The key architectural point the document makes is correct: **graph APIs instead of pass APIs** is what enables AI generation. An AI system that must emit valid wgpu draw calls is integrating at the wrong layer. An AI system that emits a `KvasirGraph` fragment with resource declarations and node connections is integrating at exactly the right layer. The runtime handles all the GPU complexity.

### 5.3 Concrete AI Generator Example With Validation

This example shows a complete AI material generator — taking a text description, calling an inference endpoint, receiving a material graph description as structured JSON, validating it, and integrating it into the live renderer without a recompile. The validation step is non-negotiable: untrusted graph fragments from any external source (AI or otherwise) must pass the graph validator before execution.

```rust
// The generator runs on a background thread. The renderer receives
// a validated KvasirGraph fragment via a channel and merges it next frame.

pub struct AiMaterialGenerator {
    endpoint: String,
    tx:       Sender<Result<CompiledMaterial, GeneratorError>>,
}

impl KvasirGenerator for AiMaterialGenerator {
    fn generate(&self) -> Result<KvasirGraph, GeneratorError> {
        // Step 1: Call inference endpoint, receive structured JSON
        // The AI is prompted to emit a MaterialGraph description, not WGSL.
        // This is the correct layer boundary — AI describes intent, not GPU instructions.
        let response: MaterialGraphSpec = self.call_inference()?;

        // Step 2: Deserialize into a MaterialGraph
        // The spec uses named node types that map to built-in MaterialNode implementations.
        // Unknown node types are rejected here — not at GPU execution time.
        let mut mat = MaterialGraph::new();
        let mut node_map: HashMap<String, MatNodeKey> = HashMap::new();

        for node_spec in &response.nodes {
            let node: Box<dyn MaterialNode> = match node_spec.kind.as_str() {
                "RoundedRect" => Box::new(RoundedRectNode {
                    radius: node_spec.params.get_f32("radius")?,
                }),
                "Noise"       => Box::new(NoiseNode {
                    scale:   node_spec.params.get_f32("scale")?,
                    octaves: node_spec.params.get_u32("octaves")?,
                }),
                "Fresnel"     => Box::new(FresnelNode {
                    ior: node_spec.params.get_f32("ior")?,
                }),
                "Composite"   => Box::new(CompositeNode {
                    blend_mode: BlendMode::from_str(&node_spec.params.get_str("blend")?)?,
                    opacity:    node_spec.params.get_f32("opacity")?,
                }),
                unknown => {
                    // Reject unknown node types outright.
                    // An AI that hallucinates a "DirectGpuMemoryWrite" node
                    // gets rejected here, not executed.
                    return Err(GeneratorError::UnknownNodeType(unknown.to_string()));
                }
            };
            let key = mat.add_boxed(node);
            node_map.insert(node_spec.id.clone(), key);
        }

        // Step 3: Wire connections as declared by the AI
        for edge in &response.edges {
            let from = node_map.get(&edge.from_node)
                .ok_or(GeneratorError::UnknownNode(edge.from_node.clone()))?;
            let to = node_map.get(&edge.to_node)
                .ok_or(GeneratorError::UnknownNode(edge.to_node.clone()))?;
            mat.connect(
                *from, MatSocket::from_str(&edge.from_socket)?,
                *to,   MatSocket::from_str(&edge.to_socket)?,
            );
        }

        // Step 4: Validate the graph before compilation
        // This catches cycles, disconnected outputs, type mismatches, and
        // graphs that declare more than MAX_MATERIAL_NODES nodes (DoS guard).
        mat.validate(MaterialValidationConfig {
            max_nodes:          32,
            allow_cycles:       false,
            require_output:     true,
            allowed_node_types: &BUILTIN_NODE_TYPES,  // whitelist, not blocklist
        })?;

        // Step 5: Compile to WGSL and create the wgpu pipeline
        // This is the only point where GPU resources are allocated.
        // Validation has already confirmed the graph is structurally correct.
        let compiled = mat.compile()?;

        // Step 6: Send compiled material to renderer thread for next-frame integration.
        // The channel is the integration mechanism — the trait return value is not used
        // for delivery here. KvasirGenerator::generate() returns Ok(()) to signal that
        // generation completed without error; the compiled material travels via the channel.
        // This decouples the background generator thread from the render thread's frame timing.
        self.tx.send(Ok(compiled)).map_err(|_| GeneratorError::ChannelClosed)?;

        Ok(())
    }
}

// On the renderer side, integrating the AI-generated material is three lines:
fn begin_frame(&mut self) {
    // Drain any materials that arrived from generators since last frame
    while let Ok(result) = self.material_rx.try_recv() {
        match result {
            Ok(compiled) => self.resource_registry.register_material(compiled),
            Err(e)       => log::warn!("AI material generator failed: {e}"),
        }
    }
    // ... rest of begin_frame unchanged
}
```

The critical design constraint is the whitelist at Step 4: `allowed_node_types: &BUILTIN_NODE_TYPES`. The validator rejects any node type not in the whitelist, regardless of what the AI returns. This means an AI system cannot introduce new execution paths into the renderer — it can only combine pre-audited building blocks in novel ways. The AI authors the composition; the engineer controls the vocabulary.

### 5.3 Deliverable

- Color blindness simulation is actually implemented (not an empty file)
- Accessibility transforms are graph nodes that operate on the correct data (linear light, before tonemapping)
- A `KvasirGenerator` trait gives AI systems a stable API that doesn't break with renderer changes
- The `cvkg-cli` can expose graph generation via command-line templates for developer tooling

---

## Phase Summary

| Phase | Name | Sprints | Key Output |
|---|---|---|---|
| 0 | Bug Fixes | 1–2 | Correct renderer output — no new features |
| 1 | Resource Graph Foundation | 3–6 | `ResourceRegistry`, named GPU resources, LRU fix |
| 2 | Kvasir Graph Core + Planner | 7–14 | Graph-driven frame loop, correct barrier insertion |
| 3 | Image Pyramid | 15–18 | Shared blur infrastructure, Kawase activation |
| 4 | Material Graph + Compiler | 19–28 | Composable materials, no more mode integers |
| 5 | Accessibility + AI API | 29–36 | Native a11y, AI generation surface |

---

## Risks and Honest Assessment

**The execution planner is the highest-risk component.** Getting barrier insertion wrong produces GPU validation errors, visual corruption, or hard crashes that are difficult to reproduce because they depend on GPU scheduling. Plan for significant iteration here. Run with `WGPU_BACKEND=vulkan RUST_LOG=wgpu_hal=debug` throughout Phase 2 and treat every validation warning as a blocking bug.

**The material compiler scope can expand unboundedly.** A simple WGSL string assembler gets you 80% of the value. A full dataflow type checker, loop support, and sampling abstraction gets you 100% — and takes 5x longer. Define the MVP as: topologically sorted, no cycles, no loops, flat function composition. Ship that. Iterate.

**The transition period is the architectural danger zone.** During Phases 2 and 3, both the old hardcoded pass sequence and the new graph-driven sequence will exist simultaneously. Establish a feature flag (`KVASIR_GRAPH=1`) early so the graph path can be validated against the hardcoded path frame-by-frame before the hardcoded path is removed.

**`lib.rs` at 5,491 lines must be modularized as part of this work**, not after it. Phase 1 is the natural point to extract `resource.rs`. Phase 2 extracts `kvasir/graph.rs`, `kvasir/planner.rs`, `kvasir/node.rs`. Phase 3 extracts `pyramid.rs`. Phase 4 extracts `material/graph.rs`, `material/compiler.rs`. By the end of Phase 4, `lib.rs` should be under 800 lines — the `SurtrRenderer` struct, `forge()`, `begin_frame()`, `end_frame()`, and the execution entry point.

---

## The Architecture Is Sound

The document's core thesis — that rendering, image processing, accessibility, animation, and AI generation are all the same thing at the graph level — is correct. The systems that have moved to this model (Frostbite, Filament, modern Vulkan renderers) are more maintainable and more extensible than the pass-sequence renderers they replaced. The CVKG codebase has enough existing infrastructure that this is a migration, not a rewrite. The five phases above get there without breaking the application surface at any step.

---

*Implementation Plan v1.0 — Based on Surtr-Arch-Review.md Section 14 and Surtr Renderer Code Audit*

# Kvasir Graph Implementation Plan — Revised Edition

**Source:** Kvasir_Graph_Implementation_Plan_2.md with corrections from kvasir_update.md
**Subject:** Renderer 3.0 — Unified Visual Computation Graph (Corrected Architecture)
**Date:** 2026-06-02

---

## Executive Summary (Revised)

The Kvasir architecture is technically achievable in Rust with wgpu and represents a well-understood class of systems. This revised plan incorporates critique feedback to address

```text
One Universal Graph → Multiple Specialized Graph Domains
```

The correction establishes **five independent but synchronized graphs** under a unified runtime layer, eliminating the monolithic graph anti-pattern identified in the critique.

**Phase 0 remains the priority** — the seven critical blocking bugs must be resolved before any Kvasir work begins.

---

## New Architectural Principles

| Principle | Description |
|-----------|-------------|
| **KVASIR-012** | Kvasir Is A Runtime, Not A Graph |
| **KVASIR-013** | Multiple Specialized Graph Domains |
| **KVASIR-014** | Temporal Graph Is First-Class |
| **KVASIR-015** | Material Graph Compiles Through IR |
| **KVASIR-016** | Accessibility Split Into Visual and Semantic |
| **KVASIR-017** | Resource Virtualization Moves To Phase 1 |
| **KVASIR-018** | Execution Plans Are Cached |
| **KVASIR-019** | AI Generates Declarative Descriptions, Not Runtime Nodes |
| **KVASIR-020** | Kvasir Runtime Becomes Its Own Crate |

---

## Revised Core Architecture

```text
Application Layer
        │
        ▼

Kvasir Runtime
├── Scene Graph      (Geometry topology, retained state)
├── Execution Graph  (Render pass scheduling, barriers)
├── Resource Graph   (Lifetime management, virtualization)
├── Material Graph   (Shader generation, IR compilation)
├── Temporal Graph   (Animation, physics, cross-frame deps)
└── Accessibility Layer
        │
        ▼

Execution Planner
        │
        ▼

Raster / Compute / Hybrid
        │
        ▼

GPU Backend
```

---

## Phase 0 — Fix Blocking Bugs (Sprint 1–2)

Unchanged from original plan. Seven critical bugs must be resolved:

| Bug | Fix |
|-----|-----|
| `fs_copy` reads Mega-Atlas instead of scene texture | Change to sample `t_env` (bloom.wgsl:L3) |
| `stroke_path` DrawCall uses vertex cursor as index cursor | Capture `base_index = self.indices.len()` before tessellation (lib.rs:L4639-4650) |
| Parallel rayon passes share `ctx_scene_texture` without barrier | Encode Glass and UI passes sequentially (lib.rs:L2947-3139) |
| Bloom extract overwrites backdrop blur texture | Allocate separate `bloom_tex_a/b` (lib.rs:L3174-3199) |
| `vs_fullscreen` draws 6 vertices — second triangle is degenerate | Change all fullscreen draws to `p.draw(0..3, 0..1)` (common.wgsl:L87-92) |
| Glyph atlas fallback writes to `(0,0)` on full atlas | Return early with error log (lib.rs:L3845-3852, L3977-3983) |
| SVG tessellation `.unwrap()` panics on malformed paths | Propagate as `Result`, skip degenerate paths (lib.rs:L5118-5132) |
| Clip SDF uses `clip_position.xy` as NDC | Remove the `* 0.5 + 0.5 * resolution` transform (shapes.wgsl:L63-74) |
| Bifrost glass alpha is 1–3% | Fix the fresnel alpha calculation (shapes.wgsl:L100-171) |

---

## Phase 1 — Resource Graph Foundation (Sprint 3–6)

**NEW:** Resource Virtualization moves to Phase 1, not Phase 2.

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
    pub kind: ResourceKind,
    pub label: &'static str,
    pub lifetime: ResourceLifetime,
}

pub enum ResourceLifetime {
    Frame,          // allocation AND content are transient
    FrameContent,    // allocation persists, content rebuilt each frame
    Persistent,     // lives across frames
    Streaming,      // loaded asynchronously
}
```

### 1.2 Build the ResourceRegistry

```rust
pub struct ResourceRegistry {
    descriptors: HashMap<ResourceId, ResourceDescriptor>,
    gpu_images: HashMap<ResourceId, wgpu::Texture>,
    gpu_buffers: HashMap<ResourceId, wgpu::Buffer>,
    lru: LruCache<ResourceId, ()>,
    next_id: AtomicU64,
}

impl ResourceRegistry {
    pub fn register(&mut self, desc: ResourceDescriptor) -> ResourceId
    pub fn get_image(&self, id: ResourceId) -> Option<&wgpu::Texture>
    pub fn get_or_create_image(&mut self, desc: &ResourceDescriptor, device: &wgpu::Device) -> ResourceId
    pub fn evict_frame_resources(&mut self)
    pub fn reclaim(&mut self, budget: u64)
    pub fn cache_execution_plan(&mut self, plan: ExecutionPlan)  // KVASIR-018
}
```

---

## Phase 2 — Execution Graph Core (Sprint 7–14)

**RENAMED:** Previously "Kvasir Graph Core" — now explicitly "Execution Graph Core" since it handles only render scheduling.

### 2.1 The KvasirNode Trait (Execution-focused)

```rust
// cvkg-render-gpu/src/kvasir/execution_node.rs

pub trait ExecutionNode: Send + Sync {
    fn label(&self) -> &'static str;
    fn inputs(&self) -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];

    fn execute(
        &self,
        ctx: &mut ExecutionContext,
        registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError>;

    fn execution_hint(&self) -> ExecutionHint { ExecutionHint::Raster }
    fn barrier_dependencies(&self) -> &[BarrierType] { &[] }  // KVASIR-013
}

pub enum ExecutionHint { Raster, Compute, Hybrid }
pub enum BarrierType { Texture, Buffer, Memory }
```

### 2.2 Execution Graph Data Structure

```rust
// cvkg-render-gpu/src/kvasir/execution_graph.rs

pub struct ExecutionGraph {
    nodes: SlotMap<NodeKey, Box<dyn ExecutionNode>>,
    edges: Vec<Edge>,  // (producer NodeKey, ResourceId, consumer NodeKey)
    roots: Vec<NodeKey>,
    sinks: Vec<NodeKey>,
    cached_plans: HashMap<GraphSignature, ExecutionPlan>,  // KVASIR-018
}

impl ExecutionGraph {
    pub fn plan(&self) -> Result<ExecutionPlan, PlanningError> {
        // Topological sort with barrier insertion
        // Uses cached_plans to skip recomputation
    }
}
```

---

## Phase 3 — Scene Graph Integration (Sprint 15–18)

The Scene Graph becomes a **separate domain** from Execution Graph.

### 3.1 Scene Graph Manages Spatial Relationships

```rust
// cvkg-scene/src/lib.rs (existing) becomes Scene Graph

pub struct SceneGraph {
    nodes: HashMap<SceneNodeId, SceneNode>,
    parent_child: HashMap<SceneNodeId, Vec<SceneNodeId>>,
    transform_dirty: HashSet<SceneNodeId>,
}

// Scene Graph feeds data to Execution Graph, does not merge with it
```

### 3.2 Node-to-Pass Translation

```rust
// Translation layer between Scene Graph and Execution Graph
pub fn translate_scene_to_execution(scene: &SceneGraph) -> ExecutionGraph {
    // Creates render passes from scene nodes
    // Does NOT modify scene graph structure
}
```

---

## Phase 4 — Material Graph with IR (Sprint 19–24)

**REVISED:** Material Graph now compiles through an IR layer.

### 4.1 Material Graph (High-level)

```rust
// cvkg-render-gpu/src/kvasir/material_graph.rs

pub struct MaterialNode {
    pub graph: MaterialGraphIR,
    pub inputs: Vec<TextInput>,
    pub outputs: Vec<PinInfo>,
}
```

### 4.2 Material IR (Intermediate Representation)

```rust
// NEW: MaterialGraphIR - platform-agnostic shader description

pub enum MaterialOp {
    SampleTexture { uv: Expr, tex_index: u32 },
    Blend { src: Expr, dst: Expr, mode: BlendMode },
    Transform { position: Expr, matrix: Mat4 },
    SDFBox { position: Expr, size: Expr },
    Fresnel { view: Expr, normal: Expr },
}

pub struct MaterialGraphIR {
    pub ops: Vec<MaterialOp>,
    pub constants: HashMap<String, f32>,
}
```

### 4.3 WGSL Backend Compiler

```rust
// cvkg-render-gpu/src/kvasir/wgsl_backend.rs

pub fn compile_material_ir(ir: &MaterialGraphIR) -> Result<String, CompileError> {
    // Translates MaterialGraphIR to WGSL
    // Enables multiple backends in future (SPIR-V, MSL)
}
```

---

## Phase 5 — Accessibility Layer Split (Sprint 25–28)

**REVISED:** Split into Visual and Semantic services.

### 5.1 Visual Accessibility Service

```rust
// Produces visual modifications (high contrast, color filters)
pub struct VisualAccessibilityNode {
    pub mode: AccessibilityMode,
    pub intensity: f32,
}
```

### 5.2 Semantic Accessibility Service

```rust
// Produces semantic descriptions for screen readers
pub struct SemanticAccessibilityNode {
    pub element_id: String,
    pub role: AriaRole,
    pub state: AriaState,
}
```

---

## Phase 6 — Temporal Graph (NEW: Sprint 29–34)

**NEW PHASE:** Handles animation and cross-frame dependencies.

### 6.1 Temporal Graph Structure

```rust
// cvkg-anim/src/temporal_graph.rs

pub struct TemporalNode {
    pub duration: Duration,
    pub dependencies: Vec<NodeId>,
    pub interpolation: InterpolationType,
}

pub enum InterpolationType {
    Step, Linear, CubicBezier, Spring,
}
```

### 6.2 Cross-Frame Resource Tracking

```rust
pub struct TemporalResource {
    pub id: ResourceId,
    pub lifetime: Duration,  // how long this resource lives
    pub dependencies: Vec<(FrameOffset, ResourceId)>,  // what it depends on
}
```

---

## AI Integration Model (REVISED: KVASIR-019)

AI generates **declarative descriptions**, not runtime nodes.

```rust
// Instead of: AI creates node instances
// Do this:

pub struct AIGeneratedScene {
    pub scene_description: SceneGraphDescription,
    pub render_description: ExecutionGraphDescription,
    pub material_descriptions: Vec<MaterialGraphIR>,
}
```

AI outputs are parsed and converted to actual graph nodes at load time.

---

## File Structure Changes

```text
cvkg/
├── cvkg-runtime/           (NEW: KVASIR-020)
│   ├── src/
│   │   ├── scene_graph.rs
│   │   ├── execution_graph.rs
│   │   ├── resource_graph.rs
│   │   ├── material_graph.rs
│   │   ├── temporal_graph.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── cvkg-render-gpu/
│   └── src/
│       └── kvasir/         (imports from cvkg-runtime)
└── ... other crates
```

---

## Summary of Architectural Changes

| Area | Original | Revised |
|------|----------|--------|
| Architecture | One Universal Graph | Multiple Specialized Graphs |
| Phase 1 | Resource Registry | Resource Virtualization + Registry |
| Phase 2 | "Kvasir Graph Core" | "Execution Graph Core" |
| Phase 4 | Material → WGSL | Material → IR → WGSL |
| Phase 5 | Single Accessibility Service | Visual + Semantic Split |
| Phase 6 | None | Temporal Graph (NEW) |
| AI Integration | Runtime Node Generation | Declarative Description Generation |

---

*Document incorporates critique from kvasir_update.md — static analysis of proposed architecture.*

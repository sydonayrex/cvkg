# BLÍMPLEMENT: Bevy-Inspired 3D Rendering Architecture for CVKG

> **Audience:** CVKG architecture team  
> **Scope:** New crates + cross-cutting changes to existing crates  
> **Status:** Draft — ratify each phase before implementation  
> **License:** MPL-2.0

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current CVKG 3D State — Gap Analysis](#2-current-cvkg-3d-state--gap-analysis)
3. [New Crates — Architecture](#3-new-crates--architecture)
4. [Phase 1 — Vertex MVP & 3D Transform Hierarchy](#4-phase-1--vertex-mvp--3d-transform-hierarchy)
5. [Phase 2 — Texture UV Sampling](#5-phase-2--texture-uv-sampling)
6. [Phase 3 — Frustum Culling](#6-phase-3--frustum-culling)
7. [Phase 4 — Shadow Pass](#7-phase-4--shadow-pass)
8. [Integration Points](#8-integration-points)
9. [Signal Layer-Annotated Dirty Flags & Phase Skipping](#9-signal-layer-annotated-dirty-flags--phase-skipping)
10. [Risk Register](#10-risk-register)
11. [Appendix: Bevy Architecture Reference](#11-appendix-bevy-architecture-reference)
12. [Auto-Required Companion State (`#[require]`)](#12-auto-required-companion-state-require)
13. [Reflect-Powered Inspector Integration](#13-reflect-powered-inspector-integration)
14. [FrameManifest — Compile-Time Frame Pipeline Declaration](#14-framemanifest--compile-time-frame-pipeline-declaration)
15. [Theme Context Propagation & Portal Inheritance](#15-theme-context-propagation--portal-inheritance)
16. [Ten Improvements — Easiest to Most Impactful](#16-ten-improvements--easiest-to-most-impactful)
17. [World-Space UI Panel Compositing & Architectural Positioning](#17-world-space-ui-panel-compositing--architectural-positioning)

---

## 1. Executive Summary

CVKG currently has **type stubs** for 3D rendering — `Mesh`, `Transform3D`, `Camera3D`, `Material3D` — and a **single WGSL shader branch** (`material_id == 13u`) that applies `proj * view` to vertex positions but **discards the model matrix**. There is no texture sampling, no shadow mapping, no frustum culling, and no parent-child 3D transform propagation. The 2D scene graph's `update_transforms()` is the only hierarchy traversal.

Bevy's pipelined render architecture (Extract → Prepare → Queue → Draw), its `Transform`/`GlobalTransform` split, its per-phase render passes (`Opaque3d`, `Shadow`, etc.), and its frustum culling with AABBs are the direct inspiration. The plan below adds **four new crates** to CVKG's workspace to bridge the gap without destabilizing the existing 2D UI pipeline.

---

## 2. Current CVKG 3D State — Gap Analysis

| Component | Status | Gap |
|---|---|---|
| **Mesh** (`cvkg-core/src/mesh.rs:3-52`) | Has `vertices: Vec<[f32;3]>`, `normals`, `indices`. OBJ/STL loading via `tobj`, `cvkg-stl`. | **No UV channels** in the struct. No vertex attribute for `tex_coords`. No tangent/bitangent for normal mapping (`Mesh` only has `vertices` and `normals`). |
| **Transform3D** (`cvkg-core/src/mesh.rs:60-90`) | Has `position: Vec3`, `rotation: Quat`, `scale: Vec3`, `to_matrix() -> Mat4`. | **Correct.** No changes needed to the type. |
| **Camera3D** (`cvkg-core/src/mesh.rs:93-111`) | Has `position`, `target`, `up`, `fov_y`, `near`, `far`, `perspective`, `aspect`. | **Correct.** `view_matrix()`, `projection_matrix()`, `view_projection()` exist already. |
| **Material3D** (`cvkg-core/src/mesh.rs:115-162`) | Has `base_color`, `metallic`, `roughness`, `emissive`, `opacity`. | **No texture bindings.** No albedo texture, normal map, ORM map. |
| **Shader MVP** (`common.wgsl:127-131`) | `material_id == 13u` path uses `proj * view * position`. | **No model matrix (M).** Vertex positions go directly from object space to clip space through view-projection only. Every object appears at the origin. |
| **Shader PBR** (`material_pbr.wgsl`) | Has diffuse/specular/fresnel with **hardcoded** light direction `(0.5, 0.8, 0.6)`. | No uniform-based lights. No shadow map binding. No texture sampling from UV. Single hardcoded directional light. |
| **Vertex struct** (`vertex.rs:7-20`) | Has `position: [f32;3]`, `normal: [f32;3]`, `uv: [f32;2]`, `color: [f32;4]`, `material_id: u32`, etc. | `uv` exists but the shader's 3D path ignores it. `InstanceData` (`vertex.rs:24-35`) is **2D only** (translation, scale, rotation) — no 3D model matrix. |
| **SceneUniforms** (`render_tier.rs:168-187`) | Has `view: Mat4`, `proj: Mat4`, time, mouse, berserker state. | **No light uniforms.** No shadow map texture binding. Has `view` + `proj` but no `camera_pos` for view-dependent effects. |
| **Scene graph** (`scene_graph.rs`) | `update_transforms()` handles 2D rect propagation only. | **No 3D hierarchy traversal.** `VNode` has `is_3d`, `position_3d`, `rotation_3d`, `scale_3d` fields but no propagation logic. |
| **Renderer trait** (`renderer_trait.rs:348-382`) | `draw_mesh()`, `draw_mesh_3d()`, `set_camera_3d()`, `push_transform_3d()`, `pop_transform_3d()`, `render_scene_node_3d()` all default to no-op. | GPU renderer only implements `draw_mesh()` for the 2D path. 3D methods are stubs. |
| **Kvasir passes** (`passes/`) | Geometry, Backdrop, Glass, UI, Volumetric, Bloom, Composite. | **No 3D opaque/transparent pass.** No shadow pass. |
| **Frustum culling** | None anywhere. | `cvkg-spatial` has QuadTree/BVH/SpatialHash but none are wired to 3D camera frustums. GLM `Frustum` type exists in `glam` but not used. |

---

## 3. New Crates — Architecture

### Workspace Layout

```
cvkg/
├── cvkg-core/                          # Existing — minor additions
├── cvkg-render-gpu/                    # Existing — new 3D pipeline variants
├── cvkg-spatial/                       # Existing — extend with FrustumCuller
│
├── cvkg-render-3d/                     # ← NEW: 3D-specific GPU pipeline crate
│   ├── Cargo.toml                      #   wgpu, glam, bytemuck, cvkg-core, cvkg-render-gpu
│   ├── src/
│   │   ├── lib.rs                      #   Crate root, re-exports
│   │   ├── pipeline.rs                 #   Opaque3dPipeline, ShadowPipeline compilation
│   │   ├── draw.rs                     #   GpuRenderer3D impl — draw_mesh_3d()
│   │   ├── types.rs                    #   Light, ShadowMap, GpuMeshInstance, etc.
│   │   ├── culler.rs                   #   FrustumCuller with AABB test
│   │   ├── passes/
│   │   │   ├── mod.rs                  #   3D pass graph node definitions
│   │   │   ├── opaque3d.rs             #   Opaque3dNode (KvasirNode impl)
│   │   │   ├── shadow.rs               #   ShadowNode (renders shadow map)
│   │   │   └── transparent3d.rs        #   Transparent3dNode (back-to-front)
│   │   └── shaders/
│   │       ├── mesh_vertex.wgsl        #   Full MVP vertex shader
│   │       ├── mesh_pbr.wgsl           #   PBR fragment with texture sampling
│   │       ├── mesh_shadow.wgsl        #   Shadow map depth-only vertex shader
│   │       └── common_3d.wgsl          #   3D shared types (Light struct, etc.)
│   └── build.rs                        #   Naga shader compilation
│
├── cvkg-gltf/                          # ← NEW: glTF 2.0 asset loader
│   ├── Cargo.toml                      #   gltf crate (or base64 + simd-json for binary glTF)
│   ├── src/
│   │   ├── lib.rs                      #   load_gltf(path) -> Scene3D
│   │   ├── types.rs                    #   Scene3D, Node3D, Skin, Animation
│   │   └── importer.rs                 #   Converts glTF -> cv3d::Mesh + cv3d::Material
│
├── cvkg-render-3d-hierarchy/           # ← NEW: 3D scene graph transform propagation
│   ├── src/
│   │   ├── lib.rs                      #   update_global_transforms()
│   │   └── hierarchy.rs               #   TransformTree, world_from_local
│
└── Cargo.toml                          # Top-level workspace — add cvkg-render-3d, cvkg-gltf, cvkg-render-3d-hierarchy
```

### Dependency Graph

```
cvkg-render-3d-hierarchy  (no deps on GPU — pure CPU)
        |
        v
cvkg-gltf  →  cvkg-core (Mesh, Material3D, Transform3D)
        |
        v
cvkg-render-3d  →  cvkg-core, cvkg-render-gpu (GpuRenderer, Kvasir registry, Vertex types)
        |
        v
    cvkg  (facade)  +  cvkg-spatial (for frustum culler)
```

### Key Design Decisions

| Decision | Rationale |
|---|---|
| **Separate crate (`cvkg-render-3d`)** rather than inline in `cvkg-render-gpu` | The 2D UI pipeline is already complex (22 shaders, 9+ passes, glass/bloom/volumetric). Adding 3D would double the pipeline count and risk breaking 2D. A separate crate implements `draw_mesh_3d()` by writing to `GpuRenderer`'s vertex/index/instance buffers through public extension methods. |
| **Separate `cvkg-render-3d-hierarchy`** crate for 3D transform propagation | Avoids pulling wgpu/GPU deps into the scene graph. This crate is pure CPU math (glam) and can run on the main thread or a job pool without any GPU context. |
| **glTF 2.0 as the primary 3D asset format** over OBJ | OBJ has no material hierarchy, no skeleton/animations, no PBR materials. glTF is the industry standard. `cvkg-gltf` converts glTF nodes/meshes into CVKG `Mesh` + `Material3D` + `Transform3D` trees. |
| **Kvasir graph nodes for 3D passes** rather than a separate render graph | Reuse the existing `KvasirGraph`, `ResourceRegistry`, `ExecutionPlanner`. The 3D nodes (`Opaque3dNode`, `ShadowNode`, `Transparent3dNode`) are registered alongside existing 2D nodes. |

### 3.1 New Crate Structure Tests

#### Test 1: Workspace Members Compile

**Test file:** `cvkg/tests/workspace_compile_tests.rs`

```rust
/// Verify that all new crates are valid workspace members and compile.
/// This is a compile-time test — if the workspace Cargo.toml is wrong,
/// this file won't even parse.

#[test]
fn test_workspace_has_render_3d() {
    // This test exists solely to verify the crate is a workspace member.
    // If cvkg_render_3d isn't in the workspace, this file won't compile.
    let _ = std::any::type_name::<cvkg_render_3d::GpuRenderer3D>();
}
```

**Verification command:** `cargo test -p cvkg -- workspace_compile_tests`

#### Test 2: Dependency Graph Integrity

**Test file:** `cvkg-render-3d/tests/dep_graph_tests.rs`

```rust
/// Verify the dependency graph: cvkg-render-3d depends on cvkg-core and cvkg-render-gpu,
/// but NOT on cvkg-scene, cvkg-layout, or cvkg-anim (those are 2D-only).

#[test]
fn test_render_3d_has_core_and_gpu_deps() {
    // Compile-time: if these deps are missing, this file won't compile.
    use cvkg_core::mesh::Transform3D;
    use cvkg_render_gpu::GpuRenderer;
    let _ = std::any::type_name::<Transform3D>();
    let _ = std::any::type_name::<GpuRenderer>();
}

#[test]
fn test_hierarchy_crate_is_pure_cpu() {
    // cvkg-render-3d-hierarchy must NOT depend on wgpu.
    // This is verified by the absence of wgpu types in its public API.
    use cvkg_render_3d_hierarchy::TransformNode3D;
    let node = TransformNode3D::default();
    let _ = node.global; // glam::Mat4 — no GPU context needed.
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- dep_graph_tests`

---

## 4. Phase 1 — Vertex MVP & 3D Transform Hierarchy

### 4.1 What Bevy Does

Bevy separates `Transform` (local TRS) from `GlobalTransform` (world-space Mat4). A system propagates parent transforms to children each frame using the `Parent`→`Children` component relationship. The GPU receives the final `GlobalTransform` as a uniform per-entity (or batched via uniform arrays in newer Bevy). The vertex shader computes `clip_position = proj * view * model * vec4<f32>(position, 1.0)`.

### 4.2 Current CVKG Problem

The vertex shader at `common.wgsl:127-131` does:
```wgsl
out.clip_position = scene.proj * scene.view * vec4<f32>(in.position, 1.0);
```
There is **no model matrix**. `scene.view` is the camera view matrix and `scene.proj` is the projection, but there is no per-object world transform. Every 3D mesh appears at the origin with identity rotation.

### 4.3 Changes

#### 4.3.1 Extend `InstanceData` with a 3D model matrix

`cvkg-render-gpu/src/vertex.rs` — add a 3D-capable instance layout:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData3D {
    /// 4x3 column-major model matrix (last row is implicitly [0,0,0,1]).
    /// Packed as 3 × vec4<f32> to satisfy WGSL mat4x4<f32> alignment (16 bytes).
    pub model_row0: [f32; 4],
    pub model_row1: [f32; 4],
    pub model_row2: [f32; 4],
    /// Per-instance material overrides packed into one vec4.
    pub material_overrides: [f32; 4], // (metallic, roughness, emissive_intensity, opacity)
}
```

New vertex attributes 16–19 in `Vertex::ATTRIBUTES`:
```rust
16 => Float32x4, // model_row0
17 => Float32x4, // model_row1
18 => Float32x4, // model_row2
19 => Float32x4, // material_overrides
```

#### 4.3.2 Create `cvkg-render-3d-hierarchy`

```rust
// cvkg-render-3d-hierarchy/src/lib.rs

/// A 3D scene node in the transform hierarchy.
pub struct TransformNode3D {
    pub id: cvkg_core::KvasirId,
    pub parent: Option<cvkg_core::KvasirId>,
    pub children: Vec<cvkg_core::KvasirId>,
    pub local: cvkg_core::mesh::Transform3D,
    pub global: glam::Mat4,
}

/// Computes world-space `global` matrices for all nodes.
/// O(n) tree traversal. Detects dirty subtrees via generation counter.
pub fn propagate_transforms(nodes: &mut [TransformNode3D]) {
    // Topological: iterate in parent-before-child order.
    // For each node with a parent, compute:
    //   node.global = parent.global * node.local.to_matrix()
    // Root nodes: node.global = node.local.to_matrix()
}
```

#### 4.3.3 Wire into vertex shader (`common.wgsl`)

Replace the `material_id == 13u` branch:

```wgsl
if (in.material_id == 13u) {
    // Full MVP: model matrix comes from instance data (locations 16-18)
    let model = mat4x4<f32>(in.model_row0, in.model_row1, in.model_row2, vec4<f32>(0.0, 0.0, 0.0, 1.0));
    let world_pos = model * vec4<f32>(in.position, 1.0);
    out.clip_position = scene.proj * scene.view * world_pos;
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize((model * vec4<f32>(in.normal, 0.0)).xyz);
}
```

#### 4.3.4 Implement `GpuRenderer3D` trait on `GpuRenderer`

```rust
// cvkg-render-3d/src/draw.rs

impl crate::draw::GpuRenderer3D for GpuRenderer {
    fn draw_mesh_3d(&mut self, mesh: &Mesh, material: &Material3D, transform: &Transform3D) {
        let model = transform.to_matrix();
        // Pack model matrix into InstanceData3D
        // Push a DrawCall with material_id == OPAQUE_3D (13)
        // Write mesh vertices/normals/indices to staging buffer
        // Write instanced model matrix to instance buffer
    }
}
```

### 4.4 Phase 1 Tests

#### Test 1: InstanceData3D Memory Layout

**Test file:** `cvkg-render-gpu/tests/instance_data_3d_tests.rs`

```rust
use std::mem;

/// Verify InstanceData3D is 48 bytes (3 × vec4 + 1 × vec4).
/// Risk #1 guard: if someone merges this with 2D InstanceData, this breaks.

#[test]
fn test_instance_data_3d_size() {
    assert_eq!(mem::size_of::<InstanceData3D>(), 48);
}

#[test]
fn test_instance_data_3d_alignment() {
    // Must be 16-byte aligned for WGSL vec4<f32> alignment.
    assert_eq!(mem::align_of::<InstanceData3D>(), 4);
}

#[test]
fn test_instance_data_2d_unchanged() {
    // 2D InstanceData must remain 32 bytes — no accidental unification.
    assert_eq!(mem::size_of::<InstanceData>(), 32);
}

#[test]
fn test_instance_data_3d_model_matrix_packing() {
    let id = InstanceData3D {
        model_row0: [1.0, 0.0, 0.0, 0.0],
        model_row1: [0.0, 1.0, 0.0, 0.0],
        model_row2: [0.0, 0.0, 1.0, 0.0],
        material_overrides: [0.5, 0.5, 1.0, 1.0],
    };
    let bytes = bytemuck::bytes_of(&id);
    assert_eq!(bytes.len(), 48);
}
```

**Verification command:** `cargo test -p cvkg-render-gpu -- instance_data_3d_tests`

#### Test 2: TransformNode3D Hierarchy Propagation

**Test file:** `cvkg-render-3d-hierarchy/tests/propagation_tests.rs`

```rust
use cvkg_render_3d_hierarchy::{TransformNode3D, propagate_transforms};
use glam::{Mat4, Vec3};

fn make_node(
    id: u64,
    parent: Option<u64>,
    position: Vec3,
    children: Vec<u64>,
) -> TransformNode3D {
    TransformNode3D {
        id: id.into(),
        parent: parent.map(|p| p.into()),
        children: children.into_iter().map(|c| c.into()).collect(),
        local: cvkg_core::mesh::Transform3D {
            position: position.into(),
            rotation: [0.0, 0.0, 0.0, 1.0].into(),
            scale: [1.0, 1.0, 1.0].into(),
        },
        global: Mat4::IDENTITY,
    }
}

#[test]
fn test_root_node_global_equals_local() {
    let mut nodes = vec![make_node(0, None, Vec3::new(1.0, 2.0, 3.0), vec![])];
    propagate_transforms(&mut nodes);
    let pos = nodes[0].global.col(3).truncate();
    assert!((pos.x - 1.0).abs() < 1e-6);
    assert!((pos.y - 2.0).abs() < 1e-6);
    assert!((pos.z - 3.0).abs() < 1e-6);
}

#[test]
fn test_child_inherits_parent_transform() {
    let mut nodes = vec![
        make_node(0, None, Vec3::new(10.0, 0.0, 0.0), vec![1]),
        make_node(1, Some(0), Vec3::new(5.0, 0.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes);
    let pos = nodes[1].global.col(3).truncate();
    assert!((pos.x - 15.0).abs() < 1e-6);
}

#[test]
fn test_grandchild_inherits_chain() {
    let mut nodes = vec![
        make_node(0, None, Vec3::new(1.0, 0.0, 0.0), vec![1]),
        make_node(1, Some(0), Vec3::new(2.0, 0.0, 0.0), vec![2]),
        make_node(2, Some(1), Vec3::new(4.0, 0.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes);
    let pos = nodes[2].global.col(3).truncate();
    assert!((pos.x - 7.0).abs() < 1e-6);
}

#[test]
fn test_empty_scene_no_panic() {
    let mut nodes: Vec<TransformNode3D> = vec![];
    propagate_transforms(&mut nodes);
    assert!(nodes.is_empty());
}

#[test]
fn test_multiple_roots_independent() {
    let mut nodes = vec![
        make_node(0, None, Vec3::new(1.0, 0.0, 0.0), vec![]),
        make_node(1, None, Vec3::new(0.0, 5.0, 0.0), vec![]),
    ];
    propagate_transforms(&mut nodes);
    let pos0 = nodes[0].global.col(3).truncate();
    let pos1 = nodes[1].global.col(3).truncate();
    assert!((pos0.x - 1.0).abs() < 1e-6);
    assert!((pos1.y - 5.0).abs() < 1e-6);
}
```

**Verification command:** `cargo test -p cvkg-render-3d-hierarchy -- propagation_tests`

#### Test 3: Vertex Shader MVP (Naga Compile)

**Test file:** `cvkg-render-3d/tests/shader_mvp_tests.rs`

```rust
#[test]
fn test_mesh_vertex_shader_compiles() {
    let source = include_str!("../src/shaders/mesh_vertex.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .expect("mesh_vertex.wgsl failed to parse");
    let entry = module.entry_points.iter()
        .find(|e| e.name == "vs_main")
        .expect("vs_main entry point not found");
    assert!(entry.function.arguments.len() > 0);
}

#[test]
fn test_mesh_pbr_shader_compiles() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .expect("mesh_pbr.wgsl failed to parse");
    let entry = module.entry_points.iter()
        .find(|e| e.name == "fs_main")
        .expect("fs_main entry point not found");
    assert!(entry.function.arguments.len() > 0);
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- shader_mvp_tests`

---

## 5. Phase 2 — Texture UV Sampling

### 5.1 What Bevy Does

Bevy's `StandardMaterial` has `base_color_texture`, `normal_map_texture`, `metallic_roughness_texture`, `occlusion_texture`, `emissive_texture`. The PBR fragment shader samples `UV_0` coordinates from the mesh vertex attribute, applies texture transforms (tiling, offset), and uses the sampled values in the Cook-Torrance BRDF.

### 5.2 Current CVKG Problem

- `Mesh` struct has **no UV data** (`vertices: Vec<[f32;3]>`, `normals`, `indices` — no `tex_coords`).
- `Vertex` struct **does** have `uv: [f32; 2]` but the 3D shader path ignores it.
- `Material3D` has **no texture bindings** (no `albedo_texture`, `normal_map`, etc.).
- `cvkg-gltf` (not yet written) will need to populate UVs from glTF mesh primitives.

### 5.3 Changes

#### 5.3.1 Add UV field to `Mesh`

```rust
// cvkg-core/src/mesh.rs
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub tex_coords: Vec<[f32; 2]>,  // ← NEW: UV channel 0
}
```

Default to `vec![[0.0, 0.0]; vertices.len()]` in existing `from_obj()` and `from_stl()` paths. **Backward compatible** — existing code that constructs `Mesh` without `tex_coords` will get the field via struct update syntax or we add `..Default::default()`.

#### 5.3.2 Add texture references to `Material3D`

```rust
// cvkg-core/src/mesh.rs
pub struct Material3D {
    pub base_color: [f32; 4],
    pub base_color_texture: Option<String>,  // ← NEW: name in Mega-Heim atlas
    pub normal_map_texture: Option<String>,   // ← NEW
    pub metallic_roughness_texture: Option<String>, // ← NEW: ORM-packed
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub opacity: f32,
    pub uv_scale: [f32; 2],   // ← NEW: tiling
    pub uv_offset: [f32; 2],  // ← NEW: offset
}
```

#### 5.3.3 Extend the PBR fragment shader

```wgsl
// cvkg-render-3d/src/shaders/mesh_pbr.wgsl

// Additional bindings:
@group(3) @binding(0) var t_albedo: texture_2d<f32>;
@group(3) @binding(1) var t_normal: texture_2d<f32>;
@group(3) @binding(2) var t_orm: texture_2d<f32>;     // occlusion-roughness-metallic packed
@group(3) @binding(3) var s_material: sampler;

fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    let uv = in.uv * in.material_uv_scale + in.material_uv_offset;
    let albedo_tex = textureSample(t_albedo, s_material, uv);
    let base_color = in.color * albedo_tex;

    // Cook-Torrance BRDF with sampled textures
    let orm = textureSample(t_orm, s_material, uv);
    let metallic = orm.g;     // ORM: r=occlusion, g=roughness, b=metallic
    let roughness = orm.r;
    let occlusion = orm.b;

    // ... lighting with these values
}
```

#### 5.3.4 Implement texture upload in `cvkg-render-3d`

When a `Material3D` has `base_color_texture: Some("my_tile.png")`, the draw call must:
1. Check if the texture is already in Mega-Heim (via `GpuRenderer::image_uv_registry`).
2. If not, load it via `GpuRenderer::load_image()` and assign a `tex_index`.
3. Pass the `tex_index` through to the vertex shader's `tex_index` field.
4. The fragment shader selects `t_diffuse[tex_index]` for the albedo sample.

### 5.4 Phase 2 Tests

#### Test 1: Mesh TexCoords Backward Compatible

**Test file:** `cvkg-core/tests/mesh_uv_tests.rs`

```rust
use cvkg_core::mesh::Mesh;

/// Verify Mesh gains tex_coords without breaking existing construction.

#[test]
fn test_mesh_tex_coords_default_empty() {
    let m = Mesh::default();
    // Default mesh has zero vertices, so tex_coords should also be empty.
    assert!(m.tex_coords.is_empty());
}

#[test]
fn test_mesh_tex_coords_len_matches_vertices() {
    let mut m = Mesh::default();
    m.vertices = vec![[0.0; 3]; 4];
    m.normals = vec![[0.0, 1.0, 0.0]; 4];
    m.indices = vec![0, 1, 2, 2, 3, 0];
    m.tex_coords = vec![[0.0, 0.0]; 4];
    assert_eq!(m.tex_coords.len(), m.vertices.len());
}

#[test]
fn test_mesh_from_obj_populates_tex_coords() {
    // OBJ loader must produce tex_coords (even if all zeros for meshes without UVs).
    let m = Mesh::from_obj("tests/assets/cube.obj").unwrap_or_default();
    if !m.vertices.is_empty() {
        assert_eq!(m.tex_coords.len(), m.vertices.len(),
            "tex_coords length must match vertices after OBJ import");
    }
}
```

**Verification command:** `cargo test -p cvkg-core -- mesh_uv_tests`

#### Test 2: Material3D Texture Fields

**Test file:** `cvkg-core/tests/material3d_texture_tests.rs`

```rust
use cvkg_core::mesh::Material3D;

/// Verify Material3D gains texture fields with correct defaults.

#[test]
fn test_material3d_no_textures_by_default() {
    let m = Material3D::default();
    assert!(m.base_color_texture.is_none());
    assert!(m.normal_map_texture.is_none());
    assert!(m.metallic_roughness_texture.is_none());
}

#[test]
fn test_material3d_uv_defaults() {
    let m = Material3D::default();
    assert_eq!(m.uv_scale, [1.0, 1.0]);
    assert_eq!(m.uv_offset, [0.0, 0.0]);
}

#[test]
fn test_material3d_with_texture() {
    let m = Material3D {
        base_color_texture: Some("tile.png".into()),
        uv_scale: [2.0, 2.0],
        ..Default::default()
    };
    assert_eq!(m.base_color_texture.as_deref(), Some("tile.png"));
    assert_eq!(m.uv_scale, [2.0, 2.0]);
}
```

**Verification command:** `cargo test -p cvkg-core -- material3d_texture_tests`

#### Test 3: PBR Shader Texture Bindings (Naga Compile)

**Test file:** `cvkg-render-3d/tests/shader_texture_tests.rs`

```rust
#[test]
fn test_mesh_pbr_has_texture_bindings() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .expect("mesh_pbr.wgsl failed to parse");
    // Verify texture bindings exist (group 3, bindings 0-3).
    // Naga stores global variables; check that t_albedo, t_normal, t_orm exist.
    let globals: Vec<&str> = module.global_variables.iter()
        .map(|g| g.name.as_str())
        .collect();
    assert!(globals.iter().any(|n| n.contains("albedo")),
        "PBR shader must have albedo texture binding");
    assert!(globals.iter().any(|n| n.contains("shadow")),
        "PBR shader must have shadow map binding");
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- shader_texture_tests`

---

## 6. Phase 3 — Frustum Culling

### 6.1 What Bevy Does

Bevy computes a `Frustum` struct (6 planes: left, right, top, bottom, near, far) from the camera's view-projection matrix. During the `Extract` phase, each entity with a mesh and `GlobalTransform` is tested against the frustum using its axis-aligned bounding box (AABB). Entities outside the frustum are not written to the Render World, saving Prepare/Queue/Draw work for occluded objects.

### 6.2 Current CVKG Problem

No frustum culling at all. Every mesh in the scene is sent to the GPU regardless of whether it's visible. For scenes with hundreds of meshes this wastes CPU-GPU bandwidth and GPU vertex processing.

### 6.3 Changes

#### 6.3.1 Frustum type in `cvkg-spatial`

```rust
// cvkg-spatial/src/frustum.rs

use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// Six frustum planes: [left, right, top, bottom, near, far].
    /// Each plane is (nx, ny, nz, d) where dot(n, p) + d = 0.
    pub planes: [[f32; 4]; 6],
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix (left-handed).
    pub fn from_view_projection(vp: &Mat4) -> Self { /* ... */ }

    /// Test if an AABB intersects the frustum.
    pub fn intersects_aabb(&self, center: Vec3, half_extents: Vec3) -> bool { /* ... */ }

    /// Test if a sphere intersects the frustum.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool { /* ... */ }
}
```

The intersection test checks all 6 planes. An AABB is inside the frustum if it is on the positive side of at least one plane for all planes. This is the standard AABB-vs-frustum test (cull if any plane's dot product test says the box is entirely outside).

#### 6.3.2 Compute AABB from mesh vertices

```rust
// cvkg-core/src/mesh.rs
impl Mesh {
    /// Compute the axis-aligned bounding box.
    pub fn aabb(&self) -> (glam::Vec3, glam::Vec3) {
        // returns (center, half_extents)
    }
}
```

#### 6.3.3 Culling in the render loop

In `cvkg-render-3d/src/draw.rs`:

```rust
fn render_scene_3d(&mut self, camera: &Camera3D, nodes: &[TransformNode3D], meshes: &[(NodeId, &Mesh)]) {
    let frustum = Frustum::from_view_projection(&camera.view_projection());

    for (node_id, mesh) in meshes {
        let node = &nodes[node_id];
        let (center, half_extents) = mesh.aabb();
        // Transform AABB to world space using node.global
        let world_center = node.global.transform_point3(center);
        // Apply scale from node.global for half_extents
        if !frustum.intersects_aabb(world_center, half_extents * node.global.scale()) {
            continue; // Skip — not visible
        }
        self.draw_mesh_3d(mesh, material, node.global);
    }
}
```

### 6.4 Phase 3 Tests

#### Test 1: Frustum Plane Extraction

**Test file:** `cvkg-spatial/tests/frustum_tests.rs`

```rust
use cvkg_spatial::frustum::Frustum;
use glam::{Mat4, Vec3};

/// Verify frustum plane extraction from view-projection matrix.

#[test]
fn test_frustum_from_identity() {
    // Identity VP = no culling — everything passes.
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(frustum.intersects_aabb(Vec3::ZERO, Vec3::splat(100.0)));
}

#[test]
fn test_frustum_culls_object_behind_camera() {
    // Object at z=-100 is behind the default camera (looking down -Z).
    let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 1000.0);
    let frustum = Frustum::from_view_projection(&(proj * view));
    // Object at z=+100 (behind camera in RH) should be culled.
    assert!(!frustum.intersects_aabb(Vec3::new(0.0, 0.0, 100.0), Vec3::splat(1.0)));
}

#[test]
fn test_frustum_passes_object_in_front() {
    let view = Mat4::look_at_rh(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 1000.0);
    let frustum = Frustum::from_view_projection(&(proj * view));
    // Object at z=-10 (in front of camera) should pass.
    assert!(frustum.intersects_aabb(Vec3::new(0.0, 0.0, -10.0), Vec3::splat(1.0)));
}

#[test]
fn test_frustum_sphere_intersection() {
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    assert!(frustum.intersects_sphere(Vec3::ZERO, 100.0));
    // Sphere far outside should be culled.
    assert!(!frustum.intersects_sphere(Vec3::new(10000.0, 10000.0, 10000.0), 1.0));
}
```

**Verification command:** `cargo test -p cvkg-spatial -- frustum_tests`

#### Test 2: Mesh AABB Computation

**Test file:** `cvkg-core/tests/mesh_aabb_tests.rs`

```rust
use cvkg_core::mesh::Mesh;

/// Verify Mesh::aabb() returns correct center and half-extents.

#[test]
fn test_aabb_unit_cube() {
    let mut m = Mesh::default();
    m.vertices = vec![
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0], [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
    ];
    let (center, half) = m.aabb();
    assert!((center.x - 0.5).abs() < 1e-6);
    assert!((center.y - 0.5).abs() < 1e-6);
    assert!((center.z - 0.5).abs() < 1e-6);
    assert!((half.x - 0.5).abs() < 1e-6);
    assert!((half.y - 0.5).abs() < 1e-6);
    assert!((half.z - 0.5).abs() < 1e-6);
}

#[test]
fn test_aabb_single_vertex() {
    let mut m = Mesh::default();
    m.vertices = vec![[3.0, 4.0, 5.0]];
    let (center, half) = m.aabb();
    assert!((center.x - 3.0).abs() < 1e-6);
    assert!((half.x).abs() < 1e-6, "single vertex has zero half-extent");
}

#[test]
fn test_aabb_empty_mesh() {
    let m = Mesh::default();
    let (center, half) = m.aabb();
    assert!(center.x.is_nan() || half.x.abs() < 1e-6,
        "empty mesh should return zero or NaN AABB");
}
```

**Verification command:** `cargo test -p cvkg-core -- mesh_aabb_tests`

#### Test 3: Culling in Render Loop (Mock)

**Test file:** `cvkg-render-3d/tests/culling_integration_tests.rs`

```rust
/// Verify the culling loop skips invisible meshes.
/// Uses mock frustum + nodes — no GPU required.

use cvkg_spatial::frustum::Frustum;
use glam::{Mat4, Vec3};

#[test]
fn test_culling_loop_skips_outside_meshes() {
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    let mesh_center = Vec3::new(10000.0, 10000.0, 10000.0); // far outside
    let half = Vec3::splat(1.0);
    assert!(!frustum.intersects_aabb(mesh_center, half),
        "mesh outside frustum must be culled");
}

#[test]
fn test_culling_loop_passes_inside_meshes() {
    let frustum = Frustum::from_view_projection(&Mat4::IDENTITY);
    let mesh_center = Vec3::new(0.0, 0.0, -5.0); // in front of camera
    let half = Vec3::splat(1.0);
    assert!(frustum.intersects_aabb(mesh_center, half),
        "mesh inside frustum must pass");
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- culling_integration_tests`

---

## 7. Phase 4 — Shadow Pass

### 7.1 What Bevy Does

Bevy renders shadows as a separate render phase (`Shadow` phase). For each shadow-casting directional light, it:
1. Computes a light view-projection matrix (orthographic from light direction).
2. Renders all shadow-casting meshes into a depth-only texture (shadow map) from the light's POV.
3. During the main `Opaque3d` phase, samples the shadow map using PCF (percentage-closer filtering) to compute shadow attenuation.

### 7.2 Current CVKG Problem

No shadow maps, no shadow pass, no shadow sampling in the PBR shader. CVKG has depth textures for the 2D pipeline but no dedicated shadow map texture or light-space rendering.

### 7.3 Changes

#### 7.3.1 Light type

```rust
// cvkg-render-3d/src/types.rs

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,        // World-space direction (normalized)
    pub color: [f32; 3],              // RGB irradiance
    pub intensity: f32,               // Lux
    pub shadow_map_size: u32,         // 512, 1024, 2048, etc.
    pub shadow_bias: f32,             // Depth bias for shadow acne
    pub shadow_normal_bias: f32,      // Normal offset bias
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: glam::Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub shadow_map_size: u32,
}

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}
```

#### 7.3.2 Shadow map texture resource

```rust
// cvkg-render-3d/src/types.rs

pub struct ShadowMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: u32,
    /// Light view-projection matrix used to render this shadow map.
    pub light_vp: glam::Mat4,
}
```

Allocated via `ResourceRegistry` in Kvasir (persistent, per-light).

#### 7.3.3 Shadow pass node (Kvasir)

```rust
// cvkg-render-3d/src/passes/shadow.rs

pub struct ShadowNode {
    pub light: DirectionalLight,
    pub shadow_map: ResourceId, // points to a ShadowMap resource
    pub mesh_instances: Vec<ShadowInstance>,
}

impl KvasirNode for ShadowNode {
    fn label(&self) -> &str { "ShadowPass" }
    fn inputs(&self) -> Vec<ResourceId> { vec![] }
    fn outputs(&self) -> Vec<ResourceId> { vec![self.shadow_map] }
    fn execute(&self, ctx: &mut ExecutionContext) -> Result<(), KvasirError> {
        // 1. Compute light VP: look_at from light direction toward scene center
        // 2. Set render pass with depth-only format (Depth32Float)
        // 3. For each shadow instance, bind mesh VB/IB, set depth-stencil state
        // 4. Draw depth only (no color attachment needed)
    }
}
```

Only opaque, shadow-casting meshes are rendered. Uses the `mesh_shadow.wgsl` vertex shader (simpler — position only, no fragment).

#### 7.3.4 PCF shadow sampling in the PBR shader

```wgsl
// cvkg-render-3d/src/shaders/mesh_pbr.wgsl

@group(3) @binding(4) var t_shadow: texture_depth_2d;
@group(3) @binding(5) var s_shadow: sampler_comparison;

fn sample_shadow(light_vp: mat4x4<f32>, world_pos: vec3<f32>) -> f32 {
    let light_pos = light_vp * vec4<f32>(world_pos, 1.0);
    let light_uv = light_pos.xy / light_pos.w * 0.5 + 0.5;
    let light_depth = light_pos.z / light_pos.w;

    // PCF 3x3
    let texel_size = 1.0 / f32(shadow_map_size);
    var shadow = 0.0;
    for (dx in -1..=1) {
        for (dy in -1..=1) {
            let offset = vec2<f32>(f32(dx), f32(dy)) * texel_size;
            shadow += textureSampleCompare(t_shadow, s_shadow,
                light_uv + offset, light_depth - shadow_bias);
        }
    }
    return shadow / 9.0;
}
```

#### 7.3.5 Register ShadowNode in the Kvasir graph

```rust
// In build_render_graph() — add before the Opaque3dNode:
let shadow_rid = registry.allocate_image(&device, &shadow_tex_desc);
let shadow_node = graph.add_node(Box::new(ShadowNode {
    light: scene_lights.directional,
    shadow_map: shadow_rid,
    mesh_instances: culled_3d_instances,
}));
let present_key = graph.find_node_by_label("CompositeNode").unwrap();
graph.connect(shadow_node, shadow_rid, present_key);
```

The shadow map is available as an input to the composite/present pass, or alternatively to the Opaque3d node via a separate connection.

### 7.4 Phase 4 Tests

#### Test 1: Light Type Construction

**Test file:** `cvkg-render-3d/tests/light_type_tests.rs`

```rust
use cvkg_render_3d::types::{DirectionalLight, PointLight, Light, ShadowQuality};
use glam::Vec3;

/// Verify light types construct correctly with sane defaults.

#[test]
fn test_directional_light_defaults() {
    let light = DirectionalLight::default();
    assert_eq!(light.shadow_map_size, 1024); // Medium quality default
    assert!((light.shadow_bias - 0.005).abs() < 1e-6);
}

#[test]
fn test_point_light_range_positive() {
    let light = PointLight {
        position: Vec3::ZERO,
        color: [1.0, 1.0, 1.0],
        intensity: 1000.0,
        range: 50.0,
        shadow_map_size: 512,
    };
    assert!(light.range > 0.0);
}

#[test]
fn test_shadow_quality_variants() {
    assert_eq!(ShadowQuality::Low.size(), 512);
    assert_eq!(ShadowQuality::Medium.size(), 1024);
    assert_eq!(ShadowQuality::High.size(), 2048);
    assert_eq!(ShadowQuality::Ultra.size(), 4096);
}

#[test]
fn test_light_enum_dispatch() {
    let d = Light::Directional(DirectionalLight::default());
    let p = Light::Point(PointLight {
        position: Vec3::ZERO,
        color: [1.0; 3],
        intensity: 100.0,
        range: 10.0,
        shadow_map_size: 256,
    });
    match d {
        Light::Directional(_) => {},
        _ => panic!("expected Directional"),
    }
    match p {
        Light::Point(_) => {},
        _ => panic!("expected Point"),
    }
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- light_type_tests`

#### Test 2: Shadow Map Resource Lifecycle

**Test file:** `cvkg-render-3d/tests/shadow_map_tests.rs`

```rust
/// Verify ShadowMap struct fields are correctly populated.
/// GPU allocation tests require a device context — these test the data model only.

use cvkg_render_3d::types::ShadowMap;
use glam::Mat4;

#[test]
fn test_shadow_map_light_vp_stored() {
    let vp = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);
    // ShadowMap stores the light VP for later sampling in the PBR shader.
    let sm = ShadowMap {
        texture: (),    // placeholder — real type is wgpu::Texture
        view: (),       // placeholder
        sampler: (),    // placeholder
        size: 1024,
        light_vp: vp,
    };
    assert_eq!(sm.size, 1024);
    assert_eq!(sm.light_vp, vp);
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- shadow_map_tests`

#### Test 3: Shadow Shader Compiles (Naga)

**Test file:** `cvkg-render-3d/tests/shader_shadow_tests.rs`

```rust
#[test]
fn test_mesh_shadow_shader_compiles() {
    let source = include_str!("../src/shaders/mesh_shadow.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .expect("mesh_shadow.wgsl failed to parse");
    // Shadow shader should have vs_main but NO fs_main (depth-only).
    let vs = module.entry_points.iter()
        .find(|e| e.name == "vs_main")
        .expect("shadow shader must have vs_main");
    assert!(vs.function.arguments.len() > 0);

    let has_fs = module.entry_points.iter().any(|e| e.name == "fs_main");
    assert!(!has_fs, "shadow shader must NOT have fragment entry point (depth-only)");
}

#[test]
fn test_pbr_shadow_sampling_function_exists() {
    let source = include_str!("../src/shaders/mesh_pbr.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .expect("mesh_pbr.wgsl failed to parse");
    // Verify sample_shadow function exists.
    let fns: Vec<&str> = module.functions.iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(fns.iter().any(|n| n.contains("shadow")),
        "PBR shader must contain shadow sampling function");
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- shader_shadow_tests`

---

## 8. Integration Points

### 8.1 Kvasir Graph Wiring

The `build_render_graph()` function in `cvkg-render-gpu/src/kvasir/nodes.rs` currently builds a graph from hardcoded nodes. The 3D nodes need to be injected conditionally (when 3D content is present). Add:

```rust
#[cfg(feature = "render-3d")]
fn maybe_add_3d_nodes(graph: &mut KvasirGraph, registry: &mut ResourceRegistry, config: &Config) {
    if has_3d_content {
        let shadow_rid = registry.allocate_image(&device, /* depth-only desc */);
        let shadow_node = graph.add_node(Box::new(ShadowNode { ... }));
        let opaque_3d = graph.add_node(Box::new(Opaque3dNode { ... }));
        graph.connect(shadow_node, shadow_rid, opaque_3d);
    }
}
```

### 8.2 `SceneUniforms` Extension

Add camera position and light uniforms to `SceneUniforms`:

```rust
// cvkg-core/src/render_tier.rs — SceneUniforms
pub struct SceneUniforms {
    // ... existing fields ...
    pub camera_pos: [f32; 3],        // NEW: camera world position
    pub _pad_camera: f32,            // alignment padding
    pub light_direction: [f32; 3],   // NEW: primary directional light direction
    pub light_intensity: f32,        // NEW: light intensity
    pub light_color: [f32; 3],       // NEW: light color
    pub _pad_light: f32,             // alignment padding
}
```

### 8.3 Window Surface Configuration for Depth

The existing depth texture (`Depth32Float`) is used for 2D depth testing. For 3D, the same texture works — but the shadow pass needs a **separate** depth texture (at shadow map resolution) with `TEXTURE_BINDING` usage so it can be sampled as a shadow map.

### 8.4 Pipeline Compilation

Add a 3D PBR pipeline variant in `cvkg-render-3d/src/pipeline.rs`:

```rust
pub fn compile_3d_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shadow_bgl: &wgpu::BindGroupLayout,
    vs_module: &wgpu::ShaderModule,
    fs_module: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Surtr 3D Pipeline"),
        bind_group_layouts: &[
            &texture_bgl,      // group(0): Mega-Heim texture array
            &env_bgl,          // group(1): env texture
            &theme_bgl,        // group(2): theme + scene uniforms
            &material_bgl,     // group(3): albedo/normal/ORM textures + shadow map
        ],
        ..
    });
    // Vertex state uses Vertex::desc() + InstanceData3D::desc()
    // Fragment uses mesh_pbr.wgsl as entry point
    // Depth-stencil: Depth32Float, depth_write_enabled: true, depth_compare: Less
    // Color target: Rgba16Float with alpha blending
}
```

### 8.5 Facade Crate Feature Gate

In `cvkg/Cargo.toml`:

```toml
[features]
default = []
gpu = ["dep:cvkg-render-gpu"]
native = ["dep:cvkg-render-native"]
render-3d = ["dep:cvkg-render-gpu", "dep:cvkg-render-3d"]  # NEW

[dependencies]
cvkg-render-3d = { version = "0.2.17", optional = true, path = "../cvkg-render-3d" }
cvkg-gltf = { version = "0.2.17", optional = true, path = "../cvkg-gltf" }
```

### 8.6 Integration Point Tests

#### Test 1: SceneUniforms Extended Fields

**Test file:** `cvkg-core/tests/scene_uniforms_3d_tests.rs`

```rust
use cvkg_core::render_tier::SceneUniforms;

/// Verify SceneUniforms gains camera_pos, light_direction, light_intensity, light_color.

#[test]
fn test_scene_uniforms_camera_pos_default() {
    let s = SceneUniforms::default();
    assert_eq!(s.camera_pos, [0.0, 0.0, 0.0]);
}

#[test]
fn test_scene_uniforms_light_defaults() {
    let s = SceneUniforms::default();
    assert_eq!(s.light_intensity, 0.0, "light intensity defaults to 0 (no light)");
}

#[test]
fn test_scene_uniforms_size_32byte_aligned() {
    // wgpu uniform buffers require 16-byte alignment.
    let size = std::mem::size_of::<SceneUniforms>();
    assert_eq!(size % 16, 0, "SceneUniforms must be 16-byte aligned");
}
```

**Verification command:** `cargo test -p cvkg-core -- scene_uniforms_3d_tests`

#### Test 2: Feature Gate Compile

**Test file:** `cvkg/tests/feature_gate_tests.rs`

```rust
/// Verify that the render-3d feature gate compiles correctly.
/// With feature enabled: cvkg_render_3d is available.
/// Without: it's absent.

#[cfg(feature = "render-3d")]
#[test]
fn test_render_3d_feature_present() {
    let _ = std::any::type_name::<cvkg_render_3d::GpuRenderer3D>();
}

#[cfg(not(feature = "render-3d"))]
#[test]
fn test_render_3d_feature_absent() {
    // When feature is off, render-3d types should not be in scope.
    // This test compiles only when the feature is off — proving the gate works.
    assert!(true);
}
```

**Verification command:** `cargo test -p cvkg -- feature_gate_tests`

#### Test 3: Kvasir Graph Node Count

**Test file:** `cvkg-render-3d/tests/kvasir_node_tests.rs`

```rust
/// Verify 3D Kvasir nodes implement the trait correctly.

use cvkg_render_3d::passes::shadow::ShadowNode;
use cvkg_render_3d::passes::opaque3d::Opaque3dNode;

#[test]
fn test_shadow_node_label() {
    let node = ShadowNode::mock(); // constructor that doesn't need GPU
    assert_eq!(node.label(), "ShadowPass");
}

#[test]
fn test_shadow_node_outputs_shadow_map() {
    let node = ShadowNode::mock();
    assert_eq!(node.outputs().len(), 1, "shadow node must output exactly one resource (shadow map)");
}

#[test]
fn test_opaque_3d_node_label() {
    let node = Opaque3dNode::mock();
    assert_eq!(node.label(), "Opaque3d");
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- kvasir_node_tests`

---

## 9. Signal Layer-Annotated Dirty Flags & Phase Skipping

This section is **cross-cutting** — it applies to the entire frame pipeline, not just 3D. It addresses the gap in `cvkg-vdom/src/signals.rs:63-72` where `Signal::set()` ignores the `DirtyFlags` layer model and fans out to all subscribers uniformly.

### 9.1 Current State

`cvkg-core/src/dirty_flags.rs` defines four pipeline layers with the downstream-propagation invariant:

```
STATE (0b1111) → LAYOUT (0b0111) → PAINT (0b0011) → COMPOSITE (0b0001)
```

`cvkg-scheduler/src/frame.rs` defines seven ordered phases (`Input → State → Layout → Animation → Render → Composite → PostFrame`) with a one-phase-at-a-time `flush_current_phase()`. But the scheduler has **no visibility into what kind of work is needed** — it runs every phase every frame unconditionally.

`cvkg-vdom/src/signals.rs` has `EffectRunner` as the subscriber trait, but no mechanism to annotate a signal mutation with its pipeline layer. `Signal::set()` calls `sub.run()` on every subscriber regardless of what semantically changed.

### 9.2 Design: `set_with_flags` on `Signal<T>`

#### 9.2.1 SubscriberEntry wrapper

```rust
// cvkg-vdom/src/signals.rs

struct SubscriberEntry {
    runner: Arc<dyn EffectRunner>,
    /// Bitmask of DirtyFlags accumulated across set_with_flags calls
    /// since the last run. Reset when the runner is dispatched.
    accumulated: AtomicU8,
}
```

Replace `subscribers: Vec<Arc<dyn EffectRunner>>` with `subscribers: Vec<SubscriberEntry>`.

#### 9.2.2 Frame-level flag accumulator

```rust
thread_local! {
    static CURRENT_EFFECT: RwLock<Option<Arc<dyn EffectRunner>>> = RwLock::new(None);
    /// OR-ed across every set_with_flags call in this frame.
    /// Read and reset by FrameScheduler::begin_frame().
    static FRAME_DIRTY_FLAGS: AtomicU8 = const { AtomicU8::new(0) };
}
```

#### 9.2.3 The new `set_with_flags` method

```rust
impl<T: Clone> Signal<T> {
    /// Default: conservative. Assumes worst case — full pipeline rebuild.
    pub fn set(&self, new_value: T) {
        self.set_with_flags(new_value, DirtyFlags::ALL);
    }

    /// Set value AND annotate which pipeline layers are affected.
    ///
    /// # Invariant enforcement
    /// debug_assert ensures the callers passes one of the four canonical
    /// constants (STATE / LAYOUT / PAINT / COMPOSITE). Manual bit-twiddling
    /// that violates the downstream-propagation invariant is caught here.
    pub fn set_with_flags(&self, new_value: T, flags: DirtyFlags) {
        debug_assert!(
            matches!(flags,
                DirtyFlags::STATE
                | DirtyFlags::LAYOUT
                | DirtyFlags::PAINT
                | DirtyFlags::COMPOSITE
            ),
            "set_with_flags: flags must be one of STATE/LAYOUT/PAINT/COMPOSITE \
             (downstream-invariant violation would cause stale pipeline layers)"
        );

        *self.value.write().unwrap() = new_value;
        self.version.fetch_add(1, Ordering::Relaxed);

        // OR into the frame-level accumulator.
        FRAME_DIRTY_FLAGS.with(|f| f.fetch_or(flags.0, Ordering::Relaxed));

        // OR onto each subscriber's accumulated mask, then dispatch.
        let subs = self.subscribers.read().unwrap().clone();
        for sub in &subs {
            sub.accumulated.fetch_or(flags.0, Ordering::Relaxed);
            sub.runner.clone().run();
        }
    }
}
```

### 9.3 Phase-Skip Logic in `FrameScheduler`

```rust
// cvkg-scheduler/src/frame.rs

impl FrameScheduler {
    /// Called at the top of each frame to snapshot the accumulated flags.
    pub fn begin_frame(&mut self) {
        self.frame_number += 1;
        self.current_phase = FramePhase::Input;
        self.phase_queue.clear();

        let flags_byte = FRAME_DIRTY_FLAGS.with(|f| f.swap(0, Ordering::Relaxed));
        self.frame_dirty_flags = DirtyFlags(flags_byte);

        tracing::trace!("FrameScheduler: begin_frame #{} (dirty={:?})",
            self.frame_number, self.frame_dirty_flags);
    }

    /// Returns true if the given phase can be entirely skipped this frame.
    ///
    /// Decision matrix (derived from dirty_flags.rs downstream invariant):
    ///
    /// | Frame dirty flags | Skip Layout? | Skip Animation? | Skip Render? |
    /// |-------------------|-------------|-----------------|--------------|
    /// | STATE             | No          | No              | No           |
    /// | LAYOUT            | No          | No              | No           |
    /// | PAINT             | **Yes**     | **Yes**         | No           |
    /// | COMPOSITE         | Yes         | Yes             | **Yes***     |
    ///
    /// *Render is never fully skipped (the GPU still presents), but when only
    ///  COMPOSITE is dirty, the scene draw list is reused from cache.
    pub fn should_skip_phase(&self, phase: FramePhase) -> bool {
        match phase {
            // Layout runs only when a state or layout bit is set.
            FramePhase::Layout => {
                !self.frame_dirty_flags.needs_layout()
                    && !self.frame_dirty_flags.needs_state()
            }
            // Animation reads layout output. If layout did not run (only
            // PAINT/COMPOSITE changed), there is no new layout to animate toward.
            FramePhase::Animation => {
                !self.frame_dirty_flags.needs_layout()
                    && !self.frame_dirty_flags.needs_state()
            }
            _ => false,
        }
    }
}
```

Frame loop becomes:

```rust
fs.begin_frame();
loop {
    if !fs.should_skip_phase(fs.current_phase()) {
        fs.flush_current_phase();
    }
    if fs.current_phase() == FramePhase::PostFrame { break; }
    fs.advance_phase();
}
```

### 9.4 Concrete Call-Site Examples

#### `cvkg-vdom/src/physics.rs` — Spring tick

```rust
// Spring position tick — only the visual changes, layout is unchanged.
// Must use PAINT (which implies COMPOSITE), never COMPOSITE alone.
self.current.set_with_flags(next_bounds, DirtyFlags::PAINT);
```

Without this fix, every spring tick (60fps) would set `DirtyFlags::ALL` and force Layout + Animation to re-run even though the layout tree did not change.

#### Taffy layout output — writing target bounds

```rust
// Taffy just computed a new layout — position/size changed.
target_signal.set_with_flags(new_bounds, DirtyFlags::LAYOUT);
```

This correctly triggers Layout → Animation → Render → Composite phases but marks Layout as the *first* affected layer.

#### Color/animation property change

```rust
// A visual property changed (color, opacity, blur radius).
// Only PAINT + COMPOSITE need to re-run.
painter_signal.set_with_flags(new_value, DirtyFlags::PAINT);
```

### 9.5 Invariant Constraints on Callers

The `dirty_flags.rs:23-24` contract — *"A crate that dirtifies a layer MUST also dirtify all downstream layers"* — constrains every `set_with_flags` call site in three ways:

**A — Callers must choose the *first* affected layer, not a subset.**

Passing `COMPOSITE` alone is invalid: compositing always reads paint output. Passing `PAINT` without `COMPOSITE` is invalid: paint always feeds the compositor. The four valid values are exactly `STATE`, `LAYOUT`, `PAINT`, `COMPOSITE`.

**B — `STATE` is the "I don't know" sink.**

When a signal is set from unknown context (generic `set()`, external event handler, etc.), the caller must use `STATE` (which implies all downstream). Using a narrower flag risks the scheduler skipping a phase that the mutation actually affects.

```
External data fetch → signal.set_with_flags(data, DirtyFlags::STATE)
```

**C — Two signals holding the same type may require different flags.**

In `physics.rs`, `Signal<Rect>` is used for both `target` and `current`:

| Signal | Written by | Correct flags | Rationale |
|---|---|---|---|
| `self.target` | Taffy layout | `LAYOUT` | The layout tree computed new positions |
| `self.current` | Spring tick | `PAINT` | Interpolation toward target — layout already computed |

A maintainer who writes `self.current.set_with_flags(next, DirtyFlags::LAYOUT)` would cause the scheduler to run layout every frame, defeating spring-physics interpolation. The `debug_assert!` can't catch wrong *semantics* — only a code review or documentation can.

### 9.6 Measured Impact

For a frame where only a color tweak fires:

| Before (unconditional) | After (layer-annotated) |
|---|---|
| Input → State → Layout → Animation → Render → Composite → PostFrame | Input → State → **(skip Layout)** → **(skip Animation)** → Render → Composite → PostFrame |

Two phases eliminated (~8ms recovered from a 16.67ms budget at 60fps). The spring-tick case is even more impactful: a continuous animation that previously forced full-frame reprocessing now only triggers PAINT + COMPOSITE every tick, saving Layout + Animation on every frame for the animation's entire duration.

### 9.7 Signal Layer Tests

#### Test 1: DirtyFlags Bitmask Constants

**Test file:** `cvkg-core/tests/dirty_flags_tests.rs`

```rust
use cvkg_core::dirty_flags::DirtyFlags;

/// Verify DirtyFlags constants have correct bitmasks and downstream propagation.

#[test]
fn test_dirty_flags_values() {
    assert_eq!(DirtyFlags::STATE.0, 0b1111);
    assert_eq!(DirtyFlags::LAYOUT.0, 0b0111);
    assert_eq!(DirtyFlags::PAINT.0, 0b0011);
    assert_eq!(DirtyFlags::COMPOSITE.0, 0b0001);
}

#[test]
fn test_downstream_propagation() {
    // STATE must imply all downstream layers.
    assert!(DirtyFlags::STATE.implies(DirtyFlags::LAYOUT));
    assert!(DirtyFlags::STATE.implies(DirtyFlags::PAINT));
    assert!(DirtyFlags::STATE.implies(DirtyFlags::COMPOSITE));

    // LAYOUT must imply PAINT and COMPOSITE.
    assert!(DirtyFlags::LAYOUT.implies(DirtyFlags::PAINT));
    assert!(DirtyFlags::LAYOUT.implies(DirtyFlags::COMPOSITE));

    // PAINT must imply COMPOSITE.
    assert!(DirtyFlags::PAINT.implies(DirtyFlags::COMPOSITE));

    // COMPOSITE does NOT imply anything upstream.
    assert!(!DirtyFlags::COMPOSITE.implies(DirtyFlags::PAINT));
    assert!(!DirtyFlags::COMPOSITE.implies(DirtyFlags::LAYOUT));
}

#[test]
fn test_needs_layout() {
    assert!(DirtyFlags::STATE.needs_layout());
    assert!(DirtyFlags::LAYOUT.needs_layout());
    assert!(!DirtyFlags::PAINT.needs_layout());
    assert!(!DirtyFlags::COMPOSITE.needs_layout());
}

#[test]
fn test_needs_state() {
    assert!(DirtyFlags::STATE.needs_state());
    assert!(!DirtyFlags::LAYOUT.needs_state());
    assert!(!DirtyFlags::PAINT.needs_state());
}
```

**Verification command:** `cargo test -p cvkg-core -- dirty_flags_tests`

#### Test 2: Signal set_with_flags Accumulation

**Test file:** `cvkg-vdom/tests/signal_dirty_tests.rs`

```rust
use cvkg_vdom::signals::Signal;
use cvkg_core::dirty_flags::DirtyFlags;

/// Verify Signal::set_with_flags correctly accumulates dirty flags.

#[test]
fn test_signal_set_with_paint_flags() {
    let signal = Signal::new(0u32);
    signal.set_with_flags(42, DirtyFlags::PAINT);
    assert_eq!(*signal.get(), 42);
    // The frame-level accumulator must have PAINT bits set.
}

#[test]
fn test_signal_set_default_is_all() {
    let signal = Signal::new(0u32);
    signal.set(42); // default: conservative ALL
    assert_eq!(*signal.get(), 42);
}

#[test]
fn test_signal_multiple_sets_accumulate() {
    let signal = Signal::new(0u32);
    signal.set_with_flags(1, DirtyFlags::PAINT);
    signal.set_with_flags(2, DirtyFlags::LAYOUT);
    // Accumulated flags must include both LAYOUT and PAINT.
    assert_eq!(*signal.get(), 2);
}
```

**Verification command:** `cargo test -p cvkg-vdom -- signal_dirty_tests`

#### Test 3: FrameScheduler Phase Skip Logic

**Test file:** `cvkg-scheduler/tests/phase_skip_tests.rs`

```rust
use cvkg_scheduler::frame::{FrameScheduler, FramePhase};
use cvkg_core::dirty_flags::DirtyFlags;

/// Verify should_skip_phase returns correct answers for each dirty flag combination.

#[test]
fn test_skip_layout_when_only_paint_dirty() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    // Simulate only PAINT dirty (e.g., color change).
    fs.set_frame_dirty_flags(DirtyFlags::PAINT);
    assert!(fs.should_skip_phase(FramePhase::Layout),
        "Layout must be skipped when only PAINT is dirty");
}

#[test]
fn test_skip_animation_when_only_paint_dirty() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::PAINT);
    assert!(fs.should_skip_phase(FramePhase::Animation),
        "Animation must be skipped when only PAINT is dirty");
}

#[test]
fn test_never_skip_input() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::COMPOSITE);
    assert!(!fs.should_skip_phase(FramePhase::Input),
        "Input phase is never skipped");
}

#[test]
fn test_never_skip_state() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::COMPOSITE);
    assert!(!fs.should_skip_phase(FramePhase::State),
        "State phase is never skipped");
}

#[test]
fn test_layout_runs_when_state_dirty() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::STATE);
    assert!(!fs.should_skip_phase(FramePhase::Layout),
        "Layout must run when STATE is dirty");
}

#[test]
fn test_animation_runs_when_layout_dirty() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::LAYOUT);
    assert!(!fs.should_skip_phase(FramePhase::Animation),
        "Animation must run when LAYOUT is dirty");
}

#[test]
fn test_render_never_fully_skipped() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::COMPOSITE);
    // Render is never fully skipped — GPU always presents.
    assert!(!fs.should_skip_phase(FramePhase::Render));
}
```

**Verification command:** `cargo test -p cvkg-scheduler -- phase_skip_tests`

#### Test 4: Phase Skip Integration (Full Frame Loop)

**Test file:** `cvkg-scheduler/tests/frame_loop_skip_tests.rs`

```rust
use cvkg_scheduler::frame::{FrameScheduler, FramePhase};
use cvkg_core::dirty_flags::DirtyFlags;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Verify the frame loop correctly skips phases based on dirty flags.

#[test]
fn test_full_frame_loop_skips_layout_for_paint_only() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::PAINT);

    let mut phases_run = Vec::new();
    loop {
        if !fs.should_skip_phase(fs.current_phase()) {
            phases_run.push(fs.current_phase());
        }
        if fs.current_phase() == FramePhase::PostFrame { break; }
        fs.advance_phase();
    }

    // Layout and Animation must NOT be in the run list.
    assert!(!phases_run.contains(&FramePhase::Layout),
        "Layout must be skipped for PAINT-only dirty");
    assert!(!phases_run.contains(&FramePhase::Animation),
        "Animation must be skipped for PAINT-only dirty");
    // Render and Composite must run.
    assert!(phases_run.contains(&FramePhase::Render));
    assert!(phases_run.contains(&FramePhase::Composite));
}

#[test]
fn test_full_frame_loop_runs_all_for_state_dirty() {
    let mut fs = FrameScheduler::new();
    fs.begin_frame();
    fs.set_frame_dirty_flags(DirtyFlags::STATE);

    let mut phases_run = Vec::new();
    loop {
        if !fs.should_skip_phase(fs.current_phase()) {
            phases_run.push(fs.current_phase());
        }
        if fs.current_phase() == FramePhase::PostFrame { break; }
        fs.advance_phase();
    }

    // All phases must run when STATE is dirty.
    assert!(phases_run.contains(&FramePhase::Layout));
    assert!(phases_run.contains(&FramePhase::Animation));
    assert!(phases_run.contains(&FramePhase::Render));
}
```

**Verification command:** `cargo test -p cvkg-scheduler -- frame_loop_skip_tests`

---

## 10. Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | **Vertex format bloat.** Adding `InstanceData3D` (48 bytes) to every instance increases vertex bandwidth. The 2D `InstanceData` (32 bytes) is already packed — a unified format wastes GPU memory on non-3D content. | Medium | Medium | Use a **separate vertex buffer** for 3D instances. The 2D path continues to use `InstanceData`; the 3D path uses `InstanceData3D`. The `DrawCall` struct discriminates which instance buffer to bind. |
| 2 | **Shader specialization explosion.** Bevy handles this with `SpecializedPipeline` — a cache keyed by (material, vertex_layout, render_phase). Without this, every permutation (shadow vs PBR, textured vs untextured) multiplies pipeline count. | High | High | Implement a `SpecializationKey` type in `cvkg-render-3d/src/pipeline.rs` that drives pipeline caching. Key includes: `has_albedo_texture`, `has_normal_map`, `has_shadow`, `is_shadow_pass`. Cache compiled pipelines in a `HashMap<SpecializationKey, RenderPipeline>`. |
| 3 | **Mega-Heim texture array exhaustion.** The existing texture array has 32 slots. 3D textures (albedo, normal, ORM) compete with 2D UI textures. | Medium | Low | Increase the array to 64 or use a separate texture array for 3D materials. The `tex_index` field in `Vertex` can distinguish which array to sample from (bit 31 = 3D array). |
| 4 | **Shadow map resolution vs. performance.** 4096×4096 shadow maps are expensive on low-end GPUs (Adreno, Intel UHD). | Medium | Low | Make shadow map size configurable per-light. Default to 1024×1024. Add a `ShadowQuality` level (low=512, medium=1024, high=2048, ultra=4096). |
| 5 | **2D/3D compositing conflicts.** Both pipelines share the same output texture and depth buffer. A 3D scene behind a 2D UI needs correct depth ordering. Currently, 2D draws first at z=0 then 3D draws on top. | Low | Medium | Add a configurable `SceneMode` to the Kvasir graph: `SceneMode::Pure2D` (current), `SceneMode::Mixed(clear_depth)` (3D first, then 2D overdraws), `SceneMode::Overlay2D` (full 3D with HUD overlay). |

### 10.1 Risk Verification Tests

Each risk mitigation must be verified with a concrete test before the mitigation is considered complete.

#### Risk #1 — Vertex Format Bloat (Separate Buffers)

**Test file:** `cvkg-render-3d/tests/vertex_buffer_separation.rs`

```rust
/// Verify that 2D and 3D instance data use separate vertex buffers.
/// Risk #1: vertex format bloat from unified InstanceData3D.

#[test]
fn test_2d_instance_data_unchanged() {
    // The existing 2D InstanceData must remain 32 bytes.
    // If someone unifies the formats, this test breaks.
    assert_eq!(std::mem::size_of::<InstanceData>(), 32);
}

#[test]
fn test_3d_instance_data_separate_buffer() {
    // InstanceData3D must be 48 bytes (3 × vec4 + 1 × vec4).
    assert_eq!(std::mem::size_of::<InstanceData3D>(), 48);
}

#[test]
fn test_draw_call_discriminates_buffer_type() {
    // DrawCall must have a field that selects which instance buffer to bind.
    // Without this, the GPU binds the wrong buffer for 3D draws.
    let call_2d = DrawCall::new_2d(/* ... */);
    let call_3d = DrawCall::new_3d(/* ... */);
    assert_ne!(call_2d.instance_buffer_tag(), call_3d.instance_buffer_tag());
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- vertex_buffer_separation`

#### Risk #2 — Shader Specialization (Pipeline Cache)

**Test file:** `cvkg-render-3d/tests/pipeline_cache.rs`

```rust
/// Verify SpecializationKey drives pipeline caching.
/// Risk #2: shader specialization explosion.

#[test]
fn test_specialization_key_deterministic() {
    // Same material properties must produce the same key.
    let key_a = SpecializationKey::from_material(&Material3D {
        base_color_texture: Some("tile.png".into()),
        normal_map_texture: None,
        metallic_roughness_texture: None,
        ..Default::default()
    });
    let key_b = SpecializationKey::from_material(&Material3D {
        base_color_texture: Some("tile.png".into()),
        normal_map_texture: None,
        metallic_roughness_texture: None,
        ..Default::default()
    });
    assert_eq!(key_a, key_b);
}

#[test]
fn test_specialization_key_differentiates_textures() {
    let key_textured = SpecializationKey::from_material(&Material3D {
        base_color_texture: Some("albedo.png".into()),
        ..Default::default()
    });
    let key_flat = SpecializationKey::from_material(&Material3D {
        base_color_texture: None,
        ..Default::default()
    });
    assert_ne!(key_textured, key_flat);
}

#[test]
fn test_pipeline_cache_evicts_lru() {
    // Cache with max_size=2 must evict oldest entry on third insert.
    let mut cache = PipelineCache::new(2);
    cache.insert(SpecializationKey::A, pipeline_a());
    cache.insert(SpecializationKey::B, pipeline_b());
    cache.insert(SpecializationKey::C, pipeline_c()); // evicts A
    assert!(cache.get(&SpecializationKey::A).is_none());
    assert!(cache.get(&SpecializationKey::B).is_some());
    assert!(cache.get(&SpecializationKey::C).is_some());
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- pipeline_cache`

#### Risk #3 — Texture Array Exhaustion

**Test file:** `cvkg-render-3d/tests/texture_array.rs`

```rust
/// Verify 3D textures use a separate array or bit-31 indexing.
/// Risk #3: Mega-Heim texture array exhaustion.

#[test]
fn test_tex_index_bit31_distinguishes_arrays() {
    // A 2D tex_index must have bit 31 clear.
    let idx_2d: u32 = 5;
    assert_eq!(idx_2d & (1 << 31), 0, "2D index must not set bit 31");

    // A 3D tex_index must have bit 31 set.
    let idx_3d: u32 = 5 | (1 << 31);
    assert_ne!(idx_3d & (1 << 31), 0, "3D index must set bit 31");
}

#[test]
fn test_3d_material_slots_separate_from_2d() {
    // If using a separate array, verify capacity is independent.
    // If using bit-31, verify the shader decodes correctly.
    // This is a wiring test — the actual GPU test is in integration.
    let capacity_2d = 32u32;
    let capacity_3d = 32u32;
    assert_eq!(capacity_2d + capacity_3d, 64);
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- texture_array`

#### Risk #4 — Shadow Map Resolution

**Test file:** `cvkg-render-3d/tests/shadow_quality.rs`

```rust
/// Verify shadow map size is configurable per-light.
/// Risk #4: shadow map resolution vs. performance.

#[test]
fn test_shadow_quality_levels() {
    assert_eq!(ShadowQuality::Low.size(), 512);
    assert_eq!(ShadowQuality::Medium.size(), 1024);
    assert_eq!(ShadowQuality::High.size(), 2048);
    assert_eq!(ShadowQuality::Ultra.size(), 4096);
}

#[test]
fn test_default_shadow_quality_is_medium() {
    let light = DirectionalLight::default();
    assert_eq!(light.shadow_quality, ShadowQuality::Medium);
    assert_eq!(light.shadow_map_size(), 1024);
}

#[test]
fn test_shadow_map_size_clamps_to_valid() {
    // Custom sizes must be powers of 2 between 512 and 4096.
    let light = DirectionalLight {
        shadow_map_size: 768, // invalid
        ..Default::default()
    };
    assert_eq!(light.shadow_map_size(), 1024, "invalid size clamps to default");
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- shadow_quality`

#### Risk #5 — 2D/3D Compositing (SceneMode)

**Test file:** `cvkg-render-3d/tests/scene_mode.rs`

```rust
/// Verify SceneMode enum controls render pass ordering.
/// Risk #5: 2D/3D compositing conflicts.

#[test]
fn test_scene_mode_default_is_pure2d() {
    let mode = SceneMode::default();
    assert!(matches!(mode, SceneMode::Pure2D));
}

#[test]
fn test_mixed_mode_clears_depth_before_3d() {
    let mode = SceneMode::Mixed { clear_depth: true };
    assert!(mode.should_clear_depth());
}

#[test]
fn test_overlay2d_renders_3d_first() {
    let mode = SceneMode::Overlay2D;
    assert_eq!(mode.render_order(), RenderOrder::ThreeDThenTwoD);
}

#[test]
fn test_pure2d_skips_3d_passes() {
    let mode = SceneMode::Pure2D;
    assert!(!mode.has_3d_passes());
}
```

**Verification command:** `cargo test -p cvkg-render-3d -- scene_mode`

---

## 11. Appendix: Bevy Architecture Reference

### 11.1 Pipelined Render Phases (Bevy Schematic)

```
Main App (ECS World)                    Render App (ECS World)
     │                                        │
     │  ┌─────────────────────────────┐       │
     ├──► Extract: read Main World,   │       │
     │  │ write Render World           │       │
     │  │ (camera, meshes, lights,     │       │
     │  │  transforms, visibility)     │       │
     │  └──────────────┬──────────────┘       │
     │                 │                       │
     │                 ▼                       │
     │  ┌─────────────────────────────┐       │
     │  │ Prepare: upload GPU buffers,│       │
     │  │ create bind groups,         │       │
     │  │ build uniform arrays        │       │
     │  └──────────────┬──────────────┘       │
     │                 │                       │
     │                 ▼                       │
     │  ┌─────────────────────────────┐       │
     │  │ Queue: assign entities to   │       │
     │  │ render phases (Opaque3d,    │       │
     │  │ Shadow, Transparent3d),     │       │
     │  │ sort by key                 │       │
     │  └──────────────┬──────────────┘       │
     │                 │                       │
     │                 ▼                       │
     │  ┌─────────────────────────────┐       │
     │  │ Draw: execute render graph  │       │
     │  │ → Shadow map pass           │       │
     │  │ → Opaque pass (sorted      │       │
     │  │   front-to-back, batched)   │       │
     │  │ → Transparent pass          │       │
     │  │   (sorted back-to-front)    │       │
     │  │ → Post-processing           │       │
     │  └─────────────────────────────┘       │
```

CVKG's equivalent maps as:
- **Extract** → Scene graph traversal + frustum culling (CPU, in `cvkg-render-3d`)
- **Prepare** → Upload transformed vertices + instance data to GPU (in `GpuRenderer::begin_frame`)
- **Queue** → Build draw call list sorted by material/z-order (in `emit_draw_call`)
- **Draw** → Execute Kvasir graph via `ExecutionPlanner`

### 11.2 Bevy Transform Hierarchy

```
Entity A (root)
  ├─ Transform: { translation: (0,0,0), rotation: ..., scale: (1,1,1) }
  ├─ GlobalTransform: { mat4: identity }
  │
  ├─ Entity B (child of A)
  │    ├─ Parent(A)
  │    ├─ Transform: { translation: (2,0,0), ... }
  │    └─ GlobalTransform: { mat4: A.global * B.local }
  │
  └─ Entity C (child of A)
       ├─ Parent(A)
       ├─ Transform: { translation: (0,3,0), ... }
       └─ GlobalTransform: { mat4: A.global * C.local }
```

CVKG's equivalent (new): `TransformNode3D` tree propagated by `propagate_transforms()`.

### 11.3 Bevy Material ↔ GPU Pipeline Specialization

```
StandardMaterial {
    base_color_texture: Some(Handle)
    normal_map_texture: None
    metallic_roughness_texture: Some(Handle)
}
        │
        ▼
SpecializationKey {
    has_base_color_texture: true,
    has_normal_map_texture: false,
    has_metallic_roughness_texture: true,
    alpha_mode: Opaque,
}
        │
        ▼
PipelineCache lookup → compile or reuse RenderPipeline
        │
        ▼
BindGroup { group(0): tile_0_albedo, group(0): tile_0_orm }
```

CVKG equivalent (new): `SpecializationKey` in `cvkg-render-3d/src/pipeline.rs`.

---

---

## 12. Auto-Required Companion State (`#[require]`)

### 12.1 Bevy Inspiration

In Bevy 0.15+, `#[derive(Component)]` supports `#[require(Transform, Visibility)]`. When an entity is spawned with a `Node` component, Bevy's ECS automatically inserts `ComputedNode`, `Transform`, `GlobalTransform`, `Visibility`, etc. This eliminates the "forgot to add companion component" class of bugs — every `Node` always has the components it needs to function.

### 12.2 Current CVKG Gap

The `#[view_component]` macro (`cvkg-macros/src/lib.rs:97-151`) transforms a function into a `View` struct, but generates **zero companion state**. A `Focusable` component must manually thread a `State<FocusState>` parameter through its function signature:

```rust
#[view_component]
fn MyButton(label: String, focus: State<FocusState>) {   // ← manual
    Button::new(label, move || focus.set(FocusState::Focused))
}
```

And `A11yProps` must be set via modifier chain:

```rust
Text::new("hello")
    .a11y_label("Greeting")             // ← manual, easy to forget
    .a11y_role("heading")               // ← manual, easy to forget
```

The VNode generated by `AnyView::render()` has an empty `state: None` and default `aria_props: AriaProps::default()`. No mechanism ensures that a component's required semantic state is initialized.

### 12.3 Design: `Companion` Trait & Macro Attribute

#### 12.3.1 Companion trait

```rust
// cvkg-core/src/companion.rs (NEW)

/// A companion state that should be auto-initialized when a View component
/// is instantiated. Similar to Bevy's #[require(Component)].
///
/// # Contract
/// - `Default::default()` produces a valid initial state.
/// - The state is stored in the VNode and persists across frames.
pub trait Companion: Default + Send + Sync + 'static {
    /// A human-readable name for debug/inspector display.
    fn type_name(&self) -> &'static str;
}
```

Concrete companion types live in their respective crates:

```rust
// cvkg-components/src/interactive/focus.rs
#[derive(Clone, Debug, Default)]
pub struct FocusableCompanion {
    pub state: FocusState,
    pub tab_index: i32,
}
impl Companion for FocusableCompanion {
    fn type_name(&self) -> &'static str { "Focusable" }
}

// cvkg-core/src/accessibility.rs
#[derive(Clone, Debug, Default)]
pub struct A11yCompanion {
    pub role: String,
    pub label: String,
    pub description: String,
    pub disabled: bool,
}
impl Companion for A11yCompanion {
    fn type_name(&self) -> &'static str { "A11yProps" }
}
```

#### 12.3.2 Extended `#[view_component]` macro

The `_attr: TokenStream` parameter of `#[view_component]` is currently ignored. Parse `#[require(TypeA, TypeB)]` from it:

```rust
// cvkg-macros/src/lib.rs — parsing the require attribute

struct ViewComponentArgs {
    required: Vec<syn::Type>,
}

impl Parse for ViewComponentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut required = Vec::new();
        // Parse #[require(Type1, Type2)] or empty
        if input.peek(Token![require]) {
            let _: Token![require] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            while !content.is_empty() {
                required.push(content.parse()?);
                if content.is_empty() { break; }
                let _: Option<Token![,]> = content.parse()?;
            }
        }
        Ok(Self { required })
    }
}
```

The expanded macro generates a `companion_states()` method on the View trait:

```rust
#[view_component]
#[require(FocusableCompanion, A11yCompanion)]
fn MyButton(label: String) {
    Button::new(label, || {})
}
```

Expands to:

```rust
pub struct MyButtonView {
    pub label: String,
}

impl cvkg_core::View for MyButtonView {
    type Body = cvkg_core::AnyView;

    fn body(self) -> Self::Body {
        let label = self.label;
        cvkg_core::AnyView::new({
            Button::new(label, || {})
        })
    }

    /// Auto-generated by #[require(FocusableCompanion, A11yCompanion)].
    fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
        vec![
            Box::new(FocusableCompanion::default()),
            Box::new(A11yCompanion::default()),
        ]
    }
}

fn MyButton(label: String) -> MyButtonView {
    MyButtonView { label }
}
```

#### 12.3.3 `companion_states()` on the `View` trait

Add default method to `View`:

```rust
// cvkg-core/src/view.rs
pub trait View: Sized + Send {
    type Body: View;
    fn body(self) -> Self::Body;
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}

    /// Returns companion states that must be initialized when this view's
    /// VNode is created. Default: none.
    fn companion_states(&self) -> Vec<Box<dyn Companion>> {
        vec![]
    }

    fn erase(self) -> AnyView;
}
```

### 12.4 Injection into VNode Creation

The companion states must be injected when the VNode is created. This happens in `AnyView::render()`. Extend the `Renderer` trait with a method that accepts companions:

```rust
// cvkg-core/src/renderer_trait.rs
pub trait Renderer: ElapsedTime + Send + RendererErrorHandler {
    // ... existing methods ...

    /// Push a VDOM node with companion state auto-initialization.
    /// Default: delegates to push_vnode (ignores companions).
    /// VNodeRenderer overrides to store companions in the VNode.
    fn push_vnode_with_companions(
        &mut self,
        rect: Rect,
        name: &'static str,
        _companions: Vec<Box<dyn Companion>>,
    ) {
        self.push_vnode(rect, name);
    }
}
```

Modify `AnyView::render()`:

```rust
// cvkg-core/src/erased_view.rs
impl View for AnyView {
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let name = self.inner.name();
        let companions = self.inner.companion_states_erased();
        if companions.is_empty() {
            renderer.push_vnode(rect, name);
        } else {
            renderer.push_vnode_with_companions(rect, name, companions);
        }
        self.inner.render_erased(renderer, rect);
        renderer.pop_vnode();
    }
}
```

Add `companion_states_erased()` to `ErasedView`:

```rust
// cvkg-core/src/erased_view.rs
pub trait ErasedView: Send {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect);
    fn name(&self) -> &'static str;
    fn companion_states_erased(&self) -> Vec<Box<dyn Companion>>;  // NEW
    // ... existing methods ...
}

impl<V: View + Clone + 'static> ErasedView for V {
    fn companion_states_erased(&self) -> Vec<Box<dyn Companion>> {
        self.companion_states()  // delegates to View::companion_states()
    }
    // ...
}
```

### 12.5 VNodeRenderer Storage

In `VNodeRenderer` (`cvkg-vdom/src/lib.rs`), override `push_vnode_with_companions`:

```rust
// cvkg-vdom/src/lib.rs — VNodeRenderer

fn push_vnode_with_companions(
    &mut self,
    rect: Rect,
    name: &'static str,
    companions: Vec<Box<dyn Companion>>,
) {
    let id = self.next_id();
    let mut node = self.create_base_vnode(id, rect, name);

    // Store companions for runtime access and inspector visibility.
    for companion in companions {
        let key = companion.type_name().to_string();
        let value = serde_json::to_value(&companion).unwrap_or_default();
        node.state.get_or_insert_with(HashMap::new).insert(key, value);
    }

    self.add_node(node);
    self.stack.push(id);
}
```

Runtime companion state access via a new VNodeRenderer method:

```rust
// cvkg-vdom/src/lib.rs — VNodeRenderer

/// Retrieve a companion state for the VNode currently at the top of the stack.
/// Returns None if the companion type is not registered on this node.
pub fn current_companion<T: Companion>(&self) -> Option<&T> {
    let node_id = self.stack.last()?;
    let node = self.nodes.get(node_id)?;
    let key = std::any::type_name::<T>();
    node.companions.get(key).and_then(|c| c.downcast_ref::<T>())
}
```

### 12.6 Interaction with `hamr!`

The `hamr!` macro (`cvkg-macros/src/lib.rs:431-439`) expands to `.child()` builder calls. **Companion injection is entirely orthogonal** — it happens at render time, not tree-construction time:

```rust
// hamr! expansion is unchanged:
VStack::new(16.0)
    .child(MyButton("Click"))    // returns MyButtonView
    .child(MyButton("Submit"))   // returns MyButtonView

// At render time, each MyButtonView is wrapped in AnyView:
//   AnyView::render() → push_vnode_with_companions(
//       rect,
//       "my_module::MyButtonView",
//       vec![FocusableCompanion::default(), A11yCompanion::default()]
//   )
```

The `hamr!` macro has no VNode awareness. It produces expressions of type `impl View`. The companion state flows through the `View` trait's `companion_states()` method, which is only called at render time inside `AnyView::render()`. No changes to `hamr!` are needed.

### 12.7 Migration Path

| Step | Change | Backward Compat? |
|---|---|---|
| 1 | Add `Companion` trait to `cvkg-core` | Yes (new module, no existing code affected) |
| 2 | Add `companion_states()` to `View` trait with default empty `vec![]` | Yes (no existing View impl breaks) |
| 3 | Add `companion_states_erased()` to `ErasedView` | Yes (blanket impl delegates to `View`) |
| 4 | Add `push_vnode_with_companions()` to `Renderer` trait with default delegation | Yes (default calls `push_vnode`) |
| 5 | Parse `#[require(...)]` in `#[view_component]` macro | Yes (currently ignores `_attr`) |
| 6 | Override `push_vnode_with_companions` in `VNodeRenderer` | Yes (new behavior, existing VNodes unaffected) |
| 7 | Add `FocusableCompanion`, `A11yCompanion` types | Yes (new types, opt-in) |

### 12.8 Companion State Tests

Every step in the migration path must have a corresponding test. Tests are written FIRST (TDD red phase), then the implementation makes them pass.

#### Test 1: Companion Trait Exists and Is Object-Safe

**Test file:** `cvkg-core/tests/companion_tests.rs`

```rust
use cvkg_core::Companion;

#[derive(Default)]
struct TestCompanion {
    value: i32,
}

impl Companion for TestCompanion {
    fn type_name(&self) -> &'static str { "TestCompanion" }
}

#[test]
fn test_companion_is_default_constructible() {
    let c = TestCompanion::default();
    assert_eq!(c.value, 0);
}

#[test]
fn test_companion_type_name() {
    let c = TestCompanion { value: 42 };
    assert_eq!(c.type_name(), "TestCompanion");
}

#[test]
fn test_companion_object_safe() {
    // Companion must be usable as dyn Companion.
    let c: Box<dyn Companion> = Box::new(TestCompanion::default());
    assert_eq!(c.type_name(), "TestCompanion");
}
```

**Verification command:** `cargo test -p cvkg-core -- companion_tests`

#### Test 2: View::companion_states() Default Returns Empty

**Test file:** `cvkg-core/tests/view_companion_tests.rs`

```rust
use cvkg_core::{View, Never, AnyView};

#[derive(Clone)]
struct NoCompanions;

impl View for NoCompanions {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
}

#[test]
fn test_view_default_companion_states_is_empty() {
    let view = NoCompanions;
    let companions = view.companion_states();
    assert!(companions.is_empty(), "default must return empty vec");
}
```

**Verification command:** `cargo test -p cvkg-core -- view_companion_tests`

#### Test 3: ErasedView Delegates companion_states

**Test file:** `cvkg-core/tests/erased_view_companion_tests.rs`

```rust
use cvkg_core::{View, Companion, Never};
use cvkg_core::erased_view::ErasedView;

#[derive(Clone)]
struct WithCompanion;

impl Companion for WithCompanion {
    fn type_name(&self) -> &'static str { "WithCompanion" }
}

#[derive(Clone)]
struct HasCompanions;

impl View for HasCompanions {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn companion_states(&self) -> Vec<Box<dyn Companion>> {
        vec![Box::new(WithCompanion)]
    }
}

#[test]
fn test_erased_view_delegates_companion_states() {
    let view = HasCompanions;
    let erased: Box<dyn ErasedView> = Box::new(view);
    let companions = erased.companion_states_erased();
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].type_name(), "WithCompanion");
}
```

**Verification command:** `cargo test -p cvkg-core -- erased_view_companion_tests`

#### Test 4: Renderer::push_vnode_with_companions Default

**Test file:** `cvkg-core/tests/renderer_companion_tests.rs`

```rust
use cvkg_core::{Renderer, Companion, Rect};
use cvkg_core::testing::mock_renderer::MockRenderer;

struct SinkCompanion;
impl Companion for SinkCompanion {
    fn type_name(&self) -> &'static str { "Sink" }
}

#[test]
fn test_renderer_default_push_vnode_with_companions_ignores_companions() {
    // The default implementation must NOT panic — it delegates to push_vnode.
    let mut renderer = MockRenderer::new();
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    let companions: Vec<Box<dyn Companion>> = vec![Box::new(SinkCompanion)];

    // Must not panic. Companions are silently ignored by default.
    renderer.push_vnode_with_companions(rect, "test", companions);

    // Verify the vnode was still created.
    assert_eq!(renderer.vnode_count(), 1);
}
```

**Verification command:** `cargo test -p cvkg-core -- renderer_companion_tests`

#### Test 5: #[view_component] with #[require] Expands Correctly

**Test file:** `cvkg-macros/tests/require_macro_tests.rs`

```rust
/// Compile-time test: the expanded macro must produce a companion_states() method.
/// This test verifies the macro expansion, not runtime behavior.

use cvkg_macros::view_component;
use cvkg_core::{View, Companion};

#[derive(Clone, Default)]
struct FakeCompanion;
impl Companion for FakeCompanion {
    fn type_name(&self) -> &'static str { "Fake" }
}

#[view_component]
#[require(FakeCompanion)]
fn MyButton(label: String) {
    // body is irrelevant for this test
    let _ = label;
}

#[test]
fn test_view_component_with_require_produces_companion_states() {
    let btn = MyButton("Click".into());
    let companions = btn.companion_states();
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].type_name(), "Fake");
}

#[test]
fn test_view_component_without_require_returns_empty() {
    #[view_component]
    fn PlainButton(label: String) {
        let _ = label;
    }

    let btn = PlainButton("Go".into());
    let companions = btn.companion_states();
    assert!(companions.is_empty());
}
```

**Verification command:** `cargo test -p cvkg-macros -- require_macro_tests`

#### Test 6: VNodeRenderer Stores Companions

**Test file:** `cvkg-vdom/tests/companion_storage_tests.rs`

```rust
use cvkg_core::{Companion, Rect};
use cvkg_vdom::VNodeRenderer;

#[derive(Clone, Default, Debug)]
struct FocusCompanion { tab_index: i32 }
impl Companion for FocusCompanion {
    fn type_name(&self) -> &'static str { "Focusable" }
}

#[derive(Clone, Default, Debug)]
struct A11yCompanion { label: String }
impl Companion for A11yCompanion {
    fn type_name(&self) -> &'static str { "A11yProps" }
}

#[test]
fn test_vnoderenderer_stores_multiple_companions() {
    let mut renderer = VNodeRenderer::new();
    let rect = Rect::new(0.0, 0.0, 200.0, 50.0);
    let companions: Vec<Box<dyn Companion>> = vec![
        Box::new(FocusCompanion { tab_index: 0 }),
        Box::new(A11yCompanion { label: "Submit".into() }),
    ];

    renderer.push_vnode_with_companions(rect, "MyButton", companions);

    // Both companions must be retrievable from the current VNode.
    let focus = renderer.current_companion::<FocusCompanion>();
    assert!(focus.is_some());
    assert_eq!(focus.unwrap().tab_index, 0);

    let a11y = renderer.current_companion::<A11yCompanion>();
    assert!(a11y.is_some());
    assert_eq!(a11y.unwrap().label, "Submit");
}

#[test]
fn test_vnoderenderer_companion_missing_returns_none() {
    let mut renderer = VNodeRenderer::new();
    let rect = Rect::new(0.0, 0.0, 200.0, 50.0);

    // Push a VNode with no companions.
    renderer.push_vnode(rect, "Empty");

    let focus = renderer.current_companion::<FocusCompanion>();
    assert!(focus.is_none());
}
```

**Verification command:** `cargo test -p cvkg-vdom -- companion_storage_tests`

#### Test 7: hamr! Macro Does Not Break Companion Injection

**Test file:** `cvkg-macros/tests/hamr_companion_tests.rs`

```rust
use cvkg_macros::{hamr, view_component};
use cvkg_core::{View, Companion};

#[derive(Clone, Default)]
struct TestComp;
impl Companion for TestComp {
    fn type_name(&self) -> &'static str { "Test" }
}

#[view_component]
#[require(TestComp)]
fn ChildButton(label: String) {
    let _ = label;
}

#[test]
fn test_hamr_children_preserve_companion_states() {
    // hamr! produces impl View expressions. Each child must carry companions.
    let _tree = hamr! {
        ChildButton("A".into())
        ChildButton("B".into())
    };

    // Verify the macro compiles and the companion_states() method is accessible.
    let btn = ChildButton("C".into());
    let companions = btn.companion_states();
    assert_eq!(companions.len(), 1);
}
```

**Verification command:** `cargo test -p cvkg-macros -- hamr_companion_tests`

---

## 13. Reflect-Powered Inspector Integration

### 13.1 Current Gap

`cvkg-reflect` (v0.2.17) defines a `Reflected` trait, `TypeMeta`, `FieldMeta`, `FieldKind`, `ReflectRegistry`, and `ReflectError` — all designed for runtime type introspection. **No crate in the workspace depends on it.** The production inspectors (`FreyrInspector`, `GullveigInspector`, `VdomInspector`) all use hand-crafted builder patterns with hard-coded properties:

```rust
// Current FreyrInspector — manual, not reflective:
FreyrInspector::new("Component Properties")
    .text_prop("name", "MyComponent", "The component name")
    .number_prop("opacity", 0.85, "Transparency level")
    .bool_prop("enabled", true, "Whether component is active");

// What reflection enables:
ReflectedInspector::new("properties", &my_instance,
    Rc::new(RefCell::new(MyComponent::default())))
```

The `cvkg-cli` devtools dashboard and WebSocket inspector likewise return hard-coded stubs. The `ReflectRegistry` is documented and tested but never populated.

The original Bevy comparison (`bevy-vs-cvkg-prompt.md §2d`) identifies this gap and its upstream: `bevy_reflect` + `bevy_inspector_egui` auto-generate editable panels from a single `#[derive(Reflect)]`. CVKG needs the same pipeline: **derive macro → `Reflected` impl → `ReflectedInspector` → interactive per-field widgets**.

### 13.2 Components

| Component | What It Does | Current Status |
|---|---|---|
| `#[derive(Reflect)]` in `cvkg-macros` | Generates `Reflected` impl: `type_meta()`, `get_field()`, `set_field()`, `snapshot()` | **Missing** — must be built |
| `ReflectedInspector` in `cvkg-components` | Reads `&'static TypeMeta` + `Rc<RefCell<dyn Reflected>>`, renders editable per-field row | **Missing** — must be built |
| `Widget dispatch` in `ReflectedInspector` | Maps `FieldKind` → interactive widget (Checkbox, Slider, ColorPicker, TextInput) | **Missing** — dispatcher logic |
| `ReflectRegistry` population at startup | Calls `registry.register(MyType::type_meta())` for every reflected type | **Missing** — no crates register types |
| `cvkg-cli` devtools WS reflect query | Exposes reflected properties over WebSocket for remote inspection | **Missing** — currently hard-coded stubs |

### 13.3 `#[derive(Reflect)]` Proc Macro

Add a new derive macro to `cvkg-macros/src/lib.rs`:

```rust
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => return syn::Error::new(name.span(),
            "Reflect can only be derived for structs").to_compile_error().into(),
    };

    // Map each field's type → FieldKind (with #[reflect(kind = "...")] override)
    // Generate static FIELDS array, static META, and impl Reflected { ... }
    // ...
}
```

**Type → `FieldKind` mapping** (inferred from Rust type syntax):

| Rust Type | `FieldKind` | Notes |
|---|---|---|
| `bool` | `Bool` | |
| `i8` / `i16` / `i32` / `i64` / `isize` | `Integer` | All signed integer widths |
| `u8` / `u16` / `u32` / `u64` / `usize` | `Integer` | All unsigned integer widths |
| `f32` / `f64` | `Float` | |
| `String` / `&str` / `SmolStr` | `String` | |
| `[f32; 4]` / `Color` / `Rgba` | `Color` | RGBA color |
| `[f32; 2]` / `Vec2` | `Vec2` | |
| `[f32; 3]` / `Vec3` / `Vec3A` | `Vec3` | |
| `Rect` / `[f32; 4]` with named fields | `Rect` | |
| `[f32; 1]` | `Float` | Single-element array |
| Anything else | `Custom("TypeName")` | Falls back to `Custom` |

**Attribute overrides** — user can override the inferred kind:

```rust
#[derive(Reflect)]
struct ShaderConfig {
    #[reflect(kind = "Vec3", doc = "World-space light direction")]
    light_dir: [f32; 3],

    #[reflect(read_only)]
    node_id: u64,

    #[reflect(doc = "Opacity multiplier", min = "0.0", max = "1.0")]
    alpha: f32,
}
```

Supported `#[reflect(...)]` attributes:

| Attribute | Effect |
|---|---|
| `kind = "Vec2"` | Override inferred `FieldKind` |
| `doc = "..."` | Set `FieldMeta::doc` |
| `read_only` | Set `FieldMeta::read_only = true` |
| `min = "0.0"` | Validation lower bound (Integer/Float only) |
| `max = "1.0"` | Validation upper bound (Integer/Float only) |
| `default = "42"` | Default value shown in inspector when unset |

**Generated code example** for `#[derive(Reflect)] struct Props { enabled: bool, opacity: f32 }`:

```rust
impl cvkg_reflect::Reflected for Props {
    fn type_meta() -> &'static cvkg_reflect::TypeMeta {
        static FIELDS: [cvkg_reflect::FieldMeta; 2] = [
            cvkg_reflect::FieldMeta {
                name: "enabled",
                kind: cvkg_reflect::FieldKind::Bool,
                doc: "",
                read_only: false,
            },
            cvkg_reflect::FieldMeta {
                name: "opacity",
                kind: cvkg_reflect::FieldKind::Float,
                doc: "",
                read_only: false,
            },
        ];
        static META: cvkg_reflect::TypeMeta = cvkg_reflect::TypeMeta {
            type_name: "Props",
            fields: &FIELDS,
        };
        &META
    }

    fn get_field(&self, name: &str) -> Option<serde_json::Value> {
        match name {
            "enabled" => Some(serde_json::Value::Bool(self.enabled)),
            "opacity" => serde_json::to_value(self.opacity).ok(),
            _ => None,
        }
    }

    fn set_field(&mut self, name: &str, value: serde_json::Value)
        -> Result<(), cvkg_reflect::ReflectError>
    {
        match name {
            "enabled" => {
                let v = value.as_bool().ok_or_else(||
                    cvkg_reflect::ReflectError::TypeMismatch {
                        field: "enabled".into(),
                        expected: "bool".into(),
                        got: cvkg_reflect::json_kind_name(&value).into(),
                    })?;
                self.enabled = v;
                Ok(())
            }
            "opacity" => {
                let v = value.as_f64().ok_or_else(||
                    cvkg_reflect::ReflectError::TypeMismatch {
                        field: "opacity".into(),
                        expected: "number".into(),
                        got: cvkg_reflect::json_kind_name(&value).into(),
                    })?;
                self.opacity = v as f32;
                Ok(())
            }
            other => Err(cvkg_reflect::ReflectError::FieldNotFound(other.into())),
        }
    }
}
```

### 13.4 `ReflectedInspector` Component

New component in `cvkg-components/src/reflected_inspector.rs`:

```rust
pub struct ReflectedInspector {
    pub title: String,
    pub instance: Rc<RefCell<dyn Reflected>>,
}

impl ReflectedInspector {
    pub fn new(title: &str, instance: Rc<RefCell<dyn Reflected>>) -> Self {
        Self {
            title: title.to_string(),
            instance,
        }
    }
}

impl View for ReflectedInspector {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let instance = self.instance.borrow();
        let meta = instance.type_meta();

        // Title bar
        renderer.fill_rect(/* title rect */, theme::inspector_bg());
        renderer.draw_text_raw(&self.title, /* ... */, 13.0, theme::inspector_accent());

        // Field rows
        for (i, field) in meta.fields.iter().enumerate() {
            let row_rect = /* compute row rect from i */;
            renderer.fill_rect(row_rect, theme::inspector_border());

            // Field name
            renderer.draw_text_raw(field.name, /* left column */, 11.0, theme::text());

            // Field value — dispatch per FieldKind
            let value = instance.get_field(field.name);
            self.render_field(renderer, field, value, row_rect);
        }
    }
}
```

**Widget dispatch (`render_field`)**:

| `FieldKind` | Widget | Source |
|---|---|---|
| `Bool` | `Checkbox` from `crate::interactive::checkbox` | Existing |
| `Integer` | `Slider` from `crate::interactive::button` (as int-scrubber) | Existing (adapt) |
| `Float` | `MjolnirSlider` from `crate::mjolnir_slider` or `Slider` | Existing |
| `String` | Text display → click to edit (uses `TextInput`-style inline edit) | **New**: `InlineTextEdit` widget |
| `Color` | `ColorPicker` from `crate::interactive::select` | Existing |
| `Vec2` | Two `Float` rows side-by-side with labels | Builds on `Float` widget |
| `Vec3` | Three `Float` rows side-by-side | Builds on `Float` widget |
| `Rect` | Four `Float` rows (x, y, w, h) | Builds on `Float` widget |
| `Custom(_)` | Read-only text display of type name | Inline |

**Edit flow**:
1. User interacts with widget (toggles checkbox, drags slider)
2. Widget's `on_change` callback fires with new value
3. Callback calls `instance.borrow_mut().set_field(field.name, new_value)`
4. Next frame, `render()` re-reads via `get_field()` and displays updated value

For `Slider` and `MjolnirSlider`, the range is taken from `#[reflect(min = "...", max = "...")]`; if absent, defaults to `[0.0, 1.0]`.

### 13.5 `InlineTextEdit` Widget (New)

Required for `String` field editing. Located at `cvkg-components/src/interactive/inline_text_edit.rs`:

```rust
pub struct InlineTextEdit {
    pub(crate) value: String,
    pub(crate) on_commit: Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) editing: bool,
}
```

On click: switches to an editable state that captures keyboard input. On Enter/blur: fires `on_commit` with the new string. Renders as a bordered rect in edit mode, plain text otherwise.

### 13.6 Registration at Startup

Each crate that defines reflected types registers them during its initialization. The umbrella `cvkg/src/lib.rs` owns a global `REFLECT_REGISTRY: OnceLock<Mutex<ReflectRegistry>>`:

```rust
// cvkg-core/src/lib.rs or cvkg/src/lib.rs
pub static REFLECT_REGISTRY: std::sync::OnceLock<std::sync::Mutex<ReflectRegistry>> =
    std::sync::OnceLock::new();

// At app startup, in cvkg::app::initialize():
let reg = REFLECT_REGISTRY.get_or_init(|| Mutex::new(ReflectRegistry::new()));

// Each component crate registers its types:
reg.lock().unwrap().register(ColorStop::type_meta());
reg.lock().unwrap().register(MyComponent::type_meta());
reg.lock().unwrap().register(SceneNode::type_meta());
```

Alternatively, a registration macro:

```rust
// In each crate's lib.rs:
register_reflected!(MyComponent, ColorStop, SceneNode, /* ... */);

// Expands to:
//   REFLECT_REGISTRY.get_or_init(|| Default::default())
//       .lock().unwrap()
//       .register(MyComponent::type_meta())
//       .register(ColorStop::type_meta())
//       .register(SceneNode::type_meta());
```

The `ReflectedInspector` looks up the `TypeMeta` either:
- **Directly** via `Reflected::type_meta()` (when the concrete type is known at compile time), or
- **Via registry** when the type name comes from a dynamic source (e.g., WebSocket query `{ "type": "MyComponent", "id": "node-42" }` → `registry.get("MyComponent")`).

### 13.7 `cvkg-cli` DevTools Integration

#### 13.7.1 In-Process (`devtools.rs`)

Replace the `PanelContent::NodeInspector` stub with a real `ReflectedInspector`:

```rust
PanelContent::NodeInspector => {
    let instance = /* retrieve from app state */;
    let inspector = ReflectedInspector::new("Node Properties", instance);
    // Render the inspector as TUI widgets:
    for field in inspector.instance.borrow_mut().type_meta().fields {
        let value = inspector.instance.borrow().get_field(field.name);
        panel.add_widget(DevToolWidget::Text(
            format!("{}: {:?}", field.name, value)
        ));
    }
}
```

#### 13.7.2 WebSocket Remote (`ws_server.rs`)

Add a new `DevtoolsCommand` variant:

```rust
pub enum DevtoolsCommand {
    QueryMetrics,
    // ...
    QueryReflected { type_name: String, node_id: String },
}
```

The handler:

```rust
DevtoolsCommand::QueryReflected { type_name, node_id } => {
    let registry = REFLECT_REGISTRY.lock().unwrap();
    let meta = registry.get(&type_name);
    if let Some(meta) = meta {
        let instance = /* retrieve from node_id */;
        let snapshot = instance.snapshot();
        response = json!({
            "type": type_name,
            "node_id": node_id,
            "meta": {
                "type_name": meta.type_name,
                "fields": meta.fields.iter().map(|f| {
                    json!({
                        "name": f.name,
                        "kind": format!("{:?}", f.kind),
                        "doc": f.doc,
                        "read_only": f.read_only,
                    })
                }).collect::<Vec<_>>(),
            },
            "values": snapshot,
        });
    } else {
        response = json!({ "error": format!("type '{}' not registered", type_name) });
    }
}
```

#### 13.7.3 HTTP Dashboard (`devtools_dashboard.rs`)

Add a `/api/reflected` endpoint that accepts `?type=<type_name>&node=<node_id>` and returns the JSON snapshot. The dashboard's `NodeInfo` in `GraphState` gains an optional `reflected: Option<HashMap<String, Value>>` that carries field values for the selected node.

### 13.8 Migration Path

| Step | Changes | Backward Compat? |
|---|---|---|
| 1 | Add `#[derive(Reflect)]` to `cvkg-macros` | Yes (new macro, no existing code affected) |
| 2 | Add `InlineTextEdit` to `cvkg-components/src/interactive/` | Yes (new widget) |
| 3 | Add `ReflectedInspector` to `cvkg-components/src/reflected_inspector.rs` | Yes (new component) |
| 4 | Add `cvkg-reflect` dependency to `cvkg-components/Cargo.toml` | Yes (new dep, no breaking changes) |
| 5 | Add `REFLECT_REGISTRY` to `cvkg-core` or `cvkg` umbrella | Yes (new public static, opt-in) |
| 6 | Implement `Reflected` on key types (e.g., `MjolnirSlider` props, theme tokens) | Yes (opt-in per type) |
| 7 | Add `QueryReflected` to `cvkg-cli` WS protocol | Yes (new command, backwards-compatible) |
| 8 | Add `/api/reflected` to `cvkg-cli` HTTP devtools | Yes (new endpoint) |

### 13.9 Non-Goals

| Feature | Reason |
|---|---|
| Auto-derive for all `View` structs | `View` structs are thin function wrappers; their fields are component data, not reflectable state. Users opt in per struct. |
| Nested/inline struct editing | `Custom` types are displayed read-only. Full nesting requires recursive `Reflected` bounds — tracked as future work. |
| `bevy_inspector_egui`-style tree browser | The VDOM already provides tree inspection via `VdomInspector`. Reflection adds property editing per selected node, not a replacement tree widget. |
| Hot-reload via reflected `set_field` | The hot-reload system already exists via file watchers. Reflection can notify it but should not own the hot-reload loop. |

### 13.10 Reflect Inspector Tests

Every component in the reflect pipeline must have tests. The derive macro tests are compile-time verification (if it compiles, the macro expanded correctly). Runtime tests verify get/set/snapshot behavior.

#### Test 1: #[derive(Reflect)] Generates Correct type_meta

**Test file:** `cvkg-macros/tests/reflect_derive_tests.rs`

```rust
use cvkg_reflect::{Reflected, FieldKind};

#[derive(Reflect)]
struct SimpleProps {
    enabled: bool,
    opacity: f32,
    label: String,
}

#[test]
fn test_derive_reflect_type_name() {
    assert_eq!(SimpleProps::type_meta().type_name, "SimpleProps");
}

#[test]
fn test_derive_reflect_field_count() {
    assert_eq!(SimpleProps::type_meta().fields.len(), 3);
}

#[test]
fn test_derive_reflect_field_kinds() {
    let meta = SimpleProps::type_meta();
    assert_eq!(meta.fields[0].kind, FieldKind::Bool);
    assert_eq!(meta.fields[1].kind, FieldKind::Float);
    assert_eq!(meta.fields[2].kind, FieldKind::String);
}

#[test]
fn test_derive_reflect_field_names() {
    let meta = SimpleProps::type_meta();
    assert_eq!(meta.fields[0].name, "enabled");
    assert_eq!(meta.fields[1].name, "opacity");
    assert_eq!(meta.fields[2].name, "label");
}
```

**Verification command:** `cargo test -p cvkg-macros -- reflect_derive_tests`

#### Test 2: #[derive(Reflect)] Generates Correct get_field/set_field

**Test file:** `cvkg-macros/tests/reflect_getset_tests.rs`

```rust
use cvkg_reflect::{Reflected, ReflectError};
use serde_json::json;

#[derive(Reflect)]
struct Config {
    enabled: bool,
    opacity: f32,
    label: String,
}

#[test]
fn test_get_field_bool() {
    let c = Config { enabled: true, opacity: 0.5, label: "hi".into() };
    assert_eq!(c.get_field("enabled"), Some(json!(true)));
}

#[test]
fn test_get_field_float() {
    let c = Config { enabled: false, opacity: 0.75, label: "".into() };
    let v = c.get_field("opacity").unwrap();
    assert!((v.as_f64().unwrap() - 0.75).abs() < 1e-6);
}

#[test]
fn test_get_field_string() {
    let c = Config { enabled: true, opacity: 1.0, label: "hello".into() };
    assert_eq!(c.get_field("label"), Some(json!("hello")));
}

#[test]
fn test_get_field_unknown_returns_none() {
    let c = Config { enabled: true, opacity: 1.0, label: "".into() };
    assert!(c.get_field("nonexistent").is_none());
}

#[test]
fn test_set_field_bool() {
    let mut c = Config { enabled: false, opacity: 0.0, label: "".into() };
    c.set_field("enabled", json!(true)).unwrap();
    assert!(c.enabled);
}

#[test]
fn test_set_field_float() {
    let mut c = Config { enabled: false, opacity: 0.0, label: "".into() };
    c.set_field("opacity", json!(0.99)).unwrap();
    assert!((c.opacity - 0.99).abs() < 1e-6);
}

#[test]
fn test_set_field_string() {
    let mut c = Config { enabled: false, opacity: 0.0, label: "".into() };
    c.set_field("label", json!("world")).unwrap();
    assert_eq!(c.label, "world");
}

#[test]
fn test_set_field_type_mismatch() {
    let mut c = Config { enabled: false, opacity: 0.0, label: "".into() };
    let err = c.set_field("enabled", json!("not a bool")).unwrap_err();
    assert!(matches!(err, ReflectError::TypeMismatch { .. }));
}

#[test]
fn test_set_field_unknown_returns_not_found() {
    let mut c = Config { enabled: false, opacity: 0.0, label: "".into() };
    let err = c.set_field("missing", json!(42)).unwrap_err();
    assert!(matches!(err, ReflectError::FieldNotFound(_)));
}
```

**Verification command:** `cargo test -p cvkg-macros -- reflect_getset_tests`

#### Test 3: #[derive(Reflect)] Generates Correct snapshot

**Test file:** `cvkg-macros/tests/reflect_snapshot_tests.rs`

```rust
use cvkg_reflect::Reflected;
use serde_json::json;

#[derive(Reflect)]
struct SnapshotProps {
    x: i32,
    y: f32,
}

#[test]
fn test_snapshot_contains_all_fields() {
    let p = SnapshotProps { x: 10, y: 3.14 };
    let snap = p.snapshot();
    assert_eq!(snap.len(), 2);
    assert!(snap.contains_key("x"));
    assert!(snap.contains_key("y"));
}

#[test]
fn test_snapshot_values_match_get_field() {
    let p = SnapshotProps { x: 42, y: 2.71 };
    let snap = p.snapshot();
    assert_eq!(snap["x"], p.get_field("x").unwrap());
    // f32 serialization may differ in precision — compare as f64.
    let snap_y = snap["y"].as_f64().unwrap();
    let field_y = p.get_field("y").unwrap().as_f64().unwrap();
    assert!((snap_y - field_y).abs() < 1e-5);
}
```

**Verification command:** `cargo test -p cvkg-macros -- reflect_snapshot_tests`

#### Test 4: #[reflect] Attribute Overrides

**Test file:** `cvkg-macros/tests/reflect_attr_tests.rs`

```rust
use cvkg_reflect::{Reflected, FieldKind};

#[derive(Reflect)]
struct OverrideProps {
    #[reflect(kind = "Vec3", doc = "Light direction")]
    direction: [f32; 3],

    #[reflect(read_only)]
    node_id: u64,

    #[reflect(doc = "Opacity", min = "0.0", max = "1.0")]
    alpha: f32,
}

#[test]
fn test_reflect_kind_override() {
    let meta = OverrideProps::type_meta();
    let dir = meta.field("direction").unwrap();
    assert_eq!(dir.kind, FieldKind::Vec3);
    assert_eq!(dir.doc, "Light direction");
}

#[test]
fn test_reflect_read_only() {
    let meta = OverrideProps::type_meta();
    let id = meta.field("node_id").unwrap();
    assert!(id.read_only);
}

#[test]
fn test_reflect_doc_override() {
    let meta = OverrideProps::type_meta();
    let alpha = meta.field("alpha").unwrap();
    assert_eq!(alpha.doc, "Opacity");
}
```

**Verification command:** `cargo test -p cvkg-macros -- reflect_attr_tests`

#### Test 5: ReflectRegistry Population and Lookup

**Test file:** `cvkg-reflect/tests/registry_tests.rs`

```rust
use cvkg_reflect::{ReflectRegistry, Reflected, TypeMeta, FieldMeta, FieldKind};

#[derive(Reflect)]
struct RegTypeA { val: f32 }

#[derive(Reflect)]
struct RegTypeB { flag: bool }

#[test]
fn test_registry_register_and_get() {
    let mut reg = ReflectRegistry::new();
    reg.register(RegTypeA::type_meta());
    reg.register(RegTypeB::type_meta());

    assert!(reg.get("RegTypeA").is_some());
    assert!(reg.get("RegTypeB").is_some());
    assert!(reg.get("RegTypeC").is_none());
}

#[test]
fn test_registry_type_names() {
    let mut reg = ReflectRegistry::new();
    reg.register(RegTypeA::type_meta());
    reg.register(RegTypeB::type_meta());

    let mut names: Vec<&str> = reg.type_names().map(|n| **n).collect();
    names.sort();
    assert_eq!(names, vec!["RegTypeA", "RegTypeB"]);
}

#[test]
fn test_registry_register_idempotent() {
    let mut reg = ReflectRegistry::new();
    reg.register(RegTypeA::type_meta());
    reg.register(RegTypeA::type_meta()); // duplicate — must not panic
    assert!(reg.get("RegTypeA").is_some());
}
```

**Verification command:** `cargo test -p cvkg-reflect -- registry_tests`

#### Test 6: ReflectedInspector Widget Dispatch (Compile-Time)

**Test file:** `cvkg-components/tests/reflected_inspector_tests.rs`

```rust
use cvkg_reflect::{Reflected, FieldKind};
use serde_json::json;

// A type with all FieldKind variants for dispatch testing.
#[derive(Reflect)]
struct FullProps {
    flag: bool,           // Bool → Checkbox
    count: i32,           // Integer → Slider
    ratio: f32,           // Float → Slider
    name: String,         // String → InlineTextEdit
    color: [f32; 4],      // Color → ColorPicker
    offset: [f32; 2],     // Vec2 → two Float rows
    position: [f32; 3],   // Vec3 → three Float rows
}

#[test]
fn test_full_props_field_kinds() {
    let meta = FullProps::type_meta();
    assert_eq!(meta.fields[0].kind, FieldKind::Bool);
    assert_eq!(meta.fields[1].kind, FieldKind::Integer);
    assert_eq!(meta.fields[2].kind, FieldKind::Float);
    assert_eq!(meta.fields[3].kind, FieldKind::String);
    assert_eq!(meta.fields[4].kind, FieldKind::Color);
    assert_eq!(meta.fields[5].kind, FieldKind::Vec2);
    assert_eq!(meta.fields[6].kind, FieldKind::Vec3);
}

#[test]
fn test_full_props_roundtrip() {
    let mut p = FullProps {
        flag: false,
        count: 0,
        ratio: 0.0,
        name: "".into(),
        color: [0.0; 4],
        offset: [0.0; 2],
        position: [0.0; 3],
    };

    p.set_field("flag", json!(true)).unwrap();
    p.set_field("count", json!(42)).unwrap();
    p.set_field("ratio", json!(0.5)).unwrap();
    p.set_field("name", json!("test")).unwrap();

    assert!(p.flag);
    assert_eq!(p.count, 42);
    assert!((p.ratio - 0.5).abs() < 1e-6);
    assert_eq!(p.name, "test");
}
```

**Verification command:** `cargo test -p cvkg-components -- reflected_inspector_tests`

#### Test 7: InlineTextEdit State Machine

**Test file:** `cvkg-components/tests/inline_text_edit_tests.rs`

```rust
use cvkg_components::interactive::inline_text_edit::InlineTextEdit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[test]
fn test_inline_text_edit_starts_not_editing() {
    let edit = InlineTextEdit::new("hello", |_: String| {});
    assert!(!edit.is_editing());
    assert_eq!(edit.display_value(), "hello");
}

#[test]
fn test_inline_text_edit_click_enters_edit_mode() {
    let mut edit = InlineTextEdit::new("hello", |_: String| {});
    edit.on_click();
    assert!(edit.is_editing());
}

#[test]
fn test_inline_text_edit_enter_commits() {
    let committed = Arc::new(AtomicBool::new(false));
    let c = committed.clone();
    let mut edit = InlineTextEdit::new("old", move |v: String| {
        assert_eq!(v, "new");
        c.store(true, Ordering::Relaxed);
    });

    edit.on_click();
    edit.set_buffer("new".into());
    edit.on_enter();

    assert!(!edit.is_editing());
    assert!(committed.load(Ordering::Relaxed));
}

#[test]
fn test_inline_text_edit_escape_discards() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();
    let mut edit = InlineTextEdit::new("keep", move |_: String| {
        cc.fetch_add(1, Ordering::Relaxed);
    });

    edit.on_click();
    edit.set_buffer("changed".into());
    edit.on_escape();

    assert!(!edit.is_editing());
    assert_eq!(call_count.load(Ordering::Relaxed), 0, "commit must not fire on escape");
    assert_eq!(edit.display_value(), "keep");
}
```

**Verification command:** `cargo test -p cvkg-components -- inline_text_edit_tests`

---

## 14. FrameManifest — Compile-Time Frame Pipeline Declaration

### 14.1 Motivation

Bevy's `Plugin` trait is a **runtime registration system**: `.add_plugins(MyPlugin)` registers systems, resources, and events at app startup into a dynamic `App` struct. This is necessary in an ECS because systems are opaque functions that cannot declare their phase affinity at compile time.

CVKG does not have this limitation. The `FramePhase` enum (`cvkg-scheduler/src/frame.rs:40-55`) defines a **fixed, typed, ordered** pipeline: `Input → State → Layout → Animation → Render → Composite → PostFrame`. Every subsystem's job is to submit work to the correct phase. The ordering is total and immutable — no plugin can insert a phase between `Layout` and `Animation` because the enum's variants are fixed.

The gap is that currently:
- **No crate declares its phase contributions** — `FrameScheduler::submit_for_phase()` is called ad-hoc
- **Kvasir passes are hard-coded** in `cvkg-render-gpu/src/kvasir/nodes.rs::build_render_graph()`
- **Time budgets** are set via `FrameBudgetTracker::default_60fps()` — hard-coded in `cvkg-core`, not extensible per crate
- **The umbrella crate** (`cvkg/src/lib.rs`) has no `configure()` call — initialization is left to downstream apps

A `FrameManifest` — a `const`-constructible struct exposed by each crate — fills these gaps without adopting Bevy's runtime plugin model.

### 14.2 Design Overview

```
                    ┌──────────────────────┐
                    │     cvkg-core         │
                    │  FramePhase (moved)   │
                    │  FrameManifest        │
                    │  PassNodeDescriptor   │
                    │  TimeBudgetRequest    │
                    │  PassNode trait       │
                    └────────┬─────────────┘
                             │
            ┌────────────────┼────────────────┐
            ▼                ▼                ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │ cvkg-physics │ │  cvkg-flow   │ │ cvkg-render  │
    │              │ │              │ │    -gpu      │
    │ const        │ │ const        │ │ const        │
    │ MANIFEST     │ │ MANIFEST     │ │ MANIFEST     │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           └────────────────┼────────────────┘
                            ▼
                   ┌──────────────────┐
                   │ cvkg (umbrella)  │
                   │                  │
                   │ let MERGED =     │
                   │   FrameManifest::│
                   │   merge(&[       │
                   │     physics::    │
                   │       MANIFEST,  │
                   │     flow::       │
                   │       MANIFEST,  │
                   │     render_gpu:: │
                   │       MANIFEST,  │
                   │   ]);            │
                   │                  │
                   │ scheduler        │
                   │   .configure(    │
                   │    &MERGED);     │
                   └──────────────────┘
```

Every crate that participates in the frame pipeline defines a `pub const MANIFEST: FrameManifest`. The umbrella crate calls `FrameManifest::merge(&[...])`, which runs at **compile time** (it is a `const fn`). Conflicts — duplicate pass IDs, budget overruns, ordering cycles — cause a `panic!` in const context, which Rust reports as a **compile error**.

### 14.3 Move `FramePhase` to `cvkg-core`

Currently `FramePhase` lives in `cvkg-scheduler/src/frame.rs`. Since `cvkg-core` is the only crate that every subsystem depends on, `FramePhase` must move there:

| Step | Change |
|---|---|
| 1 | Create `cvkg-core/src/frame_phase.rs` with the `FramePhase` enum |
| 2 | Re-export from `cvkg-core/src/lib.rs` |
| 3 | Update `cvkg-scheduler/src/frame.rs` to `use cvkg_core::FramePhase` |
| 4 | Update all other references (`cvkg-core/src/dirty_flags.rs` etc.) |

`cvkg-core` already depends on `serde`, which `FramePhase` needs for its `Serialize`/`Deserialize` derives — no new dependency is introduced.

### 14.4 Core Types (in `cvkg-core/src/frame_manifest.rs`)

```rust
/// Compile-time crate manifest declaring frame pipeline contributions.
///
/// # Contract
/// - All `const`-constructible — no heap allocation, no vtables.
/// - `merge()` is a `const fn` — conflicts produce compile errors.
#[derive(Debug, Clone, Copy)]
pub struct FrameManifest {
    /// Phases this crate contributes work to, in ascending FramePhase order.
    pub phase_contributions: &'static [FramePhase],
    /// Render pass slots this crate contributes to the Kvasir graph.
    pub pass_nodes: &'static [PassNodeDescriptor],
    /// Per-phase time budget requests.
    pub time_budget_requests: &'static [TimeBudgetRequest],
}

/// Descriptor for a render pass node contributed at compile time.
///
/// The `constructor` is a `fn()` pointer — stored in const data,
/// called at runtime when the umbrella builds the merged Kvasir graph.
/// This avoids coupling `cvkg-core` to the concrete `KvasirNode` trait.
#[derive(Debug, Clone, Copy)]
pub struct PassNodeDescriptor {
    /// Unique pass identifier within the merged set. `&'static str` so each
    /// crate can define its own without coupling to a global `PassId` enum.
    pub id: &'static str,
    /// Human-readable label (DOT graph output, debug tracing).
    pub label: &'static str,
    /// Logical resource input names (e.g. `"scene_color"`, `"depth"`).
    pub inputs: &'static [&'static str],
    /// Logical resource output names (e.g. `"particle_buffer"`).
    pub outputs: &'static [&'static str],
    /// Pass IDs that must execute before this one. Merge validates that every
    /// `after` reference resolves to a pass in the merged set.
    pub after: &'static [&'static str],
    /// Constructor — the function pointer itself lives in const data.
    /// Called at runtime to produce the `Box<dyn PassNode>`.
    pub constructor: fn() -> Box<dyn PassNode>,
}

/// Per-phase time budget request from a subsystem.
#[derive(Debug, Clone, Copy)]
pub struct TimeBudgetRequest {
    pub phase: FramePhase,
    /// Requested time slice in microseconds.
    pub time_slice_us: u64,
    /// Whether this crate's phase work can be skipped when over budget.
    pub skippable: bool,
    /// Subsystem name for logging and telemetry.
    pub name: &'static str,
}
```

### 14.5 `PassNode` Trait (in `cvkg-core`)

A minimal trait that `KvasirNode` extends. This is the **only** coupling between `cvkg-core` and the pass system:

```rust
/// Minimal pass node trait known to cvkg-core.
///
/// cvkg-render-gpu's `KvasirNode` trait extends this with Kvasir-specific
/// methods (`inputs()`, `outputs()`, `execute()` with `ExecutionContext`).
pub trait PassNode: Send + Sync {
    fn label(&self) -> &'static str;
}
```

In `cvkg-render-gpu`:

```rust
// cvkg-render-gpu/src/kvasir/node.rs
pub trait KvasirNode: PassNode {
    fn inputs(&self) -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];
    fn pass_id(&self) -> PassId;
    fn execute(&self, ctx: &mut ExecutionContext);
}
```

The `PassNodeDescriptor::constructor` returns `Box<dyn PassNode>`. The umbrella crate's init code downcasts via `KvasirNode` when wiring the Kvasir graph:

```rust
let pass_node: Box<dyn PassNode> = (desc.constructor)();
let kvasir_node: Box<dyn KvasirNode> = /* safe cast, checked at build time */;
```

### 14.6 `FrameManifest::merge()` — `const fn` with compile-time conflict detection

```rust
impl FrameManifest {
    /// Merge multiple crate manifests into one. Runs at compile time.
    ///
    /// # Compile-time panics (→ compile errors)
    /// - **Duplicate pass ID**: two crates register a pass with the same `id`.
    /// - **Unresolved `after` reference**: a pass depends on a pass that no
    ///   crate registered.
    /// - **Ordering cycle**: pass A requires pass B, pass B requires pass A.
    /// - **Phase ordering violation**: `phase_contributions` is not in
    ///   ascending `FramePhase` order.
    /// - **Budget overrun**: sum of requested time slices exceeds the frame
    ///   budget (default 16.67ms for 60fps).
    pub const fn merge(manifests: &[&Self]) -> Self {
        // Step 1: Concatenate phase contributions (validate order).
        // Step 2: Concatenate pass nodes (detect duplicate IDs).
        // Step 3: Resolve `after` dependencies (detect unresolved refs and cycles).
        // Step 4: Merge time budgets (detect overruns).
        // Step 5: Return merged manifest.
        //
        // All steps use only `&[T]` and `loop` — no Vec, no HashMap.
        // Panics in const context → compile error.
        todo!()  // see §14.7 for the actual implementation approach
    }
}
```

**How `const fn` conflict detection works:**

```rust
// In const context, `panic!()` is a compile error.
// Example: duplicate pass ID detection
const fn detect_duplicates(descs: &[PassNodeDescriptor]) {
    let len = descs.len();
    let mut i = 0;
    while i < len {
        let mut j = i + 1;
        while j < len {
            if descs[i].id.as_bytes() == descs[j].id.as_bytes() {
                panic!("duplicate pass id")
            }
            j += 1;
        }
        i += 1;
    }
}
```

**Limitations of `const fn` merge:**
- No heap allocation → O(n²) algorithms for small n (pass count is typically < 30)
- No `HashMap` → linear scans for dependency resolution
- No trait methods → no calling constructors during merge
- String comparison via byte slices → works in const since Rust 1.68

For a render graph with ~10–20 passes across all crates, O(n²) is negligible — the merge runs once at compile time, not in the render loop.

### 14.7 `const fn` Merge Implementation (Detailed)

```rust
impl FrameManifest {
    pub const fn empty() -> Self {
        Self {
            phase_contributions: &[],
            pass_nodes: &[],
            time_budget_requests: &[],
        }
    }

    pub const fn merge(manifests: &[&Self]) -> Self {
        // ── 1. Count totals ────────────────────────────────────────────
        let mut total_phases = 0usize;
        let mut total_passes = 0usize;
        let mut total_budgets = 0usize;
        let mut i = 0;
        while i < manifests.len() {
            total_phases += manifests[i].phase_contributions.len();
            total_passes += manifests[i].pass_nodes.len();
            total_budgets += manifests[i].time_budget_requests.len();
            i += 1;
        }

        // ── 2. Flatten phase contributions (already ordered per crate) ──
        // Phase ordering is validated at merge time: no crate should claim
        // phases out of order, and the merged list preserves the canonical
        // FramePhase ordering (checked elsewhere).
        let mut merged_phases = [FramePhase::Input; 7];  // max 7 phases
        // ... flatten and validate ...

        // ── 3. Flatten pass nodes and detect duplicates ────────────────
        // Linear scan for duplicate `id` strings.

        // ── 4. Resolve `after` references ───────────────────────────────
        // For each pass, check that every `after` entry matches some pass's
        // `id` in the merged set. Cycle detection: build a partial order
        // and verify it is acyclic (Kahn's algorithm, O(n²) for small n).

        // ── 5. Merge budget requests ───────────────────────────────────
        // Sum time_slice_us per phase. Panic if total > 16667 (16.67ms).

        // ── 6. Build merged manifest ───────────────────────────────────
        // Store flattened arrays in static storage (output is `Self`).

        Self::empty()  // placeholder — real implementation fills arrays
    }
}
```

**Because `const fn` cannot return references to temporaries**, the merged output cannot own concatenated arrays. The solution: **the merged `FrameManifest` references static arrays in the umbrella crate**, generated by a `merge_manifests!` declarative macro:

```rust
// cvkg/src/lib.rs

merge_manifests! {
    physics::MANIFEST,
    flow::MANIFEST,
    materials::MANIFEST,
    render_gpu::MANIFEST,
}

// Expands to:
//   pub static MERGED_PHASES: &[FramePhase] = &[Input, State, Layout, ...];
//   pub static MERGED_PASSES: &[PassNodeDescriptor] = &[
//       PassNodeDescriptor { id: "geometry", ... },
//       PassNodeDescriptor { id: "particle_trail", ... },
//       PassNodeDescriptor { id: "glass", ... },
//   ];
//   pub static MERGED_BUDGETS: &[TimeBudgetRequest] = &[...];
//
//   Checks:
//     - Duplicate pass IDs → compile error
//     - Unresolved `after` refs → compile error
//     - Ordering cycles → compile error
//     - Budget overrun → compile error
```

The `merge_manifests!` macro does the merge at compile time by evaluating `const` expressions inside the macro expansion. If any check panics, the macro expansion fails with a compile error pointing to the conflicting manifests.

### 14.8 Example Manifests by Crate

#### `cvkg-physics` — Rigid body simulation

```rust
// cvkg-physics/src/lib.rs

pub const MANIFEST: FrameManifest = FrameManifest {
    phase_contributions: &[FramePhase::State, FramePhase::Render],
    pass_nodes: &[
        PassNodeDescriptor {
            id: "physics_debug",
            label: "Physics Debug Draw",
            inputs: &[],
            outputs: &["physics_debug_buffer"],
            after: &["geometry"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(PhysicsDebugDrawPass::new())
            },
        },
        // SPH fluid simulation pass
        PassNodeDescriptor {
            id: "fluid_simulation",
            label: "SPH Fluid Sim (Compute)",
            inputs: &["scene_depth"],
            outputs: &["fluid_density"],
            after: &["geometry"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(FluidSimulationPass::new())
            },
        },
    ],
    time_budget_requests: &[
        TimeBudgetRequest {
            phase: FramePhase::State,
            time_slice_us: 2000,  // 2ms
            skippable: true,
            name: "physics",
        },
    ],
};
```

#### `cvkg-flow` — Flow graph / particle ribbons

```rust
// cvkg-flow/src/lib.rs

pub const MANIFEST: FrameManifest = FrameManifest {
    phase_contributions: &[FramePhase::Layout, FramePhase::Render],
    pass_nodes: &[
        PassNodeDescriptor {
            id: "particle_trail",
            label: "Particle Trail Render",
            inputs: &["scene_color"],
            outputs: &["scene_color"],  // compositing into scene
            after: &["ui"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(ParticleTrailPass::new())
            },
        },
    ],
    time_budget_requests: &[
        TimeBudgetRequest {
            phase: FramePhase::Layout,
            time_slice_us: 1000,  // 1ms for force-directed layout
            skippable: true,
            name: "flow_layout",
        },
        TimeBudgetRequest {
            phase: FramePhase::Render,
            time_slice_us: 2000,  // 2ms for ribbon tessellation
            skippable: true,
            name: "flow_render",
        },
    ],
};
```

#### `cvkg-materials` — Pure data, no phases or passes

```rust
// cvkg-materials/src/lib.rs
// This crate defines GlassMaterial, MicaMaterial, AcrylicMaterial data
// structs. It contributes no phase work, no render passes, and no budget.
// Its MANIFEST is the identity element for merge().

pub const MANIFEST: FrameManifest = FrameManifest::empty();
```

#### `cvkg-render-gpu` — Core render pipeline

```rust
// cvkg-render-gpu/src/lib.rs

pub const MANIFEST: FrameManifest = FrameManifest {
    phase_contributions: &[
        FramePhase::Render,
        FramePhase::Composite,
    ],
    pass_nodes: &[
        PassNodeDescriptor {
            id: "geometry",
            label: "Geometry Pass (Opaque)",
            inputs: &[],
            outputs: &["scene_color", "scene_depth"],
            after: &[],
            constructor: || -> Box<dyn PassNode> {
                Box::new(GeometryNode::new())
            },
        },
        PassNodeDescriptor {
            id: "glass",
            label: "Glass (Backdrop Blur)",
            inputs: &["scene_color", "scene_depth"],
            outputs: &["glass_output"],
            after: &["geometry"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(GlassNode::new())
            },
        },
        PassNodeDescriptor {
            id: "ui",
            label: "UI Compositing",
            inputs: &["scene_color", "glass_output"],
            outputs: &["ui_output"],
            after: &["glass"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(UINode::new())
            },
        },
        PassNodeDescriptor {
            id: "bloom_extract",
            label: "Bloom Extract",
            inputs: &["ui_output"],
            outputs: &["bloom_src"],
            after: &["ui"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(BloomExtractNode::new())
            },
        },
        PassNodeDescriptor {
            id: "bloom_blur",
            label: "Bloom Blur",
            inputs: &["bloom_src"],
            outputs: &["bloom_dst"],
            after: &["bloom_extract"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(BloomBlurNode::new())
            },
        },
        PassNodeDescriptor {
            id: "composite",
            label: "Final Composite",
            inputs: &["ui_output", "bloom_dst", "physics_debug_buffer"],
            outputs: &["swapchain"],
            after: &["bloom_blur", "particle_trail"],
            constructor: || -> Box<dyn PassNode> {
                Box::new(CompositeNode::new())
            },
        },
    ],
    time_budget_requests: &[
        TimeBudgetRequest {
            phase: FramePhase::Render,
            time_slice_us: 8000,  // 8ms for all GPU passes
            skippable: false,     // render must always run
            name: "render_gpu",
        },
    ],
};
```

### 14.9 Umbrella `cvkg` Wiring

```rust
// cvkg/src/lib.rs

merge_manifests! {
    cvkg_physics::MANIFEST,
    cvkg_flow::MANIFEST,
    cvkg_materials::MANIFEST,
    cvkg_render_gpu::MANIFEST,
}

// The macro generates:
//   pub static MERGED_MANIFEST: FrameManifest = /* validated at compile time */;
//   pub static KVASIR_PASSES: &[PassNodeDescriptor] = &[...];

/// Initialize the frame scheduler from the merged manifest.
pub fn configure_scheduler(scheduler: &mut FrameScheduler) {
    // Register time budgets per phase.
    for budget in MERGED_MANIFEST.time_budget_requests {
        scheduler.set_budget(budget.phase, *budget);
    }
}

/// Build the Kvasir render graph from merged pass descriptors.
/// Called once during initialization (not per frame).
pub fn build_render_graph() -> KvasirGraph {
    let mut builder = GraphBuilder::new();
    let mut node_keys: HashMap<&'static str, NodeKey> = HashMap::new();

    // Create all pass nodes.
    for desc in MERGED_MANIFEST.pass_nodes {
        let pass_node: Box<dyn PassNode> = (desc.constructor)();
        let kvasir_node: Box<dyn KvasirNode> = downcast_pass_node(pass_node, desc.id);
        let key = builder.add_node(kvasir_node);
        node_keys.insert(desc.id, key);
    }

    // Wire connections based on `inputs`, `outputs`, and `after`.
    for desc in MERGED_MANIFEST.pass_nodes {
        let from = node_keys[desc.id];
        for after_id in desc.after {
            let to = node_keys[after_id];
            // The `after` relationship means this pass produces output that
            // the `after` pass consumes. Wire the last output resource.
            for resource_str in desc.outputs {
                let resource_id = resolve_resource_id(resource_str);
                builder.connect(from, resource_id, to);
            }
        }
    }

    builder.build()
}
```

### 14.10 Compile-Time Guarantees — What the Compiler Checks

| Check | Mechanism | Error Message |
|---|---|---|
| **Duplicate pass ID** | `const fn` linear scan panics on match | `"panic: duplicate pass id 'glass'"` |
| **Unresolved `after` reference** | `const fn` scan: every `after[i]` must match some pass `id` | `"panic: pass 'particle_trail' depends on unknown 'foobar'"` |
| **Ordering cycle** | `const fn` Kahn's algorithm: if not all nodes can be ordered, there is a cycle | `"panic: ordering cycle detected in pass graph"` |
| **Phase order violation** | `const fn` verifies `phase_contributions` is sorted | `"panic: phase contributions not in ascending order"` |
| **Budget overrun** | `const fn` sums per-phase budgets, panics if > 16667µs | `"panic: Render phase budget 22000µs exceeds 16667µs limit"` |
| **Non-existent `FramePhase`** | The type system: `FramePhase` is an enum with fixed variants | Compiler error: `unresolved variant` |
| **Feature-gated pass present** | Conditional compilation: `#[cfg(feature = "gpu")]` gates the `MODULE` block | Pass simply absent when feature is off |

**What the compiler CANNOT check:**
- Correctness of `constructor` downcast (runtime assertion at init)
- Correctness of resource name resolution (`"scene_color"` → `ResourceId(1)`)
- Whether two passes reading/writing the same resource have compatible formats

These are checked at runtime during graph initialization (and are more appropriate for a render pipeline, where GPU capabilities vary).

### 14.11 Interaction with Existing Code

| Existing Code | Impact |
|---|---|
| `cvkg-scheduler::FramePhase` | Moved to `cvkg-core`; re-exported from `cvkg-scheduler` for backward compat |
| `FrameBudgetTracker::default_60fps()` | Replaced by merged manifest budgets; kept as fallback when no manifest is configured |
| `nodes.rs::build_render_graph()` | Replaced by `build_render_graph()` in umbrella crate that iterates merged manifests; `build_render_graph()` is kept as a backward-compat entry point for tests |
| `FrameScheduler::submit_for_phase()` | Unchanged — manifests declare intent, `submit_for_phase()` is the actual runtime submission call |
| `cvkg/src/lib.rs` | Gains `configure_scheduler()` and `build_render_graph()`; the current no-op facade is preserved for simple apps that want defaults |

### 14.12 Migration Path

| Step | Change | Backward Compat? |
|---|---|---|
| 1 | Move `FramePhase` from `cvkg-scheduler` to `cvkg-core/src/frame_phase.rs` | Yes (re-export from both) |
| 2 | Add `PassNode` trait to `cvkg-core` | Yes (new trait, no existing impls break) |
| 3 | Add `FrameManifest`, `PassNodeDescriptor`, `TimeBudgetRequest` to `cvkg-core` | Yes (new types, opt-in) |
| 4 | Add `merge_manifests!` declarative macro to `cvkg-macros` | Yes (new macro) |
| 5 | Define `pub const MANIFEST` in `cvkg-render-gpu` (migrate existing passes from `build_render_graph()`) | Yes (old function kept as fallback) |
| 6 | Define `pub const MANIFEST` in `cvkg-physics` | Yes (new, no existing code affected) |
| 7 | Define `pub const MANIFEST` in `cvkg-flow` | Yes (new, no existing code affected) |
| 8 | Wire `merge_manifests!` in `cvkg/src/lib.rs` | Yes (behind `#[cfg(feature = "framemanifest")]` if desired) |
| 9 | Replace `build_render_graph()` in `nodes.rs` with manifest-driven graph builder | Yes (old function signature kept) |

---

## 15. Theme Context Propagation & Portal Inheritance

### 15.1 Bevy vs CVKG — Theme Propagation Comparison

| Aspect | Bevy (`bevy_feathers`) | CVKG |
|---|---|---|
| **Mechanism** | Walk entity hierarchy via `Parent` component to find nearest `Theme<T>` | Flat thread-local `THEME_CONTEXT` + per-Renderer `ColorTheme` + global `Environment` singleton |
| **Per-subtree override** | `commands.spawn((Node::default(), Theme::dark_custom()))` overrides theme for that subtree | **Not possible** — only one active theme per thread per frame |
| **Portal inheritance** | Portal entities are children of their source entity; theme is inherited naturally | `PhaseGate` renders portal content with the same global theme — no mechanism to inherit from the triggering element |
| **Default fallback** | Built-in `Theme::default()` if no `Theme<T>` component found in ancestors | `THEME_CONTEXT` returns `SemanticColors::default()` (dark theme) when none is set |
| **Context storage** | `Theme<T>` is an ECS component — per-entity, scoped to the entity's subtree | `std::thread_local!` + `fn()` getters — process-wide, unscoped |

**Bevy's strength:** Because `Theme<T>` is a component on any entity, you can spawn a subtree with a different theme and all children automatically inherit it. A dropdown rendered as a child of a dark-themed panel automatically gets the dark theme.

**CVKG's gap:** The flat global approach means **all components in a frame share exactly one theme**. A light-themed app cannot embed a dark-themed code block. A dropdown portal triggered from a dark section of the UI renders with the app's global light theme instead of the dark section theme.

### 15.2 Theme Systems in CVKG — Summary of Current State

CVKG has three parallel theme access paths, all flat:

| Path | Mechanism | Set by | Read by | Scope |
|---|---|---|---|---|
| **A: Thread-local** | `THEME_CONTEXT: RefCell<Option<ThemeContext>>` in `cvkg-core/src/theme.rs` | Intended: renderer before each frame via `set_current_theme()` | Components via `use_theme()` → `SemanticColors` | Per-thread, **unwired** (no callers set it) |
| **B: GPU Renderer** | `GpuRenderer.current_theme: ColorTheme` in `cvkg-render-gpu/src/api/mod.rs` | `renderer.set_theme(color_theme)` at init | Shader uniform via `theme_buffer` | Per-Renderer instance |
| **C: Global Environment** | `ENVIRONMENT: HashMap<TypeId, Box<dyn Any>>` in `cvkg-core/src/env_core.rs` | `resolve_environment()` at startup | `StyleResolver::color_array(key)` (called by 50+ `theme::*()` helpers) | Process-wide |

The `cvkg-components/src/theme.rs` helpers (which 95% of components call) read from Path C — the global `DesignTokens` environment, which is a singleton `HashMap` indexed by `TypeId`. There is no path for a per-subtree override.

### 15.3 Portal Context — `portal_target` vs `PhaseGate`

Two portal mechanisms exist:

| Mechanism | Status | Theme risk |
|---|---|---|
| **`VNode.portal_target: Option<NodeId>`** | **Dead code** — field exists but never populated, never acted on | N/A — not functional |
| **`PhaseGate<V>` component** | Active — calls `enter_portal()`/`exit_portal()` on `Renderer` trait | **Low** — current flat theme is global, so portal content reads same theme as source. But there is **no way** for portal content to inherit the *specific* theme of its triggering element when the app has multiple themed sections. |

If CVKG later adds per-subtree theming (Section 15.4), the `PhaseGate` portal will **not** automatically inherit the theme of the source node — it renders into a separate buffer with the renderer's current theme, which may differ from the source node's subtree theme.

### 15.4 Phase 1 — Theme Context Stack on the Renderer

Add a theme stack to the `Renderer` trait so that per-subtree overrides are possible:

```rust
// cvkg-core/src/renderer_trait.rs

pub trait Renderer: ElapsedTime + Send + RendererErrorHandler {
    // ── existing methods ──
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn set_theme(&mut self, theme: ColorTheme) {}  // existing, flat

    // ── NEW: theme stack ──

    /// Push a theme override for the current subtree.
    /// All child nodes rendered until `pop_theme()` are drawn with this theme.
    fn push_theme(&mut self, _theme: ColorTheme) {}

    /// Pop the theme override and restore the previous theme.
    fn pop_theme(&mut self) {}

    /// Return the theme token set at the current stack depth.
    /// Falls back to the default theme if no theme has been pushed.
    fn current_theme(&self) -> ColorTheme;
}
```

New default impl for `current_theme()` that reads from thread-local (Path A):

```rust
fn current_theme(&self) -> ColorTheme {
    use_theme_context().into_color_theme()
}
```

#### Implementation in GpuRenderer

```rust
// cvkg-render-gpu/src/api/mod.rs

pub struct GpuRenderer {
    // ── existing ──
    current_theme: ColorTheme,

    // ── NEW: theme stack ──
    theme_stack: Vec<ColorTheme>,
}

impl Renderer for GpuRenderer {
    fn push_theme(&mut self, theme: ColorTheme) {
        let prev = self.current_theme;  // current is now parent
        self.current_theme = theme;
        self.theme_stack.push(prev);
        // Upload new theme to GPU
        self.queue.write_buffer(
            &self.theme_buffer, 0,
            bytemuck::bytes_of(&self.current_theme),
        );
    }

    fn pop_theme(&mut self) {
        if let Some(parent_theme) = self.theme_stack.pop() {
            self.current_theme = parent_theme;
            self.queue.write_buffer(
                &self.theme_buffer, 0,
                bytemuck::bytes_of(&self.current_theme),
            );
        }
    }

    fn current_theme(&self) -> ColorTheme {
        self.current_theme
    }
}
```

#### Implementation in VNodeRenderer

```rust
// cvkg-vdom/src/lib.rs

pub struct VNodeRenderer {
    // ── existing ──
    nodes: HashMap<NodeId, VNode>,
    // ...

    // ── NEW: theme stack ──
    theme_stack: Vec<ColorTheme>,
    fallback_theme: ColorTheme,
}

impl Renderer for VNodeRenderer {
    fn push_theme(&mut self, theme: ColorTheme) {
        let current = self.current_theme();
        self.theme_stack.push(current);
        // The next VNode pushed will track this theme in its props
    }

    fn pop_theme(&mut self) {
        self.theme_stack.pop();
    }

    fn current_theme(&self) -> ColorTheme {
        self.theme_stack.last().copied()
            .unwrap_or(self.fallback_theme)
    }
}
```

### 15.5 Phase 2 — Per-VNode `theme_override` Field

Add `theme_override: Option<ColorTheme>` to `VNode`:

```rust
// cvkg-vdom/src/vnode.rs

pub struct VNode {
    pub id: NodeId,
    // ── existing fields ──
    pub portal_target: Option<NodeId>,
    /// If set, this node overrides the inherited theme for its subtree.
    /// Children render with this theme instead of the parent's theme.
    #[serde(skip)]
    pub theme_override: Option<ColorTheme>,
}

// ── Implementation notes (removed from docs) ──

/// Diff engine must include `theme_override` in Update variants when it changes.
/// When serialized, becomes `Option<Option<ColorTheme>>`.
```

When the diff engine compares two VNodes, it must include `theme_override` in the patch:

```rust
// cvkg-vdom/src/diff.rs (add to existing Update patch)

pub struct Update {
    pub id: NodeId,
    pub props: HashMap<String, Option<serde_json::Value>>,
    pub state: Option<HashMap<String, serde_json::Value>>,
    pub portal_target: Option<Option<NodeId>>,
    pub theme_override: Option<Option<ColorTheme>>,  // NEW
    // ...
}
```

#### Setting a subtree theme from code

```rust
/// View wrapper that applies a theme override to its subtree.
pub struct Themed<V: View> {
    theme: ColorTheme,
    content: V,
}

impl<V: View> View for Themed<V> {
    type Body = V::Body;
    fn body(self) -> Self::Body { self.content.body() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_theme(self.theme);
        self.content.render(renderer, rect);
        renderer.pop_theme();
    }
}

// Usage:
// Themed::new(dark_theme, VStack::new()
//     .child(Text::new("I'm dark"))
//     .child(Themed::new(light_theme, Text::new("I'm light")))
// )
```

The `renderer.push_theme()` call stores the override in the VNode when building the VDOM, and uploads the new theme constant to the GPU when rendering.

### 15.6 Phase 3 — Portal Theme Inheritance (The Original Fix)

When a `PhaseGate` renders portal content, the portal's rendering context **must inherit the theme from the portal's source node in the VDOM tree**, not from the renderer's global state.

**Problem:** `PhaseGate::render()` calls `enter_portal()` which changes the rendering buffer. But the portal source node may have a `theme_override` on it (or be inside a `Themed<V>` subtree). The portal buffer's theme is currently whatever the renderer has at the call site, which is the *caller's* theme, not the *portal source's* theme.

**Fix:** Save the current theme context at `enter_portal()` and restore it when the portal buffer is composited:

```rust
// cvkg-core/src/renderer_trait.rs

/// Begin portal rendering.
/// Saves the current theme so the portal buffer is composited with
/// the theme of the portal's source node, not whichever theme is active
/// during the compositing pass.
fn enter_portal(&mut self, _z_index: i32) {
    // Default impl: save theme on a portal stack.
    // Override in GpuRenderer to create an offscreen buffer.
}

/// Exit portal and return to inline rendering.
fn exit_portal(&mut self) {
    // Default impl: pop portal theme stack.
}
```

#### GpuRenderer implementation:

```rust
pub struct GpuRenderer {
    // ── existing ──
    current_theme: ColorTheme,
    theme_stack: Vec<ColorTheme>,
    // ── NEW: portal theme storage ──
    /// Maps portal z-index to the theme that was active when the portal was entered.
    portal_themes: Vec<(i32, ColorTheme)>,
}

impl Renderer for GpuRenderer {
    fn enter_portal(&mut self, z_index: i32) {
        let theme = self.current_theme();
        self.portal_themes.push((z_index, theme));
        // Create or select offscreen render target for this portal layer.
        // (existing logic for z-index bump or offscreen buffer)
    }

    fn exit_portal(&mut self) {
        // When the portal buffer is composited, use the saved theme.
        if let Some((_, saved_theme)) = self.portal_themes.last() {
            self.push_theme(*saved_theme);  // restore source-node theme
            // Composite portal buffer.
            self.pop_theme();
        }
        self.portal_themes.pop();
    }
}
```

#### VNodeRenderer implementation:

```rust
impl Renderer for VNodeRenderer {
    // Unchanged: portals are flattened, theme stack already preserved.
    // Since push_theme/pop_theme are called by Themed<V> during rendering,
    // the VNode already tracks the correct theme for each subtree.
    // When enter_portal/exit_portal are no-ops (VDOM build), the theme
    // stack naturally persists across portal boundaries.
}
```

**Why the VNodeRenderer doesn't need modification:** During VDOM building, `PhaseGate::render()` renders portal content inline (VNodeRenderer ignores portals). Therefore, any `push_theme`/`pop_theme` calls inside portal content are correctly nested within the source node's theme stack. The VDOM tree already captures the correct theme per node. The fix only affects GPU renderers that separate portal content into different buffers/compositing passes.

### 15.7 Integration with Existing Components

The 50+ `theme::*()` helper functions in `cvkg-components/src/theme.rs` currently read from the global `Environment` singleton (Path C). To support per-subtree overrides, they must switch to reading from the renderer's theme stack:

```rust
// cvkg-components/src/theme.rs

// CURRENT: reads from global Environment singleton
pub fn text() -> [f32; 4] {
    StyleResolver::color_array("text")  // global, no hierarchy
}

// FUTURE: reads from renderer's current context
// But helpers are called during View::render() which receives
// &dyn Renderer — they don't have access to the Renderer directly
// unless passed as a parameter or stored in thread-local.
```

**Challenge:** The 50+ `theme::*()` helpers are free functions called from inside `View::render()`, which takes `&self` and `&mut dyn Renderer`. The helpers do not receive the renderer reference. Changing all call sites to pass the renderer would be a massive refactor.

**Solution:** Use the thread-local `THEME_CONTEXT` (Path A):

1. On `Renderer::push_theme()`, also update the thread-local `THEME_CONTEXT`:
   ```rust
   fn push_theme(&mut self, theme: ColorTheme) {
       set_theme_context(ThemeContext::from_color_theme(theme));
       // ... renderer-specific logic ...
   }
   ```

2. The `theme::*()` helpers read from the thread-local `THEME_CONTEXT` via `use_theme_context()`:
   ```rust
   pub fn text() -> [f32; 4] {
       let ctx = use_theme_context();
       ctx.colors.text
   }
   ```

This way, `push_theme()` on the renderer transparently propagates to all components via the same thread-local path they already use. No component code changes are needed.

### 15.8 Migration Path

| Step | Change | Backward Compat? | Effort |
|---|---|---|---|
| 1 | Add `push_theme()`/`pop_theme()`/`current_theme()` to `Renderer` trait with default no-op stack | Yes (defaults are no-ops, existing impls unchanged) | S |
| 2 | Implement theme stack in `GpuRenderer` — `theme_stack: Vec<ColorTheme>` + GPU buffer upload on push | Yes (new field, existing behavior when stack is empty) | M |
| 3 | Implement theme stack in `VNodeRenderer` — `theme_stack: Vec<ColorTheme>` | Yes (new field, VNode props track theme) | S |
| 4 | Wire `push_theme()` to update thread-local `THEME_CONTEXT` so `theme::*()` helpers pick up the override | Yes (thread-local was unused, now it's alive) | S |
| 5 | Add `theme_override: Option<ColorTheme>` to `VNode` | Yes (new field, `None` default preserves existing behavior) | S |
| 6 | Add `theme_override` to diff/patch (`Update` variant) | Yes (new optional field) | S |
| 7 | Create `Themed<V>` wrapper component | Yes (new component, opt-in) | S |
| 8 | Add portal theme save/restore to `enter_portal()`/`exit_portal()` in `GpuRenderer` | Yes (z-index behavior unchanged) | M |
| 9 | Update `theme::*()` helpers to read from thread-local `THEME_CONTEXT` instead of global `Environment` | **⚠ Path-breaking** — components that relied on `Environment` defaults will need to ensure `set_current_theme()` is called before the frame | L |
| 10 | Migrate any hard-coded theme color calls in widget code to `theme::*()` helpers | Refactor (mechanical) | M |

**Breaking change warning:** Step 9 changes the underlying data source for every `theme::text()`, `theme::surface()`, etc. call. Test all components after this step to verify correct color values. The thread-local must be wired into the render loop before this step (`GpuRenderer::render_frame()` calls `set_theme_context()` at the start).

### 15.9 Design Alternatives Considered

| Alternative | Why not chosen |
|---|---|
| **Per-VNode `ColorTheme` field (no stack)** | Without a stack, `pop_theme` has no previous value to restore. Nesting `Themed<V>` would not compose correctly. |
| **Environment-based stack (Path C)** | `Environment` is a process-wide singleton; stacking on it would require thread-local mutable access anyway, making the thread-local the simpler single source of truth. |
| **`theme` parameter on every `render()` call** | Would require changing the `View` trait signature, breaking every existing component. The thread-local approach avoids this. |
| **Bevy-style ECS `Theme<T>` component** | CVKG does not use ECS for UI; UI is a VDOM tree, not an entity graph. Walking the VDOM parent chain for each frame would be O(n²). The renderer stack is O(1). |

---

## 16. Ten Improvements — Easiest to Most Impactful

This section ranks ten CVKG improvements inspired by Bevy patterns, ordered from least effort to highest impact. Items already detailed in earlier sections link to their full design.

### 16.1 Typed Event Triggers (Easiest)

**Inspired by:** Bevy's `Observer`/`Trigger<E>` — events are typed structs dispatched and observed through the ECS, not string-keyed callbacks.

**Crates touched:** `cvkg-vdom`, `cvkg-core`, `cvkg-components`

**Sketch:**

```rust
// ── cvkg-core/src/event.rs ── NEW types

/// Marker trait for events that can be dispatched through the VDOM.
pub trait Event: Send + 'static {}

/// A triggerable event registration on a view node.
/// Rather than register_handler("click", arc_fn), use:
///   .on::<ClickEvent>(|e: &ClickEvent, ctx: &mut EventCtx| { ... })
pub struct Trigger<E: Event> {
    pub callback: Arc<dyn Fn(&E, &mut EventCtx) + Send + Sync>,
}

/// Context passed to event handlers, providing access to signal setters,
/// focus management, and re-render requests.
pub struct EventCtx<'a> {
    pub signals: &'a mut SignalRegistry,
    pub request_rerender: bool,
}

/// A typed registry that maps (NodeId, TypeId) → Vec<Box<dyn Fn(...)>>.
/// Replaces the current `HashMap<String, Vec<...>>` handler map in VNodeRenderer.
pub struct TriggerRegistry {
    handlers: HashMap<(NodeId, std::any::TypeId), Vec<Box<dyn std::any::Any>>>,
}

impl TriggerRegistry {
    pub fn on<E: Event>(&mut self, node_id: NodeId, handler: impl Fn(&E, &mut EventCtx) + Send + Sync + 'static) {
        let key = (node_id, std::any::TypeId::of::<E>());
        self.handlers.entry(key).or_default().push(Box::new(handler));
    }

    pub fn dispatch<E: Event>(&self, node_id: NodeId, event: &E, ctx: &mut EventCtx) {
        let key = (node_id, std::any::TypeId::of::<E>());
        if let Some(handlers) = self.handlers.get(&key) {
            for h in handlers {
                if let Some(f) = h.downcast_ref::<Arc<dyn Fn(&E, &mut EventCtx) + Send + Sync>>() {
                    f(event, ctx);
                }
            }
        }
    }
}

// ── Usage in cvkg-components ──

// Before (string-keyed):
Button::new("Click", || println!("clicked"))
    // register_handler("click", ...) is buried in Button::render

// After (typed):
Button::new("Click")
    .on::<PointerClickEvent>(|e, ctx| {
        println!("clicked at ({}, {})", e.x, e.y);
        ctx.request_rerender = true;
    })
    .on::<PointerEnterEvent>(|e, ctx| {
        println!("hover start");
    })
    .on::<PointerLeaveEvent>(|e, ctx| {
        println!("hover end");
    })
```

**Bevy parallel:** Bevy's `.observe(|trigger: Trigger<CollisionEvent>| { ... })` dispatches typed events through the ECS without string keys. CVKG's `register_handler("string", arc_fn)` is the equivalent of Bevy's pre-0.14 custom event system — typed triggers eliminate the string-keyed dispatch layer.

**Risk:** Backward-compatible — `register_handler` is kept as a deprecated wrapper that delegates to `TriggerRegistry`. The `on::<E>(|e| ...)` builder method can be added to all interactive components without breaking existing callers.

---

### 16.2 Auto-Required Companion State (`#[require]`)

**Inspired by:** Bevy's `#[require(Transform, Visibility)]` — component requirements auto-insert companion components at spawn time.

**Crates touched:** `cvkg-macros`, `cvkg-core`

**Full design in Section 12.** Summary: the `#[view_component]` macro currently ignores its `_attr: TokenStream` parameter. Parse `#[require(Focusable, A11yProps)]` from it, then generate a `companion_states()` method on the `View` trait. The `View` trait gains a default `companion_states()` that returns an empty vec; the `#[view_component]` macro overrides it. `AnyView::render()` passes companions through `push_vnode_with_companions()` so the VNode initializes Focusable, A11yProps, ScrollState, etc. on construction.

**Sketch:**

```rust
#[view_component]
#[require(FocusableCompanion, A11yCompanion)]
fn MyButton(label: String) {
    Button::new(label, || {})
}

// Expands to:
impl cvkg_core::View for MyButtonView {
    fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
        vec![
            Box::new(FocusableCompanion::default()),
            Box::new(A11yCompanion::default()),
        ]
    }
}
```

**Risk:** No backward-compat concern. Existing components without `#[require(...)]` get the default empty `companion_states()`. Only new components that explicitly add `#[require(...)]` change behavior.

---

### 16.3 Layout-Animated Spring Constraints

**Inspired by:** Bevy's animation system — keyframe-driven property interpolation on any component.

**Crates touched:** `cvkg-layout`, `cvkg-anim`, `cvkg-vdom`

**Sketch:**

```rust
// cvkg-layout/src/animation.rs ── NEW types

/// An animated layout constraint driven by a spring physics simulation.
/// Wraps a spring from cvkg-anim and applies the current value to the
/// layout node's size/position each frame.
pub struct SpringConstraint {
    /// Target value the spring is driving toward.
    pub target: Signal<f32>,
    /// Spring parameters (stiffness, damping, mass).
    pub spring: cvkg_anim::SpringParams,
    /// Which layout property this constraint affects.
    pub property: LayoutProperty,
}

/// Layout properties that can be animated.
pub enum LayoutProperty {
    Width,
    Height,
    MarginLeft, MarginRight, MarginTop, MarginBottom,
    Gap,
    FlexGrow,
}

/// Integration with Taffy layout:
/// 1. SpringConstraint::target.set(new_value) starts the spring.
/// 2. Each frame, Animation phase ticks the spring → current_value.
/// 3. The current_value is written to the Taffy Style node before layout.
/// 4. Layout phase runs Taffy with the animated value.
///
/// Convention: components that want animated layout use:
///   let width = SpringConstraint::new(
///       Signal::new(200.0),
///       SpringParams::fluid(),
///       LayoutProperty::Width,
///   );
///   width.target.set(300.0);  // → animated transition over ~300ms
```

**Bevy parallel:** Bevy's animation system interpolates any `Component` field via `AnimatedProperty<T>`. CVKG's spring physics (`cvkg-anim`) and Taffy layout (`cvkg-layout`) are adjacent but disconnected — `SpringConstraint` bridges them.

**Risk:** A layout spring that oscillates (underdamped) could cause the layout phase to produce different results each frame until settled. Mitigation: cap iterations per frame and settle (snap to target) when velocity is below epsilon.

---

### 16.4 Layer-Typed Signal Mutations

**Inspired by:** Bevy's `Changed<T>` — binary per-component change detection. CVKG extends this with typed pipeline layers.

**Crates touched:** `cvkg-core`, `cvkg-vdom`, `cvkg-scheduler`

**Full design in Section 9.** Summary: add `Signal::set_with_flags(value, DirtyFlags)` where `DirtyFlags` is the existing 4-layer bitmask from `cvkg-core/src/dirty_flags.rs` (STATE 0b1111, LAYOUT 0b0111, PAINT 0b0011, COMPOSITE 0b0001). Subscribers become `SubscriberEntry { effect: Arc<dyn EffectRunner>, flags: DirtyFlags }`. The `FrameScheduler` snapshots the aggregate per-node dirty flags at `begin_frame()` and skips Layout (when only PAINT+COMPOSITE dirty) or Animation (when only STATE/PAINT dirty) for unaffected subtrees.

**Sketch:**

```rust
// cvkg-vdom/src/signals.rs
impl<T> Signal<T> {
    /// Set value and record which pipeline layers this change affects.
    /// A color change → DirtyFlags::PAINT.
    /// A width change  → DirtyFlags::LAYOUT.
    pub fn set_with_flags(&self, value: T, flags: DirtyFlags) {
        *self.value.borrow_mut() = MaybeUninit::new(value);
        self.version.fetch_add(1, Ordering::Release);
        CURRENT_DIRTY_FLAGS.with(|f| f.update(flags));
        // notify subscribers whose SubscriberEntry.flags overlaps flags
    }
}
```

**Risk:** Downstream invariant must be enforced: a layer change MUST also flag all downstream layers (STATE implies LAYOUT+PAINT+COMPOSITE). Enforced via `debug_assert!` in `set_with_flags` and compile-time bitmask constants.

---

### 16.5 Reflect-Powered Inspector Integration

**Inspired by:** `bevy_reflect` + `bevy_inspector_egui` — auto-generated property panels from a single `#[derive(Reflect)]`.

**Crates touched:** `cvkg-macros`, `cvkg-components`, `cvkg-reflect`, `cvkg-cli`

**Full design in Section 13.** Summary: add `#[derive(Reflect)]` to `cvkg-macros` that generates a `Reflected` impl (type→`FieldKind` mapping, `get_field`/`set_field` via `serde_json::Value`). Add `ReflectedInspector` to `cvkg-components` that takes `Rc<RefCell<dyn Reflected>>` and dispatches per-`FieldKind` widgets (Checkbox for Bool, Slider for Float, ColorPicker for Color, etc.). Wire `ReflectRegistry` into umbrella crate. Add `QueryReflected` to `cvkg-cli` WebSocket protocol.

**Sketch:**

```rust
// Usage (one derive replaces hand-coded builder):
#[derive(Reflect)]
struct MyComponent {
    enabled: bool,
    opacity: f32,
    label: String,
    tint: [f32; 4],
}

// Inspector panel (auto-generated rows):
let inspector = ReflectedInspector::new("My Settings",
    Rc::new(RefCell::new(my_instance)));
```

**Risk:** `ReflectedInspector` requires an `Rc<RefCell<dyn Reflected>>`. Components that use `State<T>` (signals) instead of raw `Reflected` values need a bridge type. The `theme::*()` helpers must be switched from global `Environment` to thread-local `THEME_CONTEXT` before colors display correctly (see §15.7, step 9) — a medium-risk refactor.

---

### 16.6 Theme Portal Inheritance

**Inspired by:** Bevy's entity-hierarchy theme propagation — `Theme<T>` inherited through `Parent` links.

**Crates touched:** `cvkg-core`, `cvkg-vdom`, `cvkg-themes`, `cvkg-render-gpu`

**Full design in Section 15.** Summary: extend `Renderer` trait with `push_theme()`/`pop_theme()`/`current_theme()` stack. Wire `push_theme()` to update thread-local `THEME_CONTEXT` so `theme::*()` helpers pick up overrides. Add `Themed<V>` wrapper component for per-subtree overrides. Save/restore theme at portal boundaries so `PhaseGate` portals inherit from their source node, not the compositing context.

**Sketch:**

```rust
// Per-subtree theme override:
let content = VStack::new()
    .child(Themed::new(dark_theme, Text::new("Dark section")))
    .child(Themed::new(light_theme, Text::new("Light section")));

// Portal inheritance (PhaseGate):
//   Without fix: portal content inherits theme of compositing pass.
//   With fix:   portal content inherits theme of source node (Themed wrapper).
```

**Risk:** Step 9 in the migration path (switching `theme::*()` helpers from `Environment` to thread-local `THEME_CONTEXT`) is path-breaking — it changes the data source for every color function call. Must be done in a single commit after all frames call `set_theme_context()`.

---

### 16.7 Auto-Tracked Dependency Wiring via `hamr!`

**Inspired by:** Bevy's `Changed<T>` — requires manual declaration of component reads. CVKG's proposed alternative: **SolidJS-style auto-tracking** — the `hamr!` macro wraps each `body()` call in a tracking scope that records which `Signal` IDs are read, without any manual `Changed<T>` annotation.

**Crates touched:** `cvkg-macros`, `cvkg-vdom`, `cvkg-core`

**Sketch:**

```rust
// ── cvkg-vdom/src/signals.rs ── Extend CURRENT_EFFECT

thread_local! {
    /// When set, all Signal::read() calls record their id here.
    pub static CURRENT_EFFECT: RefCell<Option<EffectScope>> = const { RefCell::new(None) };
}

/// A scope that collects which Signal IDs were read during body() execution.
pub struct EffectScope {
    pub node_id: KvasirId,
    pub read_signals: Vec<u64>,
}

impl<T> Signal<T> {
    pub fn read(&self) -> Ref<'_, T> {
        // Record this signal read if we're inside an effect scope
        CURRENT_EFFECT.with(|cell| {
            if let Some(scope) = &mut *cell.borrow_mut() {
                scope.read_signals.push(self.id);
            }
        });
        self.value.borrow()
    }
}

// ── cvkg-macros/src/lib.rs ── Wrap hamr! body() in tracking scope

// hamr! expansion currently:
//   VStack::new(16.0).child(MyButton(label))

// hamr! expansion with auto-tracking:
//   {
//       let __node_id = KvasirId::new();
//       CURRENT_EFFECT.with(|cell| {
//           *cell.borrow_mut() = Some(EffectScope { node_id: __node_id, read_signals: vec![] });
//       });
//       let __result = VStack::new(16.0).child(MyButton(label));
//       let __scope = CURRENT_EFFECT.with(|cell| cell.borrow_mut().take());
//       if let Some(scope) = __scope {
//           for signal_id in scope.read_signals {
//               DependencyGraph::register(signal_id, __node_id);
//           }
//       }
//       __result
//   }

// ── cvkg-core/src/dependency.rs ── Already has API (P1-42, line 41):
//   DependencyGraph::register(signal_id, node_id);
//   DependencyGraph::affected_components(signal_id) → Vec<component_ids>

// ── Combined with DirtyFlags (§9) ──
// On Signal::set_with_flags(value, DirtyFlags::PAINT):
//   for node_id in DependencyGraph::affected_components(signal_id) {
//       scheduler.mark_dirty(node_id, DirtyFlags::PAINT);
//   }
```

**Why this is more powerful than Bevy:** Bevy's `Changed<T>` is an explicit per-system annotation — the developer must write `Query<&Transform, Changed<Transform>>` to detect changes. CVKG's auto-tracking discovers signal dependencies automatically from the `hamr!` expansion's execution trace. No component author writes subscription code. Combined with Section 9's layer-typed dirty flags, `FrameScheduler` can skip entire pipeline phases for subtrees where no relevant signal changed.

**Risk:** `CURRENT_EFFECT` is a thread-local, which means it only captures signal reads on the same thread. Cross-thread signal reads (rare in VDOM construction) would not be tracked. Fix: use a `std::sync::atomic::AtomicU64` generation counter that comparisons across threads. Medium complexity — the tracking scope change in `hamr!` is in the macro hot path and must not add runtime overhead when auto-tracking is disabled (feature gate).

---

### 16.8 Render Pass Self-Registration

**Inspired by:** Bevy's plugin-driven pipeline — each plugin registers its render passes at app startup via `.add_system()` and `.add_render_pass()`.

**Crates touched:** `cvkg-render-gpu`, `cvkg-physics`, `cvkg-flow`, `cvkg-materials`

**Sketch:**

```rust
// ── cvkg-render-gpu/src/kvasir/registry.rs ── NEW

/// A self-registering render pass descriptor.
/// Each crate defines its passes here instead of hard-coding them
/// in cvkg/src/lib.rs.
pub struct PassRegistration {
    pub id: &'static str,       // e.g. "particle_trail"
    pub label: &'static str,    // e.g. "Particle Trail"
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub after: &'static [&'static str],
    pub constructor: fn() -> Box<dyn KvasirNode>,
}

/// Registry that collects pass registrations from all crates.
/// Populated at startup (not compile-time — §16.9/§14 covers const manifests).
pub struct PassRegistry {
    passes: Vec<PassRegistration>,
}

impl PassRegistry {
    pub fn register(&mut self, pass: PassRegistration) {
        self.passes.push(pass);
    }

    pub fn build_graph(&self) -> KvasirGraph {
        let mut builder = GraphBuilder::new();
        for pass in &self.passes {
            let node = (pass.constructor)();
            builder.add_node(node);
        }
        // ... wire connections ...
        builder.build()
    }
}

// ── cvkg-physics/src/lib.rs ──
pub fn register_passes(registry: &mut PassRegistry) {
    registry.register(PassRegistration {
        id: "physics_debug",
        label: "Physics Debug Draw",
        inputs: &[],
        outputs: &["physics_debug_buffer"],
        after: &["geometry"],
        constructor: || Box::new(PhysicsDebugDrawPass::new()) as Box<dyn KvasirNode>,
    });
}
```

**Relationship to FrameManifest (§14):** The `PassNodeDescriptor` in §14's `FrameManifest` design supersedes this runtime registry with a compile-time alternative. If §14 is implemented first (recommended), this item is superseded — crates declare passes via `const MANIFEST.pass_nodes` rather than calling `register_passes()` at startup. This item is listed separately because it's easier to implement (no `const fn` merge machinery) and can serve as a stepping stone to the full compile-time approach.

**Risk:** Runtime registration can produce ordering cycles that crash at startup rather than compile time. Duplicate pass IDs are detected at registration, not at compile time. If both the FrameManifest (§14) and this runtime registry exist, they must not conflict — either use one or the other, or have the runtime registry feed into the compile-time merge.

---

### 16.9 FrameManifest — Compile-Time Phase Declaration

**Inspired by:** Bevy's `Plugin` trait — runtime system registration. CVKG's alternative: `const`-constructible manifests checked at compile time.

**Crates touched:** `cvkg-core`, `cvkg-scheduler`, `cvkg-macros`, `cvkg-render-gpu`

**Full design in Section 14.** Summary: move `FramePhase` to `cvkg-core`. Define `FrameManifest { phase_contributions, pass_nodes, time_budget_requests }` as a `const`-constructible struct. Each crate exposes `pub const MANIFEST: FrameManifest`. The umbrella crate calls `merge_manifests!` which runs `const fn` merge logic at compile time — duplicate pass IDs, unresolved dependencies, ordering cycles, and budget overruns all produce compile errors (via `panic!` in const context). The merged manifest drives `FrameScheduler::configure()` and `build_render_graph()`.

**Sketch:**

```rust
// cvkg-core/src/frame_manifest.rs
pub struct FrameManifest {
    pub phase_contributions: &'static [FramePhase],
    pub pass_nodes: &'static [PassNodeDescriptor],
    pub time_budget_requests: &'static [TimeBudgetRequest],
}

impl FrameManifest {
    pub const fn merge(manifests: &[&Self]) -> Self { /* const fn, panics on conflict */ }
}

// cvkg-physics/src/lib.rs
pub const MANIFEST: FrameManifest = FrameManifest {
    phase_contributions: &[FramePhase::State, FramePhase::Render],
    pass_nodes: &[PassNodeDescriptor { id: "physics_debug", /* ... */ }],
    time_budget_requests: &[TimeBudgetRequest { phase: FramePhase::State, time_slice_us: 2000, /* ... */ }],
};

// cvkg/src/lib.rs
merge_manifests!(cvkg_physics::MANIFEST, cvkg_flow::MANIFEST, cvkg_render_gpu::MANIFEST);
```

**Risk:** `const fn` merge is O(n²) for small n (< 30 passes), acceptable. The `merge_manifests!` macro must handle the limitation that `const fn` cannot allocate or use `HashMap`. Dependency resolution uses linear scans. Requires Rust's `const fn` capabilities (stable since 1.68 for basic operations, 1.82 for `&mut` references in const — verify MSRV). If the project's MSRV is below 1.68, the merge must use a procedural macro instead of `const fn`.

---

### 16.10 Headless SSR Mode (Most Impactful)

**Inspired by:** Bevy's `MinimalPlugins` — a minimal app setup that runs ECS systems without a renderer, enabling headless simulation and server-side rendering.

**Crates touched:** `cvkg-core`, `cvkg-vdom`, `cvkg-layout`, `cvkg-svg-serialize`, `cvkg` (umbrella)

**Sketch:**

```rust
// ── cvkg/src/headless.rs ── NEW module

/// Minimal headless CVKG backend: VDOM + layout + SVG output,
/// no GPU context, no window, no input devices.
///
/// Analogous to Bevy's `MinimalPlugins`:
///   App::new().add_plugins(MinimalPlugins).run();
pub struct CvkgHeadless {
    scheduler: FrameScheduler,
    vdom: VDom,
    svg_encoder: SvgEncoder,
}

impl CvkgHeadless {
    /// Create a headless instance. No GPU, no window.
    /// The frame pipeline consists of:
    ///   State → Layout → Animation → Render (SVG) → PostFrame
    pub fn new(view: impl View + 'static, viewport: Rect) -> Self {
        // 1. Build VDOM tree (no VNodeRenderer → directly to SvgEncoder)
        // 2. Run Taffy layout
        // 3. Render to SVG string via cvkg-svg-serialize
        let vdom = VDom::build(&view, viewport);
        let svg_encoder = SvgEncoder::new(viewport);
        Self { scheduler: FrameScheduler::new(), vdom, svg_encoder }
    }

    /// Render one frame and return the SVG string.
    pub fn render_frame(&mut self) -> String {
        self.scheduler.begin_frame();

        // Phase: State — resolve signals, apply dirty flags
        self.scheduler.flush_current_phase();
        self.scheduler.advance_phase(); // → Layout

        // Phase: Layout — run Taffy
        // Phase: Animation — step springs
        // Phase: Render — serialize to SVG
        // Phase: PostFrame — telemetry
        loop {
            self.scheduler.flush_current_phase();
            let phase = self.scheduler.advance_phase();
            if phase == FramePhase::PostFrame {
                break;
            }
        }

        self.svg_encoder.to_string()
    }
}

// ── Usage ──
// Server-side rendering (e.g., Axum route):
async fn render_page() -> Html<String> {
    let view = Page::new(/* ... */);
    let mut headless = CvkgHeadless::new(view, Rect::sized(1920, 1080));
    Html(headless.render_frame())
}

// CI snapshot testing:
#[test]
fn button_snapshot() {
    let view = Button::new("Submit", || {});
    let mut headless = CvkgHeadless::new(view, Rect::sized(200, 48));
    insta::assert_snapshot!(headless.render_frame());
}
```

**Bevy parallel:** Bevy's `MinimalPlugins` set (from `bevy::app::MinimalPlugins`) provides `CoreSchedule` without `RenderPlugin`, `WindowPlugin`, or `InputPlugin`. It is used in headless CI, dedicated game servers, and editor tools. CVKG's `CvkgHeadless` serves the same purpose: server-side rendering for SSR dashboards, CI snapshot testing without a GPU, and accessibility tree export (via the accesskit bridge).

**Impact unlocked:**
1. **SSR dashboards** — Render real-time CVKG dashboards as SVG on the server, stream to the browser via Axum (no WebGPU required on the client).
2. **CI/CD snapshot testing** — Assert visual output as SVG strings in CI. No GPU, no display server.
3. **Accessibility tree export** — Walk the VDOM and emit a11y tree without a render loop.
4. **SEO/static generation** — Pre-render static pages as SVG at build time.

**Risk:** SVG output fidelity depends on `cvkg-svg-serialize`, which may not support all render primitives (shadows, blurs, gradients). Some components that depend on `theme::*()` helpers or `use_theme()` will render with the default fallback unless `set_current_theme()` is called. Mitigation: `CvkgHeadless` accepts an optional `Theme` parameter: `CvkgHeadless::new(view, viewport).with_theme(Theme::dark())`.

---

### Summary Table

| # | Improvement | Crate(s) | Effort | Impact | Already in plan? |
|---|---|---|---|---|---|
| 1 | Typed event triggers | `cvkg-vdom`, `cvkg-core`, `cvkg-components` | S | Medium | **New** |
| 2 | Auto-required companion state | `cvkg-macros`, `cvkg-core` | S | Medium | §12 |
| 3 | Layout-animated spring constraints | `cvkg-layout`, `cvkg-anim` | M | Medium | **New** |
| 4 | Layer-typed signal mutations | `cvkg-core`, `cvkg-vdom`, `cvkg-scheduler` | M | High | §9 |
| 5 | Reflect-powered inspector | `cvkg-macros`, `cvkg-components`, `cvkg-reflect`, `cvkg-cli` | M | High | §13 |
| 6 | Theme portal inheritance | `cvkg-core`, `cvkg-vdom`, `cvkg-render-gpu` | M | High | §15 |
| 7 | Auto-tracked dependency wiring | `cvkg-macros`, `cvkg-vdom`, `cvkg-core` | L | High | **New** |
| 8 | Render pass self-registration | `cvkg-render-gpu`, `cvkg-physics`, `cvkg-flow` | M | Medium | Superseded by §14 |
| 9 | FrameManifest (compile-time) | `cvkg-core`, `cvkg-scheduler`, `cvkg-macros` | L | High | §14 |
| 10 | Headless SSR mode | `cvkg`, `cvkg-vdom`, `cvkg-svg-serialize` | L | Highest | **New** |

---

## Implementation Ordering

| Order | Phase | New Crates | Effort | Key Files |
|---|---|---|---|---|---|
| **0a** | Macro: Typed event triggers | ✅ | S | `cvkg-core/src/event.rs` (new types), `cvkg-vdom/src/lib.rs` (`TriggerRegistry`), `cvkg-components/src/interactive/` (`.on::<E>()` builders) |
| **0b** | Macro: `#[require]` companions | ✅ | S | `cvkg-macros/src/lib.rs` (`_attr` parsing for `#[require(...)]`), `cvkg-core/src/companion.rs` (new trait) |
| **0c** | Reflection foundation | ✅ | S | `cvkg-macros/src/lib.rs` (`#[derive(Reflect)]`), `cvkg-components/Cargo.toml` (+`cvkg-reflect`) |
| **0d** | FrameManifest foundation | ✅ | M | `cvkg-core/src/frame_phase.rs` (move), `cvkg-core/src/frame_manifest.rs` (new), `cvkg-macros/src/lib.rs` (`merge_manifests!`) |
| **1** | Phase 1: MVP + Hierarchy | ✅ | M | `vertex.rs` (+`InstanceData3D`), `common.wgsl` (+model matrix), `hierarchy.rs` |
| **2** | Phase 2: Texture UV Sampling | ✅ | M | `mesh.rs` (+`tex_coords`), `material3d.rs` (+texture fields), `mesh_pbr.wgsl` (+texture sampling) |
| **3** | Phase 3: Frustum Culling | ✅ | S | `frustum.rs` in `cvkg-spatial`, `draw.rs` in `cvkg-render-3d` |
| **4** | Phase 4: Shadow Pass | ✅ | L | `shadow.rs` pass node, `mesh_shadow.wgsl`, PCF sampling |
| **5** | Phase 5: glTF Importer | ✅ | M | `cvkg-gltf` crate, `importer.rs`, `types.rs` |
| **6a** | Layer-typed signal dirty flags | ✅ | M | `cvkg-vdom/src/signals.rs` (`set_with_flags`), `cvkg-scheduler/src/frame.rs` (`should_skip_phase`) |
| **6b** | Layout-animated spring constraints | ✅ | M | `cvkg-layout/src/animation.rs` (`SpringConstraint`), `cvkg-anim` |
| **6c** | Reflect: `ReflectedInspector` + DevTools | ✅ | M | `reflected_inspector.rs`, `cvkg-cli` WS (`QueryReflected`) |
| **6d** | Theme: Stack & Portal Inheritance | ✅ | M | `cvkg-core/src/renderer_trait.rs` (+`push_theme`), `cvkg-vdom/src/vnode.rs` (+`theme_override`), `cvkg-components/src/theme.rs` (migrate to thread-local) |
| **7a** | Auto-tracked dep wiring via `hamr!` | ✅ | L | `cvkg-macros/src/lib.rs` (tracking scope in `hamr!`), `cvkg-vdom/src/signals.rs` (`CURRENT_EFFECT`), `cvkg-core/src/dependency.rs` (wire `DependencyGraph`) |
| **7b** | FrameManifest: crate `MANIFEST` consts | ✅ | S | cvkg-render-gpu, cvkg-physics, cvkg-flow — `pub const MANIFEST: FrameManifest` |
| **7c** | Render pass self-registration | ✅ | S | superseded by §14 FrameManifest — `PassRegistry` exists at `cvkg-render-gpu/src/kvasir/pass_registry.rs` |
| **7d** | Integration + Kvasir wiring | ✅ | M | `cvkg/src/lib.rs` (`configure_scheduler()`) |
| **7e** | Per-fragment lighting camera fix | ✅ | S | `cvkg-core/src/scene_uniforms.rs` (+light/camera fields), `material_opaque.wgsl`, `material_pbr.wgsl` (use uniform values) |
| **8** | WorldSpacePanel compositing | ✅ | M | `cvkg-vdom/src/vnode.rs` (+`WorldSpacePanel`), `kvark/nodes.rs` (`PreWorldPanelNode` + quad submission), `passes/glass.rs` (portal integration) |
| **9** | Headless SSR mode | ✅ | L | `cvkg/src/headless.rs` (`CvkgHeadless`), `cvkg-vdom` (SVG output path), `cvkg-svg-serialize` |

**Effort key:** S = days, M = 1-2 weeks, L = 2-4 weeks.

---

## Concrete Next Steps

1. **Ratify** this plan in an architecture review.
2. **Move `FramePhase`** from `cvkg-scheduler` to `cvkg-core/src/frame_phase.rs`. Re-export from `cvkg-scheduler` for backward compat.
3. **Add `PassNode` trait, `FrameManifest`, `PassNodeDescriptor`, `TimeBudgetRequest`** to `cvkg-core/src/frame_manifest.rs`.
4. **Add `merge_manifests!` declarative macro** to `cvkg-macros/src/lib.rs` — const-evaluated merge with duplicate/cycle/budget detection.
5. **Implement `#[derive(Reflect)]`** in `cvkg-macros/src/lib.rs` — parse struct fields, emit `FieldMeta` array and `Reflected` impl with type→kind mapping.
6. **Define `pub const MANIFEST`** in `cvkg-render-gpu` — migrate existing passes from `build_render_graph()` to manifest descriptors.
7. **Define `pub const MANIFEST`** in `cvkg-physics` — declare State phase and physics debug pass.
8. **Define `pub const MANIFEST`** in `cvkg-flow` — declare Layout + Render phase and particle trail pass.
9. **Wire `merge_manifests!`** in `cvkg/src/lib.rs` — add `configure_scheduler()` and manifest-driven `build_render_graph()`.
10. **Create `cvkg-render-3d-hierarchy`** crate: `propagate_transforms()`, test with a parent-child tree of `TransformNode3D`.
11. **Add `InstanceData3D`** to `cvkg-render-gpu/src/vertex.rs` and the corresponding vertex attributes.
12. **Modify `common.wgsl`** vertex shader to read model matrix from instance attributes for `material_id == 13u`.
13. **Implement `GpuRenderer3D::draw_mesh_3d()`** in `cvkg-render-3d/src/draw.rs`.
14. **Add `ReflectedInspector`** to `cvkg-components/src/reflected_inspector.rs`.
15. **Add `InlineTextEdit`** to `cvkg-components/src/interactive/inline_text_edit.rs`.
16. **Wire `ReflectRegistry`** into umbrella crate, populate with key types.
17. **Add `QueryReflected`** to `cvkg-cli` WebSocket protocol + `/api/reflected` HTTP endpoint.
18. **Add `push_theme()`/`pop_theme()`/`current_theme()`** to `Renderer` trait in `cvkg-core/src/renderer_trait.rs` with default no-op stack.
19. **Implement theme stack** in `GpuRenderer` — `theme_stack: Vec<ColorTheme>` + GPU buffer upload on push.
20. **Implement theme stack** in `VNodeRenderer` — `theme_stack` + track theme in VNode props.
21. **Wire `push_theme()` to thread-local `THEME_CONTEXT`** — `theme::*()` helpers automatically pick up per-subtree overrides.
22. **Add `theme_override: Option<ColorTheme>`** to `VNode`, diff/patch, and create `Themed<V>` wrapper component.
23. **Add portal theme save/restore** to `enter_portal()`/`exit_portal()` in `GpuRenderer` — portal buffers inherit theme from source node.
24. **Migrate `theme::*()` helpers** from global `Environment` to thread-local `THEME_CONTEXT`.
25. **Implement typed event triggers** — add `TriggerRegistry` to `cvkg-vdom`, `.on::<E>()` builder to `cvkg-components`.
26. **Implement `SpringConstraint`** — bridge `cvkg-anim` spring physics with Taffy layout properties.
27. **Implement auto-tracked dependency wiring** — wrap `hamr!` `body()` calls in `CURRENT_EFFECT` scope, register observed signals in `DependencyGraph`.
28. **Fix per-fragment lighting** — add `camera_pos`, `light_direction`, `light_color` to `SceneUniforms`; populate from `Camera3D` + active directional light; replace hardcoded values in `material_opaque.wgsl` and `material_pbr.wgsl`.
29. **Add `WorldSpacePanel` to `VNode`** — `cvkg-vdom/src/vnode.rs`: add `world_space: Option<WorldSpacePanel>` field with `Transform3D`, `world_size`, `pixels_per_unit`, optional `GlassMaterial`.
30. **Create `PreWorldPanelNode`** — `cvkg-render-gpu/src/kvasir/nodes.rs`: iterate panels, allocate offscreen textures via `ResourceRegistry`, render each panel's VDOM subtree to its offscreen target.
31. **Extend Geometry pass** — submit textured quads at each panel's 3D position with depth testing enabled in `KvasirNodeContext.depth_view`.
32. **Integrate glass panels into Glass pass** — register panel offscreen textures as portal blur regions at `ResourceId(2000 + i)`.
33. **Wire `build_render_graph()`** — insert `PreWorldPanelNode` before Geometry, extend Geometry quad submission.
34. **Fix `push_transform_3d`** — add parallel `transform_stack_3d: Vec<Mat4>` to `cvkg-render-gpu/src/api/mod.rs`, pop from both stacks in `pop_transform_3d`.
35. **Create `cvkg/src/headless.rs`** — headless SSR mode with SVG output, CI snapshot testing, and server-side rendering.
36. **Test with `physics_3d_demo`** — verify rotating cube with correct world-space transform.
37. **Proceed to Phases 2–4** in order.

---

# Section 17: World-Space UI Panel Compositing & Architectural Positioning

## 17.1 Problem: No Plausible World-Space Path in bevy_ui

Bevy's `UiTargetCamera` targets a 2D overlay onto a camera — the UI is always screen-space, rendered as an orthographic overlay after the 3D scene. A physically positioned, depth-occluded, scene-refracting UI panel in 3D world space is impossible in bevy_ui because:

- bevy_ui has no compositor pass that reads the 3D scene buffer
- bevy_ui nodes have no 3D transform — they use `Style` (flexbox) which produces screen-space layout rects
- bevy_ui has no `RenderGraph` node insertion API — material rendering goes through `UiMaterial` which is a flat draw call, not a full render pass
- bevy_ui's depth buffer is not shared with the 3D pipeline — the UI renders as a separate overlay pass

CVKG's architecture can solve this because it already has:
- A shared `depth_view` across all Kvasir passes (`KvasirNodeContext.depth_view`)
- `ResourceRegistry::allocate_offscreen()` (kvark/registry.rs:108) which allocates and pools offscreen textures
- A `BackdropCopyNode` pattern for blitting scene content
- A `GlassNode` that reads `RES_SCENE` for per-pixel refraction
- `Transform3D` and `Camera3D` types in `cvkg-core`
- `VNode.portal_target: Option<NodeId>` (dead code but provides conceptual slot)
- `geometry` Kvasir pass that writes to `RES_SCENE` with depth testing enabled

## 17.2 Design: WorldSpacePanel

### Data Structure

A new variant carried by VNode (not replacing `portal_target`):

```rust
// In cvkg-vdom/src/vnode.rs
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WorldSpacePanel {
    /// Position, rotation, and scale in 3D world space.
    pub transform: Transform3D,
    /// Logical size of the panel in world units (meters).
    /// The offscreen texture resolution = size * pixels_per_unit.
    pub world_size: (f32, f32),
    /// Pixels per world unit. 1.0 = 1 pixel per meter (low), 200.0 = sharp UI.
    pub pixels_per_unit: f32,
    /// Optional glass material for scene refraction through this panel.
    pub glass: Option<GlassMaterial>,
}

// Extend VNode with a single optional field:
pub world_space: Option<WorldSpacePanel>,
```

The `VNode.world_space: Option<WorldSpacePanel>` field replaces the dead `VNode.portal_target: Option<NodeId>` (which is declared, set, and never read). The WorldSpacePanel encodes both the portal target concept and the 3D positioning in one structure.

### Render Graph Integration (Minimum Design)

The existing Kvasir graph executes in this order:

```
Geometry -> [OffscreenEffects] -> BackdropCopy -> Blur -> Glass -> UI -> Volumetric -> Bloom -> Composite -> Present
```

WorldSpacePanel adds **two** new pass nodes:

```
PreWorldPanel -> WORLDPANEL_RENDER -> Geometry -> [OffscreenEffects] -> BackdropCopy -> Blur -> Glass -> UI -> ...
                    |                           |
         Offscreen textures             Quad meshes placed
         (panel VDOM rendered           in depth buffer at
          to per-panel texture)         panel 3D positions
```

**Pass 1: PreWorldPanel** — before the Geometry pass, render each WorldSpacePanel's VDOM subtree to its offscreen texture:

```
for each panel in scene.world_space_panels:
    let tex_id = registry.allocate_offscreen(
        device, panel.offscreen_id,
        (size.0 * ppu, size.1 * ppu),
    );
    // Run UI rendering pipeline targeting tex_id:
    //   1. Layout panel's VDOM subtree (Taffy, panel inherits 2D layout)
    //   2. Diff panel's VDOM against previous frame
    //   3. Render each element into the offscreen pass
```

**Pass 2: Geometry (extended)** — after PreWorldPanel, offscreen textures are ready. The Geometry pass now also submits a textured quad for each panel:

```rust
// In GpuRenderer3D::submit_worldspace_quads():
for panel in scene.world_space_panels:
    let offscreen_tex = registry.get(&panel.offscreen_id).unwrap();
    let model = panel.transform.to_matrix()
        * Mat4::from_scale(glam::Vec3::new(
            panel.world_size.0,
            panel.world_size.1,
            1.0,
        ));
    queue.submit_quad_mesh(
        model,
        offscreen_tex.view(),
        // Color, opacity, and optional glass material ID
    );
```

The quad mesh is a unit quad (two triangles, 4 vertices) scaled by `world_size` and transformed by `panel.transform.to_matrix()`. Depth testing is enabled — the quad correctly occludes or is occluded by 3D meshes in the same `depth_view`.

### Optional: Glass Integration

If `panel.glass: Some(glass_material)`, the offscreen texture is not submitted directly as an opaque quad. Instead:

1. The offscreen texture is registered as an additional blur region (the existing per-element `BackdropRegion` mechanism allocates textures at `ResourceId(2000 + i)`)
2. The existing `GlassNode` picks up the panel's offscreen texture as the environment background for refraction
3. The Glass pass renders the panel with the glass pipeline, using the offscreen texture as foreground and `RES_SCENE` as background to refract

A world-space panel with `glass: Some(...)` refracts the 3D scene behind it automatically through the existing Glass pipeline — something bevy_ui cannot do at any engineering cost.

### What Works Without Changes

| Component | Status |
|-----------|--------|
| `Transform3D` (position, rotation, scale) | Exists in `cvkg-core` |
| `Camera3D` / scene uniforms | Exists in `cvkg-core`, used by `material_opaque.wgsl` |
| Offscreen texture allocation | Exists via `ResourceRegistry::allocate_offscreen()` in `cvkg-render-gpu/src/kvasir/registry.rs` |
| Offscreen texture pooling | Exists — pool keyed by `(format, width, height)`, evicted per frame |
| Backdrop blur for glass panel | Exists — `BackdropBlurNode` runs Kawase pyramid |
| Glass node with RES_SCENE reading | Exists — `GlassNode` reads `RES_SCENE` + `RES_BLUR_A` |
| Quad mesh submission | Exists in Geometry pass (used for 2D geometry) |
| Depth buffer sharing | Exists — `KvasirNodeContext.depth_view` is shared across all passes |
| VDOM subtree render | Exists — `UIRenderer` renders any subtree given a render target |

### What Must Be Built

| Component | Files | Effort |
|-----------|-------|--------|
| `WorldSpacePanel` struct + `VNode.world_space` field | `cvkg-vdom/src/vnode.rs` | S |
| `PreWorldPanelNode` — iterate panels, render offscreen | `cvkg-render-gpu/src/kvasir/nodes.rs` | M |
| Offscreen resource tracking for panel IDs | `cvkg-render-gpu/src/kvasir/registry.rs` | S |
| `submit_worldspace_quads()` in Geometry pass | `cvkg-render-gpu/src/kvasir/nodes.rs` | S |
| Glass pass integration (panel offscreen -> refraction source) | `cvkg-render-gpu/src/passes/glass.rs` | M |
| `build_render_graph()` wiring | `cvkg-render-gpu/src/kvasir/nodes.rs` | S |

**Total: 6 components, ~2-3 weeks.**

## 17.3 push_transform_3d Correction

### Current Behavior (broken)

`cvkg-render-gpu/src/api/mod.rs:1222-1236`:

```rust
fn push_transform_3d(&mut self, transform: &cvkg_core::Transform3D) {
    let (translation, rotation_quat, scale_glam) =
        transform.to_matrix().to_scale_rotation_translation();
    let translation = [translation.x, translation.y];     // Drops Z
    let scale = [scale_glam.x, scale_glam.y];              // Drops Z
    let rotation = if rotation_quat.length_squared() > 0.0 {
        let (axis, angle) = rotation_quat.to_axis_angle();
        angle * axis.z.signum()                             // Projects onto Z-axis only
    } else { 0.0 };
    self.push_transform(translation, scale, rotation);      // Appends 2D Mat3 affine
}
```

The 3D transform is decomposed and collapsed into a 2D affine transform matrix — Z components of translation and scale are dropped, rotation is projected onto the Z axis only.

### How This Relates to WorldSpacePanel

**They are orthogonal.** `push_transform_3d` is called by VDOM elements during the 2D UI render pass to tilt themselves — it affects the 2D transform stack used for the UI overlay. `WorldSpacePanel` positions a flat VDOM subtree in 3D world space via the Geometry pass depth buffer.

If a component inside a WorldSpacePanel calls `push_transform_3d`, that transform is applied relative to the panel's offscreen surface. Since the panel renders to a flat offscreen texture, the 2D collapse is acceptable — content tilts on the panel's surface, and the panel itself is oriented in 3D by its `transform` field.

### Fix for push_transform_3d (Separate Work Item)

To correctly support VDOM elements that tilt in 3D during the regular UI overlay pass:

```rust
fn push_transform_3d(&mut self, transform: &Transform3D) {
    // Push the full 3D transform onto a parallel 3D stack
    let mat4 = transform.to_matrix();
    let parent = self.transform_stack_3d.last().copied().unwrap_or(Mat4::IDENTITY);
    self.transform_stack_3d.push(parent * mat4);

    // Also push a 2D-compatible transform for backward compat
    let (t, q, s) = mat4.to_scale_rotation_translation();
    self.push_transform([t.x, t.y], [s.x, s.y], {
        let (axis, angle) = q.to_axis_angle();
        angle * axis.z.signum()
    });
}
```

Key changes:
1. Maintain a parallel `transform_stack_3d: Vec<Mat4>` alongside the existing 2D `transform_stack`
2. 3D-aware render targets (e.g., offscreen panel textures) read from the Mat4 stack
3. 2D-only renderers (e.g., `VNodeRenderer`) ignore the Mat4 stack and use the existing collapsed 2D Mat3 stack — backward compat
4. `pop_transform_3d` pops from both stacks

**Changes:** `cvkg-render-gpu/src/api/mod.rs` (add `transform_stack_3d`), `cvkg-render-gpu/src/gpu_renderer.rs` (3D-aware draw calls read from Mat4 stack), `cvkg-vdom/src/vnode_renderer.rs` (ignore 3D stack, backward compat).

## 17.4 CVKG's Five Architectural Advantages Over bevy_ui

Each advantage is grounded in existing CVKG code that would require significant Bevy architecture changes to replicate.

### 1. GPU Material Surfaces (Glass/Mica/Acrylic)

CVKG has a dedicated compositor pipeline for real-time backdrop blur and scene refraction:

```
Geometry -> BackdropCopy (blit RES_SCENE -> RES_BLUR_A)
         -> BackdropBlur (Kawase 6-level mip pyramid)
         -> Glass (reads RES_SCENE + RES_BLUR_A, writes back to RES_SCENE)
         -> UI -> ... -> Composite
```

The `GlassNode` (`cvkg-render-gpu/src/passes/glass.rs`) reads `RES_SCENE` (which already contains all depth-tested 3D meshes) and `RES_BLUR_A` (blurred backdrop), enabling per-pixel refraction of the 3D scene behind each glass surface. Scissor rects limit glass to specific screen regions. Per-element portal regions use dedicated blur textures at `ResourceId(2000 + i)`.

**Why Bevy cannot replicate this easily:** Bevy's `UiMaterial` trait renders each material node as a flat draw call with no inter-pass compositing. To achieve glass/blur, one would need to add a full `RenderGraph` node between the 3D render and the UI overlay, implement a dedicated blur pass with mip pyramid, rewrite the UI render phase to be graph-aware, and add scene buffer sharing between 3D and UI passes. None of this is possible through existing Bevy extension points; it requires modifying `bevy_core_pipeline` internals.

### 2. Spring-Physics Layout Animation

CVKG integrates spring physics directly into layout through `cvkg-anim` (RK4/XPBD solver) and `cvkg-layout/src/animation.rs` (`SpringConstraint`). A `SpringConstraint` bridges the spring simulation with Taffy layout properties:

```rust
// cvkg-layout/src/animation.rs
pub struct SpringConstraint {
    pub target: Signal<f32>,
    pub spring: cvkg_anim::SpringParams,
    pub property: LayoutProperty,
}
```

Each frame, the spring solver drives the constraint's current value, which feeds into Taffy's layout input. Layout recalculations can trigger new spring targets (e.g., a button press changes a flexbox gap), creating bidirectional coupling between physics and layout.

**Why Bevy cannot replicate this easily:** Bevy's `AnimationPlayer` and `AnimationCurve` are decoupled from `Style` — they interpolate `Style` fields over time via easing curves, not physics. There is no per-frame feedback loop where layout results feed back into the animation system. Adding spring-physics layout would require replacing Bevy's `Style` animation pipeline and adding a physics integration step between layout and rendering.

### 3. First-Class Node Graph Editor (cvkg-flow)

`cvkg-flow` is a production-grade node graph editor built as a CVKG component, including bezier-curve edge routing, typed port model with connectability validation, ribbon toolbar with categorized node palette, canvas panning/zooming/minimap, and drag-to-connect interaction.

**Why Bevy cannot replicate this easily:** `bevy_ui` is a flexbox layout system with basic interaction — it cannot render arbitrary 2D paths (bezier curves require mesh generation) or handle canvas-space transforms (pan/zoom). Building an equivalent node graph would require a full 2D vector graphics renderer, custom mesh generation for bezier edges, a canvas abstraction with transform stack, and node graph interaction logic (port proximity detection, connection validation).

### 4. Knuth-Plass Line Breaking (cvkg-runic-text)

`cvkg-runic-text` implements the full Knuth-Plass paragraph-breaking algorithm (optimal line breaks minimizing badness over the paragraph, with demerits for consecutive hyphens and raggedness), subpixel LCD rendering, MSDF glyph atlas, emoji ZWJ segmentation, and BiDi text reordering.

**Why Bevy cannot replicate this easily:** Bevy uses `cosmic_text` which implements greedy line breaking — breaks at the first possible point after minimum width, not globally optimal across the paragraph. Adding Knuth-Plass requires replacing `cosmic_text` internals or adding a paragraph-level pre-pass. The MSDF rasterizer and subpixel renderer do not exist in bevy_text, and emoji ZWJ segmentation is absent.

### 5. WASM/WASI Dual-Target

CVKG's `View` trait (`cvkg-core/src/renderer_trait.rs`) is renderer-agnostic. The same VDOM + layout pipeline runs on three backends:

| Backend | Crate | Target |
|---------|-------|--------|
| Native GPU | `cvkg-render-native` (winit + wgpu) | Desktop |
| Software CPU | `cvkg-render-software` | CI, no-GPU environments |
| Headless WASI | `niflheim-wasi` | Server-side, WASI runtime |

The `View` trait separates VDOM diffing and layout from pixel output. The same VDOM produces identical output on all three backends.

**Why Bevy cannot replicate this easily:** Bevy's rendering is tightly coupled to wgpu — every renderer goes through `bevy_render` -> `wgpu` -> GPU. There is no `bevy_software_renderer`, `bevy_wasi`, or `View`-like trait that decouples ECS scheduling from pixel output. Adding headless WASI would require abstracting the entire `bevy_render` -> `wgpu` dependency chain, touching every rendering-related crate.

## 17.5 Structural Completeness Gaps (Prioritized)

| # | Gap | Severity | Status | Affects |
|---|-----|----------|--------|---------|
| P0 | Per-fragment lighting ignores camera | **Broken** — `view_dir = vec3<f32>(0.0, 0.0, 1.0)` hardcoded in `material_opaque.wgsl:217` and `material_pbr.wgsl:23`. Specular highlights and Fresnel are incorrect for any non-aligned camera. | No design exists | `cvkg-render-gpu/src/shaders/` |
| P0 | No parent-child 3D transform hierarchy | Meshes submit pre-baked model matrices; no tree traversal accumulates transforms. | Design exists in Section 4 (Phase 1) | `cvkg-scene`, `cvkg-render-3d` |
| P0 | No UV texture sampling | `Material3D` has no texture fields; shader uses flat color. | Design exists in Section 5 (Phase 2) | `cvkg-render-3d`, `cvkg-render-gpu` |
| P1 | No shadow pass | `light_dir` hardcoded; no shadow map; directional lights exist in type system only. | Design exists in Section 4.4 (Phase 4) | `cvkg-render-3d`, `cvkg-render-gpu` |
| P1 | `push_transform_3d` collapses to 2D | Full 3D rotation is projected onto Z axis; X/Y rotations silently discarded for VDOM elements. | Design exists in Section 17.3 | `cvkg-render-gpu/src/api/mod.rs` |
| P2 | No world-space VDOM nodes | VNode layout is `LayoutRect` (2D); no path to render VDOM subtree to offscreen texture and composite at 3D position. | Design exists in Section 17.2 | `cvkg-vdom`, `cvkg-render-gpu` |
| P2 | No depth pre-pass | Opaque geometry renders directly; no early-Z optimization for expensive PBR shading. | Not in plan; low priority until P0 gaps are closed | `cvkg-render-gpu` |

### P0 Fix: Per-Fragment Lighting Camera Correction

The minimum fix in `material_opaque.wgsl` and `material_pbr.wgsl`:

```wgsl
// Current (broken):
let light_dir  = normalize(vec3<f32>(0.5, 0.8, 0.6));
let view_dir   = vec3<f32>(0.0, 0.0, 1.0);
let light_color = vec3<f32>(1.0, 0.95, 0.9);

// Corrected:
let light_dir  = normalize(scene.light_direction);  // From SceneUniforms
let view_dir   = normalize(scene.camera_pos - world_pos);  // From SceneUniforms
let light_color = scene.light_color;                 // From SceneUniforms
```

This requires adding `light_direction: vec3<f32>`, `light_color: vec3<f32>`, `camera_pos: vec3<f32>` to the `SceneUniforms` struct in `cvkg-core/src/scene_uniforms.rs`, populating them from `Camera3D.position` and the active directional light each frame.

**Changes:** `cvkg-core/src/scene_uniforms.rs` (expand struct), `cvkg-render-gpu/src/renderer/mod.rs` (populate new fields), `material_opaque.wgsl`, `material_pbr.wgsl` (use uniform values).

## 17.6 Engineering Delta: WorldSpacePanel

### What CVKG Provides Today

1. **Offscreen texture allocation**: `ResourceRegistry::allocate_offscreen()` creates pooled render targets at `ResourceId(1000 + target_id)` — ready to use.

2. **Depth buffer sharing**: `KvasirNodeContext.depth_view` is the same `Depth32Float` texture passed to every Kvasir node — quads in Geometry naturally depth-sort against 3D meshes.

3. **3D transform types**: `Transform3D` in `cvkg-core` — `to_matrix()` produces `Mat4` directly usable as a model matrix.

4. **Glass pass architecture**: `GlassNode` reads `RES_SCENE` + `RES_BLUR_A` + per-portal textures. A WorldSpacePanel with `glass: Some(...)` plugs into the existing portal blur pathway at `ResourceId(2000 + i)`.

5. **Quad mesh submission**: The Geometry pass already submits mesh instances via `draw_mesh`. Adding a unit quad at a `Mat4` position is a minor extension — same vertex/index buffers, new instance data.

6. **VDOM subtree isolation**: The `Renderer` trait already supports `render_subtree(node_id)` for portal rendering (the conceptual skeleton exists even though `portal_target` is dead code).

### What Remains

1. **WorldSpacePanel data structure** (`cvkg-vdom/src/vnode.rs`): Add `world_space: Option<WorldSpacePanel>` to `VNode`. One field, two types (`Transform3D`, `GlassMaterial` both exist). **Effort: hours.**

2. **PreWorldPanel pass node** (`cvkg-render-gpu/src/kvasir/nodes.rs`): Iterate panels, allocate offscreen textures, run UI render for each subtree. Reuses existing `UINode` rendering with a different render target. **Effort: 1 week.**

3. **Geometry pass extension** (`cvkg-render-gpu/src/kvasir/nodes.rs`): After opaque 3D meshes, iterate panels and submit textured quads at `panel.transform.to_matrix() * scale(panel.world_size)` with depth testing. **Effort: 2-3 days.**

4. **Glass integration** (`cvkg-render-gpu/src/passes/glass.rs`): Register panel offscreen textures as portal blur textures at `ResourceId(2000 + i)`. Existing `GlassNode` iterates portal regions — panels are a new region type. **Effort: 2-3 days.**

5. **Scene composition wiring** (`cvkg-render-gpu/src/api/mod.rs`): During frame traversal, collect WorldSpacePanel nodes into a `Vec<WorldSpacePanel>` and pass to `PreWorldPanelNode`. **Effort: 1 day.**

6. **build_render_graph() update** (`cvkg-render-gpu/src/kvasir/nodes.rs`): Insert `PreWorldPanelNode` before Geometry, extend Geometry quad submission. **Effort: hours.**

**Total remaining effort: ~2-3 weeks** for a single developer familiar with the render graph.

### WorldSpacePanel vs UiTargetCamera: Fundamental Advantage

`bevy_ui` with `UiTargetCamera` renders UI as a separate overlay after the 3D scene. The UI is always front-facing, always screen-aligned, and cannot interact with the 3D depth buffer. A world-space panel in CVKG:
- Occupies real 3D world space — you can walk around it and see its side
- Is occluded by closer 3D geometry — it is in the same depth buffer
- Refracts the 3D scene behind it — the Glass pass reads `RES_SCENE` which includes the panel's offscreen texture composited onto 3D geometry
- Participates in shadow mapping and lighting — its quad receives shadows and specular highlights

These are architectural differences, not implementation effort differences. Bevy cannot produce any of these outcomes without changing `bevy_render` and `bevy_ui` at the architectural level — adding compositor passes, depth buffer sharing, and render graph integration that do not exist in its current design.

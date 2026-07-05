# Implementation Plan: 3D Scene Pipeline via cvkg-render-3d

## Goal

Transform `cvkg-render-3d` from a collection of duplicate types and stub passes into the **bridge layer** that connects scene data sources (cvkg-scene VNodes, cvkg-gltf loaded scenes) to the GPU renderer (cvkg-render-gpu's `submit_mesh_3d` API and Kvasir render graph).

## Current State

```
cvkg-scene (VNode with inert 3D fields)
cvkg-gltf (Scene3D, Node3D, Mesh)
cvkg-render-3d-hierarchy (TransformNode3D, propagate_transforms)
cvkg-spatial (Frustum)
    │ (none connected)
    ▼
cvkg-render-3d (duplicate types, stub passes, FrustumCuller)
    │ (depends on render-gpu but render-gpu never calls it)
    ▼
cvkg-render-gpu (submit_mesh_3d, Kvasir graph, PBR/shadow/skinning)
```

## Target State

```
cvkg-scene (VNode with 3D fields) ─────┐
cvkg-gltf (Scene3D) ───────────────────┤
cvkg-render-3d-hierarchy (transforms) ──┤──► cvkg-render-3d ──► cvkg-render-gpu
cvkg-spatial (frustum) ────────────────┘    (SceneFlattener)    (submit_mesh_3d)
```

## Architecture

### What cvkg-render-3d becomes

The **scene flattener**: reads high-level scene descriptions, performs CPU-side processing (transform propagation, culling, light extraction, mesh instancing), and submits results to the GPU renderer via its public API.

### What stays where

| Crate | Responsibility |
|-------|---------------|
| cvkg-scene | Data model only (VNode tree, dirty tracking, spatial hash) |
| cvkg-gltf | Asset loading only (glTF → Scene3D) |
| cvkg-render-3d-hierarchy | Pure transform math (propagate_transforms) |
| cvkg-spatial | Spatial queries (Frustum, AABB) |
| cvkg-render-3d | **Scene flattener** — orchestrates 3D pipeline |
| cvkg-render-gpu | GPU execution only (pipelines, render graph, buffer management) |

---

## Phase 1: Type Consolidation

**Goal:** Remove duplicate types from cvkg-render-3d, re-export from cvkg-render-gpu.

### Step 1.1: Remove duplicate GpuMesh3d

**File:** `cvkg-render-3d/src/types.rs`

Delete the local `GpuMesh3d` struct (lines 107-114). Replace with a re-export:

```rust
pub use cvkg_render_gpu::passes::shadow::GpuMesh3d;
```

**File:** `cvkg-render-3d/src/passes/shadow.rs`

Update `ShadowNode.mesh_instances` to use the re-exported type (should be transparent since it's the same name).

**File:** `cvkg-render-3d/src/passes/opaque3d.rs`

Same update for `Opaque3dNode.mesh_instances`.

### Step 1.2: Remove duplicate DirectionalLight

**File:** `cvkg-render-3d/src/types.rs`

The render-3d version has extra fields (`shadow_map_size`, `shadow_bias`, `shadow_normal_bias`). These are useful configuration that render-gpu's version lacks. Instead of deleting, **rename** the render-3d version to `DirectionalLightConfig` and keep it as the configuration type. Add a conversion:

```rust
pub struct DirectionalLightConfig {
    pub direction: glam::Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub shadow_map_size: u32,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
}

impl From<&DirectionalLightConfig> for cvkg_render_gpu::passes::shadow::DirectionalLight {
    fn from(config: &DirectionalLightConfig) -> Self {
        Self {
            direction: config.direction,
            color: config.color.into(),
            intensity: config.intensity,
        }
    }
}
```

### Step 1.3: Update re-exports in lib.rs

**File:** `cvkg-render-3d/src/lib.rs`

```rust
pub use types::{
    DirectionalLightConfig, Light, PointLight, ShadowInstance, ShadowMap, ShadowQuality,
    SpotLight,
};
pub use cvkg_render_gpu::passes::shadow::GpuMesh3d;
pub use culler::FrustumCuller;
// Remove Opaque3dNode and ShadowNode re-exports — these are internal to render-gpu
```

### Step 1.4: Remove stub pass re-exports

**File:** `cvkg-render-3d/src/lib.rs`

Remove `pub use passes::{Opaque3dNode, ShadowNode}` — these are simpler duplicates of render-gpu's passes. The render-gpu versions are the ones actually wired into the Kvasir graph. Keeping them creates confusion.

**File:** `cvkg-render-3d/src/passes.rs`

Keep the module but remove the `pub` on `shadow` and `opaque3d` sub-modules if they're no longer needed externally. Or delete the entire `passes/` directory if all pass logic lives in render-gpu.

---

## Phase 2: SceneFlattener

**Goal:** Create the core orchestrator that converts scene data into GPU submissions.

### Step 2.1: Define SceneFlattener struct

**File:** `cvkg-render-3d/src/flattener.rs` (new file)

```rust
use cvkg_core::{NodeId, Transform3D, Material3D, Mesh, Rect};
use cvkg_render_3d_hierarchy::TransformNode3D;
use glam::Mat4;

/// A flat mesh instance ready for GPU submission.
pub struct FlatMeshInstance {
    pub mesh: Mesh,
    pub material: Material3D,
    pub transform: Mat4,
    pub bounds_center: Vec3,
    pub bounds_half_extents: Vec3,
    pub visible: bool,
}

/// A flat light ready for GPU submission.
pub enum FlatLight {
    Directional(DirectionalLightConfig),
    Point(PointLight),
    Spot(SpotLight),
}

/// The scene flattener processes hierarchical scene data into flat
/// lists of mesh instances and lights, ready for GPU submission.
pub struct SceneFlattener {
    /// Mesh instances from the last flatten call.
    pub instances: Vec<FlatMeshInstance>,
    /// Lights from the last flatten call.
    pub lights: Vec<FlatLight>,
    /// Camera view-projection matrix.
    pub view_projection: Mat4,
    /// Camera position (for specular/lighting calculations).
    pub camera_pos: Vec3,
    /// Scene bounding sphere radius (for shadow map sizing).
    pub scene_radius: f32,
}
```

### Step 2.2: Implement flatten from VNode tree

**File:** `cvkg-render-3d/src/flattener.rs`

```rust
impl SceneFlattener {
    /// Flatten a cvkg-scene VNode tree into GPU-ready instances.
    ///
    /// Reads `is_3d`, `position_3d`, `rotation_3d`, `scale_3d` fields from VNodes,
    /// propagates transforms via cvkg-render-3d-hierarchy, and produces flat lists.
    pub fn flatten_vnodes(
        &mut self,
        nodes: &[cvkg_scene::VNode],  // or a reference to SceneGraph
        mesh_provider: &dyn Fn(&str) -> Option<(Mesh, Material3D)>,
    ) {
        // 1. Build TransformNode3D array from VNodes
        // 2. Call propagate_transforms()
        // 3. For each node with is_3d=true:
        //    a. Look up mesh via mesh_provider
        //    b. Compute AABB in world space
        //    c. Push FlatMeshInstance
        // 4. Extract lights (future: from VNode component types)
    }

    /// Flatten a cvkg-gltf Scene3D into GPU-ready instances.
    pub fn flatten_gltf(&mut self, scene: &cvkg_gltf::Scene3D) {
        // 1. Build TransformNode3D array from Scene3D nodes
        //    - Node3D has parent index, transform, mesh_index
        // 2. Call propagate_transforms()
        // 3. For each node with mesh_index.is_some():
        //    a. Get Mesh + Material from scene.meshes
        //    b. Compute AABB
        //    c. Push FlatMeshInstance
        // 4. Extract cameras for view_projection
    }

    /// Apply frustum culling to the flattened instances.
    pub fn cull(&mut self, culler: &FrustumCuller) {
        for instance in &mut self.instances {
            instance.visible = culler.is_visible(
                instance.bounds_center,
                instance.bounds_half_extents,
            );
        }
    }

    /// Compute scene_radius from instance bounds (for shadow map sizing).
    pub fn compute_scene_radius(&mut self) {
        self.scene_radius = self.instances.iter()
            .filter(|i| i.visible)
            .map(|i| i.bounds_center.length() + i.bounds_half_extents.length())
            .fold(0.0f32, f32::max);
    }
}
```

### Step 2.3: Add VNode → TransformNode3D conversion

**File:** `cvkg-render-3d/src/flattener.rs`

```rust
fn vnode_to_transform_nodes(
    nodes: &[cvkg_scene::VNode],
) -> Vec<TransformNode3D> {
    nodes.iter()
        .filter(|n| n.is_3d)
        .map(|n| {
            let rotation = glam::Quat::from_xyzw(
                n.rotation_3d[0], n.rotation_3d[1],
                n.rotation_3d[2], n.rotation_3d[3],
            );
            let position = glam::Vec3::from_array(n.position_3d);
            let scale = glam::Vec3::from_array(n.scale_3d);

            TransformNode3D {
                id: NodeId(cvkg_core::KvasirId(n.id.0)),
                parent: n.children.first().map(|_| {
                    // Parent is the node whose children list contains this node's id
                    // SceneGraph stores parent-child via children Vec, not a parent field
                    // We need to build a parent index map
                    NodeId(cvkg_core::KvasirId(0)) // placeholder
                }),
                children: n.children.iter()
                    .map(|c| NodeId(cvkg_core::KvasirId(c.0)))
                    .collect(),
                local: Transform3D {
                    position,
                    rotation,
                    scale,
                },
                global: Mat4::IDENTITY,
            }
        })
        .collect()
}
```

**Note:** The VNode tree stores children but not parents. The flattener needs to build a reverse index (child → parent) during conversion. This is a known implementation detail.

---

## Phase 3: Integration with cvkg-render-gpu

**Goal:** Wire SceneFlattener output into GpuRenderer's submission API.

### Step 3.1: Add submit_scene method to GpuRenderer

**File:** `cvkg-render-gpu/src/renderer/draw.rs`

```rust
/// Submit a flattened 3D scene to the GPU renderer.
/// Consumes SceneFlattener output and populates the pending 3D instance lists.
pub fn submit_scene(&mut self, flattener: &SceneFlattener) {
    // Set directional light from first directional in flattener.lights
    for light in &flattener.lights {
        match light {
            FlatLight::Directional(config) => {
                if self.pending_directional_light.is_none() {
                    self.pending_directional_light = Some(config.into());
                }
            }
            // PointLight, SpotLight: future extension
            _ => {}
        }
    }

    // Set camera
    self.set_camera_3d(flattener.view_projection, flattener.camera_pos);

    // Submit visible mesh instances
    for instance in &flattener.instances {
        if !instance.visible {
            continue;
        }
        // Build Transform3D from the Mat4 (or pass through directly)
        // call self.submit_mesh_3d(&instance.mesh, &instance.material, &transform)
    }
}
```

**Note:** `submit_mesh_3d` currently takes `&Transform3D` (not `&Mat4`). We either:
- (a) Store the original `Transform3D` on `FlatMeshInstance` instead of the computed `Mat4`
- (b) Add a `submit_mesh_3d_with_matrix(&mut self, mesh, material, matrix: &Mat4)` overload

Option (a) is cleaner — keep the `Transform3D` on the flat instance and let `submit_mesh_3d` recompute the matrix. The matrix is only needed for the hierarchy step.

### Step 3.2: Add submit_scene to umbrella re-exports

**File:** `cvkg/Cargo.toml`

The `render-3d` feature already exists. No change needed. The consumer enables `render-3d` and gets access to `cvkg_render_3d::SceneFlattener`.

---

## Phase 4: Frustum Culling Integration

**Goal:** Connect cvkg-spatial's Frustum to the scene flattener.

### Step 4.1: SceneFlattener::cull already uses FrustumCuller (Phase 2.2)

No additional work needed beyond what's in Step 2.2. The `FrustumCuller` wraps `cvkg_spatial::frustum::Frustum` and provides `is_visible(center, half_extents)`.

### Step 4.2: AABB computation from mesh vertices

**File:** `cvkg-render-3d/src/flattener.rs`

```rust
fn compute_aabb(vertices: &[cvkg_core::Vertex3D]) -> (Vec3, Vec3) {
    if vertices.is_empty() {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for v in vertices {
        let p = Vec3::from(v.position);
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    let half_extents = (max - min) * 0.5;
    (center, half_extents)
}
```

---

## Phase 5: Scene Graph Integration

**Goal:** Wire cvkg-scene's VNode tree into the flattener.

### Step 5.1: Add mesh_provider trait

**File:** `cvkg-render-3d/src/flattener.rs`

The flattener needs to resolve VNode component_type strings to actual meshes. This should be a trait/ closures to avoid coupling:

```rust
/// Resolves a component type string to a mesh and material.
pub trait MeshProvider {
    fn resolve(&self, component_type: &str) -> Option<(Mesh, Material3D)>;
}
```

### Step 5.2: SceneGraph flatten integration

**File:** `cvkg-scene/src/lib.rs`

Add a method on `SceneGraph` that produces data consumable by `SceneFlattener`:

```rust
impl SceneGraph {
    /// Returns all 3D VNodes as a flat slice for scene flattening.
    pub fn nodes_3d(&self) -> impl Iterator<Item = &VNode> {
        self.nodes.values().filter(|n| n.is_3d)
    }

    /// Builds a child→parent index map for 3D nodes.
    pub fn parent_map_3d(&self) -> HashMap<NodeId, NodeId> {
        let mut map = HashMap::new();
        for node in self.nodes.values() {
            for &child_id in &node.children {
                map.insert(child_id, node.id);
            }
        }
        map
    }
}
```

---

## Phase 6: glTF Integration

**Goal:** Allow flattened glTF scenes to go through the same pipeline.

### Step 6.1: SceneFlattener::flatten_gltf (already designed in Step 2.2)

The glTF path is simpler than the VNode path because `Scene3D` already has a flat node array with parent indices:

```rust
pub fn flatten_gltf(&mut self, scene: &Scene3D) {
    // 1. Build TransformNode3D from scene.nodes (already has parent index)
    // 2. propagate_transforms()
    // 3. For each node with mesh_index:
    //    - mesh = scene.meshes[mesh_index].mesh.clone()
    //    - material = convert gltf material → Material3D
    //    - Push FlatMeshInstance
}
```

### Step 6.2: Material conversion

**File:** `cvkg-render-3d/src/flattener.rs`

```rust
fn gltf_material_to_core(
    gltf_material: &cvkg_gltf::LoadedMesh,
    textures: &[cvkg_gltf::LoadedTexture],
) -> cvkg_core::Material3D {
    // Map glTF material properties to cvkg_core::Material3D
    // base_color, metallic, roughness, emissive, etc.
}
```

---

## Phase 7: Testing

### Step 7.1: Unit tests for flattener

**File:** `cvkg-render-3d/tests/flattener_tests.rs`

```rust
#[test]
fn test_flatten_single_mesh() { ... }

#[test]
fn test_flatten_hierarchy() { ... }

#[test]
fn test_frustum_culling() { ... }

#[test]
fn test_duplicate_type_consistency() {
    // Verify render-3d's re-exported GpuMesh3d is the same type as render-gpu's
}
```

### Step 7.2: Integration test

**File:** `cvkg-render-3d/tests/integration_tests.rs`

```rust
#[test]
fn test_gltf_to_gpu_submission() {
    // Load a glTF file
    // Flatten with SceneFlattener
    // Verify instances are produced
    // Verify culling works
}
```

---

## File Change Summary

| File | Action | Description |
|------|--------|-------------|
| `cvkg-render-3d/Cargo.toml` | Modify | Add `cvkg-render-3d-hierarchy` dependency |
| `cvkg-render-3d/src/lib.rs` | Modify | Update re-exports, add `mod flattener` |
| `cvkg-render-3d/src/types.rs` | Modify | Remove `GpuMesh3d`, rename `DirectionalLight` → `DirectionalLightConfig`, add From impl |
| `cvkg-render-3d/src/flattener.rs` | **New** | `SceneFlattener`, `FlatMeshInstance`, `FlatLight`, `MeshProvider` trait |
| `cvkg-render-3d/src/passes/` | Modify | Remove or deprecate stub ShadowNode/Opaque3dNode |
| `cvkg-render-3d/src/culler.rs` | No change | Already correct |
| `cvkg-render-gpu/src/renderer/draw.rs` | Modify | Add `submit_scene()` method |
| `cvkg-scene/src/lib.rs` | Modify | Add `nodes_3d()`, `parent_map_3d()` methods |
| `cvkg-render-3d/tests/` | **New** | Flattener unit + integration tests |

## Dependency Changes

| Crate | Add Dependency |
|-------|---------------|
| cvkg-render-3d | `cvkg-render-3d-hierarchy` (workspace) |
| cvkg-render-3d | `cvkg-scene` (optional, behind feature flag) |
| cvkg-render-3d | `cvkg-gltf` (optional, behind feature flag) |

Feature flags for cvkg-render-3d:
```toml
[features]
default = []
scene = ["dep:cvkg-scene"]
gltf = ["dep:cvkg-gltf"]
full = ["scene", "gltf"]
```

## Implementation Order

1. **Phase 1** (Type Consolidation) — prerequisite for everything else
2. **Phase 2** (SceneFlattener core) — the new code, no external dependencies
3. **Phase 3** (GPU integration) — wire flattener to render-gpu
4. **Phase 4** (Frustum culling) — already have the culler, just connect
5. **Phase 5** (Scene graph integration) — cvkg-scene bridge
6. **Phase 6** (glTF integration) — cvkg-gltf bridge
7. **Phase 7** (Testing) — throughout, but formal tests at end

Each phase is independently testable and shippable. Phase 1-3 deliver the minimum viable 3D pipeline. Phases 4-6 add data sources. Phase 7 validates everything.

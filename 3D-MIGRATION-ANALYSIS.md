# CVKG 3D Migration Analysis

**Assessment:** What's required to evolve from 2D-only to 2.5D and full 3D rendering

---

## Current State Assessment

The project is **deeply 2D-oriented** at every layer:

| Layer | Current | 3D Needs |
|-------|---------|----------|
| **Math** | `glam::Vec2`, `glam::Mat3` | `glam::Vec3`, `glam::Mat4` throughout |
| **Physics** | 2D GJK/EPA, 2D spatial hash, 2D constraints | 3D GJK/EPA, BVH broadphase, 3D constraints |
| **Renderer** | 2D vertex format (pos.xy, 2D UV), 2D ortho projection | 3D vertex format (pos.xyz, 3D UV), perspective projection |
| **Shaders** | 2D SDF shapes, 2D transforms, fullscreen blit | 3D SDF/mesh shaders, MVP matrices, depth buffer |
| **Compositor** | 2D layers with z-index | True 3D layers with depth sorting |
| **Scene Graph** | 2D transforms (x, y, rotation) | 3D transforms (x, y, z, quaternion) |

### What Already Has 3D Primitives

Surprisingly, the foundation is partially there:

1. **`cvkg-core`** already uses `glam::Mat4` for SceneUniforms (view + proj matrices, line 3146)
2. **`cvkg-core`** has `draw_mesh(&mut self, mesh: &Mesh, color: [f32; 4], transform: Mat4)` in the Renderer trait (line 2775)
3. **`cvkg-render-gpu`** has a `draw_mesh()` implementation (line 4304) that transforms vertices through a Mat4
4. **Vertex struct** already has `position: [f32; 3]` (line 270) — the z component exists but is underutilized
5. **Physics** has `glam::Vec3` usage in skeletal animation
6. **Animation** has `Vec4` for color/keyframe interpolation

### What's Missing / Blocking

1. **Shaders are 2D-only** — No 3D vertex shaders, no perspective division, no 3D SDF primitives
2. **No 3D camera system** — The projection matrix is orthographic 2D
3. **No 3D mesh pipeline** — No vertex buffers for 3D meshes, no index buffers for 3D
4. **Physics is 2D-only** — GJK/EPA only handles 2D shapes, no 3D BVH
5. **No depth testing in UI pipeline** — The depth buffer exists but is only used for opaque/glass ordering
6. **Compositor is 2D** — No 3D layer sorting, no perspective-correct interpolation

---

## Migration Path: Three Phases

### Phase 1: 2.5D (Parallax / Layered 3D)

**Goal:** Add depth to the existing 2D pipeline without breaking compatibility.

**Changes needed:**

1. **Vertex format upgrade** (low risk)
   - Already has `position: [f32; 3]` — start using z for depth
   - Add `glam::Mat4` transform stack (currently only 2D Mat3)

2. **Projection matrix** (low risk)
   - Add a `Camera` struct with perspective projection
   - Keep orthographic as default, allow perspective override
   - Change `SceneUniforms.proj` from ortho to perspective when 3D camera active

3. **Compositor 2.5D** (medium risk)
   - Add z-position to layers
   - Sort layers by depth for correct over/under
   - Add parallax scrolling support

4. **Simple 3D transforms** (medium risk)
   - Add `Renderer::push_3d_transform(translation: Vec3, rotation: Quat, scale: Vec3)`
   - Add `Renderer::pop_3d_transform()`

**Estimated effort:** 2-3 weeks, ~500 lines changed

---

### Phase 2: Full 3D Rendering

**Goal:** Complete 3D rendering pipeline with perspective, depth testing, 3D meshes.

**Changes needed:**

1. **Shader rewrite** (high risk)
   - 3D vertex shader: `vec4(position, 1.0) * model * view * proj`
   - 3D fragment shader: proper depth output, 3D lighting
   - Keep 2D shaders as fallback via separate pipeline

2. **Vertex buffer format** (medium risk)
   - Add normals: `[f32; 3]`
   - Add tangents for normal mapping: `[f32; 4]`
   - Add 3D UVs: already supported

3. **3D mesh pipeline** (high risk)
   - Vertex/index buffer management for 3D meshes
   - Material system (diffuse, specular, normal maps)
   - 3D model loading (glTF/GLB)

4. **Depth buffer** (already exists)
   - Already have Depth32Float in the pipeline
   - Just need to enable it for 3D passes

5. **Camera system** (medium risk)
   - `Camera3D` struct: position, target, up, fov, near/far
   - View matrix computation
   - Projection matrix computation
   - Orbit/flythrough controllers

**Estimated effort:** 6-8 weeks, ~3000 lines changed

---

### Phase 3: 3D Physics

**Goal:** 3D rigid body physics with 3D collision detection.

**Changes needed:**

1. **Math upgrade** (medium risk)
   - `glam::Vec3` for positions, velocities, forces
   - `glam::Quat` for rotations (replaces `f32` angle)
   - 3D inertia tensor (3x3 matrix, replaces scalar)

2. **3D GJK/EPA** (high risk)
   - GJK already has the right algorithm — just needs 3D simplex (tetrahedron vs triangle)
   - EPA needs 3D (polyhedron vs polygon)
   - Support mapping: `support(dir: Vec3) -> Vec3`

3. **3D Broadphase** (medium risk)
   - Replace spatial hash with BVH (Bounding Volume Hierarchy)
   - Or use sweep-and-prune for axis-aligned cases

4. **3D Constraints** (medium risk)
   - Ball-and-socket joint (3D pin)
   - Hinge joint (3D, with axis)
   - Slider, spring, etc.

5. **Collision shapes** (medium risk)
   - Sphere, Box, Capsule (3D versions)
   - Convex hull (3D vertices)
   - Heightmap / triangle mesh (static)

**Estimated effort:** 8-10 weeks, ~5000 lines changed

---

## Critical Path Items

These are the must-do-everything-else-depends-on changes:

1. **Camera system** → Without this, nothing 3D can be displayed correctly
2. **3D vertex shader** → Without this, no 3D geometry renders
3. **Mat4 transform stack** → Without this, no 3D positioning
4. **Depth testing** → Without this, 3D objects overlap incorrectly
5. **3D GJK** → Without this, no 3D collision detection

---

## Compatibility Strategy

To avoid breaking the existing 2D pipeline:

```rust
// Renderer trait — add 3D methods with default no-op impls
pub trait Renderer {
    // Existing 2D methods (unchanged)
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    
    // New 3D methods (default no-op for 2D renderers)
    fn draw_mesh_3d(&mut self, mesh: &Mesh3D, material: &Material, transform: &Transform3D) {}
    fn set_camera_3d(&mut self, camera: &Camera3D) {}
    fn push_transform_3d(&mut self, transform: &Transform3D) {}
    fn pop_transform_3d(&mut self) {}
}
```

The 2D renderer (SurtrRenderer) would implement the 3D methods by:
- Ignoring z coordinate (flattening to 2D)
- Using orthographic projection
- Skipping depth testing

---

## Risk Assessment

| Phase | Risk | Blocker? | Rollback |
|-------|------|----------|----------|
| 1 (2.5D) | Low | No | Easy — just don't use z |
| 2 (3D Render) | Medium | Yes — shader changes are invasive | Moderate — need 2D fallback |
| 3 (3D Physics) | Medium | No — separate crate | Easy — physics is isolated |

---

## Recommendation

Start with **Phase 1** — 2.5D layered rendering. It:
- Adds visual depth without breaking 2D
- Tests the Mat4/transform infrastructure
- Enables parallax backgrounds (immediate visual win)
- Low risk, high impact

Then skip to **Phase 2** for the 3D rendering pipeline. Physics (Phase 3) can be parallelized since it's in a separate crate.

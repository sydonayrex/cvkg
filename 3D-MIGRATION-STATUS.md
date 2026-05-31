# CVKG 3D Migration — Implementation Status

## Completed

### Phase 1: 2.5D Core Types ✅
- `Transform3D` (position: Vec3, rotation: Quat, scale: Vec3) — `cvkg-core`
- `Camera3D` (position, target, up, fov, near/far, perspective/orthographic) — `cvkg-core`
- `Material3D` (base_color, metallic, roughness, emissive, opacity) — `cvkg-core`
- 5 new Renderer trait methods: `draw_mesh_3d`, `set_camera_3d`, `push_transform_3d`, `pop_transform_3d`
- Full implementations in `cvkg-render-gpu` SurtrRenderer
- Stub implementations in `cvkg-render-web`

### Phase 2: 3D Shaders ✅
- Mode 13 (3D Surface) vertex shader: MVP transformation (`proj * view * position`)
- Mode 13 fragment shader: PBR lighting (Lambert diffuse, Blinn-Phong specular, Schlick Fresnel)
- Material params passed via vertex slice field (metallic, roughness, opacity)
- Depth fog for visual depth perception

### Phase 3: 3D Physics Data Structures ✅
- `RigidBody` extended with `is_3d` flag and 3D fields:
  - `position_3d: Vec3`, `velocity_3d: Vec3`, `force_3d: Vec3`
  - `rotation: Quat`, `angular_velocity_3d: Vec3`, `torque_3d: Vec3`
  - `inv_inertia_3d: Vec3`
- 3D shapes: `Sphere`, `Box3D`, `Capsule3D` with `support()` method
- `SpatialHash3D` with configurable `cell_size`
- 3D broadphase AABB queries

## Remaining Work

### Critical: 3D Narrowphase (GJK/EPA)
**File:** `cvkg-physics/src/narrowphase.rs`

The 2D GJK/EPA needs 3D equivalents:

1. **3D GJK** — Changes from 2D:
   - Simplex: `[Vec2; 3]` (triangle) → `[Vec3; 4]` (tetrahedron)
   - `process_simplex`: 2D line/triangle cases → 3D line/triangle/tetrahedron cases
   - Support function: uses `Shape::support(Vec3)` (already implemented for 3D shapes)
   - Minkowski support: same formula, but with Vec3

2. **3D EPA** — Changes from 2D:
   - Initial simplex: triangle (3 points) → tetrahedron (4 points)
   - Edge expansion: polygon edges → polyhedron faces
   - Closest face tracking: edge list → face list with normals

3. **Contact manifold**: `Contact` struct needs `normal: Vec3`, `point: Vec3`

**Effort estimate:** ~500 lines of carefully tested geometry code.

### Critical: 3D Integration
**File:** `cvkg-physics/src/integration.rs`

Add `semi_implicit_euler_3d()` that:
- Uses `Vec3` for position, velocity, force
- Uses `Quat` for rotation (angular velocity integration via quaternion derivative)
- Uses `Vec3` for angular velocity, torque
- Uses `inv_inertia_3d` for angular acceleration

**Effort estimate:** ~100 lines.

### Important: 3D Constraints
**Files:** `cvkg-physics/src/constraint.rs`, `cvkg-physics/src/solver.rs`

Add 3D constraint kinds:
- `BallSocket3D { anchor: Vec3 }` — point-to-point constraint in 3D
- `Hinge3D { axis: Vec3, ... }` — rotation around an axis in 3D
- `Fixed3D` — locks all 6 DOF between two bodies

Update solver to handle 3D constraints (same Gauss-Seidel approach but with 3D Jacobians).

**Effort estimate:** ~300 lines.

### Important: World Step Integration
**File:** `cvkg-physics/src/world.rs`

The `step_substep()` method needs to branch on `body.is_3d`:
- 2D bodies: existing pipeline
- 3D bodies: new 3D integration → 3D broadphase → 3D narrowphase → 3D solver

**Effort estimate:** ~150 lines modifying existing code.

### Nice-to-Have: 3D Scene Bridge
**File:** `cvkg-physics/src/scene_bridge.rs`

Update `sync_to_scene()` to handle 3D transforms (Quat rotation → scene graph).

**Effort estimate:** ~50 lines.

## Architecture Decision

The 2D and 3D physics pipelines should **coexist** via the `is_3d` flag, not be separate crates. This:
- Avoids code duplication for shared logic (solver, constraint framework)
- Allows gradual migration of components from 2D to 3D
- Keeps compile times manageable
- All existing 2D tests continue to pass unchanged

## Testing Strategy

For each 3D component:
1. Unit test with known geometries (e.g., two overlapping spheres → contact normal should be along center line)
2. Fuzz test: random 3D shapes → GJK should not panic, EPA should produce valid contacts
3. Integration test: 3D falling body with 3D collision against static 3D floor

## Priority Order

1. **3D Narrowphase** (GJK/EPA) — Everything depends on this
2. **3D Integration** — Needed for bodies to move
3. **World Step** — Wires it all together
4. **3D Constraints** — Needed for joints/contacts
5. **Scene Bridge** — Needed for visual feedback

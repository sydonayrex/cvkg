# How to Use 3D Materials

## Goal

Render 3D meshes with physically-based rendering (PBR) materials using the CVKG GPU pipeline. This covers configuring materials, lighting, and integrating 3D geometry.

## Prerequisites

- `cvkg-render-gpu` in your project dependencies
- Basic understanding of 3D transforms (position, rotation, scale)
- UV-mapped meshes (or generated primitives)

## Overview

CVKG uses a two-path approach for 3D rendering:

| Path | Entry Point | Use Case |
|------|-------------|----------|
| VDOM API | `render_scene_node_3d()` | Declarative 3D nodes in UI hierarchy |
| Raw API | `submit_mesh_3d()` | Direct mesh submission with full control |

## Material3D Struct

The core material configuration for 3D rendering:

```rust
use cvkg_core::Material3D;

let material = Material3D {
    base_color: [1.0, 0.8, 0.6, 1.0],      // RGBA base color (default: white)
    metallic: 0.0,                            // 0.0 = dielectric, 1.0 = metal
    roughness: 0.5,                           // 0.0 = smooth, 1.0 = rough
    opacity: 1.0,                             // Alpha (default: opaque)
    base_color_texture: Some(texture_id),       // Optional albedo texture
    normal_map_texture: Some(texture_id),       // Optional normal map
    metallic_roughness_texture: Some(texture_id), // Optional ORM texture
    emissive: [0.0, 0.0, 0.0],                // Self-illumination
    uv_scale: [1.0, 1.0],                     // Tiling factor
    uv_offset: [0.0, 0.0],                    // UV offset
};
```

### PBR Parameters

| Parameter | Range | Effect |
|-----------|-------|--------|
| `metallic` | 0.0 - 1.0 | Controls metalness. 0 = dielectric (plastic), 1 = metal |
| `roughness` | 0.0 - 1.0 | Controls microsurface roughness. 0 = mirror smooth, 1 = matte |
| `opacity` | 0.0 - 1.0 | Alpha blending. < 1.0 enables transparent pass |

## VDOM API: render_scene_node_3d

For declarative 3D rendering within the VDOM hierarchy:

```rust
use cvkg::prelude::*;
use cvkg_core::{Camera3D, Renderer, Transform3D};

// In your View::render() implementation:
r.set_camera_3d(&Camera3D {
    fov: 45.0,
    aspect: rect.width() / rect.height(),
    near: 0.1,
    far: 1000.0,
    position: [0.0, 50.0, 100.0],
    target: [0.0, 0.0, 0.0],
    up: [0.0, 1.0, 0.0],
});

// Render a 3D node with default cube geometry
r.render_scene_node_3d(
    position: [0.0, 0.0, 0.0],
    rotation: [0.0, 0.0, 0.0, 1.0],  // Quaternion (x, y, z, w)
    scale: [50.0, 50.0, 50.0],       // Cube size
    color: [0.8, 0.2, 0.2, 1.0],     // Base color
    meshes: &[],                      // Empty = generate unit cube
);

// Or provide custom mesh geometry:
let mesh = Mesh {
    vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], /* ... */],
    normals: vec![[0.0, 0.0, 1.0], /* ... */],
    indices: vec![0, 1, 2, /* ... */],
    tex_coords: vec![[0.0, 0.0], /* ... */],
    tangents: Vec::new(), // Will be computed if empty
};
r.render_scene_node_3d(
    position: [0.0, 0.0, 0.0],
    rotation: [0.0, 0.0, 0.0, 1.0],
    scale: [1.0, 1.0, 1.0],
    color: [1.0, 1.0, 1.0, 1.0],
    meshes: &[mesh],
);
```

## Raw API: submit_mesh_3d

For direct mesh submission with instanced rendering:

```rust
use cvkg_core::{Mesh, Material3D, Transform3D};
use cvkg_render_gpu::GpuRenderer;

// Create your mesh and material
let mesh = /* ... */;
let material = Material3D {
    base_color: [0.9, 0.9, 1.0, 1.0],
    metallic: 0.1,
    roughness: 0.7,
    ..Default::default()
};

// Submit for rendering
gpu_renderer.submit_mesh_3d(&mesh, &material, &Transform3D {
    position: glam::Vec3::new(0.0, 0.0, 0.0),
    rotation: glam::Quat::IDENTITY,
    scale: glam::Vec3::new(1.0, 1.0, 1.0),
});
```

This creates dedicated per-mesh vertex buffers and uses instanced rendering with the PBR pipeline.

## Lighting

### Directional Light

The 3D pipeline includes cascaded shadow mapping (CSM) with a single directional light:

```rust
// Set up scene lighting
use cvkg_core::SceneUniforms;

let mut scene = SceneUniforms::default();
scene.light_direction = [0.5, 0.8, 0.6];  // Normalized direction
scene.light_color = [1.0, 0.95, 0.9];      // Warm white
scene.ambient_color = [0.1, 0.1, 0.12, 1.0]; // Ambient illumination
scene.shadow_map_size = 2048.0;             // Shadow map resolution
scene.shadow_bias = 0.005;                  // Shadow acne prevention
```

The directional light is automatically configured when meshes are submitted. Override by setting `Renderer::set_scene_uniforms` before rendering.

### Image-Based Lighting (IBL)

The PBR pipeline samples from a pre-filtered environment map for realistic reflections:

```rust
// IBL is configured in the renderer initialization
// The blur pass provides a pre-filtered cubemap for specular reflections
// Set in RendererConfig:
let config = RendererConfig {
    ibl_enabled: true,
    ..Default::default()
};
```

## Vertex Format: Vertex3D

3D meshes use a dedicated vertex format that separates position data from material properties:

```rust
// WGSL vertex attributes (locations)
struct Vertex3D {
    position: vec3<f32>,    // location 0
    normal: vec3<f32>,      // location 1
    uv: vec2<f32>,          // location 2
    color: vec4<f32>,       // location 3
    tangent: vec4<f32>,     // location 9 (xyz = direction, w = handedness)
}

// Instance attributes (per-mesh)
struct InstanceData3D {
    model_row0: vec4<f32>,  // location 16-18 (model matrix)
    model_row1: vec4<f32>,
    model_row2: vec4<f32>,
    material_overrides: vec4<f32>,  // location 19 (metallic, roughness, _, opacity)
    uv_scale: vec2<f32>,   // location 20
    uv_offset: vec2<f32>,  // location 21
}
```

## Shader Pipeline

The 3D rendering uses specialized WGSL shaders:

| Shader | Purpose | Entry Point |
|--------|---------|-------------|
| `material_pbr.wgsl` | PBR shading with shadows | `vs_main_3d`, `fs_main` |
| `material_shadow.wgsl` | Depth-only shadow pass | `vs_shadow` |
| `common.wgsl` | Shared vertex input/output | `VertexInput3D` |

### PBR BRDF Implementation

The fragment shader computes the Cook-Torrance BRDF:

```wgsl
// GGX Normal Distribution Function
fn ggx_ndf(n_dot_h: f32, roughness: f32) -> f32

// Smith Geometry Function
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32

// Fresnel
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32>

// Full BRDF: (D * G * F) / (4 * n_dot_v * n_dot_l)
```

## Transparency

Transparent 3D materials use the same PBR pipeline with `opacity < 1.0`:

```rust
let transparent_material = Material3D {
    base_color: [0.5, 0.5, 1.0, 0.8],  // Alpha < 1.0
    opacity: 0.8,
    metallic: 0.0,
    roughness: 0.1,
    ..Default::default()
};
// Automatically routed to transparent pass with back-to-front sorting
```

## Example: Complete 3D Scene

```rust
use cvkg::prelude::*;
use cvkg_core::{Camera3D, Mesh, Transform3D, Material3D, Renderer};

struct My3DView {
    rotation: glam::Quat,
}

impl View for My3DView {
    type Body = Self;

    fn body(self) -> Self::Body { self }

    fn render(&self, r: &mut dyn Renderer, rect: Rect) {
        // Configure 3D camera
        r.set_camera_3d(&Camera3D {
            fov: 45.0,
            aspect: rect.width() / rect.height(),
            near: 0.1,
            far: 500.0,
            position: [0.0, 100.0, 150.0],
            target: [0.0, 0.0, 0.0],
            ..Default::default()
        });

        // Render a metallic sphere
        r.render_scene_node_3d(
            position: [0.0, 50.0, 0.0],
            rotation: self.rotation.to_array(),
            scale: [30.0, 30.0, 30.0],
            color: [0.95, 0.95, 1.0, 1.0],  // Slightly blue
            meshes: &[],
        );

        // Render a rough floor
        r.render_scene_node_3d(
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [200.0, 10.0, 200.0],
            color: [0.8, 0.8, 0.8, 1.0],
            meshes: &[],
        );
    }
}
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Mesh appears black | Check `base_color` is not [0,0,0,0]; verify light_direction is set |
| No shadows | Ensure `cvkg-render-gpu` has shadow map textures initialized; check `shadow_bias` |
| Z-fighting with 2D UI | Use `r.draw()` calls before 3D rendering; UI renders on top |
| Wrong lighting | Verify using `VertexOutput.world_pos_3d` not `world_pos` in shaders |

## See Also

- [architecture.md](../../../docs/architecture.md#gpu-renderer-3d-materials) - Architecture details
- `cvkg-render-gpu/src/shaders/material_pbr.wgsl` - Shader source
- `cvkg/examples/physics_3d_demo.rs` - Live example
//! Shadow shader — depth-only rendering for cascaded shadow maps.
//! Renders 3D meshes to a depth texture from the light's perspective.

// Vertex shader for shadow map rendering (3D meshes)
// Uses the same VertexInput3D as material_pbr.wgsl but outputs depth only
@vertex
fn vs_shadow(in: VertexInput3D) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(
        in.model_row0,
        in.model_row1,
        in.model_row2,
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    
    // Use light_vp from SceneUniforms for shadow map projection
    let light_pos = scene.light_vp * model * vec4<f32>(in.position, 1.0);
    return light_pos;
}
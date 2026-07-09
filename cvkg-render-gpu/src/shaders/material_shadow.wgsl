//! Shadow shader — depth-only rendering for cascaded shadow maps.
//! Renders 3D meshes to a depth texture from the light's perspective.
//!
//! NOTE: Common definitions (SceneUniforms, VertexInput3D, etc.) are
//! prepended via WGSL_COMMON string concatenation in init.rs. Do NOT
//! redefine VertexInput3D here.

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

// =============================================================================
// DUAL PARABOLOID SHADOW MAPPING (for point lights)
// =============================================================================
// INVARIANT: Both hemispheres must cover full sphere, no gaps

struct DualParaboloidOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct DualParaboloidVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(5) model_row0: vec4<f32>,
    @location(6) model_row1: vec4<f32>,
    @location(7) model_row2: vec4<f32>,
};

@vertex
fn vs_paraboloid(in: DualParaboloidVertexInput, @builtin(instance_index) idx: u32) -> DualParaboloidOutput {
    // idx 0 = +Z hemisphere, idx 1 = -Z hemisphere
    // Transform to paraboloid space for point light shadow
    let model = mat4x4<f32>(
        in.model_row0,
        in.model_row1,
        in.model_row2,
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    
    let world_pos = (model * vec4<f32>(in.position, 1.0)).xyz;
    let eye_dir = world_pos - scene.camera_pos;
    
    // Paraboloid projection: map sphere to plane
    // For +Z hemisphere (idx=0): forward projection
    // For -Z hemisphere (idx=1): reverse projection with slice
    var out: DualParaboloidOutput;
    out.uv = in.position.xy; // Simplified - actual needs proper projection
    
    // Use appropriate VP based on hemisphere
    // For point lights, we'd use a custom VP matrix per hemisphere
    // This is a simplified version using scene projection
    out.clip_position = scene.proj * scene.view * vec4<f32>(world_pos, 1.0);
    
    // Set the slice for dual-paraboloid (0 or 1)
    out.clip_position.z = f32(idx) + out.clip_position.z / out.clip_position.w * 0.5;
    out.clip_position.w = out.clip_position.w;
    
    return out;
}
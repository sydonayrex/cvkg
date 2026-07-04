// =============================================================================
// Common 3D types — shared between mesh_vertex.wgsl and mesh_pbr.wgsl
// =============================================================================

// These bindings reference the SceneUniforms from common.wgsl (group 2, binding 1).
// The shadow map texture lives in a separate binding group.

// --- Group 3: Shadow Map ---
@group(3) @binding(0) var t_shadow: texture_depth_2d;
@group(3) @binding(1) var s_shadow: sampler_comparison;

// --- Group 3: Material Textures (optional) ---
// @group(3) @binding(2) var t_albedo: texture_2d<f32>;
// @group(3) @binding(3) var s_material: sampler;

/// PCF 3x3 shadow sampling.
/// Returns a shadow factor in [0.0, 1.0] where 0.0 = fully shadowed, 1.0 = fully lit.
///
/// Parameters:
///   light_vp     - Light's view-projection matrix (from SceneUniforms)
///   world_pos    - World-space position of the fragment
///   shadow_bias  - Depth bias to prevent shadow acne (from SceneUniforms)
///   shadow_size  - Shadow map resolution in pixels (from SceneUniforms)
fn sample_shadow_pcf3(
    light_vp: mat4x4<f32>,
    world_pos: vec3<f32>,
    shadow_bias: f32,
    shadow_size: f32,
) -> f32 {
    // Transform world position to light clip space
    let light_clip = light_vp * vec4<f32>(world_pos, 1.0);
    let light_depth = light_clip.z / light_clip.w;

    // Perspective divide + NDC to UV [0, 1]
    let light_uv = light_clip.xy / light_clip.w * 0.5 + 0.5;

    // PCF 3x3 kernel
    let texel_size = 1.0 / shadow_size;
    var shadow = 0.0;

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            let offset = vec2<f32>(f32(dx), f32(dy)) * texel_size;
            shadow += textureSampleCompare(
                t_shadow,
                s_shadow,
                light_uv + offset,
                light_depth - shadow_bias
            );
        }
    }

    return shadow / 9.0;
}

/// Simple directional light contribution (Lambert diffuse + Blinn-Phong specular).
fn directional_light_diffuse(
    light_dir: vec3<f32>,
    normal: vec3<f32>,
) -> f32 {
    return max(dot(normal, normalize(-light_dir)), 0.0);
}

/// Fresnel Schlick approximation.
fn fresnel_schlick(cos_theta: f32, f0: f32) -> f32 {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// =============================================================================
// PBR Fragment Shader for 3D meshes — Cook-Torrance BRDF with PCF shadows
// =============================================================================
// This shader is compiled separately from common.wgsl. It includes the same
// SceneUniforms definition and shadow map bindings.

// ─── Shared Types (matching common.wgsl) ────────────────────────────────────

struct SceneUniforms {
    view:            mat4x4<f32>,
    proj:            mat4x4<f32>,
    time:            f32,
    delta_time:      f32,
    resolution:      vec2<f32>,
    mouse:           vec2<f32>,
    mouse_velocity:  vec2<f32>,
    shatter_origin:  vec2<f32>,
    shatter_time:    f32,
    shatter_force:   f32,
    berzerker_rage:  f32,
    berzerker_mode:  u32,
    scroll_offset:   f32,
    scale_factor:    f32,
    scene_type:      u32,
    _pad_vec2_align: u32,
    fireball_pos:    vec2<f32>,
    camera_pos:      vec3<f32>,
    _pad2:           f32,
    light_direction: vec3<f32>,
    _pad3:           f32,
    light_color:     vec3<f32>,
    ibl_enabled:     u32,
    // Shadow map parameters
    shadow_map_size: f32,
    shadow_bias:     f32,
    _pad_shadow:     u32,
    _pad_shadow2:    u32,
    light_vp:        mat4x4<f32>,
    ambient_color:   vec4<f32>,
};

// ─── Bindings ───────────────────────────────────────────────────────────────

@group(2) @binding(1) var<uniform> scene: SceneUniforms;

// Material textures (group 3)
@group(3) @binding(0) var t_albedo: texture_2d<f32>;
@group(3) @binding(1) var t_normal: texture_2d<f32>;
@group(3) @binding(2) var t_orm: texture_2d<f32>;
@group(3) @binding(3) var s_material: sampler;

// Shadow map (group 4)
@group(4) @binding(0) var t_shadow: texture_depth_2d;
@group(4) @binding(1) var s_shadow: sampler_comparison;

// ─── Vertex Output (must match mesh_vertex.wgsl) ────────────────────────────

struct VertexOutput3D {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv:              vec2<f32>,
    @location(1) color:           vec4<f32>,
    @location(2) @interpolate(flat) material_id: u32,
    @location(3) world_pos:       vec3<f32>,
    @location(4) world_normal:    vec3<f32>,
    @location(5) material_uv_scale:  vec2<f32>,
    @location(6) material_uv_offset: vec2<f32>,
};

// ─── PCF Shadow Sampling ────────────────────────────────────────────────────

fn sample_shadow_pcf3(
    light_vp: mat4x4<f32>,
    world_pos: vec3<f32>,
    shadow_bias: f32,
    shadow_size: f32,
) -> f32 {
    let light_clip = light_vp * vec4<f32>(world_pos, 1.0);
    let light_depth = light_clip.z / light_clip.w;
    let light_uv = light_clip.xy / light_clip.w * 0.5 + 0.5;

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

// ─── PBR Helpers ────────────────────────────────────────────────────────────

const PI: f32 = 3.14159265359;

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    return geometry_schlick_ggx(n_dot_v, roughness) * geometry_schlick_ggx(n_dot_l, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// ─── Fragment Entry Point ────────────────────────────────────────────────────

@fragment
fn fs_main(in: VertexOutput3D) -> @location(0) vec4<f32> {
    // Transform UVs with material scale and offset
    let uv = in.uv * in.material_uv_scale + in.material_uv_offset;

    // Sample albedo texture and combine with vertex color
    let albedo_tex = textureSample(t_albedo, s_material, uv);
    let base_color = in.color * albedo_tex;

    // Sample ORM texture (r=occlusion, g=roughness, b=metallic)
    let orm = textureSample(t_orm, s_material, uv);
    let ao = orm.r;
    let roughness = orm.g;
    let metallic = orm.b;

    // Normal mapping (sample and perturb normal)
    let normal_tex = textureSample(t_normal, s_material, uv);
    let normal = normalize(in.world_normal + (normal_tex.rgb * 2.0 - 1.0) * 0.0);

    let view_dir = normalize(scene.camera_pos - in.world_pos);
    let light_dir = normalize(-scene.light_direction);

    // Cook-Torrance BRDF
    let half_vec = normalize(view_dir + light_dir);
    let n_dot_h = max(dot(normal, half_vec), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.001);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let h_dot_v = max(dot(half_vec, view_dir), 0.0);

    let ndf = distribution_ggx(n_dot_h, roughness);
    let geo = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f0 = mix(vec3<f32>(0.04), base_color.rgb, metallic);
    let fresnel = fresnel_schlick(h_dot_v, f0);

    // Specular + diffuse
    let numerator = ndf * geo * fresnel;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.001;
    let specular = numerator / denominator;

    let k_d = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic);
    let diffuse = k_d * base_color.rgb / PI;

    // Shadow sampling
    let shadow_factor = sample_shadow_pcf3(
        scene.light_vp,
        in.world_pos,
        scene.shadow_bias,
        scene.shadow_map_size,
    );

    // Combine
    let radiance = scene.light_color * scene.light_direction;
    let lo = (diffuse + specular) * radiance * n_dot_l * shadow_factor;

    // Ambient with occlusion
    let ambient = vec3<f32>(0.03) * base_color.rgb * ao;

    let color = ambient + lo;

    return vec4<f32>(color, base_color.a);
}

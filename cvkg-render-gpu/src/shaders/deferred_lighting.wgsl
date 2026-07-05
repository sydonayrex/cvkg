// Shader: deferred_lighting.wgsl
// Purpose: Deferred PBR lighting resolve pass evaluating lighting equations from G-Buffer targets.

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
    shadow_map_size: f32,
    shadow_bias:     f32,
    _pad_shadow:     u32,
    _pad_shadow2:    u32,
    light_vp:        mat4x4<f32>,
    ambient_color:   vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniforms;

@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var s_normal: sampler;
@group(1) @binding(4) var t_depth: texture_depth_2d;
@group(1) @binding(5) var s_depth: sampler;
@group(1) @binding(6) var t_ssao: texture_2d<f32>;
@group(1) @binding(7) var s_ssao: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_deferred_resolve(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Fetch values from the G-Buffer
    let albedo = textureSample(t_albedo, s_albedo, in.uv);
    let normal_val = textureSample(t_normal, s_normal, in.uv).xyz;
    let normal = normalize(normal_val * 2.0 - 1.0);
    let depth = textureSample(t_depth, s_depth, in.uv);

    // Skip lighting resolve on background sky pixels
    if (depth >= 1.0) {
        return vec4<f32>(0.02, 0.02, 0.05, 1.0);
    }

    // 2. Fetch SSAO occlusion factor
    let ssao = textureSample(t_ssao, s_ssao, in.uv).r;

    // 3. Compute lighting equations (Lambert diffuse + ambient scaled by occlusion)
    let light_dir = normalize(scene.light_direction);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    
    let diffuse = albedo.rgb * n_dot_l * scene.light_color;
    let ambient = albedo.rgb * scene.ambient_color.rgb * scene.ambient_color.w * ssao;

    let final_color = ambient + diffuse;
    return vec4<f32>(final_color, albedo.a);
}

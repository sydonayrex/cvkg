// Shader: ssao.wgsl
// Purpose: Screen Space Ambient Occlusion (SSAO) generation and bilateral blur pass.

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
@group(1) @binding(0) var t_depth: texture_depth_2d;
@group(1) @binding(1) var s_depth: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var s_normal: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Generates a simple pseudo-random direction based on fragment coordinates.
fn random_direction(co: vec2<f32>) -> vec2<f32> {
    let theta = fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453) * 6.28318530718;
    return vec2<f32>(cos(theta), sin(theta));
}

// Fragment shader to compute SSAO occlusion factor.
@fragment
fn fs_ssao(in: VertexOutput) -> @location(0) f32 {
    let depth = textureSample(t_depth, s_depth, in.uv);
    if (depth >= 1.0) {
        return 1.0;
    }

    let normal = normalize(textureSample(t_normal, s_normal, in.uv).xyz * 2.0 - 1.0);
    let sample_radius = 0.05;
    var occlusion = 0.0;
    let sample_count = 16;

    // Evaluate occlusion by sampling a sphere/hemisphere around the fragment
    for (var i = 0; i < sample_count; i = i + 1) {
        let offset_dir = random_direction(in.uv + vec2<f32>(f32(i), 0.0));
        let sample_uv = in.uv + offset_dir * sample_radius;
        let sample_depth = textureSample(t_depth, s_depth, sample_uv);

        if (sample_depth < depth - 0.0001) {
            occlusion += 1.0;
        }
    }

    return 1.0 - (occlusion / f32(sample_count));
}

// Bilateral blur pass to smooth SSAO while preserving depth edges.
@fragment
fn fs_blur(in: VertexOutput) -> @location(0) f32 {
    let texel_size = 1.0 / scene.resolution;
    var result = 0.0;
    var weight_sum = 0.0;
    
    let center_val = textureSample(t_depth, s_depth, in.uv);
    
    for (var x = -2; x <= 2; x = x + 1) {
        for (var y = -2; y <= 2; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_val = textureSample(t_depth, s_depth, in.uv + offset);
            
            // Bilateral weight based on coordinate distance and value difference
            let d_val = abs(sample_val - center_val);
            let weight = exp(-f32(x*x + y*y) / 8.0) * exp(-d_val * d_val / 0.01);
            
            result += sample_val * weight;
            weight_sum += weight;
        }
    }
    
    return result / max(weight_sum, 0.001);
}

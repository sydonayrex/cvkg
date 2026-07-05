// Shader: taa.wgsl
// Purpose: Temporal Anti-Aliasing (TAA) history re-projection and neighborhood clamping.

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

@group(1) @binding(0) var t_current: texture_2d<f32>;
@group(1) @binding(1) var s_current: sampler;
@group(1) @binding(2) var t_history: texture_2d<f32>;
@group(1) @binding(3) var s_history: sampler;
@group(1) @binding(4) var t_motion: texture_2d<f32>;
@group(1) @binding(5) var s_motion: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_taa(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Fetch current frame color
    let current_color = textureSample(t_current, s_current, in.uv);

    // 2. Fetch velocity motion vectors to locate corresponding pixel in the history buffer
    let velocity = textureSample(t_motion, s_motion, in.uv).xy;
    let history_uv = in.uv - velocity;

    // Reject out-of-screen history coords
    if (history_uv.x < 0.0 || history_uv.x > 1.0 || history_uv.y < 0.0 || history_uv.y > 1.0) {
        return current_color;
    }

    let history_color = textureSample(t_history, s_history, history_uv);

    // 3. Compute 3x3 local color bounding box to perform neighborhood clamping/clipping
    let texel_size = 1.0 / scene.resolution;
    var c_min = current_color.rgb;
    var c_max = current_color.rgb;
    
    for (var x = -1; x <= 1; x = x + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            if (x == 0 && y == 0) { continue; }
            let sample_color = textureSample(t_current, s_current, in.uv + vec2<f32>(f32(x), f32(y)) * texel_size).rgb;
            c_min = min(c_min, sample_color);
            c_max = max(c_max, sample_color);
        }
    }

    // Clamp history color to local color bounding box to prevent ghosting artifacts
    let clamped_history = clamp(history_color.rgb, c_min, c_max);

    // 4. Perform dynamic blend resolve
    let blend_weight = 0.1;
    let resolved_color = mix(clamped_history, current_color.rgb, blend_weight);

    return vec4<f32>(resolved_color, current_color.a);
}

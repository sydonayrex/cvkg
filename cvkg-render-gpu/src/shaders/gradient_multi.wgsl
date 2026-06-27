//! Multi-stop gradient shader.
//! Performs per-pixel gradient interpolation using a stop array.
//! Supports both linear and radial modes with configurable quality.

struct GradientUniforms {
    num_stops: u32,
    angle_radians: f32,
    mode: u32, // 0 = linear, 1 = radial
    quality: u32, // 16, 32, 64, 128, 256, or 0 = smooth (per-pixel)
}

@group(0) @binding(0) var<uniform> grad: GradientUniforms;
@group(0) @binding(1) var grad_tex: texture_2d<f32>;
@group(0) @binding(2) var grad_sampler: sampler;

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> @location(0) vec4<f32> {
    var pos = vec2<f32>(
        select(-1.0, 3.0, vid == 1u),
        0.0, 1.0
    );
    let uv = vec2<f32>(
        select(0.0, 2.0, vid == 1u),
        select(0.0, 2.0, vid > 0u),
    );
    return vec4<f32>(pos, 0.0, 1.0);
}

fn interpolate_gradient(t: f32) -> vec4<f32> {
    let num_stops = i32(grad.num_stops);
    if num_stops <= 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    if num_stops == 1 {
        return textureLoad(grad_tex, vec2<u32>(0u, 0u), 0).rgba;
    }

    let clamped_t = clamp(t, 0.0, 1.0);

    for (var i = 0; i < num_stops - 1; i = i + 1) {
        let stop_i = textureLoad(grad_tex, vec2<u32>(i, 0u), 0).rgba;
        let stop_j = textureLoad(grad_tex, vec2<u32>(i + 1u, 0u), 0).rgba;

        // Store stop position in alpha channel of stop_i
        let pos_i = stop_i.a;
        let pos_j = stop_j.a;

        // Store color in rgb channels
        let color_i = stop_i.rgb;
        let color_j = stop_j.rgb;

        if clamped_t >= pos_i && clamped_t <= pos_j {
            let range = pos_j - pos_i;
            let local_t = select((clamped_t - pos_i) / range, 0.0, range <= 0.0);
            return mix(vec4<f32>(color_i, 1.0), vec4<f32>(color_j, 1.0), local_t);
        }
    }
    // Fallback: return first stop
    return textureLoad(grad_tex, vec2<u32>(0u, 0u), 0).rgba;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t: f32;

    if grad.mode == 0u {
        // Linear gradient: interpolate along angle
        let dir = vec2<f32>(cos(grad.angle_radians), sin(grad.angle_radians));
        t = dot(uv - 0.5, dir) + 0.5;
    } else {
        // Radial gradient: interpolate from center outward
        t = length(uv - 0.5) * 2.0;
    }

    return interpolate_gradient(t);
}

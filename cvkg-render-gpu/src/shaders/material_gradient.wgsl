//! Material shader — Gradient and shadow rendering path.
//! Handles modes: 15 (animated linear gradient), 18 (drop shadow).

#import common.wgsl

fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;

    if in.material_id == 15u {
        let angle = in.uv.x + scene.time * 0.5;
        let t = dot(in.logical / in.size - 0.5, vec2(cos(angle), sin(angle))) + 0.5;
        let end_color = vec4<f32>(in.slice.rgb, in.color.a);
        color = mix(in.color, end_color, clamp(t, 0.0, 1.0));
    } else if in.material_id == 18u {
        // Drop Shadow Logic
        let margin = in.uv.x;
        let blur = max(in.uv.y, 1.0);
        let original_size = in.size - 2.0 * margin;
        let half_size = original_size * 0.5;
        let p = in.logical - margin - half_size;
        let d = sd_round_rect(p, half_size - in.radius, in.radius);
        color.a *= smoothstep(blur, 0.0, d);
    }

    if color.a <= 0.0 { discard; }
    return color;
}

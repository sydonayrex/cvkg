//! Material shader — Glass/frosted-glass rendering path.
//! Handles mode 7 only. Separated from opaque to reduce register pressure
//! (glass shader is ~150 lines of complex math vs ~100 for opaque).
//! Glass samples the backdrop blur mip chain via textureSampleLevel(t_env, s_env, uv, blur_mip).

#import common.wgsl

fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));

    // SDF Clipping (same as opaque)
    let p_clip_pos = in.clip.xy * scene.scale_factor;
    let p_clip_size = in.clip.zw * scene.scale_factor;
    let pixel_pos = (in.clip_position.xy * 0.5 + 0.5) * scene.resolution * scene.scale_factor;
    let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
    var clip_alpha = 1.0 - smoothstep(-1.0, 1.0, clip_d);
    if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
    color.a *= clip_alpha;

    // 1. Screen-Space UV & Clamped Sampling
    let uv = clamp(in.uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    let panel_id = floor(in.uv.x * 3.0);
    let seed = fract(sin(panel_id * 91.7) * 47453.5453);
    let variation = 0.85 + seed * 0.3;

    // 2. Local Lensing Direction (Structured Bending)
    let local = in.logical / in.size;
    let centered = local - vec2<f32>(0.5, 0.5);
    let lens_dir = normalize(centered + vec2<f32>(1e-5, 1e-5));
    let lens_dist = length(centered);
    let fresnel = pow(lens_dist * 1.8, 2.5);

    // 3. Lens Distortion (Stronger near edges)
    let lens = lens_dir * lens_dist * 0.08 * variation;

    // 4. Subtle Material Noise (Instability)
    let hash_noise = vec2<f32>(
        fract(sin(dot(local, vec2<f32>(12.9898, 78.233))) * 43758.5453),
        fract(sin(dot(local, vec2<f32>(93.9898, 67.345))) * 24634.6345)
    ) * 0.01;

    // 5. Directional Stress (Organic internal flow)
    let noise1 = fbm(uv * 6.0 + scene.time * 0.2);
    let stress_offset = normalize(vec2<f32>(0.5, 0.8)) * noise1 * 0.02;

    // 6. Reactive Warp (Fire interaction)
    let blur_mip = theme.glass_blur_strength;
    let env_base = textureSampleLevel(t_env, s_env, uv, blur_mip).rgb;
    let brightness = dot(env_base, vec3<f32>(0.299, 0.587, 0.114));

    // Final Refraction Offset
    var distortion = lens * 1.2;
    distortion += stress_offset * 0.6;
    distortion += hash_noise * 0.3;
    distortion *= (1.0 + brightness * 0.7);
    distortion *= 2.0;

    // 7. Chromatic Aberration (High Impact RGB Split)
    let ab_offset = distortion * 0.04;
    let r_sample = textureSampleLevel(t_env, s_env, uv + distortion + ab_offset * 1.2, blur_mip).r;
    let g_sample = textureSampleLevel(t_env, s_env, uv + distortion, blur_mip).g;
    let b_sample = textureSampleLevel(t_env, s_env, uv + distortion - ab_offset * 1.2, blur_mip).b;
    let refracted = vec3<f32>(r_sample, g_sample, b_sample);

    // High-Fidelity Thickness Layering
    let half_size = in.size * 0.5;
    let p_sdf = in.logical - half_size;
    let q_sdf = abs(p_sdf) - (half_size - in.radius);
    let d_sdf = length(max(q_sdf, vec2(0.0))) + min(max(q_sdf.x, q_sdf.y), 0.0) - in.radius;

    let d_norm = clamp(-d_sdf / 20.0, 0.0, 1.0);
    let flicker = 0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1;
    let rim_light = smoothstep(1.0, 0.96, d_norm) * 0.25 * flicker;
    let inner_absorption = smoothstep(0.96, 0.88, d_norm) * 0.15;

    let tint = vec3<f32>(0.85, 0.9, 1.0);
    var final_rgb = refracted * tint;
    final_rgb += (brightness * 0.2) * flicker;
    final_rgb += rim_light * vec3<f32>(0.7, 1.0, 1.3);
    final_rgb -= inner_absorption;

    // Specular Highlight
    let light_dir_h = normalize(vec2<f32>(-0.4, -0.8));
    let l = dot(uv, light_dir_h);
    let spec = smoothstep(0.45, 0.55, l) * 0.12;
    final_rgb += spec;

    color = vec4<f32>(final_rgb, 0.02 + fresnel * 0.15);
    color.a *= (1.0 - smoothstep(-fw, fw, d_sdf));

    if color.a <= 0.0 { discard; }
    return color;
}

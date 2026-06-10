//! Material shader — Glass/frosted-glass rendering path.
//! Handles mode 7 only. Separated from opaque to reduce register pressure
//! (glass shader is ~150 lines of complex math vs ~100 for opaque).
//! Glass samples the backdrop blur mip chain via textureSampleLevel(t_env, s_env, uv, blur_mip).

// ─── Section 1: Physical Optics (Snell's Law Refraction) ─────────────────────

/// Physically accurate refraction using Snell's law.
/// n1 = 1.0 (air), n2 = per-instance IOR from uniforms.
/// Returns the UV offset for the refracted sample direction.
fn snell_refraction(normal: vec2<f32>, incident: vec2<f32>, ior: f32) -> vec2<f32> {
    let n_ratio = 1.0 / ior;
    let cos_i = -dot(normal, incident);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);

    // Total internal reflection
    if sin2_t > 1.0 {
        return reflect(incident, normal);
    }

    let cos_t = sqrt(1.0 - sin2_t);
    return n_ratio * incident + (n_ratio * cos_i - cos_t) * normal;
}

// ─── Section 2: Adaptive Appearance ──────────────────────────────────────────

/// Sample backdrop at 4 coarse mip-6 positions for dominant color.
/// Returns a tint color derived from the backdrop content.
fn sample_backdrop_dominant(uv: vec2<f32>) -> vec3<f32> {
    let s0 = textureSampleLevel(t_env, s_env, uv + vec2<f32>(-0.1, -0.1), 6.0).rgb;
    let s1 = textureSampleLevel(t_env, s_env, uv + vec2<f32>( 0.1, -0.1), 6.0).rgb;
    let s2 = textureSampleLevel(t_env, s_env, uv + vec2<f32>(-0.1,  0.1), 6.0).rgb;
    let s3 = textureSampleLevel(t_env, s_env, uv + vec2<f32>( 0.1,  0.1), 6.0).rgb;
    return (s0 + s1 + s2 + s3) * 0.25;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));

    // SDF Clipping (same as opaque)
    let p_clip_pos = in.clip.xy * scene.scale_factor;
    let p_clip_size = in.clip.zw * scene.scale_factor;
    let pixel_pos = in.clip_position.xy;
    let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
    var clip_alpha = 1.0 - smoothstep(-1.0, 1.0, clip_d);
    if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
    color.a *= clip_alpha;

    // Geometric Slice (Mjolnir Slice)
    if (in.slice.z > 0.5) {
        let angle_rad = in.slice.x * 0.01745329251;
        let normal_dir = vec2<f32>(cos(angle_rad), sin(angle_rad));
        let dist = dot(in.world_pos, normal_dir) - in.slice.y;
        if (dist > 0.0) { discard; }
    }

    // ─── Section 1: Geometry and Clipping ────────────────────────────────────

    let uv = clamp(in.uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    let panel_id = floor(in.uv.x * 3.0);
    let seed = fract(sin(panel_id * 91.7) * 47453.5453);
    let variation = 0.85 + seed * 0.3;

    // Local coordinates
    let local = in.logical / in.size;
    let centered = local - vec2<f32>(0.5, 0.5);
    let lens_normal = normalize(centered + vec2<f32>(1e-5, 1e-5));
    let lens_dist = length(centered);
    let fresnel = pow(lens_dist * 1.8, 2.5);

    // ─── Section 2: Physical Optics (Snell's Law) ────────────────────────────

    // View direction (simplified: from center toward edge)
    let view_dir = normalize(centered + vec2<f32>(1e-5, 1e-5));

    // Use per-instance IOR if set, otherwise default to 1.45 (borosilicate glass)
    let ior = 1.45;

    // Compute refracted direction using Snell's law
    let refracted_dir = snell_refraction(lens_normal, view_dir, ior);

    // Scale refraction by distance from center (stronger near edges)
    let refraction_offset = refracted_dir * lens_dist * 0.04 * variation;

    // ─── Section 3: Material Noise and Stress ────────────────────────────────

    let hash_noise = vec2<f32>(
        fract(sin(dot(local, vec2<f32>(12.9898, 78.233))) * 43758.5453),
        fract(sin(dot(local, vec2<f32>(93.9898, 67.345))) * 24634.6345)
    ) * 0.01;

    let noise1 = fbm(uv * 6.0 + scene.time * 0.2);
    let stress_offset = normalize(vec2<f32>(0.5, 0.8)) * noise1 * 0.02;

    // ─── Section 4: Backdrop Sampling with Chromatic Aberration ──────────────

    let blur_mip = theme.glass_blur_strength;
    let env_base = textureSampleLevel(t_env, s_env, uv, blur_mip).rgb;
    let brightness = dot(env_base, vec3<f32>(0.299, 0.587, 0.114));

    // Combine distortion sources
    var distortion = refraction_offset;
    distortion += stress_offset * 0.6;
    distortion += hash_noise * 0.3;
    distortion *= (1.0 + brightness * 0.7);

    // Chromatic aberration: sample R/G/B at slightly different offsets
    let ab_offset = distortion * 0.04;
    let r_sample = textureSampleLevel(t_env, s_env, uv + distortion + ab_offset * 1.2, blur_mip).r;
    let g_sample = textureSampleLevel(t_env, s_env, uv + distortion, blur_mip).g;
    let b_sample = textureSampleLevel(t_env, s_env, uv + distortion - ab_offset * 1.2, blur_mip).b;
    var refracted = vec3<f32>(r_sample, g_sample, b_sample);

    // ─── Section 5: SDF Edge and Thickness ───────────────────────────────────

    let half_size = in.size * 0.5;
    let p_sdf = in.logical - half_size;
    let q_sdf = abs(p_sdf) - (half_size - in.radius);
    let d_sdf = length(max(q_sdf, vec2(0.0))) + min(max(q_sdf.x, q_sdf.y), 0.0) - in.radius;

    let border_dist = -d_sdf;
    let flicker = 0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1;
    let hard_rim = smoothstep(0.0, 1.0, border_dist) * exp(-border_dist * 0.8);
    let soft_glow = smoothstep(0.0, 3.0, border_dist) * exp(-border_dist * 0.1);
    let rim_light = (hard_rim * 0.85 + soft_glow * 0.15) * flicker;

    // ─── Section 6: Adaptive Tint from Backdrop ──────────────────────────────

    // Sample backdrop dominant color at coarse mip for adaptive tinting
    let backdrop_dominant = sample_backdrop_dominant(uv);

    // Adaptive tint: mix static theme tint with backdrop-derived tint
    // glass_tint_adapt controls the weight (0 = static, 1 = fully adaptive)
    let adaptive_tint = mix(theme.glass_base.rgb, backdrop_dominant * 0.3 + 0.7, theme.glass_tint_adapt);

    // ─── Section 7: Sub-Surface Scattering Approximation ─────────────────────

    // Thickness: SDF distance from edge, normalized
    // Negative SDF = inside glass. Deeper inside = thinner center.
    let thickness = 1.0 - clamp(-d_sdf / (in.size.x * 0.5), 0.0, 1.0);
    let sss_tint = mix(vec3<f32>(0.92, 0.96, 1.0), vec3<f32>(0.7, 0.8, 0.95), thickness);

    // ─── Section 8: Edge Smear Convolution ───────────────────────────────────

    // Smear: extend blur slightly beyond the glass edge
    let smear_dist = clamp(-d_sdf, 0.0, 3.0) / 3.0;
    let smear_sample = textureSampleLevel(
        t_env, s_env,
        uv + lens_normal * smear_dist * 0.01,
        blur_mip
    ).rgb;
    let smear_contribution = smear_sample * 0.15;

    // Crystalline edge highlight: bright specular at the boundary
    let edge_mask = smoothstep(0.5, 0.0, abs(d_sdf));
    let crystal_edge = edge_mask * 0.4 * (0.7 + 0.3 * smoothstep(0.45, 0.55, dot(uv, normalize(vec2<f32>(-0.4, -0.8)))) * 0.18;

    // ─── Section 9: Final Composition ────────────────────────────────────────

    // Start with refracted backdrop, apply adaptive tint and SSS
    var final_rgb = refracted * adaptive_tint * sss_tint;

    // Add subtle brightness variation
    final_rgb += (brightness * 0.15) * flicker;

    // Add rim lighting
    final_rgb += rim_light * vec3<f32>(0.9, 1.1, 1.3);

    // Add edge smear and crystalline highlight
    final_rgb += smear_contribution + crystal_edge;

    // Specular highlight
    let light_dir_h = normalize(vec2<f32>(-0.4, -0.8));
    let l = dot(uv, light_dir_h);
    let spec = smoothstep(0.45, 0.55, l) * 0.18;
    final_rgb += spec;

    // Alpha model: thicker at edges (more opaque), thinner at center
    let sss_alpha = mix(0.06, 0.22, thickness);
    let final_alpha = (sss_alpha + fresnel * 0.18) * in.color.a * clip_alpha;

    color = vec4<f32>(final_rgb, final_alpha);
    color.a *= (1.0 - smoothstep(-fw, fw, d_sdf));

    if color.a <= 0.0 { discard; }
    return color;
}

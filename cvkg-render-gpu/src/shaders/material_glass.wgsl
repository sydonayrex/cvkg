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

// ─── Section 1b: GGX Specular Highlight ─────────────────────────────────────

/// GGX/Trowbridge-Reitz normal distribution function.
fn ggx_ndf(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (3.14159265 * denom * denom);
}

/// Compute specular highlight for liquid glass surface.
/// light_dir: normalized direction to light source (in screen space)
/// view_dir: normalized view direction
/// normal: surface normal
/// roughness: surface roughness (0 = mirror, 1 = diffuse)
/// intensity: specular intensity multiplier
fn ggx_specular(light_dir: vec2<f32>, view_dir: vec2<f32>, normal: vec2<f32>, roughness: f32, intensity: f32) -> f32 {
    let half_vec = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(normal, half_vec), 0.0);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);

    // Fresnel (Schlick approximation)
    let f0 = 0.04; // IOR 1.5 glass
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - n_dot_v, 5.0);

    // GGX distribution
    let d = ggx_ndf(n_dot_h, roughness);

    // Geometry function (Smith's method)
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let g = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let g2 = g * g;

    let spec = (d * fresnel * g2) / (4.0 * n_dot_l * n_dot_v + 0.001);
    return spec * intensity;
}

// ─── Section 1c: Displacement Map (feDisplacementMap analog) ─────────────────

/// Compute displacement offset for liquid glass edge distortion.
/// Uses screen-space derivatives to create the "wet glass" edge effect.
/// The displacement is strongest at the edges and fades toward the center.
fn displacement_offset(
    uv: vec2<f32>,
    local: vec2<f32>,
    lens_dist: f32,
    lens_normal: vec2<f32>,
    time: f32,
) -> vec2<f32> {
    // Edge displacement: strongest at edges, fades inward
    let edge_factor = smoothstep(0.3, 0.5, lens_dist);

    // Use screen-space derivatives for distortion magnitude
    let dx = length(vec2(dpdx(uv.x), dpdy(uv.x)));
    let dy = length(vec2(dpdx(uv.y), dpdy(uv.y)));
    let deriv_scale = max(dx, dy) * 50.0;

    // Displacement direction: along the normal, pushing outward at edges
    let disp_mag = edge_factor * deriv_scale * 0.15;

    // Add subtle time-varying turbulence
    let turb = sin(local.x * 20.0 + time * 0.5) * cos(local.y * 20.0 + time * 0.3) * 0.002;

    return lens_normal * (disp_mag + turb * edge_factor);
}

// ─── Section 2: Adaptive Appearance ──────────────────────────────────────────

/// Sample backdrop at 9 positions (3x3 grid) at mip-4 for dominant color.
/// Uses a wider spread than the old 4-sample mip-6 approach for better
/// detection of high-frequency backdrop patterns that can kill legibility.
fn sample_backdrop_dominant(uv: vec2<f32>) -> vec3<f32> {
    let offsets = array<vec2<f32>, 9>(
        vec2<f32>(-0.15, -0.15), vec2<f32>(0.0, -0.15), vec2<f32>(0.15, -0.15),
        vec2<f32>(-0.15,  0.0),  vec2<f32>(0.0,  0.0),  vec2<f32>(0.15,  0.0),
        vec2<f32>(-0.15,  0.15), vec2<f32>(0.0,  0.15), vec2<f32>(0.15,  0.15)
    );
    var sum = vec3<f32>(0.0);
    for (var i = 0u; i < 9u; i++) {
        sum += textureSampleLevel(t_env, s_env, uv + offsets[i], 4.0).rgb;
    }
    return sum / 9.0;
}

/// Compute backdrop variance across the 9 sample positions.
/// Returns 0.0 (uniform) to 1.0 (high variance / busy backdrop).
/// `mean` is the pre-computed dominant color from sample_backdrop_dominant().
fn sample_backdrop_variance(uv: vec2<f32>, mean: vec3<f32>) -> f32 {
    let offsets = array<vec2<f32>, 9>(
        vec2<f32>(-0.15, -0.15), vec2<f32>(0.0, -0.15), vec2<f32>(0.15, -0.15),
        vec2<f32>(-0.15,  0.0),  vec2<f32>(0.0,  0.0),  vec2<f32>(0.15,  0.0),
        vec2<f32>(-0.15,  0.15), vec2<f32>(0.0,  0.15), vec2<f32>(0.15,  0.15)
    );
    var var_sum = 0.0;
    for (var i = 0u; i < 9u; i++) {
        let s = textureSampleLevel(t_env, s_env, uv + offsets[i], 4.0).rgb;
        let diff = s - mean;
        var_sum += dot(diff, diff);
    }
    return clamp(var_sum / 9.0 * 3.0, 0.0, 1.0);
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
    // Early exit for zero intensity: skip all expensive glass computation
    let gi = in.glass_intensity;
    if gi < 0.01 {
        let alpha = color.a * (1.0 - smoothstep(-fw, fw, d_sdf));
        if alpha <= 0.0 { discard; }
        return vec4<f32>(color.rgb, alpha);
    }


    // Geometric Slice (Mjolnir Slice)
    if (in.slice.z > 0.5) {
        let angle_rad = in.slice.x * 0.01745329251;
        let normal_dir = vec2<f32>(cos(angle_rad), sin(angle_rad));
        let dist = dot(in.world_pos, normal_dir) - in.slice.y;
        if (dist > 0.0) { discard; }
    }

    // ─── Section 1: Geometry and Clipping ────────────────────────────────────

    let uv = clamp(in.uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    let screen_uv = in.clip_position.xy / (scene.resolution * scene.scale_factor);

    // Clean, constant variation factor to prevent high-frequency Moire/brushed-metal artifacts on rotating cards
    let variation = 1.0;

    // Local coordinates
    let local = in.logical / in.size;
    let centered = local - vec2<f32>(0.5, 0.5);
    let lens_normal = normalize(centered + vec2<f32>(1e-5, 1e-5));
    let lens_dist = length(centered);
    let fresnel = pow(lens_dist * 1.8, 2.5);

    // ─── Section 2: Physical Optics (Snell's Law) ────────────────────────────

    // View direction (simplified: from center toward edge)
    let view_dir = normalize(centered + vec2<f32>(1e-5, 1e-5));

    // Resolve IOR: prioritize per-instance ior_override, fall back to theme, then standard borosilicate (1.45)
    let base_ior = select(1.45, theme.glass_ior, theme.glass_ior > 0.0);
    let ior = select(base_ior, in.ior_override, in.ior_override > 0.0);

    // Compute refracted direction using Snell's law
    let refracted_dir = snell_refraction(lens_normal, view_dir, ior);

    // Non-trivial algorithm: Magnifying Lens Refraction
    // WHY: Traditional Snell refraction on a 2D quad often shrinks/displaces the backdrop. To simulate
    // a premium convex liquid/ice lens, we must shift the texture coordinate inward towards the center.
    // Near the center, magnification is strongest. Near the edges, it transitions to refraction roll-off
    // to prevent edge-sampling artifacts outside the card bounds.
    // CONTRACT: Returns a vec2 offset that contracts screen UV lookups towards the card center.
    let mag_strength = 0.16 * (1.0 - smoothstep(0.0, 0.8, lens_dist));
    var refraction_offset = refracted_dir * lens_dist * 0.08 * variation;
    refraction_offset += -view_dir * lens_dist * mag_strength;

    // ─── Section 3: Material Noise and Stress ────────────────────────────────

    let hash_noise = vec2<f32>(
        fract(sin(dot(local, vec2<f32>(12.9898, 78.233))) * 43758.5453),
        fract(sin(dot(local, vec2<f32>(93.9898, 67.345))) * 24634.6345)
    ) * 0.01;

    let noise1 = vnoise(uv * 6.0 + scene.time * 0.2);
    let stress_offset = normalize(vec2<f32>(0.5, 0.8)) * noise1 * 0.02;

    // ─── Section 5: SDF Edge and Thickness ───────────────────────────────────

    let half_size = in.size * 0.5;
    let squircle_n = select(0.0, in.slice.y, in.slice.y > 1.5);
    var d_sdf: f32;
    if (squircle_n > 1.5) {
        d_sdf = sd_squircle(in.logical - half_size, half_size, squircle_n);
    } else {
        d_sdf = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
    }

    // ─── Section 4: Backdrop Sampling with Chromatic Aberration ──────────────

    // Use per-element blur radius, falling back to theme default if 0
    let blur_mip = select(theme.glass_blur_strength, in.blur_radius, in.blur_radius > 0.0);
    let env_base = textureSampleLevel(t_env, s_env, screen_uv, blur_mip).rgb;
    let brightness = dot(env_base, vec3<f32>(0.299, 0.587, 0.114));

    // Combine distortion sources
    var distortion = refraction_offset;
    distortion += stress_offset * 0.6;
    distortion += hash_noise * 0.3;

    // Tactical pointer proximity hover pressure/refraction distortion
    let frag_logical_pos = in.clip_position.xy / scene.scale_factor;
    let dist_to_mouse = distance(frag_logical_pos, scene.mouse);
    let hover_radius = 120.0;
    if (dist_to_mouse < hover_radius) {
        let hover_factor = 1.0 - (dist_to_mouse / hover_radius);
        let hover_pulse = smoothstep(0.0, 1.0, hover_factor);
        let hover_dir = normalize(frag_logical_pos - scene.mouse + vec2<f32>(1e-5, 1e-5));
        let mouse_speed = length(scene.mouse_velocity);
        let hover_displacement = hover_dir * hover_pulse * (0.015 + mouse_speed * 0.003);
        distortion += hover_displacement;
    }

    distortion *= (1.0 + brightness * 0.7);

    // Dynamic shape masking: Scale down distortion near the glass edges to prevent sampling pixels outside the glass geometry.
    // If d_sdf is positive (outside/near edge), we clamp the distortion.
    let dist_fade = smoothstep(10.0, 0.0, d_sdf);
    let safe_distortion = distortion * dist_fade;

    // Chromatic aberration: sample R/G/B at slightly different offsets
    let ab_offset = safe_distortion * 0.04;
    let r_sample = textureSampleLevel(t_env, s_env, screen_uv + safe_distortion + ab_offset * 1.2, blur_mip).r;
    let g_sample = textureSampleLevel(t_env, s_env, screen_uv + safe_distortion, blur_mip).g;
    let b_sample = textureSampleLevel(t_env, s_env, screen_uv + safe_distortion - ab_offset * 1.2, blur_mip).b;
    var refracted = vec3<f32>(r_sample, g_sample, b_sample);

    let border_dist = -d_sdf;
    let flicker = 0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1;
    let hard_rim = smoothstep(0.0, 1.0, border_dist) * exp(-border_dist * 0.8);
    let soft_glow = smoothstep(0.0, 3.0, border_dist) * exp(-border_dist * 0.1);
    let rim_light = (hard_rim * 0.85 + soft_glow * 0.15) * flicker;

    // ─── Section 6: Adaptive Tint from Backdrop ──────────────────────────────

    // Sample backdrop dominant color and variance for adaptive tinting
    let backdrop_dominant = sample_backdrop_dominant(screen_uv);
    let backdrop_var = sample_backdrop_variance(screen_uv, backdrop_dominant);

    // Adaptive tint: mix static theme tint with backdrop-derived tint.
    // High variance (busy backdrop) reduces adaptation to prevent legibility issues.
    // glass_tint_adapt controls the max weight (0 = static, 1 = fully adaptive).
    let effective_adapt = theme.glass_tint_adapt * (1.0 - backdrop_var);
    let adaptive_tint = mix(theme.glass_base.rgb, backdrop_dominant * 0.3 + 0.7, effective_adapt);

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
        screen_uv + lens_normal * smear_dist * 0.01,
        blur_mip
    ).rgb;
    let smear_contribution = smear_sample * 0.15;

    // Crystalline edge highlight: bright specular at the boundary
    let edge_mask = smoothstep(0.5, 0.0, abs(d_sdf));
    let crystal_edge = edge_mask * 0.4 * (0.7 + 0.3 * smoothstep(0.45, 0.55, dot(uv, normalize(vec2<f32>(-0.4, -0.8))))) * 0.18;

    // ─── Section 9: Displacement + Specular ────────────────────────────────────

    // Apply displacement offset to the refraction for "wet glass" edge distortion
    let disp = displacement_offset(screen_uv, local, lens_dist, lens_normal, scene.time);
    let displaced_screen_uv = screen_uv + disp * dist_fade;
    let displaced_refracted = vec3<f32>(
        textureSampleLevel(t_env, s_env, displaced_screen_uv + safe_distortion * 0.04, blur_mip).r,
        textureSampleLevel(t_env, s_env, displaced_screen_uv, blur_mip).g,
        textureSampleLevel(t_env, s_env, displaced_screen_uv - safe_distortion * 0.04, blur_mip).b,
    );

    // Blend between normal refraction and displaced refraction based on edge proximity
    let disp_blend = smoothstep(0.3, 0.5, lens_dist) * 0.6;
    refracted = mix(refracted, displaced_refracted, disp_blend);

    // GGX specular highlight — dynamic light from fireball position
    // Compute light direction from fireball position to fragment world position
    let frag_world = in.clip_position.xy / scene.scale_factor;
    let to_fireball = scene.fireball_pos - frag_world;
    let fireball_dist = length(to_fireball);
    // Normalize; default to top-left if fireball is at origin (uninitialized)
    let light_dir = select(normalize(vec2<f32>(-0.6, -0.8)), normalize(to_fireball + vec2<f32>(1e-5, 1e-5)), fireball_dist > 1.0);
    // Specular intensity falls off with distance (closer fireball = brighter spec)
    let fireball_intensity = 2.5 * clamp(300.0 / (fireball_dist + 100.0), 0.0, 1.0);
    let spec = ggx_specular(light_dir, view_dir, lens_normal, 0.15, fireball_intensity);
    let specular_contribution = spec * vec3<f32>(1.0, 0.98, 0.95) * (1.0 - fresnel * 0.5);

    // ─── Section 10: Final Composition ───────────────────────────────────────

    // Start with refracted backdrop, apply adaptive tint and SSS
    var final_rgb = refracted * adaptive_tint * sss_tint;

    // Add subtle brightness variation
    final_rgb += (brightness * 0.15) * flicker;

    // Add rim lighting
    final_rgb += rim_light * vec3<f32>(0.9, 1.1, 1.3);

    // Add edge smear and crystalline highlight
    final_rgb += smear_contribution + crystal_edge;

    // Add GGX specular highlight
    final_rgb += specular_contribution * smoothstep(0.0, 0.4, 1.0 - lens_dist);


    // Apply SDF anti-aliasing to glass alpha
    let glass_alpha = color.a * (1.0 - smoothstep(-fw, fw, d_sdf));

    // Modulate glass effect by per-instance glass_intensity.
    // intensity=0 -> simple transparent fill (no refraction/blur/rim/spec)
    // intensity=1 -> full glass effect
    final_rgb = mix(color.rgb, final_rgb, gi);
    let final_alpha = mix(color.a * 0.3, glass_alpha, gi);
    color = vec4<f32>(final_rgb, final_alpha);

    if color.a <= 0.0 { discard; }
    return color;
}

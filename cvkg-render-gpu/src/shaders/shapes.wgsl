@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos = in.position.xy;

    // ── Mode 13 (3D Surface): Skip 2D transforms, use full MVP ──────────
    if (in.mode == 13u) {
        // Position is already model-space (transformed on CPU by draw_mesh_3d).
        // Apply only view and projection on GPU.
        out.clip_position = scene.proj * scene.view * vec4<f32>(in.position, 1.0);
    } else {
    // Apply 2D Transform: Rotate -> Scale -> Translate
    // Rotation
    let s2 = sin(in.rotation);
    let c2 = cos(in.rotation);
    let rot_matrix = mat2x2<f32>(c2, s2, -s2, c2);
    pos = rot_matrix * pos;

    // Scale
    pos = pos * in.scale;

    // Translation
    pos = pos + in.translation;

    // ── Hardware Shatter Effect (Berserker Physics) ─────────────────────
    let shatter_dt = scene.time - scene.shatter_time;
    if (shatter_dt > 0.0 && shatter_dt < 2.0) {
        // Calculate displacement from shatter origin
        let dist = distance(pos, scene.shatter_origin);
        let dir = normalize(pos - scene.shatter_origin + vec2<f32>(1e-5, 1e-5));

        // Force falloff: stronger near origin, decays over time
        let explosion = (1.0 / (dist * 0.01 + 0.1)) * scene.shatter_force;
        let expansion = explosion * shatter_dt * 100.0;

        pos += dir * expansion;
    }

    out.clip_position = scene.proj * scene.view * vec4<f32>(pos, in.position.z, 1.0);
    }
    out.uv = in.uv;
    out.color = in.color;
    out.mode = in.mode;
    out.radius = in.radius;
    out.slice = in.slice;
    out.logical = in.logical;
    out.size = in.size;
    out.screen = in.screen;
    out.normal = in.normal;
    out.clip = in.clip;
    out.tex_index = in.tex_index;
    
    // Orthographic projection: [0, width] -> [-1, 1], [0, height] -> [1, -1]
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));
    
    // ── High-Fidelity SDF Clipping ───────────────────────────────────────
    let p_clip_pos = in.clip.xy * scene.scale_factor;
    let p_clip_size = in.clip.zw * scene.scale_factor;
    let pixel_pos = (in.clip_position.xy * 0.5 + 0.5) * scene.resolution * scene.scale_factor;
    
    // Using a soft-edged box SDF for sub-pixel anti-aliased clipping
    let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
    var clip_alpha = 1.0 - smoothstep(-1.0, 1.0, clip_d); 
    
    // Large sentinel check (e.g., -10000) to bypass clipping for global elements
    if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
    color.a *= clip_alpha;
    

    if in.mode == 1u {
        // Neon Line
        color = in.color * 1.5; // Slight boost
    } else if in.mode == 3u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let aa = fwidth(d);
        color.a *= 1.0 - smoothstep(0.0, aa, d);
    } else if in.mode == 4u {
        let half_size = in.size * 0.5;
        let safe_half = max(half_size, vec2<f32>(0.001));
        let d = length((in.logical - half_size) / safe_half) - 1.0;
        let aa = fwidth(d);
        color.a *= 1.0 - smoothstep(0.0, aa, d);
    } else if in.mode == 7u {
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
        
        // 6. High-Fidelity Thickness Layering (Depth Pressure + Instability)
        let half_size = in.size * 0.5;
        let p_sdf = in.logical - half_size;
        let q_sdf = abs(p_sdf) - (half_size - in.radius);
        let d_sdf = length(max(q_sdf, vec2(0.0))) + min(max(q_sdf.x, q_sdf.y), 0.0) - in.radius;
        
        // Normalize distance: 1.0 at edge, 0.0 inside (capped at 20px)
        let d_norm = clamp(-d_sdf / 20.0, 0.0, 1.0); 

        // Berserker Instability (Flicker/Jitter)
        let flicker = 0.9 + vnoise(uv * 20.0 + scene.time * 3.0) * 0.1;

        // Stage 1: Edge Highlight (Rim Light)
        let rim_light = smoothstep(1.0, 0.96, d_norm) * 0.25 * flicker;
        
        // Stage 2: Inner Absorption (Depth Illusion)
        let inner_absorption = smoothstep(0.96, 0.88, d_norm) * 0.15;

        let tint = vec3<f32>(0.85, 0.9, 1.0); // Subtle cool-blue tint
        var final_rgb = refracted * tint;
        
        // Composite Thickness Layers (Reactive + Volatile)
        final_rgb += (brightness * 0.2) * flicker; // Unstable fire interaction
        final_rgb += rim_light * vec3<f32>(0.7, 1.0, 1.3); // Edge highlight
        final_rgb -= inner_absorption; // Inner volume depth

        // 7. Specular Highlight
        let light_dir_h = normalize(vec2<f32>(-0.4, -0.8));
        let l = dot(uv, light_dir_h);
        let spec = smoothstep(0.45, 0.55, l) * 0.12;
        final_rgb += spec;
        
        color = vec4<f32>(final_rgb, 0.01 + fresnel * 0.01);
        color.a *= (1.0 - smoothstep(-fw, fw, d_sdf));

} else if in.mode == 13u {
        // ── Mode 13: 3D Surface — Basic PBR Lighting Model ──────────────
        // slice.x = metallic, slice.y = roughness, slice.z = opacity
        let metallic = in.slice.x;
        let roughness = in.slice.y;
        let opacity  = in.slice.z;

        // Normalize the interpolated normal
        let n = normalize(in.normal);

        // Basic directional light (top-right-front)
        let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.6));
        let light_color = vec3<f32>(1.0, 0.95, 0.9);

        // Diffuse (Lambert)
        let n_dot_l = max(dot(n, light_dir), 0.0);
        let diffuse = n_dot_l * light_color;

        // Specular (Blinn-Phong with roughness control)
        let view_dir = vec3<f32>(0.0, 0.0, 1.0); // Approximate view direction
        let half_dir = normalize(light_dir + view_dir);
        let n_dot_h = max(dot(n, half_dir), 0.0);
        // Shininess from roughness: lower roughness = higher shininess
        let shininess = mix(8.0, 256.0, 1.0 - roughness);
        let spec = pow(n_dot_h, shininess) * light_color;

        // Fresnel approximation (Schlick)
        let f0 = mix(vec3<f32>(0.04), in.color.rgb, metallic);
        let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - max(dot(n, -view_dir), 0.0), 5.0);

        // Ambient with theme influence
        let ambient = vec3<f32>(0.06, 0.07, 0.1);

        // Combine: diffuse for dielectrics, specular tinted by base color for metals
        var lit_color = in.color.rgb * (ambient + diffuse);
        lit_color += spec * mix(vec3<f32>(1.0), in.color.rgb, metallic) * fresnel;

        // Simple fog falloff for depth perception
        let depth = in.clip_position.z;
        let fog_factor = clamp(1.0 - depth * 0.0005, 0.7, 1.0);
        lit_color *= fog_factor;

        color = vec4<f32>(lit_color, in.color.a * opacity);

    } else if in.mode == 18u {
        // Drop Shadow Logic (Mode 18u)
        let margin = in.uv.x;
        let blur = max(in.uv.y, 1.0);
        let original_size = in.size - 2.0 * margin;
        let half_size = original_size * 0.5;
        let p = in.logical - margin - half_size;
        let d = sd_round_rect(p, half_size - in.radius, in.radius);
        
        // Grounding Falloff (Smooth linear step for UI anchor)
        color.a *= smoothstep(blur, 0.0, d);
    } else if in.mode == 9u {
        let d = length((in.uv - 0.5) * vec2<f32>(1.0, 4.0));
        color = theme.primary_neon * neon_glow(d, 0.01, 0.2);
    } else if in.mode == 10u {
        let p = (in.uv - 0.5) * 2.0;
        let d = min(sd_segment(p, vec2(-0.5, -0.8), vec2(0.5, 0.8)), sd_segment(p, vec2(0.5, -0.8), vec2(-0.5, 0.8)));
        color = theme.rune_glow * neon_glow(d, 0.02, 0.15) * theme.rune_opacity;
    } else if in.mode == 16u {
        // Radial Gradient Logic
        let dist = length(in.uv - 0.5) * 2.0; // 0 at center, 1 at edge
        let t = clamp(dist, 0.0, 1.0);
        let end_color = vec4<f32>(in.slice.rgb, in.slice.a);
        color = mix(in.color, end_color, t);
    } else if in.mode == 14u {
        // Ray Marched Reflections
        let ro = vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, -2.0);
        let rd = normalize(vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, 1.0));
        let t = ray_march(ro, rd);
        if t > 0.0 {
            let p = ro + rd * t;
            let n = calc_normal(p);
            let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
            let diff = max(dot(n, light_dir), 0.2);
            let ref_rd = reflect(rd, n);
            let ref_t = ray_march(p + n * 0.01, ref_rd);
            var reflection_color = vec3<f32>(0.05, 0.05, 0.1);
            if ref_t > 0.0 { reflection_color = mix(theme.primary_neon.rgb, theme.shatter_neon.rgb, 0.5); }
            color = vec4<f32>(mix(in.color.rgb * diff, reflection_color, 0.3), 1.0);
        } else { discard; }
    } else if in.mode == 15u {
        let angle = in.uv.x + scene.time * 0.5;
        let t = dot(in.logical / in.size - 0.5, vec2(cos(angle), sin(angle))) + 0.5;
        let end_color = vec4<f32>(in.slice.rgb, in.color.a);
        color = mix(in.color, end_color, clamp(t, 0.0, 1.0));
    } else if in.mode == 17u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let thickness = max(in.slice.x, 1.0);
        color.a *= (1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5));
    } else if in.mode == 19u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let thickness = max(in.slice.x, 1.0);
        let perimeter = (in.uv.x + in.uv.y) * max(in.size.x, in.size.y);
        var alpha = 1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5);
        if (perimeter + scene.time * 20.0) % (max(in.slice.y, 1.0) + max(in.slice.z, 1.0)) > max(in.slice.y, 1.0) { alpha = 0.0; }
        color.a *= alpha;
    } else if in.mode == 2u || in.mode == 6u {
        let tex_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
        if in.mode == 6u {
            // Premultiplied blending for single-channel font atlas
            color = vec4<f32>(in.color.rgb, in.color.a * tex_color.a);
        } else {
            color *= tex_color;
        }
    } else if in.mode == 12u {
        // Heatmap — sample texture and apply heatmap palette
        let val = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv).r;
        color = vec4<f32>(heatmap_palette(val), in.color.a);
    } else if in.mode == 20u {
        // 9-slice — textured rect with UV remapping handled on CPU via uv_rect
        // uv_rect.xy = inner_left, uv_rect.zw = inner_right (CPU-computed slice boundaries)
        let tex_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
        color *= tex_color;
    } else if in.mode == 21u {
        // High-Fidelity Raymarched Cube (Mode 21u)
        let uv = (in.uv - 0.5) * 2.0;
        let ro = vec3<f32>(0.0, 0.0, -2.5);
        let rd = normalize(vec3<f32>(uv.x, uv.y, 1.5));
        
        let m = rotX(in.slice.x) * rotY(in.slice.y) * rotZ(in.slice.z);
        
        var t = 0.0;
        var hit = false;
        var d = 0.0;
        for (var i = 0; i < 40; i++) {
            let p = m * (ro + rd * t);
            d = sd_box_3d(p, vec3(0.5, 0.5, 0.5));
            if d < 0.001 { hit = true; break; }
            t += d;
            if t > 5.0 { break; }
        }
        
        if hit {
            let p = m * (ro + rd * t);
            let eps = vec2(0.001, 0.0);
            let n = normalize(vec3(
                sd_box_3d(p + eps.xyy, vec3(0.5)) - sd_box_3d(p - eps.xyy, vec3(0.5)),
                sd_box_3d(p + eps.yxy, vec3(0.5)) - sd_box_3d(p - eps.yxy, vec3(0.5)),
                sd_box_3d(p + eps.yyx, vec3(0.5)) - sd_box_3d(p - eps.yyx, vec3(0.5))
            ));
            let light_dir = normalize(vec3(1.0, 1.0, -2.0));
            let diff = max(dot(n, light_dir), 0.1);
            let rim = pow(1.0 - max(dot(n, -rd), 0.0), 3.0) * 0.5;
            color = vec4<f32>(in.color.rgb * diff + rim, in.color.a);
        } else {
            discard;
        }
    } else if in.mode == 8u {
        // Neon Glow (Gungnir) — expanding additive glow layers
        let center = in.size * 0.5;
        let dist = length(in.logical - center) / max(in.size.x, in.size.y);
        let glow = exp(-dist * 4.0) * 1.5;
        color = vec4<f32>(color.rgb * glow, color.a);
    }
    
    let rage = scene.berzerker_rage;
    if rage > 0.05 {
        // Dynamic heat distortion and ember glow
        let noise_coord = in.logical * 0.05 + vec2(scene.time * 0.5);
        let n = fbm(noise_coord);
        
        // Pulsing glow based on rage level
        let pulse = 0.5 + 0.5 * sin(scene.time * 10.0 * rage);
        let rage_color = mix(theme.ember_core, theme.shatter_neon, pulse * 0.3);
        
        color = mix(color, rage_color, n * rage * 0.7);
        
        // Subtle chromatic aberration boost during rage
        if rage > 0.8 {
            color.r *= 1.1;
            color.b *= 0.9;
        }
    }
    if color.a <= 0.0 { discard; }
    return color;
}

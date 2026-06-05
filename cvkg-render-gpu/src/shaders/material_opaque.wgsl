//! Material shader — Opaque/2D rendering path.
//! Handles modes: 0 (solid), 1 (neon), 2 (texture), 3 (rounded), 4 (ellipse),
//! 6 (text), 8 (glow), 9 (lightning), 10 (rune), 12 (heatmap), 16 (radial grad),
//! 17 (stroke), 19 (dashed), 20 (9-slice).
//! Excludes: 7 (glass), 13/14/21 (3D PBR/raymarch), 15/18 (gradient/shadow).



fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));

    // ── High-Fidelity SDF Clipping ───────────────────────────────────────
    let p_clip_pos = in.clip.xy * scene.scale_factor;
    let p_clip_size = in.clip.zw * scene.scale_factor;
    let pixel_pos = (in.clip_position.xy * 0.5 + 0.5) * scene.resolution * scene.scale_factor;

    let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
    var clip_alpha = 1.0 - smoothstep(-1.0, 1.0, clip_d);

    if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
    color.a *= clip_alpha;

    if in.material_id == 1u {
        // Neon Line
        color = in.color * 1.5;
    } else if in.material_id == 3u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let aa = fwidth(d);
        color.a *= 1.0 - smoothstep(0.0, aa, d);
    } else if in.material_id == 4u {
        let half_size = in.size * 0.5;
        let safe_half = max(half_size, vec2<f32>(0.001));
        let d = length((in.logical - half_size) / safe_half) - 1.0;
        let aa = fwidth(d);
        color.a *= 1.0 - smoothstep(0.0, aa, d);
    } else if in.material_id == 8u {
        // Neon Glow (Gungnir)
        let center = in.size * 0.5;
        let dist = length(in.logical - center) / max(in.size.x, in.size.y);
        let glow = exp(-dist * 4.0) * 1.5;
        color = vec4<f32>(color.rgb * glow, color.a);
    } else if in.material_id == 9u {
        let d = length((in.uv - 0.5) * vec2<f32>(1.0, 4.0));
        color = theme.primary_neon * neon_glow(d, 0.01, 0.2);
    } else if in.material_id == 10u {
        let p = (in.uv - 0.5) * 2.0;
        let d = min(sd_segment(p, vec2(-0.5, -0.8), vec2(0.5, 0.8)), sd_segment(p, vec2(0.5, -0.8), vec2(-0.5, 0.8)));
        color = theme.rune_glow * neon_glow(d, 0.02, 0.15) * theme.rune_opacity;
    } else if in.material_id == 16u {
        // Radial Gradient Logic
        let dist = length(in.uv - 0.5) * 2.0;
        let t = clamp(dist, 0.0, 1.0);
        let end_color = vec4<f32>(in.slice.rgb, in.slice.a);
        color = mix(in.color, end_color, t);
    } else if in.material_id == 17u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let thickness = max(in.slice.x, 1.0);
        color.a *= (1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5));
    } else if in.material_id == 19u {
        let half_size = in.size * 0.5;
        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
        let thickness = max(in.slice.x, 1.0);
        let perimeter = (in.uv.x + in.uv.y) * max(in.size.x, in.size.y);
        var alpha = 1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5);
        if (perimeter + scene.time * 20.0) % (max(in.slice.y, 1.0) + max(in.slice.z, 1.0)) > max(in.slice.y, 1.0) { alpha = 0.0; }
        color.a *= alpha;
    } else if in.material_id == 2u || in.material_id == 6u {
        let tex_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
        if in.material_id == 6u {
            color = vec4<f32>(in.color.rgb, in.color.a * tex_color.a);
        } else {
            color *= tex_color;
        }
    } else if in.material_id == 12u {
        let val = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv).r;
        color = vec4<f32>(heatmap_palette(val), in.color.a);
    } else if in.material_id == 20u {
        let tex_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
        color *= tex_color;
    }

    // Rage effect (applied to all opaque modes)
    let rage = scene.berzerker_rage;
    if rage > 0.05 {
        let noise_coord = in.logical * 0.05 + vec2(scene.time * 0.5);
        let n = fbm(noise_coord);
        let pulse = 0.5 + 0.5 * sin(scene.time * 10.0 * rage);
        let rage_color = mix(theme.ember_core, theme.shatter_neon, pulse * 0.3);
        color = mix(color, rage_color, n * rage * 0.7);
        if rage > 0.8 {
            color.r *= 1.1;
            color.b *= 0.9;
        }
    }

    if color.a <= 0.0 { discard; }
    return color;
}

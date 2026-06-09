//! Material shader — Opaque/2D rendering path.
//! Handles all non-glass material modes: 0 (solid), 1 (neon), 2 (texture),
//! 3 (rounded), 4 (ellipse), 6 (text), 8 (glow), 9 (lightning), 10 (rune),
//! 12 (heatmap), 13 (PBR surface), 14 (raymarched reflections),
//! 15 (animated linear gradient), 16 (radial grad), 17 (stroke),
//! 18 (drop shadow), 19 (dashed), 20 (9-slice), 21 (raymarched cube).
//! Excludes: 7 (glass) — handled by material_glass.wgsl.



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));

    // ── High-Fidelity SDF Clipping ───────────────────────────────────────
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
    } else if in.material_id == 15u {
        // ── Mode 15: Animated Linear Gradient ──
        // Rotates a linear gradient across the element based on elapsed time to create dynamic flow.
        let angle = in.uv.x + scene.time * 0.5;
        let t = dot(in.logical / in.size - 0.5, vec2(cos(angle), sin(angle))) + 0.5;
        let end_color = vec4<f32>(in.slice.rgb, in.color.a);
        color = mix(in.color, end_color, clamp(t, 0.0, 1.0));
    } else if in.material_id == 18u {
        // ── Mode 18: Drop Shadow ──
        // Renders a soft drop shadow outside the margins of the rounded rectangle using smoothstep of the SDF.
        let margin = in.uv.x;
        let blur = max(in.uv.y, 1.0);
        let original_size = in.size - 2.0 * margin;
        let half_size = original_size * 0.5;
        let p = in.logical - margin - half_size;
        let d = sd_round_rect(p, half_size - in.radius, in.radius);
        color.a *= smoothstep(blur, 0.0, d);
    } else if in.material_id == 13u {
        // ── Mode 13: 3D Surface — Basic PBR Lighting ──
        // Simulates realistic lighting on a 3D surface mesh using diffuse, specular, fresnel reflection, and fog depth cues.
        let metallic = in.slice.x;
        let roughness = in.slice.y;
        let opacity  = in.slice.z;
        let n = normalize(in.normal);
        let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.6));
        let light_color = vec3<f32>(1.0, 0.95, 0.9);
        let n_dot_l = max(dot(n, light_dir), 0.0);
        let diffuse = n_dot_l * light_color;
        let view_dir = vec3<f32>(0.0, 0.0, 1.0);
        let half_dir = normalize(light_dir + view_dir);
        let n_dot_h = max(dot(n, half_dir), 0.0);
        let shininess = mix(8.0, 256.0, 1.0 - roughness);
        let spec = pow(n_dot_h, shininess) * light_color;
        let f0 = mix(vec3<f32>(0.04), in.color.rgb, metallic);
        let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - max(dot(n, -view_dir), 0.0), 5.0);
        let ambient = vec3<f32>(0.06, 0.07, 0.1);
        var lit_color = in.color.rgb * (ambient + diffuse);
        lit_color += spec * mix(vec3<f32>(1.0), in.color.rgb, metallic) * fresnel;
        let depth = in.clip_position.z;
        let fog_factor = clamp(1.0 - depth * 0.0005, 0.7, 1.0);
        lit_color *= fog_factor;
        color = vec4<f32>(lit_color, in.color.a * opacity);
    } else if in.material_id == 14u {
        // ── Mode 14: Raymarched Reflections ──
        // Renders reflections by marching a ray through a procedural 3D scene and computing lighting/reflection vectors.
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
    } else if in.material_id == 21u {
        // ── Mode 21: Raymarched Cube ──
        // Procedurally raymarches a rotating 3D box, applying specular lighting and rim lighting.
        let uv_local = (in.uv - 0.5) * 2.0;
        let ro = vec3<f32>(0.0, 0.0, -2.5);
        let rd = normalize(vec3<f32>(uv_local.x, uv_local.y, 1.5));
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
    }

    // Rage effect (applied to all opaque modes)
    let rage = scene.berzerker_rage;
    if rage > 0.05 {
        let noise_coord = in.logical * 0.05 + vec2(scene.time * 0.5);
        let n = fbm(noise_coord);
        let pulse = 0.5 + 0.5 * sin(scene.time * 10.0 * rage);
        let rage_color = mix(theme.ember_core, theme.shatter_neon, pulse * 0.3);
        let original_alpha = color.a;
        color = mix(color, rage_color, n * rage * 0.7);
        color.a = original_alpha;
        if rage > 0.8 {
            color.r *= 1.1;
            color.b *= 0.9;
        }
    }

    if color.a <= 0.0 { discard; }
    return color;
}

/// Apply battle-worn surface damage: scratches, cracks, burn marks.
/// damage_level: [0.0, 1.0] — 0 = pristine, 1 = heavily damaged.
/// damage_seed: per-component random seed for variation.
fn worn_surface(
    uv: vec2<f32>,
    base_color: vec4<f32>,
    damage_level: f32,
    damage_seed: f32,
) -> vec4<f32> {
    var color = base_color;

    // Scratches: high-frequency noise along a directional gradient
    let scratch_dir = normalize(vec2(0.7, 0.3) + vec2(damage_seed * 0.2, damage_seed * 0.15));
    let scratch_uv = vec2(dot(uv, scratch_dir), dot(uv, vec2(-scratch_dir.y, scratch_dir.x)));
    let scratch = fbm(scratch_uv * 80.0 + damage_seed * 10.0);
    let scratch_mask = smoothstep(0.72, 0.78, scratch) * damage_level;

    // Cracks: larger, branching fractures
    let crack_n = fbm(uv * 12.0 + damage_seed * 7.0);
    let crack_mask = smoothstep(0.68, 0.73, crack_n) * damage_level * 0.6;

    // Burn marks: radial dark patches
    let burn_center = vec2(fract(damage_seed * 3.7), fract(damage_seed * 5.3));
    let burn_dist = distance(uv, burn_center);
    let burn_mask = smoothstep(0.3, 0.0, burn_dist) * damage_level * vnoise(uv * 5.0) * 0.7;

    // Apply: scratches lighten (exposed metal), cracks and burns darken
    let new_rgb = color.rgb + vec3<f32>(scratch_mask * 0.25 - crack_mask * 0.4 - burn_mask * 0.5);
    color = vec4<f32>(new_rgb, color.a);

    return color;
}

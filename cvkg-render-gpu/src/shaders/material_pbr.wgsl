//! Material shader — 3D PBR rendering path.
//! Handles modes: 13 (PBR surface), 14 (raymarched reflections), 21 (raymarched cube).
//! Separated from opaque to reduce register pressure from raymarching loops.



fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;

    if in.material_id == 13u {
        // ── Mode 13: 3D Surface — Basic PBR Lighting Model
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
        // ── Mode 14: Ray Marched Reflections
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
        // ── Mode 21: High-Fidelity Raymarched Cube
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

    if color.a <= 0.0 { discard; }
    return color;
}

//! Material shader — 3D PBR rendering path.
//! Handles modes: 13 (PBR surface), 14 (raymarched reflections), 21 (raymarched cube).
//! Separated from opaque to reduce register pressure from raymarching loops.


@group(3) @binding(0) var t_shadow: texture_depth_2d_array;
@group(3) @binding(1) var s_shadow: sampler_comparison;
@group(3) @binding(8) var t_ibl: texture_2d<f32>;
@group(3) @binding(9) var s_ibl: sampler;
@group(3) @binding(6) var t_normal: texture_2d<f32>;
@group(3) @binding(7) var s_normal: sampler;

fn ggx_ndf(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (3.1415926535 * denom * denom);
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let denom = n_dot_v * (1.0 - k) + k;
    return n_dot_v / denom;
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn sample_shadow(cascade_idx: u32, light_vp: mat4x4<f32>, world_pos: vec3<f32>) -> f32 {
    let light_pos = light_vp * vec4<f32>(world_pos, 1.0);
    let light_uv = light_pos.xy / light_pos.w * 0.5 + 0.5;
    let light_depth = light_pos.z / light_pos.w;

    // PCF 3x3
    let texel_size = 1.0 / scene.shadow_map_size;
    var shadow = 0.0;
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            let offset = vec2<f32>(f32(dx), f32(dy)) * texel_size;
            shadow += textureSampleCompare(t_shadow, s_shadow,
                light_uv + offset, cascade_idx, light_depth - scene.shadow_bias);
        }
    }
    return shadow / 9.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;

    if in.material_id == 13u {
        // ── Mode 13: 3D Surface — Basic PBR Lighting Model with Shadows
        let metallic = in.slice.x;
        let roughness = in.slice.y;
        let opacity  = in.slice.z;

        let n = normalize(in.normal);
        let light_dir = normalize(scene.light_direction);
        let light_color = scene.light_color;

        // Shadow mapping (CSM)
        let depth_val = length(in.world_pos_3d - scene.camera_pos);
        var cascade_idx = 3u;
        if (depth_val < csm.cascade_splits.x) {
            cascade_idx = 0u;
        } else if (depth_val < csm.cascade_splits.y) {
            cascade_idx = 1u;
        } else if (depth_val < csm.cascade_splits.z) {
            cascade_idx = 2u;
        }
        let light_vp = csm.cascade_vps[cascade_idx];
        let shadow = sample_shadow(cascade_idx, light_vp, in.world_pos_3d);

        let view_dir = normalize(scene.camera_pos - in.world_pos_3d);
        let half_dir = normalize(light_dir + view_dir);

        let n_dot_v = max(dot(n, view_dir), 0.0001);
        let n_dot_l = max(dot(n, light_dir), 0.0001);
        let n_dot_h = max(dot(n, half_dir), 0.0001);
        let h_dot_v = max(dot(half_dir, view_dir), 0.0);

        // Cook-Torrance Specular BRDF
        let f0 = mix(vec3<f32>(0.04), in.color.rgb, metallic);
        let F = fresnel_schlick(h_dot_v, f0);
        let D = ggx_ndf(n_dot_h, roughness);
        let G = geometry_smith(n_dot_v, n_dot_l, roughness);

        let numerator = D * G * F;
        let denominator = 4.0 * n_dot_v * n_dot_l;
        let specular = numerator / max(denominator, 0.001);

        // Diffuse (Lambert)
        let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        let diffuse = kD * in.color.rgb * n_dot_l * light_color * shadow;

        let spec_term = specular * light_color * n_dot_l * shadow;

        let ambient = scene.ambient_color.rgb * scene.ambient_color.w;
        var lit_color = in.color.rgb * ambient + diffuse + spec_term;
        let fresnel = F; // for IBL lookup compatibility

        if scene.ibl_enabled != 0u {
            let reflect_ws  = reflect(-view_dir, n);
            let reflect_cs  = scene.proj * scene.view * vec4<f32>(in.world_pos + reflect_ws, 1.0);
            let screen_uv   = reflect_cs.xy / reflect_cs.w * 0.5 + 0.5;
            let ibl_mip     = roughness * 4.0;
            let ibl_sample  = textureSampleLevel(t_ibl, s_ibl, screen_uv, ibl_mip);
            lit_color      += ibl_sample.rgb * fresnel * (1.0 - roughness);
        }

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

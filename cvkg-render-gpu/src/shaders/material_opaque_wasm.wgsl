// Material shader -- WASM variant (P1-12 fix)
// WebGL2 does not support binding_array, so t_diffuse is a single
// texture (declared in common_wasm.wgsl). in.tex_index is ignored;
// only index 0 is used in practice.

1|//! Material shader — Opaque/2D rendering path.
2|//! Handles all non-glass material modes: 0 (solid), 1 (neon), 2 (texture),
3|//! 3 (rounded), 4 (ellipse), 6 (text), 8 (glow), 9 (lightning), 10 (rune),
4|//! 12 (heatmap), 13 (PBR surface), 14 (raymarched reflections),
5|//! 15 (animated linear gradient), 16 (radial grad), 17 (stroke),
6|//! 18 (drop shadow), 19 (dashed), 20 (9-slice), 21 (raymarched cube).
7|//! Excludes: 7 (glass) — handled by material_glass.wgsl.
8|
9|
10|
11|@fragment
12|fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
13|    var color = in.color;
14|    let fw = length(vec2(dpdx(in.logical.x), dpdy(in.logical.y)));
15|
16|    // ── High-Fidelity SDF Clipping ───────────────────────────────────────
17|    let p_clip_pos = in.clip.xy * scene.scale_factor;
18|    let p_clip_size = in.clip.zw * scene.scale_factor;
19|    let pixel_pos = in.clip_position.xy;
20|
21|    let clip_d = sd_box(pixel_pos - (p_clip_pos + p_clip_size * 0.5), p_clip_size * 0.5);
22|    var clip_alpha = 1.0 - smoothstep(-1.0, 1.0, clip_d);
23|
24|    if (in.clip.z > 15000.0) { clip_alpha = 1.0; }
25|    color.a *= clip_alpha;
26|
27|    // Geometric Slice (Mjolnir Slice)
28|    if (in.slice.z > 0.5) {
29|        let angle_rad = in.slice.x * 0.01745329251;
30|        let normal_dir = vec2<f32>(cos(angle_rad), sin(angle_rad));
31|        let dist = dot(in.world_pos, normal_dir) - in.slice.y;
32|        if (dist > 0.0) { discard; }
33|    }
34|
35|    // SVG Path/Stroke Tracing Animation
36|    // Works for any non-glass material that has valid path length data (uv.y > 0)
37|    // and a tracing threshold set (slice.w < 0.999).
38|    if (in.uv.y > 0.0 && in.slice.w < 0.999 && in.material_id != 7u) {
39|        if (in.uv.x / max(in.uv.y, 0.0001) > in.slice.w) {
40|            discard;
41|        }
42|    }
43|
44|    if in.material_id == 1u {
45|        // Neon Line
46|        color = in.color * 1.5;
47|    } else if in.material_id == 3u {
48|        let half_size = in.size * 0.5;
49|        let squircle_n = select(0.0, in.slice.y, in.slice.y > 1.5);
50|        var d: f32;
51|        if (squircle_n > 1.5) {
52|            d = sd_squircle(in.logical - half_size, half_size, squircle_n);
53|        } else {
54|            d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
55|        }
56|        let aa = fwidth(d);
57|        color.a *= 1.0 - smoothstep(0.0, aa, d);
58|    } else if in.material_id == 4u {
59|        let half_size = in.size * 0.5;
60|        let safe_half = max(half_size, vec2<f32>(0.001));
61|        let d = length((in.logical - half_size) / safe_half) - 1.0;
62|        let aa = fwidth(d);
63|        color.a *= 1.0 - smoothstep(0.0, aa, d);
64|    } else if in.material_id == 8u {
65|        // Neon Glow (Gungnir)
66|        let center = in.size * 0.5;
67|        let dist = length(in.logical - center) / max(in.size.x, in.size.y);
68|        let glow = exp(-dist * 4.0) * 1.5;
69|        color = vec4<f32>(color.rgb * glow, color.a);
70|    } else if in.material_id == 9u {
71|        let d = length((in.uv - 0.5) * vec2<f32>(1.0, 4.0));
72|        color = theme.primary_neon * neon_glow(d, 0.01, 0.2);
73|    } else if in.material_id == 10u {
74|        let p = (in.uv - 0.5) * 2.0;
75|        let d = min(sd_segment(p, vec2(-0.5, -0.8), vec2(0.5, 0.8)), sd_segment(p, vec2(0.5, -0.8), vec2(-0.5, 0.8)));
76|        color = theme.rune_glow * neon_glow(d, 0.02, 0.15) * theme.rune_opacity;
77|    } else if in.material_id == 16u {
78|        // Radial Gradient Logic
79|        let dist = length(in.uv - 0.5) * 2.0;
80|        let t = clamp(dist, 0.0, 1.0);
81|        let end_color = vec4<f32>(in.slice.rgb, in.slice.a);
82|        color = mix(in.color, end_color, t);
83|    } else if in.material_id == 17u {
84|        let half_size = in.size * 0.5;
85|        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
86|        let thickness = max(in.slice.x, 1.0);
87|        color.a *= (1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5));
88|    } else if in.material_id == 19u {
89|        let half_size = in.size * 0.5;
90|        let d = sd_round_rect(in.logical - half_size, half_size - in.radius, in.radius);
91|        let thickness = max(in.slice.x, 1.0);
92|        let perimeter = (in.uv.x + in.uv.y) * max(in.size.x, in.size.y);
93|        var alpha = 1.0 - smoothstep(-fw, fw, abs(d + thickness * 0.5) - thickness * 0.5);
94|        if (perimeter + scene.time * 20.0) % (max(in.slice.y, 1.0) + max(in.slice.z, 1.0)) > max(in.slice.y, 1.0) { alpha = 0.0; }
95|        color.a *= alpha;
96|    } else if in.material_id == 2u || in.material_id == 6u {
97|        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
98|        if in.material_id == 6u {
99|            // Apply subpixel typography (LCD horizontal masking) by blending each subpixel component independently.
100|            // CONTRACT: If the texture contains a subpixel coverage mask (RenderMode::Subpixel), we blend color
101|            // channels using the mask's RGB components. For grayscale, tex_color.rgb is vec3(1.0) and tex_color.a is alpha.
102|            color = vec4<f32>(in.color.rgb * tex_color.rgb, in.color.a * tex_color.a);
103|        } else {
104|            color *= tex_color;
105|        }
106|    } else if in.material_id == 12u {
107|        let val = textureSample(t_diffuse, s_diffuse, in.uv).r;
108|        color = vec4<f32>(heatmap_palette(val), in.color.a);
109|    } else if in.material_id == 20u {
110|        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
111|        color *= tex_color;
112|    } else if in.material_id == 15u {
113|        // ── Mode 15: Animated Linear Gradient ──
114|        // Rotates a linear gradient across the element based on elapsed time to create dynamic flow.
115|        let angle = in.uv.x + scene.time * 0.5;
116|        let t = dot(in.logical / in.size - 0.5, vec2(cos(angle), sin(angle))) + 0.5;
117|        let end_color = vec4<f32>(in.slice.rgb, in.color.a);
118|        color = mix(in.color, end_color, clamp(t, 0.0, 1.0));
119|    } else if in.material_id == 18u {
120|        // ── Mode 18: Drop Shadow ──
121|        // Renders a soft drop shadow outside the margins of the rounded rectangle using smoothstep of the SDF.
122|        let margin = in.uv.x;
123|        let blur = max(in.uv.y, 1.0);
124|        let original_size = in.size - 2.0 * margin;
125|        let half_size = original_size * 0.5;
126|        let p = in.logical - margin - half_size;
127|        let d = sd_round_rect(p, half_size - in.radius, in.radius);
128|        color.a *= smoothstep(blur, 0.0, d);
129|    } else if in.material_id == 13u {
130|        // ── Mode 13: 3D Surface — Basic PBR Lighting ──
131|        // Simulates realistic lighting on a 3D surface mesh using diffuse, specular, fresnel reflection, and fog depth cues.
132|        let metallic = in.slice.x;
133|        let roughness = in.slice.y;
134|        let opacity  = in.slice.z;
135|        let n = normalize(in.normal);
136|        let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.6));
137|        let light_color = vec3<f32>(1.0, 0.95, 0.9);
138|        let n_dot_l = max(dot(n, light_dir), 0.0);
139|        let diffuse = n_dot_l * light_color;
140|        let view_dir = vec3<f32>(0.0, 0.0, 1.0);
141|        let half_dir = normalize(light_dir + view_dir);
142|        let n_dot_h = max(dot(n, half_dir), 0.0);
143|        let shininess = mix(8.0, 256.0, 1.0 - roughness);
144|        let spec = pow(n_dot_h, shininess) * light_color;
145|        let f0 = mix(vec3<f32>(0.04), in.color.rgb, metallic);
146|        let fresnel = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - max(dot(n, -view_dir), 0.0), 5.0);
147|        let ambient = vec3<f32>(0.06, 0.07, 0.1);
148|        var lit_color = in.color.rgb * (ambient + diffuse);
149|        lit_color += spec * mix(vec3<f32>(1.0), in.color.rgb, metallic) * fresnel;
150|        let depth = in.clip_position.z;
151|        let fog_factor = clamp(1.0 - depth * 0.0005, 0.7, 1.0);
152|        lit_color *= fog_factor;
153|        color = vec4<f32>(lit_color, in.color.a * opacity);
154|    } else if in.material_id == 14u {
155|        // ── Mode 14: Raymarched Reflections ──
156|        // Renders reflections by marching a ray through a procedural 3D scene and computing lighting/reflection vectors.
157|        let ro = vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, -2.0);
158|        let rd = normalize(vec3<f32>(in.uv.x - 0.5, in.uv.y - 0.5, 1.0));
159|        let t = ray_march(ro, rd);
160|        if t > 0.0 {
161|            let p = ro + rd * t;
162|            let n = calc_normal(p);
163|            let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
164|            let diff = max(dot(n, light_dir), 0.2);
165|            let ref_rd = reflect(rd, n);
166|            let ref_t = ray_march(p + n * 0.01, ref_rd);
167|            var reflection_color = vec3<f32>(0.05, 0.05, 0.1);
168|            if ref_t > 0.0 { reflection_color = mix(theme.primary_neon.rgb, theme.shatter_neon.rgb, 0.5); }
169|            color = vec4<f32>(mix(in.color.rgb * diff, reflection_color, 0.3), 1.0);
170|        } else { discard; }
171|    } else if in.material_id == 21u {
172|        // ── Mode 21: Raymarched Cube ──
173|        // Procedurally raymarches a rotating 3D box, applying specular lighting and rim lighting.
174|        let uv_local = (in.uv - 0.5) * 2.0;
175|        let ro = vec3<f32>(0.0, 0.0, -2.5);
176|        let rd = normalize(vec3<f32>(uv_local.x, uv_local.y, 1.5));
177|        let m = rotX(in.slice.x) * rotY(in.slice.y) * rotZ(in.slice.z);
178|        var t = 0.0;
179|        var hit = false;
180|        var d = 0.0;
181|        for (var i = 0; i < 40; i++) {
182|            let p = m * (ro + rd * t);
183|            d = sd_box_3d(p, vec3(0.5, 0.5, 0.5));
184|            if d < 0.001 { hit = true; break; }
185|            t += d;
186|            if t > 5.0 { break; }
187|        }
188|        if hit {
189|            let p = m * (ro + rd * t);
190|            let eps = vec2(0.001, 0.0);
191|            let n = normalize(vec3(
192|                sd_box_3d(p + eps.xyy, vec3(0.5)) - sd_box_3d(p - eps.xyy, vec3(0.5)),
193|                sd_box_3d(p + eps.yxy, vec3(0.5)) - sd_box_3d(p - eps.yxy, vec3(0.5)),
194|                sd_box_3d(p + eps.yyx, vec3(0.5)) - sd_box_3d(p - eps.yyx, vec3(0.5))
195|            ));
196|            let light_dir = normalize(vec3(1.0, 1.0, -2.0));
197|            let diff = max(dot(n, light_dir), 0.1);
198|            let rim = pow(1.0 - max(dot(n, -rd), 0.0), 3.0) * 0.5;
199|            color = vec4<f32>(in.color.rgb * diff + rim, in.color.a);
200|        } else {
201|            discard;
202|        }
203|    }
204|
205|    // Rage effect (applied to all opaque modes)
206|    let rage = scene.berzerker_rage;
207|    if rage > 0.05 {
208|        let noise_coord = in.logical * 0.05 + vec2(scene.time * 0.5);
209|        let n = fbm(noise_coord);
210|        let pulse = 0.5 + 0.5 * sin(scene.time * 10.0 * rage);
211|        let rage_color = mix(theme.ember_core, theme.shatter_neon, pulse * 0.3);
212|        let original_alpha = color.a;
213|        color = mix(color, rage_color, n * rage * 0.7);
214|        color.a = original_alpha;
215|        if rage > 0.8 {
216|            color.r *= 1.1;
217|            color.b *= 0.9;
218|        }
219|    }
220|
221|    if color.a <= 0.0 { discard; }
222|    return color;
223|}
224|
225|/// Apply battle-worn surface damage: scratches, cracks, burn marks.
226|/// damage_level: [0.0, 1.0] — 0 = pristine, 1 = heavily damaged.
227|/// damage_seed: per-component random seed for variation.
228|fn worn_surface(
229|    uv: vec2<f32>,
230|    base_color: vec4<f32>,
231|    damage_level: f32,
232|    damage_seed: f32,
233|) -> vec4<f32> {
234|    var color = base_color;
235|
236|    // Scratches: high-frequency noise along a directional gradient
237|    let scratch_dir = normalize(vec2(0.7, 0.3) + vec2(damage_seed * 0.2, damage_seed * 0.15));
238|    let scratch_uv = vec2(dot(uv, scratch_dir), dot(uv, vec2(-scratch_dir.y, scratch_dir.x)));
239|    let scratch = fbm(scratch_uv * 80.0 + damage_seed * 10.0);
240|    let scratch_mask = smoothstep(0.72, 0.78, scratch) * damage_level;
241|
242|    // Cracks: larger, branching fractures
243|    let crack_n = fbm(uv * 12.0 + damage_seed * 7.0);
244|    let crack_mask = smoothstep(0.68, 0.73, crack_n) * damage_level * 0.6;
245|
246|    // Burn marks: radial dark patches
247|    let burn_center = vec2(fract(damage_seed * 3.7), fract(damage_seed * 5.3));
248|    let burn_dist = distance(uv, burn_center);
249|    let burn_mask = smoothstep(0.3, 0.0, burn_dist) * damage_level * vnoise(uv * 5.0) * 0.7;
250|
251|    // Apply: scratches lighten (exposed metal), cracks and burns darken
252|    let new_rgb = color.rgb + vec3<f32>(scratch_mask * 0.25 - crack_mask * 0.4 - burn_mask * 0.5);
253|    color = vec4<f32>(new_rgb, color.a);
254|
255|    return color;
256|}
257|
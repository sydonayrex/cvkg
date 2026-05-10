// =============================================================================
// CYBERPUNK VIKING BERZERKER SHADER
// Obsidian Glassmorphism · Neon Cyan Illumination · Magenta Shatter Physics
// =============================================================================

struct ColorTheme {
    primary_neon:    vec4<f32>,
    shatter_neon:    vec4<f32>,
    glass_base:      vec4<f32>,
    glass_edge:      vec4<f32>,
    rune_glow:       vec4<f32>,
    ember_core:      vec4<f32>,
    background_deep: vec4<f32>,
    glass_blur_strength:  f32,
    shatter_edge_width:   f32,
    neon_bloom_radius:    f32,
    rune_opacity:         f32,
    _pad0: f32, _pad1: f32, _pad2: f32, _pad3: f32,
};

struct SceneUniforms {
    view:            mat4x4<f32>,
    proj:            mat4x4<f32>,
    time:            f32,
    delta_time:      f32,
    resolution:      vec2<f32>,
    mouse:           vec2<f32>,
    mouse_velocity:  vec2<f32>,
    shatter_origin:  vec2<f32>,
    shatter_time:    f32,
    shatter_force:   f32,
    berzerker_rage:  f32,
    scroll_offset:   f32,
    scale_factor:    f32,
    scene_type:      u32,
    _pad0:           f32,
    _pad1:           f32,
    _pad2:           f32,
};

// --- Group 2: Berserker Uniforms ---
@group(2) @binding(0) var<uniform> theme: ColorTheme;
@group(2) @binding(1) var<uniform> scene: SceneUniforms;

// --- Group 0: Main Texture Array ---
@group(0) @binding(0) var t_diffuse: binding_array<texture_2d<f32>, 256>;
@group(0) @binding(1) var s_diffuse: sampler;

// --- Group 1: Environment / Blur ---
@group(1) @binding(0) var t_env: texture_2d<f32>;
@group(1) @binding(1) var s_env: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
    @location(4) mode:     u32,
    @location(5) radius:   f32,
    @location(6) slice:    vec4<f32>,
    @location(7) logical:  vec2<f32>,
    @location(8) size:     vec2<f32>,
    @location(9) screen:   vec2<f32>,
    @location(10) clip:    vec4<f32>,
    @location(11) translation: vec2<f32>,
    @location(12) scale:       vec2<f32>,
    @location(13) rotation:    f32,
    @location(14) tex_index:   u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv:          vec2<f32>,
    @location(1) color:       vec4<f32>,
    @location(2) @interpolate(flat) mode: u32,
    @location(3) radius:      f32,
    @location(4) slice:       vec4<f32>,
    @location(5) logical:     vec2<f32>,
    @location(6) size:        vec2<f32>,
    @location(7) screen:      vec2<f32>,
    @location(8) normal:      vec3<f32>,
    @location(9) clip:        vec4<f32>,
    @location(10) @interpolate(flat) tex_index: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply 2D Transform: Rotate -> Scale -> Translate
    var pos = in.position.xy;
    
    // Rotation
    let s = sin(in.rotation);
    let c = cos(in.rotation);
    let rot_matrix = mat2x2<f32>(c, s, -s, c);
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

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) % 2) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 1.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    // Fullscreen passes don't use per-vertex data; supply safe defaults so
    // fs_main's clip SDF and mode branches don't discard/misrender fragments.
    out.color  = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.mode   = 0u;
    out.radius = 0.0;
    out.slice  = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.logical = vec2<f32>(0.0, 0.0);
    out.tex_index = 0u;
    // Large sentinel so sd_box clip test always passes (clip_alpha → 1).
    out.clip   = vec4<f32>(-10000.0, -10000.0, 20000.0, 20000.0);
    out.size   = vec2<f32>(scene.resolution.x, scene.resolution.y);
    out.screen = scene.resolution * scene.scale_factor;
    out.normal = vec3<f32>(0.0, 0.0, 1.0);
    return out;
}


// --- SDF MATH ---
fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sd_round_rect(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - r;
}

fn sd_triangle(p: vec2<f32>, r: f32) -> f32 {
    let k = sqrt(3.0);
    var pp = p;
    pp.x = abs(pp.x) - r;
    pp.y = pp.y + r / k;
    if (pp.x + k * pp.y > 0.0) {
        pp = vec2<f32>(pp.x - k * pp.y, -k * pp.x - pp.y) / 2.0;
    }
    pp.x -= clamp(pp.x, -2.0 * r, 0.0);
    return -length(pp) * sign(pp.y);
}

fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h  = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn rot2(angle: f32) -> mat2x2<f32> {
    let s = sin(angle); let c = cos(angle);
    return mat2x2<f32>(c, s, -s, c);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p); let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x), mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0; var amp = 0.5; var freq = 1.0; var pp = p;
    for (var i = 0; i < 5; i++) {
        val += amp * vnoise(pp * freq);
        freq *= 2.1; amp *= 0.5; pp = pp * rot2(0.37);
    }
    return val;
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn neon_glow(dist: f32, width: f32, bloom: f32) -> f32 {
    let core  = smoothstep(width, 0.0, dist);
    let glow  = exp(-dist * dist / (bloom * bloom));
    return core + glow * 0.6;
}

fn heatmap_palette(t: f32) -> vec3<f32> {
    let low = vec3<f32>(0.0, 0.05, 0.2);
    let mid = theme.primary_neon.rgb;
    let high = theme.shatter_neon.rgb;
    return mix(mix(low, mid, smoothstep(0.0, 0.5, t)), high, smoothstep(0.5, 1.0, t));
}

// ── 3D Rotation Helpers ──────────────────────────────────────────────────
fn rotX(a: f32) -> mat3x3<f32> {
    let s = sin(a); let c = cos(a);
    return mat3x3<f32>(vec3(1.0, 0.0, 0.0), vec3(0.0, c, s), vec3(0.0, -s, c));
}
fn rotY(a: f32) -> mat3x3<f32> {
    let s = sin(a); let c = cos(a);
    return mat3x3<f32>(vec3(c, 0.0, -s), vec3(0.0, 1.0, 0.0), vec3(s, 0.0, c));
}
fn rotZ(a: f32) -> mat3x3<f32> {
    let s = sin(a); let c = cos(a);
    return mat3x3<f32>(vec3(c, s, 0.0), vec3(-s, c, 0.0), vec3(0.0, 0.0, 1.0));
}

// ── 3D SDF Library ──────────────────────────────────────────────────────────
fn sd_sphere(p: vec3<f32>, s: f32) -> f32 { return length(p) - s; }
fn sd_box_3d(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
fn scene_sdf(p: vec3<f32>) -> f32 {
    let s1 = sd_sphere(p - vec3<f32>(0.0, 0.2 * sin(scene.time), 0.0), 0.4);
    let b1 = sd_box_3d(p - vec3<f32>(0.6, 0.0, 0.0), vec3<f32>(0.2, 0.2, 0.2));
    return smin(s1, b1, 0.1);
}
fn ray_march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 64; i++) {
        let d = scene_sdf(ro + rd * t);
        if d < 0.001 { return t; }
        if t > 20.0  { break; }
        t += d;
    }
    return -1.0;
}
fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(scene_sdf(p + e.xyy) - scene_sdf(p - e.xyy), scene_sdf(p + e.yxy) - scene_sdf(p - e.yxy), scene_sdf(p + e.yyx) - scene_sdf(p - e.yyx)));
}

// --- FRAGMENT STAGE ---
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
        // 1. Screen-Space UV & Hard Clipping
        let uv = in.uv; 
        if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            discard;
        }
        
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
        let env_base = textureSampleLevel(t_env, s_env, uv, 0.0).rgb;
        let brightness = dot(env_base, vec3<f32>(0.299, 0.587, 0.114));
        
        // Final Refraction Offset
        var distortion = lens * 1.2;
        distortion += stress_offset * 0.6;
        distortion += hash_noise * 0.3;

        distortion *= (1.0 + brightness * 0.7);
        distortion *= 2.0;
        
        // 7. Chromatic Aberration (High Impact RGB Split)
        let ab_offset = distortion * 0.04;
        let r_sample = textureSampleLevel(t_env, s_env, uv + distortion + ab_offset * 1.2, 0.0).r;
        let g_sample = textureSampleLevel(t_env, s_env, uv + distortion, 0.0).g;
        let b_sample = textureSampleLevel(t_env, s_env, uv + distortion - ab_offset * 1.2, 0.0).b;
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
        // FIX: Use in.slice.a (outer alpha) instead of forcing in.color.a. 
        // This allows radial gradients to fade to transparency at the corners.
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

@fragment
fn fs_background(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Screen-Space UV (Continuous across the whole field)
    let uv = in.uv;
    let time = scene.time;
    
    // 2. Global Center-Based Gradient (No more slabs)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);
    
    var base = theme.background_deep;
    
    if scene.scene_type == 0u {
        // --- AURORA BOREALIS ---
        let band = sin(uv.y * 15.0 + time * 0.4) * 0.03;
        let n1 = fbm(uv * 2.5 + time * 0.15);
        let n2 = vnoise(uv * 10.0 + time * 0.1) * 0.02;
        let glow_field = dist + band + n1 * 0.08 + n2;
        base = mix(theme.background_deep, theme.primary_neon * 0.4, clamp(1.0 - glow_field, 0.0, 1.0));
        base *= (0.96 + sin(time * 0.8) * 0.04);
        
    } else if scene.scene_type == 1u {
        // --- VOID (Minimalist) ---
        base = mix(theme.background_deep, vec4<f32>(0.02, 0.02, 0.03, 1.0), dist);
        let stars = hash21(uv * 500.0);
        if stars > 0.998 {
            base += vec4<f32>(1.0, 1.0, 1.0, 0.8) * (0.5 + 0.5 * sin(time * 2.0 + stars * 100.0));
        }
        
    } else if scene.scene_type == 2u {
        // --- NEBULA ---
        let n1 = fbm(uv * 1.5 + time * 0.05);
        let n2 = fbm(uv * 4.0 - time * 0.03);
        let nebula = mix(theme.primary_neon, theme.shatter_neon, n1);
        base = mix(theme.background_deep, nebula * 0.5, n2 * n1);
        
    } else if scene.scene_type == 3u {
        // --- GLITCH ---
        var guv = uv;
        let glitch = hash21(vec2(floor(time * 10.0), floor(uv.y * 40.0)));
        if glitch > 0.95 {
            guv.x += (glitch - 0.95) * 0.2;
        }
        base = mix(theme.background_deep, theme.shatter_neon * 0.3, fbm(guv * 10.0 + time));
        if glitch > 0.98 {
            base += vec4<f32>(0.0, 1.0, 1.0, 0.2);
        }
        
    } else if scene.scene_type == 4u {
        // --- YGGDRASIL (Tree of Life) ---
        let n = fbm(uv * 2.0 + vec2(0.0, time * 0.1));
        let root_glow = 1.0 - smoothstep(0.0, 0.8, abs(uv.x - 0.5 + 0.1 * sin(uv.y * 4.0 + time)));
        base = mix(theme.background_deep, vec4<f32>(0.0, 0.8, 0.4, 1.0) * 0.3, root_glow * n);
    }
    
    // 6. Global Vignette
    let vignette = 1.0 - dist * 0.85;
    return vec4<f32>(base.rgb * vignette, 1.0);
}

@fragment
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.8 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    // High-Fidelity 9-tap Gaussian Blur
    let weight = array<f32, 9>(0.153423, 0.143254, 0.117031, 0.081827, 0.049003, 0.025135, 0.010861, 0.00392, 0.0011);
    let tex_offset = 6.0 / scene.resolution.x;
    
    // Explicitly sample from index 0 of the texture array for post-process passes
    result += textureSample(t_diffuse[0], s_diffuse, in.uv).rgb * weight[0];
    for (var i = 1; i < 9; i++) {
        result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * f32(i), 0.0)).rgb * weight[i];
        result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * f32(i), 0.0)).rgb * weight[i];
    }
    return vec4<f32>(result, 1.0);
}

@fragment
fn fs_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    // High-Fidelity 9-tap Gaussian Blur
    let weight = array<f32, 9>(0.153423, 0.143254, 0.117031, 0.081827, 0.049003, 0.025135, 0.010861, 0.00392, 0.0011);
    let tex_offset = 6.0 / scene.resolution.y;
    
    // Explicitly sample from index 0 of the texture array for post-process passes
    result += textureSample(t_diffuse[0], s_diffuse, in.uv).rgb * weight[0];
    for (var i = 1; i < 9; i++) {
        result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * f32(i))).rgb * weight[i];
        result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * f32(i))).rgb * weight[i];
    }
    return vec4<f32>(result, 1.0);
}

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv_screen = in.uv;
    let scene_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let bloom_color = textureSample(t_env, s_env, in.uv);
    
    // Berserker Glow Instability (Temporal + Spatial Pulse)
    let flicker = 0.92 + sin(scene.time * 6.0 + uv_screen.x * 10.0) * 0.08;
    
    // HDR Bloom Fusion
    let hdr_color = scene_color.rgb + (bloom_color.rgb * 1.5 * flicker);
    
    // ACES Filmic Tonemapping (Asgard Quality)
    let ldr_color = aces_tonemap(hdr_color);
    
    return vec4<f32>(ldr_color, scene_color.a);
}

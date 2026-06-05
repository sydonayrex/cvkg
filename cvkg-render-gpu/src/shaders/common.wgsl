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
    @location(4) material_id: u32,
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
    @location(2) @interpolate(flat) material_id: u32,
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
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index) / 2) * 4.0 - 1.0;
    let y = f32(i32(vertex_index) % 2) * 4.0 - 1.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    out.color  = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.material_id = 0u;
    out.radius = 0.0;
    out.slice  = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.logical = vec2<f32>(0.0, 0.0);
    out.tex_index = 0u;
    out.clip   = vec4<f32>(-10000.0, -10000.0, 20000.0, 20000.0);
    out.size   = vec2<f32>(scene.resolution.x, scene.resolution.y);
    out.screen = scene.resolution * scene.scale_factor;
    out.normal = vec3<f32>(0.0, 0.0, 1.0);
    return out;
}

/// Main vertex shader — transforms 2D quads with rotation/scale/translation.
/// Used by all pipelines (main + specialized).
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = in.position.xy;

    // ── Material 13 (3D Surface): Skip 2D transforms, use full MVP ──────────
    if (in.material_id == 13u) {
        out.clip_position = scene.proj * scene.view * vec4<f32>(in.position, 1.0);
    } else {
        // Apply 2D Transform: Rotate -> Scale -> Translate
        let s2 = sin(in.rotation);
        let c2 = cos(in.rotation);
        let rot_matrix = mat2x2<f32>(c2, s2, -s2, c2);
        pos = rot_matrix * pos;
        pos = pos * in.scale;
        pos = pos + in.translation;

        // ── Hardware Shatter Effect (Berzerker Physics) ─────────────────────
        let shatter_dt = scene.time - scene.shatter_time;
        if (shatter_dt > 0.0 && shatter_dt < 2.0) {
            let dist = distance(pos, scene.shatter_origin);
            let dir = normalize(pos - scene.shatter_origin + vec2<f32>(1e-5, 1e-5));
            let explosion = (1.0 / (dist * 0.01 + 0.1)) * scene.shatter_force;
            let expansion = explosion * shatter_dt * 100.0;
            pos += dir * expansion;
        }

        out.clip_position = scene.proj * scene.view * vec4<f32>(pos, in.position.z, 1.0);
    }
    out.uv = in.uv;
    out.color = in.color;
    out.material_id = in.material_id;
    out.radius = in.radius;
    out.slice = in.slice;
    out.logical = in.logical;
    out.size = in.size;
    out.screen = in.screen;
    out.normal = in.normal;
    out.clip = in.clip;
    out.tex_index = in.tex_index;

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

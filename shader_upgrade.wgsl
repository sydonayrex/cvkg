// =============================================================================
// CYBERPUNK VIKING BERZERKER SHADER
// Obsidian Glassmorphism · Neon Cyan Illumination · Magenta Shatter Physics
// -----------------------------------------------------------------------------
// SwiftUI-inspired architecture: declarative uniform structs, modular fn blocks,
// composable visual layers, and a single themeable ColorTheme binding.
// Compatible with wgpu / Bevy / any WebGPU-based Rust renderer.
// =============================================================================


// -----------------------------------------------------------------------------
// SECTION 1 — THEME UNIFORMS
// Bind group 0, binding 0: update at runtime to retheme everything.
// SwiftUI analogy: @Environment(\.colorScheme) / @Binding var theme: Theme
// -----------------------------------------------------------------------------
struct ColorTheme {
    // Core palette — vec4<f32> = (R, G, B, intensity_multiplier)
    primary_neon:    vec4<f32>,  // default: cyan    (0.0, 1.0, 0.95, 1.2)
    shatter_neon:    vec4<f32>,  // default: magenta (1.0, 0.0, 0.75, 1.5)
    glass_base:      vec4<f32>,  // default: obsidian black (0.04, 0.04, 0.06, 0.82)
    glass_edge:      vec4<f32>,  // default: dark cyan rim  (0.0, 0.45, 0.55, 0.6)
    rune_glow:       vec4<f32>,  // default: pale ice  (0.75, 0.98, 1.0, 0.9)
    ember_core:      vec4<f32>,  // default: berzerker blood (0.95, 0.12, 0.12, 1.0)
    background_deep: vec4<f32>,  // default: void black (0.01, 0.01, 0.03, 1.0)
    // Scalar controls
    glass_blur_strength:  f32,   // 0.0–1.0, default 0.6
    shatter_edge_width:   f32,   // pixels, default 1.8
    neon_bloom_radius:    f32,   // 0.0–0.05, default 0.022
    rune_opacity:         f32,   // 0.0–1.0, default 0.55
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> theme: ColorTheme;


// -----------------------------------------------------------------------------
// SECTION 2 — SCENE UNIFORMS
// Bind group 0, binding 1: time, resolution, interaction inputs.
// SwiftUI analogy: @State / @Published scene state driving view updates.
// -----------------------------------------------------------------------------
struct SceneUniforms {
    time:            f32,   // seconds elapsed
    delta_time:      f32,   // frame delta for physics
    resolution:      vec2<f32>,
    mouse:           vec2<f32>,   // normalized 0..1
    mouse_velocity:  vec2<f32>,   // for shatter impulse
    shatter_origin:  vec2<f32>,   // epicenter of last shatter event
    shatter_time:    f32,         // timestamp of last shatter (for animation)
    shatter_force:   f32,         // 0.0–1.0 impact magnitude
    berzerker_rage:  f32,         // 0.0–1.0 global intensity modifier
    scroll_offset:   f32,         // vertical parallax
    _pad0:           f32,
};

@group(0) @binding(1)
var<uniform> scene: SceneUniforms;


// -----------------------------------------------------------------------------
// SECTION 3 — VERTEX STAGE
// Fullscreen quad via vertex_index trick — zero vertex buffer needed.
// Passes UV, world position, and screen position to fragment stage.
// SwiftUI analogy: GeometryReader providing coordinate space to children.
// -----------------------------------------------------------------------------
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv:          vec2<f32>,   // 0..1 screen UV
    @location(1) world_pos:   vec2<f32>,   // centered -1..1
    @location(2) screen_pos:  vec2<f32>,   // pixel coords
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    // Two-triangle fullscreen quad from 6 indices (or 3 for single-triangle)
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    let pos   = positions[vid];
    var out:  VertexOutput;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.uv            = pos * 0.5 + 0.5;
    out.world_pos     = pos;
    out.screen_pos    = (pos * 0.5 + 0.5) * scene.resolution;
    return out;
}


// =============================================================================
// SECTION 4 — MATH & UTILITY LIBRARY
// Pure functions — no side effects, fully composable.
// SwiftUI analogy: ViewModifier / extension on View
// =============================================================================

// --- 2D rotation matrix
fn rot2(angle: f32) -> mat2x2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return mat2x2<f32>(c, -s, s, c);
}

// --- Hash / noise primitives (GPU-friendly, no texture dependency)
fn hash11(p: f32) -> f32 {
    var x = fract(p * 0.1031);
    x *= x + 33.33;
    x *= x + x;
    return fract(x);
}

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

// --- Value noise 2D (smooth)
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// --- Fractal Brownian Motion (fBm) — 5 octaves
fn fbm(p: vec2<f32>) -> f32 {
    var val  = 0.0;
    var amp  = 0.5;
    var freq = 1.0;
    var pp   = p;
    for (var i = 0; i < 5; i++) {
        val += amp * vnoise(pp * freq);
        freq *= 2.1;
        amp  *= 0.5;
        pp = pp * rot2(0.37);
    }
    return val;
}

// --- Signed distance: line segment (for rune strokes)
fn sd_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h  = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// --- Signed distance: box
fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

// --- Signed distance: rounded triangle (for shatter shards)
fn sd_triangle(p: vec2<f32>, r: f32) -> f32 {
    let k = sqrt(3.0);
    var pp = p;
    pp.x = abs(pp.x) - 1.0;
    pp.y = pp.y + 1.0 / k;
    if pp.x + k * pp.y > 0.0 { pp = vec2<f32>(pp.x - k * pp.y, -k * pp.x - pp.y) / 2.0; }
    pp.x -= clamp(pp.x, -2.0, 0.0);
    return -length(pp) * sign(pp.y) - r;
}

// --- Smooth minimum (organic blending)
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// --- Neon glow falloff: sharp core + soft bloom
fn neon_glow(dist: f32, width: f32, bloom: f32) -> f32 {
    let core  = smoothstep(width, 0.0, dist);
    let glow  = exp(-dist * dist / (bloom * bloom));
    return core + glow * 0.6;
}

// --- Chromatic aberration offset
fn chrom_aberr(uv: vec2<f32>, offset: f32) -> vec2<f32> {
    return uv + normalize(uv - vec2<f32>(0.5)) * offset;
}


// =============================================================================
// SECTION 5 — LAYER: OBSIDIAN GLASSMORPHISM BACKGROUND
// Layered dark glass panels with depth, frost, and iridescent rim lighting.
// SwiftUI analogy: .background(.ultraThinMaterial) with custom stroke
// =============================================================================
fn layer_glass_bg(uv: vec2<f32>) -> vec4<f32> {
    let t   = scene.time;
    let asp = scene.resolution.x / scene.resolution.y;

    // Drifting glass panel coordinates with subtle parallax
    let scroll = scene.scroll_offset * 0.15;
    var p = uv * vec2<f32>(asp, 1.0) + vec2<f32>(0.0, scroll);

    // Multi-layer fBm for frosted glass depth
    let n1  = fbm(p * 2.3 + vec2<f32>(t * 0.04, t * 0.02));
    let n2  = fbm(p * 4.1 - vec2<f32>(t * 0.03, t * 0.05));
    let fog = mix(n1, n2, 0.4) * theme.glass_blur_strength;

    // Glass base color: near-black with microscopic blue-teal tint
    let glass_col = theme.glass_base.rgb + fog * 0.035;

    // Panel edge vignette — reinforces the "floating panel" feel
    let panel_uv  = uv * 2.0 - 1.0;
    let edge_dist = sd_box(panel_uv, vec2<f32>(0.72, 0.82));
    let rim       = neon_glow(-edge_dist, 0.003, theme.neon_bloom_radius);
    let rim_col   = theme.glass_edge.rgb * theme.glass_edge.a * rim;

    // Iridescent sheen: angle-dependent color shift
    let view_angle = dot(normalize(panel_uv), vec2<f32>(0.707, 0.707));
    let sheen      = sin(view_angle * 12.0 + t * 0.5) * 0.5 + 0.5;
    let irid_col   = mix(theme.primary_neon.rgb, theme.shatter_neon.rgb, sheen) * 0.04;

    return vec4<f32>(glass_col + rim_col + irid_col, theme.glass_base.a);
}


// =============================================================================
// SECTION 6 — LAYER: PROCEDURAL ELDER FUTHARK RUNE GRID
// Draws glowing Norse rune strokes via SDF line segments.
// Animated: runes pulse, flicker, and "charge up" with berzerker rage.
// SwiftUI analogy: Canvas { context in context.stroke(...) } with .animation
// =============================================================================

// Draw a single rune stroke between two points — returns glow intensity
fn rune_stroke(uv: vec2<f32>, a: vec2<f32>, b: vec2<f32>, glyph_scale: f32) -> f32 {
    let d = sd_segment(uv, a * glyph_scale, b * glyph_scale);
    return neon_glow(d, 0.003 * glyph_scale, 0.012 * glyph_scale);
}

// Algiz (ᛉ) — protection rune, 5 strokes
fn rune_algiz(p: vec2<f32>, s: f32) -> f32 {
    var g = 0.0;
    g += rune_stroke(p, vec2<f32>(0.0, -1.0), vec2<f32>(0.0,  1.0), s);
    g += rune_stroke(p, vec2<f32>(0.0,  0.0), vec2<f32>(-0.5, -0.5), s);
    g += rune_stroke(p, vec2<f32>(0.0,  0.0), vec2<f32>( 0.5, -0.5), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.5), vec2<f32>(-0.35,-0.85), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.5), vec2<f32>( 0.35,-0.85), s);
    return g;
}

// Tiwaz (ᛏ) — warrior / victory rune, 3 strokes
fn rune_tiwaz(p: vec2<f32>, s: f32) -> f32 {
    var g = 0.0;
    g += rune_stroke(p, vec2<f32>(0.0, -1.0), vec2<f32>( 0.0,  1.0), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.6), vec2<f32>(-0.45,-0.1), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.6), vec2<f32>( 0.45,-0.1), s);
    return g;
}

// Isa (ᛁ) — ice/stillness rune, 1 stroke
fn rune_isa(p: vec2<f32>, s: f32) -> f32 {
    return rune_stroke(p, vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 1.0), s);
}

// Fehu (ᚠ) — power/cattle rune, 3 strokes
fn rune_fehu(p: vec2<f32>, s: f32) -> f32 {
    var g = 0.0;
    g += rune_stroke(p, vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 1.0), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.2), vec2<f32>(0.5, 0.1), s);
    g += rune_stroke(p, vec2<f32>(0.0, -0.6), vec2<f32>(0.5,-0.3), s);
    return g;
}

fn layer_runes(uv: vec2<f32>) -> vec4<f32> {
    let t   = scene.time;
    let asp = scene.resolution.x / scene.resolution.y;

    // Tile runes in a 4×3 grid
    let grid = vec2<f32>(4.0, 3.0);
    let cell = uv * grid;
    let cell_id = floor(cell);
    let cell_uv = (fract(cell) - 0.5) * 2.0;   // centered -1..1 per cell

    let id    = cell_id.x + cell_id.y * grid.x;
    let h     = hash21(cell_id);

    // Per-rune flicker: some runes glow bright, others dim, others off
    let flicker_speed = 0.3 + h * 1.2;
    let phase         = h * 6.2831;
    let flicker       = pow(max(0.0, sin(t * flicker_speed + phase)), 2.5);

    // "Charging" animation driven by berzerker_rage
    let rage_pulse = 1.0 + scene.berzerker_rage * sin(t * 8.0 + id * 0.7) * 0.5;

    let scale = 0.38;
    var glyph_glow = 0.0;

    // Assign rune type by cell hash
    let rune_type = u32(h * 4.0);
    if rune_type == 0u { glyph_glow = rune_algiz(cell_uv, scale); }
    else if rune_type == 1u { glyph_glow = rune_tiwaz(cell_uv, scale); }
    else if rune_type == 2u { glyph_glow = rune_isa  (cell_uv, scale); }
    else                    { glyph_glow = rune_fehu (cell_uv, scale); }

    glyph_glow = clamp(glyph_glow, 0.0, 1.5);

    // Rune color: base rune_glow theme color, intensified by rage
    let intensity = flicker * rage_pulse * theme.rune_opacity;
    let col = theme.rune_glow.rgb * glyph_glow * intensity;

    // Accent inner glow with primary_neon on high-intensity runes
    let accent = theme.primary_neon.rgb * glyph_glow * intensity * scene.berzerker_rage * 0.4;

    return vec4<f32>(col + accent, glyph_glow * intensity * 0.35);
}


// =============================================================================
// SECTION 7 — LAYER: NEON CYAN ILLUMINATION SYSTEM
// Point lights + area scan lines + plasma arc tendrils.
// SwiftUI analogy: .shadow(color:radius:) stacked with ZStack compositing
// =============================================================================
fn layer_neon_lighting(uv: vec2<f32>) -> vec4<f32> {
    let t   = scene.time;
    let asp = scene.resolution.x / scene.resolution.y;
    var p   = uv - vec2<f32>(0.5);
    p.x    *= asp;

    var total_light = vec3<f32>(0.0);

    // --- 1. Primary arc light (top — overhead war-light)
    let arc_pos   = vec2<f32>(0.0, 0.85) * vec2<f32>(asp, 1.0) * 0.5;
    let arc_dist  = length(p - arc_pos);
    let arc_flare = exp(-arc_dist * arc_dist / (theme.neon_bloom_radius * 8.0));
    total_light  += theme.primary_neon.rgb * theme.primary_neon.a * arc_flare * 0.7;

    // --- 2. Secondary fill lights (left/right flanking)
    let fl_pos   = vec2<f32>(-0.5 * asp, 0.0);
    let fr_pos   = vec2<f32>( 0.5 * asp, 0.0);
    let fl_dist  = length(p - fl_pos);
    let fr_dist  = length(p - fr_pos);
    let fl_flare = exp(-fl_dist * 3.5) * 0.25;
    let fr_flare = exp(-fr_dist * 3.5) * 0.25;
    total_light += theme.primary_neon.rgb * (fl_flare + fr_flare);

    // --- 3. Scan lines — horizontal CRT-style plasma bands
    let scan_speed  = 0.18;
    let scan_y      = fract(uv.y * 48.0 - t * scan_speed);
    let scan_line   = pow(sin(scan_y * 3.14159), 12.0) * 0.04;
    // Every 7th line is brighter (berzerker tempo)
    let beat_line   = pow(sin(fract(uv.y * 7.0 - t * 0.5) * 3.14159), 20.0) * 0.12;
    total_light    += theme.primary_neon.rgb * (scan_line + beat_line * scene.berzerker_rage);

    // --- 4. Plasma arc tendrils (animated Lissajous-like curves)
    for (var i = 0; i < 3; i++) {
        let fi      = f32(i);
        let spd     = 0.6 + fi * 0.3;
        let freq    = 3.0 + fi * 1.5;
        let tendril_x = sin(uv.y * freq * 3.14159 + t * spd + fi * 2.094) * 0.35;
        let tendril_p = vec2<f32>(uv.x - 0.5 - tendril_x, 0.0);
        let td        = length(tendril_p) * 1.5;
        let tg        = exp(-td * td / 0.002) * 0.18;
        total_light  += theme.primary_neon.rgb * tg;
    }

    // --- 5. Mouse proximity glow — interactive illumination
    let mouse_p = (scene.mouse - vec2<f32>(0.5)) * vec2<f32>(asp, 1.0);
    let md      = length(p - mouse_p);
    let mglow   = exp(-md * md / 0.04) * 0.35;
    total_light += theme.primary_neon.rgb * mglow;

    return vec4<f32>(total_light, 1.0);
}


// =============================================================================
// SECTION 8 — LAYER: SHATTER PHYSICS RENDERING
// Voronoi-seeded fracture planes + magenta neon crack illumination +
// shard drift animation timed to last shatter event.
// SwiftUI analogy: withAnimation(.spring()) { state.shattered = true }
//                  + matchedGeometryEffect for shard motion
// =============================================================================

// Voronoi distance returns (dist_to_edge, dist_to_cell_center)
fn voronoi(p: vec2<f32>) -> vec2<f32> {
    let i  = floor(p);
    let f  = fract(p);
    var min_d1 = 8.0;
    var min_d2 = 8.0;

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y));
            let cell   = i + offset;
            let jitter = hash22(cell) * 0.8 + 0.1;
            let r      = offset + jitter - f;
            let d      = dot(r, r);
            if d < min_d1 { min_d2 = min_d1; min_d1 = d; }
            else if d < min_d2 { min_d2 = d; }
        }
    }
    return vec2<f32>(sqrt(min_d1), sqrt(min_d2));
}

fn layer_shatter(uv: vec2<f32>) -> vec4<f32> {
    let t_since = scene.time - scene.shatter_time;
    if t_since > 4.0 || scene.shatter_force < 0.01 {
        return vec4<f32>(0.0);
    }

    // Shatter decay envelope: fast crack reveal, slow fade
    let crack_reveal  = smoothstep(0.0, 0.12, t_since);
    let fade_out      = 1.0 - smoothstep(2.5, 4.0, t_since);
    let envelope      = crack_reveal * fade_out * scene.shatter_force;

    let asp = scene.resolution.x / scene.resolution.y;
    var p   = uv - vec2<f32>(0.5);
    p.x    *= asp;

    // Distance from shatter epicenter (normalized)
    let epi   = (scene.shatter_origin - vec2<f32>(0.5)) * vec2<f32>(asp, 1.0);
    let dist  = length(p - epi);

    // Voronoi fracture: tighter cells near epicenter
    let cell_scale  = 6.0 + (1.0 / (dist + 0.1)) * 2.0;
    var frac_uv     = (p + epi * 0.3) * cell_scale;

    // Rotation jitter from impact
    let impact_twist = (1.0 - smoothstep(0.0, 1.5, t_since)) * dist * 3.0;
    frac_uv          = frac_uv * rot2(impact_twist);

    let vor = voronoi(frac_uv);
    let edge_dist = vor.y - vor.x;  // distance to Voronoi edge = crack

    // Crack glow
    let crack_width = theme.shatter_edge_width / scene.resolution.y;
    let crack_glow  = neon_glow(edge_dist, crack_width, theme.neon_bloom_radius * 0.8);

    // Radial falloff from epicenter
    let radial = exp(-dist * dist / (0.35 * scene.shatter_force));

    // Shard drift: individual shards translate outward over time
    let shard_id    = hash21(floor(frac_uv));
    let drift_dir   = normalize((vec2<f32>(shard_id, hash11(shard_id)) - 0.5));
    let drift_speed = 0.4 + shard_id * 0.6;
    let drift_amt   = t_since * drift_speed * scene.shatter_force * 0.15;
    let drifted_uv  = frac_uv + drift_dir * drift_amt;
    let dvor        = voronoi(drifted_uv);
    let dcrack      = neon_glow(dvor.y - dvor.x, crack_width * 1.5, theme.neon_bloom_radius);

    // Color: magenta crack core, fading to theme edge color
    let crack_col   = mix(theme.shatter_neon.rgb, theme.primary_neon.rgb * 0.3, smoothstep(0.0, 0.5, t_since));
    let shard_inner = theme.shatter_neon.rgb * 0.08 * (1.0 - vor.x / 2.0) * radial;

    let total       = (crack_glow + dcrack * 0.6) * radial * theme.shatter_neon.a;
    return vec4<f32>(crack_col * total + shard_inner, total * envelope);
}


// =============================================================================
// SECTION 9 — LAYER: BERZERKER RAGE VFX
// Vignette darkening + blood-ember particle field + screen distortion.
// Driven by berzerker_rage uniform (0.0 = calm, 1.0 = full rage).
// SwiftUI analogy: .overlay(RageOverlay().opacity(rage)) with GeometryEffect
// =============================================================================
fn layer_berzerker_rage(uv: vec2<f32>) -> vec4<f32> {
    let t    = scene.time;
    let rage = scene.berzerker_rage;
    if rage < 0.01 { return vec4<f32>(0.0); }

    // --- Vignette darkening at screen edges
    let vig_uv  = uv * 2.0 - 1.0;
    let vig     = 1.0 - dot(vig_uv * 0.65, vig_uv * 0.65);
    let vig_inv = 1.0 - smoothstep(0.2, 1.0, vig);

    // --- Ember/blood particles: hash-seeded point sprites
    var ember_total = vec3<f32>(0.0);
    for (var i = 0; i < 24; i++) {
        let fi   = f32(i);
        let seed = vec2<f32>(fi * 0.137, fi * 0.271);
        let h    = hash22(seed);

        // Rising ember motion
        let speed  = 0.08 + h.y * 0.12;
        let px     = h.x + sin(t * (0.5 + h.x) + fi) * 0.04;
        let py     = fract(h.y + t * speed);
        let ep     = vec2<f32>(px, 1.0 - py);  // rises upward
        let ed     = length(uv - ep);
        let eg     = exp(-ed * ed / 0.0004) * (0.4 + h.x * 0.6);

        // Color: mix ember_core red with shatter magenta based on hash
        let ecol = mix(theme.ember_core.rgb, theme.shatter_neon.rgb, h.x * 0.5);
        ember_total += ecol * eg;
    }

    // --- Screen shake distortion lines
    let dist_line = sin(uv.y * 220.0 + t * 25.0) * rage * 0.003;
    let shake_sample = fbm(vec2<f32>(uv.x + dist_line, uv.y) * 8.0 + t * 2.0);
    let shock_wave = shake_sample * rage * 0.06;

    // --- Red vignette pulse
    let pulse     = sin(t * 6.0 + 1.5707) * 0.5 + 0.5;
    let vig_col   = theme.ember_core.rgb * vig_inv * rage * pulse * 0.45;

    return vec4<f32>(vig_col + ember_total + shock_wave * theme.ember_core.rgb, rage * vig_inv * 0.6);
}


// =============================================================================
// SECTION 10 — LAYER: DEPTH FOG & ATMOSPHERIC PARALLAX
// Multi-depth fog planes drifting at different speeds simulate Z-depth.
// SwiftUI analogy: ZStack with .offset() and .blur() per layer
// =============================================================================
fn layer_depth_fog(uv: vec2<f32>) -> vec4<f32> {
    let t = scene.time;

    // Three fog planes at different depths (speed/opacity differentiated)
    var fog_total = 0.0;
    let speeds  = array<f32, 3>(0.018, 0.009, 0.004);
    let scales  = array<f32, 3>(2.2,   1.4,   0.8);
    let opacity = array<f32, 3>(0.04,  0.025, 0.015);

    for (var i = 0; i < 3; i++) {
        let drift  = vec2<f32>(t * speeds[i], t * speeds[i] * 0.4);
        let n      = fbm((uv + drift) * scales[i]);
        fog_total += n * opacity[i];
    }

    // Fog color: dark teal, consistent with glass_edge theme
    let fog_col = mix(theme.background_deep.rgb, theme.glass_edge.rgb, fog_total * 4.0);
    return vec4<f32>(fog_col, fog_total);
}


// =============================================================================
// SECTION 11 — COMPOSITING & POST-PROCESSING
// Layers blended using physically-inspired operators.
// SwiftUI analogy: compositingGroup() + blendMode(.screen/.multiply)
// =============================================================================
fn blend_screen(base: vec3<f32>, top: vec3<f32>, alpha: f32) -> vec3<f32> {
    let b = 1.0 - (1.0 - base) * (1.0 - top * alpha);
    return mix(base, b, alpha);
}

fn blend_add(base: vec3<f32>, top: vec3<f32>, alpha: f32) -> vec3<f32> {
    return base + top * alpha;
}

fn blend_over(base: vec3<f32>, top: vec3<f32>, alpha: f32) -> vec3<f32> {
    return mix(base, top, clamp(alpha, 0.0, 1.0));
}

// Tone mapping: ACES filmic curve (Narkowicz approximation)
fn tonemap_aces(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Vignette
fn vignette(uv: vec2<f32>, strength: f32) -> f32 {
    let p = uv * 2.0 - 1.0;
    return 1.0 - dot(p * strength, p * strength);
}

// Chromatic aberration (RGB split)
fn chromatic_aberration(uv: vec2<f32>, amount: f32) -> vec3<f32> {
    // We compute offsets; actual sampling would need a texture —
    // here we simulate the color-split by offsetting the UV for each channel
    // in the neon layer (approximated analytically).
    let dir    = normalize(uv - vec2<f32>(0.5) + vec2<f32>(0.001));
    let r_uv   = uv + dir * amount;
    let b_uv   = uv - dir * amount;
    // Return per-channel UV shifts as color coefficients for downstream mixing
    return vec3<f32>(r_uv.x - uv.x, 0.0, b_uv.x - uv.x);
}

// Film grain
fn grain(uv: vec2<f32>, t: f32, strength: f32) -> f32 {
    return (hash21(uv * 1000.0 + t * 0.03) - 0.5) * strength;
}


// =============================================================================
// SECTION 12 — FRAGMENT ENTRY POINT
// All layers composed here in a clear, declarative stack.
// SwiftUI analogy: body: some View { ZStack { layers... }.modifier(postFX) }
// =============================================================================
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv  = in.uv;
    let t   = scene.time;

    // --- Subtle chromatic aberration UV warp for neon layers
    let ca_amount = 0.0015 + scene.berzerker_rage * 0.003;
    let ca_shift  = chromatic_aberration(uv, ca_amount);

    // Warp UV slightly by berzerker rage for screen distortion
    let rage_warp  = sin(uv.y * 180.0 + t * 30.0) * scene.berzerker_rage * 0.0018;
    let warped_uv  = vec2<f32>(uv.x + rage_warp + ca_shift.x * 0.3, uv.y);

    // ---- LAYER STACK (bottom to top) ----------------------------------------

    // 0. Deep void background
    var color = theme.background_deep.rgb;

    // 1. Depth fog atmosphere
    let fog     = layer_depth_fog(uv);
    color       = blend_over(color, fog.rgb, fog.a * 0.7);

    // 2. Obsidian glass panel (primary surface)
    let glass   = layer_glass_bg(uv);
    color       = blend_over(color, glass.rgb, glass.a);

    // 3. Elder Futhark rune grid (sub-surface glow, under main lighting)
    let runes   = layer_runes(uv);
    color       = blend_screen(color, runes.rgb, runes.a * 0.9);

    // 4. Neon cyan illumination (screen blend for light addition)
    let neon    = layer_neon_lighting(warped_uv);
    color       = blend_screen(color, neon.rgb, 0.85);

    // 5. Shatter physics — magenta cracks (additive, always on top of glass)
    let shatter = layer_shatter(uv);
    color       = blend_add(color, shatter.rgb, shatter.a);

    // 6. Berzerker rage vfx (additive embers + multiplicative vignette)
    let rage_vfx = layer_berzerker_rage(uv);
    color        = blend_add(color, rage_vfx.rgb, rage_vfx.a);

    // ---- POST-PROCESSING -------------------------------------------------------

    // Film grain
    color += grain(uv, t, 0.018);

    // Vignette darkening at border
    let vig    = clamp(vignette(uv, 0.55), 0.0, 1.0);
    color     *= vig;

    // ACES filmic tone mapping — maps HDR neon values to displayable range
    color      = tonemap_aces(color * 1.15);

    // Gamma correction (linear → sRGB)
    color      = pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}

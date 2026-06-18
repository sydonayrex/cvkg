// =============================================================================
// CYBERPUNK VIKING BERZERKER SHADER -- WASM variant
// P1-12 fix: on wasm32/WebGL2, texture binding arrays are not supported.
// This file mirrors common.wgsl but with a SINGLE-texture binding (no array)
// to match the bind group layout that uses count: None on WASM.
// =============================================================================

struct ColorTheme {
    primary_neon:    vec4<f32>,
    shatter_neon:    vec4<f32>,
    glass_base:      vec4<f32>,
    glass_edge:      vec4<f32>,
    rune_glow:       vec4<f32>,
    ember_core:      vec4<f32>,
    background_deep: vec4<f32>,
    mani_glow:       vec4<f32>,
    glass_blur_strength:  f32,
    shatter_edge_width:   f32,
    neon_bloom_radius:    f32,
    rune_opacity:         f32,
    glass_tint_adapt:     f32,
    glass_ior:            f32,
    color_space:          u32,
    _pad0: f32, _pad1: f32,
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
    berzerker_mode:  u32,
    scroll_offset:   f32,
    scale_factor:    f32,
    scene_type:      u32,
    fireball_pos:    vec2<f32>,
    _pad0:           f32,
    _pad1:           f32,
};

// --- Group 2: Berserker Uniforms ---
@group(2) @binding(0) var<uniform> theme: ColorTheme;
@group(2) @binding(1) var<uniform> scene: SceneUniforms;

// --- Group 0: Main Texture (single, not array, for WASM) ---
// P1-12: WebGL2 does not support binding_array. Match the Rust bind
// group layout (count: None) by using a single texture here.
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

// --- Group 1: Environment / Blur ---
@group(1) @binding(0) var t_env: texture_2d<f32>;
@group(1) @binding(1) var s_env: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       world_pos:     vec3<f32>,
    @location(1)       normal:        vec3<f32>,
    @location(2)       uv:            vec2<f32>,
    @location(3)       color:         vec4<f32>,
    @location(4)       tex_index:     u32,
    @location(5)       corner_radius: f32,
    @location(6)       border_mask:   vec4<f32>,
};

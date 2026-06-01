// =============================================================================
// SVG Filter Shaders — WGPU WGSL
// =============================================================================
// Implements all 16 SVG filter primitives as fullscreen quad fragment shaders.
// Each primitive is selected via a mode uniform. All shaders share the same
// vertex stage (fullscreen triangle) and bind group layout.
//
// Bind group layout (all shaders):
//   @group(0) @binding(0) var<uniform> params: FilterParams;
//   @group(0) @binding(1) var t_src: texture_2d<f32>;
//   @group(0) @binding(2) var s_src: sampler;
//   @group(0) @binding(3) var t_src2: texture_2d<f32>;  // second input (blend/composite/displacement)
//   @group(0) @binding(4) var s_src2: sampler;
//
// =============================================================================

// ── Shared Uniforms ──────────────────────────────────────────────────────────

struct FilterParams {
    // Region: x, y, width, height in pixels
    region:       vec4<f32>,
    // Source texture size (for texel step computation)
    src_size:     vec4<f32>,
    // Second source texture size
    src2_size:    vec4<f32>,
    // Mode selector (see constants below)
    mode:         u32,
    // Sub-mode / variant (e.g. blend type, composite operator)
    sub_mode:     u32,
    // Generic parameters (usage varies by shader)
    param0:       f32,
    param1:       f32,
    param2:       f32,
    param3:       f32,
    // Color matrix (4x5 stored as 5 vec4 rows, last component of each is the offset)
    cm_row0:      vec4<f32>,
    cm_row1:      vec4<f32>,
    cm_row2:      vec4<f32>,
    cm_row3:      vec4<f32>,
    // Flood color + opacity
    flood_color:  vec4<f32>,
    // Offset
    offset:       vec2<f32>,
    // Convolve matrix (up to 9 coefficients for 3x3)
    kernel:       vec4<f32>,  // k0 k1 k2 k3
    kernel2:      vec4<f32>,  // k4 k5 k6 k7
    kernel3:      f32,        // k8
    kernel_divisor: f32,
    kernel_bias:    f32,
    _kpad:          f32,
    // Displacement map scale
    disp_scale:   f32,
    _dpad0:       f32,
    _dpad1:       f32,
    _dpad2:       f32,
    // Turbulence parameters
    turb_base_freq: vec2<f32>,
    turb_seed:      f32,
    turb_num_octaves: f32,
    _tpad:            f32,
};

// ── Mode Constants ───────────────────────────────────────────────────────────

const MODE_GAUSSIAN_BLUR_H: u32 = 0u;
const MODE_GAUSSIAN_BLUR_V: u32 = 1u;
const MODE_COLOR_MATRIX:    u32 = 2u;
const MODE_BLEND:           u32 = 3u;
const MODE_COMPOSITE:       u32 = 4u;
const MODE_FLOOD:           u32 = 5u;
const MODE_OFFSET:          u32 = 6u;
const MODE_MERGE:           u32 = 7u;
const MODE_COMPONENT_XFER:  u32 = 8u;
const MODE_CONVOLVE:        u32 = 9u;
const MODE_DISPLACEMENT:    u32 = 10u;
const MODE_MORPHOLOGY:      u32 = 11u;
const MODE_TILE:            u32 = 12u;
const MODE_TURBULENCE:      u32 = 13u;

// Blend sub-modes
const BLEND_NORMAL:   u32 = 0u;
const BLEND_MULTIPLY: u32 = 1u;
const BLEND_SCREEN:   u32 = 2u;
const BLEND_DARKEN:   u32 = 3u;
const BLEND_LIGHTEN:  u32 = 4u;

// Composite sub-modes
const COMPOSITE_OVER:    u32 = 0u;
const COMPOSITE_IN:      u32 = 1u;
const COMPOSITE_OUT:     u32 = 2u;
const COMPOSITE_ATOP:    u32 = 3u;
const COMPOSITE_XOR:     u32 = 4u;
const COMPOSITE_LIGHTER: u32 = 5u;

// Morphology sub-modes
const MORPH_ERODE:  u32 = 0u;
const MORPH_DILATE: u32 = 1u;

// Component transfer sub-modes
const XFER_IDENTITY:     u32 = 0u;
const XFER_TABLE:        u32 = 1u;
const XFER_DISCRETE:     u32 = 2u;
const XFER_LINEAR:       u32 = 3u;
const XFER_GAMMA:        u32 = 4u;

// ── Bindings ────────────────────────────────────────────────────────────────

@group(0) @binding(0) var<uniform> params: FilterParams;
@group(0) @binding(1) var t_src: texture_2d<f32>;
@group(0) @binding(2) var s_src: sampler;
@group(0) @binding(3) var t_src2: texture_2d<f32>;
@group(0) @binding(4) var s_src2: sampler;

// ── Fullscreen Triangle Vertex ──────────────────────────────────────────────

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0)             texcoord: vec2<f32>,
};

@vertex
fn vs_filter(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    // Fullscreen triangle: (-1,-1), (3,-1), (-1,3)
    let x = f32((idx & 1u) << 2u) - 1.0;
    let y = f32((idx >> 1u) << 2u) - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ── Fragment: Mode Dispatch ─────────────────────────────────────────────────

@fragment
fn fs_filter(in: VertexOutput) -> @location(0) vec4<f32> {
    switch params.mode {
        case MODE_GAUSSIAN_BLUR_H: { return gaussian_blur_h(in); }
        case MODE_GAUSSIAN_BLUR_V: { return gaussian_blur_v(in); }
        case MODE_COLOR_MATRIX:    { return color_matrix(in); }
        case MODE_BLEND:           { return blend(in); }
        case MODE_COMPOSITE:       { return composite(in); }
        case MODE_FLOOD:           { return flood(in); }
        case MODE_OFFSET:          { return offset(in); }
        case MODE_MERGE:           { return merge(in); }
        case MODE_COMPONENT_XFER:  { return component_transfer(in); }
        case MODE_CONVOLVE:        { return convolve(in); }
        case MODE_DISPLACEMENT:    { return displacement(in); }
        case MODE_MORPHOLOGY:      { return morphology(in); }
        case MODE_TILE:            { return tile(in); }
        case MODE_TURBULENCE:      { return turbulence(in); }
        default: { return textureSample(t_src, s_src, in.texcoord); }
    }
}

// ── Gaussian Blur (Separable) ───────────────────────────────────────────────
// param0 = kernel radius in pixels
// param1 = sigma

fn gaussian_weight(offset: f32, sigma: f32) -> f32 {
    let s = max(sigma, 0.3); // avoid div by zero
    return exp(-0.5 * (offset * offset) / (s * s));
}

fn gaussian_blur_h(in: VertexOutput) -> vec4<f32> {
    let radius = i32(params.param0);
    let sigma = params.param1;
    let texel = 1.0 / params.src_size.xy;
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;

    for (var i: i32 = -radius; i <= radius; i++) {
        let w = gaussian_weight(f32(i), sigma);
        let uv = in.texcoord + vec2<f32>(f32(i) * texel.x, 0.0);
        color += textureSample(t_src, s_src, uv) * w;
        weight_sum += w;
    }

    return color / max(weight_sum, 0.001);
}

fn gaussian_blur_v(in: VertexOutput) -> vec4<f32> {
    let radius = i32(params.param0);
    let sigma = params.param1;
    let texel = 1.0 / params.src_size.xy;
    var color = vec4<f32>(0.0);
    var weight_sum = 0.0;

    for (var i: i32 = -radius; i <= radius; i++) {
        let w = gaussian_weight(f32(i), sigma);
        let uv = in.texcoord + vec2<f32>(0.0, f32(i) * texel.y);
        color += textureSample(t_src, s_src, uv) * w;
        weight_sum += w;
    }

    return color / max(weight_sum, 0.001);
}

// ── Color Matrix ────────────────────────────────────────────────────────────

fn color_matrix(in: VertexOutput) -> vec4<f32> {
    let src = textureSample(t_src, s_src, in.texcoord);
    let r = dot(params.cm_row0, vec4<f32>(src.rgb, 1.0));
    let g = dot(params.cm_row1, vec4<f32>(src.rgb, 1.0));
    let b = dot(params.cm_row2, vec4<f32>(src.rgb, 1.0));
    let a = dot(params.cm_row3, vec4<f32>(src.rgb, 1.0));
    return vec4<f32>(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0), clamp(a, 0.0, 1.0));
}

// ── Blend ───────────────────────────────────────────────────────────────────

fn blend_normal(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> { return a; }
fn blend_multiply(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> { return a * b; }
fn blend_screen(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> { return a + b - a * b; }
fn blend_darken(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> { return min(a, b); }
fn blend_lighten(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> { return max(a, b); }

fn blend(in: VertexOutput) -> vec4<f32> {
    let a = textureSample(t_src, s_src, in.texcoord);
    let b = textureSample(t_src2, s_src2, in.texcoord);
    switch params.sub_mode {
        case BLEND_MULTIPLY: { return blend_multiply(a, b); }
        case BLEND_SCREEN:   { return blend_screen(a, b); }
        case BLEND_DARKEN:   { return blend_darken(a, b); }
        case BLEND_LIGHTEN:  { return blend_lighten(a, b); }
        default:             { return blend_normal(a, b); }
    }
}

// ── Composite ────────────────────────────────────────────────────────────────

fn composite(in: VertexOutput) -> vec4<f32> {
    let a = textureSample(t_src, s_src, in.texcoord);
    let b = textureSample(t_src2, s_src2, in.texcoord);
    let fa = a.a;
    let fb = b.a;
    switch params.sub_mode {
        case COMPOSITE_IN: {
            return a * fb;
        }
        case COMPOSITE_OUT: {
            return a * (1.0 - fb);
        }
        case COMPOSITE_ATOP: {
            return a * fb + b * (1.0 - fa);
        }
        case COMPOSITE_XOR: {
            return a * (1.0 - fb) + b * (1.0 - fa);
        }
        case COMPOSITE_LIGHTER: {
            return a + b;
        }
        default { // OVER
            return a + b * (1.0 - fa);
        }
    }
}

// ── Flood ───────────────────────────────────────────────────────────────────

fn flood(in: VertexOutput) -> vec4<f32> {
    return params.flood_color;
}

// ── Offset ──────────────────────────────────────────────────────────────────

fn offset(in: VertexOutput) -> vec4<f32> {
    let uv = in.texcoord - params.offset / params.src_size.xy;
    return textureSample(t_src, s_src, uv);
}

// ── Merge ───────────────────────────────────────────────────────────────────
// Merge layers multiple inputs by simple alpha compositing (over).
// For 2 inputs: src over src2. For more, we'd need multiple passes.

fn merge(in: VertexOutput) -> vec4<f32> {
    let a = textureSample(t_src, s_src, in.texcoord);
    let b = textureSample(t_src2, s_src2, in.texcoord);
    // Alpha compositing: a over b
    let out_a = a.a + b.a * (1.0 - a.a);
    if out_a < 0.001 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>((a.rgb * a.a + b.rgb * b.a * (1.0 - a.a)) / out_a, out_a);
}

// ── Component Transfer ──────────────────────────────────────────────────────
// param0-3: linear slope/intercept per channel
// sub_mode: 0=identity, 1=table, 2=discrete, 3=linear, 4=gamma
// param0 = gamma (for gamma mode), param1 = linear slope, param2 = linear intercept

fn component_transfer(in: VertexOutput) -> vec4<f32> {
    let src = textureSample(t_src, s_src, in.texcoord);
    switch params.sub_mode {
        case XFER_LINEAR: {
            let slope = params.param1;
            let intercept = params.param2;
            return vec4<f32>(
                clamp(src.r * slope + intercept, 0.0, 1.0),
                clamp(src.g * slope + intercept, 0.0, 1.0),
                clamp(src.b * slope + intercept, 0.0, 1.0),
                clamp(src.a * slope + intercept, 0.0, 1.0),
            );
        }
        case XFER_GAMMA: {
            let gamma = max(params.param0, 0.01);
            return vec4<f32>(
                pow(src.r, gamma),
                pow(src.g, gamma),
                pow(src.b, gamma),
                src.a,
            );
        }
        default { // IDENTITY, TABLE, DISCRETE: pass through
            return src;
        }
    }
}

// ── Convolve Matrix (3x3) ───────────────────────────────────────────────────
// kernel: k0-k8 stored in params.kernel (vec4), params.kernel2 (vec4), params.kernel3.x
// kernel_divisor, kernel_bias

fn convolve(in: VertexOutput) -> vec4<f32> {
    let texel = 1.0 / params.src_size.xy;
    let k = array<vec4<f32>, 3>(
        params.kernel,
        params.kernel2,
        vec4<f32>(params.kernel3, 0.0, 0.0, 0.0)
    );
    let divisor = select(params.kernel_divisor, 1.0, params.kernel_divisor == 0.0);
    let bias = params.kernel_bias;

    var color = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            let uv = in.texcoord + vec2<f32>(f32(x) * texel.x, f32(y) * texel.y);
            let sample = textureSample(t_src, s_src, uv);
            let ki = k[u32(y + 1)][u32(x + 1)];
            color += sample * ki;
        }
    }

    color = color / divisor + vec4<f32>(bias);
    return vec4<f32>(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), max(color.a, 0.0));
}

// ── Displacement Map ────────────────────────────────────────────────────────
// param0 = scale
// Channel selectors encoded in sub_mode bits:
//   bits 0-1: x channel (0=R, 1=G, 2=B, 3=A)
//   bits 2-3: y channel

fn channel_from_select(sample: vec4<f32>, sel: u32) -> f32 {
    switch sel {
        case 0u: { return sample.r; }
        case 1u: { return sample.g; }
        case 2u: { return sample.b; }
        default: { return sample.a; }
    }
}

fn displacement(in: VertexOutput) -> vec4<f32> {
    let scale = params.disp_scale;
    let x_sel = params.sub_mode & 3u;
    let y_sel = (params.sub_mode >> 2u) & 3u;
    let disp = textureSample(t_src2, s_src2, in.texcoord);
    let dx = (channel_from_select(disp, x_sel) - 0.5) * scale;
    let dy = (channel_from_select(disp, y_sel) - 0.5) * scale;
    let uv = in.texcoord + vec2<f32>(dx, dy);
    return textureSample(t_src, s_src, uv);
}

// ── Morphology ──────────────────────────────────────────────────────────────
// sub_mode: 0=erode, 1=dilate
// param0 = radius_x, param1 = radius_y

fn morphology(in: VertexOutput) -> vec4<f32> {
    let rx = i32(params.param0);
    let ry = i32(params.param1);
    let texel = 1.0 / params.src_size.xy;
    var result = textureSample(t_src, s_src, in.texcoord);

    if params.sub_mode == u32(MORPH_DILATE) {
        // Dilate: take max
        for (var y: i32 = -ry; y <= ry; y++) {
            for (var x: i32 = -rx; x <= rx; x++) {
                let uv = in.texcoord + vec2<f32>(f32(x) * texel.x, f32(y) * texel.y);
                let s = textureSample(t_src, s_src, uv);
                result = max(result, s);
            }
        }
    } else {
        // Erode: take min
        for (var y: i32 = -ry; y <= ry; y++) {
            for (var x: i32 = -rx; x <= rx; x++) {
                let uv = in.texcoord + vec2<f32>(f32(x) * texel.x, f32(y) * texel.y);
                let s = textureSample(t_src, s_src, uv);
                result = min(result, s);
            }
        }
    }

    return result;
}

// ── Tile ────────────────────────────────────────────────────────────────────
// Tile the source (alpha-only) into the filter region.

fn tile(in: VertexOutput) -> vec4<f32> {
    let src_w = params.src_size.x;
    let src_h = params.src_size.y;
    let region = params.region;
    // Map UV to source texture space, wrapping
    let u = in.texcoord.x * region.z;
    let v = in.texcoord.y * region.w;
    let src_uv = vec2<f32>(u / src_w, v / src_h) % 1.0;
    return textureSample(t_src, s_src, fract(src_uv));
}

// ── Turbulence (Perlin-like) ────────────────────────────────────────────────
// Uses a simple value-noise approach with octave summation.
// turb_base_freq = base frequency (x, y)
// turb_seed = random seed
// turb_num_octaves = number of octaves

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h + params.turb_seed) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn turbulence(in: VertexOutput) -> vec4<f32> {
    let region = params.region;
    let pos = in.texcoord * region.zw;
    let base_freq = params.turb_base_freq;
    let octaves = i32(params.turb_num_octaves);

    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var i: i32 = 0; i < 8; i++) {
        if i >= octaves { break; }
        value += amplitude * noise(pos * frequency * base_freq);
        frequency *= 2.0;
        amplitude *= 0.5;
    }

    // Return as RGBA (same value in all channels for turbulence)
    let v = clamp(value, 0.0, 1.0);
    return vec4<f32>(v, v, v, 1.0);
}

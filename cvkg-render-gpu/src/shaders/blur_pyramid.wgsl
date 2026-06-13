// =============================================================================
// Dual Kawase Blur Pyramid — Backdrop Capture Architecture
// =============================================================================
//
// This shader implements a Dual Kawase Blur for generating a mip-chain
// backdrop pyramid used by glass/frosted-ui elements.
//
// Modes:
//   0 = Downsample (read from mip N, write to mip N+1, offset = iteration)
//   1 = Upsample   (read from mip N+1, accumulate into mip N, offset = iteration)
//   2 = Composite  (sample blurred backdrop by blur_radius, blend over glass quad)
//
// The Kawase offset pattern uses a diagonal cross kernel:
//   offsets = [(+o,+o), (-o,+o), (-o,-o), (+o,-o)]
// where o = iteration_index, producing increasingly wide sampling.
//
// =============================================================================

// Re-use the shared uniforms and fullscreen vertex from common.wgsl.
// This shader must be #included by the pipeline shader or compiled standalone.
// For standalone pipeline compilation, we define our own bindings.

struct BlurUniforms {
    // xy = src_texture_size (for computing texel UV step)
    // z = mip_level being written
    // w = kernel_width (Kawase offset = iteration index)
    params:       vec4<f32>,
    mode:         u32,      // 0=down, 1=up, 2=composite
    _pad0:        u32,
    _pad1:        u32,
    _pad2:        u32,
};

@group(0) @binding(0) var<uniform> blur: BlurUniforms;

// Source texture (the mip level to read from)
@group(0) @binding(1) var t_src: texture_2d<f32>;
@group(0) @binding(2) var s_src: sampler;

// --- Fullscreen triangle vertex ---
struct BlurVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0)             texcoord: vec2<f32>,
};

@vertex
fn vs_blur(@builtin(vertex_index) idx: u32) -> BlurVertexOutput {
    var out: BlurVertexOutput;
    let x = f32(i32(idx) / 2) * 4.0 - 1.0;
    let y = f32(i32(idx) % 2) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// --- Kawase Downsample Fragment ---
// Reads 4 diagonal taps from current mip, writes to next mip (half res).
// Each invocation covers one output pixel; offset = blur_params.w
@fragment
fn fs_kawase_down(in: BlurVertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / blur.params.xy;
    let offset = blur.params.w;

    // Kawase diagonal offsets: 4-tap sparse kernel
    let o = offset * texel;
    var c = vec4<f32>(0.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2<f32>( o.x,  o.y));
    c += textureSample(t_src, s_src, in.texcoord + vec2<f32>(-o.x,  o.y));
    c += textureSample(t_src, s_src, in.texcoord + vec2<f32>(-o.x, -o.y));
    c += textureSample(t_src, s_src, in.texcoord + vec2<f32>( o.x, -o.y));

    return c * 0.25;
}

// --- Kawase Upsample Fragment ---
// Reads 8 taps (4 diagonal + 4 axis-aligned) from higher mip (N+1),
// accumulates into current mip (N). This is the canonical Dual Kawase
// upsample kernel that prevents box artifacts at low mip levels.
@fragment
fn fs_kawase_up(in: BlurVertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / blur.params.xy;
    let offset = blur.params.w;
    let o = offset * texel;

    // 8-tap Kawase upsample: 4 diagonal + 4 axis-aligned
    var c = vec4<f32>(0.0);
    // Diagonal taps (weight: 1/12 each)
    c += textureSample(t_src, s_src, in.texcoord + vec2( o.x,  o.y)) * (1.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2(-o.x,  o.y)) * (1.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2(-o.x, -o.y)) * (1.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2( o.x, -o.y)) * (1.0/12.0);
    // Axis-aligned taps (weight: 2/12 each)
    c += textureSample(t_src, s_src, in.texcoord + vec2( o.x, 0.0)) * (2.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2(-o.x, 0.0)) * (2.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2(0.0,  o.y)) * (2.0/12.0);
    c += textureSample(t_src, s_src, in.texcoord + vec2(0.0, -o.y)) * (2.0/12.0);

    return c;
}

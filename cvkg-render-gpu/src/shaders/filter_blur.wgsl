// =============================================================================
// SVG Filter: Separable Gaussian Blur
// =============================================================================
// Two-pass separable Gaussian blur for feGaussianBlur.
// Pass 1: Horizontal blur (read from input, write to temp)
// Pass 2: Vertical blur (read from temp, write to output)
//
// Uniforms:
//   params.xy = input texture dimensions (for texel size)
//   params.z  = standard deviation
//   params.w  = direction (0 = horizontal, 1 = vertical)
// =============================================================================

struct BlurUniforms {
    params: vec4<f32>,  // xy = texture size, z = std_deviation, w = direction
    kernel_size: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> blur: BlurUniforms;
@group(0) @binding(1) var t_input: texture_2d<f32>;
@group(0) @binding(2) var s_input: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

@vertex
fn vs_filter(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(idx) / 2) * 4.0 - 1.0;
    let y = f32(i32(idx) % 2) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.texcoord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_gaussian_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = vec2<f32>(1.0 / blur.params.x, 1.0 / blur.params.y);
    let sigma = max(blur.params.z, 0.5);
    let kernel_radius = min(i32(sigma * 3.0), 16);

    var result = vec4<f32>(0.0);
    var weight_sum = 0.0;

    let is_vertical = blur.params.w > 0.5;
    let dir = select(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0), is_vertical);

    for (var i: i32 = -kernel_radius; i <= kernel_radius; i++) {
        let offset = dir * f32(i) * texel;
        let weight = exp(-0.5 * f32(i * i) / (sigma * sigma));
        result += textureSample(t_input, s_input, in.texcoord + offset) * weight;
        weight_sum += weight;
    }

    return result / weight_sum;
}

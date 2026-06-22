// =============================================================================
// SVG Filter: Blend (feBlend)
// =============================================================================
// Blends the source texture over the destination using the specified blend mode.
// The destination is already in the output texture; we read it and composite.
//
// Blend modes (blend_mode uniform):
//   0 = normal (src-over-dst alpha compositing)
//   1 = multiply
//   2 = screen
//   3 = darken
//   4 = lighten
// =============================================================================

struct BlendUniforms {
    blend_mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> blend: BlendUniforms;
@group(0) @binding(1) var t_source: texture_2d<f32>;
@group(0) @binding(2) var s_source: sampler;
@group(0) @binding(3) var t_dest: texture_2d<f32>;
@group(0) @binding(4) var s_dest: sampler;

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
fn fs_blend(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(t_source, s_source, in.texcoord);
    let dst = textureSample(t_dest, s_dest, in.texcoord);

    // Compute blended RGB based on mode
    var blended_rgb: vec3<f32>;

    switch blend.blend_mode {
        case 0u: { // normal
            blended_rgb = src.rgb;
        }
        case 1u: { // multiply
            blended_rgb = src.rgb * dst.rgb;
        }
        case 2u: { // screen
            blended_rgb = 1.0 - (1.0 - src.rgb) * (1.0 - dst.rgb);
        }
        case 3u: { // darken
            blended_rgb = min(src.rgb, dst.rgb);
        }
        case 4u: { // lighten
            blended_rgb = max(src.rgb, dst.rgb);
        }
        default: {
            blended_rgb = src.rgb;
        }
    }

    // Porter-Duff src-over compositing for alpha
    let out_alpha = src.a + dst.a * (1.0 - src.a);
    let out_rgb = blended_rgb * src.a + dst.rgb * dst.a * (1.0 - src.a);

    return vec4<f32>(select(out_rgb / max(out_alpha, 0.001), vec3<f32>(0.0), out_alpha < 0.001), out_alpha);
}

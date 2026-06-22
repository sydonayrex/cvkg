// =============================================================================
// SVG Filter: Composite (feComposite)
// =============================================================================
// Composites source over destination using Porter-Duff operators.
//
// Operators (op uniform):
//   0 = over
//   1 = in
//   2 = out
//   3 = atop
//   4 = xor
// =============================================================================

struct CompositeUniforms {
    op: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> comp: CompositeUniforms;
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
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(t_source, s_source, in.texcoord);
    let dst = textureSample(t_dest, s_dest, in.texcoord);

    // Porter-Duff coefficients
    var fa: f32; // source contribution to alpha
    var fb: f32; // dest contribution to alpha

    switch comp.op {
        case 0u: { // over
            fa = 1.0;
            fb = 1.0 - src.a;
        }
        case 1u: { // in
            fa = dst.a;
            fb = 0.0;
        }
        case 2u: { // out
            fa = 1.0 - dst.a;
            fb = 0.0;
        }
        case 3u: { // atop
            fa = dst.a;
            fb = 1.0 - src.a;
        }
        case 4u: { // xor
            fa = 1.0 - dst.a;
            fb = 1.0 - src.a;
        }
        default: { // over
            fa = 1.0;
            fb = 1.0 - src.a;
        }
    }

    let out_alpha = src.a * fa + dst.a * fb;
    let out_rgb = src.rgb * src.a * fa + dst.rgb * dst.a * fb;

    return vec4<f32>(select(out_rgb / max(out_alpha, 0.001), vec3<f32>(0.0), out_alpha < 0.001), out_alpha);
}

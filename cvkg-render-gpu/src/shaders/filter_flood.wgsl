// =============================================================================
// SVG Filter: Flood (feFlood)
// =============================================================================
// Fills the entire output with a solid color.
// =============================================================================

struct FloodUniforms {
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> flood: FloodUniforms;

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
fn fs_flood(in: VertexOutput) -> @location(0) vec4<f32> {
    return flood.color;
}

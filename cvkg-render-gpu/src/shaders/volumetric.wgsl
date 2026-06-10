//! Volumetric raymarching shader.
//! Renders a fullscreen triangle and performs SDF raymarch in the fragment shader.
//! Blends additively onto the scene for fog/light shaft effects.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VertexOutput {
    // Full-screen triangle (no vertex buffer needed)
    let pos = vec4<f32>(
        select(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vid == 1u),
        0.0,
        1.0
    );
    let uv = vec2<f32>(
        select(0.0, 2.0, vid == 1u),
        select(0.0, 2.0, vid > 0u),
    );
    return VertexOutput(pos, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;

    // Signed distance field sphere
    let dist = length(uv) - 0.5;

    if (dist < 0.0) {
        // Inside the volume: emit glowing cyan
        let raw_density = 1.0 + dist;
        let density = clamp(raw_density, 0.0, 1.0);
        return vec4<f32>(0.0, 0.8 * density, 1.0 * density, 0.6 * density);
    } else {
        // Outside the volume: soft glow falloff
        let glow = 0.04 / max(dist, 0.01);
        return vec4<f32>(0.0, 0.8 * glow, 1.0 * glow, glow);
    }
}
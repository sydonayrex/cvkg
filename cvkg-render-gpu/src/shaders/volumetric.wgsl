//! Volumetric raymarching shader.
//! Renders a fullscreen triangle and performs SDF raymarch in the fragment shader.
//! When a hologram rect is active (holo_count > 0), the effect is constrained to
//! the bounding rectangle. Each hologram gets a unique pulsation frequency derived
//! from its id_hash, enabling visual variation across multiple hologram instances.
//! Blends additively onto the scene for fog/light shaft effects.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vid) / 2) * 4.0 - 1.0;
    let y = f32(i32(vid) % 2) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Placeholder: contributes nothing under the additive blend configured on
    // the volumetric pipeline. A real raymarch implementation (SDF fog, light
    // shafts, hologram glow) replaces this body and outputs glow colors that
    // will be added onto the loaded scene.
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

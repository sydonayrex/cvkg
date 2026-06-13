//! Tone mapping shader.
//! Converts HDR (Rgba16Float) scene to LDR (Rgba8UnormSrgb) for display.
//! Implements ACES filmic tone mapping with optional P3 color space output.

struct ToneMapUniforms {
    exposure: f32,
    gamma: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> uniforms: ToneMapUniforms;
@group(0) @binding(1) var hdr_scene: texture_2d<f32>;
@group(0) @binding(2) var hdr_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VertexOutput {
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

// ACES filmic tone mapping curve
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(hdr_scene, hdr_sampler, in.uv).rgb;

    // Apply exposure
    let exposed = hdr_color * uniforms.exposure;

    // ACES tone mapping
    let mapped = aces_tonemap(exposed);

    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / uniforms.gamma));

    return vec4<f32>(gamma_corrected, 1.0);
}

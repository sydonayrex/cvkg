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

// AgX Tonemapping constants and functions.
// Maps high dynamic range scene colors to standard display bounds
// in a perceptually calibrated logarithmic space, preventing highlight shifts.

// Input transform matrix from sRGB to the AgX internal color space
const AGX_IN_MATRIX = mat3x3<f32>(
    vec3<f32>(0.84247906229, 0.07843358840, 0.07922374822),
    vec3<f32>(0.09784776100, 0.84803738020, 0.05413186259),
    vec3<f32>(0.06050045173, 0.07369234857, 0.86617576595)
);

// Output transform matrix back to sRGB display linear space
const AGX_OUT_MATRIX = mat3x3<f32>(
    vec3<f32>(1.19682190398, -0.09702284903, -0.09979925827),
    vec3<f32>(-0.13843969460, 1.20249767228, -0.06405828459),
    vec3<f32>(-0.05838220938, -0.10547482325, 1.16385754286)
);

/// Linear to AgX Logarithmic space conversion
fn linear_to_agx_log(x: vec3<f32>) -> vec3<f32> {
    let min_ev = -10.0;
    let max_ev = 6.5;
    let log_color = log2(clamp(x, vec3<f32>(1e-5), vec3<f32>(65536.0)));
    return clamp((log_color - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Hermite cubic polynomial approximation of the AgX contrast curve
fn agx_contrast_curve(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x3 * x;
    let x5 = x4 * x;
    return clamp(
        15.5 * x5 - 40.14 * x4 + 37.96 * x3 - 14.285 * x2 + 1.92 * x + 0.005,
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );
}

/// Core AgX tonemapping transformation pipeline
fn agx_tonemap(color: vec3<f32>) -> vec3<f32> {
    // 1. Transform to AgX input color space
    let agx_in = transpose(AGX_IN_MATRIX) * color;
    
    // 2. Convert to logarithmic scale
    let log_scale = linear_to_agx_log(agx_in);
    
    // 3. Apply contrast curve
    let curve = agx_contrast_curve(log_scale);
    
    // 4. Transform back to display linear space
    return clamp(transpose(AGX_OUT_MATRIX) * curve, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(hdr_scene, hdr_sampler, in.uv).rgb;

    // Apply exposure
    let exposed = hdr_color * uniforms.exposure;

    // AgX tone mapping
    let mapped = agx_tonemap(exposed);

    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / uniforms.gamma));

    return vec4<f32>(gamma_corrected, 1.0);
}

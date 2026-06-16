// Identity fragment shader for backdrop copies.
// Samples from the main scene texture (t_diffuse[0]) to preserve the backdrop image.
@fragment
fn fs_copy(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample from the scene texture (Group 0, index 0) instead of the environment texture.
    let color = textureSample(t_diffuse[0], s_diffuse, in.uv);
    return vec4<f32>(color.rgb, 1.0);
}

// Extract bright regions of the scene for bloom post-processing.
// Uses a relative luminance gate of 0.5 to isolate high-energy light sources.
// Lowered from 0.8 to capture fireball/particle glow which is HDR-scaled.
@fragment
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.5 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// ACES Filmic Tonemapping function.
// Maps high dynamic range scene colors to low dynamic range display output.
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Composite fragment shader combining the base scene and the blurred bloom texture.
// Applies ACES tonemapping to target the final display output color space.
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(t_diffuse[0], s_diffuse, in.uv);
    let bloom_color = textureSample(t_env, s_env, in.uv);

    // HDR Bloom Fusion (Restrained Apple-style discipline)
    // Raised from 0.2 to 0.6 for visible fire glow bleed
    let hdr_color = scene_color.rgb + (bloom_color.rgb * 0.6);

    // ACES Filmic Tonemapping (Asgard Quality)
    let ldr_color = aces_tonemap(hdr_color);

    return vec4<f32>(ldr_color, scene_color.a);
}
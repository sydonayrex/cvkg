// Identity fragment shader for backdrop copies.
// P1-12 WASM variant: WebGL2 does not support binding_array, so t_diffuse
// is a single texture (declared in common_wasm.wgsl) rather than an array.
@fragment
fn fs_copy(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.uv);
    return vec4<f32>(color.rgb, 1.0);
}

// Extract bright regions of the scene for bloom post-processing.
// P1-12 WASM variant: tex_index is ignored because t_diffuse is a single
// texture on WASM. In practice, only index 0 is ever used.
@fragment
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.5 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// ACES Filmic Tonemapping function.
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Composite fragment shader combining the base scene and the blurred bloom texture.
// P1-12 WASM variant: t_diffuse is a single texture.
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let bloom_color = textureSample(t_env, s_env, in.uv);

    let hdr_color = scene_color.rgb + (bloom_color.rgb * 0.6);
    let ldr_color = aces_tonemap(hdr_color);

    return vec4<f32>(ldr_color, scene_color.a);
}

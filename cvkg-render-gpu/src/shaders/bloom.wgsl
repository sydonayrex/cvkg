@fragment
fn fs_copy(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_env, s_env, in.uv);
    return vec4<f32>(color.rgb, 1.0);
}

@fragment
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.8 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv_screen = in.uv;
    let scene_color = textureSample(t_diffuse[0], s_diffuse, in.uv);
    let bloom_color = textureSample(t_env, s_env, in.uv);

    // HDR Bloom Fusion (Restrained Apple-style discipline)
    let hdr_color = scene_color.rgb + (bloom_color.rgb * 0.2);

    // ACES Filmic Tonemapping (Asgard Quality)
    let ldr_color = aces_tonemap(hdr_color);

    return vec4<f32>(ldr_color, scene_color.a);
}
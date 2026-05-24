@fragment
fn fs_bloom_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let brightness = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if brightness > 0.8 { return color; }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    // High-Fidelity 9-tap Gaussian Blur (Unrolled to avoid dynamic indexing)
    let w0 = 0.153423; let w1 = 0.143254; let w2 = 0.117031; let w3 = 0.081827;
    let w4 = 0.049003; let w5 = 0.025135; let w6 = 0.010861; let w7 = 0.00392; let w8 = 0.0011;
    let tex_offset = 6.0 / scene.resolution.x;
    
    result += textureSample(t_diffuse[0], s_diffuse, in.uv).rgb * w0;
    
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 1.0, 0.0)).rgb * w1;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 1.0, 0.0)).rgb * w1;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 2.0, 0.0)).rgb * w2;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 2.0, 0.0)).rgb * w2;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 3.0, 0.0)).rgb * w3;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 3.0, 0.0)).rgb * w3;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 4.0, 0.0)).rgb * w4;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 4.0, 0.0)).rgb * w4;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 5.0, 0.0)).rgb * w5;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 5.0, 0.0)).rgb * w5;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 6.0, 0.0)).rgb * w6;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 6.0, 0.0)).rgb * w6;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 7.0, 0.0)).rgb * w7;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 7.0, 0.0)).rgb * w7;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(tex_offset * 8.0, 0.0)).rgb * w8;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(tex_offset * 8.0, 0.0)).rgb * w8;
    
    return vec4<f32>(result, 1.0);
}

@fragment
fn fs_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0);
    // High-Fidelity 9-tap Gaussian Blur (Unrolled to avoid dynamic indexing)
    let w0 = 0.153423; let w1 = 0.143254; let w2 = 0.117031; let w3 = 0.081827;
    let w4 = 0.049003; let w5 = 0.025135; let w6 = 0.010861; let w7 = 0.00392; let w8 = 0.0011;
    let tex_offset = 6.0 / scene.resolution.y;
    
    result += textureSample(t_diffuse[0], s_diffuse, in.uv).rgb * w0;
    
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 1.0)).rgb * w1;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 1.0)).rgb * w1;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 2.0)).rgb * w2;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 2.0)).rgb * w2;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 3.0)).rgb * w3;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 3.0)).rgb * w3;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 4.0)).rgb * w4;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 4.0)).rgb * w4;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 5.0)).rgb * w5;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 5.0)).rgb * w5;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 6.0)).rgb * w6;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 6.0)).rgb * w6;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 7.0)).rgb * w7;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 7.0)).rgb * w7;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv + vec2(0.0, tex_offset * 8.0)).rgb * w8;
    result += textureSample(t_diffuse[0], s_diffuse, in.uv - vec2(0.0, tex_offset * 8.0)).rgb * w8;
    
    return vec4<f32>(result, 1.0);
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
    let scene_color = textureSample(t_diffuse[in.tex_index], s_diffuse, in.uv);
    let bloom_color = textureSample(t_env, s_env, in.uv);
    
    // HDR Bloom Fusion (Restrained Apple-style discipline)
    let hdr_color = scene_color.rgb + (bloom_color.rgb * 0.2);
    
    // ACES Filmic Tonemapping (Asgard Quality)
    let ldr_color = aces_tonemap(hdr_color);
    
    return vec4<f32>(ldr_color, scene_color.a);
}

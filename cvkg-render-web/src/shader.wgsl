struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec2<f32>,
    @location(2) screen_pos: vec2<f32>,
}

struct SceneUniforms {
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<uniform> scene: SceneUniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Standard high-fidelity fullscreen triangle trick
    // Indices 0, 1, 2 map to:
    // 0: (-1, -1), UV: (0, 1)
    // 1: ( 3, -1), UV: (2, 1)
    // 2: (-1,  3), UV: (0, -1)
    let x = f32(i32((in_vertex_index << 1u) & 2u)) * 2.0 - 1.0;
    let y = f32(i32(in_vertex_index & 2u)) * 2.0 - 1.0;
    
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    out.world_pos = vec2<f32>(x, y);
    out.screen_pos = out.uv * scene.resolution;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // High-Fidelity Procedural Background: Neon Scanlines & Grid
    let uv = in.uv;
    
    // Pulse based on time
    let pulse = 0.5 + 0.5 * sin(scene.time * 2.0);
    
    // Subtle grid
    let grid_size = 40.0;
    let grid = step(0.98, fract(uv.x * grid_size)) + step(0.98, fract(uv.y * grid_size * (scene.resolution.y / scene.resolution.x)));
    
    // Scanlines
    let scanline = sin(uv.y * 400.0 + scene.time * 5.0) * 0.05;
    
    // Gradient background (Deep Cyberpunk Blue/Purple)
    let base_color = mix(
        vec4<f32>(0.02, 0.01, 0.05, 1.0),
        vec4<f32>(0.05, 0.02, 0.1, 1.0),
        uv.y
    );
    
    var color = base_color + grid * 0.05 + scanline;
    
    // Vignette
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    color *= 1.0 - smoothstep(0.4, 0.8, d);
    
    // Neon accent glow (Cyan/Magenta pulse)
    let glow = smoothstep(0.3, 0.0, abs(uv.y - 0.5 + sin(uv.x * 3.0 + scene.time) * 0.1)) * 0.1;
    color += vec4<f32>(0.0, 0.8, 1.0, 1.0) * glow * pulse;
    
    return color;
}

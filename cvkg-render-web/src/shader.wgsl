struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec2<f32>,
    @location(2) screen_pos: vec2<f32>,
}

struct ComputeParams {
    node_count: u32,
    time: f32,
    delta_time: f32,
    _pad: f32,
}

struct SceneNode {
    position: vec2<f32>,
    size: vec2<f32>,
    color: vec4<f32>,
    flags: u32,
    animation_phase: f32,
}

struct SceneUniforms {
    resolution: vec2<f32>,
    time: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<uniform> scene: SceneUniforms;

@group(1) @binding(0)
var<storage, read_write> nodes: array<SceneNode>;

@group(1) @binding(1)
var<uniform> params: ComputeParams;

// Standard high-fidelity fullscreen triangle trick
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
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
    let uv = in.uv;
    let screen_pos = in.screen_pos;
    
    // 1. Draw animated background
    let pulse = 0.5 + 0.5 * sin(scene.time * 2.0);
    let grid_size = 40.0;
    let grid = step(0.98, fract(uv.x * grid_size)) + step(0.98, fract(uv.y * grid_size * (scene.resolution.y / scene.resolution.x)));
    let scanline = sin(uv.y * 400.0 + scene.time * 5.0) * 0.05;
    let base_color = mix(
        vec4<f32>(0.02, 0.01, 0.05, 1.0),
        vec4<f32>(0.05, 0.02, 0.1, 1.0),
        uv.y
    );
    var color = base_color + grid * 0.05 + scanline;
    let d_bg = distance(uv, vec2<f32>(0.5, 0.5));
    color *= 1.0 - smoothstep(0.4, 0.8, d_bg);
    let glow = smoothstep(0.3, 0.0, abs(uv.y - 0.5 + sin(uv.x * 3.0 + scene.time) * 0.1)) * 0.1;
    color += vec4<f32>(0.0, 0.8, 1.0, 1.0) * glow * pulse;

    // 2. Render Scene Nodes (SDF)
    let node_count = params.node_count;
    for (var i = 0u; i < node_count; i = i + 1u) {
        let node = nodes[i];
        let p = screen_pos - node.position - node.size * 0.5;
        let b = node.size * 0.5;
        
        var d = 1e10;
        if (node.flags == 0u) {
            // Rect
            let q = abs(p) - b;
            d = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
        } else if (node.flags == 1u) {
            // Rounded Rect
            let r = node.animation_phase;
            let q = abs(p) - b + r;
            d = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
        } else if (node.flags == 2u) {
            // Ellipse
            let k0 = length(p / b);
            let k1 = length(p / (b * b));
            d = k0 * (k0 - 1.0) / k1;
        }
        
        // Alpha blending
        let s = smoothstep(1.0, -1.0, d);
        color = mix(color, node.color, s * node.color.a);
    }

    return color;
}

// ============================================
// VDOM RENDERING SHADERS FOR WEBGL2
// ============================================

struct VDomVertex {
    position: vec2<f32>,
    uv: vec2<f32>,
    color: vec4<f32>,
}

struct VDomUniforms {
    transform: mat3x2<f32>,
    resolution: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> vdom_uniforms: VDomUniforms;

@vertex
fn vs_vdom_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) color: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    // Apply 2D transform matrix (translation, scale, rotation)
    let pos = vdom_uniforms.transform * vec3<f32>(position, 1.0);
    out.clip_position = vec4<f32>(pos.x, pos.y, 0.0, 1.0);
    out.uv = uv;
    out.world_pos = pos.xy;
    out.screen_pos = pos.xy * vdom_uniforms.resolution;
    return out;
}

@fragment
fn fs_vdom_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // VDOM rendering: use procedural cyberpunk styling with uv-driven color
    // This creates a neon grid effect suitable for VDOM elements
    let uv = in.uv;
    let time = scene.time;
    
    // Generate cyberpunk-style color based on UV position
    let grid_size = 10.0;
    let grid = step(0.9, fract(uv.x * grid_size)) + step(0.9, fract(uv.y * grid_size));
    
    // Animated pulse effect
    let pulse = 0.5 + 0.5 * sin(time * 3.0 + uv.x * 5.0);
    
    // Base neon color (cyan/blue cyberpunk palette)
    let base_color = vec4<f32>(0.0, 0.8, 1.0, 1.0);
    
    // Add grid lines and pulsing glow
    var color = base_color * (0.5 + grid * 0.5) * pulse;
    
    // Add subtle scanline effect
    let scanline = sin(uv.y * 50.0 + time * 10.0) * 0.1;
    color = vec4<f32>(color.r + scanline * 0.1, color.g + scanline * 0.2, color.b + scanline * 0.3, color.a);
    
    return color;
}

// ============================================
// WEBGPU COMPUTE SHADER FOR SCENE PROCESSING
// ============================================

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.node_count) {
        return;
    }
    
    // Load node data (using array indexing pattern)
    // In real implementation, this would use structured buffer access
    let base_offset = id.x * 32u; // sizeof(SceneNode) bytes
    
    // Perform scene processing:
    // 1. Culling - nodes outside viewport are flagged
    // 2. Animation - update animation phases
    // 3. Transform - apply parent transforms
    
    // Simple animation update
    let phase = params.time * 0.5 + f32(id.x) * 0.1;
    let pulse = 0.5 + 0.5 * sin(phase);
    
    // This is a placeholder - real implementation would update node properties
    // The results would be written back to the node buffer
}

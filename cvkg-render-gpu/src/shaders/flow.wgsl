struct FlowVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) width: f32,
    @location(4) flow_speed: f32,
    @location(5) pulse_color: vec4<f32>,
};

struct FlowVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) width: f32,
    @location(3) flow_speed: f32,
    @location(4) pulse_color: vec4<f32>,
};

struct SceneUniforms {
    resolution: vec2<f32>,
    time: f32,
    scale_factor: f32,
};

@group(0) @binding(0) var<uniform> scene: SceneUniforms;

@vertex
fn vs_main(in: FlowVertexInput) -> FlowVertexOutput {
    var out: FlowVertexOutput;
    
    // Transform from canvas space (or world space) to NDC
    // For simplicity, assuming a standard orthographic projection happens here
    let aspect = scene.resolution.x / scene.resolution.y;
    
    // Normalize position based on resolution (simplified)
    let ndc_x = (in.position.x / scene.resolution.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.position.y / scene.resolution.y) * 2.0;
    
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.width = in.width;
    out.flow_speed = in.flow_speed;
    out.pulse_color = in.pulse_color;
    
    return out;
}

@fragment
fn fs_main(in: FlowVertexOutput) -> @location(0) vec4<f32> {
    let t = in.uv.x; // along the curve
    let v = in.uv.y; // across the curve

    // Base color of the wire
    var color = in.color;

    // Glowing data pulses
    if in.flow_speed > 0.0 {
        // Calculate pulse intensity based on time, distance along curve, and speed
        let pulse_frequency = 10.0;
        let pulse_phase = scene.time * in.flow_speed;
        let wave = sin(t * pulse_frequency - pulse_phase) * 0.5 + 0.5;
        
        // Sharpen the pulse to look like a discrete energy packet
        let pulse = pow(wave, 8.0);
        
        // Add the glowing pulse color
        color += in.pulse_color * pulse;
    }

    // Edge anti-aliasing / soft falloff
    // Distance from the center of the wire (0.5 is center)
    let dist_from_center = abs(v - 0.5) * 2.0;
    let alpha_falloff = smoothstep(1.0, 0.8, dist_from_center);

    return vec4<f32>(color.rgb, color.a * alpha_falloff);
}

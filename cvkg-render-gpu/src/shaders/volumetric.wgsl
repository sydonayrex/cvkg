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

struct VolumetricUniforms {
    time: f32,
    resolution: vec2<f32>,
    light_pos: vec3<f32>,
    light_color: vec3<f32>,
    density: f32,
    falloff: f32,
    msaa_count: f32,
    _pad1: f32,
    // -- Hologram extension --
    holo_rect: vec4<f32>,   // x, y, width, height in logical pixels
    holo_id_hash: f32,
    holo_time: f32,
    holo_count: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> uniforms: VolumetricUniforms;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var depth_texture_msaa: texture_depth_multisampled_2d;
@group(0) @binding(3) var depth_sampler: sampler_comparison;

@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VertexOutput {
    // Full-screen triangle (no vertex buffer needed)
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // If no hologram instances are active, output transparent (no contribution).
    if (uniforms.holo_count < 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Convert UV from [0,1] to logical pixel coordinates.
    let logical_uv = in.uv * uniforms.resolution;

    // Read scene depth at this fragment to occlude volumetric effects behind geometry.
    var scene_depth: f32 = 1.0;
    if (uniforms.msaa_count > 1.5) {
        // MSAA depth: manually resolve sample 0 via textureLoad
        let coord = vec2<i32>(floor(in.uv * uniforms.resolution));
        scene_depth = textureLoad(depth_texture_msaa, coord, 0);
    } else {
        // Single-sample depth
        scene_depth = textureSampleCompare(depth_texture, depth_sampler, in.uv, 0.5);
    }

    // If scene geometry is in front of the volumetric plane, discard.
    // Volumetric renders at depth 0.5 (mid-range); if scene is closer, skip.
    if (scene_depth < 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Rect-constrained rendering: discard fragments outside the hologram bounding box.
    let rect_min = uniforms.holo_rect.xy;
    let rect_max = uniforms.holo_rect.xy + uniforms.holo_rect.zw;
    let margin = 1.0;
    let inside = all(logical_uv >= rect_min - vec2<f32>(margin)) && all(logical_uv <= rect_max + vec2<f32>(margin));
    if (!inside) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Compute UV relative to the rect center, normalized to rect dimensions.
    let rect_center = (rect_min + rect_max) * 0.5;
    let rect_half = uniforms.holo_rect.zw * 0.5;
    let local_uv = (logical_uv - rect_center) / max(rect_half, vec2<f32>(0.001));

    // Aspect-corrected local UV for SDF operations
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let local_uv_aspect = vec2<f32>(local_uv.x * aspect, local_uv.y);

    // Per-hologram variation: derive pulsation frequency from id_hash.
    // This creates visually distinct holograms even with the same time.
    let id_freq = 0.3 + fract(uniforms.holo_id_hash * 0.000001) * 0.7;

    // Animated SDF: pulsating sphere with noise-like distortion
    let time = uniforms.holo_time;
    let pulse = sin(time * id_freq * 2.0) * 0.1 + 0.9;
    let dist = length(local_uv_aspect) - 0.5 * pulse;

    // Light shaft effect: directional glow from light position
    let light_dir = normalize(vec2<f32>(
        local_uv.x - uniforms.light_pos.x,
        local_uv.y - uniforms.light_pos.y
    ));
    let shaft = max(0.0, dot(normalize(local_uv_aspect), light_dir));
    let shaft_intensity = pow(shaft, 3.0) * 0.3;

    // Per-hologram color tint based on id_hash
    let hue_shift = fract(uniforms.holo_id_hash * 0.000003);
    let base_color = mix(vec3<f32>(0.0, 0.8, 1.0), vec3<f32>(0.8, 0.0, 1.0), hue_shift);

    // Edge feathering within the rect for smooth falloff
    let edge_x = smoothstep(0.0, margin / max(rect_half.x, 0.001), min(local_uv.x + 1.0, 1.0 - local_uv.x));
    let edge_y = smoothstep(0.0, margin / max(rect_half.y, 0.001), min(local_uv.y + 1.0, 1.0 - local_uv.y));
    let edge_feather = edge_x * edge_y;

    if (dist < 0.0) {
        // Inside the volume: emit colored light
        let raw_density = (1.0 + dist * 2.0) * uniforms.density;
        let density = clamp(raw_density, 0.0, 1.0);
        let color = mix(base_color, uniforms.light_color, shaft_intensity);
        return vec4<f32>(color * density, 0.6 * density) * edge_feather;
    } else {
        // Outside the volume: soft glow falloff with light shaft
        let glow = uniforms.falloff / max(dist, 0.01);
        let color = mix(base_color, uniforms.light_color, shaft_intensity);
        return vec4<f32>(color * glow, glow) * edge_feather;
    }
}

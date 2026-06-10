
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct EffectParams {
    time: f32,
    pad0: f32,
    size: vec2<f32>,
    args: array<f32, 16>,
};

@group(0) @binding(0) var t_layer: texture_2d<f32>;
@group(0) @binding(1) var s_layer: sampler;
@group(1) @binding(0) var<uniform> params: EffectParams;

fn mod_f32(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

fn mod_vec3(x: vec3<f32>, y: vec3<f32>) -> vec3<f32> {
    return x - y * floor(x / y);
}

// SwiftUIShaders, a collection of SwiftUI Metal effects.
// Extracted from the Epilogue reading app's shader lab. MIT licensed.
// Each [[ stitchable ]] function is a SwiftUI layerEffect. See README / ShaderEffects.swift
// for the typed Swift wrappers. The `bcs_` prefix is just the shader namespace.




// MARK: - Shared Noise Utilities
// Prefixed with bcs_ to avoid symbol collisions

fn bcs_hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn bcs_valueNoise(st: vec2<f32>) -> f32 {
    var i = floor(st);
    var f = fract(st);
    var u = f * f * (3.0 - 2.0 * f);

    var a = bcs_hash(i);
    var b = bcs_hash(i + vec2<f32>(1.0, 0.0));
    var c = bcs_hash(i + vec2<f32>(0.0, 1.0));
    var d = bcs_hash(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn bcs_fbm(st: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var i = 0; i < octaves; i++) {
        value += amplitude * bcs_valueNoise(st * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

// HSB to RGB
fn bcs_hsb2rgb(c: vec3<f32>) -> vec3<f32> {
    var rgb = clamp(
        abs(mod_vec3(c.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0), vec3<f32>(6.0)) - vec3<f32>(3.0)) - vec3<f32>(1.0),
        vec3<f32>(0.0), vec3<f32>(1.0)
    );
    rgb = rgb * rgb * (3.0 - 2.0 * rgb);
    return c.z * mix(vec3<f32>(1.0), rgb, c.y);
}

// MARK: - 1. Emboss / Relief
// Creates a 3D carved look from the cover art using edge detection

@fragment
fn bcs_emboss(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var strength = params.args[0];
    var angle = params.args[1];
    var mix_amount = params.args[2];

    var dir = vec2<f32>(cos(angle), sin(angle));
    var offset = 1.5; // pixel offset for edge detection

    // Sample neighbors along light direction
    var ahead = textureSample(t_layer, s_layer, position + dir * offset / params.size);
    var behind = textureSample(t_layer, s_layer, position - dir * offset / params.size);
    var center = textureSample(t_layer, s_layer, position / params.size);

    // Luminance of neighbors
    var lumAhead = dot(vec3<f32>(ahead.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lumBehind = dot(vec3<f32>(behind.rgb), vec3<f32>(0.299, 0.587, 0.114));

    // Height difference = emboss
    var emboss = (lumAhead - lumBehind) * strength;

    // Apply emboss to original color
    var embossed = center;
    embossed = vec4<f32>(embossed.rgb + (f32(emboss)), embossed.a);

    return mix(center, embossed, f32(mix_amount));
}

// MARK: - 2. Heat Shimmer
// Animated wavering distortion like heat rising off pavement

@fragment
fn bcs_heatShimmer(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var amplitude = params.args[0];
    var frequency = params.args[1];
    var speed = params.args[2];
    var vertical_bias = params.args[3];

    var uv = position / size;

    // Vertical bias: stronger shimmer toward the top
    var bias = mix(1.0, 1.0 - uv.y, vertical_bias);

    // Two sine waves at different frequencies for organic feel
    var wave1 = sin(uv.y * frequency + time * speed) * amplitude * bias;
    var wave2 = sin(uv.y * frequency * 1.7 + time * speed * 0.8 + 2.0) * amplitude * 0.5 * bias;

    // Add subtle vertical displacement too
    var waveY = cos(uv.x * frequency * 0.5 + time * speed * 1.2) * amplitude * 0.3 * bias;

    var displaced = position + vec2<f32>(wave1 + wave2, waveY);

    // Keep in bounds
    displaced = clamp(displaced, vec2<f32>(0.0), size);

    return textureSample(t_layer, s_layer, displaced / params.size);
}

// MARK: - 3. Holographic / Prismatic
// Rainbow foil effect that shifts with time, like a holographic trading card

@fragment
fn bcs_holographic(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var scale = params.args[1];
    var speed = params.args[2];
    var angle_offset = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    // Diagonal position for rainbow bands
    var diagonal = (uv.x * cos(angle_offset) + uv.y * sin(angle_offset)) * scale;

    // Animated rainbow using phase-offset sine waves
    var phase = diagonal + time * speed;
    var rainbow: vec3<f32>;
    rainbow.r = sin(phase) * 0.5 + 0.5;
    rainbow.g = sin(phase + 2.094) * 0.5 + 0.5;  // 2pi/3
    rainbow.b = sin(phase + 4.189) * 0.5 + 0.5;  // 4pi/3

    // Luminance-driven: brighter areas catch more "light"
    var lum = dot(vec3<f32>(original.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var hologramMask = smoothstep(0.3, 0.8, lum);

    // Additive blend weighted by luminance
    var result = original;
    result = vec4<f32>(result.rgb + (rainbow * f32(intensity * hologramMask)), result.a);

    // Subtle saturation boost
    var gray = dot(result.rgb, vec3<f32>(0.299, 0.587, 0.114));
    result = vec4<f32>(mix(vec3<f32>(gray), result.rgb, 1.1), result.a);

    return result;
}

// MARK: - 4. Ink Bleed / Domain Warp
// Makes the cover look like watercolor bleeding into wet paper

@fragment
fn bcs_inkBleed(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var warp_strength = params.args[0];
    var scale = params.args[1];
    var speed = params.args[2];
    var detail = params.args[3];

    var uv = position / size;

    // Domain warping: noise feeding into noise
    var st = uv * scale;

    var q = vec2<f32>(
        bcs_fbm(st + vec2<f32>(time * speed * 0.1, 0.0), i32(detail)),
        bcs_fbm(st + vec2<f32>(5.2, 1.3 + time * speed * 0.08), i32(detail))
    );

    var r = vec2<f32>(
        bcs_fbm(st + 4.0 * q + vec2<f32>(1.7, 9.2) + time * speed * 0.05, i32(detail)),
        bcs_fbm(st + 4.0 * q + vec2<f32>(8.3, 2.8) + time * speed * 0.04, i32(detail))
    );

    // Final warp offset
    var warpOffset = (q + r) * warp_strength;

    var displaced = position + warpOffset;
    displaced = clamp(displaced, vec2<f32>(0.0), size);

    return textureSample(t_layer, s_layer, displaced / params.size);
}

// MARK: - 5. Frosted Glass
// Partial blur with a clear window, like breathing on cold glass

@fragment
fn bcs_frosted(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var frost_amount = params.args[0];
    var grain_size = params.args[1];
    var clear_radius = params.args[2];
    var clear_softness = params.args[3];

    var uv = position / size;

    // Clear window in the center
    var center = vec2<f32>(0.5, 0.5);
    var dist = distance(uv, center);
    var frost_mask = smoothstep(clear_radius, clear_radius + clear_softness, dist);
    frost_mask *= frost_amount;

    // Frost displacement: scatter sampling based on noise
    var noise_uv = uv * grain_size;
    var nx = bcs_hash(floor(noise_uv) + vec2<f32>(0.0, 0.0)) * 2.0 - 1.0;
    var ny = bcs_hash(floor(noise_uv) + vec2<f32>(7.3, 3.1)) * 2.0 - 1.0;

    // Multi-sample for blur approximation (5 taps)
    var scatter = frost_mask * 8.0;
    var sum = textureSample(t_layer, s_layer, position / params.size);
    sum += textureSample(t_layer, s_layer, position + vec2<f32>(nx, ny / params.size) * scatter);
    sum += textureSample(t_layer, s_layer, position + vec2<f32>(-ny, nx / params.size) * scatter);
    sum += textureSample(t_layer, s_layer, position + vec2<f32>(-nx, -ny / params.size) * scatter * 0.7);
    sum += textureSample(t_layer, s_layer, position + vec2<f32>(ny, -nx / params.size) * scatter * 0.7);
    sum /= 5.0;

    // Blend between sharp original and frosted
    var original = textureSample(t_layer, s_layer, position / params.size);
    var result = mix(original, sum, f32(frost_mask));

    // Add subtle frost brightness
    result = vec4<f32>(result.rgb + (vec3<f32>(frost_mask * 0.05)), result.a);

    return result;
}

// MARK: - 6. Chromatic Split
// RGB channel separation with directional control

@fragment
fn bcs_chromaticSplit(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var spread = params.args[0];
    var angle = params.args[1];
    var edge_only = params.args[2];
    var time = params.time;
    var animate = params.args[3];

    var uv = position / size;

    // Optional edge-only mask
    var center = vec2<f32>(0.5, 0.5);
    var dist = distance(uv, center);
    var mask = mix(1.0, smoothstep(0.1, 0.5, dist), edge_only);

    // Animated spread
    var animatedSpread = spread;
    if (animate > 0.01) {
        animatedSpread += sin(time * 2.0) * spread * 0.3 * animate;
    }

    var effectiveSpread = animatedSpread * mask;

    // Direction vector
    var dir = vec2<f32>(cos(angle), sin(angle)) * effectiveSpread;

    // Sample each channel at offset positions
    var r = textureSample(t_layer, s_layer, position + dir / params.size);
    var g = textureSample(t_layer, s_layer, position / params.size);
    var b = textureSample(t_layer, s_layer, position - dir / params.size);

    return vec4<f32>(r.r, g.g, b.b, g.a);
}

// MARK: - 7. Live Ripple
// Concentric water ripples expanding outward continuously from center

@fragment
fn bcs_liveRipple(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var amplitude = params.args[0];
    var frequency = params.args[1];
    var speed = params.args[2];
    var damping = params.args[3];
    var ring_count = params.args[4];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var aspectRatio = size.x / size.y;

    var totalOffset = vec2<f32>(0.0);

    for (var i = 0; i < i32(ring_count); i++) {
        // Each ring source has a slight offset and phase
        var phase = f32(i) * 1.256; // 2pi/5 spacing
        var ringCenter = center + vec2<f32>(
            sin(time * 0.3 + phase) * 0.05,
            cos(time * 0.4 + phase) * 0.05
        );

        var delta = uv - ringCenter;
        delta.x *= aspectRatio;
        var dist = length(delta);

        // Expanding concentric rings
        var wave = sin(dist * frequency - time * speed + phase);

        // Fade with distance
        var envelope = exp(-dist * damping);

        // Radial displacement direction
        var dir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
        dir.x /= aspectRatio;

        totalOffset += dir * wave * envelope * amplitude / ring_count;
    }

    var displaced = clamp(position + totalOffset, vec2<f32>(0.0), size);
    return textureSample(t_layer, s_layer, displaced / params.size);
}

// MARK: - 8. Touch Ripple
// Ripples expand from a touch point, decay over time

@fragment
fn bcs_touchRipple(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var touchPos = vec2<f32>(params.args[0], params.args[1]);
    var touchAge = params.args[2];
    var amplitude = params.args[3];
    var frequency = params.args[4];
    var speed = params.args[5];
    var decay = params.args[6];

    if (touchAge < 0.01 || touchAge > 5.0) {
        return textureSample(t_layer, s_layer, position / params.size);
    }

    var delta = position - touchPos;
    var dist = length(delta);

    // Expanding wavefront
    var rippleRadius = touchAge * speed;
    var distFromFront = dist - rippleRadius;

    // Wider, smoother gaussian envelope, more liquid, less sharp
    var waveWidth = 60.0 + touchAge * 40.0;
    var envelope = exp(-(distFromFront * distFromFront) / (2.0 * waveWidth * waveWidth));

    // Time fade
    var timeFade = exp(-touchAge * decay);

    // Multiple layered sine waves for smoother, more organic ripples
    var wave1 = sin(distFromFront * frequency * 0.008);
    var wave2 = sin(distFromFront * frequency * 0.005 + 1.0) * 0.5;
    var wave = (wave1 + wave2) * 0.67 * envelope * timeFade * amplitude;

    // Smooth radial direction
    var dir = select(vec2<f32>(0.0), normalize(delta), dist > 0.5);

    var displaced = clamp(position + dir * wave, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, displaced / params.size);

    // Subtle chromatic shift on the ripple
    var chromaAmt = abs(wave) * 0.08;
    var rSamp = textureSample(t_layer, s_layer, clamp(displaced + dir * chromaAmt, vec2<f32>(0.0 / params.size), size));
    var bSamp = textureSample(t_layer, s_layer, clamp(displaced - dir * chromaAmt, vec2<f32>(0.0 / params.size), size));
    color.r = mix(color.r, rSamp.r, f32(envelope * timeFade * 0.3));
    color.b = mix(color.b, bSamp.b, f32(envelope * timeFade * 0.3));

    return color;
}

// MARK: - 9. Liquid Chrome
// Metallic mercury reflection with animated highlights

@fragment
fn bcs_liquidChrome(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var distortion = params.args[0];
    var chrome_intensity = params.args[1];
    var flow_speed = params.args[2];
    var reflection_scale = params.args[3];

    var uv = position / size;

    // Flowing noise field for displacement
    var st = uv * reflection_scale;
    var n1 = bcs_fbm(st + vec2<f32>(time * flow_speed * 0.2, time * flow_speed * 0.15), 4);
    var n2 = bcs_fbm(st + vec2<f32>(5.0, 3.0) + vec2<f32>(time * flow_speed * 0.18, time * flow_speed * 0.22), 4);

    // Displacement
    var offset = vec2<f32>(n1, n2) * distortion;
    var displaced = clamp(position + offset, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, displaced / params.size);

    // Chrome specular highlights based on noise gradient
    var epsilon = 0.01;
    var h0 = bcs_fbm(st + vec2<f32>(time * flow_speed * 0.2, time * flow_speed * 0.15), 3);
    var hx = bcs_fbm(st + vec2<f32>(epsilon, 0.0) + vec2<f32>(time * flow_speed * 0.2, time * flow_speed * 0.15), 3);
    var hy = bcs_fbm(st + vec2<f32>(0.0, epsilon) + vec2<f32>(time * flow_speed * 0.2, time * flow_speed * 0.15), 3);

    // Surface normal from height field
    var normal = normalize(vec3<f32>((h0 - hx) / epsilon, (h0 - hy) / epsilon, 1.0));

    // Specular: how much the surface faces the "camera"
    var specular = pow(max(normal.z, 0.0), 4.0);

    // Chrome highlight, bright white where surface is angled just right
    var highlight = pow(1.0 - abs(dot(normal, vec3<f32>(0, 0, 1))), 3.0) * chrome_intensity;

    // Desaturate for metallic look, then add highlights
    var lum = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    var metallic = mix(color.rgb, vec3<f32>(lum), f32(chrome_intensity * 0.5));
    metallic += vec3<f32>(highlight);
    metallic *= f32(0.8 + specular * 0.4);

    return vec4<f32>(metallic, color.a);
}

// MARK: - 10. Glitch
// Digital glitch with scan lines, block displacement, and color corruption

@fragment
fn bcs_glitch(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var block_size = params.args[1];
    var scan_lines = params.args[2];
    var color_shift = params.args[3];

    var uv = position / size;

    // Pseudo-random glitch trigger (changes every ~0.1 seconds)
    var glitchTime = floor(time * 10.0);
    var glitchRand = bcs_hash(vec2<f32>(glitchTime, 0.0));

    // Only glitch some of the time
    var glitchActive = step(1.0 - intensity * 0.5, glitchRand);

    // Block displacement
    var blockY = floor(uv.y * (size.y / block_size));
    var blockRand = bcs_hash(vec2<f32>(blockY, glitchTime));
    var blockShift = (blockRand - 0.5) * 2.0 * intensity * glitchActive;

    var displaced = position;
    displaced.x += blockShift * block_size * 2.0;

    // Per-block vertical jitter
    var vertRand = bcs_hash(vec2<f32>(blockY + 100.0, glitchTime));
    if (vertRand > 0.95 && glitchActive > 0.5) {
        displaced.y += (bcs_hash(vec2<f32>(blockY, glitchTime + 50.0)) - 0.5) * block_size;
    }

    displaced = clamp(displaced, vec2<f32>(0.0), size);

    // Color channel separation during glitch
    var shift = color_shift * glitchActive;
    var r = textureSample(t_layer, s_layer, displaced + vec2<f32>(shift, 0.0 / params.size));
    var g = textureSample(t_layer, s_layer, displaced / params.size);
    var b = textureSample(t_layer, s_layer, displaced - vec2<f32>(shift, 0.0 / params.size));

    var result = vec4<f32>(r.r, g.g, b.b, g.a);

    // Scan lines
    var scanLine = sin(position.y * 3.14159 * 2.0) * 0.5 + 0.5;
    scanLine = pow(scanLine, 4.0);
    result = vec4<f32>(result.rgb * (1.0 - f32(scanLine * scan_lines * 0.3)), result.a);

    // Occasional bright flash on glitch blocks
    if (blockRand > 0.92 && glitchActive > 0.5) {
        result = vec4<f32>(result.rgb + (vec3<f32>(0.15)), result.a);
    }

    return result;
}

// MARK: - 11. Vortex Spiral
// Swirling distortion that twists the cover art

@fragment
fn bcs_vortex(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var twist_amount = params.args[0];
    var radius = params.args[1];
    var speed = params.args[2];
    var falloff = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var delta = uv - center;

    var aspectRatio = size.x / size.y;
    delta.x *= aspectRatio;

    var dist = length(delta);

    // Twist angle based on distance from center
    var normalizedDist = dist / radius;
    var twistFalloff = exp(-normalizedDist * falloff);
    var angle = twist_amount * twistFalloff + time * speed;

    // Rotate UV around center
    var cosA = cos(angle);
    var sinA = sin(angle);
    var rotated = vec2<f32>(
        delta.x * cosA - delta.y * sinA,
        delta.x * sinA + delta.y * cosA
    );

    rotated.x /= aspectRatio;
    var newUV = rotated + center;

    var samplePos = clamp(newUV * size, vec2<f32>(0.0), size);
    return textureSample(t_layer, s_layer, samplePos / params.size);
}

// MARK: - 12. Pulse / Heartbeat
// Rhythmic radial expansion and contraction like a breathing cover

@fragment
fn bcs_pulse(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var amplitude = params.args[0];
    var bpm = params.args[1];
    var sharpness = params.args[2];
    var glow_intensity = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var delta = uv - center;
    var dist = length(delta);

    // Heartbeat-like pulse: sharp attack, smooth decay
    var beatFreq = bpm / 60.0;
    var beat = sin(time * beatFreq * 3.14159 * 2.0);
    beat = pow(abs(beat), 1.0 / sharpness) * sign(beat);
    beat = beat * 0.5 + 0.5; // 0-1 range

    // Radial displacement: pushes pixels outward on beat
    var dir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
    var displacement = beat * amplitude * smoothstep(0.0, 0.3, dist);

    var displaced = position + dir * displacement;
    displaced = clamp(displaced, vec2<f32>(0.0), size);

    var color = textureSample(t_layer, s_layer, displaced / params.size);

    // Edge glow on beat
    var edgeDist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    var edgeGlow = (1.0 - smoothstep(0.0, 0.15, edgeDist)) * beat * glow_intensity;
    color = vec4<f32>(color.rgb + (vec3<f32>(edgeGlow * 0.5, edgeGlow * 0.3, edgeGlow * 0.6)), color.a);

    return color;
}

// MARK: - 14. Wave Pool
// Multiple overlapping sine wave displacements creating interference patterns

@fragment
fn bcs_wavePool(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var amplitude = params.args[0];
    var wavelength = params.args[1];
    var speed = params.args[2];
    var complexity = params.args[3];

    var uv = position / size;
    var totalOffset = vec2<f32>(0.0);

    var waves = i32(complexity);

    for (var i = 0; i < waves; i++) {
        var angle = f32(i) * 3.14159 / f32(waves); // evenly spaced angles
        var dir = vec2<f32>(cos(angle), sin(angle));

        // Wave along this direction
        var phase = dot(uv, dir) * wavelength + time * speed + f32(i) * 1.5;
        var wave = sin(phase);

        // Displace perpendicular to wave direction
        var perpDir = vec2<f32>(-dir.y, dir.x);
        totalOffset += perpDir * wave * amplitude / f32(waves);
    }

    var displaced = clamp(position + totalOffset, vec2<f32>(0.0), size);
    return textureSample(t_layer, s_layer, displaced / params.size);
}

// MARK: - 16. Ethereal Aura (v2)
// The cover BREATHES, edges warp and glow with visible liquid displacement

@fragment
fn bcs_etherealAura(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var aura_width = params.args[0];
    var aura_intensity = params.args[1];
    var pulse_speed = params.args[2];
    var distortion = params.args[3];
    var hue_shift = params.args[4];

    var uv = position / size;

    // Edge distance field
    var edgeX = min(uv.x, 1.0 - uv.x);
    var edgeY = min(uv.y, 1.0 - uv.y);
    var edgeDist = min(edgeX, edgeY);

    // Organic edge with domain-warped noise
    var st = uv * 6.0;
    var q = vec2<f32>(
        bcs_fbm(st + vec2<f32>(time * 0.15, time * 0.1), 5),
        bcs_fbm(st + vec2<f32>(5.2, 1.3) + vec2<f32>(time * 0.12, time * 0.18), 5)
    );
    var edgeWarp = bcs_fbm(st + 3.0 * q, 4);

    var auraMask = smoothstep(aura_width + edgeWarp * aura_width, 0.0, edgeDist);

    // Breathing pulse
    var pulse = 0.6 + 0.4 * sin(time * pulse_speed);
    var pulsedMask = auraMask * pulse;

    // HEAVY displacement everywhere, strongest at edges
    var dispSt = uv * 4.0;
    var dispQ = vec2<f32>(
        bcs_fbm(dispSt + vec2<f32>(time * 0.25, time * 0.2), 5),
        bcs_fbm(dispSt + vec2<f32>(3.0, 7.0) + vec2<f32>(time * 0.2, time * 0.3), 5)
    );
    var dispR = vec2<f32>(
        bcs_fbm(dispSt + 3.0 * dispQ + vec2<f32>(time * 0.1, 0.0), 4),
        bcs_fbm(dispSt + 3.0 * dispQ + vec2<f32>(0.0, time * 0.08), 4)
    );

    // Displacement pushes inward from edges + organic wander
    var edgeDir = vec2<f32>(
        select(-1.0, 1.0, uv.x < 0.5),
        select(-1.0, 1.0, uv.y < 0.5)
    );
    var disp = vec2<f32>(dispR.x - 0.5, dispR.y - 0.5) * distortion * pulsedMask;
    disp += edgeDir * pulsedMask * distortion * 0.3; // push inward

    // Apply displacement across the WHOLE image, fading from edges
    var globalDisp = smoothstep(0.3, 0.0, edgeDist);
    disp *= globalDisp;

    var displaced = clamp(position + disp, vec2<f32>(0.0), size);

    // Chromatic aberration at edges
    var chromaAmount = pulsedMask * distortion * 0.12;
    var chromaDir = normalize(vec2<f32>(uv.x - 0.5, uv.y - 0.5) + 0.001) * chromaAmount;

    var rr = textureSample(t_layer, s_layer, clamp(displaced + chromaDir, vec2<f32>(0.0 / params.size), size));
    var gg = textureSample(t_layer, s_layer, displaced / params.size);
    var bb = textureSample(t_layer, s_layer, clamp(displaced - chromaDir, vec2<f32>(0.0 / params.size), size));
    var color = vec4<f32>(rr.r, gg.g, bb.b, gg.a);

    // Aura glow color
    var auraColor: vec3<f32>;
    auraColor.r = sin(f32(hue_shift)) * 0.5 + 0.5;
    auraColor.g = sin(f32(hue_shift) + 2.094) * 0.5 + 0.5;
    auraColor.b = sin(f32(hue_shift) + 4.189) * 0.5 + 0.5;

    var glow = pulsedMask * aura_intensity;
    color = vec4<f32>(color.rgb + (auraColor * f32(glow * 0.6)), color.a);
    color = vec4<f32>(color.rgb + (vec3<f32>(glow * 0.15)), color.a); // white bloom

    return color;
}

// MARK: - 17. Black Hole
// Gravitational lensing, warps space around a singularity

@fragment
fn bcs_blackHole(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var mass = params.args[0];
    var spin = params.args[1];
    var distortion = params.args[2];
    var ring_brightness = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var aspectRatio = size.x / size.y;

    var delta = uv - center;
    delta.x *= aspectRatio;
    var dist = length(delta);
    var angle = atan2(delta.y, delta.x);

    // Gravitational lensing: bend light around the mass
    // Closer to the singularity = more bending
    var schwarzschild = mass * 0.3;
    var bendStrength = schwarzschild / max(dist * dist, 0.001);
    bendStrength = min(bendStrength, 5.0); // cap it

    // Warp UV: push pixels radially outward near the hole
    var warpDir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
    var warped = delta + warpDir * bendStrength * 0.1;

    // Add rotational frame-dragging
    var dragAngle = spin * schwarzschild / max(dist, 0.01) * time;
    var cosD = cos(dragAngle);
    var sinD = sin(dragAngle);
    warped = vec2<f32>(warped.x * cosD - warped.y * sinD,
                    warped.x * sinD + warped.y * cosD);

    warped.x /= aspectRatio;
    var sampleUV = (warped + center) * size;
    sampleUV = clamp(sampleUV, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, sampleUV / params.size);

    // Event horizon: fade to black inside schwarzschild radius
    var horizon = smoothstep(schwarzschild * 0.5, schwarzschild * 1.5, dist);
    color = vec4<f32>(color.rgb * (f32(horizon)), color.a);

    // Accretion disk: bright ring around the black hole
    var ringDist = abs(dist - schwarzschild * 2.5);
    var ring = exp(-ringDist * ringDist / (schwarzschild * schwarzschild * 0.3));

    // Rotating ring pattern
    var ringPattern = sin(angle * 8.0 - time * spin * 3.0) * 0.5 + 0.5;
    ringPattern = pow(ringPattern, 2.0);
    ring *= (0.5 + ringPattern * 0.5);

    // Ring color: hot blue-white inner, orange outer
    var innerRing = vec3<f32>(0.7, 0.85, 1.0);
    var outerRing = vec3<f32>(1.0, 0.6, 0.2);
    var ringPos = smoothstep(schwarzschild * 1.5, schwarzschild * 4.0, dist);
    var ringColor = mix(innerRing, outerRing, f32(ringPos));

    color = vec4<f32>(color.rgb + (ringColor * f32(ring * ring_brightness)), color.a);

    return color;
}

// MARK: - 18. Melt
// The image melts downward like hot wax, gravity pulls pixels down

@fragment
fn bcs_melt(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var melt_amount = params.args[0];
    var drip_scale = params.args[1];
    var speed = params.args[2];
    var heat = params.args[3];

    var uv = position / size;

    // Per-column drip amount, each vertical strip melts at different rate
    var column = uv.x * drip_scale;
    var dripNoise = bcs_fbm(vec2<f32>(column, time * speed * 0.3), 4);
    var dripNoise2 = bcs_fbm(vec2<f32>(column * 1.7 + 3.0, time * speed * 0.25), 3);

    // Drip amount increases toward the bottom
    var gravity = uv.y * uv.y; // quadratic, bottom melts more
    var drip = (dripNoise * 0.7 + dripNoise2 * 0.3) * melt_amount * gravity;

    // Add some horizontal wobble as things melt
    var wobble = sin(uv.y * 10.0 + time * speed * 2.0 + dripNoise * 5.0) * melt_amount * 0.05 * gravity;

    var displaced = position + vec2<f32>(wobble, -drip); // negative Y = pull up = melt down
    displaced = clamp(displaced, vec2<f32>(0.0), size);

    var color = textureSample(t_layer, s_layer, displaced / params.size);

    // Heat distortion: warm color shift in melting areas
    var meltFactor = drip / max(melt_amount, 1.0);
    color.r += f32(meltFactor * heat * 0.3);
    color.g -= f32(meltFactor * heat * 0.1);
    color.b -= f32(meltFactor * heat * 0.2);

    // Slight brightening at drip edges (specular on liquid)
    var dripEdge = abs(bcs_fbm(vec2<f32>(column + 0.01, time * speed * 0.3), 4) - dripNoise);
    var specular = pow(dripEdge * 5.0, 3.0) * gravity * 0.4;
    color = vec4<f32>(color.rgb + (vec3<f32>(specular)), color.a);

    return color;
}

// MARK: - 19. Kaleidoscope
// Mirrors and rotates the cover into mesmerizing symmetrical patterns

@fragment
fn bcs_kaleidoscope(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var segments = params.args[0];
    var rotation = params.args[1];
    var zoom = params.args[2];
    var animate_speed = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var delta = uv - center;

    var aspectRatio = size.x / size.y;
    delta.x *= aspectRatio;

    // Polar coordinates
    var angle = atan2(delta.y, delta.x) + rotation + time * animate_speed;
    var dist = length(delta);

    // Kaleidoscope: fold angle into segment
    var segAngle = 3.14159 * 2.0 / segments;
    angle = angle - segAngle * floor(angle / segAngle); // mod into segment
    if (angle > segAngle * 0.5) {
        angle = segAngle - angle; // mirror
    }

    // Back to cartesian
    var kaleido = vec2<f32>(cos(angle), sin(angle)) * dist / zoom;
    kaleido.x /= aspectRatio;
    var sampleUV = (kaleido + center) * size;
    sampleUV = clamp(sampleUV, vec2<f32>(0.0), size);

    return textureSample(t_layer, s_layer, sampleUV / params.size);
}

// MARK: - 21. Refract Lens (Interactive)
// Thick glass sphere, drag to move the lens around the cover

@fragment
fn bcs_refractLens(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var touch_pos = vec2<f32>(params.args[0], params.args[1]);
    var lens_radius = params.args[2];
    var refraction = params.args[3];
    var aberration = params.args[4];
    var wobble = params.args[5];

    var uv = position / size;
    var aspectRatio = size.x / size.y;

    // Lens center from touch position (normalized)
    var lensCenter = touch_pos / size;
    // Clamp to valid area
    lensCenter = clamp(lensCenter, vec2<f32>(0.05), vec2<f32>(0.95));

    var delta = uv - lensCenter;
    delta.x *= aspectRatio;
    var dist = length(delta);

    // Soft edge: slight distortion ring outside the lens
    var outerRing = smoothstep(lens_radius * 1.3, lens_radius, dist);
    if (dist > lens_radius * 1.3) {
        return textureSample(t_layer, s_layer, position / params.size);
    }

    if (dist > lens_radius) {
        // Subtle outer distortion ring
        var pushDir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
        pushDir.x /= aspectRatio;
        var pushAmount = outerRing * 8.0;
        var pushed = position + pushDir * pushAmount;
        return textureSample(t_layer, s_layer, clamp(pushed, vec2<f32>(0.0 / params.size), size));
    }

    // Sphere surface normal
    var normalizedDist = dist / lens_radius;
    var z = sqrt(1.0 - normalizedDist * normalizedDist);
    var normal = normalize(vec3<f32>(delta / lens_radius, z));

    // Refraction via Snell's law
    var incident = vec3<f32>(0, 0, -1);
    var eta = 1.0 / refraction;
    var cosI = -dot(normal, incident);
    var sinT2 = eta * eta * (1.0 - cosI * cosI);
    var refracted = eta * incident + (eta * cosI - sqrt(max(0.0, 1.0 - sinT2))) * normal;

    var refractedUV = uv + refracted.xy * lens_radius * 0.5;

    // Chromatic aberration, stronger at edges
    var chroma = aberration * (1.0 - z) * 0.01;
    var chromaDir = normalize(delta + 0.001);
    chromaDir.x /= aspectRatio;

    var rr = textureSample(t_layer, s_layer, clamp((refractedUV + chromaDir * chroma / params.size) * size, vec2<f32>(0.0), size));
    var gg = textureSample(t_layer, s_layer, clamp(refractedUV * size, vec2<f32>(0.0 / params.size), size));
    var bb = textureSample(t_layer, s_layer, clamp((refractedUV - chromaDir * chroma / params.size) * size, vec2<f32>(0.0), size));

    var color = vec4<f32>(rr.r, gg.g, bb.b, 1.0);

    // Specular highlight
    var lightDir = normalize(vec3<f32>(0.3, -0.3, 1.0));
    var halfVec = normalize(lightDir + vec3<f32>(0, 0, 1));
    var spec = pow(max(dot(normal, halfVec), 0.0), 64.0);
    color = vec4<f32>(color.rgb + (vec3<f32>(spec * 0.6)), color.a);

    // Fresnel rim
    var fresnel = pow(1.0 - z, 4.0);
    color = vec4<f32>(color.rgb + (vec3<f32>(fresnel * 0.2)), color.a);

    // Edge ring glow
    var rimGlow = pow(normalizedDist, 6.0) * 0.3;
    color = vec4<f32>(color.rgb + (vec3<f32>(rimGlow * 0.5, rimGlow * 0.6, rimGlow * 0.8)), color.a);

    return color;
}

// MARK: - 22. Plasma
// Electric plasma tendrils crawling across the surface

@fragment
fn bcs_plasma(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var scale = params.args[1];
    var speed = params.args[2];
    var color_mode = params.args[3];

    var uv = position / size;
    var color = textureSample(t_layer, s_layer, position / params.size);

    // Classic plasma function: sum of sines
    var st = uv * scale;
    var v1 = sin(st.x + time * speed);
    var v2 = sin(st.y + time * speed * 0.7);
    var v3 = sin(st.x + st.y + time * speed * 0.5);
    var v4 = sin(length(st - vec2<f32>(scale * 0.5)) + time * speed * 1.3);

    var plasma = (v1 + v2 + v3 + v4) * 0.25; // -1 to 1

    // Sharp plasma lines from the zero-crossings
    var lines = 1.0 / (1.0 + abs(plasma) * 20.0);
    lines = pow(lines, 2.0);

    // Secondary tendrils
    var v5 = sin(st.x * 2.0 - st.y * 1.5 + time * speed * 0.9);
    var v6 = sin(length(st - vec2<f32>(scale * 0.3, scale * 0.7)) * 2.0 + time * speed);
    var plasma2 = (v5 + v6) * 0.5;
    var lines2 = 1.0 / (1.0 + abs(plasma2) * 15.0);
    lines2 = pow(lines2, 2.0);

    var totalPlasma = (lines + lines2 * 0.5) * intensity;

    // Plasma color based on mode
    var plasmaColor: vec3<f32>;
    if (color_mode < 0.33) {
        // Electric blue
        plasmaColor = vec3<f32>(0.3, 0.6, 1.0);
    } else if (color_mode < 0.66) {
        // Matrix green
        plasmaColor = vec3<f32>(0.2, 1.0, 0.4);
    } else {
        // Arcane purple
        plasmaColor = vec3<f32>(0.8, 0.2, 1.0);
    }

    // Add plasma glow
    color = vec4<f32>(color.rgb + (plasmaColor * f32(totalPlasma)), color.a);

    // Brighten where plasma is strongest
    color = vec4<f32>(color.rgb + (vec3<f32>(totalPlasma * 0.3)), color.a);

    return color;
}

// MARK: - 23. Echo / Ghost
// Multiple trailing offset copies that create a spectral echo

@fragment
fn bcs_echo(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var echo_count = params.args[0];
    var spread = params.args[1];
    var direction = params.args[2];
    var fade = params.args[3];

    var base = textureSample(t_layer, s_layer, position / params.size);
    var result = base;

    var dir = vec2<f32>(cos(direction), sin(direction)) * spread;
    var echoes = i32(echo_count);

    var totalWeight = 1.0;

    for (var i = 1; i <= echoes; i++) {
        var weight = pow(fade, f32(i));
        var offset = dir * f32(i);

        // Add slight organic wobble to each echo
        offset.x += sin(time * 2.0 + f32(i) * 1.5) * spread * 0.1;
        offset.y += cos(time * 1.7 + f32(i) * 2.0) * spread * 0.1;

        var samplePos = clamp(position - offset, vec2<f32>(0.0), size);
        var echo = textureSample(t_layer, s_layer, samplePos / params.size);

        // Tint echoes: shift toward blue/purple with distance
        echo.r *= f32(1.0 - f32(i) * 0.08);
        echo.b *= f32(1.0 + f32(i) * 0.05);

        result = vec4<f32>(result.rgb + (echo.rgb * f32(weight)), result.a);
        totalWeight += weight;
    }

    result = vec4<f32>(result.rgb / (f32(totalWeight)), result.a);
    return result;
}

// MARK: - 24. Shatter
// Refined glass shard explosion with depth, reflections, and shadow

@fragment
fn bcs_shatter(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var shard_count = params.args[0];
    var explode = params.args[1];
    var rotation_amt = params.args[2];
    var edge_glow = params.args[3];

    var uv = position / size;

    // Voronoi for shard geometry
    var cellUV = uv * shard_count;
    var cellID = floor(cellUV);
    var cellF = fract(cellUV);

    var minDist = 10.0;
    var secondDist = 10.0;
    var closestCell = vec2<f32>(0.0);

    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            var neighbor = vec2<f32>(f32(i), f32(j));
            var id = cellID + neighbor;
            var point = vec2<f32>(
                bcs_hash(id),
                bcs_hash(id + vec2<f32>(37.0, 91.0))
            );
            var diff = neighbor + point - cellF;
            var d = length(diff);
            if (d < minDist) {
                secondDist = minDist;
                minDist = d;
                closestCell = id;
            } else if (d < secondDist) {
                secondDist = d;
            }
        }
    }

    // Per-shard deterministic randoms
    var shardRand = bcs_hash(closestCell * 7.3);
    var shardRand2 = bcs_hash(closestCell * 13.7 + vec2<f32>(5.0, 3.0));
    var shardRand3 = bcs_hash(closestCell * 23.1 + vec2<f32>(11.0, 7.0));

    // Eased explode for natural feel
    var eased = explode * explode * (3.0 - 2.0 * explode);

    // Shard offset, radial from center with stagger
    var center = vec2<f32>(0.5, 0.5);
    var shardCenter = (closestCell + 0.5) / shard_count;
    var driftDir = normalize(shardCenter - center + vec2<f32>(0.001));
    var driftDist = eased * (0.3 + shardRand * 0.7) * 120.0;

    // 3D rotation per shard (tilt in perspective)
    var angle = (shardRand2 - 0.5) * rotation_amt * eased;
    var tiltX = (shardRand3 - 0.5) * eased * 0.15; // perspective tilt
    var ca = cos(angle);
    var sa = sin(angle);
    var rotatedOffset = vec2<f32>(
        ca * driftDir.x - sa * driftDir.y,
        sa * driftDir.x + ca * driftDir.y
    ) * driftDist;

    // Gravity with slight delay per shard
    var delay = shardRand * 0.3;
    var fallEased = max(eased - delay, 0.0);
    var fallAmount = fallEased * fallEased * 100.0 * (0.4 + shardRand * 0.6);

    var samplePos = position - rotatedOffset + vec2<f32>(0.0, -fallAmount);

    // Perspective scale (shards shrink slightly as they fly away)
    var perspectiveScale = 1.0 - eased * shardRand * 0.15;
    var shardCenterPx = shardCenter * size;
    samplePos = shardCenterPx + (samplePos - shardCenterPx) / perspectiveScale;

    samplePos = clamp(samplePos, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Glass reflection: slight brightness gradient across each shard
    var reflectionGradient = dot(normalize(vec2<f32>(cellF - 0.5)), vec2<f32>(0.5, -0.3));
    var glassReflection = smoothstep(-0.3, 0.5, reflectionGradient) * 0.12 * (1.0 + eased);
    color = vec4<f32>(color.rgb + (vec3<f32>(glassReflection)), color.a);

    // Perspective tilt darkening (shards angled away get darker)
    var tiltDarken = 1.0 - abs(tiltX) * eased * 2.0;
    color = vec4<f32>(color.rgb * (f32(max(tiltDarken, 0.6))), color.a);

    // Refined edge lines, thin, clean, with glow
    var edgeDist = secondDist - minDist;
    var thinEdge = 1.0 - smoothstep(0.0, 0.03, edgeDist); // thin line
    var softEdge = 1.0 - smoothstep(0.0, 0.1, edgeDist);  // soft glow

    // Edge color: cool glass tint
    var edgeColor = vec3<f32>(0.7, 0.85, 1.0) * f32(edge_glow);
    color = vec4<f32>(color.rgb + (edgeColor * f32(thinEdge * 0.8 + softEdge * 0.2)), color.a);

    // Shadow under separated shards
    var shadowDist = eased * 0.03;
    color = vec4<f32>(color.rgb * (f32(1.0 - eased * 0.15 * shardRand)), color.a);

    // Fade shards flying far
    var fadeFactor = 1.0 - eased * shardRand * 0.4;
    color = vec4<f32>(color.rgb * (f32(fadeFactor)), color.a);

    return color;
}

// MARK: - 25. Neon Edge
// Glowing neon contour lines extracted from the image

@fragment
fn bcs_neonEdge(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var edge_strength = params.args[0];
    var glow_amount = params.args[1];
    var color_cycle = params.args[2];
    var mix_original = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    // Sobel edge detection
    var step_x = 1.0;
    var step_y = 1.0;

    var tl = textureSample(t_layer, s_layer, position + vec2<f32>(-step_x, -step_y / params.size));
    var tc = textureSample(t_layer, s_layer, position + vec2<f32>(0, -step_y / params.size));
    var tr = textureSample(t_layer, s_layer, position + vec2<f32>(step_x, -step_y / params.size));
    var ml = textureSample(t_layer, s_layer, position + vec2<f32>(-step_x, 0 / params.size));
    var mr = textureSample(t_layer, s_layer, position + vec2<f32>(step_x, 0 / params.size));
    var bl = textureSample(t_layer, s_layer, position + vec2<f32>(-step_x, step_y / params.size));
    var bc = textureSample(t_layer, s_layer, position + vec2<f32>(0, step_y / params.size));
    var br = textureSample(t_layer, s_layer, position + vec2<f32>(step_x, step_y / params.size));

    // Luminance of each sample
    var ltl = dot(vec3<f32>(tl.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var ltc = dot(vec3<f32>(tc.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var ltr = dot(vec3<f32>(tr.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lml = dot(vec3<f32>(ml.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lmr = dot(vec3<f32>(mr.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lbl = dot(vec3<f32>(bl.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lbc = dot(vec3<f32>(bc.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var lbr = dot(vec3<f32>(br.rgb), vec3<f32>(0.299, 0.587, 0.114));

    var gx = -ltl - 2.0*lml - lbl + ltr + 2.0*lmr + lbr;
    var gy = -ltl - 2.0*ltc - ltr + lbl + 2.0*lbc + lbr;
    var edgeMag = sqrt(gx*gx + gy*gy) * edge_strength;
    edgeMag = clamp(edgeMag, 0.0, 1.0);

    // Neon color cycling based on edge direction and time
    var edgeAngle = atan2(gy, gx);
    var hue = fract(edgeAngle / 6.2832 + time * color_cycle * 0.3 + uv.y * 0.5);
    var neonColor = bcs_hsb2rgb(vec3<f32>(f32(hue), 1.0, 1.0));

    // Glow: power curve on edge magnitude for bloom
    var bloom = pow(edgeMag, 0.7) * glow_amount;

    // Dark background + neon edges
    var darkBG = original.rgb * f32(mix_original * 0.5);
    var neon = neonColor * f32(edgeMag + bloom);

    var result = vec4<f32>(darkBG + neon, original.a);
    return result;
}

// MARK: - 26. Pixelate Storm
// Dynamic mosaic that pulses, shifts, and swirls

@fragment
fn bcs_pixelateStorm(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var pixel_size = params.args[0];
    var storm_amount = params.args[1];
    var swirl = params.args[2];
    var pulse = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);

    // Pulsing pixel size
    var pxSize = pixel_size * (1.0 + sin(time * pulse) * 0.3 * storm_amount);

    // Swirl the UV coordinates
    var delta = uv - center;
    var dist = length(delta);
    var angle = atan2(delta.y, delta.x);
    var swirlAngle = swirl * (1.0 - dist) * sin(time * 0.5);
    var swirledUV = center + dist * vec2<f32>(cos(angle + swirlAngle), sin(angle + swirlAngle));

    // Snap to pixel grid
    var pixelUV = floor(swirledUV * size / pxSize) * pxSize / size;

    // Storm: randomly offset some blocks
    var blockRand = bcs_hash(floor(swirledUV * size / pxSize));
    var stormActive = step(1.0 - storm_amount * 0.8, blockRand);
    var stormOffset = vec2<f32>(
        sin(time * 3.0 + blockRand * 20.0) * storm_amount * pxSize * 0.5,
        cos(time * 2.5 + blockRand * 15.0) * storm_amount * pxSize * 0.5
    ) * stormActive;

    var samplePos = pixelUV * size + stormOffset;
    samplePos = clamp(samplePos, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Scanline overlay for digital feel
    var scanline = sin(position.y * 3.14159 / 2.0) * 0.5 + 0.5;
    color = vec4<f32>(color.rgb * (f32(0.92 + scanline * 0.08)), color.a);

    return color;
}

// MARK: - 27. Shockwave
// Expanding rings of distortion from the center

@fragment
fn bcs_shockwave(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var wave_speed = params.args[0];
    var ring_width = params.args[1];
    var strength = params.args[2];
    var repeat_rate = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var aspectRatio = size.x / size.y;

    var delta = uv - center;
    delta.x *= aspectRatio;
    var dist = length(delta) * size.y; // pixel distance from center

    // Repeating wave
    var cycleTime = mod_f32(time, repeat_rate);
    var waveFront = cycleTime * wave_speed;

    // Ring shape: distance from the wave front
    var ringDist = abs(dist - waveFront);
    var ringMask = 1.0 - smoothstep(0.0, ring_width, ringDist);
    ringMask *= ringMask; // sharpen

    // Fade wave as it expands
    var fadeWithDist = exp(-waveFront * 0.003);
    ringMask *= fadeWithDist;

    // Displacement: push outward along the radial direction
    var dir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
    var disp = dir * ringMask * strength;

    // Second wave slightly behind for depth
    var waveFront2 = max(cycleTime - 0.15, 0.0) * wave_speed * 0.9;
    var ringDist2 = abs(dist - waveFront2);
    var ringMask2 = 1.0 - smoothstep(0.0, ring_width * 0.7, ringDist2);
    ringMask2 *= ringMask2 * fadeWithDist * 0.5;
    disp += dir * ringMask2 * strength * 0.4;

    var samplePos = clamp(position + disp, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Chromatic split on the ring
    var chromaAmt = ringMask * strength * 0.15;
    var chromaDir = vec2<f32>(dir.x * chromaAmt, dir.y * chromaAmt);
    var rSamp = textureSample(t_layer, s_layer, clamp(samplePos + chromaDir, vec2<f32>(0.0 / params.size), size));
    var bSamp = textureSample(t_layer, s_layer, clamp(samplePos - chromaDir, vec2<f32>(0.0 / params.size), size));
    color.r = mix(color.r, rSamp.r, f32(ringMask * 0.6));
    color.b = mix(color.b, bSamp.b, f32(ringMask * 0.6));

    // Bright flash on the ring edge
    color = vec4<f32>(color.rgb + (vec3<f32>(ringMask * 0.15)), color.a);

    return color;
}

// MARK: - 28. Thermal
// Thermal / infrared vision with heat shimmer

@fragment
fn bcs_thermal(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var shimmer = params.args[1];
    var noise_speed = params.args[2];
    var palette_shift = params.args[3];

    var uv = position / size;

    // Shimmer distortion, rising heat waves
    var st = uv * 8.0;
    var n1 = bcs_valueNoise(st + vec2<f32>(0.0, time * noise_speed * 2.0));
    var n2 = bcs_valueNoise(st * 1.3 + vec2<f32>(time * noise_speed * 1.5, 0.0));
    var heatDisp = vec2<f32>(
        (n1 - 0.5) * shimmer,
        (n2 - 0.5) * shimmer * 0.6 - shimmer * 0.3 // rising bias
    );

    var samplePos = clamp(position + heatDisp, vec2<f32>(0.0), size);
    var original = textureSample(t_layer, s_layer, samplePos / params.size);

    // Convert to "heat" value (luminance)
    var heat = dot(vec3<f32>(original.rgb), vec3<f32>(0.299, 0.587, 0.114));

    // Add some noise to break up flat areas
    heat += (bcs_valueNoise(uv * 20.0 + time * 0.5) - 0.5) * 0.05;
    heat = clamp(heat + palette_shift * 0.3, 0.0, 1.0);

    // Thermal palette: black → blue → purple → red → orange → yellow → white
    var thermal: vec3<f32>;
    if (heat < 0.15) {
        thermal = mix(vec3<f32>(0.0), vec3<f32>(0.0, 0.0, 0.3), f32(heat / 0.15));
    } else if (heat < 0.35) {
        thermal = mix(vec3<f32>(0.0, 0.0, 0.3), vec3<f32>(0.5, 0.0, 0.5), f32((heat - 0.15) / 0.2));
    } else if (heat < 0.55) {
        thermal = mix(vec3<f32>(0.5, 0.0, 0.5), vec3<f32>(1.0, 0.0, 0.0), f32((heat - 0.35) / 0.2));
    } else if (heat < 0.75) {
        thermal = mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.6, 0.0), f32((heat - 0.55) / 0.2));
    } else if (heat < 0.9) {
        thermal = mix(vec3<f32>(1.0, 0.6, 0.0), vec3<f32>(1.0, 1.0, 0.0), f32((heat - 0.75) / 0.15));
    } else {
        thermal = mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), f32((heat - 0.9) / 0.1));
    }

    var result = mix(original.rgb, thermal, f32(intensity));
    return vec4<f32>(result, original.a);
}

// MARK: - 29. Morph Breathe
// The image breathes and morphs like a living organism

@fragment
fn bcs_morphBreathe(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var breathe_depth = params.args[0];
    var breathe_rate = params.args[1];
    var warp_complexity = params.args[2];
    var organic = params.args[3];

    var uv = position / size;

    // Multi-layered breathing rhythms
    var breathe1 = sin(time * breathe_rate) * 0.5 + 0.5;
    var breathe2 = sin(time * breathe_rate * 0.7 + 1.5) * 0.5 + 0.5;
    var breathe3 = sin(time * breathe_rate * 1.3 + 3.0) * 0.5 + 0.5;

    // Organic displacement field
    var t = time * breathe_rate * 0.3;
    var st = uv * warp_complexity;

    var q = vec2<f32>(
        bcs_fbm(st + vec2<f32>(t * 0.5, t * 0.3), 4),
        bcs_fbm(st + vec2<f32>(5.2, 1.3) + vec2<f32>(t * 0.4, t * 0.6), 4)
    );

    // Mix organic warping with simple radial breathing
    var center = vec2<f32>(0.5, 0.5);
    var fromCenter = uv - center;
    var dist = length(fromCenter);

    // Radial breathe: expand/contract from center
    var radialPulse = breathe1 * (1.0 - organic) + breathe2 * organic;
    var radialDisp = fromCenter * (radialPulse - 0.5) * 2.0;

    // Organic warp: flowing noise displacement
    var organicDisp = vec2<f32>(
        (q.x - 0.5) * 2.0 * breathe2,
        (q.y - 0.5) * 2.0 * breathe3
    );

    var disp = mix(radialDisp, organicDisp, organic) * breathe_depth;

    // Edge softening, less displacement at edges to prevent harsh cutoff
    var edgeFade = smoothstep(0.0, 0.15, min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y)));
    disp *= edgeFade;

    var samplePos = clamp(position + disp, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Subtle color shift with breathing
    var colorPulse = breathe1 * 0.05;
    color.r *= f32(1.0 + colorPulse);
    color.b *= f32(1.0 - colorPulse);

    // Slight brightness pulse
    var brightPulse = 1.0 + (breathe1 - 0.5) * 0.08;
    color = vec4<f32>(color.rgb * (f32(brightPulse)), color.a);

    return color;
}

// MARK: - 30. Gravity Wells
// Multiple points of gravitational distortion pulling the image

@fragment
fn bcs_gravityWells(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var well_strength = params.args[0];
    var well_count = params.args[1];
    var orbit_speed = params.args[2];
    var warp_falloff = params.args[3];

    var uv = position / size;
    var aspectRatio = size.x / size.y;
    var totalDisp = vec2<f32>(0.0);

    var wells = i32(clamp(well_count, 1.0, 5.0));

    for (var i = 0; i < wells; i++) {
        // Each well orbits on its own path
        var phase = f32(i) * 6.2832 / f32(wells);
        var speed1 = orbit_speed * (0.7 + f32(i) * 0.15);
        var orbitRadius = 0.2 + f32(i) * 0.06;

        var wellPos = vec2<f32>(
            0.5 + cos(time * speed1 + phase) * orbitRadius,
            0.5 + sin(time * speed1 * 0.8 + phase * 1.3) * orbitRadius
        );

        var delta = uv - wellPos;
        delta.x *= aspectRatio;
        var dist = length(delta);

        // Gravity: inverse power law
        var pull = well_strength / (pow(dist, warp_falloff) * size.y + 10.0);
        pull = min(pull, well_strength * 0.5);

        var dir = select(vec2<f32>(0.0), normalize(delta), dist > 0.001);
        totalDisp -= dir * pull;
    }

    var samplePos = clamp(position + totalDisp, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Chromatic aberration proportional to total displacement
    var dispMag = length(totalDisp) * 0.1;
    var chromaDir = totalDisp * 0.08;
    var rSamp = textureSample(t_layer, s_layer, clamp(samplePos + chromaDir, vec2<f32>(0.0 / params.size), size));
    var bSamp = textureSample(t_layer, s_layer, clamp(samplePos - chromaDir, vec2<f32>(0.0 / params.size), size));
    var chromaBlend = clamp(dispMag * 0.02, 0.0, 0.5);
    color.r = mix(color.r, rSamp.r, f32(chromaBlend));
    color.b = mix(color.b, bSamp.b, f32(chromaBlend));

    return color;
}

// MARK: - 32. Liquid Mirror
// Seamless water-like reflection, no visible mirror line

@fragment
fn bcs_liquidMirror(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var mirror_axis = params.args[0];
    var ripple = params.args[1];
    var speed = params.args[2];
    var depth = params.args[3];

    var uv = position / size;

    // Soft transition zone instead of a hard line
    var transitionWidth = 0.08;
    var reflectionStart = mirror_axis - transitionWidth;
    var reflectionFull = mirror_axis + transitionWidth;

    // How deep into the reflection zone (0=start, 1=bottom of screen)
    var reflectionDepth = smoothstep(reflectionStart, 1.0, uv.y);
    // Blend factor: 0 in original image, ramps to 1 in full reflection
    var reflectionBlend = smoothstep(reflectionStart, reflectionFull, uv.y);

    // Mirror UV: flip around the axis
    var mirrorUV = uv;
    if (uv.y > reflectionStart) {
        mirrorUV.y = mirror_axis - (uv.y - mirror_axis);
    }

    // Liquid ripple displacement, only in reflected area
    var t = time * speed;
    var rippleSt = mirrorUV * 6.0;

    var ripple1 = sin(rippleSt.x * 4.0 + t * 1.3) * cos(rippleSt.y * 3.0 + t * 0.9);
    var ripple2 = sin(rippleSt.x * 7.0 - t * 1.7) * cos(rippleSt.y * 5.0 + t * 1.1);
    var ripple3 = bcs_valueNoise(rippleSt + vec2<f32>(t * 0.5, t * 0.3));

    var rippleStrength = reflectionBlend * reflectionDepth;
    var rippleDisp = vec2<f32>(
        (ripple1 * 0.5 + ripple2 * 0.3 + (ripple3 - 0.5) * 0.4),
        (ripple1 * 0.3 + ripple2 * 0.5 + (ripple3 - 0.5) * 0.3)
    ) * ripple * rippleStrength;

    // Sample reflected image
    var reflectedPos = clamp(mirrorUV * size + rippleDisp, vec2<f32>(0.0), size);
    var reflectedColor = textureSample(t_layer, s_layer, reflectedPos / params.size);

    // Sample original image
    var originalColor = textureSample(t_layer, s_layer, position / params.size);

    // Fade reflection based on depth parameter
    var fadeFactor = 1.0 - reflectionDepth * depth;
    reflectedColor = vec4<f32>(reflectedColor.rgb * (f32(max(fadeFactor, 0.2))), reflectedColor.a);

    // Slight desaturation on reflection for realism
    var luma = dot(reflectedColor.rgb, vec3<f32>(0.299, 0.587, 0.114));
    reflectedColor = vec4<f32>(mix(reflectedColor.rgb, vec3<f32>(luma), f32(reflectionDepth * 0.2)), reflectedColor.a);

    // Blend: smooth crossfade from original to reflection
    var color = mix(originalColor, reflectedColor, f32(reflectionBlend));

    // Subtle caustic highlight shimmer on the water
    var caustic = pow(max(ripple1 * ripple2 + 0.5, 0.0), 10.0) * 0.15 * rippleStrength;
    color = vec4<f32>(color.rgb + (vec3<f32>(caustic)), color.a);

    return color;
}

// MARK: - 33. Aurora
// Northern lights: flowing bands of colored light across the image

@fragment
fn bcs_aurora(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var bands = params.args[1];
    var speed = params.args[2];
    var color_shift = params.args[3];

    var uv = position / size;
    var color = textureSample(t_layer, s_layer, position / params.size);

    var t = time * speed;

    // Aurora is brightest in the upper portion
    var heightMask = smoothstep(0.8, 0.1, uv.y);
    heightMask *= smoothstep(0.0, 0.15, uv.y); // fade at very top

    // Flowing bands using layered sine waves
    var auroraVal = 0.0;
    var hueAccum = 0.0;

    for (var i = 0; i < i32(bands); i++) {
        var fi = f32(i);
        var freq = 2.0 + fi * 1.5;
        var phase = fi * 1.7;

        // Wavy band shape
        var wave = sin(uv.x * freq * 3.14159 + t * (0.8 + fi * 0.3) + phase);
        wave += sin(uv.x * freq * 1.7 + t * 0.5 + fi * 2.3) * 0.5;

        // Band position oscillates vertically
        var bandY = 0.3 + fi / bands * 0.4 + wave * 0.08;
        var bandDist = abs(uv.y - bandY);

        // Soft band shape
        var band = exp(-bandDist * bandDist * 200.0) * (0.6 + fi * 0.1);

        // Noise-driven intensity variation along the band
        var noiseVal = bcs_fbm(vec2<f32>(uv.x * 3.0 + t * 0.3, fi * 5.0 + t * 0.1), 3);
        band *= noiseVal;

        auroraVal += band;
        hueAccum += band * (fi / bands);
    }

    auroraVal = clamp(auroraVal, 0.0, 1.0) * heightMask * intensity;

    // Color: aurora greens, teals, purples, pinks
    var hue = fract(color_shift + hueAccum * 0.3 + 0.35); // base green
    var auroraColor = bcs_hsb2rgb(vec3<f32>(f32(hue), 0.7, 1.0));

    // Additive blend, light overlay
    color = vec4<f32>(color.rgb + (auroraColor * f32(auroraVal * 0.7)), color.a);

    // Subtle vertical shimmer
    var shimmer = sin(uv.y * 80.0 + t * 5.0) * 0.02 * auroraVal;
    color = vec4<f32>(color.rgb + (vec3<f32>(shimmer)), color.a);

    return color;
}

// MARK: - 34. Wormhole
// Tunnel zoom into the image center with spiral distortion

@fragment
fn bcs_wormhole(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var depth = params.args[0];
    var speed = params.args[1];
    var twist = params.args[2];
    var radius = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var aspectRatio = size.x / size.y;

    var delta = uv - center;
    delta.x *= aspectRatio;
    var dist = length(delta);
    var angle = atan2(delta.y, delta.x);

    // Tunnel mapping: remap distance to depth
    var t = time * speed;
    var tunnelDepth = radius / max(dist, 0.001);

    // Spiral twist increases with depth
    var twistAngle = angle + twist * tunnelDepth * 0.3 + t * 0.5;

    // Map tunnel coordinates back to image space
    var zoomFactor = fract(tunnelDepth * depth * 0.1 - t * 0.3);
    var scale = mix(0.2, 2.0, zoomFactor);

    var tunnelUV = center + vec2<f32>(
        cos(twistAngle) * scale * 0.3,
        sin(twistAngle) * scale * 0.3
    );

    // Keep in bounds with wrapping feel
    tunnelUV = fract(tunnelUV);

    var samplePos = clamp(tunnelUV * size, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    // Depth fog: darker toward the center (deeper in tunnel)
    var fog = smoothstep(0.0, radius * 2.0, dist);
    color = vec4<f32>(color.rgb * (f32(0.3 + fog * 0.7)), color.a);

    // Tunnel ring highlights
    var ringPattern = fract(tunnelDepth * depth * 0.1 - t * 0.3);
    var ring = exp(-pow((ringPattern - 0.5) * 8.0, 2.0)) * 0.2;
    color = vec4<f32>(color.rgb + (vec3<f32>(ring * 0.5, ring * 0.6, ring * 1.0)), color.a);

    // Edge vignette
    var vignette = 1.0 - smoothstep(0.3, 0.7, dist);
    color = vec4<f32>(color.rgb + (vec3<f32>(vignette * 0.05)), color.a);

    // Chromatic aberration at tunnel edges
    var chromaAmt = (1.0 - fog) * 3.0;
    var chromaDir = select(vec2<f32>(0.0), normalize(delta) * chromaAmt, dist > 0.001);
    chromaDir.x /= aspectRatio;
    var rSamp = textureSample(t_layer, s_layer, clamp((tunnelUV + chromaDir * 0.003 / params.size) * size, vec2<f32>(0.0), size));
    var bSamp = textureSample(t_layer, s_layer, clamp((tunnelUV - chromaDir * 0.003 / params.size) * size, vec2<f32>(0.0), size));
    var chromaBlend = (1.0 - fog) * 0.4;
    color.r = mix(color.r, rSamp.r, f32(chromaBlend));
    color.b = mix(color.b, bSamp.b, f32(chromaBlend));

    return color;
}

// MARK: - 35. Duochrome
// Two-tone color mapping with contrast control, dramatic poster effect

@fragment
fn bcs_duochrome(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var intensity = params.args[0];
    var hue1 = params.args[1];
    var hue2 = params.args[2];
    var contrast = params.args[3];

    var original = textureSample(t_layer, s_layer, position / params.size);

    // Luminance
    var luma = dot(vec3<f32>(original.rgb), vec3<f32>(0.299, 0.587, 0.114));

    // Contrast curve
    luma = clamp((luma - 0.5) * contrast + 0.5, 0.0, 1.0);

    // Slow hue animation
    var animHue1 = fract(hue1 + sin(time * 0.3) * 0.02);
    var animHue2 = fract(hue2 + cos(time * 0.25) * 0.02);

    // Two-tone mapping: shadows → hue1, highlights → hue2
    var shadowColor = bcs_hsb2rgb(vec3<f32>(f32(animHue1), 0.85, 0.4));
    var highlightColor = bcs_hsb2rgb(vec3<f32>(f32(animHue2), 0.7, 1.0));

    // Smooth interpolation with midtone richness
    var duoColor: vec3<f32>;
    if (luma < 0.5) {
        var t = luma * 2.0;
        // Dark to shadow color
        duoColor = mix(vec3<f32>(0.02), shadowColor, f32(t));
    } else {
        var t = (luma - 0.5) * 2.0;
        // Shadow to highlight
        duoColor = mix(shadowColor, highlightColor, f32(t));
    }

    var result = mix(original.rgb, duoColor, f32(intensity));
    return vec4<f32>(result, original.a);
}

// MARK: - Disintegrate
// Thanos-snap style particle dissolution, pixels scatter into dust

@fragment
fn bcs_disintegrate(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var threshold = params.args[0];
    var edgeWidth = params.args[1];
    var driftAmount = params.args[2];
    var direction = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    if (original.a < 0.01) { return original; }

    // Multi-octave noise for organic dissolve pattern
    var noise = bcs_fbm(uv * 6.0 + vec2<f32>(time * 0.1), 5);
    var noise2 = bcs_fbm(uv * 12.0 + vec2<f32>(17.0, 31.0), 4);
    var combinedNoise = noise * 0.7 + noise2 * 0.3;

    // Dissolve threshold with spatial sweep (bottom-right to top-left)
    var sweep = (uv.x * 0.4 + (1.0 - uv.y) * 0.6);
    var dissolveValue = combinedNoise * 0.6 + sweep * 0.4;

    // Core dissolve mask
    var edge = smoothstep(threshold - edgeWidth, threshold, dissolveValue);
    var innerEdge = smoothstep(threshold - edgeWidth * 0.3, threshold, dissolveValue);

    if (dissolveValue < threshold - edgeWidth * 1.5) {
        return vec4<f32>(0.0);
    }

    // Ember/glow edge
    var edgeMask = edge - innerEdge;
    var emberColor = vec3<f32>(1.0, 0.4, 0.05);
    var whiteHot = vec3<f32>(1.0, 0.95, 0.8);
    var glowColor = mix(emberColor, whiteHot, f32(innerEdge * 0.8));

    // Particle drift at the edge
    var driftDir = vec2<f32>(cos(direction), sin(direction));
    var particleDrift = (1.0 - edge) * driftAmount;
    var scatter = bcs_valueNoise(uv * 30.0 + time * 2.0);
    var driftOffset = driftDir * particleDrift + vec2<f32>(scatter - 0.5, scatter - 0.5) * particleDrift * 0.5;

    var driftedPos = clamp(position + driftOffset, vec2<f32>(0.0), size);
    var driftedColor = textureSample(t_layer, s_layer, driftedPos / params.size);

    var result = mix(driftedColor, original, f32(edge));
    result = vec4<f32>(mix(result.rgb, glowColor, f32(edgeMask * 3.0)), result.a);
    result = vec4<f32>(result.rgb + (glowColor * f32(edgeMask * 2.0)), result.a);
    result.a *= f32(edge);

    // Flickering sparks
    var sparkle = step(0.97, bcs_valueNoise(uv * 50.0 + time * 5.0)) * edgeMask * 5.0;
    result = vec4<f32>(result.rgb + (vec3<f32>(sparkle * 1.0, sparkle * 0.7, sparkle * 0.3)), result.a);

    return result;
}

// MARK: - Solarize
// Film solarization, psychedelic inversion at selective luminance thresholds

@fragment
fn bcs_solarize(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var threshold = params.args[0];
    var curveIntensity = params.args[1];
    var colorSeparation = params.args[2];
    var animate = params.args[3];

    var original = textureSample(t_layer, s_layer, position / params.size);
    var uv = position / size;

    var animOffset = sin(time * 1.5 + uv.x * 3.0) * animate * 0.15;
    var t = threshold + animOffset;

    var result: vec3<f32>;
    for (var ch = 0; ch < 3; ch++) {
        var channelOffset = f32(ch) * colorSeparation * 0.08;
        var channelThreshold = t + channelOffset;
        var val = f32(original.rgb[ch]);
        var dist = abs(val - channelThreshold);
        var curve = 1.0 - pow(dist * curveIntensity, 2.0);
        curve = clamp(curve, 0.0, 1.0);
        var inverted = 1.0 - val;
        var solarized = mix(val, inverted, curve);
        result[ch] = f32(solarized);
    }

    var grain = (bcs_hash(uv * 500.0 + fract(time * 0.1)) - 0.5) * 0.04;
    result += vec3<f32>(grain);

    return vec4<f32>(result, original.a);
}

// MARK: - Pixelate Mosaic
// 3D beveled tiles with animated assembly, not flat pixelation

@fragment
fn bcs_pixelateMosaic(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var pixelSize = params.args[0];
    var bevel = params.args[1];
    var animateAssemble = params.args[2];
    var gap = params.args[3];

    var uv = position / size;

    var gridUV = floor(uv * size / pixelSize) * pixelSize / size;
    var cellUV = fract(uv * size / pixelSize);

    var gapMask = 1.0;
    if (gap > 0.001) {
        var gapEdge = step(vec2<f32>(gap * 0.5), cellUV) * step(vec2<f32>(gap * 0.5), 1.0 - cellUV);
        gapMask = gapEdge.x * gapEdge.y;
    }

    if (gapMask < 0.5) {
        return vec4<f32>(0.02, 0.02, 0.03, 1.0);
    }

    var tileCenter = (gridUV + 0.5 * pixelSize / size);
    var tileHash = bcs_hash(gridUV * 100.0);
    var assembleProgress = clamp(time * 0.5 - tileHash * animateAssemble * 2.0, 0.0, 1.0);
    assembleProgress = assembleProgress * assembleProgress * (3.0 - 2.0 * assembleProgress);

    var scatteredPos = tileCenter + vec2<f32>(
        (bcs_hash(gridUV * 200.0) - 0.5) * 0.5,
        (bcs_hash(gridUV * 300.0) - 0.5) * 0.5
    ) * (1.0 - assembleProgress);

    var samplePos = clamp(scatteredPos * size, vec2<f32>(0.0), size);
    var tileColor = textureSample(t_layer, s_layer, samplePos / params.size);

    // 3D bevel lighting
    var bevelUV = (cellUV - 0.5) * 2.0;
    var topLight = smoothstep(0.0, -0.8, bevelUV.y) * bevel;
    var leftLight = smoothstep(0.0, -0.8, bevelUV.x) * bevel * 0.5;
    var bottomShadow = smoothstep(0.0, 0.8, bevelUV.y) * bevel;

    tileColor = vec4<f32>(tileColor.rgb + (vec3<f32>(topLight * 0.15 + leftLight * 0.1)), tileColor.a);
    tileColor = vec4<f32>(tileColor.rgb - (vec3<f32>(bottomShadow * 0.2)), tileColor.a);

    var edgeDist = min(min(cellUV.x, 1.0 - cellUV.x), min(cellUV.y, 1.0 - cellUV.y));
    var edgeHighlight = (1.0 - smoothstep(0.0, 0.08, edgeDist)) * bevel * 0.3;
    tileColor = vec4<f32>(tileColor.rgb + (vec3<f32>(edgeHighlight)), tileColor.a);

    tileColor.a *= f32(assembleProgress * 0.5 + 0.5);

    return tileColor;
}

// MARK: - Datamosh
// Digital codec corruption, smeared motion vectors, macro-blocking, I-frame bleed

@fragment
fn bcs_datamosh(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var blockCorruption = params.args[0];
    var smearAmount = params.args[1];
    var colorBleed = params.args[2];
    var glitchRate = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    var blockSize = 16.0;
    var blockUV = floor(uv * size / blockSize) / (size / blockSize);
    var blockHash = bcs_hash(blockUV * 73.0 + floor(time * glitchRate) * 0.1);

    var isCorrupted = step(1.0 - blockCorruption, blockHash);

    if (isCorrupted < 0.5) {
        return original;
    }

    var smearAngle = bcs_hash(blockUV * 137.0 + floor(time * glitchRate * 0.5) * 0.3) * 6.28;
    var smearDir = vec2<f32>(cos(smearAngle), sin(smearAngle));
    var blockSmear = smearAmount * (0.5 + blockHash * 0.5);
    var smearOffset = smearDir * blockSmear;

    var smearPos = clamp(position + smearOffset, vec2<f32>(0.0), size);
    var smeared = textureSample(t_layer, s_layer, smearPos / params.size);

    var rOffset = smearOffset * (1.0 + colorBleed * 0.3);
    var bOffset = smearOffset * (1.0 - colorBleed * 0.2);
    var rSamp = textureSample(t_layer, s_layer, clamp(position + rOffset, vec2<f32>(0.0 / params.size), size));
    var bSamp = textureSample(t_layer, s_layer, clamp(position + bOffset, vec2<f32>(0.0 / params.size), size));

    var result = smeared;
    result.r = mix(smeared.r, rSamp.r, f32(colorBleed));
    result.b = mix(smeared.b, bSamp.b, f32(colorBleed));

    var quantize = 16.0;
    result = vec4<f32>(floor(result.rgb * f32(quantize)) / f32(quantize), result.a);

    var blockCell = fract(uv * size / blockSize);
    var blockEdge = 1.0 - step(0.03, min(blockCell.x, blockCell.y));
    result = vec4<f32>(result.rgb + (vec3<f32>(blockEdge * 0.1)), result.a);

    return result;
}

// MARK: - Magnetic Field
// Ferrofluid-inspired displacement, lines of force warp the image

@fragment
fn bcs_magneticField(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var fieldStrength = params.args[0];
    var lineCount = params.args[1];
    var fieldTurbulence = params.args[2];
    var polarity = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);

    var pole1 = center + vec2<f32>(-0.25, 0.0);
    var pole2 = center + vec2<f32>(0.25, 0.0);

    var toP1 = uv - pole1;
    var toP2 = uv - pole2;
    var r1 = max(length(toP1), 0.001);
    var r2 = max(length(toP2), 0.001);

    var field = toP1 / (r1 * r1) - toP2 / (r2 * r2);

    if (polarity > 0.01) {
        var pole3 = center + vec2<f32>(0.0, -0.2);
        var pole4 = center + vec2<f32>(0.0, 0.2);
        var toP3 = uv - pole3;
        var toP4 = uv - pole4;
        var r3 = max(length(toP3), 0.001);
        var r4 = max(length(toP4), 0.001);
        var quadField = toP3 / (r3 * r3) - toP4 / (r4 * r4);
        field = mix(field, field + quadField, polarity);
    }

    var fieldMag = length(field);
    var fieldDir = select(vec2<f32>(0.0), field / fieldMag, fieldMag > 0.001);

    var angle = atan2(field.y, field.x);
    var stripes = sin(angle * lineCount + time * 2.0);
    stripes = stripes * stripes;

    var turb = bcs_fbm(uv * 8.0 + time * 0.5, 4) * fieldTurbulence;

    var offset = fieldDir * fieldStrength * stripes * (0.5 + turb);
    var perpDir = vec2<f32>(-fieldDir.y, fieldDir.x);
    var perpStripe = sin(dot(uv, fieldDir) * lineCount * 10.0 + time);
    offset += perpDir * perpStripe * fieldStrength * 0.15;

    var displaced = clamp(position + offset, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, displaced / params.size);

    var sheen = stripes * fieldMag * 0.3;
    color = vec4<f32>(color.rgb + (vec3<f32>(sheen * 0.3, sheen * 0.35, sheen * 0.4)), color.a);

    return color;
}

// MARK: - Underwater Caustics
// Dancing light refractions like sunlight through water

@fragment
fn bcs_underwaterCaustics(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var causticScale = params.args[0];
    var causticIntensity = params.args[1];
    var waterDistortion = params.args[2];
    var waterDepth = params.args[3];

    var uv = position / size;

    var n1 = bcs_fbm(uv * 4.0 + vec2<f32>(time * 0.3, time * 0.2), 4);
    var n2 = bcs_fbm(uv * 4.0 + vec2<f32>(-time * 0.25, time * 0.35) + 10.0, 4);
    var waterDisp = vec2<f32>(n1 - 0.5, n2 - 0.5) * waterDistortion;

    var displaced = clamp(position + waterDisp, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, displaced / params.size);

    // Voronoi-based caustic pattern
    var causticUV = uv * causticScale;
    var animUV1 = causticUV + vec2<f32>(time * 0.4, time * 0.3);
    var animUV2 = causticUV * 1.3 + vec2<f32>(-time * 0.35, time * 0.45);

    var caustic1 = 0.0;
    var caustic2 = 0.0;

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            var neighbor = vec2<f32>(f32(x), f32(y));
            var cell1 = floor(animUV1) + neighbor;
            var point1 = cell1 + vec2<f32>(bcs_hash(cell1), bcs_hash(cell1 + 100.0));
            caustic1 = max(caustic1, 1.0 - length(fract(animUV1) - fract(point1)) * 2.5);

            var cell2 = floor(animUV2) + neighbor;
            var point2 = cell2 + vec2<f32>(bcs_hash(cell2 + 50.0), bcs_hash(cell2 + 150.0));
            caustic2 = max(caustic2, 1.0 - length(fract(animUV2) - fract(point2)) * 2.5);
        }
    }

    var caustic = caustic1 * caustic2;
    caustic = pow(max(caustic, 0.0), 3.0) * causticIntensity;

    var causticColor = vec3<f32>(0.95, 0.98, 1.0);
    color = vec4<f32>(color.rgb + (causticColor * f32(caustic)), color.a);

    var depthTint = vec3<f32>(0.2, 0.5, 0.7);
    color = vec4<f32>(mix(color.rgb, color.rgb * (1.0 - f32(waterDepth * 0.3)) + depthTint * f32(waterDepth * 0.15), f32(waterDepth)), color.a);

    var rays = sin(uv.x * 20.0 + time * 0.5) * 0.5 + 0.5;
    rays *= smoothstep(1.0, 0.0, uv.y) * waterDepth * 0.1;
    color = vec4<f32>(color.rgb + (vec3<f32>(rays * 0.3, rays * 0.5, rays * 0.6)), color.a);

    return color;
}

// MARK: - Topographic
// Contour map visualization, converts image into elevation lines

@fragment
fn bcs_topographic(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var lineCount = params.args[0];
    var lineWidth = params.args[1];
    var colorize = params.args[2];
    var animate = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    var lum = dot(vec3<f32>(original.rgb), vec3<f32>(0.299, 0.587, 0.114));
    var elevation = lum + time * animate * 0.05;

    var contourValue = fract(elevation * lineCount);
    var contourLine = 1.0 - smoothstep(lineWidth, lineWidth + 0.02, contourValue)
                       + 1.0 - smoothstep(lineWidth, lineWidth + 0.02, 1.0 - contourValue);
    contourLine = clamp(contourLine, 0.0, 1.0);

    var majorContour = fract(elevation * lineCount / 5.0);
    var majorLine = 1.0 - smoothstep(lineWidth * 2.0, lineWidth * 2.0 + 0.03, majorContour)
                     + 1.0 - smoothstep(lineWidth * 2.0, lineWidth * 2.0 + 0.03, 1.0 - majorContour);
    majorLine = clamp(majorLine, 0.0, 1.0);

    var topoColor: vec3<f32>;
    if (lum < 0.2) {
        topoColor = mix(vec3<f32>(0.1, 0.3, 0.5), vec3<f32>(0.15, 0.45, 0.3), f32(lum * 5.0));
    } else if (lum < 0.5) {
        topoColor = mix(vec3<f32>(0.15, 0.45, 0.3), vec3<f32>(0.8, 0.75, 0.4), f32((lum - 0.2) * 3.33));
    } else if (lum < 0.75) {
        topoColor = mix(vec3<f32>(0.8, 0.75, 0.4), vec3<f32>(0.65, 0.45, 0.3), f32((lum - 0.5) * 4.0));
    } else {
        topoColor = mix(vec3<f32>(0.65, 0.45, 0.3), vec3<f32>(0.95, 0.95, 0.97), f32((lum - 0.75) * 4.0));
    }

    var baseColor = mix(original.rgb, topoColor, f32(colorize));
    var lineColor = vec3<f32>(0.15, 0.12, 0.1);
    var majorLineColor = vec3<f32>(0.05, 0.04, 0.03);

    var result = baseColor;
    result = mix(result, lineColor, f32(contourLine * 0.7));
    result = mix(result, majorLineColor, f32(majorLine * 0.9));

    var paper = bcs_valueNoise(uv * 200.0) * 0.06 - 0.03;
    result += vec3<f32>(paper);

    return vec4<f32>(result, original.a);
}

// MARK: - Smoke Reveal
// Swirling smoke that clears to reveal the image underneath

@fragment
fn bcs_smokeReveal(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var smokeAmount = params.args[0];
    var smokeScale = params.args[1];
    var windSpeed = params.args[2];
    var smokeTurb = params.args[3];

    var uv = position / size;
    var original = textureSample(t_layer, s_layer, position / params.size);

    var smokeUV = uv * smokeScale;
    var warp1x = bcs_fbm(smokeUV + vec2<f32>(time * windSpeed * 0.3, time * windSpeed * 0.1), 5);
    var warp1y = bcs_fbm(smokeUV + vec2<f32>(time * windSpeed * 0.1, -time * windSpeed * 0.2) + 5.2, 5);

    var warped = smokeUV + vec2<f32>(warp1x, warp1y) * smokeTurb;
    var smokeDensity = bcs_fbm(warped + vec2<f32>(time * windSpeed * 0.15, time * windSpeed * 0.08), 6);

    smokeDensity = smokeDensity * smokeDensity;
    smokeDensity *= smokeAmount * 1.5;
    smokeDensity = clamp(smokeDensity, 0.0, 1.0);

    var lightVariation = bcs_valueNoise(uv * 3.0 + time * 0.2);
    var smokeColor = vec3<f32>(0.7 + f32(lightVariation) * 0.15,
                             0.68 + f32(lightVariation) * 0.12,
                             0.66 + f32(lightVariation) * 0.1);

    var edgeGlow = smoothstep(0.2, 0.5, smokeDensity) - smoothstep(0.5, 0.8, smokeDensity);
    smokeColor += vec3<f32>(edgeGlow * 0.2);

    var smokeDisp = vec2<f32>(warp1x - 0.5, warp1y - 0.5) * 8.0 * smokeDensity;
    var displacedPos = clamp(position + smokeDisp, vec2<f32>(0.0), size);
    var displacedColor = textureSample(t_layer, s_layer, displacedPos / params.size);

    var result: vec4<f32>;
    result = vec4<f32>(mix(displacedColor.rgb, smokeColor, f32(smokeDensity)), result.a);
    result.a = original.a;

    var ray = sin(uv.x * 8.0 + time * 0.3) * 0.5 + 0.5;
    ray *= smoothstep(1.0, 0.3, uv.y) * smokeDensity * 0.15;
    result = vec4<f32>(result.rgb + (vec3<f32>(ray * 0.8, ray * 0.7, ray * 0.5)), result.a);

    return result;
}

// MARK: - Geometric Warp
// Droste effect / Escher-inspired infinite spiral zoom

@fragment
fn bcs_geometricWarp(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var spiralTight = params.args[0];
    var zoomRepeat = params.args[1];
    var rotation = params.args[2];
    var blend = params.args[3];

    var uv = position / size;
    var center = vec2<f32>(0.5, 0.5);
    var delta = uv - center;

    var r = length(delta);
    var theta = atan2(delta.y, delta.x);

    var logR = log(max(r, 0.0001));
    var spiralAngle = theta + logR * spiralTight + time * 0.5 + rotation;

    var zoomPhase = fract(logR * zoomRepeat + time * 0.2);
    var repeatedR = exp(zoomPhase / zoomRepeat);

    var segments = 6.0;
    var kAngle = mod_f32(spiralAngle, 6.28 / segments);
    if (mod_f32(floor(spiralAngle / (6.28 / segments)), 2.0) > 0.5) {
        kAngle = 6.28 / segments - kAngle;
    }

    var finalAngle = mix(spiralAngle, kAngle, blend);

    var warpedUV = center + vec2<f32>(cos(finalAngle), sin(finalAngle)) * repeatedR * 0.3;
    warpedUV = fract(warpedUV);

    var samplePos = clamp(warpedUV * size, vec2<f32>(0.0), size);
    var color = textureSample(t_layer, s_layer, samplePos / params.size);

    var centerGlow = exp(-r * r * 8.0) * 0.15;
    color = vec4<f32>(color.rgb + (vec3<f32>(centerGlow * 0.5, centerGlow * 0.7, centerGlow)), color.a);

    var boundary = 1.0 - smoothstep(0.0, 0.02, abs(fract(logR * zoomRepeat + time * 0.2) - 0.5) - 0.48);
    color = vec4<f32>(color.rgb + (vec3<f32>(boundary * 0.05, boundary * 0.02, boundary * 0.08)), color.a);

    return color;
}

// MARK: - Shatter Glass
// Cracked glass with refraction and prismatic splitting at cracks

@fragment
fn bcs_shatterGlass(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.uv * params.size;
    var size = params.size;
    var time = params.time;
    var crackDensity = params.args[0];
    var glassRefraction = params.args[1];
    var prismStrength = params.args[2];
    var shatterSpread = params.args[3];

    var uv = position / size;

    var cellUV = uv * crackDensity;
    var iCell = floor(cellUV);
    var fCell = fract(cellUV);

    var minDist = 10.0;
    var secondDist = 10.0;
    var nearestPoint = vec2<f32>(0.0);
    var secondPoint = vec2<f32>(0.0);
    var nearestHash = 0.0;

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            var neighbor = vec2<f32>(f32(x), f32(y));
            var cell = iCell + neighbor;
            var point = vec2<f32>(
                bcs_hash(cell),
                bcs_hash(cell + vec2<f32>(127.1, 311.7))
            );

            var diff = neighbor + point - fCell;
            var d = length(diff);

            if (d < minDist) {
                secondDist = minDist;
                secondPoint = nearestPoint;
                minDist = d;
                nearestPoint = diff;
                nearestHash = bcs_hash(cell + 500.0);
            } else if (d < secondDist) {
                secondDist = d;
                secondPoint = diff;
            }
        }
    }

    var edgeDist = secondDist - minDist;
    var crackLine = 1.0 - smoothstep(0.0, 0.06, edgeDist);

    var shardOffset = nearestPoint * shatterSpread * 15.0;
    var shardAngle = nearestHash * 0.3 * shatterSpread;
    var rotatedPos = vec2<f32>(
        cos(shardAngle) * shardOffset.x - sin(shardAngle) * shardOffset.y,
        sin(shardAngle) * shardOffset.x + cos(shardAngle) * shardOffset.y
    );

    var displaced = clamp(position + rotatedPos, vec2<f32>(0.0), size);

    var refrDir = normalize(nearestPoint - secondPoint);
    var refrOffset = refrDir * glassRefraction * crackLine;
    displaced = clamp(displaced + refrOffset, vec2<f32>(0.0), size);

    var color = textureSample(t_layer, s_layer, displaced / params.size);

    if (prismStrength > 0.01 && crackLine > 0.1) {
        var rPos = clamp(displaced + refrDir * prismStrength * 5.0, vec2<f32>(0.0), size);
        var bPos = clamp(displaced - refrDir * prismStrength * 5.0, vec2<f32>(0.0), size);
        color.r = textureSample(t_layer, s_layer, rPos / params.size).r;
        color.b = textureSample(t_layer, s_layer, bPos / params.size).b;
    }

    var edgeHighlight = crackLine * (0.5 + 0.5 * sin(edgeDist * 100.0 + time));
    color = vec4<f32>(color.rgb + (vec3<f32>(edgeHighlight * 0.6)), color.a);

    var shardBrightness = nearestHash * 0.15 - 0.075;
    color = vec4<f32>(color.rgb + (vec3<f32>(shardBrightness)), color.a);

    var crackShadow = crackLine * 0.4;
    color = vec4<f32>(color.rgb - (vec3<f32>(crackShadow)), color.a);

    return color;
}

// ============================================================================
// MARK: - MESH BACKGROUND GENERATORS
// These shaders sample colors from the book cover image and use them
// to generate procedural animated backgrounds
// ============================================================================

// Shared helper: sample a blurred region from the cover (average of nearby pixels)


// (End of cover gradient generators)

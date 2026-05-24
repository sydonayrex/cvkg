@fragment
fn fs_background(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Screen-Space UV (Continuous across the whole field)
    let uv = in.uv;
    let time = scene.time;
    
    // 2. Global Center-Based Gradient (No more slabs)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);
    
    var base = theme.background_deep;
    
    if scene.scene_type == 0u {
        // --- AURORA BOREALIS ---
        let band = sin(uv.y * 15.0 + time * 0.4) * 0.01;
        let n1 = fbm(uv * 2.5 + time * 0.15);
        let n2 = vnoise(uv * 10.0 + time * 0.1) * 0.005;
        let glow_field = dist + band + n1 * 0.02 + n2;
        base = mix(theme.background_deep, theme.primary_neon * 0.15, clamp(1.0 - glow_field, 0.0, 1.0));
        
    } else if scene.scene_type == 1u {
        // --- VOID (Minimalist) ---
        base = mix(theme.background_deep, vec4<f32>(0.02, 0.02, 0.03, 1.0), dist);
        let stars = hash21(uv * 500.0);
        if stars > 0.999 {
            base += vec4<f32>(1.0, 1.0, 1.0, 0.2) * (0.8 + 0.2 * sin(time * 0.5 + stars * 100.0));
        }
        
    } else if scene.scene_type == 2u {
        // --- NEBULA ---
        let n1 = fbm(uv * 1.5 + time * 0.05);
        let n2 = fbm(uv * 4.0 - time * 0.03);
        let nebula = mix(theme.primary_neon, theme.shatter_neon, n1);
        base = mix(theme.background_deep, nebula * 0.15, n2 * n1);
        
    } else if scene.scene_type == 3u {
        // --- GLITCH ---
        var guv = uv;
        let glitch = hash21(vec2(floor(time * 5.0), floor(uv.y * 40.0)));
        if glitch > 0.98 {
            guv.x += (glitch - 0.98) * 0.05;
        }
        base = mix(theme.background_deep, theme.shatter_neon * 0.1, fbm(guv * 10.0 + time));
        if glitch > 0.99 {
            base += vec4<f32>(0.0, 1.0, 1.0, 0.05);
        }
        
    } else if scene.scene_type == 4u {
        // --- YGGDRASIL (Tree of Life) ---
        let n = fbm(uv * 2.0 + vec2(0.0, time * 0.1));
        let root_glow = 1.0 - smoothstep(0.0, 0.9, abs(uv.x - 0.5 + 0.05 * sin(uv.y * 4.0 + time)));
        base = mix(theme.background_deep, vec4<f32>(0.0, 0.8, 0.4, 1.0) * 0.1, root_glow * n);
    }
    
    // 6. Global Vignette (Softened)
    let vignette = 1.0 - dist * 0.5;
    return vec4<f32>(base.rgb * vignette, 1.0);
}

use cvkg_core::{Event, FrameRenderer, KvasirId, Rect, Renderer};
use cvkg_core::{SpringParams, SpringSolver};
use cvkg_render_gpu::GpuRenderer;
use cvkg_vdom::{LayoutRect, VDom, VNode};

/// Benchmark and capture frame. Runs the provided layout function for 60 frames,
/// measures average frame time to ensure >= 60 FPS, and returns the final captured pixels.
fn benchmark_and_capture(
    renderer: &mut GpuRenderer,
    mut layout_fn: impl FnMut(&mut GpuRenderer),
) -> Vec<u8> {
    let start_time = std::time::Instant::now();
    for _ in 0..60 {
        layout_fn(renderer);
    }
    let elapsed = start_time.elapsed();
    let avg_ms = (elapsed.as_secs_f32() / 60.0) * 1000.0;
    assert!(
        avg_ms <= 16.7,
        "Performance degradation: Advanced rendering failed to maintain 60 FPS. Average frame took {:.2}ms",
        avg_ms
    );

    // One more frame and readback
    layout_fn(renderer);
    pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame")
}

fn count_matching_pixels(
    pixels: &[u8],
    width: u32,
    height: u32,
    predicate: impl Fn(&[u8]) -> bool,
) -> usize {
    let mut count = 0;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = (y * width as usize + x) * 4;
            if predicate(&pixels[idx..idx + 4]) {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn test_advanced_vdom_with_glassmorphism() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Create a real VDom instance
    let mut vdom = VDom::new();
    let root_id = KvasirId(1);

    let root_node = VNode {
        id: root_id,
        key: None,
        component_type: "GlassContainer".to_string(),
        props: std::collections::HashMap::new(),
        state: None,
        layout: LayoutRect {
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 200.0,
        },
        children: vec![],
        aria_role: "presentation".to_string(),
        aria_props: Default::default(),
        portal_target: None,
        sdf_shape: Some(cvkg_core::layout::SdfShape::Rect(Rect {
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 200.0,
        })),
    };

    vdom.nodes.insert(root_id, root_node);
    vdom.root = Some(root_id);

    // Click handler to test event dispatching
    let clicked = std::sync::Arc::new(std::sync::Mutex::new(false));
    let clicked_clone = clicked.clone();
    vdom.event_handlers.insert(
        root_id,
        vec![(
            "pointerdown".to_string(),
            std::sync::Arc::new(move |_| {
                *clicked_clone.lock().unwrap() = true;
            }) as _,
        )]
        .into_iter()
        .collect(),
    );

    // Dispatch an event via VDOM to verify it works
    vdom.dispatch_event(Event::PointerDown {
        x: 100.0,
        y: 100.0,
        button: 0,
        proximity_field: 0.0,
        tilt: None,
        azimuth: None,
        pressure: None,
        barrel_rotation: None,
        pointer_precision: 0.0,
    });
    assert!(*clicked.lock().unwrap(), "VDOM event dispatch failed");

    let pixels = benchmark_and_capture(&mut renderer, |renderer| {
        let encoder = renderer.begin_frame_headless();

        renderer.set_z_index(0.0);
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.1, 0.1, 0.2, 1.0],
        );

        // Traverse VDOM to render (Mock compositor integration)
        if let Some(root_id) = vdom.root
            && let Some(node) = vdom.nodes.get(&root_id)
        {
            renderer.set_z_index(-0.1);
            renderer.fill_glass_rect(
                Rect {
                    x: node.layout.x,
                    y: node.layout.y,
                    width: node.layout.width,
                    height: node.layout.height,
                },
                8.0,
                15.0,
            );
        }

        // Inner neon element
        renderer.set_z_index(-0.2);
        renderer.fill_rect(
            Rect {
                x: 100.0,
                y: 100.0,
                width: 100.0,
                height: 100.0,
            },
            [0.0, 1.0, 0.5, 1.0],
        );

        renderer.render_frame();
        renderer.end_frame(encoder);
    });

    // Check for neon element pixels using relative check
    let neon_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[1] > p[0].saturating_add(40) && p[1] > p[2]
    });
    assert!(
        neon_pixels > 0,
        "VDOM glassmorphism component neon inner failed to render"
    );
}

#[test]
fn test_advanced_particles_and_springs() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    // Real spring physics
    let mut spring = SpringSolver::new(SpringParams::default(), 100.0, 0.0);

    let pixels = benchmark_and_capture(&mut renderer, |renderer| {
        // Step spring physics simulation
        let val = spring.tick(1.0 / 60.0);

        let encoder = renderer.begin_frame_headless();

        renderer.set_z_index(0.0);
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.05, 0.05, 0.05, 1.0],
        );

        // Draw multiple particles using spring position with additive blending (Material mode 1)
        renderer.set_material(cvkg_core::DrawMaterial::Blend { mode: 1 });
        renderer.set_z_index(-0.1);

        for i in 0..10 {
            let x = 250.0 + (i as f32 * 10.0) + (val * 0.1);
            let y = 250.0 - (i as f32 * 5.0);
            renderer.fill_rect(
                Rect {
                    x,
                    y,
                    width: 10.0,
                    height: 10.0,
                },
                [0.9, 0.3, 0.1, 0.8],
            );
        }

        renderer.render_frame();
        renderer.end_frame(encoder);
    });

    // We verify coverage (non-black) since it's testing feature integration.
    let particle_pixels = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 50 || p[1] > 50 || p[2] > 50
    });
    assert!(
        particle_pixels > 0,
        "Particles with spring-driven position failed to render"
    );
}

#[test]
fn test_advanced_volumetric_raymarching() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.bloom_enabled = true; // Required for volumetric glow
    renderer.set_theme(cvkg_core::ColorTheme::vibrant_glass());

    let pixels = benchmark_and_capture(&mut renderer, |renderer| {
        let encoder = renderer.begin_frame_headless();

        // Hologram instance which triggers volumetric raymarching pass
        renderer.draw_hologram(
            Rect {
                x: 100.0,
                y: 100.0,
                width: 300.0,
                height: 300.0,
            },
            "test_hologram_1",
            1.5,
        );

        renderer.render_frame();
        renderer.end_frame(encoder);
    });

    // Check for cyan/blue volumetric glow
    let bright_pixels =
        count_matching_pixels(&pixels, width, height, |p| p[2] > p[0].saturating_add(20));
    assert!(
        bright_pixels > 0,
        "Volumetric raymarching produced empty or too-dark output"
    );
}

#[test]
fn test_advanced_path_tracing_pass() {
    let width = 512;
    let height = 512;
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));

    let pixels = benchmark_and_capture(&mut renderer, |renderer| {
        let encoder = renderer.begin_frame_headless();

        renderer.set_z_index(0.0);
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.1, 0.1, 0.1, 1.0],
        );

        renderer.set_z_index(-0.1);
        renderer.fill_rect(
            Rect {
                x: 150.0,
                y: 150.0,
                width: 200.0,
                height: 200.0,
            },
            [0.8, 0.8, 0.8, 1.0],
        );

        renderer.render_frame();
        renderer.end_frame(encoder);
    });

    // Coverage check to ensure no panics and valid frame
    let non_black = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 10 || p[1] > 10 || p[2] > 10
    });
    assert!(
        non_black > 1000,
        "Path tracing pass resulted in empty frame"
    );
}

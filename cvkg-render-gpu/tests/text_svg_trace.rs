use cvkg_core::{FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::SurtrRenderer;

/// Capture the framebuffer from a headless renderer.
fn capture_frame(renderer: &mut SurtrRenderer) -> Vec<u8> {
    pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame")
}

#[test]
fn test_text_svg_rendering_trace() {
    let _ = env_logger::try_init();
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    
    // Load a dummy SVG
    let svg_data = br#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
        <rect width="100" height="100" fill="red"/>
        <circle cx="50" cy="50" r="40" fill="green"/>
    </svg>"#;
    renderer.load_svg("test_icon", svg_data);

    let encoder = renderer.begin_frame_headless();

    // Background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 0.0, 1.0],
    );

    // Render Text (should write to atlas)
    renderer.draw_text(
        "Trace Text",
        32.0,
        32.0,
        24.0,
        [1.0, 1.0, 1.0, 1.0],
    );

    // Draw the whole atlas to the screen
    renderer.draw_image(
        "__mega_heim",
        Rect {
            x: 0.0,
            y: 0.0,
            width: 256.0,
            height: 256.0,
        }
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    // Frame 2
    let encoder2 = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 0.0, 1.0],
    );
    renderer.draw_text(
        "Trace Text",
        32.0,
        32.0,
        24.0,
        [1.0, 1.0, 1.0, 1.0],
    );
    renderer.draw_image(
        "__mega_heim",
        Rect {
            x: 0.0,
            y: 0.0,
            width: 256.0,
            height: 256.0,
        }
    );
    renderer.render_frame();
    renderer.end_frame(encoder2);

    let pixels = capture_frame(&mut renderer);

    println!("--- TRACE RESULTS ---");
    println!("Telemetry: {:?}", renderer.telemetry);
    
    // Check for Text (White)
    let white_pixels = pixels.chunks_exact(4).filter(|p| p[0] > 200 && p[1] > 200 && p[2] > 200).count();
    println!("White pixels (Text): {}", white_pixels);

    // If both text and atlas fail, we'll see 0 counts.
    assert!(white_pixels > 0, "All text and SVG rendered invisible!");
}

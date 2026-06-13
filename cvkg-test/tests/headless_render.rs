use cvkg_core::{FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::SurtrRenderer;

#[tokio::test]
async fn test_headless_render_capture() {
    let _ = env_logger::try_init();
    let width = 128;
    let height = 128;
    let mut renderer = SurtrRenderer::forge_headless(width, height).await;

    // 1. Setup Frame
    let encoder = renderer.begin_frame_headless();

    // Clear to black first to ensure a clean slate
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 0.0, 1.0],
    );

    // Draw a prominent red square in the middle
    renderer.fill_rect(
        Rect {
            x: 32.0,
            y: 32.0,
            width: 64.0,
            height: 64.0,
        },
        [1.0, 0.0, 0.0, 1.0],
    );

    // 2. Render and End
    renderer.render_frame();
    renderer.end_frame(encoder);

    // 3. Capture and Verify
    let pixels = renderer
        .capture_frame()
        .await
        .expect("Failed to capture frame");

    // Check a pixel inside the red square (e.g., center at 64, 64)
    let idx = (64 * width + 64) as usize * 4;
    let r = pixels[idx];
    let g = pixels[idx + 1];
    let b = pixels[idx + 2];

    println!(
        "Center Pixel (64,64): R={}, G={}, B={}, A={}",
        r,
        g,
        b,
        pixels[idx + 3]
    );
    println!(
        "Telemetry: draw_calls={}, vertices={}",
        renderer.telemetry.draw_calls, renderer.telemetry.vertices
    );

    // With ACES tonemapping, 1.0 red becomes ~204 (0.80)
    assert!(r > 150, "Red component should be high, got {}", r);
    assert!(g < 100, "Green component should be low, got {}", g);
    assert!(b < 100, "Blue component should be low, got {}", b);

    // Check a pixel outside the square (e.g., at 5, 5)
    let idx_out = (5 * width + 5) as usize * 4;
    let r_out = pixels[idx_out];
    println!(
        "Corner Pixel (5,5): R={}, G={}, B={}, A={}",
        r_out,
        pixels[idx_out + 1],
        pixels[idx_out + 2],
        pixels[idx_out + 3]
    );
    assert!(
        r_out < 50,
        "Corner pixel should be black/background, got R={}",
        r_out
    );
}

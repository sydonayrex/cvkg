use cvkg_render_gpu::SurtrRenderer;
use cvkg_core::{Rect, Renderer};

#[tokio::test]
async fn test_headless_render_capture() {
    let width = 256;
    let height = 256;
    let mut renderer = SurtrRenderer::forge_headless(width, height).await;
    
    let encoder = renderer.begin_frame_headless();
    
    // Draw something simple: a red square
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 },
        [1.0, 0.0, 0.0, 1.0]
    );
    
    renderer.end_frame(encoder);
    
    let pixels = renderer.capture_frame().await;
    
    assert_eq!(pixels.len(), (width * height * 4) as usize);
    
    // Check a pixel inside the red square (e.g., at 5, 5)
    let idx = (5 * width + 5) as usize * 4;
    let r = pixels[idx];
    let g = pixels[idx+1];
    let b = pixels[idx+2];
    
    assert!(r > 200, "Red component should be high, got {}", r);
    assert!(g < 100, "Green component should be low, got {}", g);
    assert!(b < 100, "Blue component should be low, got {}", b);
    
    // Check a pixel outside (e.g., at 100, 100) - should be the background color
    let idx_out = (100 * width + 100) as usize * 4;
    let r_out = pixels[idx_out];
    assert!(r_out < 100, "Background should be dark, got {}", r_out);
}

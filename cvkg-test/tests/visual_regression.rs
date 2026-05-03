use cvkg_render_gpu::SurtrRenderer;
use cvkg_core::{Rect, Renderer};
use cvkg_test::VisualComparator;

#[tokio::test]
async fn test_visual_regression_basic() {
    let width = 128;
    let height = 128;
    let mut renderer = SurtrRenderer::forge_headless(width, height).await;
    
    // 1. Render Frame
    let encoder = renderer.begin_frame_headless();
    
    // Solid background to avoid "atmosphere" noise in tests
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.0, 0.0, 0.0, 1.0]
    );
    
    renderer.fill_rect(
        Rect { x: 10.0, y: 10.0, width: 50.0, height: 50.0 },
        [1.0, 0.0, 0.0, 1.0]
    );
    renderer.end_frame(encoder);
    
    let pixels1 = renderer.capture_frame().await;
    
    // 2. Render Again
    let encoder = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.0, 0.0, 0.0, 1.0]
    );
    renderer.fill_rect(
        Rect { x: 10.0, y: 10.0, width: 50.0, height: 50.0 },
        [1.0, 0.0, 0.0, 1.0]
    );
    renderer.end_frame(encoder);
    
    let pixels2 = renderer.capture_frame().await;
    
    // 3. Compare
    let comparator = VisualComparator::default();
    let diff = comparator.compare(&pixels1, &pixels2);
    
    println!("Determinism diff: {}%", diff);
    
    assert!(diff < 0.01, "Renderer is not deterministic: {}% difference", diff);
}

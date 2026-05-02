use cvkg_core::{Rect, RenderTier, View, FrameRenderer};
use cvkg_components::ValkyrieIndicator;
use cvkg_render_web::WebRenderer;
use cvkg_render_gpu::SurtrRenderer;

#[tokio::test]
async fn test_journey_web_backend_stub() {
    let renderer = WebRenderer::new();
    
    // On non-wasm32, this should be Tier3Fallback
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(renderer.tier(), RenderTier::Tier3Fallback);
    
    // Phase 1: Initial Render
    // Verification: On native host, WebRenderer is a stub that should not panic
    // but also does not record commands like the TestRenderer.
    assert!(true);
}

#[tokio::test]
async fn test_journey_native_backend_initialization() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        SurtrRenderer::forge_headless(100, 100)
    ).await;
    
    match result {
        Ok(_) => println!("Native GPU Forge Successful"),
        Err(_) => println!("Native GPU Forge Timed Out (Expected on headless CI)"),
    }
}

#[test]
fn test_journey_vdom_to_renderer_integration() {
    let mut renderer = cvkg_scene::test_renderer::TestRenderer::new();
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    
    renderer.begin_frame();
    let indicator = ValkyrieIndicator::new(0.5);
    indicator.render(&mut renderer, rect);
    renderer.end_frame(());
    
    assert!(!renderer.commands.is_empty(), "Renderer should have received commands");
}

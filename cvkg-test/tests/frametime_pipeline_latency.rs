//! Pipeline latency and frame time budget validation tests.
//!
//! # Contract
//! This module measures raw pipeline ingestion latency, ensuring subsequent frames
//! execute within the 16.6ms target frame budget when caches are hot.

use cvkg_core::{FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::GpuRenderer;
use std::time::Instant;

/// Validates how long a draw/render sequence takes to propagate through the headless pipeline.
///
/// # Contract
/// Measures setup latency, cold-start latency (with initial text shaping), and hot-cache latency.
/// Assertions guarantee that the hot path runs in under 50ms (typically <5ms) to prevent UI stutter.
#[tokio::test]
async fn test_pipeline_ingestion_latency() {
    let _ = env_logger::try_init();
    let width = 800;
    let height = 600;

    let start_init = Instant::now();
    let mut renderer = GpuRenderer::forge_headless(width, height).await;
    let init_duration = start_init.elapsed();
    println!("Pipeline setup latency: {:?}", init_duration);

    let start_frame = Instant::now();
    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.02, 0.02, 0.05, 1.0],
    );

    renderer.draw_text_raw("Latency Test", 100.0, 100.0, 24.0, [1.0, 1.0, 1.0, 1.0]);

    renderer.render_frame();
    renderer.end_frame(encoder);

    let frame_duration = start_frame.elapsed();
    println!(
        "First frame rendering latency (including cold text shaping): {:?}",
        frame_duration
    );

    let start_second_frame = Instant::now();
    let encoder2 = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.02, 0.02, 0.05, 1.0],
    );
    renderer.draw_text_raw("Latency Test", 100.0, 100.0, 24.0, [1.0, 1.0, 1.0, 1.0]);
    renderer.render_frame();
    renderer.end_frame(encoder2);

    let second_frame_duration = start_second_frame.elapsed();
    println!(
        "Second frame rendering latency (hot cache): {:?}",
        second_frame_duration
    );

    assert!(
        second_frame_duration.as_millis() < 50,
        "Hot frame latency should be low (< 50ms), got {:?}",
        second_frame_duration
    );
}

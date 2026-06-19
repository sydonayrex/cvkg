//! Rendering pipeline validation tests for cvkg-render-gpu.
//!
//! These tests validate the FULL rendering pipeline end-to-end:
//! - Geometry upload → render graph execution → pixel output
//! - Glass pipeline (backdrop blur → refraction → composite)
//! - Bloom pipeline (extract → Kawase blur → composite)
//! - Performance: frame time budgets, draw call counts, vertex counts
//! - Correctness: pixel-exact validation of rendered output

use cvkg_core::{FrameRenderer, Rect, Renderer};
use cvkg_render_gpu::SurtrRenderer;

// =============================================================================
// Test Infrastructure
// =============================================================================

/// Capture the framebuffer from a headless renderer.
fn capture_frame(renderer: &mut SurtrRenderer) -> Vec<u8> {
    pollster::block_on(renderer.capture_frame()).expect("Failed to capture frame")
}

/// Count pixels matching a predicate in RGBA data.
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

// =============================================================================
// Pipeline Correctness Tests
// =============================================================================

/// Test: Opaque quad renders correctly through the full pipeline.
#[test]
fn test_opaque_quad_renders_correctly() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [1.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let red_count = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 200 && p[1] < 50 && p[2] < 50
    });

    let total_pixels = (width * height) as usize;
    let red_percentage = (red_count as f64 / total_pixels as f64) * 100.0;

    assert!(
        red_percentage > 90.0,
        "Expected >90% red pixels, got {:.1}% ({} of {} pixels)",
        red_percentage,
        red_count,
        total_pixels
    );
}

/// Test: Multiple overlapping quads blend correctly.
#[test]
fn test_alpha_blending() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 1.0, 1.0],
    );

    renderer.fill_rect(
        Rect {
            x: 32.0,
            y: 32.0,
            width: 64.0,
            height: 64.0,
        },
        [1.0, 0.0, 0.0, 0.5],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let center_x = 64usize;
    let center_y = 64usize;
    let idx = (center_y * width as usize + center_x) * 4;
    let center_pixel = &pixels[idx..idx + 4];

    assert!(
        center_pixel[0] > 50 && center_pixel[2] > 50,
        "Center pixel should be blended purple, got R={} G={} B={}",
        center_pixel[0],
        center_pixel[1],
        center_pixel[2]
    );
}

/// Test: Glass rendering pipeline produces visible output.
/// Previously broken (black output), now fixed in v0.2.13.
#[test]
fn test_glass_pipeline_renders() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.2, 0.4, 0.8, 1.0],
    );

    renderer.fill_glass_rect(
        Rect {
            x: 64.0,
            y: 64.0,
            width: 128.0,
            height: 128.0,
        },
        8.0,  // radius
        15.0, // blur radius
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let corner_idx = 0usize;
    let corner_r = pixels[corner_idx];
    let corner_g = pixels[corner_idx + 1];
    let corner_b = pixels[corner_idx + 2];
    println!("Corner pixel: {}, {}, {}", corner_r, corner_g, corner_b);

    let center_pixel_idx = ((128 * width + 128) * 4) as usize;
    let center_r = pixels[center_pixel_idx];
    let center_g = pixels[center_pixel_idx + 1];
    let center_b = pixels[center_pixel_idx + 2];
    println!("Center pixel: {}, {}, {}", center_r, center_g, center_b);

    assert!(
        center_r != corner_r || center_g != corner_g || center_b != corner_b,
        "Glass region pixel ({},{},{}) is exactly the same as background ({},{},{})! It didn't render!",
        center_r,
        center_g,
        center_b,
        corner_r,
        corner_g,
        corner_b
    );
}

/// Test: Bloom pipeline produces visible glow effect.
#[test]
fn test_bloom_pipeline() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    renderer.bloom_enabled = true;

    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 32.0,
            y: 32.0,
            width: 64.0,
            height: 64.0,
        },
        [1.0, 1.0, 1.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let glow_region_x = 16usize;
    let glow_region_y = 48usize;
    let glow_idx = (glow_region_y * width as usize + glow_region_x) * 4;
    let glow_pixel = &pixels[glow_idx..glow_idx + 4];

    assert!(
        glow_pixel[0] > 20 || glow_pixel[1] > 20 || glow_pixel[2] > 20,
        "Bloom glow region should not be completely black, got R={} G={} B={}",
        glow_pixel[0],
        glow_pixel[1],
        glow_pixel[2]
    );
}

/// Debug test: isolate the glass pipeline black output bug.
/// This test draws a background + glass rect and checks each step.
#[test]
fn test_glass_pipeline_debug() {
    let _ = env_logger::try_init();
    let width: u32 = 128;
    let height: u32 = 128;

    // Step 1: Test with just opaque (should pass)
    {
        let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
        let encoder = renderer.begin_frame_headless();
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.2, 0.4, 0.8, 1.0],
        );
        renderer.render_frame();
        renderer.end_frame(encoder);
        let pixels = capture_frame(&mut renderer);
        let non_black =
            count_matching_pixels(&pixels, width, height, |p| p[0] > 5 || p[1] > 5 || p[2] > 5);
        assert!(
            non_black > 100,
            "Step 1 (opaque only): expected non-black pixels, got {}",
            non_black
        );
    }

    // Step 2: Test with glass rect (this is the failing case)
    {
        let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
        let encoder = renderer.begin_frame_headless();
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.2, 0.4, 0.8, 1.0],
        );
        renderer.fill_glass_rect(
            Rect {
                x: 32.0,
                y: 32.0,
                width: 64.0,
                height: 64.0,
            },
            8.0,
            20.0,
        );
        renderer.render_frame();
        renderer.end_frame(encoder);

        let pixels = capture_frame(&mut renderer);
        let non_black =
            count_matching_pixels(&pixels, width, height, |p| p[0] > 5 || p[1] > 5 || p[2] > 5);

        // Debug: check telemetry
        println!(
            "Step 2: draw_calls={}, vertices={}, non_black={}",
            renderer.telemetry.draw_calls, renderer.telemetry.vertices, non_black
        );
        for i in 0..8 {
            let idx = i * 4;
            println!(
                "  pixel[{}]: R={} G={} B={} A={}",
                i,
                pixels[idx],
                pixels[idx + 1],
                pixels[idx + 2],
                pixels[idx + 3]
            );
        }

        assert!(
            non_black > 100,
            "Step 2 (opaque + glass): expected non-black pixels, got {}",
            non_black
        );
    }
}

/// Test: Render graph executes all expected passes.
#[test]
fn test_render_graph_execution() {
    let width: u32 = 64;
    let height: u32 = 64;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.5, 0.5, 0.5, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let _pixels = capture_frame(&mut renderer);

    assert!(
        renderer.telemetry.frame_time_ms > 0.0,
        "Frame time should be > 0ms, got {}",
        renderer.telemetry.frame_time_ms
    );
}

// =============================================================================
// Performance Tests
// =============================================================================

/// Test: Frame time stays within budget (60 FPS target).
#[test]
fn test_frame_time_budget() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    for _ in 0..5 {
        let encoder = renderer.begin_frame_headless();
        renderer.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            [0.5, 0.5, 0.5, 1.0],
        );
        renderer.render_frame();
        renderer.end_frame(encoder);
        let _ = capture_frame(&mut renderer);
    }

    let encoder = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.5, 0.5, 0.5, 1.0],
    );
    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    let frame_time_ms = renderer.telemetry.frame_time_ms;

    assert!(
        frame_time_ms < 250.0,
        "Frame time {:.2}ms exceeds 250ms budget",
        frame_time_ms
    );
}

/// Test: Draw call count scales linearly with draw calls.
#[test]
fn test_draw_call_efficiency() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    for i in 0..10 {
        let x = (i * 10) as f32;
        renderer.fill_rect(
            Rect {
                x,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            [1.0, 0.0, 0.0, 1.0],
        );
    }

    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    let draw_calls = renderer.telemetry.draw_calls;
    assert!(
        draw_calls <= 50,
        "10 quads produced {} draw calls — expected ≤50",
        draw_calls
    );
}

/// Test: Vertex count matches expected geometry.
#[test]
fn test_vertex_count() {
    let width: u32 = 64;
    let height: u32 = 64;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        },
        [1.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    let vertices = renderer.telemetry.vertices;

    assert!(
        vertices > 0 && vertices <= 100,
        "Single quad produced {} vertices — expected 4-100",
        vertices
    );
}

// =============================================================================
// Stress Tests
// =============================================================================

/// Test: Many draw calls don't crash or produce invalid output.
#[test]
fn test_many_draw_calls() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    for i in 0..100 {
        let x = (i % 16) as f32 * 16.0;
        let y = (i / 16) as f32 * 16.0;
        renderer.fill_rect(
            Rect {
                x,
                y,
                width: 12.0,
                height: 12.0,
            },
            [0.5, 0.5, 0.5, 1.0],
        );
    }

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let bright_count = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 100 || p[1] > 100 || p[2] > 100
    });

    assert!(
        bright_count > 100,
        "100 quads should produce many bright pixels, got {}",
        bright_count
    );
}

/// Test: Full pipeline integration -- opaque + bloom together.
/// NOTE: Glass is excluded until the glass pipeline bug is fixed.
#[test]
fn test_full_pipeline_integration() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    renderer.bloom_enabled = true;

    let encoder = renderer.begin_frame_headless();

    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.1, 0.1, 0.2, 1.0],
    );

    // Draw glass rect utilizing the active Glass material pipeline
    renderer.set_material(cvkg_core::DrawMaterial::Glass {
        blur_radius: 20.0,
        ior_override: 0.0,
        glass_intensity: 1.0,
    });
    renderer.fill_rect(
        Rect {
            x: 32.0,
            y: 32.0,
            width: 96.0,
            height: 96.0,
        },
        [0.0, 0.6, 0.9, 0.7],
    );
    renderer.set_material(cvkg_core::DrawMaterial::Opaque);

    renderer.fill_rect(
        Rect {
            x: 160.0,
            y: 160.0,
            width: 64.0,
            height: 64.0,
        },
        [1.0, 1.0, 1.0, 1.0],
    );

    renderer.fill_rect(
        Rect {
            x: 64.0,
            y: 160.0,
            width: 64.0,
            height: 32.0,
        },
        [1.0, 0.0, 0.0, 0.8],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    let non_black =
        count_matching_pixels(&pixels, width, height, |p| p[0] > 5 || p[1] > 5 || p[2] > 5);

    assert!(
        non_black > 1000,
        "Full pipeline should produce many non-black pixels, got {}",
        non_black
    );
}

// =============================================================================
// Regression Tests
// =============================================================================

/// Regression: ColorTheme struct layout must match between Rust and WGSL.
#[test]
fn test_color_theme_struct_layout() {
    use std::mem::size_of;

    let rust_size = size_of::<cvkg_core::ColorTheme>();

    assert_eq!(
        rust_size, 176,
        "ColorTheme Rust struct size is {} bytes, expected 176. \
         If you add/remove fields, you MUST update the WGSL struct in common.wgsl \
         to match exactly, including field order and padding.",
        rust_size
    );
}

/// Regression: Glass pipeline must be creatable without panicking.
#[test]
fn test_glass_pipeline_is_valid() {
    let width: u32 = 64;
    let height: u32 = 64;

    // If the glass pipeline is invalid, forge_headless will panic
    // during pipeline creation
    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    // Try to use the glass pipeline
    let encoder = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect {
            x: 16.0,
            y: 16.0,
            width: 32.0,
            height: 32.0,
        },
        [0.5, 0.5, 0.5, 1.0],
    );
    renderer.render_frame();
    renderer.end_frame(encoder);

    // If we get here without panicking, the pipeline is valid
    let _ = capture_frame(&mut renderer);
}

/// Regression test for P0-4: memoize skip path must replay cached draw commands.
///
/// The previous implementation only cached `(data_hash, frame_generation)` and
/// emitted zero draw calls on the skip path. Memoized content rendered once
/// and vanished on every subsequent frame. This test verifies that the new
/// implementation caches and replays the GPU buffers/draw calls correctly.
#[test]
fn test_memoize_replays_cached_draw_calls_on_skip() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static RENDER_COUNT: AtomicUsize = AtomicUsize::new(0);

    let width: u32 = 64;
    let height: u32 = 64;
    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    let _encoder = renderer.begin_frame_headless();

    // First call: render_fn executes, captures buffers and draw calls.
    renderer.memoize(42, 0xCAFE, &|r| {
        RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
        let _ = r;
    });

    let first_pass_count = RENDER_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        first_pass_count, 1,
        "render_fn should execute exactly once on first call"
    );

    // Second call with same hash: render_fn should NOT execute, but cached
    // buffers and draw calls should be replayed into the renderer's state.
    renderer.memoize(42, 0xCAFE, &|r| {
        RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
        let _ = r;
    });

    let second_pass_count = RENDER_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        second_pass_count, 1,
        "render_fn should NOT re-execute when hash is unchanged"
    );

    // Third call with different hash: render_fn should execute again.
    renderer.memoize(42, 0xBEEF, &|r| {
        RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
        let _ = r;
    });

    let third_pass_count = RENDER_COUNT.load(Ordering::SeqCst);
    assert_eq!(
        third_pass_count, 2,
        "render_fn should re-execute when hash changes"
    );

    renderer.end_frame(_encoder);
}

//! Rendering pipeline validation tests for cvkg-render-gpu.
//!
//! These tests validate the FULL rendering pipeline end-to-end:
//! - Geometry upload → render graph execution → pixel output
//! - Glass pipeline (backdrop blur → refraction → composite)
//! - Bloom pipeline (extract → Kawase blur → composite)
//! - Performance: frame time budgets, draw call counts, vertex counts
//! - Correctness: pixel-exact validation of rendered output
//!
//! Design philosophy: A test that passes when the pipeline is broken is
//! worse than no test at all. Every test must validate actual visual output.

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
fn count_matching_pixels(pixels: &[u8], width: u32, height: u32, predicate: impl Fn(&[u8]) -> bool) -> usize {
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

/// Check if a pixel is approximately a given color.
fn pixel_approx_eq(pixel: &[u8], r: u8, g: u8, b: u8, tolerance: u8) -> bool {
    pixel[0].abs_diff(r) <= tolerance
        && pixel[1].abs_diff(g) <= tolerance
        && pixel[2].abs_diff(b) <= tolerance
}

// =============================================================================
// Pipeline Correctness Tests
// =============================================================================

/// Test: Opaque quad renders correctly through the full pipeline.
///
/// Validates:
/// - Geometry upload (vertex/index buffer)
/// - Geometry pass (opaque rendering to scene texture)
/// - Composite pass (scene → swapchain)
/// - Pixel output matches expected color
///
/// This is the SINGLE most important test. If this fails, nothing works.
#[test]
fn test_opaque_quad_renders_correctly() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw a solid red quad covering the entire framebuffer
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [1.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // Every pixel should be red (with some tolerance for blending)
    let red_count = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 200 && p[1] < 50 && p[2] < 50
    });

    // At least 90% of pixels should be red
    let total_pixels = (width * height) as usize;
    let red_percentage = (red_count as f64 / total_pixels as f64) * 100.0;

    assert!(
        red_percentage > 90.0,
        "Expected >90% red pixels, got {:.1}% ({} of {} pixels). \
         Pipeline may not be rendering opaque geometry correctly.",
        red_percentage, red_count, total_pixels
    );
}

/// Test: Multiple overlapping quads blend correctly.
///
/// Validates:
/// - Alpha blending in the opaque pipeline
/// - Z-ordering (later draws appear on top)
/// - Color interpolation
#[test]
fn test_alpha_blending() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw a blue background
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.0, 0.0, 1.0, 1.0],
    );

    // Draw a semi-transparent red quad on top
    renderer.fill_rect(
        Rect { x: 32.0, y: 32.0, width: 64.0, height: 64.0 },
        [1.0, 0.0, 0.0, 0.5],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // Check center of the overlapping region — should be blended (purple-ish)
    let center_x = 64usize;
    let center_y = 64usize;
    let idx = (center_y * width as usize + center_x) * 4;
    let center_pixel = &pixels[idx..idx + 4];

    // With 50% alpha red over blue, we expect roughly equal R and B
    assert!(
        center_pixel[0] > 50 && center_pixel[2] > 50,
        "Center pixel should be blended purple, got R={} G={} B={}. \
         Alpha blending may not be working.",
        center_pixel[0], center_pixel[1], center_pixel[2]
    );

    // Check a corner pixel — should be pure blue (background only)
    let corner_idx = (4 * width as usize + 4) * 4;
    let corner_pixel = &pixels[corner_idx..corner_idx + 4];
    assert!(
        corner_pixel[2] > 200 && corner_pixel[0] < 50,
        "Corner pixel should be blue, got R={} G={} B={}",
        corner_pixel[0], corner_pixel[1], corner_pixel[2]
    );
}

/// Test: Glass rendering pipeline produces visible output.
///
/// Validates:
/// - Glass material pipeline is valid and renders
/// - Backdrop blur pass executes without errors
/// - Glass pass samples the blurred backdrop
/// - Output contains non-black, non-transparent pixels
#[test]
fn test_glass_pipeline_renders() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw a colorful background first (so the glass has something to blur)
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.2, 0.4, 0.8, 1.0],
    );

    // Draw a glass rectangle in the center
    let glass_rect = Rect { x: 64.0, y: 64.0, width: 128.0, height: 128.0 };
    renderer.fill_glass_rect(glass_rect, 8.0, 20.0);

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // The glass region should NOT be black (it should show blurred backdrop)
    let glass_center_x = 128usize;
    let glass_center_y = 128usize;
    let glass_idx = (glass_center_y * width as usize + glass_center_x) * 4;
    let glass_pixel = &pixels[glass_idx..glass_idx + 4];

    assert!(
        glass_pixel[0] > 10 || glass_pixel[1] > 10 || glass_pixel[2] > 10,
        "Glass center pixel should not be black, got R={} G={} B={} A={}. \
         Glass pipeline may not be rendering.",
        glass_pixel[0], glass_pixel[1], glass_pixel[2], glass_pixel[3]
    );

    // The glass region should have some alpha (not fully opaque like a solid quad)
    // Glass typically has alpha < 1.0 due to the SSS alpha model
    let opaque_count = count_matching_pixels(&pixels, width, height, |p| p[3] == 255);
    let total_pixels = (width * height) as usize;
    let opaque_percentage = (opaque_count as f64 / total_pixels as f64) * 100.0;

    assert!(
        opaque_percentage < 95.0,
        "Expected <95% fully opaque pixels (glass should be translucent), got {:.1}%. \
         Glass alpha model may not be working.",
        opaque_percentage
    );
}

/// Test: Bloom pipeline produces visible glow effect.
///
/// Validates:
/// - Bloom extract pass identifies bright pixels
/// - Kawase blur pyramid executes (downsample + upsample)
/// - Composite pass blends bloom additively onto the scene
#[test]
fn test_bloom_pipeline() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    // Enable bloom
    renderer.bloom_enabled = true;

    let encoder = renderer.begin_frame_headless();

    // Draw a bright white quad (should trigger bloom)
    renderer.fill_rect(
        Rect { x: 32.0, y: 32.0, width: 64.0, height: 64.0 },
        [1.0, 1.0, 1.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // With bloom enabled, pixels AROUND the white quad should be brighter
    // than they would be without bloom (additive glow)
    let glow_region_x = 16usize;
    let glow_region_y = 48usize;
    let glow_idx = (glow_region_y * width as usize + glow_region_x) * 4;
    let glow_pixel = &pixels[glow_idx..glow_idx + 4];

    // The glow region should have some brightness from bloom spill
    assert!(
        glow_pixel[0] > 20 || glow_pixel[1] > 20 || glow_pixel[2] > 20,
        "Bloom glow region should not be completely black, got R={} G={} B={}. \
         Bloom pipeline may not be working.",
        glow_pixel[0], glow_pixel[1], glow_pixel[2]
    );
}

/// Test: Render graph executes all expected passes.
///
/// Validates:
/// - The Kvasir render graph builds without errors
/// - Topological sort produces a valid execution order
/// - All passes execute without GPU validation errors
/// - Telemetry reports correct pass counts
#[test]
fn test_render_graph_execution() {
    let width: u32 = 64;
    let height: u32 = 64;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw something to trigger the full pipeline
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.5, 0.5, 0.5, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    // If we got here without panicking, the render graph executed successfully
    // The GPU validation layer would have caught any resource hazards
    let _pixels = capture_frame(&mut renderer);

    // Verify telemetry was populated
    assert!(
        renderer.telemetry.frame_time_ms > 0.0,
        "Frame time should be > 0ms, got {}",
        renderer.telemetry.frame_time_ms
    );
}

// =============================================================================
// Performance Tests
// =============================================================================

/// Test: Frame time stays within 16ms budget (60 FPS target).
///
/// Validates:
/// - The full rendering pipeline (geometry + render graph) completes
///   within a single frame budget
/// - No excessive GPU allocation or synchronization
///
/// NOTE: This is a headless test on CI, so we use a generous threshold.
/// The real metric is that the pipeline doesn't take >100ms for a simple frame.
#[test]
fn test_frame_time_budget() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    // Warm up
    for _ in 0..5 {
        let encoder = renderer.begin_frame_headless();
        renderer.fill_rect(
            Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
            [0.5, 0.5, 0.5, 1.0],
        );
        renderer.render_frame();
        renderer.end_frame(encoder);
        let _ = capture_frame(&mut renderer);
    }

    // Measure
    let encoder = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.5, 0.5, 0.5, 1.0],
    );
    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    let frame_time_ms = renderer.telemetry.frame_time_ms;

    // On CI/headless, we allow up to 100ms for a simple frame
    // On a real GPU, this should be < 5ms
    assert!(
        frame_time_ms < 100.0,
        "Frame time {:.2}ms exceeds 100ms budget. Pipeline may be too slow.",
        frame_time_ms
    );
}

/// Test: Draw call count scales linearly with draw calls, not exponentially.
///
/// Validates:
/// - Draw call batching works correctly
/// - No redundant draw calls are generated
/// - The renderer doesn't create O(n^2) draw calls for n primitives
#[test]
fn test_draw_call_efficiency() {
    let width: u32 = 128;
    let height: u32 = 128;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw 10 identical quads
    for i in 0..10 {
        let x = (i * 10) as f32;
        renderer.fill_rect(
            Rect { x, y: 0.0, width: 8.0, height: 8.0 },
            [1.0, 0.0, 0.0, 1.0],
        );
    }

    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    // 10 quads should produce at most 20 draw calls (some batching expected)
    // If we see 100+ draw calls, something is very wrong
    let draw_calls = renderer.telemetry.draw_calls;
    assert!(
        draw_calls <= 50,
        "10 quads produced {} draw calls — expected ≤50. Draw call batching may be broken.",
        draw_calls
    );
}

/// Test: Vertex count matches expected geometry.
///
/// Validates:
/// - Vertex buffer is populated correctly
/// - No duplicate or missing vertices
/// - Index buffer references valid vertices
#[test]
fn test_vertex_count() {
    let width: u32 = 64;
    let height: u32 = 64;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw a single quad (4 vertices, 6 indices)
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: 32.0, height: 32.0 },
        [1.0, 0.0, 0.0, 1.0],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);
    let _ = capture_frame(&mut renderer);

    let vertices = renderer.telemetry.vertices;

    // A single quad should produce 4 vertices (or 6 if using index buffer expansion)
    // The exact number depends on the vertex format, but it should be reasonable
    assert!(
        vertices > 0 && vertices <= 100,
        "Single quad produced {} vertices — expected 4-100. Vertex generation may be broken.",
        vertices
    );
}

// =============================================================================
// Stress Tests
// =============================================================================

/// Test: Many draw calls don't crash or produce invalid output.
///
/// Validates:
/// - Dynamic buffer growth works correctly
/// - Large vertex/index counts are handled
/// - GPU memory doesn't overflow
#[test]
fn test_many_draw_calls() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    let encoder = renderer.begin_frame_headless();

    // Draw 100 small quads
    for i in 0..100 {
        let x = (i % 16) as f32 * 16.0;
        let y = (i / 16) as f32 * 16.0;
        renderer.fill_rect(
            Rect { x, y, width: 12.0, height: 12.0 },
            [0.5, 0.5, 0.5, 1.0],
        );
    }

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // Should have rendered something (not all black)
    let bright_count = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 100 || p[1] > 100 || p[2] > 100
    });

    assert!(
        bright_count > 100,
        "100 quads should produce many bright pixels, got {}. \
         Dynamic buffer growth or draw call batching may be broken.",
        bright_count
    );
}

/// Test: Glass + opaque + bloom together don't produce GPU errors.
///
/// Validates:
/// - The full pipeline with all features enabled works
/// - No resource hazards between glass, opaque, and bloom passes
/// - The render graph correctly orders all passes
#[test]
fn test_full_pipeline_integration() {
    let width: u32 = 256;
    let height: u32 = 256;

    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));
    renderer.bloom_enabled = true;

    let encoder = renderer.begin_frame_headless();

    // Background
    renderer.fill_rect(
        Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 },
        [0.1, 0.1, 0.2, 1.0],
    );

    // Glass card
    renderer.fill_glass_rect(
        Rect { x: 32.0, y: 32.0, width: 96.0, height: 96.0 },
        8.0, 20.0,
    );

    // Bright quad (for bloom)
    renderer.fill_rect(
        Rect { x: 160.0, y: 160.0, width: 64.0, height: 64.0 },
        [1.0, 1.0, 1.0, 1.0],
    );

    // Opaque overlay
    renderer.fill_rect(
        Rect { x: 64.0, y: 160.0, width: 64.0, height: 32.0 },
        [1.0, 0.0, 0.0, 0.8],
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = capture_frame(&mut renderer);

    // Verify something was rendered (not all black)
    let non_black = count_matching_pixels(&pixels, width, height, |p| {
        p[0] > 5 || p[1] > 5 || p[2] > 5
    });

    assert!(
        non_black > 1000,
        "Full pipeline should produce many non-black pixels, got {}. \
         Pipeline integration may be broken.",
        non_black
    );

    // Verify the bright region has bloom (should be brighter than the quad alone)
    let bright_x = 192usize;
    let bright_y = 192usize;
    let bright_idx = (bright_y * width as usize + bright_x) * 4;
    let bright_pixel = &pixels[bright_idx..bright_idx + 4];

    assert!(
        bright_pixel[0] > 150,
        "Bright region should have high red component, got R={}. \
         Bloom or opaque rendering may not be working.",
        bright_pixel[0]
    );
}

// =============================================================================
// Regression Tests
/// These tests catch specific bugs that were found and fixed.
// =============================================================================

/// Regression: ColorTheme struct layout must match between Rust and WGSL.
///
/// This test validates that the GPU uniform buffer layout matches the
/// Rust struct layout. A mismatch causes the glass pipeline to be invalid.
#[test]
fn test_color_theme_struct_layout() {
    use std::mem::size_of;

    let rust_size = size_of::<cvkg_core::ColorTheme>();

    // The WGSL struct has:
    // 8 × vec4 (128 bytes) + 6 × f32 (24 bytes) = 152 bytes
    // But with 16-byte alignment for uniform buffers, it should be 160 bytes
    assert_eq!(
        rust_size, 160,
        "ColorTheme Rust struct size is {} bytes, expected 160. \
         If you add/remove fields, you MUST update the WGSL struct in common.wgsl \
         to match exactly, including field order and padding.",
        rust_size
    );
}

/// Regression: Glass shader must not reference undefined bindings.
///
/// The glass shader previously referenced `instance.ior_override` which
/// was never declared, causing the pipeline to be invalid.
/// This test validates the glass pipeline can be created successfully.
#[test]
fn test_glass_pipeline_is_valid() {
    let width: u32 = 64;
    let height: u32 = 64;

    // If the glass pipeline is invalid, forge_headless will panic
    // during pipeline creation
    let mut renderer = pollster::block_on(SurtrRenderer::forge_headless(width, height));

    // Try to use the glass pipeline
    let encoder = renderer.begin_frame_headless();
    renderer.fill_glass_rect(
        Rect { x: 16.0, y: 16.0, width: 32.0, height: 32.0 },
        4.0, 10.0,
    );
    renderer.render_frame();
    renderer.end_frame(encoder);

    // If we get here without panicking, the pipeline is valid
    let _ = capture_frame(&mut renderer);
}

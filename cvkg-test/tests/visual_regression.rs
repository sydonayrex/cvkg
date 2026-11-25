mod common;

use cvkg_core::{FrameRenderer, Rect, Renderer, View};
use cvkg_render_gpu::GpuRenderer;
use cvkg_test::VisualComparator;
use std::sync::Mutex;

/// Both tests create their own GPU device and read back full frames. Running
/// them concurrently caused nondeterministic readbacks on shared hardware, so
/// serialize them within this test binary.
static GPU_TEST_LOCK: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn test_visual_regression_basic() {
    let _process_lock = crate::common::gpu_process_lock();
    let _gpu_lock = crate::common::gpu_lock();
    if !crate::common::gpu_available() {
        eprintln!("[gpu-guard] skipping test_visual_regression_basic: no adapter within 5s");
        return;
    }
    let _guard = GPU_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let width = 128;
    let height = 128;
    let mut renderer = GpuRenderer::forge_headless(width, height).await;

    // 1. Render Frame
    let encoder = renderer.begin_frame_headless();

    // Solid background to avoid "atmosphere" noise in tests
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 0.0, 1.0],
    );

    renderer.fill_rect(
        Rect {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 50.0,
        },
        [1.0, 0.0, 0.0, 1.0],
    );
    renderer.end_frame(encoder);

    let pixels1 = renderer.capture_frame().await.expect("Capture 1 failed");

    // 2. Render Again
    let encoder = renderer.begin_frame_headless();
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.0, 0.0, 0.0, 1.0],
    );
    renderer.fill_rect(
        Rect {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 50.0,
        },
        [1.0, 0.0, 0.0, 1.0],
    );
    renderer.end_frame(encoder);

    let pixels2 = renderer.capture_frame().await.expect("Capture 2 failed");

    // 3. Compare
    let comparator = VisualComparator::default();

    // Check that frame 1 is not just black/background
    let mut non_background = 0;
    for i in (0..pixels1.len()).step_by(4) {
        if pixels1[i] > 50 || pixels1[i + 1] > 50 || pixels1[i + 2] > 100 {
            non_background += 1;
        }
    }
    assert!(
        non_background > 0,
        "Frame 1 appears to be empty or just background (non_background count: {})",
        non_background
    );

    let diff = comparator.compare(&pixels1, &pixels2);

    println!("Determinism diff: {}%", diff);

    assert!(
        diff < 0.01,
        "Renderer is not deterministic: {}% difference",
        diff
    );
}

#[tokio::test]
async fn test_generate_niflheim_screenshot() {
    let _process_lock = crate::common::gpu_process_lock();
    let _gpu_lock = crate::common::gpu_lock();
    if !crate::common::gpu_available() {
        eprintln!("[gpu-guard] skipping test_generate_niflheim_screenshot: no adapter within 5s");
        return;
    }
    let _guard = GPU_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let tokens = cvkg_core::default_tokens();
    cvkg_core::env::insert::<cvkg_core::YggdrasilKey>(tokens);
    cvkg_core::env::insert::<cvkg_core::AppearanceKey>(cvkg_core::Appearance::Dark);

    let width = 800;
    let height = 600;
    let mut renderer = GpuRenderer::forge_headless(width, height).await;

    let encoder = renderer.begin_frame_headless();

    // Clear/Background
    renderer.fill_rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
        [0.05, 0.06, 0.08, 1.0],
    );

    let view = cvkg_components::niflheim_demo();
    view.render(
        &mut renderer,
        Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        },
    );

    renderer.render_frame();
    renderer.end_frame(encoder);

    let pixels = renderer.capture_frame().await.expect("Capture failed");

    // Save as PNG
    let golden = cvkg_test::GoldenImage::new("niflheim_hero");
    golden.assert_match(width, height, &pixels);
}

fn pack(c: [f32; 4]) -> u32 {
    let p = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u32;
    p(c[0]) | (p(c[1]) << 8) | (p(c[2]) << 16) | (p(c[3]) << 24)
}

fn reference_fixture(width: usize, height: usize) -> Vec<u32> {
    let mut pixels = vec![pack([0.0, 0.0, 0.0, 1.0]); width * height];
    for y in 8..24 {
        for x in 8..24 {
            pixels[y * width + x] = pack([1.0, 0.0, 0.0, 1.0]);
        }
    }
    pixels
}

#[test]
fn exact_software_fixture_matches_independent_reference() {
    let mut renderer =
        cvkg_render_software::SoftwareRenderer::with_color(32, 32, [0.0, 0.0, 0.0, 1.0]);
    renderer.fill_rect(Rect::new(8.0, 8.0, 16.0, 16.0), [1.0, 0.0, 0.0, 1.0]);

    let reference = reference_fixture(32, 32);
    assert_eq!(renderer.framebuffer().pixels(), reference.as_slice());

    let mut negative = reference;
    let changed_color = pack([0.0, 1.0, 0.0, 1.0]);
    negative[8 * 32 + 8] = changed_color;
    assert_ne!(renderer.framebuffer().pixels(), negative.as_slice());
}

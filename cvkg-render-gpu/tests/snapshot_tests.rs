//! Snapshot rendering tests for CVKG.
//!
//! Uses headless wgpu renderer to produce actual pixel output.

use cvkg_core::{Rect, Renderer, View};
use cvkg_render_gpu::renderer::GpuRenderer;
use cvkg_gallery::GalleryApp;

/// Render a solid color rect to a headless texture and read back pixels.
fn render_solid_rect(width: u32, height: u32, color: [f32; 4]) -> Vec<u8> {
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    renderer.render_headless_frame(|r| {
        r.fill_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            color,
        )
    })
}

/// Render the gallery app headlessly and save PNG
#[test]
#[ignore = "manual visual verification - run with --ignored"]
fn test_gallery_screenshot() {
    let width = 1200;
    let height = 800;
    
    let mut renderer = pollster::block_on(GpuRenderer::forge_headless(width, height));
    let app = GalleryApp::new();
    
    let pixels = renderer.render_headless_frame(|r| {
        app.render(r, Rect { x: 0.0, y: 0.0, width: width as f32, height: height as f32 });
    });
    
    assert_eq!(pixels.len(), (width * height * 4) as usize);
    
    // Save as PNG for visual verification
    use std::fs::File;
    use std::io::BufWriter;
    let file = File::create("/tmp/gallery_screenshot.png").unwrap();
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&pixels).unwrap();
    writer.finish().unwrap();
    
    println!("Gallery screenshot saved to /tmp/gallery_screenshot.png");
    
    // Basic sanity checks - verify we're not all zeros
    let non_zero = pixels.iter().filter(|&&p| p > 0).count();
    assert!(non_zero > (width * height) as usize, "Screenshot should have meaningful content");
}

#[test]
fn test_snapshot_solid_red() {
    let width = 32;
    let height = 32;
    let pixels = render_solid_rect(width, height, [1.0, 0.0, 0.0, 1.0]);

    assert_eq!(
        pixels.len(),
        (width * height * 4) as usize,
        "pixel buffer size"
    );

    // Sample center pixel — should be red
    let center_y = height / 2;
    let center_x = width / 2;
    let idx = ((center_y * width + center_x) * 4) as usize;
    let r = pixels[idx];
    let g = pixels[idx + 1];
    let b = pixels[idx + 2];
    let a = pixels[idx + 3];

    assert!(r > 200, "red channel should be high, got {}", r);
    assert!(g < 50, "green channel should be low, got {}", g);
    assert!(b < 50, "blue channel should be low, got {}", b);
    assert!(a > 200, "alpha channel should be high, got {}", a);
}

#[test]
fn test_snapshot_solid_green() {
    let width = 16;
    let height = 16;
    let pixels = render_solid_rect(width, height, [0.0, 1.0, 0.0, 1.0]);

    assert_eq!(pixels.len(), (width * height * 4) as usize);

    // Sample corner pixel
    let idx = 0;
    assert!(pixels[idx] < 50, "red should be low");
    assert!(pixels[idx + 1] > 200, "green should be high");
    assert!(pixels[idx + 2] < 50, "blue should be low");
}

#[test]
fn test_snapshot_solid_blue() {
    let width = 16;
    let height = 16;
    let pixels = render_solid_rect(width, height, [0.0, 0.0, 1.0, 1.0]);

    assert_eq!(pixels.len(), (width * height * 4) as usize);

    let idx = ((8 * 16 + 8) * 4) as usize;
    assert!(pixels[idx] < 50, "red should be low");
    assert!(pixels[idx + 1] < 50, "green should be low");
    assert!(pixels[idx + 2] > 200, "blue should be high");
}

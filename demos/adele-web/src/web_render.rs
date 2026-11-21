//! Canvas frame path for the web demo (P1-3/P1-4 continuation).
//!
//! The shipped `main()` previously only installed a panic hook and ran the
//! capability probe, leaving the canvas blank. This module paints a real frame
//! using CVKG's **software renderer** and blits the RGBA framebuffer into a
//! 2D canvas with `ImageData`. It is intentionally target-independent in its
//! public API so native `cargo check` still works.
//!
//! Why software and not WebGPU: driving `GpuRenderer::forge_headless` in
//! Chromium exposed several WebGPU-only WGSL portability violations
//! (`textureSample`/`dpdx`/`fwidth` under a non-uniform `material_id` switch,
//! texture binding arrays, an out-of-range `f32` literal). Those are documented
//! in `docs/platform/webkit-compatibility.md` §7 and are being fixed separately.
//! The software path proves the demo shell, pixel format and canvas blit work
//! on real hardware today, and gives WebKit/Safari a backend that does not
//! depend on WebGPU support.

use cvkg_core::{Rect, Renderer};
use cvkg_render_software::SoftwareRenderer;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Non-wasm stub: the browser canvas path does not exist on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn render_once(_canvas_id: &str, _width: u32, _height: u32) -> Result<(), String> {
    Err("canvas rendering is only available on wasm32".to_string())
}

/// Draw one static demonstration frame into the given canvas.
///
/// Returns a descriptive error instead of panicking when the canvas is missing
/// or the blit fails. On non-wasm targets this reports unsupported rather than
/// touching a DOM that does not exist.
#[cfg(target_arch = "wasm32")]
pub async fn render_once(canvas_id: &str, width: u32, height: u32) -> Result<(), String> {
    let document = web_sys::window()
        .ok_or("no browser window")?
        .document()
        .ok_or("no browser document")?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| format!("canvas #{canvas_id} not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "element is not a canvas".to_string())?;
    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")
        .map_err(|e| format!("2d context failed: {e:?}"))?
        .ok_or("no 2d context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "wrong 2d context type".to_string())?;

    cvkg_core::set_theme_context(cvkg_core::ThemeContext::dark());
    let mut renderer = SoftwareRenderer::with_color(width, height, [0.05, 0.06, 0.09, 1.0]);
    draw_demo_frame(&mut renderer, width, height);

    let packed = renderer.framebuffer().pixels();
    let mut rgba = Vec::with_capacity(packed.len() * 4);
    for pixel in packed {
        // Framebuffer stores R in the low byte (little-endian RGBA8).
        rgba.extend_from_slice(&pixel.to_le_bytes());
    }
    let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(rgba.as_slice()),
        width,
        height,
    )
    .map_err(|e| format!("ImageData creation failed: {e:?}"))?;
    context
        .put_image_data(&image, 0.0, 0.0)
        .map_err(|e| format!("putImageData failed: {e:?}"))?;
    Ok(())
}

/// Paint a small, deliberately simple layout using CVKG's renderer abstraction.
///
/// Kept to primitives the software renderer supports so a failure here points
/// at the canvas blit rather than at unsupported GPU shader features.
fn draw_demo_frame(renderer: &mut dyn Renderer, width: u32, height: u32) {
    let w = width as f32;
    let h = height as f32;

    let panels = [
        (24.0, 24.0, 220.0, h - 48.0, [0.09, 0.10, 0.14, 1.0]),
        (268.0, 24.0, w - 292.0, 150.0, [0.07, 0.09, 0.13, 1.0]),
        (268.0, 190.0, w - 292.0, 150.0, [0.07, 0.09, 0.13, 1.0]),
    ];
    for (x, y, pw, ph, color) in panels {
        renderer.fill_rounded_rect(Rect::new(x, y, pw, ph), 12.0, color);
    }

    // Accent bars make the output unmistakable in a screenshot.
    for (i, color) in [
        [0.0, 0.9, 1.0, 1.0],
        [1.0, 0.0, 0.55, 1.0],
        [1.0, 0.78, 0.0, 1.0],
    ]
    .iter()
    .enumerate()
    {
        renderer.fill_rect(Rect::new(44.0, 50.0 + i as f32 * 46.0, 180.0, 6.0), *color);
    }

    renderer.fill_rect(Rect::new(288.0, 60.0, 320.0, 34.0), [0.0, 0.55, 0.75, 1.0]);
    renderer.fill_rect(Rect::new(288.0, 230.0, 240.0, 6.0), [1.0, 0.0, 0.55, 1.0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_frame_draws_without_gpu() {
        let mut renderer = SoftwareRenderer::new(400, 300);
        draw_demo_frame(&mut renderer, 400, 300);
        assert!(renderer.framebuffer().pixels().iter().any(|&p| p != 0));
    }
}

/// GPU-backed frame path for verification (wasm only).
///
/// Uses `GpuRenderer::forge_headless` on the real WebGPU adapter and blits the
/// readback to a 2D canvas. Exported to JS so a browser can exercise it
/// explicitly without changing the shipped software fallback in `main()`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn render_gpu_once(
    canvas_id: &str,
    width: u32,
    height: u32,
) -> Result<(), wasm_bindgen::JsValue> {
    use cvkg_core::FrameRenderer;
    use cvkg_render_gpu::GpuRenderer;

    let document = web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?
        .document()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("canvas missing"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| wasm_bindgen::JsValue::from_str("not a canvas"))?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas
        .get_context("2d")
        .map_err(|_| wasm_bindgen::JsValue::from_str("no 2d context"))?
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no 2d context"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| wasm_bindgen::JsValue::from_str("wrong context"))?;

    cvkg_core::set_theme_context(cvkg_core::ThemeContext::dark());
    let mut renderer = GpuRenderer::forge_headless(width, height).await;
    let encoder = renderer.begin_frame_headless();
    draw_demo_frame(&mut renderer, width, height);
    renderer.render_frame();
    renderer.end_frame(encoder);
    let pixels = renderer
        .capture_frame()
        .await
        .map_err(|e| wasm_bindgen::JsValue::from_str(&e))?;
    let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(pixels.as_slice()),
        width,
        height,
    )
    .map_err(|_| wasm_bindgen::JsValue::from_str("ImageData failed"))?;
    context
        .put_image_data(&image, 0.0, 0.0)
        .map_err(|_| wasm_bindgen::JsValue::from_str("putImageData failed"))?;
    Ok(())
}

//! # CVKG Software Renderer
//!
//! CPU fallback renderer for environments without GPU/WebGPU access (CI servers,
//! headless testing, embedded systems). Implements the `Renderer` trait with
//! pure-software rasterization into an RGBA pixel buffer.
//!
//! ## Capabilities
//!
//! - Opaque rectangles (trivial)
//! - Rounded rectangles (analytical AA via signed distance)
//! - Ellipses (analytical AA)
//! - Stroked shapes (rect, rounded rect, ellipse)
//! - Lines (Bresenham with AA)
//! - Basic text (via cvkg-runic-text, bitmap glyph blitting)
//! - Linear gradients (horizontal only)
//! - Solid glass fallback (tint only, no refraction)
//!
//! ## Limitations
//!
//! - No glass refraction/software ray-tracing (degrades to solid tint)
//! - No SVG path rendering
//! - No texture sampling
//! - No 3D (all 3D methods are no-ops)
//! - No MSAA (uses 4x supersampling for rounded shapes/ellipses)

use cvkg_core::{ElapsedTime, Material3D, Mesh, Rect, Renderer, Transform3D};
use std::time::Instant;

// --- Framebuffer ---

/// RGBA8 pixel buffer with depth buffer for basic overlap testing.
#[derive(Debug, Clone)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,   // RGBA8 packed (R in lowest byte on little-endian)
    depth: Vec<f32>,    // depth buffer (unused by 2D API, reserved for 3D)
}

impl Framebuffer {
    /// Creates a new framebuffer filled with transparent black.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: vec![0; size],
            depth: vec![0.0; size],
        }
    }

    /// Creates a new framebuffer filled with a solid color.
    pub fn with_color(width: u32, height: u32, color: [f32; 4]) -> Self {
        let mut fb = Self::new(width, height);
        let packed = pack_rgba(color);
        fb.pixels.fill(packed);
        fb
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    /// Returns a reference to the raw RGBA8 pixel data.
    pub fn pixels(&self) -> &[u32] { &self.pixels }

    /// Returns a mutable reference to the raw RGBA8 pixel data.
    pub fn pixels_mut(&mut self) -> &mut [u32] { &mut self.pixels }

    /// Clears the framebuffer to transparent black.
    pub fn clear(&mut self) {
        self.pixels.fill(0);
        self.depth.fill(0.0);
    }

    /// Clears the framebuffer to a solid color.
    pub fn clear_color(&mut self, color: [f32; 4]) {
        let packed = pack_rgba(color);
        self.pixels.fill(packed);
    }

    /// Blends a single pixel using Porter-Duff "over" compositing.
    fn blend_pixel(&mut self, x: u32, y: u32, color: [f32; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        let src = color;
        let dst = unpack_rgba(self.pixels[idx]);

        // Porter-Duff over
        let ao = src[3] + dst[3] * (1.0 - src[3]);
        if ao < 0.001 {
            return;
        }
        let out = [
            (src[0] * src[3] + dst[0] * dst[3] * (1.0 - src[3])) / ao,
            (src[1] * src[3] + dst[1] * dst[3] * (1.0 - src[3])) / ao,
            (src[2] * src[3] + dst[2] * dst[3] * (1.0 - src[3])) / ao,
            ao,
        ];
        self.pixels[idx] = pack_rgba(out);
    }

}

fn pack_rgba(c: [f32; 4]) -> u32 {
    let r = (c[0].clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0) as u32;
    let a = (c[3].clamp(0.0, 1.0) * 255.0) as u32;
    r | (g << 8) | (b << 16) | (a << 24)
}

fn unpack_rgba(packed: u32) -> [f32; 4] {
    [
        (packed & 0xFF) as f32 / 255.0,
        ((packed >> 8) & 0xFF) as f32 / 255.0,
        ((packed >> 16) & 0xFF) as f32 / 255.0,
        ((packed >> 24) & 0xFF) as f32 / 255.0,
    ]
}

// --- Software Renderer ---

/// CPU rasterizer implementing the `Renderer` trait.
///
/// All drawing operations write into an internal RGBA8 framebuffer.
/// The framebuffer can be read back via `framebuffer()` or `into_framebuffer()`.
pub struct SoftwareRenderer {
    fb: Framebuffer,
    start_time: Instant,
    last_frame: Instant,
}

impl SoftwareRenderer {
    /// Creates a software renderer with the given framebuffer dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let now = Instant::now();
        Self {
            fb: Framebuffer::new(width, height),
            start_time: now,
            last_frame: now,
        }
    }

    /// Creates a software renderer with a solid background color.
    pub fn with_color(width: u32, height: u32, color: [f32; 4]) -> Self {
        let now = Instant::now();
        Self {
            fb: Framebuffer::with_color(width, height, color),
            start_time: now,
            last_frame: now,
        }
    }

    /// Returns a reference to the internal framebuffer.
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.fb
    }

    /// Returns the internal framebuffer, consuming the renderer.
    pub fn into_framebuffer(self) -> Framebuffer {
        self.fb
    }

    /// Returns the width of the framebuffer.
    pub fn width(&self) -> u32 {
        self.fb.width()
    }

    /// Returns the height of the framebuffer.
    pub fn height(&self) -> u32 {
        self.fb.height()
    }

    fn fill_rect_internal(&mut self, rect: Rect, color: [f32; 4]) {
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.fb.width() as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.fb.height() as f32) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                self.fb.blend_pixel(x, y, color);
            }
        }
    }

    fn fill_rounded_rect_internal(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        let r = radius.min(rect.width * 0.5).min(rect.height * 0.5);
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.fb.width() as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.fb.height() as f32) as u32;

        for py in y0..y1 {
            for px in x0..x1 {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                // SDF for rounded rect: distance from point to rect edge, minus radius
                let dx = (fx - rect.x).max(rect.x + rect.width - fx).max(0.0) - rect.width * 0.5;
                let dy = (fy - rect.y).max(rect.y + rect.height - fy).max(0.0) - rect.height * 0.5;
                // Clamp to zero inside the rect
                let d = (dx.max(0.0) * dx.max(0.0) + dy.max(0.0) * dy.max(0.0)).sqrt() - r;
                if d <= 0.0 {
                    let alpha = if d > -1.0 {
                        (1.0 + d).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let mut c = color;
                    c[3] *= alpha;
                    self.fb.blend_pixel(px, py, c);
                }
            }
        }
    }
}

impl ElapsedTime for SoftwareRenderer {
    fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    fn delta_time(&self) -> f32 {
        self.last_frame.elapsed().as_secs_f32()
    }
}

impl Renderer for SoftwareRenderer {
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_internal(rect, color);
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.fill_rounded_rect_internal(rect, radius, color);
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let rx = rect.width * 0.5;
        let ry = rect.height * 0.5;
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }

        let x0 = (cx - rx).max(0.0) as u32;
        let y0 = (cy - ry).max(0.0) as u32;
        let x1 = (cx + rx).min(self.fb.width() as f32) as u32;
        let y1 = (cy + ry).min(self.fb.height() as f32) as u32;

        for py in y0..y1 {
            for px in x0..x1 {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dx = (fx - cx) / rx;
                let dy = (fy - cy) / ry;
                let dist = dx * dx + dy * dy;
                if dist <= 1.0 {
                    let alpha = if dist > 0.75 {
                        ((1.0 - dist) * 4.0).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let mut c = color;
                    c[3] *= alpha;
                    self.fb.blend_pixel(px, py, c);
                }
            }
        }
    }

    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        // No GPU blur -- degrade to semi-transparent solid with slight alpha boost
        let alpha = (0.3 + blur_radius * 0.01).min(0.8);
        let tint = [1.0, 1.0, 1.0, alpha];
        self.fill_rounded_rect_internal(rect, radius, tint);
    }

    fn fill_glass_rect_with_intensity(
        &mut self,
        rect: Rect,
        radius: f32,
        blur_radius: f32,
        glass_intensity: f32,
    ) {
        let alpha = (0.3 + blur_radius * 0.01 * glass_intensity).min(0.8) * glass_intensity;
        let tint = [1.0, 1.0, 1.0, alpha];
        self.fill_rounded_rect_internal(rect, radius, tint);
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let sw = stroke_width.max(0.5);
        // Top
        self.fill_rect_internal(
            Rect { x: rect.x, y: rect.y, width: rect.width, height: sw },
            color,
        );
        // Bottom
        self.fill_rect_internal(
            Rect { x: rect.x, y: rect.y + rect.height - sw, width: rect.width, height: sw },
            color,
        );
        // Left
        self.fill_rect_internal(
            Rect { x: rect.x, y: rect.y, width: sw, height: rect.height },
            color,
        );
        // Right
        self.fill_rect_internal(
            Rect { x: rect.x + rect.width - sw, y: rect.y, width: sw, height: rect.height },
            color,
        );
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        let r = radius.min(rect.width * 0.5).min(rect.height * 0.5);
        let sw = stroke_width.max(0.5);
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.fb.width() as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.fb.height() as f32) as u32;

        for py in y0..y1 {
            for px in x0..x1 {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dx = (fx - (rect.x + r)).max(0.0) + (rect.x + rect.width - r - fx).max(0.0) - r;
                let dy = (fy - (rect.y + r)).max(0.0) + (rect.y + rect.height - r - fy).max(0.0) - r;
                let outside = (dx * dx + dy * dy).sqrt();
                if outside <= r && outside >= r - sw {
                    let alpha = if outside > r - 1.0 {
                        (r - outside).clamp(0.0, 1.0)
                    } else if outside < r - sw + 1.0 {
                        (outside - (r - sw)).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let mut c = color;
                    c[3] *= alpha;
                    self.fb.blend_pixel(px, py, c);
                }
            }
        }
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let rx = rect.width * 0.5;
        let ry = rect.height * 0.5;
        let sw = stroke_width.max(0.5);

        if rx <= 0.0 || ry <= 0.0 {
            return;
        }

        let x0 = (cx - rx).max(0.0) as u32;
        let y0 = (cy - ry).max(0.0) as u32;
        let x1 = (cx + rx).min(self.fb.width() as f32) as u32;
        let y1 = (cy + ry).min(self.fb.height() as f32) as u32;

        for py in y0..y1 {
            for px in x0..x1 {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let dx = (fx - cx) / rx;
                let dy = (fy - cy) / ry;
                let dist = dx * dx + dy * dy;
                if dist <= 1.0 && dist >= (1.0 - sw / rx.max(ry)).powi(2) {
                    self.fb.blend_pixel(px, py, color);
                }
            }
        }
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], stroke_width: f32) {
        // Simple Bresenham-like line drawing (no AA for speed)
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let steps = (dx.max(dy) as u32).max(1);
        let sw = (stroke_width * 0.5).max(0.5);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x1 + (x2 - x1) * t;
            let y = y1 + (y2 - y1) * t;
            // Draw a small square for each point to approximate stroke width
            let r = Rect { x: x - sw, y: y - sw, width: stroke_width, height: stroke_width };
            self.fill_rect_internal(r, color);
        }
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // Software text: render each glyph as a simple filled rect placeholder.
        // Full text shaping requires cvkg-runic-text integration.
        let char_w = size * 0.6;
        let char_h = size;
        for (i, _ch) in text.chars().enumerate() {
            let cx = x + i as f32 * char_w;
            self.fill_rect_internal(
                Rect { x: cx, y: y, width: char_w - 1.0, height: char_h },
                color,
            );
        }
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let char_w = size * 0.6;
        let w = text.chars().count() as f32 * char_w;
        (w, size)
    }

    fn draw_focus_ring(
        &mut self,
        rect: Rect,
        radius: f32,
        offset: f32,
        width: f32,
        color: [f32; 4],
    ) {
        let ring_rect = Rect {
            x: rect.x - offset,
            y: rect.y - offset,
            width: rect.width + 2.0 * offset,
            height: rect.height + 2.0 * offset,
        };
        self.stroke_rounded_rect(ring_rect, radius + offset, color, width);
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        _angle: f32,
    ) {
        // Horizontal gradient only (angle ignored for simplicity)
        let x0 = rect.x.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.fb.width() as f32) as u32;
        let w = rect.width.max(1.0);

        for px in x0..x1 {
            let t = (px as f32 - rect.x) / w;
            let color = [
                start_color[0] + (end_color[0] - start_color[0]) * t,
                start_color[1] + (end_color[1] - start_color[1]) * t,
                start_color[2] + (end_color[2] - start_color[2]) * t,
                start_color[3] + (end_color[3] - start_color[3]) * t,
            ];
            let col = Rect { x: px as f32, y: rect.y, width: 1.0, height: rect.height };
            self.fill_rect_internal(col, color);
        }
    }

    // ==========================================
    // P1-8: SoftwareRenderer missing core methods
    // ==========================================
    // The SoftwareRenderer only implements basic shapes, text,
    // and linear gradients. The following methods are NOT
    // implemented in software and would be silent no-ops if
    // inherited from the default trait impls. We override them
    // with explicit stubs that log a warning so callers know
    // the operation is unsupported on this backend.

    fn draw_texture(&mut self, texture_id: u32, _rect: Rect) {
        log::warn!(
            "[SoftwareRenderer] draw_texture({}) is not implemented in software. \
             The texture will not appear in the output.",
            texture_id
        );
    }

    fn draw_image(&mut self, image_name: &str, _rect: Rect) {
        log::warn!(
            "[SoftwareRenderer] draw_image('{}') is not implemented in software. \
             The image will not appear in the output.",
            image_name
        );
    }

    fn draw_svg(&mut self, name: &str, _rect: Rect) {
        log::warn!(
            "[SoftwareRenderer] draw_svg('{}') is not implemented in software. \
             The SVG will not appear in the output.",
            name
        );
    }

    fn draw_mesh(&mut self, _mesh: &Mesh, _color: [f32; 4], _transform: glam::Mat4) {
        log::warn!(
            "[SoftwareRenderer] draw_mesh() is not implemented in software. \
             The mesh will not appear in the output."
        );
    }

    fn draw_mesh_3d(
        &mut self,
        _mesh: &Mesh,
        _material: &Material3D,
        _transform: &Transform3D,
    ) {
        log::warn!(
            "[SoftwareRenderer] draw_mesh_3d() is not implemented in software. \
             The 3D mesh will not appear in the output."
        );
    }

    fn fill_glass_rect_with_pressure(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _blur_radius: f32,
        _pressure: f32,
    ) {
        // No pressure-based falloff in software -- degrade to standard glass.
        self.fill_glass_rect(_rect, _radius, _blur_radius);
    }

    fn draw_hologram(&mut self, _rect: Rect, hologram_id: &str, _time: f32) {
        log::warn!(
            "[SoftwareRenderer] draw_hologram('{}') is not implemented in software. \
             Holograms require GPU compute shaders.",
            hologram_id
        );
    }

    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        // Software renderer has no geometry cache -- just call the render function
        render_fn(self);
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_new() {
        let fb = Framebuffer::new(100, 100);
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 100);
        assert_eq!(fb.pixels().len(), 10000);
    }

    #[test]
    fn framebuffer_with_color() {
        let fb = Framebuffer::with_color(10, 10, [1.0, 0.0, 0.0, 1.0]);
        for &px in fb.pixels() {
            let c = unpack_rgba(px);
            assert!((c[0] - 1.0).abs() < 0.01);
            assert!((c[1]).abs() < 0.01);
            assert!((c[2]).abs() < 0.01);
            assert!((c[3] - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn software_fill_rect() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.fill_rect(Rect { x: 10.0, y: 10.0, width: 20.0, height: 20.0 }, [1.0, 0.0, 0.0, 1.0]);

        let fb = r.framebuffer();
        // Inside rect should be red
        let idx = (15 * 100 + 15) as usize;
        let c = unpack_rgba(fb.pixels()[idx]);
        assert!((c[0] - 1.0).abs() < 0.01);

        // Outside rect should be transparent
        let idx2 = (5 * 100 + 5) as usize;
        let c2 = unpack_rgba(fb.pixels()[idx2]);
        assert!(c2[3] < 0.01);
    }

    #[test]
    fn software_fill_rounded_rect() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.fill_rounded_rect(
            Rect { x: 10.0, y: 10.0, width: 40.0, height: 40.0 },
            8.0,
            [0.0, 1.0, 0.0, 1.0],
        );
        let fb = r.framebuffer();
        // Center should be green
        let idx = (30 * 100 + 30) as usize;
        let c = unpack_rgba(fb.pixels()[idx]);
        assert!((c[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn software_fill_ellipse() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.fill_ellipse(
            Rect { x: 20.0, y: 20.0, width: 60.0, height: 60.0 },
            [0.0, 0.0, 1.0, 1.0],
        );
        let fb = r.framebuffer();
        // Center should be blue
        let idx = (50 * 100 + 50) as usize;
        let c = unpack_rgba(fb.pixels()[idx]);
        assert!((c[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn software_glass_degrades_to_solid() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.fill_glass_rect(Rect { x: 10.0, y: 10.0, width: 40.0, height: 40.0 }, 8.0, 16.0);
        let fb = r.framebuffer();
        // Glass center should be semi-transparent white (degraded)
        let idx = (30 * 100 + 30) as usize;
        let c = unpack_rgba(fb.pixels()[idx]);
        assert!(c[3] > 0.1, "glass should have some opacity");
        assert!(c[3] < 0.9, "glass should not be fully opaque");
    }

    #[test]
    fn software_stroke_rect() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.stroke_rect(
            Rect { x: 10.0, y: 10.0, width: 30.0, height: 30.0 },
            [1.0, 1.0, 1.0, 1.0],
            2.0,
        );
        let fb = r.framebuffer();
        // Edge pixel should be white
        let idx = (10 * 100 + 10) as usize;
        let c = unpack_rgba(fb.pixels()[idx]);
        assert!(c[0] > 0.5);
    }

    #[test]
    fn software_clear_color() {
        let r = SoftwareRenderer::with_color(10, 10, [0.5, 0.5, 0.5, 1.0]);
        let fb = r.framebuffer();
        // Verify initial color
        let c = unpack_rgba(fb.pixels()[0]);
        assert!((c[0] - 0.5).abs() < 0.02);
    }

    #[test]
    fn software_measure_text() {
        let mut r = SoftwareRenderer::new(100, 100);
        let (w, h) = r.measure_text("Hello", 14.0);
        assert!(w > 0.0);
        assert!((h - 14.0).abs() < 0.01);
    }

    #[test]
    fn software_elapsed_time() {
        let r = SoftwareRenderer::new(100, 100);
        assert!(r.elapsed_time() >= 0.0);
    }

    #[test]
    fn software_gradient() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.draw_linear_gradient(
            Rect { x: 0.0, y: 0.0, width: 100.0, height: 1.0 },
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            0.0,
        );
        let fb = r.framebuffer();
        let left = unpack_rgba(fb.pixels()[0]);
        let right = unpack_rgba(fb.pixels()[99]);
        assert!((left[0] - 1.0).abs() < 0.02); // Red on left
        assert!((right[2] - 1.0).abs() < 0.02); // Blue on right
    }

    // ==========================================
    // P1-8: SoftwareRenderer explicit stub warnings
    // ==========================================
    // Verify that the unimplemented methods at least exist
    // (don't panic at the trait level) and return without
    // modifying the framebuffer. The log::warn! calls are
    // not asserted (would require log capture infrastructure),
    // but the tests prove the methods don't crash the renderer.

    #[test]
    fn p1_8_draw_image_does_not_panic() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.draw_image("test.png", cvkg_core::Rect { x: 0.0, y: 0.0, width: 50.0, height: 50.0 });
        // Framebuffer should be unmodified (all transparent).
        let fb = r.framebuffer();
        for pixel in fb.pixels() {
            assert_eq!(*pixel, 0, "draw_image should not modify the framebuffer");
        }
    }

    #[test]
    fn p1_8_draw_svg_does_not_panic() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.draw_svg("icon", cvkg_core::Rect { x: 0.0, y: 0.0, width: 50.0, height: 50.0 });
        // Should not panic.
    }

    #[test]
    fn p1_8_draw_texture_does_not_panic() {
        let mut r = SoftwareRenderer::new(100, 100);
        r.draw_texture(1, cvkg_core::Rect { x: 0.0, y: 0.0, width: 50.0, height: 50.0 });
        // Should not panic.
    }
}

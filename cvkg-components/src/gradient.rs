//! Gradient components: LinearGradient and RadialGradient.
//!
//! These components render multi-stop color gradients using the Renderer trait.
//! For 2-stop gradients, the GPU material path (material_id 30/31) provides
//! per-pixel interpolation with no banding. For multi-stop gradients, CPU
//! tessellation with configurable band count (16-256) is used.

use cvkg_core::{Color, Never, Rect, Renderer, View};

/// A color stop in a gradient.
#[derive(Clone, Debug)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32, // 0.0 - 1.0
}

impl GradientStop {
    pub fn new(color: Color, position: f32) -> Self {
        Self {
            color,
            position: position.clamp(0.0, 1.0),
        }
    }
}

fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// Linear gradient that interpolates colors along an angle.
#[derive(Clone)]
pub struct LinearGradient {
    stops: Vec<GradientStop>,
    angle_degrees: f32,
    quality: u32,
}

impl LinearGradient {
    /// Create a new linear gradient with the given color stops.
    pub fn new(stops: Vec<GradientStop>) -> Self {
        Self {
            stops,
            angle_degrees: 0.0,
            quality: 256, // default to high quality (near-smooth)
        }
    }

    /// Set the gradient angle in degrees (0 = left to right, 90 = top to bottom).
    pub fn angle(mut self, degrees: f32) -> Self {
        self.angle_degrees = degrees;
        self
    }

    /// Set the rendering quality.
    /// Values 16-256 control the number of CPU-tessellated bands.
    /// Value 0 = "smooth" quality (per-pixel GPU interpolation via material_id 30/31).
    /// Default is 256 (visually smooth for most gradients).
    pub fn quality(mut self, bands: u32) -> Self {
        self.quality = bands;
        self
    }

    /// Whether the gradient uses smooth interpolation.
    #[allow(dead_code)]
    pub fn is_smooth(&self) -> bool {
        self.quality == 0 || self.quality >= 256
    }

    /// Number of bands to tessellate (clamped to reasonable range).
    fn num_bands(&self) -> u32 {
        self.quality.clamp(16, 256)
    }

    fn interpolate_color(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        if self.stops.is_empty() {
            return Color::BLACK;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }
        let mut lower = &self.stops[0];
        let mut upper = &self.stops[self.stops.len() - 1];
        for window in self.stops.windows(2) {
            if t >= window[0].position && t <= window[1].position {
                lower = &window[0];
                upper = &window[1];
                break;
            }
        }
        let range = upper.position - lower.position;
        if range <= 0.0 {
            return lower.color;
        }
        let local_t = (t - lower.position) / range;
        lerp_color(lower.color, upper.color, local_t)
    }
}

impl View for LinearGradient {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!("LinearGradient renders via render()")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.stops.is_empty() {
            return;
        }
        if self.stops.len() == 1 {
            renderer.fill_rect(rect, color_to_array(self.stops[0].color));
            return;
        }

        // For 2-stop gradients, use the GPU material path (material_id 30/31)
        // which provides per-pixel interpolation with no banding.
        if self.stops.len() == 2 {
            let start = color_to_array(self.stops[0].color);
            let end = color_to_array(self.stops[1].color);
            let angle_rad = self.angle_degrees.to_radians();
            renderer.draw_linear_gradient(rect, start, end, angle_rad);
            return;
        }

        // Multi-stop: high-quality CPU tessellation with configurable bands.
        let num_bands = self.num_bands();
        let is_vertical = (self.angle_degrees.abs() % 360.0 - 90.0).abs() < 45.0
            || (self.angle_degrees.abs() % 360.0 - 270.0).abs() < 45.0;
        let (primary_size, secondary_size) = if is_vertical {
            (rect.height, rect.width)
        } else {
            (rect.width, rect.height)
        };

        for i in 0..num_bands {
            let t0 = i as f32 / num_bands as f32;
            let t1 = (i + 1) as f32 / num_bands as f32;
            let color = self.interpolate_color((t0 + t1) / 2.0);

            let band_rect = if is_vertical {
                Rect {
                    x: rect.x,
                    y: rect.y + t0 * primary_size,
                    width: secondary_size,
                    height: (t1 - t0) * primary_size,
                }
            } else {
                Rect {
                    x: rect.x + t0 * primary_size,
                    y: rect.y,
                    width: (t1 - t0) * primary_size,
                    height: secondary_size,
                }
            };
            renderer.fill_rect(band_rect, color_to_array(color));
        }
    }
}

/// Radial gradient that interpolates colors from center outward.
#[derive(Clone)]
pub struct RadialGradient {
    stops: Vec<GradientStop>,
    center_x: f32,
    center_y: f32,
}

impl RadialGradient {
    /// Create a new radial gradient with the given color stops.
    pub fn new(stops: Vec<GradientStop>) -> Self {
        Self {
            stops,
            center_x: 0.5,
            center_y: 0.5,
        }
    }

    /// Set the center point (0.0-1.0, default 0.5, 0.5).
    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center_x = x.clamp(0.0, 1.0);
        self.center_y = y.clamp(0.0, 1.0);
        self
    }

    fn interpolate_color(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        if self.stops.is_empty() {
            return Color::BLACK;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }
        let mut lower = &self.stops[0];
        let mut upper = &self.stops[self.stops.len() - 1];
        for window in self.stops.windows(2) {
            if t >= window[0].position && t <= window[1].position {
                lower = &window[0];
                upper = &window[1];
                break;
            }
        }
        let range = upper.position - lower.position;
        if range <= 0.0 {
            return lower.color;
        }
        let local_t = (t - lower.position) / range;
        lerp_color(lower.color, upper.color, local_t)
    }
}

impl View for RadialGradient {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!("RadialGradient renders via render()")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.stops.is_empty() {
            return;
        }
        if self.stops.len() == 1 {
            renderer.fill_rect(rect, color_to_array(self.stops[0].color));
            return;
        }

        // For 2-stop radial, use GPU path
        if self.stops.len() == 2 {
            let inner = color_to_array(self.stops[0].color);
            let outer = color_to_array(self.stops[1].color);
            renderer.draw_radial_gradient(rect, inner, outer);
            return;
        }

        // Multi-stop radial: CPU tessellation with rings
        let num_rings = 256u32;
        let max_radius = rect.width.max(rect.height) / 2.0;
        let cx = rect.x + self.center_x * rect.width;
        let cy = rect.y + self.center_y * rect.height;

        for i in 0..num_rings {
            let t0 = i as f32 / num_rings as f32;
            let t1 = (i + 1) as f32 / num_rings as f32;
            let color = self.interpolate_color((t0 + t1) / 2.0);
            let r = t1 * max_radius;
            let ring_rect = Rect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            };
            renderer.fill_rounded_rect(ring_rect, r, color_to_array(color));
        }
    }
}

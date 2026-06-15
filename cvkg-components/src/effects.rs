use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Seiðr - Holographic projection effect with scanline animation
#[derive(Clone)]
pub struct Seiðr {
    pub base_color: [f32; 4],
    pub scanline_speed: f32,
    pub flicker_intensity: f32,
}

impl Default for Seiðr {
    fn default() -> Self {
        Self {
            base_color: [0.0, 0.8, 1.0, 0.3],
            scanline_speed: 2.0,
            flicker_intensity: 0.1,
        }
    }
}

impl View for Seiðr {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.fill_rounded_rect(rect, 4.0, theme::with_alpha(theme::surface_elevated(), 0.5));
            return;
        }

        let t = renderer.elapsed_time();
        let flicker = 1.0 + (t * 13.0).sin() * self.flicker_intensity;
        let color = [
            self.base_color[0] * flicker,
            self.base_color[1] * flicker,
            self.base_color[2] * flicker,
            self.base_color[3],
        ];

        renderer.fill_rounded_rect(rect, 8.0, color);

        let scan_y = (t * self.scanline_speed).fract() * rect.height;
        for i in 0..5 {
            let y = rect.y + (scan_y + i as f32 * 20.0) % rect.height;
            renderer.draw_line(rect.x, y, rect.x + rect.width, y, theme::with_alpha(theme::accent(), 0.4), 1.0);
        }
    }
}

/// LokiGlitch - Digital distortion text effect
#[derive(Clone)]
pub struct LokiGlitch {
    pub content: String,
    pub font_size: f32,
    pub base_color: [f32; 4],
    pub glitch_intensity: f32,
}

impl LokiGlitch {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 16.0,
            base_color: theme::text(),
            glitch_intensity: 5.0,
        }
    }
}

impl View for LokiGlitch {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.draw_text(
                &self.content,
                rect.x,
                rect.y,
                self.font_size,
                self.base_color,
            );
            return;
        }

        let t = renderer.elapsed_time();
        renderer.draw_text(
            &self.content,
            rect.x,
            rect.y,
            self.font_size,
            self.base_color,
        );

        if (t * 10.0).sin().abs() > 0.8 {
            renderer.draw_text(
                &self.content,
                rect.x + (t * 15.0).sin() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [1.0, 0.0, 0.3, 0.8],
            );
        }
        if (t * 7.0).cos().abs() > 0.85 {
            renderer.draw_text(
                &self.content,
                rect.x - (t * 12.0).cos() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [0.3, 0.7, 1.0, 0.8],
            );
        }
    }
}

/// MidgardLines - Standalone scanline overlay effect
#[derive(Clone)]
pub struct MidgardLines {
    pub speed: f32,
    pub density: f32,
    pub color: [f32; 4],
}

impl Default for MidgardLines {
    fn default() -> Self {
        Self {
            speed: 1.0,
            density: 20.0,
            color: [0.0, 1.0, 1.0, 0.2],
        }
    }
}

impl View for MidgardLines {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let scan_y = (t * self.speed).fract() * rect.height;
        let mut y = rect.y + scan_y % self.density;
        while y < rect.y + rect.height {
            renderer.draw_line(rect.x, y, rect.x + rect.width, y, self.color, 1.0);
            y += self.density;
        }
    }
}

/// NiflheimFrost - Thick refractive ice/glass effect with Liquid Glass capabilities
///
/// Features:
/// - Frosted glass via Bifrost blur
/// - Crystal overlay animation (frost particles)
/// - Liquid Glass: morphing corners, dynamic edge highlights
/// - Clean mode: disable frost particles for pure glass
#[derive(Clone)]
pub struct NiflheimFrost<V: View> {
    pub content: V,
    pub frost_intensity: f32,
    pub blur_radius: f32,
    pub morph_progress: f32,
    pub corner_radius_rest: f32,
    pub corner_radius_hover: f32,
    pub edge_color: [f32; 4],
    pub clean_glass: bool,
}

impl<V: View> NiflheimFrost<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            frost_intensity: 0.8,
            blur_radius: 30.0,
            morph_progress: 0.0,
            corner_radius_rest: 8.0,
            corner_radius_hover: 16.0,
            edge_color: theme::accent(),
            clean_glass: false,
        }
    }

    pub fn clean(mut self) -> Self {
        self.clean_glass = true;
        self
    }

    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }

    pub fn morph_progress(mut self, progress: f32) -> Self {
        self.morph_progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn corner_radii(mut self, rest: f32, hover: f32) -> Self {
        self.corner_radius_rest = rest;
        self.corner_radius_hover = hover;
        self
    }

    pub fn edge_color(mut self, color: [f32; 4]) -> Self {
        self.edge_color = color;
        self
    }

    fn current_corner_radius(&self) -> f32 {
        let t = self.morph_progress;
        self.corner_radius_rest
            + (self.corner_radius_hover - self.corner_radius_rest) * t * t * (3.0 - 2.0 * t)
    }
}

impl<V: View> View for NiflheimFrost<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.fill_rounded_rect(rect, 4.0, theme::with_alpha(theme::surface_elevated(), 0.9));
            renderer.stroke_rounded_rect(rect, 4.0, theme::with_alpha(theme::border(), 0.8), 1.0);
            self.content.render(renderer, rect);
            return;
        }

        renderer.bifrost(rect, self.blur_radius, 1.2, 0.95);

        if !self.clean_glass {
            let t = renderer.elapsed_time();
            for i in 0..15 {
                let x_off = ((t + i as f32) * 0.5).sin() * rect.width * 0.4;
                let y_off = ((t + i as f32 * 1.5) * 0.4).cos() * rect.height * 0.4;
                renderer.draw_line(
                    rect.x + rect.width / 2.0 + x_off,
                    rect.y + rect.height / 2.0 + y_off,
                    rect.x + rect.width / 2.0 + x_off + 10.0,
                    rect.y + rect.height / 2.0 + y_off + 10.0,
                    [1.0, 1.0, 1.0, 0.1 * self.frost_intensity],
                    1.0,
                );
            }
        }

        let corner_radius = self.current_corner_radius();
        let edge_intensity = 0.5 + 0.5 * self.morph_progress;
        let edge_width = 1.0 + 2.0 * self.morph_progress;

        renderer.stroke_rounded_rect(
            rect,
            corner_radius,
            [
                self.edge_color[0],
                self.edge_color[1],
                self.edge_color[2],
                self.edge_color[3] * edge_intensity,
            ],
            edge_width,
        );

        if self.morph_progress > 0.1 {
            let inner_color = [
                self.edge_color[0] * 0.5,
                self.edge_color[1] * 0.5,
                self.edge_color[2] * 0.5,
                self.edge_color[3] * 0.3 * self.morph_progress,
            ];
            renderer.stroke_rounded_rect(
                Rect {
                    x: rect.x + 1.0,
                    y: rect.y + 1.0,
                    width: rect.width - 2.0,
                    height: rect.height - 2.0,
                },
                corner_radius - 2.0,
                inner_color,
                1.0,
            );
        }

        self.content.render(renderer, rect);
    }
}

/// FutharkFlow - Animated runic power-lines connecting components
#[derive(Clone)]
pub struct FutharkFlow {
    pub speed: f32,
    pub color: [f32; 4],
}

impl Default for FutharkFlow {
    fn default() -> Self {
        Self {
            speed: 3.0,
            color: theme::focus_ring(),
        }
    }
}

impl View for FutharkFlow {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            renderer.draw_line(
                rect.x,
                rect.y + rect.height / 2.0,
                rect.x + rect.width,
                rect.y + rect.height / 2.0,
                [0.3, 0.35, 0.4, 0.3],
                1.0,
            );
            return;
        }

        let t = renderer.elapsed_time();
        let runes = ['ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ'];
        let flow_pos = (t * self.speed).fract();
        let rune_idx = ((t * self.speed).floor() as usize) % runes.len();

        renderer.draw_line(
            rect.x,
            rect.y + rect.height / 2.0,
            rect.x + rect.width,
            rect.y + rect.height / 2.0,
            [0.0, 0.5, 0.8, 0.2],
            1.0,
        );

        let rx = rect.x + flow_pos * rect.width;
        let ry = rect.y + rect.height / 2.0;

        renderer.gungnir(
            Rect {
                x: rx - 10.0,
                y: ry - 10.0,
                width: 20.0,
                height: 20.0,
            },
            self.color,
            5.0,
            0.8,
        );
        renderer.draw_text(
            &runes[rune_idx].to_string(),
            rx - 5.0,
            ry + 5.0,
            14.0,
            self.color,
        );
    }
}

/// HeimdallSweep - A tactical radar sweep effect that reveals underlying content.
/// Named after Heimdall, the all-seeing guardian of Bifrost.
#[derive(Clone)]
pub struct HeimdallSweep<V: View> {
    pub content: V,
    pub sweep_speed: f32, // Rotations per second
    pub glow_color: [f32; 4],
}

impl<V: View> HeimdallSweep<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            sweep_speed: 0.25,
            glow_color: [0.0, 1.0, 0.8, 0.6], // Bifrost Cyan
        }
    }
}

impl<V: View> View for HeimdallSweep<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if cvkg_core::load_system_state().realm == cvkg_core::Realm::Midgard {
            self.content.render(renderer, rect);
            return;
        }

        let t = renderer.elapsed_time();
        let angle = (t * self.sweep_speed * 2.0 * std::f32::consts::PI).fract();

        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = rect.width.max(rect.height) * 0.7;

        // 1. Render Content
        self.content.render(renderer, rect);

        // 2. Render Sweep Line
        let lx = center_x + radius * angle.cos();
        let ly = center_y + radius * angle.sin();

        renderer.draw_line(center_x, center_y, lx, ly, self.glow_color, 2.0);

        // 3. Render Sweep Glow/Trail
        for i in 1..10 {
            let trail_angle = angle - (i as f32 * 0.05);
            let alpha = self.glow_color[3] * (1.0 - (i as f32 / 10.0));
            let tx = center_x + radius * trail_angle.cos();
            let ty = center_y + radius * trail_angle.sin();
            renderer.draw_line(
                center_x,
                center_y,
                tx,
                ty,
                [
                    self.glow_color[0],
                    self.glow_color[1],
                    self.glow_color[2],
                    alpha,
                ],
                1.5,
            );
        }
    }
}

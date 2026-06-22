//! CooldownOverlay — Circular countdown timer component.
//!
//! Displays a circular progress indicator that counts down from a total time.
//! Commonly used for ability cooldowns in games or timed UI states.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// A circular countdown timer overlay.
///
/// Renders a ring that depletes over time, with optional text label.
///
/// # Examples
/// ```
/// use cvkg_components::CooldownOverlay;
/// let cooldown = CooldownOverlay::new(5.0, 3.0)
///     .label("Ready in 3s");
/// ```
#[derive(Clone)]
pub struct CooldownOverlay {
    /// Total duration of the cooldown in seconds.
    pub total: f32,
    /// Remaining time in seconds.
    pub remaining: f32,
    /// Optional label text displayed in the center.
    pub label: String,
    /// Size of the overlay in logical pixels.
    pub size: f32,
    /// Color of the cooldown ring.
    pub ring_color: [f32; 4],
    /// Background color of the track.
    pub track_color: [f32; 4],
    /// Width of the ring stroke.
    pub stroke_width: f32,
}

impl CooldownOverlay {
    /// Create a new CooldownOverlay with the given total and remaining time.
    pub fn new(total: f32, remaining: f32) -> Self {
        Self {
            total,
            remaining,
            label: String::new(),
            size: 48.0,
            ring_color: theme::accent(),
            track_color: theme::surface(),
            stroke_width: 3.0,
        }
    }

    /// Set the label text.
    pub fn label(mut self, text: &str) -> Self {
        self.label = text.to_string();
        self
    }

    /// Set the size of the overlay.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the ring color.
    pub fn ring_color(mut self, color: [f32; 4]) -> Self {
        self.ring_color = color;
        self
    }

    /// Set the track color.
    pub fn track_color(mut self, color: [f32; 4]) -> Self {
        self.track_color = color
    }

    /// Set the stroke width.
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width
    }
}

impl View for CooldownOverlay {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let outer_r = self.size / 2.0;
        let inner_r = outer_r - self.stroke_width;
        let progress = if self.total > 0.0 {
            (self.remaining / self.total).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Draw track (background ring) using concentric ellipses
        // Outer circle
        let outer_rect = Rect {
            x: cx - outer_r,
            y: cy - outer_r,
            width: outer_r * 2.0,
            height: outer_r * 2.0,
        };
        renderer.fill_ellipse(outer_rect, self.track_color);

        // Inner circle (to create ring effect) — only if stroke is wide enough
        if inner_r > 0.0 && self.stroke_width > 1.0 {
            let inner_rect = Rect {
                x: cx - inner_r,
                y: cy - inner_r,
                width: inner_r * 2.0,
                height: inner_r * 2.0,
            };
            renderer.fill_ellipse(inner_rect, theme::surface());
        }

        // Draw progress segments (approximated with small rectangles)
        if progress > 0.0 {
            let segments = (progress * 60.0).max(1.0) as i32;
            let angle_per_segment = progress * 2.0 * std::f32::consts::PI / segments as f32;
            let seg_width = self.stroke_width * 0.8;

            for i in 0..segments {
                let angle = -std::f32::consts::PI / 2.0 + i as f32 * angle_per_segment;
                let seg_cx = cx + angle.cos() * (outer_r - self.stroke_width / 2.0);
                let seg_cy = cy + angle.sin() * (outer_r - self.stroke_width / 2.0);
                let seg_size = seg_width.min(angle_per_segment * (outer_r - self.stroke_width / 2.0) * 0.5);

                renderer.fill_rect(
                    Rect {
                        x: seg_cx - seg_size / 2.0,
                        y: seg_cy - seg_size / 2.0,
                        width: seg_size,
                        height: seg_size,
                    },
                    self.ring_color,
                );
            }
        }

        // Draw label text
        if !self.label.is_empty() {
            let text_size = (self.size * 0.3).max(10.0).min(16.0);
            let (tw, th) = renderer.measure_text(&self.label, text_size);
            renderer.draw_text(
                &self.label,
                cx - tw / 2.0,
                cy - th / 2.0,
                text_size,
                theme::text(),
            );
        }
    }
}

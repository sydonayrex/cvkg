//! Confetti — Particle explosion effect component.
//!
//! Displays an animated confetti explosion with colored particles.
//! Used for celebrations, achievements, or visual feedback.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// A confetti explosion effect component.
///
/// Renders animated colored particles that burst outward from a point.
///
/// # Examples
/// ```
/// use cvkg_components::Confetti;
/// let confetti = Confetti::new([200.0, 100.0])
///     .particle_count(30)
///     .colors(vec![[1.0, 0.0, 0.5, 1.0], [0.0, 0.5, 1.0, 1.0]]);
/// ```
#[derive(Clone)]
pub struct Confetti {
    /// Origin point [x, y] of the explosion.
    pub origin: [f32; 2],
    /// Number of confetti particles.
    pub particle_count: u32,
    /// Colors for particles (randomly selected).
    pub colors: Vec<[f32; 4]>,
    /// Animation progress (0.0 = start, 1.0 = end).
    pub progress: f32,
    /// Spread radius in pixels.
    pub spread: f32,
}

impl Confetti {
    /// Create a new Confetti at the given origin point.
    pub fn origin(pos: [f32; 2]) -> Self {
        Self {
            origin: pos,
            particle_count: 20,
            colors: vec![
                theme::accent(),
                theme::viking_gold(),
                theme::magenta_liquid(),
                [0.0, 1.0, 0.5, 1.0],
                [1.0, 0.8, 0.0, 1.0],
            ],
            progress: 0.0,
            spread: 100.0,
        }
    }

    /// Set the particle count.
    pub fn particle_count(mut self, count: u32) -> Self {
        self.particle_count = count;
        self
    }

    /// Set the colors.
    pub fn colors(mut self, colors: Vec<[f32; 4]>) -> Self {
        self.colors = colors;
        self
    }

    /// Set the spread radius.
    pub fn spread(mut self, radius: f32) -> Self {
        self.spread = radius;
        self
    }
}

impl View for Confetti {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        if self.progress >= 1.0 || self.colors.is_empty() {
            return;
        }

        let dt = self.progress;
        let g = 200.0; // gravity

        for i in 0..self.particle_count {
            // Deterministic pseudo-random based on particle index
            let seed_f = i as f32 * 1.618_034;
            let angle = (seed_f * 137.5).to_radians();
            let speed = 50.0 + (seed_f * 37.0) % 80.0;
            let color_idx = (i as usize) % self.colors.len();

            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            let px = self.origin[0] + vx * dt;
            let py = self.origin[1] + vy * dt + 0.5 * g * dt * dt;

            let alpha = (1.0 - dt).max(0.0);
            let color = self.colors[color_idx];
            let particle_color = [color[0], color[1], color[2], color[3] * alpha];

            let size = 4.0 + (i % 3) as f32 * 2.0;
            renderer.fill_rect(
                Rect {
                    x: px - size / 2.0,
                    y: py - size / 2.0,
                    width: size,
                    height: size,
                },
                particle_color,
            );
        }
    }
}

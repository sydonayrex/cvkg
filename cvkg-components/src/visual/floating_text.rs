//! FloatingText — Animated flyout text component.
//!
//! Displays text that animates upward and fades out, commonly used for
//! damage numbers, score popups, or transient notifications.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// An animated floating text component.
///
/// Renders text that moves upward with a fade-out animation.
///
/// # Examples
/// ```
/// use cvkg_components::FloatingText;
/// let floating = FloatingText::new("+100")
///     .origin([100.0, 200.0])
///     .velocity([0.0, -30.0])
///     .color([1.0, 0.8, 0.0, 1.0]);
/// ```
#[derive(Clone)]
pub struct FloatingText {
    /// The text to display.
    pub text: String,
    /// Origin position [x, y].
    pub origin: [f32; 2],
    /// Velocity [vx, vy] in pixels per second.
    pub velocity: [f32; 2],
    /// Text color.
    pub color: [f32; 4],
    /// Font size.
    pub font_size: f32,
    /// Animation progress (0.0 = start, 1.0 = end).
    pub progress: f32,
    /// Lifetime in seconds.
    pub lifetime: f32,
}

impl FloatingText {
    /// Create a new FloatingText with the given text.
    pub fn text(content: &str) -> Self {
        Self {
            text: content.to_string(),
            origin: [0.0, 0.0],
            velocity: [0.0, -40.0],
            color: theme::text(),
            font_size: 14.0,
            progress: 0.0,
            lifetime: 1.5,
        }
    }

    /// Set the origin position.
    pub fn origin(mut self, pos: [f32; 2]) -> Self {
        self.origin = pos;
        self
    }

    /// Set the velocity.
    pub fn velocity(mut self, vel: [f32; 2]) -> Self {
        self.velocity = vel;
        self
    }

    /// Set the text color.
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the lifetime in seconds.
    pub fn lifetime(mut self, seconds: f32) -> Self {
        self.lifetime = seconds;
        self
    }
}

impl View for FloatingText {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        if self.progress >= 1.0 {
            return;
        }

        let dt = self.progress * self.lifetime;
        let x = self.origin[0] + self.velocity[0] * dt;
        let y = self.origin[1] + self.velocity[1] * dt;
        let alpha = (1.0 - self.progress).max(0.0);
        let color = [self.color[0], self.color[1], self.color[2], self.color[3] * alpha];

        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        renderer.draw_text_raw(&self.text, x - tw / 2.0, y - th / 2.0, self.font_size, color);
    }
}

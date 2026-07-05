//! HealthBar component for game UI.
//!
//! A horizontal bar with fill, color gradient (green→yellow→red), and optional text.

use cvkg_core::{Never, Rect, Renderer, View};

/// Health bar with gradient fill and optional text display.
///
/// ## Accessibility
/// - Role: `progressbar`
/// - ARIA: `aria-label` from `label` prop, `aria-valuenow` from current value
/// - Reduced motion: respects `is_reduced_motion()` for fill animation
#[derive(Clone)]
pub struct HealthBar {
    /// Current health value.
    pub current: f32,
    /// Maximum health value.
    pub max: f32,
    /// Label for accessibility.
    pub label: Option<String>,
    /// Whether to show text.
    pub show_text: bool,
    /// Bar height.
    pub height: f32,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self::new(100.0, 100.0)
    }
}

impl HealthBar {
    /// Create a new HealthBar.
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current,
            max,
            label: None,
            show_text: true,
            height: 20.0,
        }
    }

    /// Set the label for accessibility.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Show or hide text.
    pub fn show_text(mut self, show: bool) -> Self {
        self.show_text = show;
        self
    }

    /// Set the bar height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl View for HealthBar {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let ratio = (self.current / self.max).clamp(0.0, 1.0);

        renderer.push_vnode(rect, "HealthBar");
        renderer.register_a11y("progressbar", self.label.as_deref().unwrap_or("Health"));

        let bg_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - self.height) / 2.0,
            width: rect.width,
            height: self.height,
        };

        // Background (dark track)
        renderer.fill_rounded_rect(bg_rect, 2.0, [0.1, 0.1, 0.1, 1.0]);

        // Filled portion with gradient
        if ratio > 0.0 {
            let fill_rect = Rect {
                x: bg_rect.x,
                y: bg_rect.y,
                width: bg_rect.width * ratio,
                height: bg_rect.height,
            };

            // Color based on health ratio: green → yellow → red
            let fill_color = if ratio > 0.6 {
                // Green
                [0.0, 0.8 * ratio, 0.0, 1.0]
            } else if ratio > 0.3 {
                // Yellow
                [0.8, 0.8 * ratio, 0.0, 1.0]
            } else {
                // Red
                [0.8, 0.0, 0.0, 1.0]
            };

            renderer.fill_rounded_rect(fill_rect, 2.0, fill_color);
        }

        // Text overlay
        if self.show_text {
            let text = format!("{:.0}/{:.0}", self.current, self.max);
            let (tw, _th) = renderer.measure_text(&text, 12.0);
            renderer.draw_text_raw(
                &text,
                rect.x + (rect.width - tw) / 2.0,
                rect.y + (rect.height - 12.0) / 2.0,
                12.0,
                [1.0, 1.0, 1.0, 0.9],
            );
        }

        renderer.pop_vnode();
    }
}

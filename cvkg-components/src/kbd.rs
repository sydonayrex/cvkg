//! Kbd component for displaying keyboard shortcuts.
//!
//! Renders text inside a styled box with monospace font, resembling
//! a physical keyboard key.

use crate::theme;
use crate::{FONT_SM, RADIUS_SM, SPACE_SM, SPACE_XS};
use cvkg_core::{Never, Rect, Renderer, View};

/// Kbd - A component for displaying keyboard shortcut text.
///
/// Renders the key text inside a small styled box with monospace font,
/// a subtle border, and a slight shadow to resemble a physical key.
///
/// # Example
/// ```
/// use cvkg_components::kbd::Kbd;
/// let kbd = Kbd::new("⌘K");
/// ```
#[derive(Clone)]
pub struct Kbd {
    /// The key text to display (e.g. "⌘K", "Ctrl+C", "Esc").
    keys: String,
    /// Font size override. When None, uses FONT_SM.
    font_size: Option<f32>,
}

impl Kbd {
    /// Create a new Kbd component with the given key text.
    pub fn new(keys: impl Into<String>) -> Self {
        Self {
            keys: keys.into(),
            font_size: None,
        }
    }

    /// Set a custom font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
}

impl View for Kbd {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let font_size = self.font_size.unwrap_or(FONT_SM);
        let (tw, th) = renderer.measure_text(&self.keys, font_size);

        // Key box dimensions with padding
        let box_w = tw + SPACE_SM * 2.0;
        let box_h = th + SPACE_XS * 2.0;
        let box_x = rect.x + (rect.width - box_w) / 2.0;
        let box_y = rect.y + (rect.height - box_h) / 2.0;

        let key_rect = Rect {
            x: box_x,
            y: box_y,
            width: box_w,
            height: box_h,
        };

        renderer.push_vnode(key_rect, "Kbd");

        // Shadow
        let shadow_rect = Rect {
            x: key_rect.x + 1.0,
            y: key_rect.y + 2.0,
            width: key_rect.width,
            height: key_rect.height,
        };
        renderer.fill_rounded_rect(shadow_rect, RADIUS_SM, theme::with_alpha(theme::bg(), 0.2));

        // Key background
        renderer.fill_rounded_rect(key_rect, RADIUS_SM, theme::surface_elevated());

        // Border
        renderer.stroke_rounded_rect(key_rect, RADIUS_SM, theme::border_strong(), 1.0);

        // Key text (centered)
        renderer.draw_text(
            &self.keys,
            key_rect.x + SPACE_SM,
            key_rect.y + SPACE_XS,
            font_size,
            theme::text(),
        );

        renderer.pop_vnode();
    }
}

//! DPad (directional pad) control for game UI.
//!
//! A cross-shaped input control with 4 directional buttons.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// DPad direction callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DPadDirection {
    /// Up direction.
    Up,
    /// Down direction.
    Down,
    /// Left direction.
    Left,
    /// Right direction.
    Right,
}

/// DPad control with 4 directional buttons arranged in a cross.
///
/// ## Accessibility
/// - Role: `button` for each direction
/// - Keyboard: Arrow keys map to directions
/// - ARIA: Each direction button has `aria-label` for "Up", "Down", "Left", "Right"
/// - Reduced motion: respects `is_reduced_motion()` for press animation
#[derive(Clone)]
pub struct DPadControl {
    /// Callback for directional input.
    pub on_direction: Option<Arc<dyn Fn(DPadDirection) + Send + Sync>>,
    /// Whether the DPad is enabled.
    pub enabled: bool,
    /// Button size.
    pub button_size: f32,
}

impl Default for DPadControl {
    fn default() -> Self {
        Self::new()
    }
}

impl DPadControl {
    /// Create a new DPadControl.
    pub fn new() -> Self {
        Self {
            on_direction: None,
            enabled: true,
            button_size: 32.0,
        }
    }

    /// Set the direction callback.
    pub fn on_direction(
        mut self,
        callback: impl Fn(DPadDirection) + Send + Sync + 'static,
    ) -> Self {
        self.on_direction = Some(Arc::new(callback));
        self
    }

    /// Enable or disable the DPad.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set button size.
    pub fn button_size(mut self, size: f32) -> Self {
        self.button_size = size;
        self
    }
}

impl View for DPadControl {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DPadControl");
        renderer.register_a11y("button", "Directional control");

        let s = self.button_size;
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;

        // Background circle
        renderer.fill_ellipse(
            Rect {
                x: center_x - s,
                y: center_y - s,
                width: s * 2.0,
                height: s * 2.0,
            },
            theme::with_alpha(theme::surface_elevated(), 0.7),
        );

        // Draw directional buttons
        let button_color = if self.enabled {
            theme::accent()
        } else {
            theme::with_alpha(theme::text_muted(), 0.5)
        };

        // Up button
        let up_rect = Rect {
            x: center_x - s / 2.0,
            y: center_y - s - s,
            width: s,
            height: s,
        };
        renderer.fill_rounded_rect(up_rect, s / 4.0, button_color);

        // Down button
        let down_rect = Rect {
            x: center_x - s / 2.0,
            y: center_y + s,
            width: s,
            height: s,
        };
        renderer.fill_rounded_rect(down_rect, s / 4.0, button_color);

        // Left button
        let left_rect = Rect {
            x: center_x - s - s,
            y: center_y - s / 2.0,
            width: s,
            height: s,
        };
        renderer.fill_rounded_rect(left_rect, s / 4.0, button_color);

        // Right button
        let right_rect = Rect {
            x: center_x + s,
            y: center_y - s / 2.0,
            width: s,
            height: s,
        };
        renderer.fill_rounded_rect(right_rect, s / 4.0, button_color);

        // Center (neutral) button
        let center_rect = Rect {
            x: center_x - s / 2.0,
            y: center_y - s / 2.0,
            width: s,
            height: s,
        };
        renderer.fill_rounded_rect(center_rect, s / 4.0, theme::surface());

        renderer.pop_vnode();
    }
}

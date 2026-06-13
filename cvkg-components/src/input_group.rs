//! InputGroup component for wrapping an Input with leading/trailing elements.
//!
//! Renders an input field with optional leading icons/labels and trailing
//! icons/buttons, all styled as a single cohesive input group.

use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, SPACE_SM};
use cvkg_core::{Never, Rect, Renderer, View};

/// InputGroup - A wrapper that renders an input with optional leading/trailing elements.
///
/// The leading element (e.g. an icon or label) is rendered to the left of the input,
/// and the trailing element (e.g. a button or icon) is rendered to the right.
/// All elements share a single rounded border.
///
/// # Example
/// ```
/// use cvkg_components::input_group::InputGroup;
/// let group = InputGroup::new("Search...")
///     .leading("🔍")
///     .trailing("Go");
/// ```
#[derive(Clone)]
pub struct InputGroup {
    /// Placeholder text for the input field.
    placeholder: String,
    /// Current input value.
    value: String,
    /// Optional leading element text (icon or label).
    leading: Option<String>,
    /// Optional trailing element text (icon or button label).
    trailing: Option<String>,
    /// Whether the input is disabled.
    disabled: bool,
}

impl InputGroup {
    /// Create a new InputGroup with the given placeholder text.
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: String::new(),
            leading: None,
            trailing: None,
            disabled: false,
        }
    }

    /// Set the current input value.
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self
    }

    /// Set the leading element text (e.g. an icon or label).
    pub fn leading(mut self, leading: impl Into<String>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    /// Set the trailing element text (e.g. an icon or button label).
    pub fn trailing(mut self, trailing: impl Into<String>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl View for InputGroup {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "InputGroup");

        let bg_color = if self.disabled {
            theme::disabled()
        } else {
            theme::surface()
        };
        let border_color = theme::border();

        // Outer rounded rect (the group border)
        renderer.fill_rounded_rect(rect, RADIUS_MD, bg_color);
        renderer.stroke_rounded_rect(rect, RADIUS_MD, border_color, 1.0);

        let mut input_x = rect.x;
        let _input_w = rect.width;

        // Leading element
        if let Some(ref lead) = self.leading {
            let (lw, _) = renderer.measure_text(lead, FONT_BASE);
            let lead_w = lw + SPACE_SM * 2.0;
            renderer.draw_text(
                lead,
                rect.x + SPACE_SM,
                rect.y + (rect.height - FONT_BASE) / 2.0,
                FONT_BASE,
                theme::text_muted(),
            );
            input_x += lead_w;
        }

        // Trailing element
        if let Some(ref trail) = self.trailing {
            let (tw, _) = renderer.measure_text(trail, FONT_BASE);
            let trail_w = tw + SPACE_SM * 2.0;
            renderer.draw_text(
                trail,
                rect.x + rect.width - trail_w + SPACE_SM,
                rect.y + (rect.height - FONT_BASE) / 2.0,
                FONT_BASE,
                theme::text_muted(),
            );
        }

        // Input text
        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() {
            theme::text_muted()
        } else {
            theme::text()
        };

        renderer.draw_text(
            display_text,
            input_x + SPACE_SM,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            text_color,
        );

        // Focus ring
        if !self.disabled {
            crate::draw_focus_ring(renderer, rect);
        }

        renderer.pop_vnode();
    }
}

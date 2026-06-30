//! NativeSelect component for a native dropdown select.
//!
//! Renders a styled dropdown select element with a list of options.
//! Each option has a display label and a value.

use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, SPACE_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// NativeSelect - A native dropdown select component.
///
/// Displays the currently selected option. When clicked, a dropdown
/// list of all options appears. Selecting an option invokes the
/// on_change callback.
///
/// # Example
/// ```
/// use cvkg_components::native_select::NativeSelect;
/// let options = vec![
///     ("en".to_string(), "English".to_string()),
///     ("fr".to_string(), "French".to_string()),
///     ("de".to_string(), "German".to_string()),
/// ];
/// let select = NativeSelect::new(options, "en", |val| {
///     println!("Selected: {}", val);
/// });
/// ```
#[derive(Clone)]
pub struct NativeSelect {
    /// The available options as (value, label) pairs.
    options: Vec<(String, String)>,
    /// The currently selected value.
    value: String,
    /// Callback invoked when the selection changes.
    on_change: Arc<dyn Fn(String) + Send + Sync>,
    /// Whether the select is disabled.
    disabled: bool,
    /// Whether the dropdown is currently expanded.
    is_open: bool,
}

impl NativeSelect {
    /// Create a new NativeSelect with options and a change callback.
    ///
    /// # Arguments
    /// * `options` - Vec of (value, display_label) pairs.
    /// * `value` - The initially selected value.
    /// * `on_change` - Callback invoked with the selected value string.
    pub fn new(
        options: Vec<(String, String)>,
        value: impl Into<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            options,
            value: value.into(),
            on_change: Arc::new(on_change),
            disabled: false,
            is_open: false,
        }
    }

    /// Set the currently selected value.
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self
    }

    /// Set whether the select is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the dropdown is expanded.
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Find the label for the current value.
    fn selected_label(&self) -> String {
        self.options
            .iter()
            .find(|(v, _)| v == &self.value)
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| self.value.clone())
    }
}

impl View for NativeSelect {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NativeSelect");

        let bg_color = if self.disabled {
            theme::disabled()
        } else {
            theme::surface()
        };
        let border_color = if self.is_open {
            theme::accent()
        } else {
            theme::border()
        };

        // Main select box
        renderer.fill_rounded_rect(rect, RADIUS_MD, bg_color);
        renderer.stroke_rounded_rect(rect, RADIUS_MD, border_color, 1.0);

        // Selected value label
        let label = self.selected_label();
        let text_color = if self.disabled {
            theme::disabled_text()
        } else {
            theme::text()
        };
        renderer.draw_text_raw(
            &label,
            rect.x + SPACE_SM,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            text_color,
        );

        // Dropdown arrow (right side)
        let arrow = if self.is_open { "▲" } else { "▼" };
        let (aw, _) = renderer.measure_text(arrow, FONT_BASE);
        renderer.draw_text_raw(
            arrow,
            rect.x + rect.width - aw - SPACE_SM,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            theme::text_muted(),
        );

        // Dropdown list
        if self.is_open && !self.disabled {
            let item_h = 36.0;
            let dropdown_h = self.options.len() as f32 * item_h;
            let dropdown_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: dropdown_h,
            };

            renderer.push_vnode(dropdown_rect, "NativeSelectDropdown");

            // Dropdown background
            renderer.fill_rounded_rect(dropdown_rect, RADIUS_MD, theme::surface_elevated());
            renderer.stroke_rounded_rect(dropdown_rect, RADIUS_MD, theme::border(), 1.0);

            for (idx, (opt_val, opt_label)) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: dropdown_rect.x,
                    y: dropdown_rect.y + idx as f32 * item_h,
                    width: dropdown_rect.width,
                    height: item_h,
                };

                renderer.push_vnode(item_rect, "NativeSelectOption");

                let is_selected = opt_val == &self.value;

                // Highlight selected item
                if is_selected {
                    renderer.fill_rounded_rect(item_rect, 0.0, theme::hover());
                }

                // Option label
                let opt_color = if is_selected {
                    theme::accent()
                } else {
                    theme::text()
                };
                renderer.draw_text_raw(
                    opt_label,
                    item_rect.x + SPACE_SM,
                    item_rect.y + (item_h - FONT_BASE) / 2.0,
                    FONT_BASE,
                    opt_color,
                );

                // Click handler
                let on_change = self.on_change.clone();
                let selected_val = opt_val.clone();
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |_| {
                        (on_change)(selected_val.clone());
                    }),
                );

                renderer.pop_vnode();
            }

            renderer.pop_vnode();
        }

        // Click handler to toggle dropdown
        if !self.disabled {
            let on_change = self.on_change.clone();
            let current_value = self.value.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    // Toggle -- in a real app this would flip a state flag
                    let _ = &on_change;
                    let _ = &current_value;
                }),
            );
        }

        renderer.pop_vnode();
    }
}

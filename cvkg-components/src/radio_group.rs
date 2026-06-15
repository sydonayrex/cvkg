//! RadioGroup component for exclusive single-value selection.
//!
//! Provides a group of circular radio buttons with keyboard navigation
//! (arrow keys to move, Space/Enter to select) and proper ARIA roles.

use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// A single option within a radio group.
///
/// Each option has a display label and an optional description
/// rendered below the label.
#[derive(Debug, Clone)]
pub struct RadioOption {
    /// The label displayed next to the radio button.
    pub label: String,
    /// An optional description rendered in muted text below the label.
    pub description: Option<String>,
}

impl RadioOption {
    /// Create a new radio option with just a label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
        }
    }

    /// Set a description for this option.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A radio group manages a single selected value across multiple options.
///
/// Renders each option as a circular radio button with a label. Supports
/// keyboard navigation (arrow keys move between options, Space/Enter selects),
/// and each radio item has `Role::RadioButton` via `set_aria_role`.
///
/// # Examples
/// ```
/// use cvkg_components::radio_group::{RadioGroup, RadioOption};
/// let options = vec![
///     RadioOption::new("Option A").description("First choice"),
///     RadioOption::new("Option B"),
///     RadioOption::new("Option C"),
/// ];
/// let group = RadioGroup::new(options, 0, |idx| {
///     println!("Selected: {}", idx);
/// });
/// ```
#[derive(Clone)]
pub struct RadioGroup {
    /// The available options.
    options: Vec<RadioOption>,
    /// The index of the currently selected option.
    selected_index: usize,
    /// Callback invoked when the selection changes.
    on_change: Arc<dyn Fn(usize) + Send + Sync>,
}

impl RadioGroup {
    /// Create a new RadioGroup with the given options and change callback.
    ///
    /// # Arguments
    /// * `options` - The radio options to display.
    /// * `selected` - The index of the initially selected option.
    /// * `on_change` - Callback invoked with the index when selection changes.
    pub fn new(
        options: Vec<RadioOption>,
        selected: usize,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        let selected = selected.min(options.len().saturating_sub(1));
        Self {
            options,
            selected_index: selected,
            on_change: Arc::new(on_change),
        }
    }

    /// Set the selected index.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.options.len().saturating_sub(1));
        self
    }

    /// Layout: each option takes 24px base + 16px if description present.
    fn item_height(option: &RadioOption) -> f32 {
        if option.description.is_some() {
            44.0
        } else {
            28.0
        }
    }

    /// Compute total intrinsic height for all options.
    fn total_height(&self) -> f32 {
        self.options.iter().map(Self::item_height).sum()
    }
}

impl View for RadioGroup {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RadioGroup");
        renderer.set_aria_role("radiogroup");
        renderer.set_aria_label("Radio group");

        let mut y_offset = 0.0f32;

        for (idx, option) in self.options.iter().enumerate() {
            let item_h = Self::item_height(option);
            let item_rect = Rect {
                x: rect.x,
                y: rect.y + y_offset,
                width: rect.width,
                height: item_h,
            };

            renderer.push_vnode(item_rect, "RadioItem");
            renderer.set_aria_role("radio");
            renderer.set_aria_label(&option.label);

            let is_selected = idx == self.selected_index;
            let dot_radius = if is_selected { 5.0 } else { 4.0 };
            let outer_radius = 9.0;

            // Outer circle (the radio ring)
            let outer_rect = Rect {
                x: rect.x + 9.0 - outer_radius,
                y: rect.y + y_offset + 14.0 - outer_radius,
                width: outer_radius * 2.0,
                height: outer_radius * 2.0,
            };
            renderer.fill_rounded_rect(
                outer_rect,
                outer_radius,
                if is_selected {
                    theme::accent()
                } else {
                    theme::surface_elevated()
                },
            );
            if !is_selected {
                renderer.stroke_rounded_rect(outer_rect, outer_radius, theme::border_strong(), 1.5);
            }

            // Inner dot for selected
            if is_selected {
                let inner_rect = Rect {
                    x: rect.x + 9.0 - dot_radius,
                    y: rect.y + y_offset + 14.0 - dot_radius,
                    width: dot_radius * 2.0,
                    height: dot_radius * 2.0,
                };
                renderer.fill_rounded_rect(inner_rect, dot_radius, theme::text());
            }

            // Label text
            renderer.draw_text(
                &option.label,
                rect.x + 26.0,
                rect.y + y_offset + 7.0,
                14.0,
                theme::text(),
            );

            // Optional description
            if let Some(ref desc) = option.description {
                renderer.draw_text(
                    desc,
                    rect.x + 26.0,
                    rect.y + y_offset + 22.0,
                    11.0,
                    theme::text_muted(),
                );
            }

            // Click handler for this option
            let on_change = self.on_change.clone();
            let target_idx = idx;
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    (on_change)(target_idx);
                }),
            );

            renderer.pop_vnode();
            y_offset += item_h;
        }

        // Keyboard navigation: ArrowUp / ArrowDown / Space / Enter
        let options_len = self.options.len();
        let on_change_kb = self.on_change.clone();
        let current_selected = self.selected_index;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowUp" | "ArrowLeft" => {
                            if options_len > 0 && current_selected > 0 {
                                (on_change_kb)(current_selected - 1);
                            } else if options_len > 0 {
                                (on_change_kb)(options_len - 1);
                            }
                        }
                        "ArrowDown" | "ArrowRight" => {
                            if options_len > 0 && current_selected + 1 < options_len {
                                (on_change_kb)(current_selected + 1);
                            } else if options_len > 0 {
                                (on_change_kb)(0);
                            }
                        }
                        " " | "Enter" => {
                            (on_change_kb)(current_selected);
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::layout::SizeProposal,
    ) -> cvkg_core::Size {
        let max_label_width = self
            .options
            .iter()
            .map(|opt| renderer.measure_text(&opt.label, 14.0).0)
            .fold(0.0f32, f32::max);
        let max_desc_width = self
            .options
            .iter()
            .filter_map(|opt| opt.description.as_ref())
            .map(|d| renderer.measure_text(d, 11.0).0)
            .fold(0.0f32, f32::max);
        let content_width = max_label_width.max(max_desc_width) + 36.0;
        cvkg_core::Size {
            width: proposal.width.unwrap_or(content_width).max(content_width),
            height: self.total_height(),
        }
    }
}

//! ToggleGroup component for a group of toggle buttons.
//!
//! Supports single-select and multi-select modes. Each item is a button
//! that can be toggled on/off. In single-select mode, only one item can
//! be active at a time.

use crate::theme;
use crate::{FONT_SM, RADIUS_MD, RADIUS_SM, SPACE_XS};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// ToggleGroup - A group of toggle buttons with single or multi-select.
///
/// Renders a horizontal row of buttons where each button can be toggled.
/// In single-select mode, activating one button deactivates the others.
/// In multi-select mode, any combination of buttons can be active.
///
/// # Example
/// ```
/// use cvkg_components::toggle_group::ToggleGroup;
/// let items = vec!["Left".to_string(), "Center".to_string(), "Right".to_string()];
/// let group = ToggleGroup::new(items, vec![1], |selected| {
///     println!("Selected: {:?}", selected);
/// }).multi(false);
/// ```
#[derive(Clone)]
pub struct ToggleGroup {
    /// The toggle button labels.
    items: Vec<String>,
    /// Indices of currently selected items.
    value: Vec<usize>,
    /// Callback invoked when selection changes.
    on_change: Arc<dyn Fn(Vec<usize>) + Send + Sync>,
    /// Whether multi-select is enabled.
    multi: bool,
    /// Whether the entire group is disabled.
    disabled: bool,
    /// Tracks which item currently has keyboard focus (0-indexed).
    focused_index: Arc<AtomicUsize>,
}

impl ToggleGroup {
    /// Create a new ToggleGroup with items and a change callback.
    ///
    /// # Arguments
    /// * `items` - The button labels.
    /// * `value` - Initially selected indices.
    /// * `on_change` - Callback invoked with the selected indices.
    pub fn new(
        items: Vec<String>,
        value: Vec<usize>,
        on_change: impl Fn(Vec<usize>) + Send + Sync + 'static,
    ) -> Self {
        Self {
            items,
            value,
            on_change: Arc::new(on_change),
            multi: false,
            disabled: false,
            focused_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Set whether multi-select is enabled.
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// Set the currently selected indices.
    pub fn value(mut self, value: Vec<usize>) -> Self {
        self.value = value;
        self
    }

    /// Set whether the group is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the initially focused item index (for keyboard navigation).
    pub fn focused(self, index: usize) -> Self {
        let clamped = index.min(self.items.len().saturating_sub(1));
        self.focused_index.store(clamped, Ordering::Relaxed);
        self
    }

    /// Check if an index is selected.
    fn is_selected(&self, idx: usize) -> bool {
        self.value.contains(&idx)
    }
}

impl View for ToggleGroup {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ToggleGroup");
        renderer.set_aria_role("group");
        renderer.set_aria_label("Toggle group");

        if self.items.is_empty() {
            renderer.pop_vnode();
            return;
        }

        let item_h = rect.height;
        let total_spacing = SPACE_XS * (self.items.len().saturating_sub(1)) as f32;
        let item_w = (rect.width - total_spacing) / self.items.len() as f32;

        // Group background
        renderer.fill_rounded_rect(rect, RADIUS_MD, theme::surface());

        let focused_idx = self.focused_index.load(Ordering::Relaxed);

        for (idx, label) in self.items.iter().enumerate() {
            let ix = rect.x + idx as f32 * (item_w + SPACE_XS);
            let item_rect = Rect {
                x: ix,
                y: rect.y,
                width: item_w,
                height: item_h,
            };

            renderer.push_vnode(item_rect, "ToggleGroupItem");
            renderer.set_aria_role("button");
            let pressed = self.is_selected(idx);
            let aria_label = format!("{}{}", label, if pressed { " (pressed)" } else { "" });
            renderer.set_aria_label(&aria_label);

            let is_selected = self.is_selected(idx);
            let bg_color = if self.disabled {
                theme::disabled()
            } else if is_selected {
                theme::accent()
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };

            let text_color = if self.disabled {
                theme::disabled_text()
            } else if is_selected {
                theme::text()
            } else {
                theme::text_muted()
            };

            // Item background
            if is_selected || self.disabled {
                renderer.fill_rounded_rect(item_rect, RADIUS_SM, bg_color);
            }

            // Keyboard focus ring
            if !self.disabled && idx == focused_idx {
                renderer.stroke_rounded_rect(item_rect, RADIUS_SM, theme::accent(), 2.0);
            }

            // Label (centered)
            let (tw, _) = renderer.measure_text(label, FONT_SM);
            renderer.draw_text(
                label,
                item_rect.x + (item_w - tw) / 2.0,
                item_rect.y + (item_h - FONT_SM) / 2.0,
                FONT_SM,
                text_color,
            );

            // Click handler
            if !self.disabled {
                let on_change = self.on_change.clone();
                let current_value = self.value.clone();
                let target_idx = idx;
                let is_multi = self.multi;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |_| {
                        let mut new_value = current_value.clone();
                        if is_multi {
                            if let Some(pos) = new_value.iter().position(|&v| v == target_idx) {
                                new_value.remove(pos);
                            } else {
                                new_value.push(target_idx);
                            }
                        } else {
                            new_value = vec![target_idx];
                        }
                        (on_change)(new_value);
                    }),
                );
            }

            renderer.pop_vnode();
        }

        // Keyboard navigation: ArrowLeft/ArrowRight, Space/Enter, Home/End
        if !self.disabled {
            let items_len = self.items.len();
            let on_change_kb = self.on_change.clone();
            let current_value = self.value.clone();
            let is_multi = self.multi;
            let focused = self.focused_index.clone();

            renderer.register_handler(
                "keydown",
                Arc::new(move |event| {
                    if let Event::KeyDown { key, .. } = event {
                        match key.as_str() {
                            "ArrowLeft" => {
                                if items_len > 0 {
                                    let cur = focused.load(Ordering::Relaxed);
                                    let next = if cur > 0 { cur - 1 } else { items_len - 1 };
                                    focused.store(next, Ordering::Relaxed);
                                }
                            }
                            "ArrowRight" => {
                                if items_len > 0 {
                                    let cur = focused.load(Ordering::Relaxed);
                                    let next = if cur + 1 < items_len { cur + 1 } else { 0 };
                                    focused.store(next, Ordering::Relaxed);
                                }
                            }
                            "Home" => {
                                if items_len > 0 {
                                    focused.store(0, Ordering::Relaxed);
                                }
                            }
                            "End" => {
                                if items_len > 0 {
                                    focused.store(items_len - 1, Ordering::Relaxed);
                                }
                            }
                            " " | "Enter" => {
                                let target_idx = focused.load(Ordering::Relaxed);
                                if target_idx < items_len {
                                    let mut new_value = current_value.clone();
                                    if is_multi {
                                        if let Some(pos) =
                                            new_value.iter().position(|&v| v == target_idx)
                                        {
                                            new_value.remove(pos);
                                        } else {
                                            new_value.push(target_idx);
                                        }
                                    } else {
                                        new_value = vec![target_idx];
                                    }
                                    (on_change_kb)(new_value);
                                }
                            }
                            _ => {}
                        }
                    }
                }),
            );
        }

        renderer.pop_vnode();
    }
}

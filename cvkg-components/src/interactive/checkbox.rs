use crate::theme;
use cvkg_core::{AriaProperties, AriaRole, KeyModifiers, Never, Rect, Renderer, View};
use std::sync::Arc as StdArc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckboxState {
    Unchecked,
    Checked,
    Indeterminate,
}

#[derive(Clone)]
pub struct Checkbox {
    pub(crate) state: CheckboxState,
    pub(crate) label: Option<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(bool) + Send + Sync>,
}

impl Checkbox {
    /// Create a new Checkbox.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Checkbox;
    /// let checkbox = Checkbox::new(false, |checked| println!("Checked: {}", checked))
    ///     .label("Enable feature");
    /// ```
    pub fn new(is_checked: bool, on_change: impl Fn(bool) + Send + Sync + 'static) -> Self {
        Self {
            state: if is_checked {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            },
            label: None,
            on_change: std::sync::Arc::new(on_change),
        }
    }

    /// Set the checkbox to an indeterminate state.
    pub fn indeterminate(mut self, is_indeterminate: bool) -> Self {
        if is_indeterminate {
            self.state = CheckboxState::Indeterminate;
        }
        self
    }

    /// Set the label for the checkbox.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl View for Checkbox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let focus_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.label.hash(&mut s);
            "checkbox_focus".hash(&mut s);
            s.finish()
        };
        let (is_focused, set_focused) = cvkg_vdom::use_state(focus_hash, false);

        renderer.push_vnode(rect, "Checkbox");
        renderer.set_aria_role("checkbox");
        renderer.set_aria_label(self.label.as_deref().unwrap_or("Checkbox"));

        let box_size = 18.0;
        let box_rect = Rect {
            x: rect.x,
            y: rect.y + (rect.height - box_size) / 2.0,
            width: box_size,
            height: box_size,
        };

        let is_active = matches!(
            self.state,
            CheckboxState::Checked | CheckboxState::Indeterminate
        );

        let bg = if is_active {
            theme::accent()
        } else {
            theme::surface_elevated()
        };

        renderer.fill_rounded_rect(box_rect, 3.0, bg);
        renderer.stroke_rect(box_rect, theme::border(), 1.0);

        match self.state {
            CheckboxState::Checked => {
                // Draw checkmark using lines (SVG-like path approximation)
                let c = theme::text();
                // Checkmark path: starts mid-left, goes down-right, then up-right.
                renderer.draw_line(
                    box_rect.x + 4.0,
                    box_rect.y + 9.0,
                    box_rect.x + 8.0,
                    box_rect.y + 13.0,
                    c,
                    2.0,
                );
                renderer.draw_line(
                    box_rect.x + 8.0,
                    box_rect.y + 13.0,
                    box_rect.x + 14.0,
                    box_rect.y + 5.0,
                    c,
                    2.0,
                );
            }
            CheckboxState::Indeterminate => {
                // Draw a horizontal dash
                renderer.draw_line(
                    box_rect.x + 4.0,
                    box_rect.y + 9.0,
                    box_rect.x + 14.0,
                    box_rect.y + 9.0,
                    theme::text(),
                    2.0,
                );
            }
            CheckboxState::Unchecked => {}
        }

        if let Some(label) = &self.label {
            renderer.draw_text(
                label,
                box_rect.x + box_size + 8.0,
                rect.y + (rect.height - 14.0) / 2.0,
                14.0,
                theme::text(),
            );
        }

        let is_checked = matches!(self.state, CheckboxState::Checked);
        let on_change = self.on_change.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                (on_change)(!is_checked);
            }),
        );

        // Focus handlers
        let set_focused_in = set_focused.clone();
        renderer.register_handler(
            "focus",
            StdArc::new(move |_| {
                (set_focused_in)(true);
            }),
        );

        let set_focused_out = set_focused.clone();
        renderer.register_handler(
            "blur",
            StdArc::new(move |_| {
                (set_focused_out)(false);
            }),
        );

        renderer.pop_vnode();

        // Focus ring -- WCAG 2.4.7
        if is_focused {
            crate::draw_focus_ring(renderer, rect);
        }
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let label_width = self
            .label
            .as_ref()
            .map_or(0.0, |l| renderer.measure_text(l, 14.0).0);
        cvkg_core::Size {
            width: 18.0
                + if self.label.is_some() {
                    8.0 + label_width
                } else {
                    0.0
                },
            height: 22.0,
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        let label = self.label.as_deref().unwrap_or("Checkbox");
        let checked = match self.state {
            CheckboxState::Checked => Some(true),
            CheckboxState::Unchecked => Some(false),
            CheckboxState::Indeterminate => None,
        };
        let mut aria = AriaProperties::new(AriaRole::Checkbox, label);
        if let Some(c) = checked {
            aria = aria.checked(c);
        }
        Some(aria)
    }

    fn on_key_event(&self, key: &str, _modifiers: KeyModifiers) -> bool {
        match key {
            "Enter" | " " => {
                let new_state = match self.state {
                    CheckboxState::Checked => false,
                    CheckboxState::Unchecked | CheckboxState::Indeterminate => true,
                };
                (self.on_change)(new_state);
                true
            }
            _ => false,
        }
    }
}

/// Tabs component for tabbed navigation.
#[derive(Clone)]
pub struct Tabs<V> {
    tabs: Vec<(String, V)>,
    selected_index: usize,
}

impl<V: View> Tabs<V> {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            selected_index: 0,
        }
    }
    pub fn tab(mut self, label: impl Into<String>, content: V) -> Self {
        self.tabs.push((label.into(), content));
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.tabs.len().saturating_sub(1));
        self
    }
}

impl<V: View> Default for Tabs<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: View> View for Tabs<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Tabs");
        let tab_height = 36.0;
        for (idx, (label, _)) in self.tabs.iter().enumerate() {
            let tab_rect = Rect {
                x: rect.x + idx as f32 * (rect.width / self.tabs.len() as f32),
                y: rect.y,
                width: rect.width / self.tabs.len() as f32,
                height: tab_height,
            };
            let is_selected = idx == self.selected_index;
            renderer.fill_rounded_rect(tab_rect, 6.0, theme::surface_elevated());
            if is_selected {
                renderer.stroke_rect(tab_rect, theme::accent(), 2.0);
            }
            renderer.draw_text(
                label,
                tab_rect.x + 12.0,
                tab_rect.y + (tab_rect.height - 14.0) / 2.0,
                14.0,
                if is_selected {
                    theme::text()
                } else {
                    theme::text_muted()
                },
            );
        }
        if let Some((_, content)) = self.tabs.get(self.selected_index) {
            let content_rect = Rect {
                x: rect.x,
                y: rect.y + tab_height + 8.0,
                width: rect.width,
                height: rect.height - tab_height - 8.0,
            };
            content.render(renderer, content_rect);
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let max_h = self
            .tabs
            .iter()
            .map(|(_, c)| c.intrinsic_size(renderer, proposal).height)
            .fold(0.0, f32::max);
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: 36.0 + 8.0 + max_h,
        }
    }
}

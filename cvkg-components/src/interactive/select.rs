use crate::theme;
use crate::{Color, FONT_BASE, RADIUS_MD, RADIUS_SM};
use cvkg_core::{AriaProperties, AriaRole, KeyModifiers, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Select/Dropdown component with keyboard navigation, dropdown popover, and focus ring.
#[derive(Clone)]
pub struct Select<V> {
    placeholder: String,
    options: Vec<(String, V)>,
    selected_index: Option<usize>,
    is_open: bool,
    hover_index: Option<usize>,
    id_hash: u64,
}

impl<V: Clone> Select<V> {
    /// Create a new Select dropdown with a placeholder label.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Select;
    /// let select = Select::new("Select an option")
    ///     .option("Option 1", "val1")
    ///     .option("Option 2", "val2");
    /// ```
    pub fn new(placeholder: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let placeholder_string = placeholder.into();
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "select".hash(&mut s);
        placeholder_string.hash(&mut s);
        let id_hash = s.finish();
        Self {
            placeholder: placeholder_string,
            options: Vec::new(),
            selected_index: None,
            is_open: false,
            hover_index: None,
            id_hash,
        }
    }

    pub fn option(mut self, label: impl Into<String>, value: V) -> Self {
        self.options.push((label.into(), value));
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }
}

impl<V: Clone + View> View for Select<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Select");
        renderer.set_aria_role("combobox");
        renderer.set_aria_label(&self.placeholder);

        // Read open state from system state
        let is_open = cvkg_core::load_system_state()
            .get_component_state::<bool>(self.id_hash)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(self.is_open);

        // Main select box
        let border_color = if is_open {
            theme::input_border_focus()
        } else {
            theme::text_dim()
        };
        renderer.fill_rounded_rect(rect, RADIUS_MD, theme::surface());
        renderer.stroke_rect(rect, border_color, if is_open { 2.0 } else { 1.0 });

        // Focus ring when open
        if is_open {
            crate::draw_focus_ring(renderer, rect);
        }

        let display_text = self
            .selected_index
            .and_then(|i| self.options.get(i))
            .map(|(l, _)| l.as_str())
            .unwrap_or(&self.placeholder);
        renderer.draw_text(
            display_text,
            rect.x + 12.0,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            if self.selected_index.is_some() {
                theme::text()
            } else {
                theme::text_muted()
            },
        );

        // Chevron
        renderer.draw_text(
            if is_open { "▲" } else { "▼" },
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            12.0,
            theme::text_muted(),
        );

        // Dropdown popover
        if is_open {
            let item_height = 32.0;
            let popover_h = (self.options.len() as f32 * item_height).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            renderer.set_z_index(100.0);
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(popover_rect, 20.0, 1.2, 0.9);
            }
            renderer.fill_rounded_rect(popover_rect, RADIUS_MD, theme::surface_overlay());
            renderer.stroke_rect(popover_rect, theme::input_border_focus(), 1.0);

            // Read hover index from system state
            let hover_idx = cvkg_core::load_system_state()
                .get_component_state::<usize>(self.id_hash.wrapping_add(1))
                .and_then(|v| v.read().ok().map(|g| *g))
                .or(self.hover_index);

            for (i, (label, _)) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: popover_rect.x,
                    y: popover_rect.y + i as f32 * item_height,
                    width: popover_rect.width,
                    height: item_height,
                };

                let is_hovered = hover_idx == Some(i);

                // Selected highlight
                if self.selected_index == Some(i) {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, theme::list_item_selected());
                } else if is_hovered {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, theme::list_item_hover());
                }

                renderer.draw_text(
                    label,
                    item_rect.x + 12.0,
                    item_rect.y + (item_height - FONT_BASE) / 2.0,
                    FONT_BASE,
                    if self.selected_index == Some(i) {
                        theme::accent()
                    } else {
                        theme::text()
                    },
                );
            }
            renderer.set_z_index(0.0);
        }

        // Toggle on click
        let id_hash = self.id_hash;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    // If click is inside the main toggle rect, toggle open
                    if x >= rect.x
                        && x <= rect.x + rect.width
                        && y >= rect.y
                        && y <= rect.y + rect.height
                    {
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            let current = s
                                .get_component_state::<bool>(id_hash)
                                .and_then(|v| v.read().ok().map(|g| *g))
                                .unwrap_or(false);
                            s.set_component_state(id_hash, !current);
                            s
                        });
                    }
                }
            }),
        );

        // Keyboard navigation
        let options_count = self.options.len();
        let id_hash = self.id_hash;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let cvkg_core::Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowDown" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                let current = s
                                    .get_component_state::<usize>(id_hash.wrapping_add(1))
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(0);
                                let next = (current + 1).min(options_count.saturating_sub(1));
                                s.set_component_state(id_hash.wrapping_add(1), next);
                                s
                            });
                        }
                        "ArrowUp" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                let current = s
                                    .get_component_state::<usize>(id_hash.wrapping_add(1))
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(0);
                                let next = current.saturating_sub(1);
                                s.set_component_state(id_hash.wrapping_add(1), next);
                                s
                            });
                        }
                        "Enter" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                // Close the dropdown
                                s.set_component_state(id_hash, false);
                                s
                            });
                        }
                        "Escape" => {
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                s.set_component_state(id_hash, false);
                                s
                            });
                        }
                        _ => {}
                    }
                }
            }),
        );

        if is_open {
            let item_height = 32.0;
            let popover_h = (self.options.len() as f32 * item_height).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Pointer hover tracking
            let id_hash_hover = self.id_hash.wrapping_add(1);
            let pr = popover_rect;
            renderer.register_handler(
                "pointermove",
                Arc::new(move |event| {
                    if let cvkg_core::Event::PointerMove { x, y, .. } = event {
                        if x >= pr.x && x <= pr.x + pr.width && y >= pr.y && y <= pr.y + pr.height {
                            let hover_idx = ((y - pr.y) / item_height) as usize;
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                s.set_component_state(id_hash_hover, hover_idx);
                                s
                            });
                        }
                    }
                }),
            );
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(150.0),
            height: 36.0,
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        Some(AriaProperties::new(AriaRole::Combobox, &self.placeholder))
    }
}

/// Dropdown component for selecting from a list of options with a popover
pub struct Dropdown {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Dropdown {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Dropdown {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Dropdown");

        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "dropdown".hash(&mut s);
            self.options.len().hash(&mut s);
            s.finish()
        };

        // Lock-free read of expanded state
        let is_expanded = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(false)
        };

        // Main button
        renderer.fill_rounded_rect(rect, 4.0, theme::surface());
        renderer.stroke_rect(rect, theme::accent_hover(), 1.0);

        let selected = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text(
            if is_expanded { "▲" } else { "▼" },
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            theme::text_muted(),
        );

        if is_expanded {
            let popover_h = (self.options.len() as f32 * 30.0).min(200.0);
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Z-Index boost for popover
            renderer.set_z_index(100.0);
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(popover_rect, 20.0, 1.2, 0.9);
            }
            renderer.fill_rounded_rect(popover_rect, 4.0, theme::surface_overlay());
            renderer.stroke_rect(popover_rect, theme::input_border_focus(), 1.0);

            for (i, opt) in self.options.iter().enumerate() {
                let item_rect = Rect {
                    x: popover_rect.x,
                    y: popover_rect.y + i as f32 * 30.0,
                    width: popover_rect.width,
                    height: 30.0,
                };

                if i == self.selection {
                    renderer.fill_rect(item_rect, theme::list_item_selected());
                }

                renderer.draw_text(
                    opt,
                    item_rect.x + 8.0,
                    item_rect.y + (item_rect.height - 14.0) / 2.0,
                    14.0,
                    theme::text(),
                );
            }
            renderer.set_z_index(0.0);
        }

        let options_count = self.options.len();
        let on_change = self.on_change.clone();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    if is_expanded {
                        let popover_h = (options_count as f32 * 30.0).min(200.0);
                        let popover_rect = Rect {
                            x: rect.x,
                            y: rect.y + rect.height + 4.0,
                            width: rect.width,
                            height: popover_h,
                        };

                        if x >= popover_rect.x
                            && x <= popover_rect.x + popover_rect.width
                            && y >= popover_rect.y
                            && y <= popover_rect.y + popover_rect.height
                        {
                            let idx = ((y - popover_rect.y) / 30.0) as usize;
                            if idx < options_count {
                                on_change(idx);
                            }
                        }
                    }

                    // Toggle expanded state atomically
                    cvkg_core::update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(id_hash, !is_expanded);
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        for opt in &self.options {
            let (w, _) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: 32.0,
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        let label = self
            .options
            .get(self.selection)
            .map(|s| s.as_str())
            .unwrap_or("Select");
        Some(
            AriaProperties::new(AriaRole::Combobox, label)
                .expanded(false)
                .value(label.to_string()),
        )
    }

    fn on_key_event(&self, key: &str, _modifiers: KeyModifiers) -> bool {
        let len = self.options.len();
        if len == 0 {
            return false;
        }
        let new_sel = match key {
            "ArrowDown" => (self.selection + 1) % len,
            "ArrowUp" => {
                if self.selection == 0 {
                    len - 1
                } else {
                    self.selection - 1
                }
            }
            _ => return false,
        };
        (self.on_change)(new_sel);
        true
    }
}

/// Picker for selection from a list of options
#[derive(Clone)]
pub struct Picker {
    pub(crate) selection: usize,
    pub(crate) options: Vec<String>,
    pub(crate) on_change: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

impl Picker {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for Picker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("combobox");

        // Picker background
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        let selected_text = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text(
            &selected_text,
            rect.x + 10.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        // Chevron
        renderer.draw_text(
            "▼",
            rect.x + rect.width - 20.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            theme::text_muted(),
        );

        // Interaction (Cycle options on click)
        let on_change = self.on_change.clone();
        let selection = self.selection;
        let count = self.options.len();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |_| {
                if count > 0 {
                    (on_change)((selection + 1) % count);
                }
            }),
        );
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for opt in &self.options {
            let (w, h) = renderer.measure_text(opt, 14.0);
            max_w = max_w.max(w);
            max_h = max_h.max(h);
        }
        cvkg_core::Size {
            width: proposal.width.unwrap_or(max_w + 40.0).max(120.0),
            height: max_h + 16.0,
        }
    }
}

/// ColorPicker for RGBA color selection
pub struct ColorPicker {
    pub(crate) color: crate::Color,
    pub(crate) on_change: std::sync::Arc<dyn Fn(crate::Color) + Send + Sync>,
}

impl ColorPicker {
    pub fn new(
        color: crate::Color,
        on_change: impl Fn(crate::Color) + Send + Sync + 'static,
    ) -> Self {
        Self {
            color,
            on_change: std::sync::Arc::new(on_change),
        }
    }
}

impl View for ColorPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.set_aria_role("colorwell");

        // ColorPicker container
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        // Current color preview
        let preview_w = 40.0;
        let preview_rect = Rect {
            x: rect.x + 8.0,
            y: rect.y + 5.0,
            width: preview_w,
            height: rect.height - 10.0,
        };
        renderer.fill_rounded_rect(preview_rect, 2.0, self.color.as_array());
        renderer.stroke_rect(preview_rect, theme::border(), 1.0);

        // Color grid (4 demo colors -- user-facing swatches, not themed UI chrome)
        let colors = [
            Color::new(0.0, 0.0, 0.0, 1.0),    // Black
            Color::new(1.0, 1.0, 1.0, 1.0),    // White
            Color::new(0.9, 0.2, 0.2, 1.0),    // Red
            Color::new(0.0, 0.8, 0.9, 1.0),    // Cyan
        ];

        let grid_relative_x = 8.0 + preview_w + 12.0;
        let available_w = (rect.width - grid_relative_x - 10.0).max(0.0);
        let cell_w = available_w / 4.0;
        let cell_h = rect.height - 10.0;

        for (i, &col) in colors.iter().enumerate() {
            let cell_rect = Rect {
                x: rect.x + grid_relative_x + (i as f32 * (cell_w + 5.0)),
                y: rect.y + 5.0,
                width: cell_w,
                height: cell_h,
            };

            renderer.fill_rounded_rect(cell_rect, 2.0, col.as_array());

            // Interaction
            let on_change = self.on_change.clone();
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerClick { x, .. } = event {
                        if x >= cell_rect.x && x <= cell_rect.x + cell_rect.width {
                            (on_change)(col);
                        }
                    }
                }),
            );
        }
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(200.0),
            height: 32.0,
        }
    }
}

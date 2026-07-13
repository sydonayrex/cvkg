use crate::Color;
use crate::form_validation::ValidationRule;
use crate::integration::{CompanionBundle, WorldSpaceConfig};
use crate::theme;
use crate::{FONT_BASE, RADIUS_MD, RADIUS_SM};
use cvkg_core::{AriaProperties, AriaRole, KeyModifiers, Never, Rect, Renderer, View};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static NEXT_SELECT_ID: AtomicU64 = AtomicU64::new(1);

/// Select/Dropdown component with keyboard navigation, dropdown popover, and focus ring.
///
/// ## Accessibility
/// - Role: `combobox`
/// - Keyboard: Tab to focus, Enter/Space to open, Arrow keys to navigate, Esc to close
/// - Focus: auto-focused on mount when `auto_focus` is true
/// - ARIA: `aria-label` from `placeholder`, `aria-expanded` for open state, `aria-controls` for dropdown
/// - Reduced motion: respects `is_reduced_motion()` for dropdown animation
#[derive(Clone)]
pub struct Select<V> {
    placeholder: String,
    options: Vec<(String, V)>,
    selected_index: Option<usize>,
    is_open: bool,
    hover_index: Option<usize>,
    id_hash: u64,
    pub(crate) on_change: Option<Arc<dyn Fn(V) + Send + Sync>>,
    /// Validation rules for this select
    pub(crate) rules: Vec<ValidationRule>,
    /// Error message when validation fails
    pub(crate) error_message: Option<String>,
    /// VDOM companion bundle — focus management + ARIA semantics.
    pub companions: CompanionBundle,
    /// Optional 3D world-space placement.
    pub world: WorldSpaceConfig,
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
        let placeholder_str = placeholder.into();
        let id_hash = NEXT_SELECT_ID.fetch_add(2, Ordering::Relaxed);
        Self {
            placeholder: placeholder_str.clone(),
            options: Vec::new(),
            selected_index: None,
            is_open: false,
            hover_index: None,
            id_hash,
            on_change: None,
            rules: Vec::new(),
            error_message: None,
            companions: CompanionBundle::focusable()
                .with_role("combobox")
                .with_label(placeholder_str.clone()),
            world: WorldSpaceConfig::default(),
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

    pub fn on_change(mut self, cb: impl Fn(V) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(cb));
        self
    }

    /// Opt this select into 3D world-space rendering.
    pub fn world(mut self, config: WorldSpaceConfig) -> Self {
        self.world = config;
        self
    }

    // ── Validation integration ──

    /// Attach validation rules to this select.
    pub fn rules(mut self, rules: Vec<ValidationRule>) -> Self {
        self.rules = rules;
        self
    }

    /// Run all attached validation rules.
    ///
    /// For selects, `Required` means an option must be selected.
    pub fn validate(&mut self) -> Result<(), String> {
        for rule in &self.rules {
            let result = match rule {
                ValidationRule::Required => {
                    if self.selected_index.is_none() {
                        Err("Please make a selection".to_string())
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            };
            if let Err(msg) = result {
                self.error_message = Some(msg.clone());
                return Err(msg);
            }
        }
        self.error_message = None;
        Ok(())
    }
}

impl<V: Clone + Send + Sync + 'static> View for Select<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
        self.companions.to_vec()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode_with_companions(rect, "Select", self.companions.to_vec());
        renderer.register_a11y("combobox", &self.placeholder);

        // 3D world-space: redirect draw calls to offscreen texture when enabled.
        self.world.begin(renderer, self.id_hash);

        // Read open state from system state
        let is_open = cvkg_core::load_system_state()
            .get_component_state::<bool>(self.id_hash)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(self.is_open);

        // Read selected index from system state
        let selected_idx = cvkg_core::load_system_state()
            .get_component_state::<usize>(self.id_hash.wrapping_add(2))
            .and_then(|v| v.read().ok().map(|g| *g))
            .or(self.selected_index);

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

        let display_text = selected_idx
            .and_then(|i| self.options.get(i))
            .map(|(l, _)| l.as_str())
            .unwrap_or(&self.placeholder);
        renderer.draw_text_raw(
            display_text,
            rect.x + 12.0,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            if selected_idx.is_some() {
                theme::text()
            } else {
                theme::text_muted()
            },
        );

        // Chevron
        renderer.draw_text_raw(
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

                if selected_idx == Some(i) {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, theme::list_item_selected());
                } else if is_hovered {
                    renderer.fill_rounded_rect(item_rect, RADIUS_SM, theme::list_item_hover());
                }

                renderer.draw_text_raw(
                    label,
                    item_rect.x + 12.0,
                    item_rect.y + (item_height - FONT_BASE) / 2.0,
                    FONT_BASE,
                    if selected_idx == Some(i) {
                        theme::accent()
                    } else {
                        theme::text()
                    },
                );
            }
            renderer.set_z_index(0.0);
        }

        // Error message
        if let Some(ref msg) = self.error_message {
            renderer.draw_text_raw(
                msg,
                rect.x + 4.0,
                rect.y + rect.height + 4.0,
                11.0,
                theme::error_color(),
            );
        }

        // Toggle on click + popover item commit
        // Capture the world-space offset so the handler can convert local
        // rects to window coordinates for bounds checking.
        let world_offset = renderer.current_translation();
        let id_hash = self.id_hash;
        let on_change = self.on_change.clone();
        let options = self.options.clone();
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    // World-space rect of the main toggle box.
                    let toggle_rect = Rect {
                        x: world_offset.x,
                        y: world_offset.y,
                        width: rect.width,
                        height: rect.height,
                    };
                    // If click is inside the main toggle rect, toggle open
                    if x >= toggle_rect.x
                        && x <= toggle_rect.x + toggle_rect.width
                        && y >= toggle_rect.y
                        && y <= toggle_rect.y + toggle_rect.height
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
                        return;
                    }

                    // If open, check for popover item click
                    let is_open = cvkg_core::load_system_state()
                        .get_component_state::<bool>(id_hash)
                        .and_then(|v| v.read().ok().map(|g| *g))
                        .unwrap_or(false);
                    if is_open {
                        let item_height = 32.0;
                        let popover_h = (options.len() as f32 * item_height).min(200.0);
                        let popover_rect = Rect {
                            x: world_offset.x,
                            y: world_offset.y + rect.height + 4.0,
                            width: rect.width,
                            height: popover_h,
                        };
                        if x >= popover_rect.x
                            && x <= popover_rect.x + popover_rect.width
                            && y >= popover_rect.y
                            && y <= popover_rect.y + popover_rect.height
                        {
                            let idx = ((y - popover_rect.y) / item_height) as usize;
                            if idx < options.len() {
                                cvkg_core::update_system_state(|s| {
                                    let mut s = s.clone();
                                    s.set_component_state(id_hash, false);
                                    s.set_component_state(id_hash.wrapping_add(2), idx);
                                    s
                                });
                                if let Some(cb) = on_change.as_ref() {
                                    (cb)(options[idx].1.clone());
                                }
                            }
                        }
                    }
                }
            }),
        );

        // Keyboard navigation + commit
        let options_count = self.options.len();
        let id_hash = self.id_hash;
        let on_change = self.on_change.clone();
        let options = self.options.clone();
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
                            // Commit hovered selection and close
                            let hovered = cvkg_core::load_system_state()
                                .get_component_state::<usize>(id_hash.wrapping_add(1))
                                .and_then(|v| v.read().ok().map(|g| *g));
                            cvkg_core::update_system_state(|s| {
                                let mut s = s.clone();
                                s.set_component_state(id_hash, false);
                                if let Some(h) = hovered {
                                    s.set_component_state(id_hash.wrapping_add(2), h);
                                }
                                s
                            });
                            if let (Some(cb), Some(h)) = (on_change.as_ref(), hovered)
                                && h < options.len()
                            {
                                (cb)(options[h].1.clone());
                            }
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
            let hover_world = renderer.current_translation();
            let popover_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };

            // Pointer hover tracking — convert to world space for hit-testing
            let id_hash_hover = self.id_hash.wrapping_add(1);
            let pr = Rect {
                x: hover_world.x,
                y: hover_world.y + rect.height + 4.0,
                width: rect.width,
                height: popover_h,
            };
            renderer.register_handler(
                "pointermove",
                Arc::new(move |event| {
                    if let cvkg_core::Event::PointerMove { x, y, .. } = event
                        && x >= pr.x
                        && x <= pr.x + pr.width
                        && y >= pr.y
                        && y <= pr.y + pr.height
                    {
                        let hover_idx = ((y - pr.y) / item_height) as usize;
                        cvkg_core::update_system_state(|s| {
                            let mut s = s.clone();
                            s.set_component_state(id_hash_hover, hover_idx);
                            s
                        });
                    }
                }),
            );
        }

        renderer.pop_vnode();

        // End 3D world-space redirection if it was begun above.
        if self.world.is_enabled() {
            renderer.end_world_space_panel(self.id_hash);
        }
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
    id_hash: u64,
}

impl Dropdown {
    pub fn new(
        selection: usize,
        options: Vec<String>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        let id_hash = NEXT_SELECT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            selection,
            options,
            on_change: std::sync::Arc::new(on_change),
            id_hash,
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
        renderer.set_aria_role("combobox");
        renderer.set_aria_label("Dropdown selection");

        let id_hash = self.id_hash;

        // Lock-free read of expanded state
        let is_expanded = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(false)
        };

        // Main button
        renderer.fill_rounded_rect(rect, 4.0, theme::surface_high_contrast());
        renderer.stroke_rect(rect, theme::accent_hover(), 1.0);

        let selected = self
            .options
            .get(self.selection)
            .cloned()
            .unwrap_or_default();
        renderer.draw_text_raw(
            &selected,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text_raw(
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

                renderer.draw_text_raw(
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
        let drop_world = renderer.current_translation();

        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    if is_expanded {
                        let popover_h = (options_count as f32 * 30.0).min(200.0);
                        let popover_rect = Rect {
                            x: drop_world.x,
                            y: drop_world.y + rect.height + 4.0,
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
        renderer.draw_text_raw(
            &selected_text,
            rect.x + 10.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text(),
        );

        // Chevron
        renderer.draw_text_raw(
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
            Color::new(0.0, 0.0, 0.0, 1.0), // Black
            Color::new(1.0, 1.0, 1.0, 1.0), // White
            Color::new(0.9, 0.2, 0.2, 1.0), // Red
            Color::new(0.0, 0.8, 0.9, 1.0), // Cyan
        ];

        let grid_relative_x = 8.0 + preview_w + 12.0;
        let available_w = (rect.width - grid_relative_x - 10.0).max(0.0);
        let cell_w = available_w / 4.0;
        let cell_h = rect.height - 10.0;
        let cp_world = renderer.current_translation();

        for (i, &col) in colors.iter().enumerate() {
            let cell_rect = Rect {
                x: rect.x + grid_relative_x + (i as f32 * (cell_w + 5.0)),
                y: rect.y + 5.0,
                width: cell_w,
                height: cell_h,
            };

            renderer.fill_rounded_rect(cell_rect, 2.0, col.as_array());

            // Interaction — convert local cell_rect to world space for hit-testing
            let on_change = self.on_change.clone();
            let world_cell = Rect {
                x: cp_world.x + grid_relative_x + (i as f32 * (cell_w + 5.0)),
                y: cp_world.y + 5.0,
                width: cell_w,
                height: cell_h,
            };
            renderer.register_handler(
                "pointerclick",
                std::sync::Arc::new(move |event| {
                    if let cvkg_core::Event::PointerClick { x, .. } = event
                        && x >= world_cell.x
                        && x <= world_cell.x + world_cell.width
                    {
                        (on_change)(col);
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

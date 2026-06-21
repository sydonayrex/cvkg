use crate::theme;
use crate::{FONT_BASE, RADIUS_MD};
use cvkg_core::{AriaProperties, AriaRole, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Input validation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Default,
    Focused,
    Error,
    Success,
    Disabled,
}

/// Single-line text input with cursor, selection, clipboard, and undo.
#[derive(Clone)]
pub struct Input {
    pub(crate) placeholder: String,
    pub(crate) text: String,
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) on_commit: Arc<dyn Fn(String) + Send + Sync>,
    pub(crate) is_focused: bool,
    pub(crate) input_state: InputState,
    pub(crate) error_message: Option<String>,
    /// Unique hash for this input instance (for system state)
    pub(crate) state_id: u64,
}

impl Input {
    /// Create a new Input field.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::Input;
    /// let input = Input::new("Placeholder text")
    ///     .on_change(|text| println!("Input: {}", text));
    /// ```
    pub fn new(placeholder: impl Into<String>) -> Self {
        use std::hash::{Hash, Hasher};
        let mut s = std::collections::hash_map::DefaultHasher::new();
        "input".hash(&mut s);
        std::time::SystemTime::now().hash(&mut s);
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            on_change: Arc::new(|_| {}),
            on_commit: Arc::new(|_| {}),
            is_focused: false,
            input_state: InputState::Default,
            error_message: None,
            state_id: s.finish(),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }

    pub fn on_commit(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_commit = Arc::new(callback);
        self
    }

    pub fn focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        if is_focused {
            self.input_state = InputState::Focused;
        }
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.input_state = InputState::Error;
        self.error_message = Some(message.into());
        self
    }

    pub fn success(mut self) -> Self {
        self.input_state = InputState::Success;
        self
    }

    fn bg_color(&self) -> [f32; 4] {
        match self.input_state {
            InputState::Disabled => theme::surface(),
            InputState::Error => theme::error_color(),
            InputState::Success => theme::success(),
            _ => theme::surface_elevated(),
        }
    }

    fn border_color(&self) -> [f32; 4] {
        match self.input_state {
            InputState::Focused => theme::accent(),
            InputState::Error => theme::error_color(),
            InputState::Success => theme::success(),
            InputState::Disabled => theme::text_dim(),
            _ => theme::border(),
        }
    }

    /// Get or initialize the TextInputState from system state.
    fn get_text_state(&self) -> cvkg_core::TextInputState {
        let sys = cvkg_core::load_system_state();
        if let Some(arc) = sys.get_component_state::<cvkg_core::TextInputState>(self.state_id) {
            arc.read().ok().map(|g| g.clone()).unwrap_or_default()
        } else {
            let mut state = cvkg_core::TextInputState::new(self.text.clone());
            state.focused = self.is_focused;
            state
        }
    }

    /// Save TextInputState to system state.
    fn save_text_state(&self, state: &cvkg_core::TextInputState) {
        cvkg_core::update_system_state(|s| {
            let mut ns = s.clone();
            ns.set_component_state(self.state_id, state.clone());
            ns
        });
    }

    /// Get the display text (or placeholder if empty).
    fn display_text(&self) -> &str {
        if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        }
    }

    /// Compute the pixel offset for a given byte position in the text.
    fn text_offset_for_pos(&self, renderer: &mut dyn Renderer, byte_pos: usize) -> f32 {
        let prefix = &self.text[..byte_pos.min(self.text.len())];
        if prefix.is_empty() {
            0.0
        } else {
            renderer.measure_text(prefix, FONT_BASE).0
        }
    }
}

impl View for Input {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Input");
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&self.placeholder);

        let bg = self.bg_color();
        let border = self.border_color();
        let is_disabled = self.input_state == InputState::Disabled;

        // Input background
        renderer.fill_rounded_rect(rect, RADIUS_MD, bg);
        renderer.stroke_rect(rect, border, if self.is_focused { 2.0 } else { 1.0 });

        // Focus ring -- WCAG 2.4.7
        if self.is_focused && !is_disabled {
            crate::draw_focus_ring(renderer, rect);
        }

        // Get text input state
        let text_state = self.get_text_state();
        let display_text = self.display_text();
        let text_color = if self.text.is_empty() {
            theme::text_muted()
        } else if is_disabled {
            theme::disabled_text()
        } else {
            theme::text()
        };

        // Text area
        let text_rect = Rect {
            x: rect.x + 8.0,
            y: rect.y,
            width: rect.width - 16.0,
            height: rect.height,
        };

        // Render selection highlight
        if text_state.focused
            && let Some((sel_start, sel_end)) = text_state.selection_range()
        {
            let x_start = self.text_offset_for_pos(renderer, sel_start);
            let x_end = self.text_offset_for_pos(renderer, sel_end);
            let sel_rect = Rect {
                x: text_rect.x + x_start,
                y: rect.y + (rect.height - FONT_BASE) - 2.0,
                width: x_end - x_start,
                height: FONT_BASE + 4.0,
            };
            renderer.fill_rounded_rect(
                sel_rect,
                2.0,
                theme::list_item_selected(),
            );
        }

        // Render text
        renderer.draw_text(
            display_text,
            text_rect.x,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            text_color,
        );

        // Render cursor (blinking)
        if text_state.focused && !is_disabled {
            let cursor_x_offset = self.text_offset_for_pos(renderer, text_state.cursor_pos);
            let cursor_x = text_rect.x + cursor_x_offset;
            let cursor_y = rect.y + (rect.height - 16.0) / 2.0;
            let time = renderer.elapsed_time();
            let alpha = if (time * 2.0).sin() > 0.0 { 1.0 } else { 0.3 };
            renderer.draw_line(
                cursor_x,
                cursor_y,
                cursor_x,
                cursor_y + 16.0,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    alpha,
                ],
                2.0,
            );
        }

        // Error message
        if let Some(ref msg) = self.error_message {
            renderer.draw_text(
                msg,
                rect.x + 8.0,
                rect.y + rect.height + 4.0,
                12.0,
                theme::error_color(),
            );
        }

        // Save state
        self.save_text_state(&text_state);

        // === Interaction handlers ===
        if !is_disabled {
            // Keydown handler
            {
                let on_change = self.on_change.clone();
                let on_commit = self.on_commit.clone();
                let state_id = self.state_id;
                renderer.register_handler(
                    "keydown",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::KeyDown { key, .. } = event {
                            let mut changed = false;
                            let mut commit = false;

                            let sys = cvkg_core::load_system_state();
                            let old_text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().ok().map(|g| g.clone()).unwrap_or_default()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let mut text_state = old_text_state.clone();

                            match key.as_str() {
                                s if s.len() == 1
                                    && !s.chars().next().expect("unexpected None").is_control() =>
                                {
                                    text_state.insert(s);
                                    changed = true;
                                }
                                "Back" | "Backspace" => {
                                    text_state.delete(true, 1);
                                    changed = true;
                                }
                                "Delete" => {
                                    text_state.delete(false, 1);
                                    changed = true;
                                }
                                "ArrowLeft" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::Backward, false);
                                }
                                "ArrowRight" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::Forward, false);
                                }
                                "ArrowUp" | "Home" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::LineStart, false);
                                }
                                "ArrowDown" | "End" => {
                                    text_state
                                        .move_cursor(cvkg_core::TextDirection::LineEnd, false);
                                }
                                "Enter" | "Return" => {
                                    commit = true;
                                }
                                _ => {}
                            }

                            if changed {
                                let new_text = text_state.text.clone();
                                let new_text_state = text_state.clone();
                                let on_change_clone1 = on_change.clone();
                                let on_change_clone2 = on_change.clone();

                                let label = match key.as_str() {
                                    "Back" | "Backspace" | "Delete" => "Delete",
                                    _ => "Type",
                                };

                                let u_state = old_text_state.clone();
                                let r_state = new_text_state.clone();

                                cvkg_core::update_system_state(move |s| {
                                    let mut ns = s.clone();
                                    ns.set_component_state(state_id, new_text_state.clone());

                                    let oc_undo = on_change_clone1.clone();
                                    let oc_redo = on_change_clone2.clone();
                                    let u_state = u_state.clone();
                                    let r_state = r_state.clone();

                                    ns.undo_manager.push_coalesceable(
                                        label,
                                        move || {
                                            cvkg_core::update_system_state({
                                                let u_state = u_state.clone();
                                                move |st| {
                                                    let mut nst = st.clone();
                                                    nst.set_component_state(
                                                        state_id,
                                                        u_state.clone(),
                                                    );
                                                    nst
                                                }
                                            });
                                            (oc_undo)(u_state.text.clone());
                                        },
                                        move || {
                                            cvkg_core::update_system_state({
                                                let r_state = r_state.clone();
                                                move |st| {
                                                    let mut nst = st.clone();
                                                    nst.set_component_state(
                                                        state_id,
                                                        r_state.clone(),
                                                    );
                                                    nst
                                                }
                                            });
                                            (oc_redo)(r_state.text.clone());
                                        },
                                    );
                                    ns
                                });
                                (on_change)(new_text);
                            }

                            if commit {
                                let sys = cvkg_core::load_system_state();
                                if let Some(arc) =
                                    sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                                {
                                    let text =
                                        arc.read().ok().map(|g| g.text.clone()).unwrap_or_default();
                                    (on_commit)(text);
                                }
                            }
                        }
                    }),
                );
            }

            // IME handler
            {
                let on_change = self.on_change.clone();
                let state_id = self.state_id;
                renderer.register_handler(
                    "ime",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::Ime(composition) = event {
                            let sys = cvkg_core::load_system_state();
                            let old_text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().ok().map(|g| g.clone()).unwrap_or_default()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let mut text_state = old_text_state.clone();
                            text_state.insert(&composition);
                            let new_text = text_state.text.clone();
                            let new_text_state = text_state.clone();
                            let on_change_clone1 = on_change.clone();
                            let on_change_clone2 = on_change.clone();

                            let u_state = old_text_state.clone();
                            let r_state = new_text_state.clone();

                            cvkg_core::update_system_state(move |s| {
                                let mut ns = s.clone();
                                ns.set_component_state(state_id, new_text_state.clone());

                                let oc_undo = on_change_clone1.clone();
                                let oc_redo = on_change_clone2.clone();
                                let u_state = u_state.clone();
                                let r_state = r_state.clone();

                                ns.undo_manager.push_coalesceable(
                                    "Type",
                                    move || {
                                        cvkg_core::update_system_state({
                                            let u_state = u_state.clone();
                                            move |st| {
                                                let mut nst = st.clone();
                                                nst.set_component_state(state_id, u_state.clone());
                                                nst
                                            }
                                        });
                                        (oc_undo)(u_state.text.clone());
                                    },
                                    move || {
                                        cvkg_core::update_system_state({
                                            let r_state = r_state.clone();
                                            move |st| {
                                                let mut nst = st.clone();
                                                nst.set_component_state(state_id, r_state.clone());
                                                nst
                                            }
                                        });
                                        (oc_redo)(r_state.text.clone());
                                    },
                                );
                                ns
                            });
                            (on_change)(new_text);
                        }
                    }),
                );
            }

            // Pointer interaction
            {
                let state_id = self.state_id;
                renderer.register_handler(
                    "pointerclick",
                    Arc::new(move |event| {
                        if let cvkg_core::Event::PointerClick { x, .. } = event {
                            let sys = cvkg_core::load_system_state();
                            let mut text_state = if let Some(arc) =
                                sys.get_component_state::<cvkg_core::TextInputState>(state_id)
                            {
                                arc.read().ok().map(|g| g.clone()).unwrap_or_default()
                            } else {
                                cvkg_core::TextInputState::new("")
                            };
                            let text_x = x - rect.x - 8.0;
                            let byte_pos =
                                Input::pos_for_text_offset_static(&text_state.text, text_x);
                            text_state.cursor_pos = byte_pos;
                            text_state.selection_anchor = None;
                            text_state.focused = true;
                            cvkg_core::update_system_state(|s| {
                                let mut ns = s.clone();
                                ns.set_component_state(state_id, text_state);
                                ns
                            });
                        }
                    }),
                );
            }
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(
        &self,
        _renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(200.0),
            height: 44.0,
        }
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        Some(
            AriaProperties::new(AriaRole::Textbox, &self.placeholder)
                .value(self.text.clone())
                .focused(self.is_focused),
        )
    }
}

impl Input {
    /// Static helper for computing byte position from pixel offset.
    fn pos_for_text_offset_static(text: &str, x_offset: f32) -> usize {
        let avg_char_width = 8.0;
        let estimated = (x_offset / avg_char_width).round() as usize;
        estimated.min(text.len())
    }
}

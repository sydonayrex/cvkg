//! AutoComplete component -- text input with a filtered dropdown list.
//!
//! Renders an Input-like text field that filters a list of options as the user
//! types. A glassmorphic dropdown shows up to 6 matching items. Supports
//! keyboard navigation (ArrowDown / ArrowUp / Enter / Escape) and pointer
//! selection.

use crate::theme;
use crate::{RADIUS_LG, RADIUS_MD, RADIUS_SM};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Internal state stored per-component instance via the global system state.
#[derive(Clone)]
struct AutoCompleteState {
    /// Whether the dropdown is currently visible.
    is_open: bool,
    /// Index into `filtered_indices` of the currently highlighted entry.
    selection: Option<usize>,
    /// Indices into the options list that match the current filter text.
    filtered_indices: Vec<usize>,
}

impl AutoCompleteState {
    fn new(options: &[String]) -> Self {
        let filtered_indices: Vec<usize> = (0..options.len()).collect();
        Self {
            is_open: false,
            selection: None,
            filtered_indices,
        }
    }
}

/// AutoComplete -- a text input with a filtered, glassmorphic dropdown.
///
/// # Examples
/// ```
/// use cvkg_components::AutoComplete;
/// let ac = AutoComplete::new(
///     "Search...",
///     vec!["alpha".into(), "beta".into(), "gamma".into()],
///     |text| println!("changed: {}", text),
///     |sel| println!("selected: {}", sel),
/// );
/// ```
#[derive(Clone)]
pub struct AutoComplete {
    /// Placeholder text shown when the input is empty.
    pub(crate) placeholder: String,
    /// The current text value.
    pub(crate) text: String,
    /// All available options.
    pub(crate) options: Vec<String>,
    /// Callback invoked when the text changes.
    pub(crate) on_change: Arc<dyn Fn(String) + Send + Sync>,
    /// Callback invoked when an option is selected.
    pub(crate) on_select: Arc<dyn Fn(String) + Send + Sync>,
}

impl AutoComplete {
    /// Create a new AutoComplete component.
    ///
    /// # Arguments
    /// * `placeholder` -- hint text shown when the field is empty.
    /// * `options` -- the full list of completable strings.
    /// * `on_change` -- called with the current text on every keystroke.
    /// * `on_select` -- called with the selected option string.
    pub fn new(
        placeholder: impl Into<String>,
        options: Vec<String>,
        on_change: impl Fn(String) + Send + Sync + 'static,
        on_select: impl Fn(String) + Send + Sync + 'static,
    ) -> Self {
        Self {
            placeholder: placeholder.into(),
            text: String::new(),
            options,
            on_change: Arc::new(on_change),
            on_select: Arc::new(on_select),
        }
    }

    /// Set the initial text value (builder method).
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    /// Compute a stable hash from the placeholder string (used as component ID).
    fn id_hash(placeholder: &str) -> u64 {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        placeholder.hash(&mut s);
        s.finish()
    }

    /// Load the per-instance state from the global system state.
    fn load_state(&self) -> AutoCompleteState {
        let id = AutoComplete::id_hash(&self.placeholder);
        let sys = cvkg_core::load_system_state();
        sys.get_component_state::<AutoCompleteState>(id)
            .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
            .unwrap_or_else(|| AutoCompleteState::new(&self.options))
    }
}

impl View for AutoComplete {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::layout::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(220.0),
            height: 38.0,
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AutoComplete");
        renderer.set_aria_role("combobox");
        renderer.set_aria_label(&self.placeholder);

        let state = self.load_state();

        // ── Text field background ──────────────────────────────────────────
        renderer.fill_rounded_rect(rect, RADIUS_MD, [0.06, 0.06, 0.08, 1.0]);
        renderer.stroke_rounded_rect(rect, RADIUS_MD, [0.25, 0.25, 0.28, 1.0], 1.5);

        // ── Display text or placeholder ────────────────────────────────────
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };
        let text_color = [1.0, 1.0, 1.0, 1.0];
        renderer.draw_text_raw(
            display_text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );

        // ── Cursor ─────────────────────────────────────────────────────────
        if state.is_open && !self.text.is_empty() {
            let (tw, _) = renderer.measure_text(&self.text, 14.0);
            let cursor_x = rect.x + 8.0 + tw;
            let cursor_y = rect.y + (rect.height - 16.0) / 2.0;
            renderer.draw_line(
                cursor_x,
                cursor_y,
                cursor_x,
                cursor_y + 16.0,
                theme::accent(),
                2.0,
            );
        }

        // ── Dropdown ───────────────────────────────────────────────────────
        let dropdown_visible = state.is_open && !state.filtered_indices.is_empty();
        let item_height = 28.0;
        let dropdown_height = if dropdown_visible {
            state.filtered_indices.len() as f32 * item_height + 8.0
        } else {
            0.0
        };
        let dropdown_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height + 4.0,
            width: rect.width,
            height: dropdown_height,
        };

        if dropdown_visible {
            // Glassmorphic background: bifrost blur + rounded rect fill
            if crate::theme::glassmorphism_enabled() {
                renderer.bifrost(dropdown_rect, 12.0, 1.2, 0.15);
            }
            renderer.fill_rounded_rect(
                dropdown_rect,
                RADIUS_LG,
                [0.05, 0.05, 0.07, 1.0],
            );
            renderer.stroke_rounded_rect(
                dropdown_rect,
                RADIUS_LG,
                [0.25, 0.25, 0.28, 1.0],
                1.5,
            );

            // Render each filtered option
            for (vis_idx, &opt_idx) in state.filtered_indices.iter().enumerate() {
                let item_rect = Rect {
                    x: dropdown_rect.x + 4.0,
                    y: dropdown_rect.y + 4.0 + vis_idx as f32 * item_height,
                    width: dropdown_rect.width - 8.0,
                    height: item_height,
                };

                // Highlight the selected item
                if state.selection == Some(vis_idx) {
                    renderer.fill_rounded_rect(
                        item_rect,
                        RADIUS_SM,
                        theme::with_alpha(theme::accent(), 0.3),
                    );
                }

                let opt_text = &self.options[opt_idx];
                let tc = if state.selection == Some(vis_idx) {
                    theme::accent_hover()
                } else {
                    theme::text()
                };
                renderer.draw_text_raw(
                    opt_text,
                    item_rect.x + 8.0,
                    item_rect.y + (item_height - 13.0) / 2.0,
                    13.0,
                    tc,
                );
            }
        }

        // ── Event handlers ─────────────────────────────────────────────────
        let on_change = self.on_change.clone();
        let on_select = self.on_select.clone();
        let options = self.options.clone();

        // Shared mutable text for keydown handler
        let text_arc = Arc::new(std::sync::Mutex::new(self.text.clone()));

        // KeyDown: typing, backspace, arrows, enter, escape
        let text_kd = text_arc.clone();
        let on_change_kd = on_change.clone();
        let on_select_kd = on_select.clone();
        let options_kd = options.clone();
        let placeholder_kd = self.placeholder.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    let mut changed = false;
                    let mut new_text = String::new();

                    if let Ok(mut guard) = text_kd.lock() {
                        if key.len() == 1 {
                            guard.push_str(&key);
                            changed = true;
                        } else if key == "Back" || key == "Backspace" {
                            guard.pop();
                            changed = true;
                        }
                        if changed {
                            new_text = guard.clone();
                        }
                    }

                    if changed {
                        (on_change_kd)(new_text.clone());

                        // Re-filter and open dropdown
                        let query_lower = new_text.to_lowercase();
                        let filtered: Vec<usize> = options_kd
                            .iter()
                            .enumerate()
                            .filter(|(_, opt)| opt.to_lowercase().contains(&query_lower))
                            .map(|(i, _)| i)
                            .take(6)
                            .collect();

                        let open = !filtered.is_empty();
                        let sel = if open { Some(0) } else { None };
                        let id = AutoComplete::id_hash(&placeholder_kd);
                        let new_state = AutoCompleteState {
                            is_open: open,
                            selection: sel,
                            filtered_indices: filtered,
                        };
                        let saved = new_state.clone();
                        cvkg_core::update_system_state(move |sys| {
                            let mut next = sys.clone();
                            next.set_component_state(id, saved.clone());
                            next
                        });
                        return;
                    }

                    // Arrow / Enter / Escape navigation
                    let id = AutoComplete::id_hash(&placeholder_kd);

                    let sys = cvkg_core::load_system_state();
                    let current_state = sys
                        .get_component_state::<AutoCompleteState>(id)
                        .and_then(|lock| lock.read().ok().map(|g| (*g).clone()));

                    if let Some(mut st) = current_state {
                        match key.as_str() {
                            "ArrowDown" => {
                                let max = st.filtered_indices.len();
                                if max > 0 {
                                    st.selection = Some((st.selection.unwrap_or(0) + 1) % max);
                                }
                                let saved = st.clone();
                                cvkg_core::update_system_state(move |sys| {
                                    let mut next = sys.clone();
                                    next.set_component_state(id, saved.clone());
                                    next
                                });
                            }
                            "ArrowUp" => {
                                let max = st.filtered_indices.len();
                                if max > 0 {
                                    st.selection = Some(
                                        st.selection
                                            .map(|s| if s == 0 { max - 1 } else { s - 1 })
                                            .unwrap_or(max - 1),
                                    );
                                }
                                let saved = st.clone();
                                cvkg_core::update_system_state(move |sys| {
                                    let mut next = sys.clone();
                                    next.set_component_state(id, saved.clone());
                                    next
                                });
                            }
                            "Enter" | "Return" => {
                                if let Some(sel_idx) = st.selection
                                    && let Some(&opt_idx) = st.filtered_indices.get(sel_idx)
                                {
                                    let selected = options_kd[opt_idx].clone();
                                    if let Ok(mut guard) = text_kd.lock() {
                                        *guard = selected.clone();
                                    }
                                    (on_select_kd)(selected);
                                    st.is_open = false;
                                    st.selection = None;
                                    let saved = st.clone();
                                    cvkg_core::update_system_state(move |sys| {
                                        let mut next = sys.clone();
                                        next.set_component_state(id, saved.clone());
                                        next
                                    });
                                }
                            }
                            "Escape" => {
                                st.is_open = false;
                                st.selection = None;
                                let saved = st.clone();
                                cvkg_core::update_system_state(move |sys| {
                                    let mut next = sys.clone();
                                    next.set_component_state(id, saved.clone());
                                    next
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }),
        );

        // Pointer click on dropdown items
        let text_click = text_arc;
        let on_select_click = on_select;
        let options_click = options;
        let placeholder_click = self.placeholder.clone();
        let rect_clone = rect;
        let dropdown_height_for_click = dropdown_height;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let Event::PointerClick { x, y, .. } = event {
                    if x >= rect_clone.x && x <= rect_clone.x + rect_clone.width {
                        let id = AutoComplete::id_hash(&placeholder_click);

                        // Check if click is inside the input field
                        if y >= rect_clone.y && y <= rect_clone.y + rect_clone.height {
                            let sys = cvkg_core::load_system_state();
                            if let Some(st) = sys
                                .get_component_state::<AutoCompleteState>(id)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                && !st.is_open
                            {
                                let mut new_st = st.clone();
                                new_st.is_open = true;
                                if new_st.filtered_indices.is_empty() {
                                    new_st.filtered_indices = (0..options_click.len()).collect();
                                }
                                new_st.selection = if new_st.filtered_indices.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                };
                                let saved = new_st.clone();
                                cvkg_core::update_system_state(move |sys| {
                                    let mut next = sys.clone();
                                    next.set_component_state(id, saved.clone());
                                    next
                                });
                            }
                            return;
                        }

                        // Click on a dropdown item
                        let rel_y = y - (rect_clone.y + rect_clone.height + 4.0);
                        if rel_y >= 0.0 && rel_y < dropdown_height_for_click {
                            let vis_idx = (rel_y / item_height) as usize;

                            let sys = cvkg_core::load_system_state();
                            if let Some(st) = sys
                                .get_component_state::<AutoCompleteState>(id)
                                .and_then(|lock| lock.read().ok().map(|g| (*g).clone()))
                                && let Some(&opt_idx) = st.filtered_indices.get(vis_idx)
                            {
                                let selected = options_click[opt_idx].clone();
                                if let Ok(mut guard) = text_click.lock() {
                                    *guard = selected.clone();
                                }
                                (on_select_click)(selected);

                                let mut new_st = st.clone();
                                new_st.is_open = false;
                                new_st.selection = None;
                                let saved = new_st.clone();
                                cvkg_core::update_system_state(move |sys| {
                                    let mut next = sys.clone();
                                    next.set_component_state(id, saved.clone());
                                    next
                                });
                            }
                        }
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}

impl cvkg_core::layout::LayoutView for AutoComplete {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(220.0),
            height: 38.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
    }
}

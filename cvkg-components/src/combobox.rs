use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View, load_system_state, update_system_state};
use std::sync::Arc;

// System-state hash keys for the combobox
const COMBO_OPEN_HASH: u64 = 0xC00_0001;
const COMBO_SELECTED_HASH: u64 = 0xC00_0002;
const COMBO_SEARCH_HASH: u64 = 0xC00_0003;

// =============================================================================
// COMBOBOX — Searchable select with keyboard navigation
// =============================================================================

/// A searchable dropdown select component.
///
/// Displays the currently selected value (or a placeholder) in a clickable field.
/// When clicked, opens a dropdown with a search input at the top and a filtered
/// list of options below. Supports keyboard navigation (Arrow Up/Down, Enter, Escape).
#[derive(Clone)]
pub struct Combobox {
    pub(crate) options: Vec<String>,
    pub(crate) selected: Option<usize>,
    pub(crate) on_change: Arc<dyn Fn(Option<usize>) + Send + Sync>,
    pub(crate) placeholder: String,
}

impl Combobox {
    /// Create a new Combobox with the given options.
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: None,
            on_change: Arc::new(|_| {}),
            placeholder: "Select...".to_string(),
        }
    }

    /// Set the pre-selected index.
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Set the callback invoked when the selection changes.
    pub fn on_change(mut self, callback: impl Fn(Option<usize>) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }

    /// Set the placeholder text shown when nothing is selected.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Filter options by fuzzy-matching against a query string.
    /// Returns indices into the original `options` vector, sorted by relevance.
    fn filtered_options(&self, options: &[String], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..options.len()).collect();
        }
        let mut matched: Vec<(usize, u32)> = options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                let score = fuzzy_match(opt, query);
                if score > 0 { Some((i, score)) } else { None }
            })
            .collect();
        matched.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
        matched.into_iter().map(|(i, _)| i).collect()
    }
}

// ── Fuzzy match (copied from command_palette.rs) ─────────────────────────────

/// Fuzzy-match a label against a query string.
/// Returns a score (higher = better match). A score of 0 means no match.
fn fuzzy_match(label: &str, query: &str) -> u32 {
    if query.is_empty() {
        return 1;
    }
    let label_lower = label.to_lowercase();
    let query_lower = query.to_lowercase();

    // Simple substring match first.
    if label_lower.contains(&query_lower) {
        if label_lower.starts_with(&query_lower) {
            return 100;
        }
        for word in label_lower.split(|c: char| !c.is_alphanumeric()) {
            if word.starts_with(&query_lower) {
                return 80;
            }
        }
        return 50;
    }

    // Character-by-character fuzzy match (order-preserved subsequence).
    let mut qi: usize = 0;
    let query_chars: Vec<char> = query_lower.chars().collect();
    for ch in label_lower.chars() {
        if qi < query_chars.len() && ch == query_chars[qi] {
            qi += 1;
        }
    }
    if qi == query_chars.len() {
        return 25;
    }
    0
}

// ── View impl ────────────────────────────────────────────────────────────────

impl View for Combobox {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Combobox");
        renderer.set_aria_role("combobox");

        // ── Sync builder-API state into system state on first render ──
        {
            let state = load_system_state();
            let sys_open: bool = state
                .get_component_state::<bool>(COMBO_OPEN_HASH)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(false);

            if state.get_component_state::<bool>(COMBO_OPEN_HASH).is_none() {
                update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(COMBO_OPEN_HASH, false);
                    s.set_component_state(COMBO_SELECTED_HASH, self.selected.unwrap_or(usize::MAX));
                    s.set_component_state(COMBO_SEARCH_HASH, String::new());
                    s
                });
            }

            // If closed, just render the trigger button and return early
            if !sys_open {
                self.render_trigger(renderer, rect);
                renderer.pop_vnode();
                return;
            }
        }

        // ── Read runtime state ──
        let (selected_index, search_text) = {
            let state = load_system_state();
            let sel = state
                .get_component_state::<usize>(COMBO_SELECTED_HASH)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(self.selected.unwrap_or(usize::MAX));
            let txt = state
                .get_component_state::<String>(COMBO_SEARCH_HASH)
                .and_then(|v| v.read().ok().map(|g| g.clone()))
                .unwrap_or_default();
            (sel, txt)
        };

        let filtered = self.filtered_options(&self.options, &search_text);
        let filtered_count = filtered.len();

        // Map selected_index from original options space to filtered list position
        let filtered_selected = if selected_index == usize::MAX || self.options.is_empty() {
            None
        } else {
            filtered.iter().position(|&idx| idx == selected_index)
        };
        let display_selected = filtered_selected.unwrap_or(0);

        // Clamp
        let display_selected = if filtered_count == 0 {
            0
        } else {
            display_selected.min(filtered_count - 1)
        };

        // ── Render trigger field ──
        self.render_trigger(renderer, rect);

        // ── Render dropdown ──
        let item_height = 32.0;
        let dropdown_padding = 4.0;
        let search_height = 32.0;
        let max_dropdown_height = 240.0;
        let dropdown_height =
            (filtered_count as f32 * item_height + search_height + dropdown_padding * 3.0)
                .min(max_dropdown_height);

        let dropdown_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height + 2.0,
            width: rect.width,
            height: dropdown_height,
        };

        // Dropdown background
        renderer.fill_rounded_rect(dropdown_rect, 6.0, theme::input_bg());
        renderer.stroke_rounded_rect(dropdown_rect, 6.0, [0.2, 0.3, 0.5, 0.8], 1.0);

        // ── Render search input ──
        let search_rect = Rect {
            x: dropdown_rect.x + dropdown_padding,
            y: dropdown_rect.y + dropdown_padding,
            width: dropdown_rect.width - dropdown_padding * 2.0,
            height: search_height,
        };
        renderer.fill_rounded_rect(search_rect, 4.0, theme::surface());
        renderer.stroke_rounded_rect(search_rect, 4.0, [0.0, 0.7, 1.0, 0.6], 1.0);

        let display_text = if search_text.is_empty() {
            "Type to search..."
        } else {
            &search_text
        };
        let text_color = if search_text.is_empty() {
            [0.4, 0.4, 0.5, 0.7]
        } else {
            theme::text()
        };
        renderer.draw_text(
            display_text,
            search_rect.x + 8.0,
            search_rect.y + 10.0,
            13.0,
            text_color,
        );

        // ── Render filtered options ──
        let list_start_y = search_rect.y + search_rect.height + dropdown_padding;
        let max_visible =
            ((dropdown_height - search_height - dropdown_padding * 3.0) / item_height) as usize;

        for (i, &original_idx) in filtered.iter().enumerate().take(max_visible) {
            let item_rect = Rect {
                x: dropdown_rect.x + dropdown_padding,
                y: list_start_y + i as f32 * item_height,
                width: dropdown_rect.width - dropdown_padding * 2.0,
                height: item_height,
            };

            let is_selected = i == display_selected;
            let bg = if is_selected {
                theme::border_strong()
            } else {
                theme::button_ghost_bg()
            };
            renderer.fill_rounded_rect(item_rect, 4.0, bg);

            if is_selected {
                renderer.stroke_rounded_rect(item_rect, 4.0, theme::accent(), 1.0);
            }

            let opt_text_color = if is_selected {
                theme::text()
            } else {
                theme::text_muted()
            };
            renderer.draw_text(
                &self.options[original_idx],
                item_rect.x + 8.0,
                item_rect.y + 9.0,
                13.0,
                opt_text_color,
            );
        }

        // ── Keyboard: ArrowUp / ArrowDown ──
        let filtered_count_kb = filtered_count;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowUp" => {
                            update_system_state(|s| {
                                let mut s = s.clone();
                                let current: usize = s
                                    .get_component_state::<usize>(COMBO_SELECTED_HASH)
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(usize::MAX);
                                // Find current position in filtered list
                                let pos = if current == usize::MAX {
                                    None
                                } else {
                                    filtered.iter().position(|&idx| idx == current)
                                };
                                let new_pos = match pos {
                                    Some(p) if p > 0 => p - 1,
                                    Some(_) if filtered_count_kb > 0 => filtered_count_kb - 1,
                                    None if filtered_count_kb > 0 => filtered_count_kb - 1,
                                    _ => 0,
                                };
                                if filtered_count_kb > 0 {
                                    let new_idx = filtered[new_pos];
                                    s.set_component_state(COMBO_SELECTED_HASH, new_idx);
                                }
                                s
                            });
                        }
                        "ArrowDown" => {
                            update_system_state(|s| {
                                let mut s = s.clone();
                                let current: usize = s
                                    .get_component_state::<usize>(COMBO_SELECTED_HASH)
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(usize::MAX);
                                let pos = if current == usize::MAX {
                                    None
                                } else {
                                    filtered.iter().position(|&idx| idx == current)
                                };
                                let new_pos = match pos {
                                    Some(p) if p + 1 < filtered_count_kb => p + 1,
                                    _ => 0,
                                };
                                if filtered_count_kb > 0 {
                                    let new_idx = filtered[new_pos];
                                    s.set_component_state(COMBO_SELECTED_HASH, new_idx);
                                }
                                s
                            });
                        }
                        _ => {}
                    }
                }
            }),
        );

        // ── Keyboard: Enter → confirm selection, close ──
        let on_change_enter = self.on_change.clone();
        let options_enter = self.options.clone();
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event
                    && (key == "Return" || key == "Enter")
                {
                    let state = load_system_state();
                    let open = state
                        .get_component_state::<bool>(COMBO_OPEN_HASH)
                        .and_then(|v| v.read().ok().map(|g| *g))
                        .unwrap_or(false);
                    if open {
                        let sel: usize = state
                            .get_component_state::<usize>(COMBO_SELECTED_HASH)
                            .and_then(|v| v.read().ok().map(|g| *g))
                            .unwrap_or(usize::MAX);
                        let sel_option = if sel < options_enter.len() {
                            Some(sel)
                        } else {
                            None
                        };
                        update_system_state(|s| {
                            let mut s = s.clone();
                            s.set_component_state(COMBO_OPEN_HASH, false);
                            s.set_component_state(COMBO_SEARCH_HASH, String::new());
                            s
                        });
                        (on_change_enter)(sel_option);
                    }
                }
            }),
        );

        // ── Keyboard: Escape → close ──
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event
                    && key == "Escape"
                {
                    update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(COMBO_OPEN_HASH, false);
                        s.set_component_state(COMBO_SEARCH_HASH, String::new());
                        s
                    });
                }
            }),
        );

        // ── Click on trigger → toggle open ──
        let on_change_click = self.on_change.clone();
        let options_click = self.options.clone();
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |_| {
                let state = load_system_state();
                let currently_open = state
                    .get_component_state::<bool>(COMBO_OPEN_HASH)
                    .and_then(|v| v.read().ok().map(|g| *g))
                    .unwrap_or(false);
                if currently_open {
                    // Close and confirm current selection
                    let sel: usize = state
                        .get_component_state::<usize>(COMBO_SELECTED_HASH)
                        .and_then(|v| v.read().ok().map(|g| *g))
                        .unwrap_or(usize::MAX);
                    let sel_option = if sel < options_click.len() {
                        Some(sel)
                    } else {
                        None
                    };
                    update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(COMBO_OPEN_HASH, false);
                        s.set_component_state(COMBO_SEARCH_HASH, String::new());
                        s
                    });
                    (on_change_click)(sel_option);
                } else {
                    update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(COMBO_OPEN_HASH, true);
                        s.set_component_state(COMBO_SEARCH_HASH, String::new());
                        s
                    });
                }
            }),
        );

        renderer.pop_vnode();
    }
}

impl Combobox {
    /// Render the trigger field (the always-visible part of the combobox).
    fn render_trigger(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let display_text = match self.selected {
            Some(idx) if idx < self.options.len() => &self.options[idx],
            _ => &self.placeholder,
        };
        let text_color = if self.selected.is_some() {
            theme::text()
        } else {
            [0.5, 0.5, 0.6, 0.7]
        };

        // Background
        renderer.fill_rounded_rect(rect, 6.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 6.0, [0.25, 0.3, 0.5, 0.8], 1.0);

        // Text
        renderer.draw_text(display_text, rect.x + 12.0, rect.y + 12.0, 14.0, text_color);

        // Chevron indicator
        let chevron = "▾";
        let (tw, _) = renderer.measure_text(chevron, 14.0);
        renderer.draw_text(
            chevron,
            rect.x + rect.width - tw - 12.0,
            rect.y + 12.0,
            14.0,
            [0.5, 0.5, 0.6, 0.8],
        );
    }
}

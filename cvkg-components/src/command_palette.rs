use crate::theme;
use cvkg_core::load_system_state;
use cvkg_core::update_system_state;
use cvkg_core::{
    Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

// System-state hash keys for the command palette
const SPOTLIGHT_OPEN_HASH: u64 = 0xD00_0001;
const SPOTLIGHT_SELECTED_HASH: u64 = 0xD00_0002;
const SPOTLIGHT_SEARCH_HASH: u64 = 0xD00_0003;

/// A command palette for searching and executing commands.
pub struct MimirSpotlight {
    /// The full list of registered commands (never modified at render time).
    pub(crate) all_commands: Vec<PaletteCommand>,
    /// The search text used for runtime filtering.
    pub(crate) search_text: String,
    /// Pre-selected index (used when opening via builder API).
    pub(crate) selected_index: usize,
    /// Pre-set open state (used when opening via builder API).
    pub(crate) is_open: bool,
}

pub struct PaletteCommand {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

/// Fuzzy-match a command label against a query string.
/// Returns a score (higher = better match). A score of 0 means no match.
fn fuzzy_match(label: &str, query: &str) -> u32 {
    if query.is_empty() {
        return 1;
    }
    let label_lower = label.to_lowercase();
    let query_lower = query.to_lowercase();

    // Simple substring match first — always works and is predictable.
    if label_lower.contains(&query_lower) {
        // Bonus for matching at start
        if label_lower.starts_with(&query_lower) {
            return 100;
        }
        // Check word-boundary match (e.g. "save" matches "Save File")
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
        // All characters found in order — scored lower than substring
        return 25;
    }
    0
}

impl MimirSpotlight {
    /// Creates a new MimirSpotlight instance.
    pub fn new() -> Self {
        Self {
            all_commands: Vec::new(),
            search_text: String::new(),
            selected_index: 0,
            is_open: false,
        }
    }

    pub fn command<F>(mut self, label: &str, shortcut: Option<&str>, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.all_commands.push(PaletteCommand {
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            action: Arc::new(action),
        });
        self
    }

    /// Set the search text for runtime filtering.
    /// Unlike the previous build-time filtering, this stores the query and
    /// the filtering happens during `render()`.
    pub fn search(mut self, text: &str) -> Self {
        self.search_text = text.to_lowercase();
        self.selected_index = 0;
        self
    }

    /// Open the palette via the builder API.
    pub fn open(mut self) -> Self {
        self.is_open = true;
        self
    }

    /// Close the palette via the builder API.
    pub fn close(mut self) -> Self {
        self.is_open = false;
        self
    }

    /// Pre-select a command index via the builder API.
    pub fn select(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.all_commands.len().saturating_sub(1));
        self
    }

    /// Returns the filtered commands based on the given search text.
    fn filtered_commands(&self, search_text: &str) -> Vec<&PaletteCommand> {
        if search_text.is_empty() {
            return self.all_commands.iter().collect();
        }
        let mut matched: Vec<(&PaletteCommand, u32)> = self
            .all_commands
            .iter()
            .filter_map(|cmd| {
                let score = fuzzy_match(&cmd.label, search_text);
                if score > 0 { Some((cmd, score)) } else { None }
            })
            .collect();
        matched.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
        matched.into_iter().map(|(cmd, _)| cmd).collect()
    }
}

impl View for MimirSpotlight {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MimirSpotlight");

        // ── Sync builder-API state into system state on first render ──
        {
            let state = load_system_state();
            let sys_open: bool = state
                .get_component_state::<bool>(SPOTLIGHT_OPEN_HASH)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(self.is_open);

            // Seed system state from builder values on first render.
            if state
                .get_component_state::<bool>(SPOTLIGHT_OPEN_HASH)
                .is_none()
            {
                update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(SPOTLIGHT_OPEN_HASH, self.is_open);
                    s.set_component_state(SPOTLIGHT_SELECTED_HASH, self.selected_index as u64);
                    s.set_component_state(SPOTLIGHT_SEARCH_HASH, self.search_text.clone());
                    s
                });
            }

            if !sys_open {
                renderer.pop_vnode();
                return;
            }
        }

        // ── Read runtime state from system state ──
        let (selected_index, search_text) = {
            let state = load_system_state();
            let sel = state
                .get_component_state::<u64>(SPOTLIGHT_SELECTED_HASH)
                .and_then(|v| v.read().ok().map(|g| *g as usize))
                .unwrap_or(self.selected_index);
            let txt = state
                .get_component_state::<String>(SPOTLIGHT_SEARCH_HASH)
                .and_then(|v| v.read().ok().map(|g| g.clone()))
                .unwrap_or_else(|| self.search_text.clone());
            (sel, txt)
        };

        // ── Filter commands at render time based on search_text ──
        let filtered = self.filtered_commands(&search_text);
        let filtered_count = filtered.len();

        // Clamp selected index to filtered list
        let selected_index = if filtered_count == 0 {
            0
        } else {
            selected_index.min(filtered_count - 1)
        };

        // ── Register keyboard: ArrowUp / ArrowDown ──
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key } = event {
                    match key.as_str() {
                        "ArrowUp" => {
                            update_system_state(|s| {
                                let mut s = s.clone();
                                let current: u64 = s
                                    .get_component_state::<u64>(SPOTLIGHT_SELECTED_HASH)
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(0);
                                let next = if current == 0 {
                                    filtered_count.saturating_sub(1) as u64
                                } else {
                                    current - 1
                                };
                                s.set_component_state(SPOTLIGHT_SELECTED_HASH, next);
                                s
                            });
                        }
                        "ArrowDown" => {
                            update_system_state(|s| {
                                let mut s = s.clone();
                                let current: u64 = s
                                    .get_component_state::<u64>(SPOTLIGHT_SELECTED_HASH)
                                    .and_then(|v| v.read().ok().map(|g| *g))
                                    .unwrap_or(0);
                                let next = if filtered_count == 0 {
                                    0
                                } else {
                                    (current + 1) % filtered_count as u64
                                };
                                s.set_component_state(SPOTLIGHT_SELECTED_HASH, next);
                                s
                            });
                        }
                        _ => {}
                    }
                }
            }),
        );

        // ── Register keyboard: Escape → close ──
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key } = event
                    && key == "Escape"
                {
                    update_system_state(|s| {
                        let mut s = s.clone();
                        s.set_component_state(SPOTLIGHT_OPEN_HASH, false);
                        s.set_component_state(SPOTLIGHT_SELECTED_HASH, 0u64);
                        s.set_component_state(SPOTLIGHT_SEARCH_HASH, String::new());
                        s
                    });
                }
            }),
        );

        // ── Register keyboard: Enter → execute selected command ──
        // Capture the action Arc of the currently-selected command so the
        // handler can invoke it without borrowing `self`.
        if let Some(cmd) = filtered.get(selected_index) {
            let action = cmd.action.clone();
            renderer.register_handler(
                "keydown",
                Arc::new(move |event| {
                    if let Event::KeyDown { key } = event
                        && (key == "Return" || key == "Enter")
                    {
                        let state = load_system_state();
                        let open = state
                            .get_component_state::<bool>(SPOTLIGHT_OPEN_HASH)
                            .and_then(|v| v.read().ok().map(|g| *g))
                            .unwrap_or(false);
                        if open {
                            (action)();
                        }
                    }
                }),
            );
        }

        // ── Render overlay background ──
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.6]);

        // ── Calculate palette dimensions ──
        let palette_width = 480.0;
        let item_height = 36.0;
        let top_padding = 56.0;
        let palette_height = (filtered_count as f32 * item_height).min(rect.height - 40.0);
        let palette_x = rect.x + (rect.width - palette_width) / 2.0;
        let palette_y = rect.y + (rect.height - palette_height) / 2.0;

        let palette_rect = Rect {
            x: palette_x,
            y: palette_y,
            width: palette_width,
            height: palette_height,
        };

        // ── Render palette background ──
        renderer.fill_rounded_rect(palette_rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(palette_rect, 8.0, theme::accent(), 1.0);

        // ── Render search input ──
        let search_rect = Rect {
            x: palette_rect.x + 12.0,
            y: palette_rect.y + 8.0,
            width: palette_width - 24.0,
            height: 36.0,
        };
        renderer.fill_rounded_rect(search_rect, 6.0, theme::input_bg());

        // Focus ring around the search input to indicate it is active/focused.
        let outline_rect = Rect {
            x: search_rect.x - 2.0,
            y: search_rect.y - 2.0,
            width: search_rect.width + 4.0,
            height: search_rect.height + 4.0,
        };
        renderer.stroke_rounded_rect(outline_rect, 6.0, theme::accent(), 2.0);

        renderer.draw_text(
            &format!("> {}", search_text),
            search_rect.x + 8.0,
            search_rect.y + 12.0,
            14.0,
            theme::text_muted(),
        );

        // ── Render commands ──
        let start_y = palette_rect.y + top_padding;
        for (i, cmd) in filtered.iter().enumerate() {
            let cmd_rect = Rect {
                x: palette_rect.x + 12.0,
                y: start_y + i as f32 * item_height,
                width: palette_width - 24.0,
                height: item_height,
            };

            let is_selected = i == selected_index;
            let bg = if is_selected {
                theme::border_strong()
            } else {
                theme::surface()
            };
            renderer.fill_rounded_rect(cmd_rect, 4.0, bg);

            // Selection highlight stroke (focus ring for selected item)
            if is_selected {
                renderer.stroke_rounded_rect(cmd_rect, 4.0, theme::accent(), 1.0);
            }

            let text_color = if is_selected {
                theme::text()
            } else {
                theme::text_muted()
            };
            renderer.draw_text(
                &cmd.label,
                cmd_rect.x + 8.0,
                cmd_rect.y + 12.0,
                13.0,
                text_color,
            );

            if let Some(ref shortcut) = cmd.shortcut {
                let shortcut_x = cmd_rect.x + cmd_rect.width - 80.0;
                renderer.draw_text(
                    shortcut,
                    shortcut_x,
                    cmd_rect.y + 12.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        }

        renderer.pop_vnode();
    }
}

impl LayoutView for MimirSpotlight {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let height = (self.all_commands.len() as f32 * 36.0).max(100.0);
        Size {
            width: 480.0,
            height: height + 56.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// BifrostLauncher provides a quick search and launch interface, representing the bridge between worlds.
pub struct BifrostLauncher {
    pub(crate) items: Vec<QuickItem>,
    pub(crate) search: String,
}

pub struct QuickItem {
    pub label: String,
    pub icon: String,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl BifrostLauncher {
    /// Creates a new BifrostLauncher instance.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            search: String::new(),
        }
    }

    pub fn item<F>(mut self, label: &str, icon: &str, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.items.push(QuickItem {
            label: label.to_string(),
            icon: icon.to_string(),
            action: Arc::new(action),
        });
        self
    }

    pub fn search(mut self, text: &str) -> Self {
        self.search = text.to_lowercase();
        self
    }
}

impl View for BifrostLauncher {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BifrostLauncher");
        let filtered: Vec<_> = self
            .items
            .iter()
            .filter(|item| item.label.to_lowercase().contains(&self.search))
            .collect();

        if filtered.is_empty() {
            renderer.draw_text(
                "No results",
                rect.x + 12.0,
                rect.y + 12.0,
                14.0,
                theme::text_muted(),
            );
            renderer.pop_vnode();
            return;
        }

        let item_height = 40.0;
        for (i, item) in filtered.iter().enumerate() {
            let item_rect = Rect {
                x: rect.x + 8.0,
                y: rect.y + i as f32 * item_height,
                width: rect.width - 16.0,
                height: item_height,
            };

            renderer.fill_rounded_rect(item_rect, 4.0, theme::input_bg());
            renderer.draw_text(
                &item.icon,
                item_rect.x + 8.0,
                item_rect.y + 12.0,
                16.0,
                theme::info(),
            );
            renderer.draw_text(
                &item.label,
                item_rect.x + 32.0,
                item_rect.y + 14.0,
                13.0,
                theme::text(),
            );
        }
        renderer.pop_vnode();
    }
}

impl LayoutView for BifrostLauncher {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let filtered_count = self
            .items
            .iter()
            .filter(|item| item.label.to_lowercase().contains(&self.search))
            .count();
        Size {
            width: 300.0,
            height: (filtered_count as f32 * 40.0).max(60.0),
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

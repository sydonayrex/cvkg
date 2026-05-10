use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never};
use std::sync::Arc;

/// A command palette for searching and executing commands.
/// 
/// INSPIRED BY: Mantine (Spotlight) and Mimir's wisdom.
pub struct MimirSpotlight {
    pub(crate) commands: Vec<Command>,
    pub(crate) search_text: String,
    pub(crate) selected_index: usize,
    pub(crate) is_open: bool,
}

pub struct Command {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl MimirSpotlight {
    /// Creates a new MimirSpotlight instance.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            search_text: String::new(),
            selected_index: 0,
            is_open: false,
        }
    }

    pub fn command<F>(mut self, label: &str, shortcut: Option<&str>, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.commands.push(Command {
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            action: Arc::new(action),
        });
        self
    }

    pub fn search(mut self, text: &str) -> Self {
        self.search_text = text.to_lowercase();
        self.selected_index = 0;
        // Filter commands
        if !self.search_text.is_empty() {
            self.commands.retain(|cmd| cmd.label.to_lowercase().contains(&self.search_text));
        }
        self
    }

    pub fn open(mut self) -> Self {
        self.is_open = true;
        self
    }

    pub fn close(mut self) -> Self {
        self.is_open = false;
        self
    }

    pub fn select(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.commands.len().saturating_sub(1));
        self
    }
}

impl View for MimirSpotlight {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MimirSpotlight");
        if !self.is_open || self.commands.is_empty() {
            return;
        }

        // Render overlay background
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.6]);

        // Calculate palette dimensions
        let palette_width = 480.0;
        let palette_height = (self.commands.len() as f32 * 36.0).min(rect.height - 40.0);
        let palette_x = rect.x + (rect.width - palette_width) / 2.0;
        let palette_y = rect.y + (rect.height - palette_height) / 2.0;

        let palette_rect = Rect {
            x: palette_x,
            y: palette_y,
            width: palette_width,
            height: palette_height,
        };

        // Render palette background
        renderer.fill_rounded_rect(palette_rect, 8.0, [0.08, 0.08, 0.15, 1.0]);
        renderer.stroke_rounded_rect(palette_rect, 8.0, [0.3, 0.5, 0.8, 1.0], 1.0);

        // Render search input
        let search_rect = Rect {
            x: palette_rect.x + 12.0,
            y: palette_rect.y + 8.0,
            width: palette_width - 24.0,
            height: 36.0,
        };
        renderer.fill_rounded_rect(search_rect, 6.0, [0.06, 0.06, 0.1, 1.0]);
        renderer.draw_text(
            &format!("> {}", self.search_text),
            search_rect.x + 8.0,
            search_rect.y + 12.0,
            14.0,
            [0.6, 0.6, 0.7, 1.0],
        );

        // Render commands
        let start_y = palette_rect.y + 56.0;
        for (i, cmd) in self.commands.iter().enumerate() {
            let cmd_rect = Rect {
                x: palette_rect.x + 12.0,
                y: start_y + i as f32 * 36.0,
                width: palette_width - 24.0,
                height: 36.0,
            };

            let is_selected = i == self.selected_index;
            let bg = if is_selected { [0.12, 0.2, 0.3, 1.0] } else { [0.04, 0.04, 0.08, 1.0] };
            renderer.fill_rounded_rect(cmd_rect, 4.0, bg);

            if is_selected {
                renderer.stroke_rounded_rect(cmd_rect, 4.0, [0.0, 0.8, 1.0, 1.0], 1.0);
            }

            let text_color = if is_selected { [1.0, 1.0, 1.0, 1.0] } else { [0.7, 0.7, 0.8, 1.0] };
            renderer.draw_text(&cmd.label, cmd_rect.x + 8.0, cmd_rect.y + 12.0, 13.0, text_color);

            if let Some(ref shortcut) = cmd.shortcut {
                let shortcut_x = cmd_rect.x + cmd_rect.width - 80.0;
                renderer.draw_text(shortcut, shortcut_x, cmd_rect.y + 12.0, 11.0, [0.4, 0.4, 0.5, 1.0]);
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
        let height = (self.commands.len() as f32 * 36.0).max(100.0);
        Size { width: 480.0, height: height + 56.0 }
    }

    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
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
        Self { items: Vec::new(), search: String::new() }
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
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BifrostLauncher");
        let filtered: Vec<_> = self.items.iter().filter(|item| {
            item.label.to_lowercase().contains(&self.search)
        }).collect();

        if filtered.is_empty() {
            renderer.draw_text("No results", rect.x + 12.0, rect.y + 12.0, 14.0, [0.5, 0.5, 0.6, 1.0]);
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

            renderer.fill_rounded_rect(item_rect, 4.0, [0.06, 0.06, 0.1, 1.0]);
            renderer.draw_text(&item.icon, item_rect.x + 8.0, item_rect.y + 12.0, 16.0, [0.6, 0.8, 1.0, 1.0]);
            renderer.draw_text(&item.label, item_rect.x + 32.0, item_rect.y + 14.0, 13.0, [0.8, 0.8, 0.9, 1.0]);
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
        let filtered_count = self.items.iter().filter(|item| item.label.to_lowercase().contains(&self.search)).count();
        Size { width: 300.0, height: (filtered_count as f32 * 40.0).max(60.0) }
    }

    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}

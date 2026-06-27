use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Command palette component for quick actions and navigation.
pub struct Command {
    pub(crate) placeholder: String,
    pub(crate) items: Vec<CommandItem>,
    pub(crate) search_text: String,
}

pub struct CommandItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub on_select: Arc<dyn Fn() + Send + Sync>,
}

impl Command {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            items: Vec::new(),
            search_text: String::new(),
        }
    }

    pub fn item(
        mut self,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        on_select: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.items.push(CommandItem {
            label: label.into(),
            shortcut: shortcut.map(|s| s.into()),
            on_select: Arc::new(on_select),
        });
        self
    }
}

impl View for Command {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Main container (centered floating palette)
        let palette_w = (rect.width * 0.9).min(500.0);
        let palette_h = (rect.height * 0.7).min(400.0);
        let palette_rect = Rect {
            x: rect.x + (rect.width - palette_w) / 2.0,
            y: rect.y + (rect.height - palette_h) / 2.0,
            width: palette_w,
            height: palette_h,
        };

        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(palette_rect, 25.0, 1.5, 0.9);
        }
        renderer.fill_rounded_rect(palette_rect, 12.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(
            palette_rect,
            12.0,
            theme::with_alpha(theme::accent(), 0.6),
            1.5,
        );

        // Search input area
        let search_h = 45.0;
        let search_rect = Rect {
            x: palette_rect.x,
            y: palette_rect.y,
            width: palette_rect.width,
            height: search_h,
        };
        let search_display = if self.search_text.is_empty() {
            self.placeholder.as_str()
        } else {
            self.search_text.as_str()
        };
        let search_color = if self.search_text.is_empty() {
            theme::text_muted()
        } else {
            theme::text()
        };

        renderer.draw_text(
            search_display,
            search_rect.x + 16.0,
            search_rect.y + 14.0,
            16.0,
            search_color,
        );
        renderer.draw_line(
            search_rect.x,
            search_rect.y + search_h,
            search_rect.x + search_rect.width,
            search_rect.y + search_h,
            theme::border_strong(),
            1.0,
        );

        // Items list
        let mut current_y = palette_rect.y + search_h + 8.0;
        for item in &self.items {
            let item_h = 36.0;
            let item_rect = Rect {
                x: palette_rect.x + 8.0,
                y: current_y,
                width: palette_rect.width - 16.0,
                height: item_h,
            };

            renderer.push_vnode(item_rect, "CommandItem");
            // Hover state simulation (just a subtle highlight for now)
            renderer.fill_rounded_rect(item_rect, 4.0, theme::hover());

            renderer.draw_text(
                &item.label,
                item_rect.x + 12.0,
                item_rect.y + 10.0,
                14.0,
                theme::text(),
            );
            if let Some(shortcut) = &item.shortcut {
                let (sw, _) = renderer.measure_text(shortcut, 12.0);
                renderer.draw_text(
                    shortcut,
                    item_rect.x + item_rect.width - sw - 12.0,
                    item_rect.y + 11.0,
                    12.0,
                    theme::text_muted(),
                );
            }

            let on_select = item.on_select.clone();
            renderer.register_handler(
                "pointerclick",
                Arc::new(move |_| {
                    on_select();
                }),
            );
            renderer.pop_vnode();

            current_y += item_h + 4.0;
        }
    }
}

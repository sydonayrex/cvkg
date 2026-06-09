//! GaldraMenu — Right-click context menu with glass styling.
//! Named after Galdr, the spoken form of Norse magic.

use cvkg_core::{Rect, Renderer, View, Never};
use std::sync::Arc;

pub struct GaldraMenu {
    pub items: Vec<GaldraMenuItem>,
    pub anchor: MenuAnchor,
}

pub enum MenuAnchor {
    Pointer,
    Rect(Rect),
}

pub enum GaldraMenuItem {
    Action {
        label: String,
        shortcut: Option<String>,
        enabled: bool,
        action: Arc<dyn Fn() + Send + Sync>,
    },
    Submenu {
        label: String,
        items: Vec<GaldraMenuItem>,
    },
    Separator,
}

impl GaldraMenu {
    pub fn new(items: Vec<GaldraMenuItem>) -> Self {
        Self {
            items,
            anchor: MenuAnchor::Pointer,
        }
    }

    pub fn render_at(&self, renderer: &mut dyn Renderer, x: f32, y: f32) {
        let item_height = 28.0;
        let menu_width = 200.0;
        let menu_height = self.items.len() as f32 * item_height + 8.0;

        let menu_rect = Rect { x, y, width: menu_width, height: menu_height };

        // Glass background
        renderer.bifrost(menu_rect, 20.0, 1.1, 0.7);
        renderer.fill_rounded_rect(menu_rect, 8.0, [0.08, 0.08, 0.1, 0.92]);

        // Render items
        let mut iy = menu_rect.y + 4.0;
        for item in &self.items {
            match item {
                GaldraMenuItem::Action { label, shortcut, enabled, .. } => {
                    let item_rect = Rect {
                        x: menu_rect.x + 4.0, y: iy,
                        width: menu_rect.width - 8.0, height: item_height,
                    };
                    let color = if *enabled { [0.9, 0.9, 0.92, 1.0] } else { [0.5, 0.5, 0.55, 0.5] };
                    renderer.draw_text(label, item_rect.x + 8.0, item_rect.y + 7.0, 12.0, color);
                    if let Some(sc) = shortcut {
                        let sw = renderer.measure_text(sc, 11.0).0;
                        renderer.draw_text(sc, item_rect.x + item_rect.width - sw - 8.0, item_rect.y + 7.0, 11.0, [0.6, 0.6, 0.65, 0.8]);
                    }
                }
                GaldraMenuItem::Separator => {
                    renderer.draw_line(
                        menu_rect.x + 8.0, iy + 12.0,
                        menu_rect.x + menu_rect.width - 8.0, iy + 12.0,
                        [0.2, 0.2, 0.25, 0.5], 1.0,
                    );
                }
                _ => {}
            }
            iy += item_height;
        }
    }
}
//! NornirBar — The application menu bar.
//! Named after the Nornir (Urd, Verdandi, Skuld), the three fates.

use cvkg_core::{MenuBar, MenuItem, Rect, Renderer, View, Never};

/// The application menu bar. Renders at the top of the window with
/// glass background, horizontal menu items, and cascading submenus.
pub struct NornirBar {
    pub menu_bar: MenuBar,
    pub floating: bool,
    open_menu: Option<usize>,
    pointer_pos: [f32; 2],
}

impl NornirBar {
    pub fn new(menu_bar: MenuBar) -> Self {
        Self {
            menu_bar,
            floating: false,
            open_menu: None,
            pointer_pos: [0.0, 0.0],
        }
    }

    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }

    pub fn handle_pointer_move(&mut self, x: f32, y: f32) {
        self.pointer_pos = [x, y];
    }

    pub fn toggle_menu(&mut self, index: usize) {
        if self.open_menu == Some(index) {
            self.open_menu = None;
        } else {
            self.open_menu = Some(index);
        }
    }

    pub fn close_menu(&mut self) {
        self.open_menu = None;
    }
}

impl View for NornirBar {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background (full width, 28pt tall)
        renderer.bifrost(rect, 25.0, 1.2, 0.65);

        // Render each top-level menu header
        let mut x = rect.x + 8.0;
        for (i, item) in self.menu_bar.items.iter().enumerate() {
            match item {
                MenuItem::Submenu { label, items: sub_items } => {
                    let label_w = renderer.measure_text(label, 13.0).0;
                    let item_rect = Rect {
                        x, y: rect.y,
                        width: label_w + 16.0,
                        height: 28.0,
                    };

                    // Highlight if open
                    if self.open_menu == Some(i) {
                        renderer.fill_rounded_rect(item_rect, 4.0, [1.0, 1.0, 1.0, 0.12]);
                    }

                    renderer.draw_text(label, x + 8.0, rect.y + 8.0, 13.0, [0.9, 0.9, 0.92, 1.0]);

                    // If open, render submenu as floating glass panel
                    if self.open_menu == Some(i) {
                        render_submenu(renderer, sub_items, item_rect);
                    }

                    x += label_w + 16.0;
                }
                MenuItem::Action { label, .. } => {
                    let label_w = renderer.measure_text(label, 13.0).0;
                    renderer.draw_text(label, x + 8.0, rect.y + 8.0, 13.0, [0.9, 0.9, 0.92, 1.0]);
                    x += label_w + 16.0;
                }
                MenuItem::Separator => {
                    renderer.draw_line(x, rect.y + 6.0, x, rect.y + 22.0, [0.3, 0.3, 0.35, 0.5], 1.0);
                    x += 12.0;
                }
            }
        }
    }
}

fn render_submenu(renderer: &mut dyn Renderer, items: &[MenuItem], anchor: Rect) {
    let item_height = 26.0;
    let menu_width = 180.0;
    let menu_height = items.len() as f32 * item_height + 8.0;
    let menu_rect = Rect {
        x: anchor.x,
        y: anchor.y + anchor.height + 2.0,
        width: menu_width,
        height: menu_height,
    };

    // Glass panel
    renderer.bifrost(menu_rect, 20.0, 1.1, 0.7);
    renderer.fill_rounded_rect(menu_rect, 8.0, [0.06, 0.06, 0.08, 0.92]);

    // Render submenu items
    let mut iy = menu_rect.y + 4.0;
    for item in items {
        match item {
            MenuItem::Action { label, enabled, .. } => {
                let color = if *enabled {
                    [0.9, 0.9, 0.92, 1.0]
                } else {
                    [0.5, 0.5, 0.55, 0.5]
                };
                renderer.draw_text(label, menu_rect.x + 8.0, iy + 5.0, 12.0, color);
            }
            MenuItem::Separator => {
                renderer.draw_line(
                    menu_rect.x + 6.0, iy + 12.0,
                    menu_rect.x + menu_rect.width - 6.0, iy + 12.0,
                    [0.2, 0.2, 0.25, 0.5], 1.0,
                );
            }
            _ => {}
        }
        iy += item_height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nornir_bar_new() {
        let menu_bar = MenuBar::new();
        let bar = NornirBar::new(menu_bar);
        assert!(!bar.floating);
        assert!(bar.open_menu.is_none());
    }

    #[test]
    fn test_nornir_bar_floating() {
        let bar = NornirBar::new(MenuBar::new()).floating(true);
        assert!(bar.floating);
    }

    #[test]
    fn test_nornir_bar_toggle_menu() {
        let mut bar = NornirBar::new(MenuBar::new());
        bar.toggle_menu(0);
        assert_eq!(bar.open_menu, Some(0));
        bar.toggle_menu(0);
        assert_eq!(bar.open_menu, None);
    }

    #[test]
    fn test_nornir_bar_close_menu() {
        let mut bar = NornirBar::new(MenuBar::new());
        bar.toggle_menu(1);
        assert_eq!(bar.open_menu, Some(1));
        bar.close_menu();
        assert_eq!(bar.open_menu, None);
    }

    #[test]
    fn test_nornir_bar_pointer_move() {
        let mut bar = NornirBar::new(MenuBar::new());
        bar.handle_pointer_move(100.0, 200.0);
        assert_eq!(bar.pointer_pos, [100.0, 200.0]);
    }
}
//! ContextMenu component for right-click / long-press context menus.
//!
//! Renders a floating card with a list of menu items, each supporting
//! labels, keyboard shortcuts, disabled state, and optional sub-menu children.

use crate::theme;
use crate::{FONT_BASE, FONT_SM, RADIUS_MD, RADIUS_SM, SPACE_SM, SPACE_XS};
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// A single item within a context menu.
///
/// Items can be regular clickable entries, disabled entries, or
/// parent entries that expand into a sub-menu on hover.
#[derive(Clone)]
pub struct ContextMenuItem {
    /// The display label for this item.
    pub label: String,
    /// Optional keyboard shortcut text (e.g. "⌘C", "Ctrl+Shift+P").
    pub shortcut: Option<String>,
    /// Whether this item is disabled (greyed out, non-interactive).
    pub disabled: bool,
    /// Callback invoked when this item is clicked.
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional sub-menu children. When present, hovering this item
    /// reveals the nested menu to the right.
    pub children: Vec<ContextMenuItem>,
}

impl ContextMenuItem {
    /// Create a new context menu item with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            disabled: false,
            on_click: None,
            children: Vec::new(),
        }
    }

    /// Set the keyboard shortcut text displayed on the right side.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set whether this item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the click callback.
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(callback));
        self
    }

    /// Add a child item (creates a sub-menu).
    pub fn child(mut self, child: ContextMenuItem) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple child items at once.
    pub fn children(mut self, children: Vec<ContextMenuItem>) -> Self {
        self.children = children;
        self
    }

    /// Returns true if this item has sub-menu children.
    pub fn has_submenu(&self) -> bool {
        !self.children.is_empty()
    }
}

/// ContextMenu - A floating card with a list of menu items.
///
/// Renders as a positioned floating panel (card style) containing
/// a vertical list of items. Each item shows a label, optional
/// shortcut text, and an optional sub-menu arrow.
///
/// # Example
/// ```
/// use cvkg_components::context_menu::{ContextMenu, ContextMenuItem};
/// let items = vec![
///     ContextMenuItem::new("Copy").shortcut("⌘C"),
///     ContextMenuItem::new("Paste").shortcut("⌘V"),
///     ContextMenuItem::new("Cut").shortcut("⌘X").disabled(true),
/// ];
/// let menu = ContextMenu::new(items).position(100.0, 200.0);
/// ```
#[derive(Clone)]
pub struct ContextMenu {
    /// The menu items to display.
    items: Vec<ContextMenuItem>,
    /// X position of the menu.
    pos_x: f32,
    /// Y position of the menu.
    pos_y: f32,
    /// Whether the menu is currently visible.
    is_open: bool,
}

impl ContextMenu {
    /// Create a new ContextMenu with the given items.
    pub fn new(items: Vec<ContextMenuItem>) -> Self {
        Self {
            items,
            pos_x: 0.0,
            pos_y: 0.0,
            is_open: true,
        }
    }

    /// Set the position of the menu.
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.pos_x = x;
        self.pos_y = y;
        self
    }

    /// Set whether the menu is visible.
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Compute the height for a single item.
    fn item_height(item: &ContextMenuItem) -> f32 {
        if item.has_submenu() { 36.0 } else { 32.0 }
    }

    /// Compute total menu height.
    fn total_height(&self) -> f32 {
        self.items.iter().map(Self::item_height).sum::<f32>() + SPACE_SM * 2.0
    }

    /// Compute the widest item width.
    fn max_item_width(&self, renderer: &mut dyn Renderer) -> f32 {
        let mut max_w = 120.0f32;
        for item in &self.items {
            let (lw, _) = renderer.measure_text(&item.label, FONT_SM);
            let sw = item
                .shortcut
                .as_ref()
                .map(|s| renderer.measure_text(s, FONT_SM).0 + SPACE_SM)
                .unwrap_or(0.0);
            let arrow_w = if item.has_submenu() { 16.0 } else { 0.0 };
            let total = lw + sw + arrow_w + SPACE_SM * 4.0;
            max_w = max_w.max(total);
        }
        max_w
    }
}

impl View for ContextMenu {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        if !self.is_open || self.items.is_empty() {
            return;
        }

        let menu_w = self.max_item_width(renderer).max(160.0);
        let menu_h = self.total_height();

        let menu_rect = Rect {
            x: self.pos_x,
            y: self.pos_y,
            width: menu_w,
            height: menu_h,
        };

        renderer.push_vnode(menu_rect, "ContextMenu");

        // Glass background
        renderer.bifrost(menu_rect, 15.0, 1.5, 0.95);
        renderer.fill_rounded_rect(menu_rect, RADIUS_MD, [0.06, 0.06, 0.1, 0.92]);
        renderer.stroke_rounded_rect(menu_rect, RADIUS_MD, theme::border(), 1.0);

        // Render each item
        let mut y_offset = SPACE_SM;
        for item in &self.items {
            let item_h = Self::item_height(item);
            let item_rect = Rect {
                x: menu_rect.x + SPACE_XS,
                y: menu_rect.y + y_offset,
                width: menu_rect.width - SPACE_XS * 2.0,
                height: item_h,
            };

            renderer.push_vnode(item_rect, "ContextMenuItem");

            let text_color = if item.disabled {
                theme::disabled_text()
            } else {
                theme::text()
            };

            // Hover highlight background
            if !item.disabled {
                renderer.fill_rounded_rect(item_rect, RADIUS_SM, theme::hover());
            }

            // Label
            let label_y = item_rect.y + (item_h - FONT_SM) / 2.0 - FONT_SM * 0.5;
            renderer.draw_text(
                &item.label,
                item_rect.x + SPACE_SM,
                label_y,
                FONT_SM,
                text_color,
            );

            // Shortcut text (right-aligned)
            if let Some(ref sc) = item.shortcut {
                let (sw, _) = renderer.measure_text(sc, FONT_SM);
                let sc_y = item_rect.y + (item_h - FONT_SM) / 2.0 - FONT_SM * 0.5;
                renderer.draw_text(
                    sc,
                    item_rect.x + item_rect.width - sw - SPACE_SM,
                    sc_y,
                    FONT_SM,
                    theme::text_muted(),
                );
            }

            // Sub-menu arrow
            if item.has_submenu() {
                let arrow_x = item_rect.x + item_rect.width - 12.0;
                let arrow_y = item_rect.y + (item_h - FONT_BASE) / 2.0 - FONT_BASE * 0.5;
                renderer.draw_text("›", arrow_x, arrow_y, FONT_BASE, text_color);
            }

            // Click handler
            if !item.disabled
                && let Some(ref cb) = item.on_click
            {
                let cb = cb.clone();
                let h_closure = Arc::new(move |_| {
                    (cb)();
                });
                renderer.register_handler("pointerdown", h_closure.clone());
                renderer.register_handler("pointerclick", h_closure);
            }

            renderer.pop_vnode();
            y_offset += item_h;
        }

        // Click-outside handler to close
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |event: Event| {
                if let Event::PointerDown { x, y, .. } = event
                    && !menu_rect.contains(x, y)
                {
                    // Close the menu — in a real app this would toggle state
                }
            }),
        );

        renderer.pop_vnode();
    }
}

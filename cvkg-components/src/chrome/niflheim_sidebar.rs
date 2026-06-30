//! NiflheimSidebar -- Glass chrome wrapper for sidebar panels.
//! Named after Niflheim, the realm of ice and mist.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// A sidebar item in the source list.
#[derive(Clone)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<u32>,
    pub children: Vec<SidebarItem>,
    pub is_expanded: bool,
}

impl SidebarItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            children: Vec::new(),
            is_expanded: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn children(mut self, children: Vec<SidebarItem>) -> Self {
        self.children = children;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }
}

/// Glass chrome configuration for sidebar panels.
pub struct NiflheimSidebar {
    pub items: Vec<SidebarItem>,
    pub selected_id: Option<String>,
    pub vibrancy: SidebarVibrancy,
    pub source_list_style: bool,
    pub on_select: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

pub enum SidebarVibrancy {
    Translucent,
    Standard,
    Heavy,
}

impl NiflheimSidebar {
    pub fn new(items: Vec<SidebarItem>) -> Self {
        Self {
            items,
            selected_id: None,
            vibrancy: SidebarVibrancy::Standard,
            source_list_style: true,
            on_select: None,
        }
    }

    pub fn vibrancy(mut self, v: SidebarVibrancy) -> Self {
        self.vibrancy = v;
        self
    }

    pub fn on_select(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Render the glass background for a sidebar region.
    pub fn render_background(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let (blur, opacity) = match self.vibrancy {
            SidebarVibrancy::Translucent => (15.0, 0.4),
            SidebarVibrancy::Standard => (25.0, 0.65),
            SidebarVibrancy::Heavy => (35.0, 0.85),
        };

        // Glass background
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, blur, 1.2, opacity);
        }
        renderer.fill_rounded_rect(rect, 0.0, theme::surface());

        // Separator line on trailing edge
        let sep_x = rect.x + rect.width - 0.5;
        renderer.draw_line(
            sep_x,
            rect.y,
            sep_x,
            rect.y + rect.height,
            theme::border(),
            1.0,
        );
    }

    /// Render a source-list row with glass highlight.
    pub fn render_row(
        &self,
        renderer: &mut dyn Renderer,
        rect: Rect,
        item: &SidebarItem,
        is_selected: bool,
        is_hovered: bool,
        depth: usize,
    ) {
        let bg = if is_selected {
            theme::active_color()
        } else if is_hovered {
            theme::hover()
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        if bg[3] > 0.0 {
            renderer.fill_rounded_rect(rect, 6.0, bg);
        }

        let indent = 12.0 + depth as f32 * 16.0;

        // Icon
        if let Some(ref icon) = item.icon {
            renderer.draw_text_raw(
                icon,
                rect.x + indent,
                rect.y + 6.0,
                14.0,
                theme::text_muted(),
            );
        }

        // Label
        let text_color = theme::text();
        renderer.draw_text_raw(
            &item.label,
            rect.x + indent + 20.0,
            rect.y + 5.0,
            12.0,
            text_color,
        );

        // Badge
        if let Some(count) = item.badge {
            let badge_rect = Rect {
                x: rect.x + rect.width - 24.0,
                y: rect.y + 4.0,
                width: 18.0,
                height: 18.0,
            };
            renderer.fill_ellipse(badge_rect, theme::accent());
            let text = if count > 99 {
                "99+".to_string()
            } else {
                count.to_string()
            };
            let tw = renderer.measure_text(&text, 9.0).0;
            renderer.draw_text_raw(
                &text,
                badge_rect.x + (18.0 - tw) / 2.0,
                badge_rect.y + 4.0,
                9.0,
                theme::text(),
            );
        }
    }
}

impl View for NiflheimSidebar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background
        self.render_background(renderer, rect);

        // Render each item
        let mut y = rect.y + 4.0;
        let item_height = 28.0;
        for item in &self.items {
            let item_rect = Rect {
                x: rect.x,
                y,
                width: rect.width,
                height: item_height,
            };
            let is_selected = self.selected_id.as_ref() == Some(&item.id);
            self.render_row(renderer, item_rect, item, is_selected, false, 0);
            y += item_height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_item_new() {
        let item = SidebarItem::new("inbox", "Inbox");
        assert_eq!(item.id, "inbox");
        assert_eq!(item.label, "Inbox");
        assert_eq!(item.icon, None);
        assert_eq!(item.badge, None);
        assert!(!item.is_expanded);
    }

    #[test]
    fn test_sidebar_item_with_badge() {
        let item = SidebarItem::new("inbox", "Inbox").badge(5);
        assert_eq!(item.badge, Some(5));
    }

    #[test]
    fn test_sidebar_item_with_children() {
        let item =
            SidebarItem::new("folder", "Folder").children(vec![SidebarItem::new("sub", "Sub")]);
        assert_eq!(item.children.len(), 1);
    }

    #[test]
    fn test_sidebar_new() {
        let sidebar = NiflheimSidebar::new(vec![
            SidebarItem::new("inbox", "Inbox"),
            SidebarItem::new("sent", "Sent"),
        ]);
        assert_eq!(sidebar.items.len(), 2);
        assert!(sidebar.selected_id.is_none());
        assert!(matches!(sidebar.vibrancy, SidebarVibrancy::Standard));
    }

    #[test]
    fn test_sidebar_vibrancy() {
        let sidebar = NiflheimSidebar::new(vec![]).vibrancy(SidebarVibrancy::Heavy);
        assert!(matches!(sidebar.vibrancy, SidebarVibrancy::Heavy));
    }

    #[test]
    fn test_sidebar_on_select() {
        let sidebar = NiflheimSidebar::new(vec![]).on_select(|id: &str| {
            let _ = id;
        });
        assert!(sidebar.on_select.is_some());
    }
}

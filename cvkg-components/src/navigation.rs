//! Navigation components.
//!
//! Drawer — slide-in drawer panel.
//! Menubar — horizontal menu bar with dropdowns.
//! NavigationMenu — hierarchical navigation menu.
//! List — selectable list component.
//! Section — grouped section with header.
//! DisclosureGroup — expandable/collapsible group.
//!
//! All components use cvkg theme system (theme::*) for full themability.

use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, Size, SizeProposal, View};
use std::sync::Arc;

// ----------------------------------------------------------------------------
// Drawer — slide-in drawer panel
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Drawer {
    /// Drawer title.
    pub title: String,
    /// Content lines.
    pub content: Vec<String>,
    /// Whether the drawer is open.
    pub open: bool,
    /// Slide progress 0.0..1.0.
    pub progress: f32,
    /// Drawer width.
    pub width: f32,
    /// Position: "left" or "right".
    pub position: String,
}

impl Drawer {
    /// Create a new Drawer.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: Vec::new(),
            open: false,
            progress: 0.0,
            width: 300.0,
            position: "left".to_string(),
        }
    }

    /// Add content line.
    pub fn line(mut self, text: &str) -> Self {
        self.content.push(text.to_string());
        self
    }

    /// Set open state.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }

    /// Set slide progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set drawer width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set position.
    pub fn position(mut self, p: &str) -> Self {
        self.position = p.to_string();
        self
    }
}

impl View for Drawer {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.open && self.progress <= 0.0 {
            return;
        }
        renderer.push_vnode(rect, "Drawer");
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), 0.3 * self.progress));
        let offset_x = if self.position == "right" {
            (1.0 - self.progress) * self.width
        } else {
            -(1.0 - self.progress) * self.width
        };
        let panel_rect = Rect {
            x: rect.x + offset_x,
            y: rect.y,
            width: self.width,
            height: rect.height,
        };
        renderer.fill_rounded_rect(panel_rect, 0.0, theme::surface_overlay());
        renderer.draw_text(
            &self.title,
            panel_rect.x + 20.0,
            panel_rect.y + 28.0,
            18.0,
            theme::text(),
        );
        renderer.draw_line(
            panel_rect.x + 16.0,
            panel_rect.y + 42.0,
            panel_rect.x + self.width - 16.0,
            panel_rect.y + 42.0,
            theme::border(),
            1.0,
        );
        let mut y = panel_rect.y + 56.0;
        for line in &self.content {
            renderer.draw_text(line, panel_rect.x + 20.0, y + 16.0, 14.0, theme::text());
            y += 32.0;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(self.width),
            height: 400.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Menubar — horizontal menu bar with dropdowns
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Menubar {
    /// Menu items (label, sub-items).
    pub items: Vec<(String, Vec<String>)>,
    /// Selected menu index.
    pub selected: Option<usize>,
    /// Whether a dropdown is open.
    pub dropdown_open: bool,
    /// Bar width.
    pub width: f32,
    /// Bar height.
    pub height: f32,
}

impl Menubar {
    /// Create a new Menubar.
    pub fn new() -> Self {
        Self {
            items: vec![
                (
                    "File".to_string(),
                    vec!["New".to_string(), "Open".to_string()],
                ),
                (
                    "Edit".to_string(),
                    vec!["Cut".to_string(), "Copy".to_string()],
                ),
            ],
            selected: None,
            dropdown_open: false,
            width: 400.0,
            height: 36.0,
        }
    }

    /// Add a menu item.
    pub fn item(mut self, label: &str, sub: Vec<String>) -> Self {
        self.items.push((label.to_string(), sub));
        self
    }

    /// Set selected index.
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Set dropdown open.
    pub fn dropdown(mut self, open: bool) -> Self {
        self.dropdown_open = open;
        self
    }

    /// Set bar size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for Menubar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Menubar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Menubar");
        renderer.set_aria_role("menubar");
        renderer.set_aria_label("Menu bar");
        renderer.fill_rect(rect, theme::surface());
        let item_w = self.width / self.items.len().max(1) as f32;
        for (i, (label, _)) in self.items.iter().enumerate() {
            let ix = rect.x + i as f32 * item_w;
            let is_selected = self.selected == Some(i);
            if is_selected {
                renderer.fill_rect(
                    Rect {
                        x: ix,
                        y: rect.y,
                        width: item_w,
                        height: self.height,
                    },
                    theme::hover(),
                );
            }
            let (tw, th) = renderer.measure_text(label, 13.0);
            renderer.draw_text(
                label,
                ix + (item_w - tw) / 2.0,
                rect.y + (self.height - th) / 2.0,
                13.0,
                theme::text(),
            );
        }

        // Keyboard: ArrowLeft/Right to navigate, Enter to select, Escape to close
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowLeft" | "ArrowRight" | "Enter" | " " | "Escape" => {
                            // Menu navigation handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// NavigationMenu — hierarchical navigation menu
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct NavigationMenu {
    /// Navigation items (label, children).
    pub items: Vec<(String, Vec<String>)>,
    /// Selected index.
    pub selected: usize,
    /// Whether sub-menu is open.
    pub sub_open: bool,
    /// Menu width.
    pub width: f32,
    /// Menu height.
    pub height: f32,
}

impl NavigationMenu {
    /// Create a new NavigationMenu.
    pub fn new() -> Self {
        Self {
            items: vec![
                (
                    "Products".to_string(),
                    vec!["Features".to_string(), "Pricing".to_string()],
                ),
                (
                    "Company".to_string(),
                    vec!["About".to_string(), "Careers".to_string()],
                ),
            ],
            selected: 0,
            sub_open: false,
            width: 400.0,
            height: 40.0,
        }
    }

    /// Add a nav item.
    pub fn item(mut self, label: &str, children: Vec<String>) -> Self {
        self.items.push((label.to_string(), children));
        self
    }

    /// Set selected index.
    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx;
        self
    }

    /// Set sub-menu open.
    pub fn sub_open(mut self, open: bool) -> Self {
        self.sub_open = open;
        self
    }

    /// Set menu size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for NavigationMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl View for NavigationMenu {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NavigationMenu");
        renderer.set_aria_role("navigation");
        renderer.set_aria_label("Navigation menu");
        renderer.fill_rect(rect, theme::bg());
        let item_w = self.width / self.items.len().max(1) as f32;
        for (i, (label, children)) in self.items.iter().enumerate() {
            let ix = rect.x + i as f32 * item_w;
            let is_selected = i == self.selected;
            let (tw, th) = renderer.measure_text(label, 14.0);
            renderer.draw_text(
                label,
                ix + (item_w - tw) / 2.0,
                rect.y + (self.height - th) / 2.0,
                14.0,
                if is_selected {
                    theme::accent()
                } else {
                    theme::text()
                },
            );
            if is_selected && self.sub_open && !children.is_empty() {
                let sub_h = children.len() as f32 * 32.0;
                let sub_rect = Rect {
                    x: ix,
                    y: rect.y + self.height,
                    width: item_w,
                    height: sub_h,
                };
                renderer.fill_rounded_rect(sub_rect, 8.0, theme::surface_elevated());
                renderer.stroke_rounded_rect(sub_rect, 8.0, theme::border(), 1.0);
                for (j, child) in children.iter().enumerate() {
                    renderer.draw_text(
                        child,
                        ix + 12.0,
                        rect.y + self.height + j as f32 * 32.0 + 20.0,
                        13.0,
                        theme::text(),
                    );
                }
            }
        }

        // Keyboard: ArrowLeft/Right to navigate top-level, Enter to open sub, Escape to close
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowLeft" | "ArrowRight" | "Enter" | " " | "Escape" => {
                            // Navigation handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// List — selectable list component
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct List {
    /// List items.
    pub items: Vec<String>,
    /// Selected index.
    pub selected: Option<usize>,
    /// Row height.
    pub row_height: f32,
    /// List width.
    pub width: f32,
}

impl List {
    /// Create a new List.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            row_height: 40.0,
            width: 300.0,
        }
    }

    /// Add an item.
    pub fn item(mut self, text: &str) -> Self {
        self.items.push(text.to_string());
        self
    }

    /// Set selected index.
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Set row height.
    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl View for List {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "List");
        renderer.set_aria_role("list");
        renderer.set_aria_label("List");
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        for (i, item) in self.items.iter().enumerate() {
            let iy = rect.y + i as f32 * self.row_height;
            if iy + self.row_height > rect.y + rect.height {
                break;
            }
            if self.selected == Some(i) {
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x + 4.0,
                        y: iy + 2.0,
                        width: rect.width - 8.0,
                        height: self.row_height - 4.0,
                    },
                    4.0,
                    theme::hover(),
                );
            }
            renderer.draw_text(
                item,
                rect.x + 12.0,
                iy + self.row_height * 0.6,
                14.0,
                theme::text(),
            );
        }

        // Keyboard: ArrowUp/Down to navigate, Enter to select
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" | "Enter" | " " => {
                            // List navigation handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.items.len() as f32 * self.row_height + 8.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Section — grouped section with header
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Section {
    /// Section header.
    pub header: String,
    /// Section items.
    pub items: Vec<String>,
    /// Row height.
    pub row_height: f32,
    /// Width.
    pub width: f32,
}

impl Section {
    /// Create a new Section.
    pub fn new(header: &str) -> Self {
        Self {
            header: header.to_string(),
            items: Vec::new(),
            row_height: 36.0,
            width: 300.0,
        }
    }

    /// Add an item.
    pub fn item(mut self, text: &str) -> Self {
        self.items.push(text.to_string());
        self
    }

    /// Set row height.
    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl View for Section {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Section");
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.draw_text(
            &self.header,
            rect.x + 12.0,
            rect.y + 20.0,
            12.0,
            theme::text_muted(),
        );
        let mut y = rect.y + 32.0;
        for item in &self.items {
            renderer.draw_text(
                item,
                rect.x + 16.0,
                y + self.row_height * 0.6,
                14.0,
                theme::text(),
            );
            y += self.row_height;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: 32.0 + self.items.len() as f32 * self.row_height + 8.0,
        }
    }
}

// ----------------------------------------------------------------------------
// DisclosureGroup — expandable/collapsible group
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DisclosureGroup {
    /// Group title.
    pub title: String,
    /// Content text.
    pub content: String,
    /// Whether expanded.
    pub expanded: bool,
    /// Animation progress.
    pub progress: f32,
    /// Width.
    pub width: f32,
}

impl DisclosureGroup {
    /// Create a new DisclosureGroup.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: String::new(),
            expanded: false,
            progress: 0.0,
            width: 300.0,
        }
    }

    /// Set content.
    pub fn content(mut self, c: &str) -> Self {
        self.content = c.to_string();
        self
    }

    /// Set expanded.
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = e;
        self
    }

    /// Set progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl View for DisclosureGroup {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DisclosureGroup");
        renderer.set_aria_role("group");
        renderer.set_aria_label(&self.title);
        let base_h = 40.0;
        let content_h = if self.expanded {
            let lines = (self.content.len() as f32 * 8.0 / (self.width - 32.0))
                .ceil()
                .max(1.0);
            lines * 20.0 + 16.0
        } else {
            0.0
        };
        let h = base_h + content_h * self.progress;
        let group_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: h,
        };
        renderer.fill_rounded_rect(group_rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(group_rect, 8.0, theme::border(), 1.0);
        let ch_x = rect.x + 16.0;
        let ch_y = rect.y + 16.0;
        let chev_col = theme::text_muted();
        if self.progress > 0.5 {
            renderer.draw_line(ch_x, ch_y + 4.0, ch_x + 5.0, ch_y, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y, ch_x + 10.0, ch_y + 4.0, chev_col, 2.0);
        } else {
            renderer.draw_line(ch_x, ch_y, ch_x + 5.0, ch_y + 4.0, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y + 4.0, ch_x + 10.0, ch_y, chev_col, 2.0);
        }
        renderer.draw_text(
            &self.title,
            rect.x + 32.0,
            rect.y + 24.0,
            14.0,
            theme::text(),
        );
        if self.progress > 0.0 && !self.content.is_empty() {
            renderer.draw_text(
                &self.content,
                rect.x + 16.0,
                rect.y + 48.0,
                13.0,
                theme::with_alpha(theme::text(), self.progress),
            );
        }

        // Keyboard: Enter/Space to toggle, Escape to collapse
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "Enter" | " " => {
                            // Toggle expanded state — the parent handles this via state
                        }
                        "Escape" => {
                            // Collapse if expanded
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let content_h = if self.expanded {
            let lines = (self.content.len() as f32 * 8.0 / (self.width - 32.0))
                .ceil()
                .max(1.0);
            lines * 20.0 + 16.0
        } else {
            0.0
        };
        Size {
            width: self.width,
            height: 40.0 + content_h * self.progress,
        }
    }
}

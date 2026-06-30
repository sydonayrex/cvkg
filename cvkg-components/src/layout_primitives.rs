//! Layout primitive components.
//!
//! AspectRatio -- fixed ratio container.
//! ZStack -- overlapping stack layout.
//! LazyVGrid / LazyHGrid -- lazy grid layouts.
//! LazyHStack -- lazy horizontal stack.
//! Resizable -- draggable resizable panel.
//! Group / GroupBox -- labeled grouping containers.
//! Separator -- visual divider.
//!
//! All components use cvkg theme system (theme::*) for full themability.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// ============================================================
// AspectRatio
// ============================================================

pub struct AspectRatio {
    /// Width / height ratio (e.g. 16.0/9.0).
    pub ratio: f32,
    /// Container width.
    pub width: f32,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Border radius.
    pub radius: f32,
}

impl AspectRatio {
    /// Create a new AspectRatio.
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: ratio.max(0.1),
            width: 300.0,
            bg_color: theme::surface_elevated(),
            radius: 8.0,
        }
    }

    /// Set the width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the border radius.
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
}

impl View for AspectRatio {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AspectRatio");
        let h = self.width / self.ratio;
        let ar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: h,
        };
        renderer.fill_rounded_rect(ar_rect, self.radius, self.bg_color);
        renderer.stroke_rounded_rect(ar_rect, self.radius, theme::border(), 1.0);
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.width / self.ratio,
        }
    }
}

// ----------------------------------------------------------------------------
// Drawer -- slide-in drawer panel
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// ZStack
// ============================================================

pub struct ZStack {
    /// Stack width.
    pub width: f32,
    /// Stack height.
    pub height: f32,
    /// Background color.
    pub bg_color: [f32; 4],
}

impl ZStack {
    /// Create a new ZStack.
    pub fn new() -> Self {
        Self {
            width: 300.0,
            height: 200.0,
            bg_color: theme::surface(),
        }
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ZStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ZStack");
        renderer.fill_rounded_rect(rect, 8.0, self.bg_color);
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
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
// LazyVGrid -- lazy vertical grid
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// LazyVGrid
// ============================================================

pub struct LazyVGrid {
    /// Number of columns.
    pub cols: usize,
    /// Cell labels.
    pub cells: Vec<String>,
    /// Gap.
    pub gap: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl LazyVGrid {
    /// Create a new LazyVGrid.
    pub fn new(cols: usize) -> Self {
        Self {
            cols,
            cells: Vec::new(),
            gap: 8.0,
            width: 400.0,
            height: 300.0,
        }
    }

    /// Add a cell.
    pub fn cell(mut self, label: &str) -> Self {
        self.cells.push(label.to_string());
        self
    }

    /// Set gap.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for LazyVGrid {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "LazyVGrid");
        let rows = (self.cells.len() + self.cols - 1) / self.cols.max(1);
        let cell_w = (rect.width - (self.cols - 1) as f32 * self.gap) / self.cols as f32;
        let cell_h = (rect.height - (rows - 1) as f32 * self.gap) / rows as f32;
        for (i, label) in self.cells.iter().enumerate() {
            let col = i % self.cols;
            let row = i / self.cols;
            let cx = rect.x + col as f32 * (cell_w + self.gap);
            let cy = rect.y + row as f32 * (cell_h + self.gap);
            let cell_rect = Rect {
                x: cx,
                y: cy,
                width: cell_w,
                height: cell_h,
            };
            renderer.fill_rounded_rect(cell_rect, 8.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(cell_rect, 8.0, theme::border(), 1.0);
            let (tw, th) = renderer.measure_text(label, 13.0);
            renderer.draw_text_raw(
                label,
                cx + (cell_w - tw) / 2.0,
                cy + (cell_h - th) / 2.0,
                13.0,
                theme::text(),
            );
        }
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
// LazyHGrid -- lazy horizontal grid
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// LazyHGrid
// ============================================================

pub struct LazyHGrid {
    /// Number of rows.
    pub rows: usize,
    /// Cell labels.
    pub cells: Vec<String>,
    /// Gap.
    pub gap: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl LazyHGrid {
    /// Create a new LazyHGrid.
    pub fn new(rows: usize) -> Self {
        Self {
            rows,
            cells: Vec::new(),
            gap: 8.0,
            width: 400.0,
            height: 300.0,
        }
    }

    /// Add a cell.
    pub fn cell(mut self, label: &str) -> Self {
        self.cells.push(label.to_string());
        self
    }

    /// Set gap.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for LazyHGrid {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "LazyHGrid");
        let cols = (self.cells.len() + self.rows - 1) / self.rows.max(1);
        let cell_w = (rect.width - (cols - 1) as f32 * self.gap) / cols as f32;
        let cell_h = (rect.height - (self.rows - 1) as f32 * self.gap) / self.rows as f32;
        for (i, label) in self.cells.iter().enumerate() {
            let row = i % self.rows;
            let col = i / self.rows;
            let cx = rect.x + col as f32 * (cell_w + self.gap);
            let cy = rect.y + row as f32 * (cell_h + self.gap);
            let cell_rect = Rect {
                x: cx,
                y: cy,
                width: cell_w,
                height: cell_h,
            };
            renderer.fill_rounded_rect(cell_rect, 8.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(cell_rect, 8.0, theme::border(), 1.0);
            let (tw, th) = renderer.measure_text(label, 13.0);
            renderer.draw_text_raw(
                label,
                cx + (cell_w - tw) / 2.0,
                cy + (cell_h - th) / 2.0,
                13.0,
                theme::text(),
            );
        }
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
// LazyHStack -- lazy horizontal stack
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// LazyHStack
// ============================================================

pub struct LazyHStack {
    /// Item labels.
    pub items: Vec<String>,
    /// Item width.
    pub item_width: f32,
    /// Gap.
    pub gap: f32,
    /// Height.
    pub height: f32,
}

impl LazyHStack {
    /// Create a new LazyHStack.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            item_width: 100.0,
            gap: 8.0,
            height: 40.0,
        }
    }

    /// Add an item.
    pub fn item(mut self, text: &str) -> Self {
        self.items.push(text.to_string());
        self
    }

    /// Set item width.
    pub fn item_width(mut self, w: f32) -> Self {
        self.item_width = w;
        self
    }

    /// Set gap.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    /// Set height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl Default for LazyHStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for LazyHStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "LazyHStack");
        let mut x = rect.x;
        for item in &self.items {
            let item_rect = Rect {
                x,
                y: rect.y,
                width: self.item_width,
                height: self.height,
            };
            renderer.fill_rounded_rect(item_rect, 8.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(item_rect, 8.0, theme::border(), 1.0);
            let (tw, th) = renderer.measure_text(item, 13.0);
            renderer.draw_text_raw(
                item,
                x + (self.item_width - tw) / 2.0,
                rect.y + (self.height - th) / 2.0,
                13.0,
                theme::text(),
            );
            x += self.item_width + self.gap;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let total_w = self.items.len() as f32 * self.item_width
            + (self.items.len().saturating_sub(1)) as f32 * self.gap;
        Size {
            width: total_w,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// FullScreenCover -- full screen overlay
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// Resizable
// ============================================================

pub struct Resizable {
    /// Current width.
    pub width: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Panel height.
    pub height: f32,
    /// Whether being resized.
    pub is_resizing: bool,
}

impl Resizable {
    /// Create a new Resizable.
    pub fn new(width: f32) -> Self {
        Self {
            width,
            min_width: 100.0,
            max_width: 600.0,
            height: 200.0,
            is_resizing: false,
        }
    }

    /// Set min width.
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }

    /// Set max width.
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }

    /// Set height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set resizing state.
    pub fn resizing(mut self, r: bool) -> Self {
        self.is_resizing = r;
        self
    }
}

impl View for Resizable {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Resizable");
        let panel_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: self.height,
        };
        renderer.fill_rounded_rect(panel_rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(panel_rect, 8.0, theme::border(), 1.0);
        // Resize handle
        let handle_color = if self.is_resizing {
            theme::accent()
        } else {
            theme::text_dim()
        };
        let hx = rect.x + self.width - 3.0;
        for i in 0..3 {
            renderer.draw_line(
                hx,
                rect.y + self.height / 2.0 - 8.0 + i as f32 * 8.0,
                hx + 2.0,
                rect.y + self.height / 2.0 - 8.0 + i as f32 * 8.0,
                handle_color,
                1.0,
            );
        }
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
// Label -- form label component
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// Group
// ============================================================

pub struct Group {
    /// Group label.
    pub label: String,
    /// Padding.
    pub padding: f32,
    /// Border radius.
    pub radius: f32,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl Group {
    /// Create a new Group.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            padding: 16.0,
            radius: 8.0,
            bg_color: theme::surface(),
            width: 300.0,
            height: 100.0,
        }
    }

    /// Set padding.
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }

    /// Set border radius.
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }

    /// Set background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for Group {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Group");
        renderer.fill_rounded_rect(rect, self.radius, self.bg_color);
        renderer.stroke_rounded_rect(rect, self.radius, theme::border(), 1.0);
        renderer.draw_text_raw(
            &self.label,
            rect.x + self.padding,
            rect.y + self.padding + 14.0,
            14.0,
            theme::text(),
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
// GroupBox -- labeled group box
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// GroupBox
// ============================================================

pub struct GroupBox {
    /// Box title.
    pub title: String,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl GroupBox {
    /// Create a new GroupBox.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            width: 300.0,
            height: 120.0,
        }
    }

    /// Set size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for GroupBox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GroupBox");
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        // Title background
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x + 12.0,
                y: rect.y - 8.0,
                width: self.title.len() as f32 * 8.0 + 16.0,
                height: 16.0,
            },
            4.0,
            theme::bg(),
        );
        renderer.draw_text_raw(
            &self.title,
            rect.x + 20.0,
            rect.y + 4.0,
            12.0,
            theme::text(),
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
// DisclosureGroup -- expandable/collapsible group
// ----------------------------------------------------------------------------

#[derive(Clone)]

// ============================================================
// Separator
// ============================================================

pub struct Separator {
    /// Orientation: true = horizontal, false = vertical.
    pub horizontal: bool,
    /// Length.
    pub length: f32,
    /// Thickness.
    pub thickness: f32,
    /// Color.
    pub color: [f32; 4],
}

impl Separator {
    /// Create a new horizontal Separator.
    pub fn new() -> Self {
        Self {
            horizontal: true,
            length: 200.0,
            thickness: 1.0,
            color: theme::border(),
        }
    }

    /// Set orientation.
    pub fn horizontal(mut self, h: bool) -> Self {
        self.horizontal = h;
        self
    }

    /// Set length.
    pub fn length(mut self, l: f32) -> Self {
        self.length = l;
        self
    }

    /// Set thickness.
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t;
        self
    }

    /// Set color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Separator {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Separator");
        if self.horizontal {
            renderer.draw_line(
                rect.x,
                rect.y,
                rect.x + self.length,
                rect.y,
                self.color,
                self.thickness,
            );
        } else {
            renderer.draw_line(
                rect.x,
                rect.y,
                rect.x,
                rect.y + self.length,
                self.color,
                self.thickness,
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        if self.horizontal {
            Size {
                width: self.length,
                height: self.thickness,
            }
        } else {
            Size {
                width: self.thickness,
                height: self.length,
            }
        }
    }
}

//! Loader components: Loader, MultiStepLoader.
//! Navigation: FloatingNavbar, NavbarMenu.
//! Carousel: Carousel, Marquee.
//! Layout: BentoGrid.
//! All components use cvkg theme system (theme::*) for full themability.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// =============================================================================
// LOADERS
// =============================================================================

/// Loader variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderVariant {
    Spinner,
    Dots,
    Pulse,
    Bar,
}

/// Loader -- animated loading spinner with variants.
#[derive(Clone)]
pub struct Loader {
    /// Size of the loader.
    pub size: f32,
    /// Animation time.
    pub time: f32,
    /// Visual variant.
    pub variant: LoaderVariant,
    /// Color override (None = theme accent).
    pub color: Option<[f32; 4]>,
    /// Label text below the loader.
    pub label: String,
}

impl Loader {
    /// Create a new Loader.
    pub fn new() -> Self {
        Self {
            size: 32.0,
            time: 0.0,
            variant: LoaderVariant::Spinner,
            color: None,
            label: String::new(),
        }
    }

    /// Set the size.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set the animation time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }

    /// Set the variant.
    pub fn variant(mut self, v: LoaderVariant) -> Self {
        self.variant = v;
        self
    }

    /// Set the color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = Some(c);
        self
    }

    /// Set the label.
    pub fn label(mut self, l: &str) -> Self {
        self.label = l.to_string();
        self
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Loader {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Loader");
        let col = self.color.unwrap_or_else(theme::accent);
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + self.size / 2.0;
        match self.variant {
            LoaderVariant::Spinner => {
                for i in 0..8 {
                    let angle = self.time * 6.0 + i as f32 * 0.785;
                    let px = cx + angle.cos() * self.size * 0.35;
                    let py = cy + angle.sin() * self.size * 0.35;
                    let alpha = 0.2 + (i as f32 * 0.1);
                    renderer.fill_ellipse(
                        Rect {
                            x: px - 3.0,
                            y: py - 3.0,
                            width: 6.0,
                            height: 6.0,
                        },
                        [col[0], col[1], col[2], alpha],
                    );
                }
            }
            LoaderVariant::Dots => {
                for i in 0..3 {
                    let offset = ((self.time * 3.0 + i as f32 * 0.33) % 1.0) * 10.0 - 5.0;
                    renderer.fill_ellipse(
                        Rect {
                            x: cx - 12.0 + i as f32 * 12.0 - 3.0,
                            y: cy + offset - 3.0,
                            width: 6.0,
                            height: 6.0,
                        },
                        col,
                    );
                }
            }
            LoaderVariant::Pulse => {
                let r = self.size * 0.3 * (1.0 + (self.time * 4.0).sin() * 0.3);
                renderer.fill_ellipse(
                    Rect {
                        x: cx - r,
                        y: cy - r,
                        width: r * 2.0,
                        height: r * 2.0,
                    },
                    [col[0], col[1], col[2], 0.3],
                );
                renderer.fill_ellipse(
                    Rect {
                        x: cx - r * 0.6,
                        y: cy - r * 0.6,
                        width: r * 1.2,
                        height: r * 1.2,
                    },
                    col,
                );
            }
            LoaderVariant::Bar => {
                let bar_w = self.size * 2.0;
                let progress = self.time % 1.0;
                renderer.fill_rounded_rect(
                    Rect {
                        x: cx - bar_w / 2.0,
                        y: cy - 2.0,
                        width: bar_w,
                        height: 4.0,
                    },
                    2.0,
                    theme::surface_elevated(),
                );
                let fill_w = bar_w * progress;
                renderer.fill_rounded_rect(
                    Rect {
                        x: cx - bar_w / 2.0,
                        y: cy - 2.0,
                        width: fill_w,
                        height: 4.0,
                    },
                    2.0,
                    col,
                );
            }
        }
        // Label
        if !self.label.is_empty() {
            renderer.draw_text_raw(
                &self.label,
                cx - self.label.len() as f32 * 4.0,
                rect.y + self.size + 8.0,
                12.0,
                theme::text_muted(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let label_h = if self.label.is_empty() { 0.0 } else { 20.0 };
        Size {
            width: self.size.max(100.0),
            height: self.size + label_h,
        }
    }
}

// ----------------------------------------------------------------------------
// MultiStepLoader -- multi-step progress loader with labels
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct MultiStepLoader {
    /// Step labels.
    pub steps: Vec<String>,
    /// Current step index.
    pub current: usize,
    /// Animation time.
    pub time: f32,
}

impl MultiStepLoader {
    /// Create a new MultiStepLoader.
    pub fn new(steps: Vec<String>) -> Self {
        Self {
            steps,
            current: 0,
            time: 0.0,
        }
    }

    /// Set current step.
    pub fn current(mut self, c: usize) -> Self {
        self.current = c;
        self
    }

    /// Set animation time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }
}

impl View for MultiStepLoader {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MultiStepLoader");
        let step_w = rect.width / self.steps.len().max(1) as f32;
        for (i, label) in self.steps.iter().enumerate() {
            let x = rect.x + i as f32 * step_w;
            let dot_x = x + step_w / 2.0;
            let dot_y = rect.y + 20.0;
            let color = if i < self.current {
                theme::success()
            } else if i == self.current {
                theme::accent()
            } else {
                theme::surface_elevated()
            };
            // Connecting line
            if i > 0 {
                renderer.draw_line(
                    x - step_w / 2.0,
                    dot_y,
                    dot_x,
                    dot_y,
                    if i <= self.current {
                        theme::accent()
                    } else {
                        theme::border()
                    },
                    2.0,
                );
            }
            // Step dot
            let dot_r = if i == self.current { 8.0 } else { 6.0 };
            renderer.fill_ellipse(
                Rect {
                    x: dot_x - dot_r,
                    y: dot_y - dot_r,
                    width: dot_r * 2.0,
                    height: dot_r * 2.0,
                },
                color,
            );
            // Step number
            let num_str = format!("{}", i + 1);
            renderer.draw_text_raw(
                &num_str,
                dot_x - 3.0,
                dot_y + 5.0,
                10.0,
                if i <= self.current {
                    theme::bg()
                } else {
                    theme::text_muted()
                },
            );
            // Label
            renderer.draw_text_raw(label, x + 8.0, rect.y + 40.0, 11.0, theme::text());
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: 56.0,
        }
    }
}

// =============================================================================
// NAVIGATION
// =============================================================================

/// FloatingNavbar -- floating navigation bar with blur backdrop.
#[derive(Clone)]
pub struct FloatingNavbar {
    /// Navigation item labels.
    pub items: Vec<String>,
    /// Selected index.
    pub selected: usize,
    /// Whether to show glass backdrop.
    pub glass: bool,
    /// Bar width.
    pub width: f32,
    /// Bar height.
    pub height: f32,
}

impl FloatingNavbar {
    /// Create a new FloatingNavbar.
    pub fn new() -> Self {
        Self {
            items: vec![
                "Home".to_string(),
                "Explore".to_string(),
                "Settings".to_string(),
            ],
            selected: 0,
            glass: true,
            width: 400.0,
            height: 56.0,
        }
    }

    /// Set the nav items.
    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    /// Set the selected index.
    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx;
        self
    }

    /// Enable/disable glass backdrop.
    pub fn glass(mut self, g: bool) -> Self {
        self.glass = g;
        self
    }

    /// Set the bar size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for FloatingNavbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for FloatingNavbar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FloatingNavbar");
        let bar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: self.height,
        };
        if self.glass {
            renderer.fill_glass_rect(bar_rect, 16.0, 20.0);
        } else {
            renderer.fill_rounded_rect(bar_rect, 16.0, theme::surface_overlay());
        }
        renderer.stroke_rounded_rect(bar_rect, 16.0, theme::border(), 1.0);
        // Nav items
        let item_w = self.width / self.items.len().max(1) as f32;
        for (i, label) in self.items.iter().enumerate() {
            let ix = rect.x + i as f32 * item_w;
            let is_selected = i == self.selected;
            if is_selected {
                renderer.fill_rounded_rect(
                    Rect {
                        x: ix + 8.0,
                        y: rect.y + 8.0,
                        width: item_w - 16.0,
                        height: self.height - 16.0,
                    },
                    10.0,
                    theme::hover(),
                );
            }
            let (tw, th) = renderer.measure_text(label, 14.0);
            renderer.draw_text_raw(
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
// NavbarMenu -- animated navbar with dropdown menus
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct NavbarMenu {
    /// Top-level menu labels.
    pub items: Vec<String>,
    /// Selected index.
    pub selected: usize,
    /// Whether a dropdown is open at the selected index.
    pub dropdown_open: bool,
    /// Bar width.
    pub width: f32,
    /// Bar height.
    pub height: f32,
}

impl NavbarMenu {
    /// Create a new NavbarMenu.
    pub fn new() -> Self {
        Self {
            items: vec!["File".to_string(), "Edit".to_string(), "View".to_string()],
            selected: 0,
            dropdown_open: false,
            width: 400.0,
            height: 40.0,
        }
    }

    /// Set the menu items.
    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    /// Set the selected index.
    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx;
        self
    }

    /// Set dropdown open state.
    pub fn dropdown(mut self, open: bool) -> Self {
        self.dropdown_open = open;
        self
    }

    /// Set the bar size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for NavbarMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl View for NavbarMenu {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NavbarMenu");
        renderer.fill_rect(rect, theme::bg());
        let item_w = self.width / self.items.len().max(1) as f32;
        for (i, label) in self.items.iter().enumerate() {
            let ix = rect.x + i as f32 * item_w;
            let is_selected = i == self.selected;
            let (tw, th) = renderer.measure_text(label, 14.0);
            renderer.draw_text_raw(
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
            if is_selected && self.dropdown_open {
                renderer.fill_rounded_rect(
                    Rect {
                        x: ix + 4.0,
                        y: rect.y + self.height,
                        width: item_w - 8.0,
                        height: 4.0,
                    },
                    2.0,
                    theme::accent(),
                );
            }
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

// =============================================================================
// CAROUSELS
// =============================================================================

/// Carousel -- generic carousel with navigation.
#[derive(Clone)]
pub struct Carousel {
    /// Page count.
    pub count: usize,
    /// Current page index.
    pub current: f32,
    /// Page width.
    pub page_width: f32,
    /// Page height.
    pub page_height: f32,
    /// Labels for each page.
    pub labels: Vec<String>,
}

impl Carousel {
    /// Create a new Carousel.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            current: 0.0,
            page_width: 300.0,
            page_height: 180.0,
            labels: (0..count).map(|i| format!("Slide {}", i + 1)).collect(),
        }
    }

    /// Set current page.
    pub fn current(mut self, c: f32) -> Self {
        self.current = c;
        self
    }

    /// Set page size.
    pub fn page_size(mut self, w: f32, h: f32) -> Self {
        self.page_width = w;
        self.page_height = h;
        self
    }

    /// Set page labels.
    pub fn labels(mut self, l: Vec<String>) -> Self {
        self.labels = l;
        self
    }
}

impl View for Carousel {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Carousel");
        // Clip region background
        renderer.fill_rounded_rect(rect, 16.0, theme::surface_elevated());
        let page_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.page_width,
            height: self.page_height,
        };
        // Current page content
        let page_idx = self.current as usize;
        if let Some(label) = self.labels.get(page_idx) {
            let (tw, th) = renderer.measure_text(label, 20.0);
            renderer.draw_text_raw(
                label,
                page_rect.x + (self.page_width - tw) / 2.0,
                page_rect.y + (self.page_height - th) / 2.0,
                20.0,
                theme::text(),
            );
        }
        // Page indicators
        let dot_y = rect.y + self.page_height + 12.0;
        let dots_total_w = self.count as f32 * 16.0;
        let dots_start_x = rect.x + (self.page_width - dots_total_w) / 2.0;
        for i in 0..self.count {
            let dx = dots_start_x + i as f32 * 16.0;
            let color = if i == page_idx {
                theme::accent()
            } else {
                theme::surface_elevated()
            };
            renderer.fill_ellipse(
                Rect {
                    x: dx,
                    y: dot_y,
                    width: 8.0,
                    height: 8.0,
                },
                color,
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.page_width,
            height: self.page_height + 32.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Marquee -- horizontal scrolling marquee
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Marquee {
    /// Text content (repeated for seamless loop).
    pub text: String,
    /// Scroll speed (pixels per second).
    pub speed: f32,
    /// Animation time.
    pub time: f32,
    /// Font size.
    pub font_size: f32,
    /// Marquee height.
    pub height: f32,
    /// Text color.
    pub color: [f32; 4],
    /// Background color.
    pub bg_color: [f32; 4],
}

impl Marquee {
    /// Create a new Marquee.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            speed: 60.0,
            time: 0.0,
            font_size: 14.0,
            height: 32.0,
            color: theme::text(),
            bg_color: theme::surface_elevated(),
        }
    }

    /// Set scroll speed.
    pub fn speed(mut self, s: f32) -> Self {
        self.speed = s;
        self
    }

    /// Set animation time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }

    /// Set font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set text color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }
}

impl View for Marquee {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Marquee");
        renderer.fill_rect(rect, self.bg_color);
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        let loop_w = tw + 40.0;
        let offset = (self.time * self.speed) % loop_w;
        // Draw text twice for seamless loop
        renderer.draw_text_raw(
            &self.text,
            rect.x - offset,
            rect.y + (self.height - th) / 2.0,
            self.font_size,
            self.color,
        );
        renderer.draw_text_raw(
            &self.text,
            rect.x - offset + loop_w,
            rect.y + (self.height - th) / 2.0,
            self.font_size,
            self.color,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: self.height,
        }
    }
}

// =============================================================================
// LAYOUT
// =============================================================================

/// BentoGrid -- bento-style grid layout.
#[derive(Clone)]
pub struct BentoGrid {
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
    /// Cell labels.
    pub cells: Vec<String>,
    /// Gap between cells.
    pub gap: f32,
    /// Grid width.
    pub width: f32,
    /// Grid height.
    pub height: f32,
}

impl BentoGrid {
    /// Create a new BentoGrid.
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: Vec::new(),
            gap: 8.0,
            width: 400.0,
            height: 300.0,
        }
    }

    /// Add a cell label.
    pub fn cell(mut self, label: &str) -> Self {
        self.cells.push(label.to_string());
        self
    }

    /// Set the gap.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    /// Set the grid size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for BentoGrid {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BentoGrid");
        let cell_w = (rect.width - (self.cols - 1) as f32 * self.gap) / self.cols as f32;
        let cell_h = (rect.height - (self.rows - 1) as f32 * self.gap) / self.rows as f32;
        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = row * self.cols + col;
                let cx = rect.x + col as f32 * (cell_w + self.gap);
                let cy = rect.y + row as f32 * (cell_h + self.gap);
                let cell_rect = Rect {
                    x: cx,
                    y: cy,
                    width: cell_w,
                    height: cell_h,
                };
                renderer.fill_rounded_rect(cell_rect, 12.0, theme::surface_elevated());
                renderer.stroke_rounded_rect(cell_rect, 12.0, theme::border(), 1.0);
                if let Some(label) = self.cells.get(idx) {
                    let (tw, th) = renderer.measure_text(label, 14.0);
                    renderer.draw_text_raw(
                        label,
                        cx + (cell_w - tw) / 2.0,
                        cy + (cell_h - th) / 2.0,
                        14.0,
                        theme::text(),
                    );
                }
            }
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

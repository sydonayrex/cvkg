//! Material 3 components: FAB, ExtendedFAB, TimePicker, DateRangePicker.
//! Cult UI components: HeroColorPanels, BgMediaHero, LogoCarousel.
//! Joy UI components: DynamicIsland, SidePanel.
//! Data components: Codeblock, Kanban.
//! All components use cvkg theme system (theme::*) for full themability.

use crate::lingua_tong;
use crate::theme;
use crate::{RADIUS_XL, RADIUS_LG, RADIUS_2XL};
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// =============================================================================
// MATERIAL 3 COMPONENTS
// =============================================================================

/// FAB (Floating Action Button) — circular, fixed-position primary action button.
#[derive(Clone)]
pub struct FAB {
    /// Icon text or label.
    pub icon: String,
    /// Button diameter.
    pub size: f32,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Icon/text color.
    pub icon_color: [f32; 4],
    /// Whether the button is pressed.
    pub pressed: bool,
}

impl FAB {
    /// Create a new FAB.
    pub fn new(icon: &str) -> Self {
        Self {
            icon: icon.to_string(),
            size: 56.0,
            bg_color: theme::accent(),
            icon_color: theme::bg(),
            pressed: false,
        }
    }

    /// Set the button size.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the icon color.
    pub fn icon_color(mut self, c: [f32; 4]) -> Self {
        self.icon_color = c;
        self
    }

    /// Set pressed state.
    pub fn pressed(mut self, p: bool) -> Self {
        self.pressed = p;
        self
    }
}

impl View for FAB {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FAB");
        let scale = if self.pressed { 0.9 } else { 1.0 };
        let r = self.size / 2.0 * scale;
        let cx = rect.x + self.size / 2.0;
        let cy = rect.y + self.size / 2.0;
        // Shadow
        renderer.push_shadow(12.0, theme::shadow(), [0.0, 4.0]);
        renderer.fill_ellipse(
            Rect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            },
            self.bg_color,
        );
        renderer.pop_shadow();
        // Icon
        let (tw, th) = renderer.measure_text(&self.icon, 20.0);
        renderer.draw_text(
            &self.icon,
            cx - tw / 2.0,
            cy - th / 2.0,
            20.0,
            self.icon_color,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.size,
            height: self.size,
        }
    }
}

// ----------------------------------------------------------------------------
// ExtendedFAB — FAB with label text for clarity
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ExtendedFAB {
    /// Icon text.
    pub icon: String,
    /// Label text.
    pub label: String,
    /// Button width.
    pub width: f32,
    /// Button height.
    pub height: f32,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Text color.
    pub text_color: [f32; 4],
    /// Whether pressed.
    pub pressed: bool,
}

impl ExtendedFAB {
    /// Create a new ExtendedFAB.
    pub fn new(icon: &str, label: &str) -> Self {
        Self {
            icon: icon.to_string(),
            label: label.to_string(),
            width: 160.0,
            height: 56.0,
            bg_color: theme::accent(),
            text_color: theme::bg(),
            pressed: false,
        }
    }

    /// Set the button size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, c: [f32; 4]) -> Self {
        self.text_color = c;
        self
    }

    /// Set pressed state.
    pub fn pressed(mut self, p: bool) -> Self {
        self.pressed = p;
        self
    }
}

impl View for ExtendedFAB {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ExtendedFAB");
        let scale = if self.pressed { 0.95 } else { 1.0 };
        let w = self.width * scale;
        let h = self.height * scale;
        let btn_rect = Rect {
            x: rect.x + (self.width - w) / 2.0,
            y: rect.y + (self.height - h) / 2.0,
            width: w,
            height: h,
        };
        renderer.push_shadow(12.0, theme::shadow(), [0.0, 4.0]);
        renderer.fill_rounded_rect(btn_rect, self.height / 2.0, self.bg_color);
        renderer.pop_shadow();
        // Icon
        renderer.draw_text(
            &self.icon,
            btn_rect.x + 16.0,
            btn_rect.y + 36.0,
            20.0,
            self.text_color,
        );
        // Label
        renderer.draw_text(
            &self.label,
            btn_rect.x + 48.0,
            btn_rect.y + 36.0,
            14.0,
            self.text_color,
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
// TimePicker — clock face time input
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct TimePicker {
    /// Selected hour (0-23).
    pub hour: u8,
    /// Selected minute (0-59).
    pub minute: u8,
    /// Clock face diameter.
    pub size: f32,
    /// Whether to show hours (true) or minutes (false).
    pub show_hours: bool,
}

impl TimePicker {
    /// Create a new TimePicker.
    pub fn new(hour: u8, minute: u8) -> Self {
        Self {
            hour: hour % 24,
            minute: minute % 60,
            size: 200.0,
            show_hours: true,
        }
    }

    /// Set the clock size.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }

    /// Toggle between hours and minutes.
    pub fn mode(mut self, show_hours: bool) -> Self {
        self.show_hours = show_hours;
        self
    }
}

impl View for TimePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TimePicker");
        let cx = rect.x + self.size / 2.0;
        let cy = rect.y + self.size / 2.0;
        let r = self.size / 2.0 - 16.0;
        // Clock face
        renderer.fill_ellipse(
            Rect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            },
            theme::surface_elevated(),
        );
        renderer.stroke_ellipse(
            Rect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            },
            theme::border(),
            1.0,
        );
        // Numbers
        let max_val = 12;
        let selected = if self.show_hours {
            self.hour % 12
        } else {
            self.minute / 5
        };
        for i in 0..max_val {
            let angle = (i as f32 / max_val as f32) * std::f32::consts::TAU - 1.57;
            let nx = cx + angle.cos() * r * 0.75;
            let ny = cy + angle.sin() * r * 0.75;
            let label = if self.show_hours {
                format!("{}", if i == 0 { 12 } else { i })
            } else {
                format!("{}", i * 5)
            };
            let color = if i == selected as usize {
                theme::accent()
            } else {
                theme::text()
            };
            let bg = if i == selected as usize {
                theme::accent()
            } else {
                theme::button_ghost_bg()
            };
            if i == selected as usize {
                renderer.fill_ellipse(
                    Rect {
                        x: nx - 14.0,
                        y: ny - 14.0,
                        width: 28.0,
                        height: 28.0,
                    },
                    bg,
                );
            }
            let (tw, th) = renderer.measure_text(&label, 12.0);
            renderer.draw_text(
                &label,
                nx - tw / 2.0,
                ny - th / 2.0,
                12.0,
                if i == selected as usize {
                    theme::bg()
                } else {
                    color
                },
            );
        }
        // Center time display
        let time_str = format!("{:02}:{:02}", self.hour, self.minute);
        let (tw, _th) = renderer.measure_text(&time_str, 16.0);
        renderer.draw_text(&time_str, cx - tw / 2.0, cy + r + 12.0, 16.0, theme::text());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.size,
            height: self.size + 32.0,
        }
    }
}

// ----------------------------------------------------------------------------
// DateRangePicker — calendar for selecting date ranges
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DateRangePicker {
    /// Month (1-12).
    pub month: u8,
    /// Year.
    pub year: u32,
    /// Range start day (0 = not set).
    pub range_start: u8,
    /// Range end day (0 = not set).
    pub range_end: u8,
    /// Currently hovered day.
    pub hovered: u8,
    /// Calendar width.
    pub width: f32,
}

impl DateRangePicker {
    /// Create a new DateRangePicker.
    pub fn new(month: u8, year: u32) -> Self {
        Self {
            month: month.clamp(1, 12),
            year,
            range_start: 0,
            range_end: 0,
            hovered: 0,
            width: 280.0,
        }
    }

    /// Set the range.
    pub fn range(mut self, start: u8, end: u8) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// Set the hovered day.
    pub fn hovered(mut self, d: u8) -> Self {
        self.hovered = d;
        self
    }

    /// Set the calendar width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl View for DateRangePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DateRangePicker");
        renderer.fill_rounded_rect(rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, RADIUS_XL, theme::border(), 1.0);
        // Month/year header
        let header = format!(
            "{} {}",
            lingua_tong::t(&format!("datepicker.month.{}", match self.month {
                1 => "jan",
                2 => "feb",
                3 => "mar",
                4 => "apr",
                5 => "may_short",
                6 => "jun",
                7 => "jul",
                8 => "aug",
                9 => "sep",
                10 => "oct",
                11 => "nov",
                12 => "dec",
                _ => "jan",
            })),
            self.year
        );
        let (tw, _th) = renderer.measure_text(&header, 16.0);
        renderer.draw_text(
            &header,
            rect.x + (self.width - tw) / 2.0,
            rect.y + 24.0,
            16.0,
            theme::text(),
        );
        // Day headers
        let days = [
            lingua_tong::t("datepicker.day.su"),
            lingua_tong::t("datepicker.day.mo"),
            lingua_tong::t("datepicker.day.tu"),
            lingua_tong::t("datepicker.day.we"),
            lingua_tong::t("datepicker.day.th"),
            lingua_tong::t("datepicker.day.fr"),
            lingua_tong::t("datepicker.day.sa"),
        ];
        let cell_w = self.width / 7.0;
        for (i, day) in days.iter().enumerate() {
            renderer.draw_text(
                day,
                rect.x + i as f32 * cell_w + cell_w / 2.0 - 8.0,
                rect.y + 48.0,
                11.0,
                theme::text_muted(),
            );
        }
        // Calendar grid (simplified: 30 days, starting on day 1 = Monday)
        let days_in_month = 30;
        let start_dow = 1; // Monday
        for day in 1..=days_in_month {
            let idx = (day + start_dow - 1) as usize;
            let col = idx % 7;
            let row = idx / 7;
            let cx = rect.x + col as f32 * cell_w;
            let cy = rect.y + 68.0 + row as f32 * 32.0;
            let in_range = if self.range_start > 0 && self.range_end > 0 {
                day >= self.range_start && day <= self.range_end
            } else {
                false
            };
            let is_start = day == self.range_start;
            let is_end = day == self.range_end;
            // Range highlight
            if in_range {
                renderer.fill_rect(
                    Rect {
                        x: cx,
                        y: cy,
                        width: cell_w,
                        height: 28.0,
                    },
                    [
                        theme::accent()[0],
                        theme::accent()[1],
                        theme::accent()[2],
                        0.15,
                    ],
                );
            }
            // Start/end circle
            if is_start || is_end {
                renderer.fill_ellipse(
                    Rect {
                        x: cx + cell_w / 2.0 - 14.0,
                        y: cy,
                        width: 28.0,
                        height: 28.0,
                    },
                    theme::accent(),
                );
            }
            let day_str = format!("{}", day);
            let (dw, dh) = renderer.measure_text(&day_str, 13.0);
            renderer.draw_text(
                &day_str,
                cx + (cell_w - dw) / 2.0,
                cy + (28.0 - dh) / 2.0,
                13.0,
                if is_start || is_end {
                    theme::bg()
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
            height: 280.0,
        }
    }
}

// =============================================================================
// CULT UI COMPONENTS
// =============================================================================

/// HeroColorPanels — hero with animated color panel grid.
#[derive(Clone)]
pub struct HeroColorPanels {
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
    /// Animation time.
    pub time: f32,
    /// Panel width.
    pub width: f32,
    /// Panel height.
    pub height: f32,
}

impl HeroColorPanels {
    /// Create a new HeroColorPanels.
    pub fn new() -> Self {
        Self {
            cols: 4,
            rows: 3,
            time: 0.0,
            width: 400.0,
            height: 240.0,
        }
    }

    /// Set grid dimensions.
    pub fn grid(mut self, cols: usize, rows: usize) -> Self {
        self.cols = cols;
        self.rows = rows;
        self
    }

    /// Set animation time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }

    /// Set panel size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl Default for HeroColorPanels {
    fn default() -> Self {
        Self::new()
    }
}

impl View for HeroColorPanels {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HeroColorPanels");
        let cell_w = self.width / self.cols as f32;
        let cell_h = self.height / self.rows as f32;
        let colors: [[f32; 4]; 6] = [
            [0.0, 1.0, 1.0, 0.8],  // Cyan
            [1.0, 0.0, 1.0, 0.8],  // Magenta
            [1.0, 0.84, 0.0, 0.8], // Gold
            [0.0, 0.8, 1.0, 0.8],  // Blue
            [1.0, 0.3, 0.1, 0.8],  // Orange
            [0.2, 1.0, 0.5, 0.8],  // Green
        ];
        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = (row * self.cols + col) % colors.len();
                let phase = self.time * 0.5 + (row + col) as f32 * 0.3;
                let alpha = 0.5 + phase.sin() * 0.3;
                let color = [colors[idx][0], colors[idx][1], colors[idx][2], alpha];
                let gap = 4.0;
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x + col as f32 * cell_w + gap,
                        y: rect.y + row as f32 * cell_h + gap,
                        width: cell_w - gap * 2.0,
                        height: cell_h - gap * 2.0,
                    },
                    8.0,
                    color,
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

// ----------------------------------------------------------------------------
// BgMediaHero — hero with video/image background
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct BgMediaHero {
    /// Hero title.
    pub title: String,
    /// Hero subtitle.
    pub subtitle: String,
    /// Background color (simulating media).
    pub bg_color: [f32; 4],
    /// Overlay opacity.
    pub overlay: f32,
    /// Hero width.
    pub width: f32,
    /// Hero height.
    pub height: f32,
}

impl BgMediaHero {
    /// Create a new BgMediaHero.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            subtitle: String::new(),
            bg_color: [0.05, 0.05, 0.15, 1.0],
            overlay: 0.5,
            width: 600.0,
            height: 300.0,
        }
    }

    /// Set the subtitle.
    pub fn subtitle(mut self, s: &str) -> Self {
        self.subtitle = s.to_string();
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set overlay opacity.
    pub fn overlay(mut self, o: f32) -> Self {
        self.overlay = o.clamp(0.0, 1.0);
        self
    }

    /// Set hero size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for BgMediaHero {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BgMediaHero");
        // Background
        renderer.fill_rect(rect, self.bg_color);
        // Gradient overlay
        renderer.fill_rect(rect, theme::with_alpha(theme::bg(), self.overlay));
        // Title
        let (tw, th) = renderer.measure_text(&self.title, 32.0);
        renderer.draw_text(
            &self.title,
            rect.x + (self.width - tw) / 2.0,
            rect.y + self.height * 0.4,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        // Subtitle
        if !self.subtitle.is_empty() {
            let (sw, _sh) = renderer.measure_text(&self.subtitle, 16.0);
            renderer.draw_text(
                &self.subtitle,
                rect.x + (self.width - sw) / 2.0,
                rect.y + self.height * 0.4 + th + 12.0,
                16.0,
                [1.0, 1.0, 1.0, 0.8],
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
// LogoCarousel — animated logo carousel (marquee)
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct LogoCarousel {
    /// Logo labels.
    pub logos: Vec<String>,
    /// Scroll speed.
    pub speed: f32,
    /// Animation time.
    pub time: f32,
    /// Carousel height.
    pub height: f32,
    /// Logo width.
    pub logo_width: f32,
}

impl LogoCarousel {
    /// Create a new LogoCarousel.
    pub fn new(logos: Vec<String>) -> Self {
        Self {
            logos,
            speed: 40.0,
            time: 0.0,
            height: 60.0,
            logo_width: 120.0,
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

    /// Set carousel height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set logo width.
    pub fn logo_width(mut self, w: f32) -> Self {
        self.logo_width = w;
        self
    }
}

impl View for LogoCarousel {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "LogoCarousel");
        renderer.fill_rect(rect, theme::bg());
        let total_w = self.logos.len() as f32 * (self.logo_width + 20.0);
        let offset = (self.time * self.speed) % total_w;
        for (i, logo) in self.logos.iter().enumerate() {
            let lx = rect.x + i as f32 * (self.logo_width + 20.0) - offset;
            let logo_rect = Rect {
                x: lx,
                y: rect.y + 10.0,
                width: self.logo_width,
                height: self.height - 20.0,
            };
            renderer.fill_rounded_rect(logo_rect, RADIUS_LG, theme::surface_elevated());
            renderer.stroke_rounded_rect(logo_rect, RADIUS_LG, theme::border(), 1.0);
            let (tw, th) = renderer.measure_text(logo, 14.0);
            renderer.draw_text(
                logo,
                logo_rect.x + (self.logo_width - tw) / 2.0,
                logo_rect.y + (self.height - 20.0 - th) / 2.0,
                14.0,
                theme::text(),
            );
        }
        // Second copy for seamless loop
        for (i, logo) in self.logos.iter().enumerate() {
            let lx = rect.x + total_w + i as f32 * (self.logo_width + 20.0) - offset;
            let logo_rect = Rect {
                x: lx,
                y: rect.y + 10.0,
                width: self.logo_width,
                height: self.height - 20.0,
            };
            renderer.fill_rounded_rect(logo_rect, RADIUS_LG, theme::surface_elevated());
            renderer.stroke_rounded_rect(logo_rect, RADIUS_LG, theme::border(), 1.0);
            let (tw, th) = renderer.measure_text(logo, 14.0);
            renderer.draw_text(
                logo,
                logo_rect.x + (self.logo_width - tw) / 2.0,
                logo_rect.y + (self.height - 20.0 - th) / 2.0,
                14.0,
                theme::text(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(600.0),
            height: self.height,
        }
    }
}

// =============================================================================
// JOY UI COMPONENTS
// =============================================================================

/// DynamicIsland — iOS Dynamic Island-style expandable.
#[derive(Clone)]
pub struct DynamicIsland {
    /// Primary content label.
    pub label: String,
    /// Expanded content label.
    pub expanded_label: String,
    /// Expansion progress 0.0..1.0.
    pub progress: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Height.
    pub height: f32,
}

impl DynamicIsland {
    /// Create a new DynamicIsland.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            expanded_label: String::new(),
            progress: 0.0,
            min_width: 120.0,
            max_width: 300.0,
            height: 36.0,
        }
    }

    /// Set the expanded label.
    pub fn expanded(mut self, label: &str) -> Self {
        self.expanded_label = label.to_string();
        self
    }

    /// Set expansion progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set size constraints.
    pub fn size(mut self, min_w: f32, max_w: f32, h: f32) -> Self {
        self.min_width = min_w;
        self.max_width = max_w;
        self.height = h;
        self
    }
}

impl View for DynamicIsland {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DynamicIsland");
        let w = self.min_width + (self.max_width - self.min_width) * self.progress;
        let h = self.height + (self.progress * 20.0);
        let island_rect = Rect {
            x: rect.x + (rect.width - w) / 2.0,
            y: rect.y,
            width: w,
            height: h,
        };
        renderer.fill_rounded_rect(island_rect, h / 2.0, theme::surface());
        // Label
        let display = if self.progress > 0.5 && !self.expanded_label.is_empty() {
            &self.expanded_label
        } else {
            &self.label
        };
        let font_size = 12.0 + self.progress * 2.0;
        let (tw, th) = renderer.measure_text(display, font_size);
        renderer.draw_text(
            display,
            island_rect.x + (w - tw) / 2.0,
            island_rect.y + (h - th) / 2.0,
            font_size,
            [1.0, 1.0, 1.0, 1.0],
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.max_width,
            height: self.height + 20.0,
        }
    }
}

// ----------------------------------------------------------------------------
// SidePanel — side panel that slides in
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SidePanel {
    /// Panel title.
    pub title: String,
    /// Panel content lines.
    pub content: Vec<String>,
    /// Slide progress 0.0..1.0.
    pub progress: f32,
    /// Panel width.
    pub width: f32,
    /// Panel height.
    pub height: f32,
    /// Position: "left" or "right".
    pub position: String,
}

impl SidePanel {
    /// Create a new SidePanel.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: Vec::new(),
            progress: 1.0,
            width: 280.0,
            height: 400.0,
            position: "right".to_string(),
        }
    }

    /// Add content line.
    pub fn line(mut self, text: &str) -> Self {
        self.content.push(text.to_string());
        self
    }

    /// Set slide progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set panel size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set position.
    pub fn position(mut self, p: &str) -> Self {
        self.position = p.to_string();
        self
    }
}

impl View for SidePanel {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SidePanel");
        let offset_x = if self.position == "right" {
            (1.0 - self.progress) * self.width
        } else {
            -(1.0 - self.progress) * self.width
        };
        let panel_rect = Rect {
            x: rect.x + offset_x,
            y: rect.y,
            width: self.width,
            height: self.height,
        };
        renderer.fill_rounded_rect(panel_rect, RADIUS_2XL, theme::surface_overlay());
        renderer.stroke_rounded_rect(panel_rect, RADIUS_2XL, theme::border(), 1.0);
        // Title
        renderer.draw_text(
            &self.title,
            panel_rect.x + 16.0,
            panel_rect.y + 28.0,
            18.0,
            theme::text(),
        );
        // Divider
        renderer.draw_line(
            panel_rect.x + 16.0,
            panel_rect.y + 40.0,
            panel_rect.x + self.width - 16.0,
            panel_rect.y + 40.0,
            theme::border(),
            1.0,
        );
        // Content
        let mut y = panel_rect.y + 56.0;
        for line in &self.content {
            renderer.draw_text(line, panel_rect.x + 16.0, y + 16.0, 14.0, theme::text());
            y += 28.0;
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
// DATA COMPONENTS
// =============================================================================

/// Codeblock — syntax-highlighted code block with copy button.
#[derive(Clone)]
pub struct Codeblock {
    /// Code content.
    pub code: String,
    /// Language label.
    pub language: String,
    /// Block width.
    pub width: f32,
    /// Block height.
    pub height: f32,
}

impl Codeblock {
    /// Create a new Codeblock.
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            language: "rust".to_string(),
            width: 400.0,
            height: 200.0,
        }
    }

    /// Set the language.
    pub fn language(mut self, lang: &str) -> Self {
        self.language = lang.to_string();
        self
    }

    /// Set the block size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for Codeblock {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Codeblock");
        renderer.fill_rounded_rect(rect, RADIUS_LG, theme::code_bg());
        renderer.stroke_rounded_rect(rect, RADIUS_LG, theme::border(), 1.0);
        // Top bar
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 32.0,
            },
            8.0,
            theme::surface_elevated(),
        );
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y + 16.0,
                width: rect.width,
                height: 16.0,
            },
            theme::surface_elevated(),
        );
        renderer.draw_line(
            rect.x,
            rect.y + 32.0,
            rect.x + rect.width,
            rect.y + 32.0,
            theme::border(),
            1.0,
        );
        // Language label
        renderer.draw_text(
            &self.language,
            rect.x + 12.0,
            rect.y + 20.0,
            12.0,
            theme::text_dim(),
        );
        // Copy button
        renderer.draw_text(
            "Copy",
            rect.x + rect.width - 50.0,
            rect.y + 20.0,
            12.0,
            theme::text_muted(),
        );
        // Code content
        let mut y = rect.y + 44.0;
        for line in self.code.lines() {
            renderer.draw_text(line, rect.x + 12.0, y + 14.0, 13.0, theme::text());
            y += 20.0;
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
// Kanban — kanban board with drag-and-drop columns
// ----------------------------------------------------------------------------

/// A single kanban card.
#[derive(Clone)]
pub struct KanbanCard {
    /// Card title.
    pub title: String,
    /// Card description.
    pub description: String,
}

impl KanbanCard {
    /// Create a new KanbanCard.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
        }
    }

    /// Set the description.
    pub fn description(mut self, d: &str) -> Self {
        self.description = d.to_string();
        self
    }
}

/// A kanban column.
#[derive(Clone)]
pub struct KanbanColumn {
    /// Column title.
    pub title: String,
    /// Cards in this column.
    pub cards: Vec<KanbanCard>,
}

impl KanbanColumn {
    /// Create a new KanbanColumn.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            cards: Vec::new(),
        }
    }

    /// Add a card.
    pub fn card(mut self, card: KanbanCard) -> Self {
        self.cards.push(card);
        self
    }
}

/// Kanban — kanban board with columns.
#[derive(Clone)]
pub struct Kanban {
    /// Columns.
    pub columns: Vec<KanbanColumn>,
    /// Board width.
    pub width: f32,
    /// Board height.
    pub height: f32,
    /// Column width.
    pub column_width: f32,
}

impl Kanban {
    /// Create a new Kanban board.
    pub fn new() -> Self {
        Self {
            columns: vec![
                KanbanColumn::new("To Do"),
                KanbanColumn::new("In Progress"),
                KanbanColumn::new("Done"),
            ],
            width: 720.0,
            height: 400.0,
            column_width: 220.0,
        }
    }

    /// Add a column.
    pub fn column(mut self, col: KanbanColumn) -> Self {
        self.columns.push(col);
        self
    }

    /// Set board size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set column width.
    pub fn column_width(mut self, w: f32) -> Self {
        self.column_width = w;
        self
    }
}

impl Default for Kanban {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Kanban {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Kanban");
        renderer.fill_rect(rect, theme::bg());
        let gap = 12.0;
        for (ci, col) in self.columns.iter().enumerate() {
            let cx = rect.x + ci as f32 * (self.column_width + gap);
            let col_rect = Rect {
                x: cx,
                y: rect.y,
                width: self.column_width,
                height: self.height,
            };
            // Column background
            renderer.fill_rounded_rect(col_rect, RADIUS_XL, theme::surface());
            // Column header
            renderer.draw_text(&col.title, cx + 12.0, rect.y + 24.0, 14.0, theme::text());
            // Card count badge
            let count_str = format!("{}", col.cards.len());
            let (cw, _) = renderer.measure_text(&count_str, 11.0);
            renderer.fill_rounded_rect(
                Rect {
                    x: cx + self.column_width - cw - 20.0,
                    y: rect.y + 12.0,
                    width: cw + 12.0,
                    height: 20.0,
                },
                10.0,
                theme::surface_elevated(),
            );
            renderer.draw_text(
                &count_str,
                cx + self.column_width - cw - 14.0,
                rect.y + 26.0,
                11.0,
                theme::text_muted(),
            );
            // Cards
            let mut card_y = rect.y + 48.0;
            for card in &col.cards {
                let card_rect = Rect {
                    x: cx + 8.0,
                    y: card_y,
                    width: self.column_width - 16.0,
                    height: 64.0,
                };
                renderer.fill_rounded_rect(card_rect, RADIUS_LG, theme::surface_elevated());
                renderer.stroke_rounded_rect(card_rect, RADIUS_LG, theme::border(), 1.0);
                renderer.draw_text(
                    &card.title,
                    card_rect.x + 12.0,
                    card_rect.y + 20.0,
                    13.0,
                    theme::text(),
                );
                if !card.description.is_empty() {
                    renderer.draw_text(
                        &card.description,
                        card_rect.x + 12.0,
                        card_rect.y + 38.0,
                        11.0,
                        theme::text_muted(),
                    );
                }
                card_y += 72.0;
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

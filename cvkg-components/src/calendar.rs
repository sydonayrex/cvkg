use cvkg_core::{Never, Rect, Renderer, Size, View, SizeProposal};
use std::sync::Arc;

/// Basic date structure for calendar components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn today() -> Self {
        // Mocking today's date for simplicity
        Self { year: 2026, month: 4, day: 30 }
    }

    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Calendar component for selecting dates.
pub struct Calendar {
    pub(crate) selected_date: Date,
    pub(crate) on_date_select: Arc<dyn Fn(Date) + Send + Sync>,
    pub(crate) min_date: Option<Date>,
    pub(crate) max_date: Option<Date>,
}

impl Calendar {
    pub fn new(on_date_select: impl Fn(Date) + Send + Sync + 'static) -> Self {
        Self {
            selected_date: Date::today(),
            on_date_select: Arc::new(on_date_select),
            min_date: None,
            max_date: None,
        }
    }

    pub fn selected_date(mut self, date: Date) -> Self {
        self.selected_date = date;
        self
    }
}

impl View for Calendar {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Calendar");
        
        // Background
        renderer.fill_rounded_rect(rect, 8.0, [0.08, 0.08, 0.12, 1.0]);
        renderer.stroke_rect(rect, [0.2, 0.2, 0.3, 1.0], 1.0);

        // Header (Month Year)
        let header_h = 40.0;
        let _header_rect = Rect { x: rect.x, y: rect.y, width: rect.width, height: header_h };
        let title = format!("{} {}", month_name(self.selected_date.month), self.selected_date.year);
        let (tw, th) = renderer.measure_text(&title, 16.0);
        renderer.draw_text(&title, rect.x + (rect.width - tw) / 2.0, rect.y + (header_h - th) / 2.0, 16.0, [1.0, 1.0, 1.0, 1.0]);

        // Days of week
        let day_w = rect.width / 7.0;
        let days = ["S", "M", "T", "W", "T", "F", "S"];
        for (i, day) in days.iter().enumerate() {
            renderer.draw_text(day, rect.x + i as f32 * day_w + (day_w - 10.0) / 2.0, rect.y + header_h + 5.0, 12.0, [0.5, 0.5, 0.6, 1.0]);
        }

        // Days grid (mocked for demo)
        let grid_y = rect.y + header_h + 25.0;
        for row in 0..5 {
            for col in 0..7 {
                let day_num = row * 7 + col + 1;
                if day_num > 30 { break; }
                let cell_rect = Rect {
                    x: rect.x + col as f32 * day_w,
                    y: grid_y + row as f32 * 30.0,
                    width: day_w,
                    height: 30.0,
                };

                let is_selected = day_num == self.selected_date.day as usize;
                if is_selected {
                    renderer.fill_rounded_rect(cell_rect, 4.0, [0.0, 0.8, 1.0, 0.4]);
                }

                let day_str = day_num.to_string();
                let (dtw, dth) = renderer.measure_text(&day_str, 14.0);
                renderer.draw_text(&day_str, cell_rect.x + (day_w - dtw) / 2.0, cell_rect.y + (30.0 - dth) / 2.0, 14.0, [1.0, 1.0, 1.0, 1.0]);
            }
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(250.0), height: 220.0 }
    }
}

fn month_name(m: u32) -> &'static str {
    match m {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown"
    }
}

/// DatePicker component using a popover calendar.
pub struct DatePicker {
    pub(crate) selected_date: Date,
    pub(crate) placeholder: String,
    pub(crate) on_date_change: Arc<dyn Fn(Date) + Send + Sync>,
}

impl DatePicker {
    pub fn new(on_date_change: impl Fn(Date) + Send + Sync + 'static) -> Self {
        Self {
            selected_date: Date::today(),
            placeholder: "Select date".into(),
            on_date_change: Arc::new(on_date_change),
        }
    }
}

impl View for DatePicker {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 4.0, [0.1, 0.1, 0.15, 1.0]);
        renderer.stroke_rect(rect, [0.3, 0.3, 0.4, 1.0], 1.0);
        
        let display = self.selected_date.format();
        renderer.draw_text(&display, rect.x + 8.0, rect.y + (rect.height - 14.0) / 2.0, 14.0, [1.0, 1.0, 1.0, 1.0]);
        renderer.draw_text("📅", rect.x + rect.width - 24.0, rect.y + (rect.height - 14.0) / 2.0, 14.0, [0.5, 0.5, 0.6, 1.0]);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(180.0), height: 32.0 }
    }
}

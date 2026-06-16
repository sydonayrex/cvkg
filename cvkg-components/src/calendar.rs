use crate::lingua_tong;
use crate::theme;
use crate::{RADIUS_LG, RADIUS_SM};
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

/// Basic date structure for calendar components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Returns today's date (mocked for simplicity).
    pub fn today() -> Self {
        // Mocking today's date for simplicity
        Self {
            year: 2026,
            month: 4,
            day: 30,
        }
    }

    /// Formats the date as YYYY-MM-DD.
    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A calendar component for selecting dates or date ranges.
pub struct TyrCalendar {
    pub(crate) selected_date: Date,
    pub(crate) range_end: Option<Date>,
    pub(crate) on_date_select: Arc<dyn Fn(Date) + Send + Sync>,
    pub(crate) min_date: Option<Date>,
    pub(crate) max_date: Option<Date>,
}

impl TyrCalendar {
    /// Creates a new TyrCalendar with the given selection handler.
    pub fn new(on_date_select: impl Fn(Date) + Send + Sync + 'static) -> Self {
        Self {
            selected_date: Date::today(),
            range_end: None,
            on_date_select: Arc::new(on_date_select),
            min_date: None,
            max_date: None,
        }
    }

    /// Sets the selected date.
    pub fn selected_date(mut self, date: Date) -> Self {
        self.selected_date = date;
        self
    }

    /// Sets the end of the selected date range.
    pub fn range_end(mut self, date: Date) -> Self {
        self.range_end = Some(date);
        self
    }

    /// Sets the minimum selectable date.
    pub fn min_date(mut self, date: Date) -> Self {
        self.min_date = Some(date);
        self
    }

    /// Sets the maximum selectable date.
    pub fn max_date(mut self, date: Date) -> Self {
        self.max_date = Some(date);
        self
    }
}

impl View for TyrCalendar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TyrCalendar");

        // Background
        renderer.fill_rounded_rect(rect, RADIUS_LG, theme::surface_elevated());
        renderer.stroke_rect(rect, theme::border_strong(), 1.0);

        // Header (Month Year)
        let header_h = 40.0;
        let _header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };
        let title = format!(
            "{} {}",
            month_name(self.selected_date.month),
            self.selected_date.year
        );
        let (tw, th) = renderer.measure_text(&title, 16.0);
        renderer.draw_text(
            &title,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (header_h - th) / 2.0,
            16.0,
            theme::text(),
        );

        // Days of week
        let day_w = rect.width / 7.0;
        let days = [
            lingua_tong::t("datepicker.day.su"),
            lingua_tong::t("datepicker.day.mo"),
            lingua_tong::t("datepicker.day.tu"),
            lingua_tong::t("datepicker.day.we"),
            lingua_tong::t("datepicker.day.th"),
            lingua_tong::t("datepicker.day.fr"),
            lingua_tong::t("datepicker.day.sa"),
        ];
        for (i, day) in days.iter().enumerate() {
            renderer.draw_text(
                day,
                rect.x + i as f32 * day_w + (day_w - 10.0) / 2.0,
                rect.y + header_h + 5.0,
                12.0,
                theme::text_muted(),
            );
        }

        // Days grid
        let grid_y = rect.y + header_h + 25.0;
        let today = self.selected_date;
        let days_in_month = match today.month {
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (today.year % 4 == 0 && today.year % 100 != 0) || today.year % 400 == 0 {
                    29
                } else {
                    28
                }
            }
            _ => 31,
        };

        for row in 0..6 {
            for col in 0..7 {
                let day_num = row * 7 + col + 1;
                if day_num > days_in_month {
                    break;
                }
                let cell_rect = Rect {
                    x: rect.x + col as f32 * day_w,
                    y: grid_y + row as f32 * 30.0,
                    width: day_w,
                    height: 30.0,
                };

                let _date = Date {
                    year: today.year,
                    month: today.month,
                    day: day_num as u32,
                };
                let is_selected = day_num == self.selected_date.day as usize;

                let is_disabled = self.min_date.is_some_and(|min| {
                    day_num < min.day as usize && today.month == min.month && today.year == min.year
                }) || self.max_date.is_some_and(|max| {
                    day_num > max.day as usize && today.month == max.month && today.year == max.year
                });

                if is_selected {
                    renderer.fill_rounded_rect(cell_rect, RADIUS_SM, theme::accent());
                } else if is_disabled {
                    renderer.fill_rounded_rect(cell_rect, RADIUS_SM, theme::with_alpha(theme::surface_elevated(), 0.2));
                }

                let day_str = day_num.to_string();
                let (dtw, dth) = renderer.measure_text(&day_str, 14.0);
                let text_color = if is_disabled {
                    [0.3, 0.3, 0.35, 1.0]
                } else {
                    theme::text()
                };
                renderer.draw_text(
                    &day_str,
                    cell_rect.x + (day_w - dtw) / 2.0,
                    cell_rect.y + (30.0 - dth) / 2.0,
                    14.0,
                    text_color,
                );
            }
        }

        // Event Handling
        let on_date_select = self.on_date_select.clone();
        let year = self.selected_date.year;
        let month = self.selected_date.month;
        let rect_clone = rect;

        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    // Simplified hit testing for the grid
                    let local_x = x - rect_clone.x;
                    let local_y = y - (rect_clone.y + header_h + 25.0);

                    if (0.0..180.0).contains(&local_y) {
                        let col = (local_x / day_w) as i32;
                        let row = (local_y / 30.0) as i32;
                        if (0..7).contains(&col) && (0..6).contains(&row) {
                            let day = row * 7 + col + 1;
                            if day >= 1 && day <= days_in_month as i32 {
                                on_date_select(Date {
                                    year,
                                    month,
                                    day: day as u32,
                                });
                            }
                        }
                    }
                }
            }),
        );

        renderer.pop_vnode();
    }
}

impl LayoutView for TyrCalendar {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 250.0,
            height: 220.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

fn month_name(m: u32) -> String {
    let key = match m {
        1 => "datepicker.month.jan",
        2 => "datepicker.month.feb",
        3 => "datepicker.month.mar",
        4 => "datepicker.month.apr",
        5 => "datepicker.month.may_short",
        6 => "datepicker.month.jun",
        7 => "datepicker.month.jul",
        8 => "datepicker.month.aug",
        9 => "datepicker.month.sep",
        10 => "datepicker.month.oct",
        11 => "datepicker.month.nov",
        12 => "datepicker.month.dec",
        _ => "???",
    };
    lingua_tong::t(key)
}

/// DatePicker component using a popover TyrCalendar.
pub struct DatePicker {
    pub(crate) selected_date: Date,
    pub(crate) placeholder: String,
    pub(crate) on_date_change: Arc<dyn Fn(Date) + Send + Sync>,
}

impl DatePicker {
    /// Creates a new DatePicker with the given change handler.
    pub fn new(on_date_change: impl Fn(Date) + Send + Sync + 'static) -> Self {
        Self {
            selected_date: Date {
                year: 0,
                month: 0,
                day: 0,
            },
            placeholder: "Select date".into(),
            on_date_change: Arc::new(on_date_change),
        }
    }
}

impl View for DatePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DatePicker");
        renderer.fill_rounded_rect(rect, RADIUS_SM, theme::surface());
        renderer.stroke_rect(rect, theme::text_dim(), 1.0);

        let display = if self.selected_date.year == 0 {
            self.placeholder.clone()
        } else {
            self.selected_date.format()
        };
        let text_color = if self.selected_date.year == 0 {
            theme::text_muted()
        } else {
            theme::text()
        };

        renderer.draw_text(
            &display,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            text_color,
        );
        renderer.draw_text(
            "📅",
            rect.x + rect.width - 24.0,
            rect.y + (rect.height - 14.0) / 2.0,
            14.0,
            theme::text_muted(),
        );

        let id_hash = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "datepicker".hash(&mut s);
            rect.x.to_bits().hash(&mut s);
            rect.y.to_bits().hash(&mut s);
            s.finish()
        };

        let is_expanded = {
            let s = cvkg_core::load_system_state();
            s.get_component_state::<bool>(id_hash)
                .and_then(|v| v.read().ok().map(|g| *g))
                .unwrap_or(false)
        };

        if is_expanded {
            let cal_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height + 4.0,
                width: 250.0,
                height: 220.0,
            };
            renderer.set_z_index(100.0);
            let on_date_change_cal = self.on_date_change.clone();
            let cal = TyrCalendar::new(move |date| {
                on_date_change_cal(date);
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, false);
                    s
                });
            })
            .selected_date(self.selected_date);
            cal.render(renderer, cal_rect);
            renderer.set_z_index(0.0);
        }

        renderer.register_handler(
            "pointerclick",
            Arc::new(move |_event| {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, !is_expanded);
                    s
                });
            }),
        );
        renderer.pop_vnode();
    }
}

impl LayoutView for DatePicker {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: proposal.width.unwrap_or(180.0),
            height: 32.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

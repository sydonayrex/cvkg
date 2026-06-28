use crate::lingua_tong;
use crate::theme;
use crate::{RADIUS_MD, RADIUS_SM, RADIUS_XL};
use cvkg_core::{Event, Never, Rect, Renderer, View, load_system_state, update_system_state};
use std::sync::Arc;

/// Mode for DatePicker: single date or range selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerMode {
    /// Select a single date.
    Single,
    /// Select a date range (start and end).
    Range,
}

/// Represents a date range (inclusive start, inclusive end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    /// Start date as (day, month, year).
    pub start: (u32, u32, u32),
    /// End date as (day, month, year).
    pub end: (u32, u32, u32),
}

/// Return the translated month name for a given month (1-12).
fn month_name(month: u32) -> String {
    let name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "January",
    };
    name.to_string()
}

/// Return translated day-of-week column headers.
fn day_headers() -> [String; 7] {
    [
        "Su".to_string(),
        "Mo".to_string(),
        "Tu".to_string(),
        "We".to_string(),
        "Th".to_string(),
        "Fr".to_string(),
        "Sa".to_string(),
    ]
}

/// Return the number of days in a given month (1-12) for a given year.
fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Return true if the given year is a leap year.
fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Compute the day of the week (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
/// for a given date using Tomohiko Sakamoto's algorithm.
fn day_of_week(year: u32, month: u32, day: u32) -> u32 {
    let mut y = year;
    let mut m = month;
    if m < 3 {
        y -= 1;
        m += 12;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // h: 0=Sat, 1=Sun, 2=Mon, ... 6=Fri
    // Convert to 0=Sun, 1=Mon, ... 6=Sat
    (h + 6) % 7
}

/// Get today's date as (day, month, year). Uses a simple approximation
/// based on the system clock via the renderer's elapsed time is not available,
/// so we use a compile-time fallback. In production this would query the OS.
/// For now we return a reasonable default (2025-01-01) that the framework
/// can override via state.
fn today_date() -> (u32, u32, u32) {
    // Default to 2025-06-15 as "today" for highlighting purposes.
    // In a real implementation this would use std::time::SystemTime.
    (15, 6, 2025)
}

/// A DatePicker component that displays a text field showing the selected date
/// and a calendar popover on click.
///
/// Supports single-date and range selection modes, month/year navigation,
/// and proper accessibility via AccessKit textbox role.
///
/// # Examples
/// ```
/// use cvkg_components::datepicker::{DatePicker, DatePickerMode};
/// let picker = DatePicker::new(|day, month, year| {
///     println!("Selected: {}/{}/{}", day, month, year);
/// })
/// .selected(15, 6, 2025);
/// ```
#[derive(Clone)]
pub struct DatePicker {
    /// The currently selected date as (day, month, year).
    selected_date: Option<(u32, u32, u32)>,
    /// Optional end date for range selection mode.
    range_end: Option<(u32, u32, u32)>,
    /// Whether the calendar popover is open.
    is_open: bool,
    /// Selection mode: single date or range.
    mode: DatePickerMode,
    /// In range mode, tracks whether the user is picking the start or end date.
    range_picking_end: bool,
    /// Callback invoked when a date is selected.
    on_change: Arc<dyn Fn(u32, u32, u32) + Send + Sync>,
    /// Optional callback for range selection: (start, end).
    on_range_change: Option<Arc<dyn Fn(DateRange) + Send + Sync>>,
    /// Stable per-instance hash used to identify this component in the system state store.
    id_hash: u64,
}

impl DatePicker {
    /// Create a new DatePicker with the given change callback.
    ///
    /// The picker defaults to no selected date, closed state, and Single mode.
    pub fn new(on_change: impl Fn(u32, u32, u32) + Send + Sync + 'static) -> Self {
        Self {
            selected_date: None,
            range_end: None,
            is_open: false,
            mode: DatePickerMode::Single,
            range_picking_end: false,
            on_change: Arc::new(on_change),
            on_range_change: None,
            id_hash: 0xE00_0000,
        }
    }

    /// Set the initially selected date (day, month, year).
    pub fn selected(mut self, day: u32, month: u32, year: u32) -> Self {
        self.selected_date = Some((day, month, year));
        self
    }

    /// Set the initially selected range. Only meaningful in Range mode.
    pub fn range(mut self, start: (u32, u32, u32), end: (u32, u32, u32)) -> Self {
        self.selected_date = Some(start);
        self.range_end = Some(end);
        self
    }

    /// Set the picker to Range selection mode with an optional range change callback.
    pub fn with_range_mode(
        mut self,
        on_range_change: impl Fn(DateRange) + Send + Sync + 'static,
    ) -> Self {
        self.mode = DatePickerMode::Range;
        self.on_range_change = Some(Arc::new(on_range_change));
        self
    }

    /// Read the open state from the system component state.
    fn is_open_state(&self) -> bool {
        let s = load_system_state();
        s.get_component_state::<bool>(self.id_hash)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or(false)
    }

    /// Write the open state into the system component state.
    pub fn set_open_state(&self, open: bool) {
        let id = self.id_hash;
        update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(id, open);
            s
        });
    }

    /// Read the currently displayed month from system state, defaulting to
    /// the selected date's month or today's month.
    fn displayed_month_state(&self) -> (u32, u32) {
        let s = load_system_state();
        s.get_component_state::<(u32, u32)>(self.id_hash + 1)
            .and_then(|v| v.read().ok().map(|g| *g))
            .unwrap_or_else(|| {
                if let Some((_d, m, y)) = self.selected_date {
                    (m, y)
                } else {
                    let (_d, m, y) = today_date();
                    (m, y)
                }
            })
    }

    /// Write the currently displayed month into system state.
    pub fn set_displayed_month(&self, month: u32, year: u32) {
        let id = self.id_hash + 1;
        update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(id, (month, year));
            s
        });
    }

    /// Format the selected date as DD/MM/YYYY, or return a placeholder.
    /// In range mode, formats as "DD/MM/YYYY – DD/MM/YYYY".
    fn format_date(&self) -> String {
        match self.mode {
            DatePickerMode::Range => {
                if let Some((sd, sm, sy)) = self.selected_date {
                    if let Some((ed, em, ey)) = self.range_end {
                        return format!("{:02}/{:02}/{} – {:02}/{:02}/{}", sd, sm, sy, ed, em, ey);
                    }
                    return format!("{:02}/{:02}/{} –", sd, sm, sy);
                }
                lingua_tong::t("datepicker.range_placeholder")
            }
            DatePickerMode::Single => {
                if let Some((day, month, year)) = self.selected_date {
                    format!("{:02}/{:02}/{}", day, month, year)
                } else {
                    lingua_tong::t("datepicker.format")
                }
            }
        }
    }

    /// Render the text field portion of the date picker.
    fn render_text_field(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DatePickerField");
        renderer.set_key(&format!("dp_field_{}", self.id_hash));
        renderer.set_aria_role("textbox");
        renderer.set_aria_label(&lingua_tong::t("datepicker.label"));

        // Background
        renderer.fill_rounded_rect(rect, RADIUS_MD, [0.06, 0.06, 0.08, 1.0]);
        // Border
        renderer.stroke_rounded_rect(rect, RADIUS_MD, [0.25, 0.25, 0.28, 1.0], 1.5);

        // Date text
        let text = self.format_date();
        let text_color = [1.0, 1.0, 1.0, 1.0];
        let text_x = rect.x + 10.0;
        let text_y = rect.y + (rect.height - 14.0) / 2.0;
        renderer.draw_text(&text, text_x, text_y, 14.0, text_color);

        // Calendar icon on the right
        let icon_x = rect.x + rect.width - 28.0;
        let icon_y = rect.y + (rect.height - 14.0) / 2.0;
        renderer.draw_text("\u{1F4C5}", icon_x, icon_y, 14.0, [1.0, 1.0, 1.0, 0.8]);

        renderer.pop_vnode();
    }

    /// Render the calendar popover with glassmorphic styling.
    fn render_calendar(&self, renderer: &mut dyn Renderer, anchor_rect: Rect) {
        let pop_w: f32 = 280.0;
        let pop_h: f32 = 260.0;
        let gap = 6.0;

        let pop_rect = Rect {
            x: anchor_rect.x,
            y: anchor_rect.y - pop_h - gap,
            width: pop_w,
            height: pop_h,
        };

        renderer.set_z_index(-900.0);

        // Semi-transparent backdrop behind the popover
        renderer.fill_rect(anchor_rect, theme::with_alpha(theme::bg(), 0.25));
        renderer.fill_rounded_rect(
            pop_rect,
            RADIUS_XL,
            [0.05, 0.05, 0.07, 1.0],
        );
        renderer.stroke_rounded_rect(
            pop_rect,
            RADIUS_XL,
            [0.25, 0.25, 0.28, 1.0],
            1.5,
        );

        let (display_month, display_year) = self.displayed_month_state();

        // Header: month/year with navigation arrows
        let header_h = 32.0;
        let header_rect = Rect {
            x: pop_rect.x,
            y: pop_rect.y,
            width: pop_w,
            height: header_h,
        };

        // Previous month button
        let prev_btn_rect = Rect {
            x: header_rect.x + 8.0,
            y: header_rect.y + 4.0,
            width: 24.0,
            height: 24.0,
        };
        renderer.draw_text(
            "<",
            prev_btn_rect.x + 6.0,
            prev_btn_rect.y + 4.0,
            14.0,
            theme::with_alpha(theme::accent(), 0.8),
        );

        // Month/year label (centered)
        let label = format!("{} {}", month_name(display_month), display_year);
        let (tw, _th) = renderer.measure_text(&label, 14.0);
        let label_x = header_rect.x + (header_rect.width - tw) / 2.0;
        let label_y = header_rect.y + (header_h - 14.0) / 2.0;
        renderer.draw_text(&label, label_x, label_y, 14.0, theme::text());

        // Next month button
        let next_btn_rect = Rect {
            x: header_rect.x + header_rect.width - 32.0,
            y: header_rect.y + 4.0,
            width: 24.0,
            height: 24.0,
        };
        renderer.draw_text(
            ">",
            next_btn_rect.x + 6.0,
            next_btn_rect.y + 4.0,
            14.0,
            theme::with_alpha(theme::accent(), 0.8),
        );

        // Day-of-week headers
        let grid_y_start = pop_rect.y + header_h + 4.0;
        let cell_w = pop_w / 7.0;
        let cell_h = 28.0;

        let day_hdrs = day_headers();
        for (i, day_name) in day_hdrs.iter().enumerate() {
            let cx = pop_rect.x + i as f32 * cell_w + cell_w / 2.0;
            let (tw, _th) = renderer.measure_text(day_name, 11.0);
            renderer.draw_text(
                day_name,
                cx - tw / 2.0,
                grid_y_start + (cell_h - 11.0) / 2.0,
                11.0,
                theme::text_muted(),
            );
        }

        // Separator line below headers
        let sep_y = grid_y_start + cell_h;
        renderer.draw_line(
            pop_rect.x + 8.0,
            sep_y,
            pop_rect.x + pop_w - 8.0,
            sep_y,
            theme::with_alpha(theme::border(), 0.5),
            1.0,
        );

        // Compute calendar grid
        let first_dow = day_of_week(display_year, display_month, 1);
        let num_days = days_in_month(display_month, display_year);
        let prev_month = if display_month == 1 {
            12
        } else {
            display_month - 1
        };
        let prev_year = if display_month == 1 {
            display_year - 1
        } else {
            display_year
        };
        let prev_month_days = days_in_month(prev_month, prev_year);

        let today = today_date();

        // Day grid: up to 6 rows
        let grid_start_y = sep_y + 4.0;
        let total_cells = first_dow as usize + num_days as usize;
        let num_rows = total_cells.div_ceil(7).min(6);

        for row in 0..num_rows {
            for col in 0..7 {
                let cell_idx = row * 7 + col;
                let cell_x = pop_rect.x + col as f32 * cell_w;
                let cell_y = grid_start_y + row as f32 * cell_h;
                let _cell_rect = Rect {
                    x: cell_x,
                    y: cell_y,
                    width: cell_w,
                    height: cell_h,
                };

                let day_num: u32;
                let is_current_month: bool;

                if cell_idx < first_dow as usize {
                    // Previous month days
                    day_num = prev_month_days - (first_dow - cell_idx as u32) + 1;
                    is_current_month = false;
                } else {
                    let d = cell_idx as u32 - first_dow + 1;
                    if d <= num_days {
                        day_num = d;
                        is_current_month = true;
                    } else {
                        // Next month days
                        day_num = d - num_days;
                        is_current_month = false;
                    }
                }

                if !is_current_month {
                    // Dimmed days from adjacent months
                    let day_str = format!("{}", day_num);
                    let (tw, _th) = renderer.measure_text(&day_str, 12.0);
                    renderer.draw_text(
                        &day_str,
                        cell_x + (cell_w - tw) / 2.0,
                        cell_y + (cell_h - 12.0) / 2.0,
                        12.0,
                        theme::with_alpha(theme::text_muted(), 0.5),
                    );
                } else {
                    let is_selected = self
                        .selected_date
                        .map(|(sd, sm, sy)| {
                            sd == day_num && sm == display_month && sy == display_year
                        })
                        .unwrap_or(false);
                    let is_range_end = self
                        .range_end
                        .map(|(ed, em, ey)| {
                            ed == day_num && em == display_month && ey == display_year
                        })
                        .unwrap_or(false);
                    let is_in_range = match (self.selected_date, self.range_end) {
                        (Some((sd, sm, sy)), Some((ed, em, ey))) => {
                            let date_val = |d: u32, m: u32, y: u32| -> u64 {
                                y as u64 * 10000 + m as u64 * 100 + d as u64
                            };
                            let v = date_val(day_num, display_month, display_year);
                            let sv = date_val(sd, sm, sy);
                            let ev = date_val(ed, em, ey);
                            self.mode == DatePickerMode::Range && v >= sv && v <= ev
                        }
                        _ => false,
                    };
                    let is_today =
                        day_num == today.0 && display_month == today.1 && display_year == today.2;

                    // Highlight selected day, range end, or in-range days
                    if is_selected {
                        let highlight_rect = Rect {
                            x: cell_x + (cell_w - 24.0) / 2.0,
                            y: cell_y + (cell_h - 24.0) / 2.0,
                            width: 24.0,
                            height: 24.0,
                        };
                        renderer.fill_rounded_rect(
                            highlight_rect,
                            RADIUS_XL,
                            theme::with_alpha(theme::primary(), 0.9),
                        );
                    } else if is_range_end {
                        let highlight_rect = Rect {
                            x: cell_x + (cell_w - 24.0) / 2.0,
                            y: cell_y + (cell_h - 24.0) / 2.0,
                            width: 24.0,
                            height: 24.0,
                        };
                        renderer.fill_rounded_rect(
                            highlight_rect,
                            RADIUS_XL,
                            theme::with_alpha(theme::accent(), 0.85),
                        );
                    } else if is_in_range {
                        // Subtle range highlight
                        let range_rect = Rect {
                            x: cell_x + 2.0,
                            y: cell_y + (cell_h - 20.0) / 2.0,
                            width: cell_w - 4.0,
                            height: 20.0,
                        };
                        renderer.fill_rounded_rect(
                            range_rect,
                            RADIUS_SM,
                            theme::with_alpha(theme::primary(), 0.15),
                        );
                    } else if is_today {
                        let highlight_rect = Rect {
                            x: cell_x + (cell_w - 24.0) / 2.0,
                            y: cell_y + (cell_h - 24.0) / 2.0,
                            width: 24.0,
                            height: 24.0,
                        };
                        renderer.stroke_rounded_rect(
                            highlight_rect,
                            RADIUS_XL,
                            theme::with_alpha(theme::primary(), 0.6),
                            1.5,
                        );
                    }

                    let day_str = format!("{}", day_num);
                    let (tw, _th) = renderer.measure_text(&day_str, 12.0);
                    let text_color = if is_selected || is_range_end {
                        theme::text()
                    } else if is_in_range || is_today {
                        theme::accent()
                    } else {
                        theme::text()
                    };
                    renderer.draw_text(
                        &day_str,
                        cell_x + (cell_w - tw) / 2.0,
                        cell_y + (cell_h - 12.0) / 2.0,
                        12.0,
                        text_color,
                    );
                }
            }
        }

        // Register click handlers
        let id = self.id_hash;
        let on_change = self.on_change.clone();
        let pr = pop_rect;
        let ar = anchor_rect;

        // Previous month button click
        let prev_r = prev_btn_rect;
        let dm = display_month;
        let dy = display_year;
        renderer.push_vnode(prev_btn_rect, "DatePickerPrev");
        renderer.set_key(&format!("dp_prev_{}", id));
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event: Event| {
                if let Event::PointerClick { x, y, .. } = event {
                    log::info!("[DatePicker] Prev Month click at ({}, {}), prev_r={:?}", x, y, prev_r);
                    let (new_m, new_y) = if dm == 1 { (12, dy - 1) } else { (dm - 1, dy) };
                    update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(id + 1, (new_m, new_y));
                        s
                    });
                }
            }),
        );
        renderer.pop_vnode();

        // Next month button click
        let next_r = next_btn_rect;
        let dm2 = display_month;
        let dy2 = display_year;
        renderer.push_vnode(next_btn_rect, "DatePickerNext");
        renderer.set_key(&format!("dp_next_{}", id));
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event: Event| {
                if let Event::PointerClick { x, y, .. } = event {
                    log::info!("[DatePicker] Next Month click at ({}, {}), next_r={:?}", x, y, next_r);
                    let (new_m, new_y) = if dm2 == 12 {
                        (1, dy2 + 1)
                    } else {
                        (dm2 + 1, dy2)
                    };
                    update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(id + 1, (new_m, new_y));
                        s
                    });
                }
            }),
        );
        renderer.pop_vnode();

        // Day cell clicks
        let mode = self.mode;
        let range_picking_end = self.range_picking_end;
        let on_range_change = self.on_range_change.clone();
        for row in 0..num_rows {
            for col in 0..7 {
                let cell_idx = row * 7 + col;
                let cell_x = pop_rect.x + col as f32 * cell_w;
                let cell_y = grid_start_y + row as f32 * cell_h;
                let cell_rect = Rect {
                    x: cell_x,
                    y: cell_y,
                    width: cell_w,
                    height: cell_h,
                };

                if cell_idx >= first_dow as usize {
                    let d = cell_idx as u32 - first_dow + 1;
                    if d <= num_days {
                        let oc = on_change.clone();
                        let id2 = id;
                        let orc = on_range_change.clone();
                        renderer.push_vnode(cell_rect, "DatePickerDay");
                        renderer.set_key(&format!("dp_day_{}_{}", id, d));
                        renderer.register_handler(
                            "pointerclick",
                            Arc::new(move |event: Event| {
                                if let Event::PointerClick { x, y, .. } = event {
                                    log::info!("[DatePicker] Day {} click at ({}, {}), cell_rect={:?}", d, x, y, cell_rect);
                                    (oc)(d, display_month, display_year);
                                    if mode == DatePickerMode::Range {
                                        // In range mode, toggle between picking start and end
                                        if !range_picking_end {
                                            // First click: set start, prepare for end
                                            update_system_state(move |s| {
                                                let mut s = s.clone();
                                                s.set_component_state(id2 + 2, true); // range_picking_end = true
                                                s.set_component_state(id2, false);
                                                s
                                            });
                                        } else {
                                            // Second click: set end, fire range callback, close
                                            if let Some(ref cb) = orc {
                                                let start: (u32, u32, u32) = {
                                                    let s = load_system_state();
                                                    s.get_component_state::<(u32, u32, u32)>(
                                                        id2 + 3,
                                                    )
                                                    .and_then(|v| v.read().ok().map(|g| *g))
                                                    .unwrap_or((d, display_month, display_year))
                                                };
                                                let range = DateRange {
                                                    start,
                                                    end: (d, display_month, display_year),
                                                };
                                                (cb)(range);
                                            }
                                            update_system_state(move |s| {
                                                let mut s = s.clone();
                                                s.set_component_state(id2 + 2, false); // reset range_picking_end
                                                s.set_component_state(id2, false); // close
                                                s
                                            });
                                        }
                                    } else {
                                        // Single mode: close after selection
                                        update_system_state(move |s| {
                                            let mut s = s.clone();
                                            s.set_component_state(id2, false);
                                            s
                                        });
                                    }
                                }
                            }),
                        );
                        renderer.pop_vnode();
                    }
                }
            }
        }

        // Click outside to close
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event: Event| {
                if let Event::PointerClick { x, y, .. } = event
                    && !pr.contains(x, y)
                    && !ar.contains(x, y)
                {
                    update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(id, false);
                        s
                    });
                }
            }),
        );

        renderer.set_z_index(0.0);
    }
}

impl View for DatePicker {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::layout::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(220.0),
            height: 38.0,
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let is_open = self.is_open_state() || self.is_open;

        // Calculate combined bounding box for input field + popped up calendar above it
        let combined_rect = if is_open {
            let pop_w: f32 = 280.0;
            let pop_h: f32 = 260.0;
            let gap = 6.0;
            Rect {
                x: rect.x,
                y: rect.y - pop_h - gap,
                width: rect.width.max(pop_w),
                height: rect.height + pop_h + gap,
            }
        } else {
            rect
        };

        renderer.push_vnode(combined_rect, "DatePicker");
        renderer.set_key(&format!("dp_main_{}", self.id_hash));

        // Render the text field
        self.render_text_field(renderer, rect);

        // Register click handler on the text field to toggle the popover
        let id = self.id_hash;
        let tr = rect;
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event: Event| {
                if let Event::PointerClick { x, y, .. } = event
                    && tr.contains(x, y)
                {
                    let current = {
                        let s = load_system_state();
                        s.get_component_state::<bool>(id)
                            .and_then(|v| v.read().ok().map(|g| *g))
                            .unwrap_or(false)
                    };
                    update_system_state(move |s| {
                        let mut s = s.clone();
                        s.set_component_state(id, !current);
                        s
                    });
                }
            }),
        );

        // Render calendar popover if open
        if is_open {
            self.render_calendar(renderer, rect);
        }

        renderer.pop_vnode();
    }
}

impl cvkg_core::layout::LayoutView for DatePicker {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::layout::SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(220.0),
            height: 38.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
    }
}

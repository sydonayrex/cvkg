use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Scheduler event item representing a scheduled block of time.
#[derive(Debug, Clone)]
pub struct SchedulerEvent {
    /// Title of the event.
    pub title: String,
    /// Starting hour (0.0 to 24.0).
    pub start_hour: f32,
    /// Duration of the event in hours.
    pub duration_hours: f32,
    /// Optional custom background color override.
    pub color: Option<[f32; 4]>,
}

/// Calendar-style Scheduler component for resource scheduling.
///
/// # Contract
/// - Start and duration times should fit within the 24-hour daily boundary.
pub struct Scheduler {
    pub(crate) events: Vec<SchedulerEvent>,
    pub(crate) day_start: f32,
    pub(crate) day_end: f32,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Create a new empty Scheduler component.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            day_start: 8.0,
            day_end: 18.0,
        }
    }

    /// Set bounds of the visible schedule day.
    pub fn day_bounds(mut self, start: f32, end: f32) -> Self {
        self.day_start = start;
        self.day_end = end;
        self
    }

    /// Add a scheduled event.
    pub fn event(mut self, title: &str, start: f32, duration: f32) -> Self {
        self.events.push(SchedulerEvent {
            title: title.to_string(),
            start_hour: start,
            duration_hours: duration,
            color: None,
        });
        self
    }

    /// Add a scheduled event with a custom color.
    pub fn event_with_color(
        mut self,
        title: &str,
        start: f32,
        duration: f32,
        color: [f32; 4],
    ) -> Self {
        self.events.push(SchedulerEvent {
            title: title.to_string(),
            start_hour: start,
            duration_hours: duration,
            color: Some(color),
        });
        self
    }
}

impl View for Scheduler {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Draw container border
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);

        let sidebar_w = 60.0;
        let timeline_w = rect.width - sidebar_w - 10.0;
        let day_range = (self.day_end - self.day_start).max(1.0);
        let hour_w = timeline_w / day_range;

        // Draw hour grid lines and labels
        let hour_count = day_range as usize;
        for i in 0..=hour_count {
            let hour = self.day_start + i as f32;
            let x = rect.x + sidebar_w + i as f32 * hour_w;

            // Hour grid divider line
            renderer.draw_line(
                x,
                rect.y,
                x,
                rect.y + rect.height,
                [
                    theme::border()[0],
                    theme::border()[1],
                    theme::border()[2],
                    0.2,
                ],
                1.0,
            );

            // Time label
            renderer.draw_text(
                &format!("{:02}:00", hour as i32),
                x - 15.0,
                rect.y + 16.0,
                10.0,
                theme::text_muted(),
            );
        }

        // Render events
        let event_h = 40.0;
        let event_padding = 4.0;
        for (i, ev) in self.events.iter().enumerate() {
            if ev.start_hour + ev.duration_hours < self.day_start || ev.start_hour > self.day_end {
                continue; // Out of range
            }

            let start_clamped = ev.start_hour.max(self.day_start);
            let end_clamped = (ev.start_hour + ev.duration_hours).min(self.day_end);
            let duration_clamped = end_clamped - start_clamped;

            let ev_x =
                rect.x + sidebar_w + (start_clamped - self.day_start) * hour_w + event_padding;
            let ev_w = (duration_clamped * hour_w - event_padding * 2.0).max(10.0);
            let ev_y = rect.y + 40.0 + i as f32 * (event_h + event_padding);

            let bg_color = ev.color.unwrap_or(theme::accent());
            let border_color = [bg_color[0] * 1.2, bg_color[1] * 1.2, bg_color[2] * 1.2, 0.8];

            let ev_rect = Rect {
                x: ev_x,
                y: ev_y,
                width: ev_w,
                height: event_h,
            };

            renderer.fill_rounded_rect(ev_rect, 4.0, bg_color);
            renderer.stroke_rounded_rect(ev_rect, 4.0, border_color, 1.0);

            // Draw event title
            renderer.draw_text(
                &ev.title,
                ev_x + 8.0,
                ev_y + event_h / 2.0 - 4.0,
                11.0,
                [1.0, 1.0, 1.0, 0.95],
            );
        }
    }
}

impl LayoutView for Scheduler {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let count = self.events.len();
        let height = 40.0 + (count as f32 * 44.0) + 20.0;
        Size {
            width: 600.0,
            height: height.max(200.0),
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

/// Gantt project management timeline task.
#[derive(Debug, Clone)]
pub struct GanttTask {
    /// Name of the task.
    pub name: String,
    /// Starting progress duration offset (e.g. days).
    pub start_day: f32,
    /// Total duration of the task in days.
    pub duration_days: f32,
    /// Completion progress from 0.0 to 1.0.
    pub progress: f32,
}

/// Gantt project timeline rendering component.
pub struct Gantt {
    pub(crate) tasks: Vec<GanttTask>,
    pub(crate) total_days: f32,
}

impl Default for Gantt {
    fn default() -> Self {
        Self::new()
    }
}

impl Gantt {
    /// Create a new empty Gantt component.
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            total_days: 30.0,
        }
    }

    /// Set total timeline range.
    pub fn total_days(mut self, days: f32) -> Self {
        self.total_days = days;
        self
    }

    /// Add a task to the Gantt chart.
    pub fn task(mut self, name: &str, start: f32, duration: f32, progress: f32) -> Self {
        self.tasks.push(GanttTask {
            name: name.to_string(),
            start_day: start,
            duration_days: duration,
            progress: progress.clamp(0.0, 1.0),
        });
        self
    }
}

impl View for Gantt {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Container background
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);

        let label_w = 120.0;
        let timeline_w = rect.width - label_w - 20.0;
        let day_w = timeline_w / self.total_days.max(1.0);

        // Header timeline markers
        let marker_step = if self.total_days > 15.0 { 5.0 } else { 1.0 };
        let mut d = 0.0f32;
        while d <= self.total_days {
            let x = rect.x + label_w + d * day_w;
            renderer.draw_line(
                x,
                rect.y,
                x,
                rect.y + rect.height,
                [
                    theme::border()[0],
                    theme::border()[1],
                    theme::border()[2],
                    0.15,
                ],
                1.0,
            );
            renderer.draw_text(
                &format!("Day {:.0}", d),
                x + 4.0,
                rect.y + 16.0,
                9.0,
                theme::text_muted(),
            );
            d += marker_step;
        }

        // Draw tasks
        let row_h = 32.0;
        let bar_h = 16.0;

        for (i, task) in self.tasks.iter().enumerate() {
            let y = rect.y + 40.0 + i as f32 * row_h;

            // Draw task name label
            renderer.draw_text(
                &task.name,
                rect.x + 10.0,
                y + row_h / 2.0 - 4.0,
                11.0,
                theme::text(),
            );

            // Draw task total range bar
            let bar_x = rect.x + label_w + task.start_day * day_w;
            let bar_w = task.duration_days * day_w;
            let bar_rect = Rect {
                x: bar_x,
                y: y + (row_h - bar_h) / 2.0,
                width: bar_w,
                height: bar_h,
            };

            renderer.fill_rounded_rect(bar_rect, 3.0, theme::surface_elevated());
            renderer.stroke_rounded_rect(bar_rect, 3.0, theme::border(), 0.5);

            // Draw progress bar inside
            let progress_w = bar_w * task.progress;
            if progress_w > 0.0 {
                let progress_rect = Rect {
                    x: bar_x,
                    y: y + (row_h - bar_h) / 2.0,
                    width: progress_w,
                    height: bar_h,
                };
                renderer.fill_rounded_rect(progress_rect, 3.0, theme::success());
            }
        }
    }
}

impl LayoutView for Gantt {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let count = self.tasks.len();
        let height = 40.0 + (count as f32 * 32.0) + 20.0;
        Size {
            width: 600.0,
            height: height.max(200.0),
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

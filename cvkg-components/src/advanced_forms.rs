use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

/// Date picker component
pub struct DatePicker {
    pub(crate) selected_date: Option<(u32, u32, u32)>, // year, month, day
    pub(crate) on_date_change: Option<Arc<dyn Fn(Option<(u32, u32, u32)>) + Send + Sync>>,
}

impl Default for DatePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl DatePicker {
    pub fn new() -> Self {
        Self {
            selected_date: None,
            on_date_change: None,
        }
    }

    pub fn date(mut self, date: (u32, u32, u32)) -> Self {
        self.selected_date = Some(date);
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(Option<(u32, u32, u32)>) + Send + Sync + 'static,
    ) -> Self {
        self.on_date_change = Some(Arc::new(f));
        self
    }
}

impl View for DatePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let text = match self.selected_date {
            Some((y, m, d)) => format!("{}/{}/{}", m, d, y),
            None => "Select date".to_string(),
        };

        let (tw, _) = renderer.measure_text(&text, 13.0);
        renderer.draw_text(
            &text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            [0.8, 0.8, 0.9, 1.0],
        );

        // Calendar icon
        renderer.draw_text(
            "📅",
            rect.x + 8.0,
            rect.y + 10.0,
            14.0,
            [0.6, 0.8, 1.0, 1.0],
        );
    }
}

impl LayoutView for DatePicker {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 160.0,
            height: 36.0,
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

/// Time picker component
pub struct TimePicker {
    pub(crate) selected_time: Option<(u32, u32)>, // hour, minute
    pub(crate) on_time_change: Option<Arc<dyn Fn(Option<(u32, u32)>) + Send + Sync>>,
}

impl Default for TimePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl TimePicker {
    pub fn new() -> Self {
        Self {
            selected_time: None,
            on_time_change: None,
        }
    }

    pub fn time(mut self, time: (u32, u32)) -> Self {
        self.selected_time = Some(time);
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<(u32, u32)>) + Send + Sync + 'static) -> Self {
        self.on_time_change = Some(Arc::new(f));
        self
    }
}

impl View for TimePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let text = match self.selected_time {
            Some((h, m)) => format!("{h:02}:{m:02}"),
            None => "Select time".to_string(),
        };

        let (tw, _) = renderer.measure_text(&text, 13.0);
        renderer.draw_text(
            &text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            [0.8, 0.8, 0.9, 1.0],
        );

        renderer.draw_text(
            "🕐",
            rect.x + 8.0,
            rect.y + 10.0,
            14.0,
            [0.6, 0.8, 1.0, 1.0],
        );
    }
}

impl LayoutView for TimePicker {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 140.0,
            height: 36.0,
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

/// Calendar component
pub struct Calendar {
    pub(crate) current_month: u32,
    pub(crate) current_year: u32,
    pub(crate) _selected_date: Option<(u32, u32, u32)>,
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

impl Calendar {
    pub fn new() -> Self {
        Self {
            current_month: 5,
            current_year: 2026,
            _selected_date: None,
        }
    }

    pub fn month(mut self, month: u32) -> Self {
        self.current_month = month;
        self
    }

    pub fn year(mut self, year: u32) -> Self {
        self.current_year = year;
        self
    }
}

impl View for Calendar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let header_h = 32.0;
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };
        renderer.fill_rounded_rect(header_rect, 6.0, [0.1, 0.15, 0.2, 1.0]);
        renderer.draw_text(
            &format!("{} {}", self.current_month, self.current_year),
            header_rect.x + 12.0,
            header_rect.y + 10.0,
            14.0,
            [0.9, 0.95, 1.0, 1.0],
        );

        // Day headers
        let day_headers = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let grid_w = (rect.width - 24.0) / 7.0;
        let start_y = header_rect.y + header_h + 8.0;

        for (i, day) in day_headers.iter().enumerate() {
            let day_rect = Rect {
                x: rect.x + 12.0 + i as f32 * grid_w,
                y: start_y,
                width: grid_w,
                height: 24.0,
            };
            renderer.draw_text(
                day,
                day_rect.x + 4.0,
                day_rect.y + 6.0,
                11.0,
                [0.5, 0.5, 0.6, 1.0],
            );
        }

        // Render days (simplified - just first week)
        let day_start_y = start_y + 28.0;
        for i in 1..=7 {
            let day_rect = Rect {
                x: rect.x + 12.0 + ((i - 1) as f32 % 7.0) * grid_w,
                y: day_start_y + ((i - 1) as f32 / 7.0) * 32.0,
                width: grid_w,
                height: 32.0,
            };
            renderer.fill_rounded_rect(day_rect, 4.0, [0.06, 0.06, 0.1, 1.0]);
            renderer.draw_text(
                &i.to_string(),
                day_rect.x + 4.0,
                day_rect.y + 10.0,
                12.0,
                [0.7, 0.7, 0.8, 1.0],
            );
        }
    }
}

impl LayoutView for Calendar {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 280.0,
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

/// Autocomplete component
pub struct Autocomplete {
    pub(crate) suggestions: Vec<String>,
    pub(crate) text: String,
    pub(crate) on_select: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

impl Autocomplete {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            text: String::new(),
            on_select: None,
        }
    }

    pub fn suggestions(mut self, items: Vec<&str>) -> Self {
        self.suggestions = items.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl View for Autocomplete {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let (_tw, _) = renderer.measure_text(&self.text, 13.0);
        renderer.draw_text(
            &self.text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            [0.8, 0.8, 0.9, 1.0],
        );

        // Show matching suggestions
        let filtered: Vec<_> = self
            .suggestions
            .iter()
            .filter(|s| s.to_lowercase().contains(&self.text.to_lowercase()))
            .take(5)
            .collect();

        if !filtered.is_empty() && rect.height > 40.0 {
            let sug_y = rect.y + rect.height + 4.0;
            for (i, sug) in filtered.iter().enumerate() {
                let sug_rect = Rect {
                    x: rect.x,
                    y: sug_y + i as f32 * 28.0,
                    width: rect.width,
                    height: 28.0,
                };
                renderer.fill_rounded_rect(sug_rect, 4.0, [0.06, 0.08, 0.12, 1.0]);
                renderer.draw_text(
                    sug,
                    sug_rect.x + 8.0,
                    sug_rect.y + 10.0,
                    12.0,
                    [0.6, 0.7, 0.8, 1.0],
                );
            }
        }
    }
}

impl LayoutView for Autocomplete {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 200.0,
            height: 36.0,
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

/// Combobox component
pub struct Combobox {
    pub(crate) options: Vec<String>,
    pub(crate) selected: Option<String>,
    pub(crate) on_change: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
}

impl Default for Combobox {
    fn default() -> Self {
        Self::new()
    }
}

impl Combobox {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            on_change: None,
        }
    }

    pub fn options(mut self, items: Vec<&str>) -> Self {
        self.options = items.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn selected(mut self, value: &str) -> Self {
        self.selected = Some(value.to_string());
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<String>) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }
}

impl View for Combobox {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let display = match &self.selected {
            Some(v) => v.clone(),
            None => "Select...".to_string(),
        };

        renderer.draw_text(
            &display,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            [0.8, 0.8, 0.9, 1.0],
        );
        renderer.draw_text(
            "▼",
            rect.x + rect.width - 20.0,
            rect.y + 10.0,
            12.0,
            [0.5, 0.5, 0.6, 1.0],
        );
    }
}

impl LayoutView for Combobox {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 180.0,
            height: 36.0,
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

/// MultiSelect component
pub struct MultiSelect {
    pub(crate) options: Vec<String>,
    pub(crate) selected: Vec<String>,
}

impl Default for MultiSelect {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSelect {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: Vec::new(),
        }
    }

    pub fn options(mut self, items: Vec<&str>) -> Self {
        self.options = items.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn selected(mut self, values: Vec<&str>) -> Self {
        self.selected = values.into_iter().map(|s| s.to_string()).collect();
        self
    }
}

impl View for MultiSelect {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let selected_display: Vec<_> = self
            .selected
            .iter()
            .filter(|s| self.options.contains(s))
            .cloned()
            .collect();

        let text = if selected_display.is_empty() {
            "Select multiple...".to_string()
        } else {
            selected_display.join(", ")
        };

        renderer.draw_text(
            &text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            [0.8, 0.8, 0.9, 1.0],
        );
    }
}

impl LayoutView for MultiSelect {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 200.0,
            height: 36.0,
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

/// TagInput component
pub struct TagInput {
    pub(crate) tags: Vec<String>,
    pub(crate) _text: String,
}

impl Default for TagInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TagInput {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            _text: String::new(),
        }
    }

    pub fn tags(mut self, items: Vec<&str>) -> Self {
        self.tags = items.into_iter().map(|s| s.to_string()).collect();
        self
    }
}

impl View for TagInput {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let bg = [0.08, 0.08, 0.12, 1.0];
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let mut current_x = rect.x + 8.0;
        for tag in &self.tags {
            let tag_w = renderer.measure_text(tag, 11.0).0 + 12.0;
            let tag_rect = Rect {
                x: current_x,
                y: rect.y + 6.0,
                width: tag_w,
                height: rect.height - 12.0,
            };
            renderer.fill_rounded_rect(tag_rect, 4.0, [0.1, 0.2, 0.4, 1.0]);
            renderer.draw_text(
                tag,
                current_x + 4.0,
                rect.y + 12.0,
                11.0,
                [0.9, 0.95, 1.0, 1.0],
            );
            current_x += tag_w + 6.0;
        }
    }
}

impl LayoutView for TagInput {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = (self.tags.len() as f32 * 60.0 + 80.0).max(200.0);
        Size {
            width,
            height: 36.0,
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

/// Validation framework
#[derive(Clone)]
pub struct ValidationRule {
    pub field: String,
    pub validator: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    pub error_message: String,
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRule")
            .field("field", &self.field)
            .field("error_message", &self.error_message)
            .finish()
    }
}

/// EikonaForm - A schema-based form validation system.
/// Named after the hybrid concept of "form/image" (Eikona).
pub struct EikonaForm {
    pub(crate) rules: Vec<ValidationRule>,
    pub(crate) _errors: Vec<String>,
}

impl EikonaForm {
    /// Creates a new EikonaForm.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            _errors: Vec::new(),
        }
    }

    /// Adds a validation rule to the form.
    pub fn rule<F>(mut self, field: &str, validator: F, error: &str) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.rules.push(ValidationRule {
            field: field.to_string(),
            validator: Arc::new(validator),
            error_message: error.to_string(),
        });
        self
    }

    pub fn validate(&self, fields: &std::collections::HashMap<String, String>) -> Vec<String> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            if let Some(value) = fields.get(&rule.field)
                && !(rule.validator)(value)
            {
                errors.push(rule.error_message.clone());
            }
        }
        errors
    }
}

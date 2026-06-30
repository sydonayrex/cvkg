use crate::theme;
use cvkg_core::{
    Event, Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::sync::Arc;

/// Date picker component
#[deprecated(note = "Use datepicker::DatePicker instead")]
pub use crate::datepicker::DatePicker;

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
        let bg = theme::surface_elevated();
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let text = match self.selected_time {
            Some((h, m)) => format!("{h:02}:{m:02}"),
            None => "Select time".to_string(),
        };

        let (tw, _) = renderer.measure_text(&text, 13.0);
        renderer.draw_text_raw(
            &text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            theme::text(),
        );

        renderer.draw_text_raw("🕐", rect.x + 8.0, rect.y + 10.0, 14.0, theme::info());
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
        renderer.push_vnode(rect, "Calendar");
        renderer.set_aria_role("application");
        renderer.set_aria_label("Calendar");

        let header_h = 32.0;
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: header_h,
        };
        renderer.fill_rounded_rect(header_rect, 6.0, theme::surface_elevated());
        renderer.draw_text_raw(
            &format!("{} {}", self.current_month, self.current_year),
            header_rect.x + 12.0,
            header_rect.y + 10.0,
            14.0,
            theme::text(),
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
            renderer.draw_text_raw(
                day,
                day_rect.x + 4.0,
                day_rect.y + 6.0,
                11.0,
                theme::text_muted(),
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
            renderer.fill_rounded_rect(day_rect, 4.0, theme::input_bg());
            renderer.draw_text_raw(
                &i.to_string(),
                day_rect.x + 4.0,
                day_rect.y + 10.0,
                12.0,
                theme::text_muted(),
            );
        }

        // Keyboard: Arrow keys to navigate days, Enter to select
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Enter" | " " => {
                            // Calendar day navigation handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
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
        let bg = theme::surface_elevated();
        renderer.fill_rounded_rect(rect, 6.0, bg);

        let (_tw, _) = renderer.measure_text(&self.text, 13.0);
        renderer.draw_text_raw(
            &self.text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            13.0,
            theme::text(),
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
                renderer.fill_rounded_rect(sug_rect, 4.0, theme::surface_elevated());
                renderer.draw_text_raw(
                    sug,
                    sug_rect.x + 8.0,
                    sug_rect.y + 10.0,
                    12.0,
                    theme::text_muted(),
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
#[deprecated(note = "Use combobox::Combobox instead")]
pub use crate::combobox::Combobox;

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
        renderer.push_vnode(rect, "MultiSelect");
        renderer.set_aria_role("listbox");
        renderer.set_aria_label("Multi-select");
        let bg = theme::surface_elevated();
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

        renderer.draw_text_raw(
            &text,
            rect.x + 8.0,
            rect.y + (rect.height - 14.0) / 2.0,
            12.0,
            theme::text(),
        );

        // Keyboard: ArrowDown to open, Enter/Escape to toggle
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowDown" | "Enter" | " " | "Escape" => {
                            // MultiSelect open/toggle handled by parent via state
                        }
                        _ => {}
                    }
                }
            }),
        );

        renderer.pop_vnode();
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
        let bg = theme::surface_elevated();
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
            renderer.fill_rounded_rect(tag_rect, 4.0, theme::primary());
            renderer.draw_text_raw(tag, current_x + 4.0, rect.y + 12.0, 11.0, theme::text());
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

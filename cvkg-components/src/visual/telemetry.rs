use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// TelemetryView - A real-time performance telemetry display.
#[derive(Clone)]
pub struct TelemetryView {
    pub label: String,
    pub value: f32,
    pub max_value: f32,
    pub unit: String,
    pub color: [f32; 4],
}

impl TelemetryView {
    pub fn new(label: &str, max_value: f32) -> Self {
        Self {
            label: label.to_string(),
            value: 0.0,
            max_value,
            unit: String::new(),
            color: theme::accent(),
        }
    }

    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    pub fn unit(mut self, u: &str) -> Self {
        self.unit = u.to_string();
        self
    }

    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
}

impl View for TelemetryView {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 4.0, theme::surface());
        let progress = (self.value / self.max_value).clamp(0.0, 1.0);
        let fill_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width * progress,
            height: rect.height,
        };
        renderer.fill_rounded_rect(fill_rect, 4.0, self.color);
        let text = format!("{}: {:.1}{}", self.label, self.value, self.unit);
        let (tw, th) = renderer.measure_text(&text, 10.0);
        renderer.draw_text_raw(
            &text,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + (rect.height - th) / 2.0,
            10.0,
            theme::text(),
        );
    }
}

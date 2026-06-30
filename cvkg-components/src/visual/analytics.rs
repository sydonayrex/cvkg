use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// Chart types for tactical visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Radar,
}

/// ValkyrieAnalytics - A real-time performance telemetry display with tactical aesthetics.
#[doc(alias = "Analytics")]
#[derive(Clone)]
pub struct ValkyrieAnalytics {
    pub title: String,
    pub data: Vec<(String, f32)>,
    pub chart_type: ChartType,
    pub width: f32,
    pub height: f32,
}

impl ValkyrieAnalytics {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            data: Vec::new(),
            chart_type: ChartType::Bar,
            width: 200.0,
            height: 120.0,
        }
    }

    pub fn data(mut self, data: Vec<(String, f32)>) -> Self {
        self.data = data;
        self
    }

    pub fn chart_type(mut self, ct: ChartType) -> Self {
        self.chart_type = ct;
        self
    }
}

impl View for ValkyrieAnalytics {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        let (_tw, _th) = renderer.measure_text(&self.title, 12.0);
        renderer.draw_text_raw(&self.title, rect.x + 8.0, rect.y + 8.0, 12.0, theme::text());
        // Simplified chart rendering
        for (i, (label, value)) in self.data.iter().enumerate() {
            let bar_h = value * (rect.height - 40.0);
            let bar_rect = Rect {
                x: rect.x + 8.0 + i as f32 * 30.0,
                y: rect.y + rect.height - bar_h - 8.0,
                width: 24.0,
                height: bar_h,
            };
            renderer.fill_rounded_rect(bar_rect, 2.0, theme::accent());
            let (lw, lh) = renderer.measure_text(label, 8.0);
            renderer.draw_text_raw(
                label,
                bar_rect.x + (24.0 - lw) / 2.0,
                bar_rect.y - lh - 2.0,
                8.0,
                theme::text_dim(),
            );
        }
    }
}

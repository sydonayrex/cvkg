use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

pub struct GaugeChart {
    pub(crate) value: f32,
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) label: String,
}

impl Default for GaugeChart {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            label: String::new(),
        }
    }
}

impl GaugeChart {
    /// Create a new empty GaugeChart.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set gauge parameters.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Set boundary metrics.
    pub fn bounds(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set descriptive label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl View for GaugeChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width.min(rect.height) / 2.0 - 20.0).max(15.0);

        // Draw background arc
        renderer.stroke_ellipse(
            Rect {
                x: center_x - radius,
                y: center_y - radius,
                width: radius * 2.0,
                height: radius * 2.0,
            },
            theme::surface_elevated(),
            10.0,
        );

        // Draw indicator needle based on current progress
        let pct = ((self.value - self.min) / (self.max - self.min).max(0.001)).clamp(0.0, 1.0);
        let angle = -std::f32::consts::PI + pct * std::f32::consts::PI; // Top semi-circle

        let needle_x = center_x + (radius - 5.0) * angle.cos();
        let needle_y = center_y + (radius - 5.0) * angle.sin();

        renderer.draw_line(center_x, center_y, needle_x, needle_y, theme::accent(), 3.0);
        renderer.fill_ellipse(
            Rect {
                x: center_x - 6.0,
                y: center_y - 6.0,
                width: 12.0,
                height: 12.0,
            },
            theme::accent(),
        );

        renderer.draw_text(
            &format!("{:.1}%", pct * 100.0),
            center_x - 20.0,
            center_y + 15.0,
            12.0,
            theme::text(),
        );
        if !self.label.is_empty() {
            renderer.draw_text(
                &self.label,
                center_x - 30.0,
                center_y + 30.0,
                10.0,
                theme::text_muted(),
            );
        }
    }
}

impl LayoutView for GaugeChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 200.0,
            height: 200.0,
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

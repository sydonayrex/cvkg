use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Line chart component
///
/// # Examples
/// ```
/// use cvkg_components::gpu_charts::LineChart;
/// let chart = LineChart::new()
///     .data(vec![0.1, 0.5, 0.3, 0.8, 0.6])
///     .color([0.0, 1.0, 1.0, 1.0]);
/// ```
pub struct LineChart {
    pub(crate) data: Vec<f32>,
    pub(crate) color: [f32; 4],
    pub(crate) show_grid: bool,
}

impl Default for LineChart {
    fn default() -> Self {
        Self::new()
    }
}

impl LineChart {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            color: theme::accent(),
            show_grid: true,
        }
    }

    pub fn data(mut self, values: Vec<f32>) -> Self {
        self.data = values;
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for LineChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
            return;
        }

        let padding = 40.0;
        let chart_w = rect.width - padding * 2.0;
        let chart_h = rect.height - padding * 2.0;
        let max_val = *self
            .data
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(&1.0);
        let min_val = *self
            .data
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(&0.0);
        let range = (max_val - min_val).max(1.0);

        if self.show_grid {
            // Draw grid lines
            for i in 0..5 {
                let y = rect.y + padding + (chart_h / 4.0) * i as f32;
                renderer.draw_line(
                    rect.x + padding,
                    y,
                    rect.x + rect.width - padding,
                    y,
                    theme::surface(),
                    0.5,
                );
            }
        }

        // Draw line
        if self.data.len() > 1 {
            for i in 0..self.data.len() - 1 {
                let x1 = rect.x + padding + (i as f32 / (self.data.len() - 1) as f32) * chart_w;
                let y1 =
                    rect.y + rect.height - padding - ((self.data[i] - min_val) / range) * chart_h;
                let x2 =
                    rect.x + padding + ((i + 1) as f32 / (self.data.len() - 1) as f32) * chart_w;
                let y2 = rect.y + rect.height
                    - padding
                    - ((self.data[i + 1] - min_val) / range) * chart_h;

                renderer.draw_line(x1, y1, x2, y2, self.color, 2.0);

                // Draw data points
                renderer.fill_ellipse(
                    Rect {
                        x: x1 - 3.0,
                        y: y1 - 3.0,
                        width: 6.0,
                        height: 6.0,
                    },
                    self.color,
                );
            }
        }
    }
}

impl LayoutView for LineChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 400.0,
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

/// Micro inline line chart.
pub struct SparkLineChart {
    pub(crate) data: Vec<f32>,
    pub(crate) color: [f32; 4],
}

impl Default for SparkLineChart {
    fn default() -> Self {
        Self::new()
    }
}

impl SparkLineChart {
    /// Create a new SparkLineChart.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            color: theme::accent(),
        }
    }

    /// Set data values.
    pub fn data(mut self, values: Vec<f32>) -> Self {
        self.data = values;
        self
    }
}

impl View for SparkLineChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.len() < 2 {
            return;
        }

        let max_val = *self
            .data
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(&1.0);
        let min_val = *self
            .data
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(&0.0);
        let range = (max_val - min_val).max(0.001);

        for i in 0..self.data.len() - 1 {
            let x1 = rect.x + (i as f32 / (self.data.len() - 1) as f32) * rect.width;
            let y1 = rect.y + rect.height - ((self.data[i] - min_val) / range) * rect.height;
            let x2 = rect.x + ((i + 1) as f32 / (self.data.len() - 1) as f32) * rect.width;
            let y2 = rect.y + rect.height - ((self.data[i + 1] - min_val) / range) * rect.height;

            renderer.draw_line(x1, y1, x2, y2, self.color, 1.5);
        }
    }
}

impl LayoutView for SparkLineChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 100.0,
            height: 30.0,
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

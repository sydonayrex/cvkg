use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Line chart component
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
            color: [0.0, 0.8, 1.0, 1.0],
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
                    [0.1, 0.1, 0.15, 1.0],
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

/// Bar chart component
pub struct BarChart {
    pub(crate) data: Vec<(String, f32)>,
    pub(crate) color: [f32; 4],
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}

impl BarChart {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            color: [0.0, 0.8, 0.4, 1.0],
        }
    }

    pub fn bars(mut self, items: Vec<(String, f32)>) -> Self {
        self.data = items;
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for BarChart {
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
        let max_val = self.data.iter().map(|(_, v)| *v).fold(0.0f32, f32::max);
        let bar_count = self.data.len() as f32;
        let bar_w = (chart_w / bar_count).max(8.0);

        for (i, (label, value)) in self.data.iter().enumerate() {
            let bar_h = (value / max_val.max(1.0)) * chart_h;
            let x = rect.x + padding + i as f32 * bar_w + bar_w * 0.15;
            let y = rect.y + rect.height - padding - bar_h;

            renderer.fill_rounded_rect(
                Rect {
                    x,
                    y,
                    width: bar_w * 0.7,
                    height: bar_h,
                },
                3.0,
                self.color,
            );

            renderer.draw_text(
                label,
                x,
                rect.y + rect.height - padding + 8.0,
                10.0,
                [0.7, 0.7, 0.8, 1.0],
            );
        }
    }
}

impl LayoutView for BarChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let width = (self.data.len() as f32 * 60.0).max(200.0);
        Size {
            width,
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

/// Scatter plot component
pub struct ScatterPlot {
    pub(crate) points: Vec<(f32, f32)>,
    pub(crate) color: [f32; 4],
}

impl Default for ScatterPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl ScatterPlot {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            color: [0.8, 0.4, 0.0, 1.0],
        }
    }

    pub fn points(mut self, pts: Vec<(f32, f32)>) -> Self {
        self.points = pts;
        self
    }
}

impl View for ScatterPlot {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.points.is_empty() {
            return;
        }

        let padding = 40.0;
        let chart_w = rect.width - padding * 2.0;
        let chart_h = rect.height - padding * 2.0;

        let xs: Vec<f32> = self.points.iter().map(|(x, _)| *x).collect();
        let ys: Vec<f32> = self.points.iter().map(|(_, y)| *y).collect();
        let max_x = *xs.iter().max_by(|a, b| a.total_cmp(b)).unwrap_or(&1.0);
        let max_y = *ys.iter().max_by(|a, b| a.total_cmp(b)).unwrap_or(&1.0);

        for (x, y) in &self.points {
            let px = rect.x + padding + (x / max_x.max(1.0)) * chart_w;
            let py = rect.y + rect.height - padding - (y / max_y.max(1.0)) * chart_h;
            renderer.fill_ellipse(
                Rect {
                    x: px - 4.0,
                    y: py - 4.0,
                    width: 8.0,
                    height: 8.0,
                },
                self.color,
            );
        }
    }
}

impl LayoutView for ScatterPlot {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 400.0,
            height: 300.0,
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

/// Histogram component
pub struct Histogram {
    pub(crate) data: Vec<f32>,
    pub(crate) bins: usize,
    pub(crate) color: [f32; 4],
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            bins: 10,
            color: [0.6, 0.2, 0.8, 1.0],
        }
    }

    pub fn data(mut self, values: Vec<f32>) -> Self {
        self.data = values;
        self
    }

    pub fn bins(mut self, n: usize) -> Self {
        self.bins = n;
        self
    }
}

impl View for Histogram {
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

        let min = *self
            .data
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(&0.0);
        let max = *self
            .data
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(&1.0);
        let range = (max - min).max(1.0);
        let bin_w = chart_w / self.bins as f32;

        let mut counts = vec![0usize; self.bins];
        for val in &self.data {
            let bin = ((val - min) / range * self.bins as f32) as usize;
            counts[bin.min(self.bins - 1)] += 1;
        }

        let max_count = *counts.iter().max().unwrap_or(&1);

        for (i, &count) in counts.iter().enumerate() {
            let bar_h = (count as f32 / max_count as f32) * chart_h;
            let x = rect.x + padding + i as f32 * bin_w + 2.0;
            let y = rect.y + rect.height - padding - bar_h;

            renderer.fill_rounded_rect(
                Rect {
                    x,
                    y,
                    width: bin_w - 4.0,
                    height: bar_h,
                },
                2.0,
                self.color,
            );
        }
    }
}

impl LayoutView for Histogram {
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

/// Radar chart component
pub struct RadarChart {
    pub(crate) data: Vec<f32>,
    pub(crate) labels: Vec<String>,
    pub(crate) color: [f32; 4],
}

impl Default for RadarChart {
    fn default() -> Self {
        Self::new()
    }
}

impl RadarChart {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            labels: Vec::new(),
            color: [0.2, 0.6, 1.0, 1.0],
        }
    }

    pub fn data(mut self, values: Vec<f32>) -> Self {
        self.data = values;
        self
    }

    pub fn labels(mut self, items: Vec<&str>) -> Self {
        self.labels = items.into_iter().map(|s| s.to_string()).collect();
        self
    }
}

impl View for RadarChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
            return;
        }

        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width.min(rect.height) / 2.0 - 30.0).max(20.0);
        let n = self.data.len() as f32;

        // Draw concentric rings
        for r in (0..3).map(|i| radius * (i + 1) as f32 / 3.0) {
            renderer.stroke_ellipse(
                Rect {
                    x: center_x - r,
                    y: center_y - r,
                    width: r * 2.0,
                    height: r * 2.0,
                },
                [0.1, 0.1, 0.15, 1.0],
                0.5,
            );
        }

        // Draw axes
        for i in 0..self.data.len() {
            let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / n) * std::f32::consts::TAU;
            let x = center_x + (radius * (angle).cos());
            let y = center_y + (radius * (angle).sin());
            renderer.draw_line(center_x, center_y, x, y, [0.2, 0.2, 0.3, 1.0], 0.5);

            // Label
            let label_x = center_x + (radius + 20.0) * (angle).cos();
            let label_y = center_y + (radius + 20.0) * (angle).sin();
            if let Some(label) = self.labels.get(i) {
                renderer.draw_text(
                    label,
                    label_x - 10.0,
                    label_y - 6.0,
                    10.0,
                    [0.6, 0.6, 0.7, 1.0],
                );
            }
        }

        // Draw data polygon
        for i in 0..self.data.len() {
            let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / n) * std::f32::consts::TAU;
            let val = self.data[i]
                / self
                    .data
                    .iter()
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(&1.0);
            let x = center_x + (radius * val * (angle).cos());
            let y = center_y + (radius * val * (angle).sin());

            if i > 0 {
                let prev_angle =
                    -std::f32::consts::FRAC_PI_2 + ((i - 1) as f32 / n) * std::f32::consts::TAU;
                let prev_val = self.data[i - 1]
                    / self
                        .data
                        .iter()
                        .max_by(|a, b| a.total_cmp(b))
                        .unwrap_or(&1.0);
                let px = center_x + (radius * prev_val * (prev_angle).cos());
                let py = center_y + (radius * prev_val * (prev_angle).sin());
                renderer.draw_line(px, py, x, y, self.color, 2.0);
            }

            renderer.fill_ellipse(
                Rect {
                    x: x - 3.0,
                    y: y - 3.0,
                    width: 6.0,
                    height: 6.0,
                },
                self.color,
            );
        }
    }
}

impl LayoutView for RadarChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
            height: 300.0,
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

pub mod bar;
pub mod line;

pub use bar::{BarChart, Histogram};
pub use line::{LineChart, SparkLineChart};

use crate::theme;
use crate::RADIUS_XS;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

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
            color: theme::warning(),
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
            color: theme::accent(),
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
                theme::surface(),
                0.5,
            );
        }

        // Draw axes
        for i in 0..self.data.len() {
            let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / n) * std::f32::consts::TAU;
            let x = center_x + (radius * (angle).cos());
            let y = center_y + (radius * (angle).sin());
            renderer.draw_line(center_x, center_y, x, y, theme::border_strong(), 0.5);

            // Label
            let label_x = center_x + (radius + 20.0) * (angle).cos();
            let label_y = center_y + (radius + 20.0) * (angle).sin();
            if let Some(label) = self.labels.get(i) {
                renderer.draw_text(
                    label,
                    label_x - 10.0,
                    label_y - 6.0,
                    10.0,
                    theme::text_muted(),
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

/// Pie chart component displaying proportional slices of data.
///
/// # Contract
/// - Data values must be positive.
/// - Slices are colored using standard accents or custom themes.
pub struct PieChart {
    pub(crate) data: Vec<(String, f32)>,
    pub(crate) colors: Vec<[f32; 4]>,
}

impl Default for PieChart {
    fn default() -> Self {
        Self::new()
    }
}

impl PieChart {
    /// Create a new empty PieChart.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            colors: Vec::new(),
        }
    }

    /// Set data slices.
    pub fn slices(mut self, slices: Vec<(String, f32)>) -> Self {
        self.data = slices;
        self
    }

    /// Set slice colors.
    pub fn colors(mut self, colors: Vec<[f32; 4]>) -> Self {
        self.colors = colors;
        self
    }
}

impl View for PieChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
            return;
        }

        let total: f32 = self.data.iter().map(|(_, v)| v).sum();
        if total <= 0.0 {
            return;
        }

        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width.min(rect.height) / 2.0 - 20.0).max(10.0);

        let mut start_angle = 0.0f32;
        for (i, (label, val)) in self.data.iter().enumerate() {
            let sweep = (val / total) * std::f32::consts::TAU;
            let color = self.colors.get(i).copied().unwrap_or(theme::accent());

            // Render mock pie slices as a combination of border circles/lines for outline
            // inside our vector renderer
            let middle_angle = start_angle + sweep * 0.5;
            let lx = center_x + radius * middle_angle.cos();
            let ly = center_y + radius * middle_angle.sin();
            renderer.draw_line(center_x, center_y, lx, ly, color, 2.0);

            // Draw labels
            let label_dist = radius + 15.0;
            let tx = center_x + label_dist * middle_angle.cos();
            let ty = center_y + label_dist * middle_angle.sin();
            renderer.draw_text(label, tx - 10.0, ty - 6.0, 10.0, theme::text_muted());

            start_angle += sweep;
        }

        // Draw central outer ring
        renderer.stroke_ellipse(
            Rect {
                x: center_x - radius,
                y: center_y - radius,
                width: radius * 2.0,
                height: radius * 2.0,
            },
            theme::border_strong(),
            1.5,
        );
    }
}

impl LayoutView for PieChart {
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

/// 2D grid density chart using color scales to display magnitudes.
///
/// # Contract
/// - Maps a grid of rows and columns using normalized magnitudes.
pub struct HeatmapChart {
    pub(crate) grid: Vec<Vec<f32>>,
    pub(crate) min_color: [f32; 4],
    pub(crate) max_color: [f32; 4],
}

impl Default for HeatmapChart {
    fn default() -> Self {
        Self::new()
    }
}

impl HeatmapChart {
    /// Create a new HeatmapChart.
    pub fn new() -> Self {
        Self {
            grid: Vec::new(),
            min_color: theme::surface(),
            max_color: theme::accent(),
        }
    }

    /// Set grid values.
    pub fn grid(mut self, grid: Vec<Vec<f32>>) -> Self {
        self.grid = grid;
        self
    }
}

impl View for HeatmapChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.grid.is_empty() || self.grid[0].is_empty() {
            return;
        }

        let rows = self.grid.len();
        let cols = self.grid[0].len();
        let cell_w = rect.width / cols as f32;
        let cell_h = rect.height / rows as f32;

        let flat: Vec<f32> = self.grid.iter().flatten().copied().collect();
        let max_val = *flat.iter().max_by(|a, b| a.total_cmp(b)).unwrap_or(&1.0);
        let min_val = *flat.iter().min_by(|a, b| a.total_cmp(b)).unwrap_or(&0.0);
        let range = (max_val - min_val).max(0.001);

        for r in 0..rows {
            for c in 0..cols {
                let val = self.grid[r][c];
                let factor = ((val - min_val) / range).clamp(0.0, 1.0);

                let cell_color = [
                    self.min_color[0] + (self.max_color[0] - self.min_color[0]) * factor,
                    self.min_color[1] + (self.max_color[1] - self.min_color[1]) * factor,
                    self.min_color[2] + (self.max_color[2] - self.min_color[2]) * factor,
                    self.min_color[3] + (self.max_color[3] - self.min_color[3]) * factor,
                ];

                let cell_rect = Rect {
                    x: rect.x + c as f32 * cell_w + 1.0,
                    y: rect.y + r as f32 * cell_h + 1.0,
                    width: cell_w - 2.0,
                    height: cell_h - 2.0,
                };
                renderer.fill_rounded_rect(cell_rect, RADIUS_XS, cell_color);
            }
        }
    }
}

impl LayoutView for HeatmapChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
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

/// Financial candlestick data point.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    /// Open price.
    pub open: f32,
    /// High price.
    pub high: f32,
    /// Low price.
    pub low: f32,
    /// Close price.
    pub close: f32,
}

/// Financial Candlestick chart displaying stock/market price trends.
pub struct CandlestickChart {
    pub(crate) data: Vec<Candle>,
}

impl Default for CandlestickChart {
    fn default() -> Self {
        Self::new()
    }
}

impl CandlestickChart {
    /// Create a new empty CandlestickChart.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Set candle items.
    pub fn candles(mut self, candles: Vec<Candle>) -> Self {
        self.data = candles;
        self
    }
}

impl View for CandlestickChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.data.is_empty() {
            return;
        }

        let padding = 30.0;
        let chart_w = rect.width - padding * 2.0;
        let chart_h = rect.height - padding * 2.0;

        let max_val = self
            .data
            .iter()
            .map(|c| c.high)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(1.0);
        let min_val = self
            .data
            .iter()
            .map(|c| c.low)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0);
        let range = (max_val - min_val).max(0.001);

        let count = self.data.len();
        let candle_w = chart_w / count as f32;

        for (i, candle) in self.data.iter().enumerate() {
            let x = rect.x + padding + i as f32 * candle_w;
            let cx = x + candle_w * 0.5;

            let y_high =
                rect.y + rect.height - padding - ((candle.high - min_val) / range) * chart_h;
            let y_low = rect.y + rect.height - padding - ((candle.low - min_val) / range) * chart_h;
            let y_open =
                rect.y + rect.height - padding - ((candle.open - min_val) / range) * chart_h;
            let y_close =
                rect.y + rect.height - padding - ((candle.close - min_val) / range) * chart_h;

            let color = if candle.close >= candle.open {
                theme::success()
            } else {
                theme::error_color()
            };

            // High/low wick line
            renderer.draw_line(cx, y_high, cx, y_low, color, 1.0);

            // Candle body
            let body_y = y_open.min(y_close);
            let body_h = (y_open - y_close).abs().max(2.0);
            let body_w = candle_w * 0.7;

            renderer.fill_rect(
                Rect {
                    x: cx - body_w * 0.5,
                    y: body_y,
                    width: body_w,
                    height: body_h,
                },
                color,
            );
        }
    }
}

impl LayoutView for CandlestickChart {
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

/// Conversion stage funnel chart.
pub struct FunnelChart {
    pub(crate) stages: Vec<(String, f32)>,
}

impl Default for FunnelChart {
    fn default() -> Self {
        Self::new()
    }
}

impl FunnelChart {
    /// Create a new empty FunnelChart.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Set stage items.
    pub fn stages(mut self, stages: Vec<(String, f32)>) -> Self {
        self.stages = stages;
        self
    }
}

impl View for FunnelChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.stages.is_empty() {
            return;
        }

        let max_val = self
            .stages
            .iter()
            .map(|(_, v)| *v)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(1.0);
        let count = self.stages.len();
        let stage_h = rect.height / count as f32;

        for (i, (label, val)) in self.stages.iter().enumerate() {
            let width_ratio = val / max_val.max(0.001);
            let w = rect.width * width_ratio;
            let x = rect.x + (rect.width - w) / 2.0;
            let y = rect.y + i as f32 * stage_h;

            renderer.fill_rounded_rect(
                Rect {
                    x: x + 4.0,
                    y: y + 4.0,
                    width: w - 8.0,
                    height: stage_h - 8.0,
                },
                4.0,
                theme::accent(),
            );

            renderer.draw_text(
                &format!("{}: {:.0}", label, val),
                rect.x + 12.0,
                y + stage_h / 2.0 - 4.0,
                11.0,
                theme::text(),
            );
        }
    }
}

impl LayoutView for FunnelChart {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
            height: 250.0,
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

/// Flow distribution Sankey diagram component.
pub struct SankeyChart {
    pub(crate) flows: Vec<(String, String, f32)>,
}

impl Default for SankeyChart {
    fn default() -> Self {
        Self::new()
    }
}

impl SankeyChart {
    /// Create a new empty SankeyChart.
    pub fn new() -> Self {
        Self { flows: Vec::new() }
    }

    /// Set flow items.
    pub fn flows(mut self, flows: Vec<(String, String, f32)>) -> Self {
        self.flows = flows;
        self
    }
}

impl View for SankeyChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.flows.is_empty() {
            return;
        }

        // Simulating node blocks on left and right, and connectors
        let left_x = rect.x + 20.0;
        let right_x = rect.x + rect.width - 40.0;
        let node_w = 20.0;

        let total_flow: f32 = self.flows.iter().map(|(_, _, v)| v).sum();
        if total_flow <= 0.0 {
            return;
        }

        let mut left_y = rect.y + 20.0;
        let mut right_y = rect.y + 20.0;

        for (from, to, val) in &self.flows {
            let flow_h = (val / total_flow) * (rect.height - 40.0);

            // Draw source node block
            renderer.fill_rounded_rect(
                Rect {
                    x: left_x,
                    y: left_y,
                    width: node_w,
                    height: flow_h.max(4.0),
                },
                2.0,
                theme::accent(),
            );

            // Draw target node block
            renderer.fill_rounded_rect(
                Rect {
                    x: right_x,
                    y: right_y,
                    width: node_w,
                    height: flow_h.max(4.0),
                },
                2.0,
                theme::success(),
            );

            // Draw flowing connector line
            renderer.draw_line(
                left_x + node_w,
                left_y + flow_h / 2.0,
                right_x,
                right_y + flow_h / 2.0,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.25,
                ],
                flow_h.max(1.0),
            );

            renderer.draw_text(from, left_x - 10.0, left_y - 8.0, 9.0, theme::text_muted());
            renderer.draw_text(to, right_x, right_y - 8.0, 9.0, theme::text_muted());

            left_y += flow_h + 10.0;
            right_y += flow_h + 10.0;
        }
    }
}

impl LayoutView for SankeyChart {
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

/// Dial/radial metric gauge chart.
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

/// Recursive nested rectangle layout tree node.
#[derive(Debug, Clone)]
pub struct TreemapNode {
    /// Label.
    pub label: String,
    /// Quantitative value weight.
    pub value: f32,
}

/// Treemap visualization mapping weights to nested rectangular areas.
pub struct TreemapChart {
    pub(crate) nodes: Vec<TreemapNode>,
}

impl Default for TreemapChart {
    fn default() -> Self {
        Self::new()
    }
}

impl TreemapChart {
    /// Create a new empty TreemapChart.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Set node data items.
    pub fn nodes(mut self, nodes: Vec<TreemapNode>) -> Self {
        self.nodes = nodes;
        self
    }
}

impl View for TreemapChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.nodes.is_empty() {
            return;
        }

        let total_val: f32 = self.nodes.iter().map(|n| n.value).sum();
        if total_val <= 0.0 {
            return;
        }

        // Simple subdivision algorithm (slice-and-dice)
        let mut remaining_rect = rect;
        let mut vertical = rect.width >= rect.height;

        for (i, node) in self.nodes.iter().enumerate() {
            let ratio = node.value / total_val;
            let color = [
                theme::accent()[0] * (1.0 - 0.1 * i as f32).max(0.1),
                theme::accent()[1] * (1.0 - 0.1 * i as f32).max(0.1),
                theme::accent()[2] * (1.0 - 0.1 * i as f32).max(0.1),
                1.0,
            ];

            if vertical {
                let w = remaining_rect.width * ratio;
                let cell_rect = Rect {
                    x: remaining_rect.x,
                    y: remaining_rect.y,
                    width: w,
                    height: remaining_rect.height,
                };
                renderer.fill_rounded_rect(cell_rect, RADIUS_XS, color);
                renderer.stroke_rounded_rect(cell_rect, RADIUS_XS, theme::border(), 0.5);
                renderer.draw_text(
                    &node.label,
                    cell_rect.x + 4.0,
                    cell_rect.y + 14.0,
                    10.0,
                    theme::text(),
                );

                remaining_rect.x += w;
                remaining_rect.width -= w;
            } else {
                let h = remaining_rect.height * ratio;
                let cell_rect = Rect {
                    x: remaining_rect.x,
                    y: remaining_rect.y,
                    width: remaining_rect.width,
                    height: h,
                };
                renderer.fill_rounded_rect(cell_rect, RADIUS_XS, color);
                renderer.stroke_rounded_rect(cell_rect, RADIUS_XS, theme::border(), 0.5);
                renderer.draw_text(
                    &node.label,
                    cell_rect.x + 4.0,
                    cell_rect.y + 14.0,
                    10.0,
                    theme::text(),
                );

                remaining_rect.y += h;
                remaining_rect.height -= h;
            }
            vertical = !vertical;
        }
    }
}

impl LayoutView for TreemapChart {
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

/// Bounded min/max area range chart.
pub struct RangeChart {
    pub(crate) upper: Vec<f32>,
    pub(crate) lower: Vec<f32>,
}

impl Default for RangeChart {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeChart {
    /// Create a new empty RangeChart.
    pub fn new() -> Self {
        Self {
            upper: Vec::new(),
            lower: Vec::new(),
        }
    }

    /// Set upper bounds.
    pub fn upper(mut self, values: Vec<f32>) -> Self {
        self.upper = values;
        self
    }

    /// Set lower bounds.
    pub fn lower(mut self, values: Vec<f32>) -> Self {
        self.lower = values;
        self
    }
}

impl View for RangeChart {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.upper.is_empty() || self.lower.is_empty() {
            return;
        }

        let max_val = *self
            .upper
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(&1.0);
        let min_val = *self
            .lower
            .iter()
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(&0.0);
        let range = (max_val - min_val).max(0.001);

        let count = self.upper.len().min(self.lower.len());

        for i in 0..count - 1 {
            let x1 = rect.x + (i as f32 / (count - 1) as f32) * rect.width;
            let y1_up = rect.y + rect.height - ((self.upper[i] - min_val) / range) * rect.height;
            let y1_down = rect.y + rect.height - ((self.lower[i] - min_val) / range) * rect.height;

            let x2 = rect.x + ((i + 1) as f32 / (count - 1) as f32) * rect.width;
            let y2_up =
                rect.y + rect.height - ((self.upper[i + 1] - min_val) / range) * rect.height;
            let y2_down =
                rect.y + rect.height - ((self.lower[i + 1] - min_val) / range) * rect.height;

            // Draw bounding lines
            renderer.draw_line(x1, y1_up, x2, y2_up, theme::accent(), 1.5);
            renderer.draw_line(x1, y1_down, x2, y2_down, theme::accent(), 1.5);

            // Shading indicator
            renderer.draw_line(
                x1,
                y1_up,
                x1,
                y1_down,
                [
                    theme::accent()[0],
                    theme::accent()[1],
                    theme::accent()[2],
                    0.15,
                ],
                2.0,
            );
        }
    }
}

impl LayoutView for RangeChart {
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

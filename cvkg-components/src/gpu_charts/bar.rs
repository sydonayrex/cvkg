use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

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
            color: theme::success(),
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

            renderer.draw_text_raw(
                label,
                x,
                rect.y + rect.height - padding + 8.0,
                10.0,
                theme::text_muted(),
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
            color: theme::secondary(),
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

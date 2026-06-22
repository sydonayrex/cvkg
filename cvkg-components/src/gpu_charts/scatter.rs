use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};

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

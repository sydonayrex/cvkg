use crate::theme;
use crate::RADIUS_XS;
use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};

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

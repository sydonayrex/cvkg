use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};

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

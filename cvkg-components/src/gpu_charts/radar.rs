use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};

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

use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

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

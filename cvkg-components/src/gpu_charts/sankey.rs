use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

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

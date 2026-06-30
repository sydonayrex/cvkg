use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

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

            renderer.draw_text_raw(
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

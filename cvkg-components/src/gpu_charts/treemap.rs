use crate::RADIUS_XS;
use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// A node in a treemap visualization.
pub struct TreemapNode {
    /// Display label.
    pub label: String,
    /// Relative size (area) of this node.
    pub value: f32,
    /// Child nodes (for hierarchical treemaps).
    pub children: Vec<TreemapNode>,
    /// Fill color override (None = auto-assigned from palette).
    pub color: Option<[f32; 4]>,
}

/// Treemap chart — hierarchical data visualization using nested rectangles.
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
                renderer.draw_text_raw(
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
                renderer.draw_text_raw(
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

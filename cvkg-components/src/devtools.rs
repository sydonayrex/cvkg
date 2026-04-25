use cvkg_core::{Never, Rect, Renderer, View};
use cvkg_vdom::VDom;
// No imports needed from crate for now

/// A HUD-style inspector for the Virtual DOM.
pub struct VdomInspector {
    vdom: std::sync::Arc<VDom>,
}

impl VdomInspector {
    /// Create a new VdomInspector.
    pub fn new(vdom: std::sync::Arc<VDom>) -> Self {
        Self { vdom }
    }
}

impl View for VdomInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render a semi-transparent panel on the right
        let panel_width = 300.0;
        let panel_rect = Rect {
            x: rect.x + rect.width - panel_width,
            y: rect.y,
            width: panel_width,
            height: rect.height,
        };

        // Bifrost background
        renderer.bifrost(panel_rect, 20.0, 0.5, 0.8);
        renderer.fill_rect(panel_rect, [0.05, 0.05, 0.05, 0.8]);
        renderer.stroke_rect(panel_rect, [0.0, 1.0, 1.0, 1.0], 1.0);

        // Title
        let title_rect = Rect {
            x: panel_rect.x + 10.0,
            y: panel_rect.y + 10.0,
            width: panel_width - 20.0,
            height: 30.0,
        };
        renderer.draw_text(
            "VDOM INSPECTOR",
            title_rect.x,
            title_rect.y,
            16.0,
            [0.0, 1.0, 1.0, 1.0],
        );

        // Hierarchy
        if let Some(root_id) = self.vdom.root {
            self.render_node(
                renderer,
                root_id,
                panel_rect.x + 10.0,
                panel_rect.y + 50.0,
                0,
            );
        }
    }
}

impl VdomInspector {
    fn render_node(
        &self,
        renderer: &mut dyn Renderer,
        id: cvkg_vdom::NodeId,
        x: f32,
        y: f32,
        depth: usize,
    ) -> f32 {
        let mut current_y = y;
        if let Some(node) = self.vdom.nodes.get(&id) {
            let indent = depth as f32 * 15.0;
            let node_label = format!("{} (ID: {})", node.component_type, id.0);

            renderer.draw_text(
                &node_label,
                x + indent,
                current_y,
                12.0,
                [1.0, 1.0, 1.0, 1.0],
            );
            current_y += 20.0;

            for child_id in &node.children {
                current_y = self.render_node(renderer, *child_id, x, current_y, depth + 1);
            }
        }
        current_y
    }
}

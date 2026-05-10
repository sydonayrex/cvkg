use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never, AnyView};

/// A node in a graph editor.
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub content: AnyView,
}

/// An edge connecting two nodes.
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// A node-based graph editor.
pub struct NodeGraphEditor {
    pub(crate) nodes: Vec<GraphNode>,
    pub(crate) edges: Vec<GraphEdge>,
    pub(crate) selected_node: Option<String>,
}

impl NodeGraphEditor {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            selected_node: None,
        }
    }

    pub fn node(mut self, id: &str, label: &str, x: f32, y: f32, width: f32, height: f32, content: impl View + Clone + 'static) -> Self {
        self.nodes.push(GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            x, y, width, height,
            content: content.erase(),
        });
        self
    }

    pub fn edge(mut self, from: &str, to: &str, label: Option<&str>) -> Self {
        self.edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(|s| s.to_string()),
        });
        self
    }

    pub fn select(mut self, node_id: &str) -> Self {
        self.selected_node = Some(node_id.to_string());
        self
    }
}

impl View for NodeGraphEditor {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        // Draw edges first (behind nodes)
        for edge in &self.edges {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.iter().find(|n| n.id == edge.from),
                self.nodes.iter().find(|n| n.id == edge.to),
            ) {
                let from_center = Rect {
                    x: from_node.x + from_node.width / 2.0,
                    y: from_node.y + from_node.height / 2.0,
                    width: 0.0,
                    height: 0.0,
                };
                let to_center = Rect {
                    x: to_node.x + to_node.width / 2.0,
                    y: to_node.y + to_node.height / 2.0,
                    width: 0.0,
                    height: 0.0,
                };

                // Draw Bézier curve
                let _control_offset = ((to_center.x - from_center.x).abs() * 0.5).max(40.0);
                renderer.draw_line(from_center.x, from_center.y, to_center.x, to_center.y, [0.3, 0.5, 0.8, 0.8], 2.0);

                // Draw arrow
                let angle = (to_center.y - from_center.y).atan2(to_center.x - from_center.x);
                let arrow_size = 8.0;
                let arrow_x = to_center.x - (arrow_size * (angle).cos());
                let arrow_y = to_center.y - (arrow_size * (angle).sin());
                renderer.draw_line(to_center.x, to_center.y, arrow_x, arrow_y, [0.3, 0.5, 0.8, 1.0], 2.0);

                // Draw label
                if let Some(ref label) = edge.label {
                    let mid_x = (from_center.x + to_center.x) / 2.0;
                    let mid_y = (from_center.y + to_center.y) / 2.0;
                    renderer.draw_text(label, mid_x - 20.0, mid_y - 8.0, 11.0, [0.5, 0.6, 0.7, 1.0]);
                }
            }
        }

        // Draw nodes
        for node in &self.nodes {
            let is_selected = self.selected_node.as_deref() == Some(&node.id);
            let bg = if is_selected { [0.1, 0.2, 0.35, 1.0] } else { [0.08, 0.08, 0.15, 1.0] };
            renderer.fill_rounded_rect(
                Rect { x: node.x, y: node.y, width: node.width, height: node.height },
                6.0, bg
            );

            if is_selected {
                renderer.stroke_rounded_rect(
                    Rect { x: node.x, y: node.y, width: node.width, height: node.height },
                    6.0, [0.0, 0.8, 1.0, 1.0], 2.0
                );
            }

            renderer.draw_text(&node.label, node.x + 8.0, node.y + 14.0, 13.0, [0.9, 0.95, 1.0, 1.0]);

            // Render node content
            let content_rect = Rect {
                x: node.x + 8.0,
                y: node.y + 28.0,
                width: node.width - 16.0,
                height: node.height - 28.0,
            };
            node.content.render(renderer, content_rect);
        }
    }
}

impl LayoutView for NodeGraphEditor {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        let max_x = self.nodes.iter().map(|n| n.x + n.width).fold(0.0, f32::max);
        let max_y = self.nodes.iter().map(|n| n.y + n.height).fold(0.0, f32::max);
        Size { width: max_x + 40.0, height: max_y + 40.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}

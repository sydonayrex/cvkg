use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// MimirsWell - A dynamic, force-directed graph visualization for the Temporal Graph.
#[doc(alias = "Well")]
#[derive(Clone)]
pub struct MimirsWell {
    pub nodes: Vec<WellNode>,
    pub edges: Vec<WellEdge>,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone)]
pub struct WellNode {
    pub id: String,
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub weight: f32,
}

#[derive(Clone)]
pub struct WellEdge {
    pub from: String,
    pub to: String,
    pub strength: f32,
}

impl MimirsWell {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            width: 300.0,
            height: 200.0,
        }
    }

    pub fn node(mut self, id: &str, label: &str, x: f32, y: f32, weight: f32) -> Self {
        self.nodes.push(WellNode {
            id: id.to_string(),
            label: label.to_string(),
            x,
            y,
            weight,
        });
        self
    }

    pub fn edge(mut self, from: &str, to: &str, strength: f32) -> Self {
        self.edges.push(WellEdge {
            from: from.to_string(),
            to: to.to_string(),
            strength,
        });
        self
    }
}

impl View for MimirsWell {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 8.0, theme::surface());
        for node in &self.nodes {
            let nx = rect.x + node.x * rect.width;
            let ny = rect.y + node.y * rect.height;
            let radius = 4.0 + node.weight * 8.0;
            renderer.fill_ellipse(
                Rect {
                    x: nx - radius,
                    y: ny - radius,
                    width: radius * 2.0,
                    height: radius * 2.0,
                },
                theme::accent(),
            );
        }
        for edge in &self.edges {
            if let Some(from) = self.nodes.iter().find(|n| n.id == edge.from) {
                if let Some(to) = self.nodes.iter().find(|n| n.id == edge.to) {
                    renderer.draw_line(
                        rect.x + from.x * rect.width,
                        rect.y + from.y * rect.height,
                        rect.x + to.x * rect.width,
                        rect.y + to.y * rect.height,
                        [theme::text_dim()[0], theme::text_dim()[1], theme::text_dim()[2], 0.3],
                        1.0,
                    );
                }
            }
        }
    }
}

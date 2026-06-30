//! Skadi Scripting - Visual scripting system
//!
//! Skadi the Vanir huntress brings precision and sharp focus - her scripting
//! system enables visual workflow construction with precise node connections.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Script node types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptNodeType {
    Input,
    Process,
    Condition,
    Output,
    Variable,
}

/// Connection between script nodes
#[derive(Debug, Clone)]
pub struct ScriptConnection {
    pub from: usize,
    pub to: usize,
    pub label: String,
}

/// Script node in visual scripting
#[derive(Debug, Clone)]
pub struct ScriptNode {
    pub id: usize,
    pub name: String,
    pub node_type: ScriptNodeType,
    pub position: (f32, f32),
    pub code: String,
}

/// Skadi Visual Scripting System
pub struct SkadiScripting {
    pub nodes: Vec<ScriptNode>,
    pub connections: Vec<ScriptConnection>,
    pub selected_node: Option<usize>,
}

impl Default for SkadiScripting {
    fn default() -> Self {
        Self::new()
    }
}

impl SkadiScripting {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            selected_node: None,
        }
    }

    pub fn node(
        mut self,
        id: usize,
        name: &str,
        node_type: ScriptNodeType,
        x: f32,
        y: f32,
    ) -> Self {
        self.nodes.push(ScriptNode {
            id,
            name: name.to_string(),
            node_type,
            position: (x, y),
            code: String::new(),
        });
        self
    }

    pub fn connect(mut self, from: usize, to: usize, label: &str) -> Self {
        self.connections.push(ScriptConnection {
            from,
            to,
            label: label.to_string(),
        });
        self
    }

    pub fn code(mut self, id: usize, code: &str) -> Self {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.code = code.to_string();
        }
        self
    }

    pub fn select(mut self, node_id: usize) -> Self {
        self.selected_node = Some(node_id);
        self
    }
}

impl View for SkadiScripting {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background grid
        for x in 0..10 {
            for y in 0..10 {
                let gx = rect.x + 20.0 + x as f32 * 40.0;
                let gy = rect.y + 20.0 + y as f32 * 40.0;
                renderer.fill_rect(
                    Rect {
                        x: gx,
                        y: gy,
                        width: 2.0,
                        height: 2.0,
                    },
                    [0.1, 0.1, 0.12, 0.5],
                );
            }
        }

        // Connections (drawn first as background)
        for conn in &self.connections {
            if let (Some(from), Some(to)) = (
                self.nodes.iter().find(|n| n.id == conn.from),
                self.nodes.iter().find(|n| n.id == conn.to),
            ) {
                let x1 = rect.x + from.position.0;
                let y1 = rect.y + from.position.1;
                let x2 = rect.x + to.position.0;
                let y2 = rect.y + to.position.1;
                renderer.draw_line(x1, y1, x2, y2, theme::with_alpha(theme::border(), 0.8), 2.0);
                renderer.draw_text_raw(
                    &conn.label,
                    (x1 + x2) / 2.0,
                    (y1 + y2) / 2.0,
                    9.0,
                    [0.5, 0.6, 0.8, 1.0],
                );
            }
        }

        // Nodes
        for node in &self.nodes {
            let color = match node.node_type {
                ScriptNodeType::Input => [0.0, 0.7, 0.9, 1.0],
                ScriptNodeType::Process => [0.4, 0.6, 0.9, 1.0],
                ScriptNodeType::Condition => [0.9, 0.7, 0.2, 1.0],
                ScriptNodeType::Output => theme::success(),
                ScriptNodeType::Variable => [0.7, 0.4, 0.9, 1.0],
            };

            let cx = rect.x + node.position.0;
            let cy = rect.y + node.position.1;
            let is_selected = self.selected_node == Some(node.id);

            renderer.fill_rect(
                Rect {
                    x: cx - 50.0,
                    y: cy - 20.0,
                    width: 100.0,
                    height: 40.0,
                },
                color,
            );
            if is_selected {
                renderer.stroke_rect(
                    Rect {
                        x: cx - 52.0,
                        y: cy - 22.0,
                        width: 104.0,
                        height: 44.0,
                    },
                    [1.0, 0.9, 0.4, 1.0],
                    2.0,
                );
            }
            renderer.draw_text_raw(
                &node.name,
                cx - 45.0,
                cy - 5.0,
                11.0,
                [0.95, 0.95, 1.0, 1.0],
            );
        }
    }
}

impl LayoutView for SkadiScripting {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 500.0,
            height: 400.0,
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

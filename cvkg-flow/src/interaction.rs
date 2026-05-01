use crate::types::{NodeId, PortId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionState {
    pub drag_state: Option<DragState>,
    pub selected_nodes: Vec<NodeId>,
    pub active_port: Option<PortId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DragState {
    Node {
        id: NodeId,
        start_pos: (f32, f32),
        mouse_start: (f32, f32),
    },
    Canvas {
        start_offset: (f32, f32),
        mouse_start: (f32, f32),
    },
    Connection {
        source_port: PortId,
        current_pos: (f32, f32),
    },
    SelectionBox {
        start_pos: (f32, f32),
        current_pos: (f32, f32),
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSettings {
    pub grid_snapping: bool,
    pub grid_size: f32,
    pub show_grid: bool,
}

impl Default for FlowSettings {
    fn default() -> Self {
        Self {
            grid_snapping: true,
            grid_size: 20.0,
            show_grid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowContainer {
    pub graph: crate::graph::FlowGraph,
    pub interaction: InteractionState,
    pub settings: FlowSettings,
    pub offset: (f32, f32),
    pub scale: f32,
    #[serde(skip)]
    pub history: Vec<crate::graph::FlowGraph>,
    #[serde(skip)]
    pub redo_stack: Vec<crate::graph::FlowGraph>,
}

impl FlowContainer {
    pub fn push_history(&mut self) {
        self.history.push(self.graph.clone());
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            let current = std::mem::replace(&mut self.graph, prev);
            self.redo_stack.push(current);
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current = std::mem::replace(&mut self.graph, next);
            self.history.push(current);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::FlowNode;
    use crate::types::NodeId;

    #[test]
    fn test_undo_redo() {
        let mut container = FlowContainer::default();
        container.graph.add_node(FlowNode::new(NodeId(1), "Initial", (0.0, 0.0)));
        
        // Push state
        container.push_history();
        
        // Modify
        container.graph.add_node(FlowNode::new(NodeId(2), "Modified", (100.0, 100.0)));
        assert_eq!(container.graph.nodes.len(), 2);
        
        // Undo
        container.undo();
        assert_eq!(container.graph.nodes.len(), 1);
        assert!(container.graph.nodes.contains_key(&NodeId(1)));
        
        // Redo
        container.redo();
        assert_eq!(container.graph.nodes.len(), 2);
    }
}

use crate::types::{NodeId, NodeType};
use crate::port::FlowPort;
use serde::{Deserialize, Serialize};

/// A node in the flow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: NodeId,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub node_type: NodeType,
    pub ports: Vec<FlowPort>,
    pub label: String,
    pub selected: bool,
}

impl FlowNode {
    pub fn new(id: NodeId, label: impl Into<String>, position: (f32, f32)) -> Self {
        Self {
            id,
            position,
            size: (150.0, 80.0),
            node_type: NodeType::Default,
            ports: Vec::new(),
            label: label.into(),
            selected: false,
        }
    }

    pub fn add_port(&mut self, port: FlowPort) {
        self.ports.push(port);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PortId, PortPosition, PortDirection};

    #[test]
    fn test_node_creation() {
        let node = FlowNode::new(NodeId(1), "Test Node", (0.0, 0.0));
        assert_eq!(node.label, "Test Node");
        assert_eq!(node.ports.len(), 0);
    }

    #[test]
    fn test_add_port() {
        let mut node = FlowNode::new(NodeId(1), "Test Node", (0.0, 0.0));
        node.add_port(FlowPort::new(PortId(10), NodeId(1), PortPosition::Right, PortDirection::Output));
        assert_eq!(node.ports.len(), 1);
    }
}

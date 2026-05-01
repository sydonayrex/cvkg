use crate::node::FlowNode;
use crate::edge::FlowEdge;
use crate::types::{NodeId, EdgeId, PortId};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// FlowGraph - collection of nodes and edges
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowGraph {
    pub nodes: HashMap<NodeId, FlowNode>,
    pub edges: HashMap<EdgeId, FlowEdge>,
}

impl FlowGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: FlowNode) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: FlowEdge) {
        self.edges.insert(edge.id, edge);
    }

    pub fn get_node_by_port(&self, port_id: PortId) -> Option<&FlowNode> {
        self.nodes.values().find(|n| n.ports.iter().any(|p| p.id == port_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::node::FlowNode;
    use crate::port::FlowPort;
    use crate::edge::FlowEdge;

    #[test]
    fn test_graph_add_node_edge() {
        let mut graph = FlowGraph::new();
        let mut n1 = FlowNode::new(NodeId(1), "N1", (0.0, 0.0));
        n1.add_port(FlowPort::new(PortId(10), NodeId(1), PortPosition::Right, PortDirection::Output));
        graph.add_node(n1);

        let mut n2 = FlowNode::new(NodeId(2), "N2", (100.0, 0.0));
        n2.add_port(FlowPort::new(PortId(20), NodeId(2), PortPosition::Left, PortDirection::Input));
        graph.add_node(n2);

        graph.add_edge(FlowEdge::new(EdgeId(100), PortId(10), PortId(20)));

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        
        let node = graph.get_node_by_port(PortId(10)).unwrap();
        assert_eq!(node.id, NodeId(1));
    }
}

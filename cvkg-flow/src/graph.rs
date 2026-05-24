use crate::edge::FlowEdge;
use crate::node::FlowNode;
use crate::types::{EdgeId, NodeId, PortId};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FlowGraph - collection of nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowGraph {
    pub nodes: HashMap<NodeId, FlowNode>,
    pub edges: HashMap<EdgeId, FlowEdge>,
}

impl FlowGraph {
    /// Creates a new empty flow graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: FlowNode) {
        self.nodes.insert(node.id, node);
    }

    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, edge: FlowEdge) {
        self.edges.insert(EdgeId(edge.id), edge);
    }

    /// Returns the node that owns the given port, if any.
    pub fn get_node_by_port(&self, port_id: PortId) -> Option<&FlowNode> {
        self.nodes
            .values()
            .find(|n| n.ports.iter().any(|p| p.id == port_id))
    }

    /// Returns all node IDs whose bounding rectangles intersect the given query rectangle.
    ///
    /// The query rectangle is defined by its origin `(x, y)` and size `(width, height)`.
    /// A node is considered intersecting if its axis-aligned bounding box overlaps the
    /// query rectangle by any amount (edge-touching counts as intersection).
    ///
    /// This performs a linear scan over all nodes. For very large graphs, consider
    /// maintaining a spatial index (e.g. an R-tree) and calling `rebuild_spatial_index`
    /// after mutations.
    pub fn nodes_in_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<NodeId> {
        let q_min_x = x;
        let q_min_y = y;
        let q_max_x = x + width;
        let q_max_y = y + height;

        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                let (nx, ny) = node.position;
                let (nw, nh) = node.size;
                let n_max_x = nx + nw;
                let n_max_y = ny + nh;

                let intersects =
                    nx <= q_max_x && n_max_x >= q_min_x && ny <= q_max_y && n_max_y >= q_min_y;

                if intersects { Some(*id) } else { None }
            })
            .collect()
    }

    /// Returns all node IDs whose bounding rectangles are within `radius` of `point`.
    ///
    /// The distance is measured from the query point to the nearest edge of each
    /// node's axis-aligned bounding box. If the point is inside a node's bounds,
    /// the distance is zero and that node is always included.
    ///
    /// # Arguments
    /// * `point` - The query point in graph space.
    /// * `radius` - Maximum distance in logical pixels. Must be non-negative.
    pub fn nodes_near_point(&self, point: Vec2, radius: f32) -> Vec<NodeId> {
        let r = radius.max(0.0);
        let r_sq = r * r;

        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                let (nx, ny) = node.position;
                let (nw, nh) = node.size;

                // Compute closest point on the node's AABB to the query point
                let closest_x = point.x.clamp(nx, nx + nw);
                let closest_y = point.y.clamp(ny, ny + nh);

                let dx = point.x - closest_x;
                let dy = point.y - closest_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= r_sq { Some(*id) } else { None }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::FlowEdge;
    use crate::node::FlowNode;
    use crate::port::FlowPort;
    use crate::types::*;

    #[test]
    fn test_graph_add_node_edge() {
        let mut graph = FlowGraph::new();
        let mut n1 = FlowNode::new(NodeId(1), "N1", (0.0, 0.0));
        n1.add_port(FlowPort::new(
            PortId(10),
            NodeId(1),
            PortPosition::Right,
            PortDirection::Output,
        ));
        graph.add_node(n1);

        let mut n2 = FlowNode::new(NodeId(2), "N2", (100.0, 0.0));
        n2.add_port(FlowPort::new(
            PortId(20),
            NodeId(2),
            PortPosition::Left,
            PortDirection::Input,
        ));
        graph.add_node(n2);

        graph.add_edge(FlowEdge::new(100, NodeId(1), 0, NodeId(2), 0));

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);

        let node = graph.get_node_by_port(PortId(10)).unwrap();
        assert_eq!(node.id, NodeId(1));
    }

    #[test]
    fn test_nodes_in_rect_basic() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(NodeId(1), "A", (10.0, 10.0)));
        graph.add_node(FlowNode::new(NodeId(2), "B", (200.0, 200.0)));
        graph.add_node(FlowNode::new(NodeId(3), "C", (50.0, 50.0)));

        // Query that covers A and C but not B
        let result = graph.nodes_in_rect(0.0, 0.0, 120.0, 120.0);
        assert!(result.contains(&NodeId(1)));
        assert!(result.contains(&NodeId(3)));
        assert!(!result.contains(&NodeId(2)));
    }

    #[test]
    fn test_nodes_in_rect_empty() {
        let graph = FlowGraph::new();
        let result = graph.nodes_in_rect(0.0, 0.0, 100.0, 100.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_nodes_in_rect_edge_touching() {
        let mut graph = FlowGraph::new();
        // Node at (0, 0) with default size (150, 80)
        graph.add_node(FlowNode::new(NodeId(1), "A", (0.0, 0.0)));

        // Query rectangle that just touches the right edge of the node
        let result = graph.nodes_in_rect(150.0, 0.0, 50.0, 80.0);
        assert!(result.contains(&NodeId(1)));
    }

    #[test]
    fn test_nodes_near_point_inside() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(NodeId(1), "A", (0.0, 0.0)));

        // Point is inside the node
        let result = graph.nodes_near_point(Vec2::new(10.0, 10.0), 5.0);
        assert!(result.contains(&NodeId(1)));
    }

    #[test]
    fn test_nodes_near_point_outside() {
        let mut graph = FlowGraph::new();
        // Node at (0, 0) with default size (150, 80)
        graph.add_node(FlowNode::new(NodeId(1), "A", (0.0, 0.0)));

        // Point is 50 units to the right of the node (right edge at x=150)
        let result = graph.nodes_near_point(Vec2::new(200.0, 40.0), 60.0);
        assert!(result.contains(&NodeId(1)));

        // Point is 200 units away, radius is 100
        let result = graph.nodes_near_point(Vec2::new(350.0, 40.0), 100.0);
        assert!(!result.contains(&NodeId(1)));
    }

    #[test]
    fn test_nodes_near_point_zero_radius() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(NodeId(1), "A", (0.0, 0.0)));

        // With zero radius, only points exactly on the node boundary are included
        let result = graph.nodes_near_point(Vec2::new(0.0, 0.0), 0.0);
        assert!(result.contains(&NodeId(1)));

        let result = graph.nodes_near_point(Vec2::new(200.0, 200.0), 0.0);
        assert!(!result.contains(&NodeId(1)));
    }

    #[test]
    fn test_nodes_near_point_empty_graph() {
        let graph = FlowGraph::new();
        let result = graph.nodes_near_point(Vec2::new(50.0, 50.0), 100.0);
        assert!(result.is_empty());
    }
}

use crate::edge::FlowEdge;
use crate::node::FlowNode;
use crate::types::{EdgeId, NodeId, PortId};
#[cfg(test)]
use cvkg_core::KvasirId;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

/// A spatial hashing grid to index and accelerate queries on node bounding boxes.
///
/// Divides 2D canvas space into fixed-size grid cells and maps each cell coordinate
/// to a list of node IDs whose bounding boxes overlap the cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialHashGrid {
    /// Dimension of each grid cell.
    pub cell_size: f32,
    /// Maps 2D cell coordinate (x, y) to a list of node IDs.
    pub cells: HashMap<(i32, i32), Vec<NodeId>>,
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self {
            cell_size: 200.0,
            cells: HashMap::new(),
        }
    }
}

impl SpatialHashGrid {
    /// Creates a new spatial hash grid with the specified cell size.
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(1.0),
            cells: HashMap::new(),
        }
    }

    /// Clears all cells in the index.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Indexes a node in the grid based on its bounding box boundaries.
    pub fn insert_node(&mut self, node_id: NodeId, position: (f32, f32), size: (f32, f32)) {
        let (nx, ny) = position;
        let (nw, nh) = size;
        let min_x = (nx / self.cell_size).floor() as i32;
        let max_x = ((nx + nw) / self.cell_size).floor() as i32;
        let min_y = (ny / self.cell_size).floor() as i32;
        let max_y = ((ny + nh) / self.cell_size).floor() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                self.cells.entry((cx, cy)).or_default().push(node_id);
            }
        }
    }
}

/// FlowGraph - collection of nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: HashMap<NodeId, FlowNode>,
    pub edges: HashMap<EdgeId, FlowEdge>,
    /// Index to accelerate spatial queries.
    #[serde(skip)]
    pub spatial_index: RefCell<SpatialHashGrid>,
    /// Tracks if the spatial index needs rebuilding.
    #[serde(skip)]
    pub spatial_index_dirty: Cell<bool>,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            spatial_index: RefCell::new(SpatialHashGrid::default()),
            spatial_index_dirty: Cell::new(true),
        }
    }
}

impl FlowGraph {
    /// Creates a new empty flow graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph and invalidates the spatial index.
    pub fn add_node(&mut self, node: FlowNode) {
        self.nodes.insert(node.id, node);
        self.spatial_index_dirty.set(true);
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
    /// # Contract
    /// Checks cell overlaps in the spatial grid index to quickly filter candidate nodes,
    /// falling back to bounds checking. Resolves queries in average O(1) time.
    pub fn nodes_in_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<NodeId> {
        if self.spatial_index_dirty.get() {
            let mut grid = self.spatial_index.borrow_mut();
            grid.clear();
            for (id, node) in &self.nodes {
                grid.insert_node(*id, node.position, node.size);
            }
            self.spatial_index_dirty.set(false);
        }

        let grid = self.spatial_index.borrow();
        let q_min_x = x;
        let q_min_y = y;
        let q_max_x = x + width;
        let q_max_y = y + height;

        let cell_min_x = (q_min_x / grid.cell_size).floor() as i32;
        let cell_max_x = (q_max_x / grid.cell_size).floor() as i32;
        let cell_min_y = (q_min_y / grid.cell_size).floor() as i32;
        let cell_max_y = (q_max_y / grid.cell_size).floor() as i32;

        let mut matched = HashSet::new();
        for cy in cell_min_y..=cell_max_y {
            for cx in cell_min_x..=cell_max_x {
                if let Some(node_ids) = grid.cells.get(&(cx, cy)) {
                    for &id in node_ids {
                        if let Some(node) = self.nodes.get(&id) {
                            let (nx, ny) = node.position;
                            let (nw, nh) = node.size;
                            let n_max_x = nx + nw;
                            let n_max_y = ny + nh;

                            let intersects = nx <= q_max_x
                                && n_max_x >= q_min_x
                                && ny <= q_max_y
                                && n_max_y >= q_min_y;

                            if intersects {
                                matched.insert(id);
                            }
                        }
                    }
                }
            }
        }

        matched.into_iter().collect()
    }

    /// Returns all node IDs whose bounding rectangles are within `radius` of `point`.
    ///
    /// # Contract
    /// Accelerates search by checking spatial cells overlapping the bounding region of
    /// `point` and `radius`, then validating exact distance. Resolves in average O(1) time.
    pub fn nodes_near_point(&self, point: Vec2, radius: f32) -> Vec<NodeId> {
        if self.spatial_index_dirty.get() {
            let mut grid = self.spatial_index.borrow_mut();
            grid.clear();
            for (id, node) in &self.nodes {
                grid.insert_node(*id, node.position, node.size);
            }
            self.spatial_index_dirty.set(false);
        }

        let grid = self.spatial_index.borrow();
        let r = radius.max(0.0);
        let r_sq = r * r;

        let q_min_x = point.x - r;
        let q_min_y = point.y - r;
        let q_max_x = point.x + r;
        let q_max_y = point.y + r;

        let cell_min_x = (q_min_x / grid.cell_size).floor() as i32;
        let cell_max_x = (q_max_x / grid.cell_size).floor() as i32;
        let cell_min_y = (q_min_y / grid.cell_size).floor() as i32;
        let cell_max_y = (q_max_y / grid.cell_size).floor() as i32;

        let mut matched = HashSet::new();
        for cy in cell_min_y..=cell_max_y {
            for cx in cell_min_x..=cell_max_x {
                if let Some(node_ids) = grid.cells.get(&(cx, cy)) {
                    for &id in node_ids {
                        if let Some(node) = self.nodes.get(&id) {
                            let (nx, ny) = node.position;
                            let (nw, nh) = node.size;

                            let closest_x = point.x.clamp(nx, nx + nw);
                            let closest_y = point.y.clamp(ny, ny + nh);

                            let dx = point.x - closest_x;
                            let dy = point.y - closest_y;
                            let dist_sq = dx * dx + dy * dy;

                            if dist_sq <= r_sq {
                                matched.insert(id);
                            }
                        }
                    }
                }
            }
        }

        matched.into_iter().collect()
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
        let mut n1 = FlowNode::new(KvasirId(1), "N1", (0.0, 0.0));
        n1.add_port(FlowPort::new(
            PortId(10),
            KvasirId(1),
            PortPosition::Right,
            PortDirection::Output,
        ));
        graph.add_node(n1);

        let mut n2 = FlowNode::new(KvasirId(2), "N2", (100.0, 0.0));
        n2.add_port(FlowPort::new(
            PortId(20),
            KvasirId(2),
            PortPosition::Left,
            PortDirection::Input,
        ));
        graph.add_node(n2);

        graph.add_edge(FlowEdge::new(100, KvasirId(1), 0, KvasirId(2), 0));

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);

        let node = graph.get_node_by_port(PortId(10)).unwrap();
        assert_eq!(node.id, KvasirId(1));
    }

    #[test]
    fn test_nodes_in_rect_basic() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(KvasirId(1), "A", (10.0, 10.0)));
        graph.add_node(FlowNode::new(KvasirId(2), "B", (200.0, 200.0)));
        graph.add_node(FlowNode::new(KvasirId(3), "C", (50.0, 50.0)));

        // Query that covers A and C but not B
        let result = graph.nodes_in_rect(0.0, 0.0, 120.0, 120.0);
        assert!(result.contains(&KvasirId(1)));
        assert!(result.contains(&KvasirId(3)));
        assert!(!result.contains(&KvasirId(2)));
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
        graph.add_node(FlowNode::new(KvasirId(1), "A", (0.0, 0.0)));

        // Query rectangle that just touches the right edge of the node
        let result = graph.nodes_in_rect(150.0, 0.0, 50.0, 80.0);
        assert!(result.contains(&KvasirId(1)));
    }

    #[test]
    fn test_nodes_near_point_inside() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(KvasirId(1), "A", (0.0, 0.0)));

        // Point is inside the node
        let result = graph.nodes_near_point(Vec2::new(10.0, 10.0), 5.0);
        assert!(result.contains(&KvasirId(1)));
    }

    #[test]
    fn test_nodes_near_point_outside() {
        let mut graph = FlowGraph::new();
        // Node at (0, 0) with default size (150, 80)
        graph.add_node(FlowNode::new(KvasirId(1), "A", (0.0, 0.0)));

        // Point is 50 units to the right of the node (right edge at x=150)
        let result = graph.nodes_near_point(Vec2::new(200.0, 40.0), 60.0);
        assert!(result.contains(&KvasirId(1)));

        // Point is 200 units away, radius is 100
        let result = graph.nodes_near_point(Vec2::new(350.0, 40.0), 100.0);
        assert!(!result.contains(&KvasirId(1)));
    }

    #[test]
    fn test_nodes_near_point_zero_radius() {
        let mut graph = FlowGraph::new();
        graph.add_node(FlowNode::new(KvasirId(1), "A", (0.0, 0.0)));

        // With zero radius, only points exactly on the node boundary are included
        let result = graph.nodes_near_point(Vec2::new(0.0, 0.0), 0.0);
        assert!(result.contains(&KvasirId(1)));

        let result = graph.nodes_near_point(Vec2::new(200.0, 200.0), 0.0);
        assert!(!result.contains(&KvasirId(1)));
    }

    #[test]
    fn test_nodes_near_point_empty_graph() {
        let graph = FlowGraph::new();
        let result = graph.nodes_near_point(Vec2::new(50.0, 50.0), 100.0);
        assert!(result.is_empty());
    }
}

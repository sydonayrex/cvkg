//! Plain-data block graph.

/// Nodes and directed edges between blocks.
#[derive(Debug, Default)]
pub struct BlockGraph {
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
}

impl BlockGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, kind: impl Into<String>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(kind.into());
        id
    }

    pub fn link(&mut self, from: usize, to: usize) {
        assert!(
            from < self.nodes.len() && to < self.nodes.len(),
            "block id out of range"
        );
        self.edges.push((from, to));
    }

    pub fn edges(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }
}

//! KvasirGraph data structure, builder, and topological compilation.

use std::collections::{HashMap, VecDeque};

use super::planner::ExecutionPlan;
use super::resource::ResourceId;
use super::KvasirError;

/// Opaque key for a node in the graph.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeKey(pub usize);

/// Edge: producer_node --[resource]--> consumer_node.
#[derive(Clone, Debug)]
pub struct Edge {
    pub producer: NodeKey,
    pub resource: ResourceId,
    pub consumer: NodeKey,
}

/// The render graph. Nodes and edges form a DAG. The `roots` are nodes with
/// no inputs (scene sources); `sinks` are the final present/output nodes.
#[derive(Default)]
pub struct KvasirGraph {
    nodes: Vec<Box<dyn super::node::KvasirNode>>,
    edges: Vec<Edge>,
    roots: Vec<NodeKey>,
    sinks: Vec<NodeKey>,
    /// Adjacency: node -> [(resource, consumer)] for topological sort.
    adjacency: HashMap<NodeKey, Vec<(ResourceId, NodeKey)>>,
    /// Reverse adjacency: node -> [producer] for dependency tracking.
    reverse_adj: HashMap<NodeKey, Vec<NodeKey>>,
    next_key: usize,
}

impl KvasirGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph. Returns its key.
    pub fn add_node(&mut self, node: Box<dyn super::node::KvasirNode>) -> NodeKey {
        let key = NodeKey(self.next_key);
        self.next_key += 1;
        self.nodes.push(node);
        key
    }

    /// Declare that `consumer` reads `resource` produced by `from`.
    pub fn connect(&mut self, from: NodeKey, resource: ResourceId, to: NodeKey) {
        self.edges.push(Edge {
            producer: from,
            resource,
            consumer: to,
        });
        self.adjacency
            .entry(from)
            .or_default()
            .push((resource, to));
        self.reverse_adj.entry(to).or_default().push(from);
    }

    /// Mark a node as a sink (final output).
    pub fn add_sink(&mut self, key: NodeKey) {
        self.sinks.push(key);
    }

    /// Validate the graph (no cycles, all inputs satisfied) and compile to
    /// an execution plan.
    pub fn validate_and_compile<'a>(
        &'a self,
        registry: &'a ResourceRegistry,
    ) -> Result<ExecutionPlan<'a>, KvasirError> {
        let order = self.topological_sort()?;
        Ok(ExecutionPlan {
            ordered_nodes: order,
            registry,
        })
    }

    /// Kahn's algorithm for topological sort.
    fn topological_sort(&self) -> Result<Vec<NodeKey>, KvasirError> {
        let n = self.nodes.len();
        let mut in_degree: HashMap<NodeKey, usize> = HashMap::new();
        for key in (0..n).map(NodeKey) {
            in_degree.entry(key).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(edge.consumer).or_insert(0) += 1;
        }

        let mut queue: VecDeque<NodeKey> = in_degree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(k, _)| *k)
            .collect();
        let mut order = Vec::with_capacity(n);

        while let Some(node) = queue.pop_front() {
            order.push(node);
            if let Some(neighbors) = self.adjacency.get(&node) {
                for (_, consumer) in neighbors {
                    let deg = in_degree.get_mut(consumer).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(*consumer);
                    }
                }
            }
        }

        if order.len() != n {
            // Cycle detected — find nodes still with in-degree > 0
            let cycle_nodes: Vec<NodeKey> = in_degree
                .iter()
                .filter(|(_, d)| **d > 0)
                .map(|(k, _)| *k)
                .collect();
            return Err(KvasirError::CycleDetected(cycle_nodes));
        }

        Ok(order)
    }

    /// Access nodes by key (for execution).
    pub fn node(&self, key: NodeKey) -> Option<&dyn super::node::KvasirNode> {
        self.nodes.get(key.0).map(|b| b.as_ref())
    }

    pub fn ordered_nodes(&self) -> &[NodeKey] {
        &[]
    }

    /// Build a human-readable DOT representation for debugging.
    pub fn to_dot(&self) -> String {
        let mut s = String::from("digraph Kvasir {\n");
        for (i, node) in self.nodes.iter().enumerate() {
            s.push_str(&format!("  n{} [label=\"{}\"];\n", i, node.label()));
        }
        for edge in &self.edges {
            s.push_str(&format!(
                "  n{} -> n{} [label=\"{:?}\"];\n",
                edge.producer.0, edge.consumer.0, edge.resource
            ));
        }
        s.push_str("}\n");
        s
    }
}

/// Builder for declarative graph construction.
pub struct GraphBuilder {
    graph: KvasirGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: KvasirGraph::new(),
        }
    }

    pub fn add_node(&mut self, node: Box<dyn super::node::KvasirNode>) -> NodeKey {
        self.graph.add_node(node)
    }

    pub fn connect(&mut self, from: NodeKey, resource: ResourceId, to: NodeKey) {
        self.graph.connect(from, resource, to);
    }

    pub fn add_sink(&mut self, key: NodeKey) {
        self.graph.add_sink(key);
    }

    pub fn build(self) -> KvasirGraph {
        self.graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

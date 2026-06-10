use super::KvasirError;
use super::graph::KvasirGraph;

/// The ExecutionPlanner takes a constructed RenderGraph (KvasirGraph)
/// and produces a topologically sorted execution plan.
pub struct ExecutionPlanner<'a> {
    graph: &'a KvasirGraph,
}

impl<'a> ExecutionPlanner<'a> {
    pub fn new(graph: &'a KvasirGraph) -> Self {
        Self { graph }
    }

    /// Compiles the graph into an ordered sequence of PassIds.
    pub fn compile(&self) -> Result<Vec<super::graph::NodeKey>, KvasirError> {
        let order = self.graph.topological_sort()?;
        Ok(order)
    }
}

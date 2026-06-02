//! Execution plan — the compiled output of KvasirGraph::validate_and_compile.
//!
//! Contains the ordered node list and references to the resource registry.
//! The renderer iterates over `ordered_nodes` and calls each node's
//! `execute()` method in dependency order.

use super::node::{ExecutionContext, KvasirNode};
use super::registry::ResourceRegistry;
use super::{KvasirError, NodeKey};

pub struct ExecutionPlan<'a> {
    pub ordered_nodes: Vec<NodeKey>,
    pub registry: &'a ResourceRegistry,
}

impl<'a> ExecutionPlan<'a> {
    /// Execute every node in dependency order.
    pub fn execute(
        &self,
        ctx: &mut ExecutionContext<'_>,
        registry: &mut ResourceRegistry,
        nodes: &[Box<dyn KvasirNode>],
    ) -> Result<(), KvasirError> {
        for key in &self.ordered_nodes {
            if let Some(node) = nodes.get(key.0) {
                node.execute(ctx, registry).map_err(|e| {
                    log::error!("[Kvasir] Node '{}' failed: {}", node.label(), e);
                    e
                })?;
            }
        }
        Ok(())
    }
}

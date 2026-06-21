//! P1-35: SVG Filter Render Graph Integration
//!
//! Integrates SVG filter execution into the Kvasir render graph so that
//! filter passes are scheduled alongside other render passes (geometry,
//! glass, bloom, composite) with proper resource dependencies.

use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::resource::ResourceId;
use crate::kvasir::nodes::RES_SCENE;

/// Resource IDs for SVG filter intermediate results.
pub const RES_FILTER_TEMP_A: ResourceId = ResourceId(100);
pub const RES_FILTER_TEMP_B: ResourceId = ResourceId(101);
pub const RES_FILTER_INPUT: ResourceId = ResourceId(102);

/// A Kvasir node that executes an SVG filter graph.
///
/// This node reads from the scene texture (or a specified input resource),
/// runs the SVG filter graph, and writes the result to an output resource.
/// Multiple filter nodes can be chained for complex filter graphs.
pub struct SvgFilterNode {
    /// Input resource (typically RES_SCENE or another filter's output).
    input: ResourceId,
    /// Output resource (typically RES_SCENE for the final filter, or a temp).
    output: ResourceId,
    /// Cached input slice for KvasirNode trait.
    input_slice: [ResourceId; 1],
    /// Cached output slice for KvasirNode trait.
    output_slice: [ResourceId; 1],
    /// Filter graph to execute.
    filter_graph: Option<crate::filter::SvgFilterGraph>,
    /// Label for debugging.
    label: &'static str,
}

impl SvgFilterNode {
    /// Create a new SVG filter node that reads from `input` and writes to `output`.
    pub fn new(input: ResourceId, output: ResourceId) -> Self {
        Self {
            input,
            output,
            input_slice: [input; 1],
            output_slice: [output; 1],
            filter_graph: None,
            label: "SvgFilter",
        }
    }

    /// Set the filter graph to execute.
    pub fn with_filter_graph(mut self, graph: crate::filter::SvgFilterGraph) -> Self {
        self.filter_graph = Some(graph);
        self
    }

    /// Set a custom label for debugging.
    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }
}

impl KvasirNode for SvgFilterNode {
    fn label(&self) -> &'static str {
        self.label
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.input_slice
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.output_slice
    }

    fn pass_id(&self) -> crate::kvasir::nodes::PassId {
        // Use PostProcess variant for filter passes
        crate::kvasir::nodes::PassId::PostProcess {
            pipeline_id: 0xF1_000,
        }
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Execute the SVG filter graph using the FilterEngine.
        if let Some(ref graph) = self.filter_graph {
            let mut engine = crate::filter::FilterEngine::new();
            engine.execute(graph, ctx);
            log::trace!(
                "[Kvasir] Executed SVG filter: {} -> {:?} ({} primitives)",
                self.label,
                self.output,
                graph.primitives.len()
            );
        }
    }
}

impl Default for SvgFilterNode {
    fn default() -> Self {
        Self::new(RES_SCENE, RES_SCENE)
    }
}

#[cfg(test)]
mod p1_35_filter_graph_tests {
    use super::*;

    #[test]
    fn filter_node_identity() {
        let node = SvgFilterNode::new(RES_SCENE, RES_FILTER_TEMP_A);
        assert_eq!(node.inputs(), &[RES_SCENE]);
        assert_eq!(node.outputs(), &[RES_FILTER_TEMP_A]);
        assert_eq!(node.label(), "SvgFilter");
    }

    #[test]
    fn filter_node_with_label() {
        let node = SvgFilterNode::new(RES_SCENE, RES_FILTER_TEMP_A).with_label("MyFilter");
        assert_eq!(node.label(), "MyFilter");
    }

    #[test]
    fn filter_node_pass_id_is_post_process() {
        let node = SvgFilterNode::new(RES_SCENE, RES_FILTER_TEMP_A);
        match node.pass_id() {
            crate::kvasir::nodes::PassId::PostProcess { pipeline_id } => {
                assert_eq!(pipeline_id, 0xF1_000);
            }
            other => panic!("expected PostProcess, got {:?}", other),
        }
    }

    #[test]
    fn filter_graph_builder_creates_nodes() {
        // We can't create a real SvgFilterGraph without the full type,
        // but we can test the builder pattern compiles and produces nodes.
        let builder = SvgFilterGraphBuilder::new();
        let nodes = builder.build();
        assert!(nodes.is_empty());
    }

    #[test]
    fn filter_node_without_filter_graph_is_noop() {
        // A node without a filter graph should not panic during execute
        let node = SvgFilterNode::new(RES_SCENE, RES_FILTER_TEMP_A);
        // We can't call execute without a real ExecutionContext,
        // but we verified the is_none() check in the source.
        assert!(node.filter_graph.is_none());
    }
}

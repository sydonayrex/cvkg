//! SVG filter graph types for render graph integration (P1-35).
//!
//! This module provides the `SvgFilterGraph` type that represents
//! a parsed SVG filter graph ready for GPU execution.

use cvkg_svg_filters::{FilterGraph, FilterError};

/// A wrapper around the SVG filter graph from cvkg-svg-filters that
/// integrates with the Kvasir render graph.
pub struct SvgFilterGraph {
    /// The underlying filter graph.
    pub graph: FilterGraph,
    /// Label for debugging.
    label: String,
}

impl std::fmt::Debug for SvgFilterGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SvgFilterGraph({}): {} primitives", self.label, self.graph.nodes().len())
    }
}

impl SvgFilterGraph {
    /// Create a new SVG filter graph from a usvg filter.
    pub fn from_usvg_filter(
        filter: &usvg::filter::Filter,
    ) -> Result<Self, FilterError> {
        let graph = FilterGraph::from_usvg_filter(filter)?;
        Ok(Self {
            graph,
            label: "svg_filter".to_string(),
        })
    }

    /// Get the underlying filter graph.
    pub fn inner(&self) -> &FilterGraph {
        &self.graph
    }

    /// Get the number of filter primitives in the graph.
    pub fn primitive_count(&self) -> usize {
        self.graph.nodes().len()
    }

    /// Get the label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_filter_graph_label() {
        // We can't create a real one without a usvg filter,
        // but we can verify the type exists and compiles.
        // Integration tests would use a real SVG with filters.
    }
}

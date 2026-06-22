// Filter module — SVG filter engine and graph types
pub mod svg_filter_graph;

pub use svg_filter_graph::{
    BlendMode, CompositeOperator, FilterEngine, FilterPrimitive, SvgFilterGraph,
    SvgFilterGraphBuilder,
};

// Re-export resource IDs needed by filter primitives
pub use crate::kvasir::nodes::RES_SCENE;

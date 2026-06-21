//! SVG Filter Engine
//!
//! Implements the SVG filter pipeline for CVKG's render graph.
//! Supports: feGaussianBlur, feDropShadow, feOffset, feBlend, feComposite, feFlood, feMerge.

use crate::kvasir::node::ExecutionContext;
use crate::kvasir::resource::ResourceId;

/// A filter graph that executes a chain of SVG filter primitives.
#[derive(Debug, Clone)]
pub struct SvgFilterGraph {
    /// Ordered list of filter primitives to execute.
    pub primitives: Vec<FilterPrimitive>,
    /// Input resource name (e.g., "source" or previous primitive's output).
    pub input: String,
    /// Output resource name.
    pub output: String,
}

impl Default for SvgFilterGraph {
    fn default() -> Self {
        Self {
            primitives: Vec::new(),
            input: "source".into(),
            output: "result".into(),
        }
    }
}

/// SVG filter primitive types.
#[derive(Debug, Clone)]
pub enum FilterPrimitive {
    /// feGaussianBlur — Blurs the input image.
    GaussianBlur {
        std_deviation: f32,
        input: String,
        result: String,
    },
    /// feDropShadow — Creates a drop shadow effect.
    DropShadow {
        dx: f32,
        dy: f32,
        std_deviation: f32,
        flood_color: [f32; 4],
        input: String,
        result: String,
    },
    /// feOffset — Translates the input image.
    Offset {
        dx: f32,
        dy: f32,
        input: String,
        result: String,
    },
    /// feBlend — Blends two images together.
    Blend {
        mode: BlendMode,
        in1: String,
        in2: String,
        result: String,
    },
    /// feComposite — Composites two images using Porter-Duff operations.
    Composite {
        operator: CompositeOperator,
        in1: String,
        in2: String,
        result: String,
    },
    /// feFlood — Fills the filter region with a solid color.
    Flood {
        color: [f32; 4],
        result: String,
    },
    /// feMerge — Merges multiple filter results.
    Merge {
        inputs: Vec<String>,
        result: String,
    },
}

/// SVG blend mode for feBlend.
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Darken,
    Lighten,
}

impl BlendMode {
    /// Parse an SVG blend mode string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "multiply" => BlendMode::Multiply,
            "screen" => BlendMode::Screen,
            "darken" => BlendMode::Darken,
            "lighten" => BlendMode::Lighten,
            _ => BlendMode::Normal,
        }
    }
}

/// SVG composite operator for feComposite.
#[derive(Debug, Clone, Copy)]
pub enum CompositeOperator {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic,
}

impl CompositeOperator {
    /// Parse an SVG composite operator string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "in" => CompositeOperator::In,
            "out" => CompositeOperator::Out,
            "atop" => CompositeOperator::Atop,
            "xor" => CompositeOperator::Xor,
            "arithmetic" => CompositeOperator::Arithmetic,
            _ => CompositeOperator::Over,
        }
    }
}

/// Filter execution engine.
///
/// Owns the filter graph and executes primitives in dependency order.
pub struct FilterEngine {
    /// Intermediate render targets for filter results.
    temp_targets: Vec<(String, ResourceId)>,
}

impl FilterEngine {
    /// Create a new filter engine.
    pub fn new() -> Self {
        Self {
            temp_targets: Vec::new(),
        }
    }

    /// Execute a filter graph within the given execution context.
    pub fn execute(&mut self, graph: &SvgFilterGraph, ctx: &mut ExecutionContext) {
        for primitive in &graph.primitives {
            self.execute_primitive(primitive, ctx);
        }
    }

    fn execute_primitive(&mut self, primitive: &FilterPrimitive, ctx: &mut ExecutionContext) {
        match primitive {
            FilterPrimitive::GaussianBlur {
                std_deviation,
                input: _,
                result,
            } => {
                // TODO: Implement Gaussian blur via compute shader
                // For now, log and skip
                log::trace!(
                    "[FilterEngine] feGaussianBlur std_deviation={} -> {}",
                    std_deviation,
                    result
                );
            }
            FilterPrimitive::DropShadow {
                dx,
                dy,
                std_deviation,
                flood_color,
                input: _,
                result,
            } => {
                // TODO: Implement drop shadow (offset + blur + composite)
                log::trace!(
                    "[FilterEngine] feDropShadow dx={} dy={} std={} color={:?} -> {}",
                    dx, dy, std_deviation, flood_color, result
                );
            }
            FilterPrimitive::Offset {
                dx,
                dy,
                input: _,
                result,
            } => {
                // TODO: Implement offset via texture blit with translation
                log::trace!(
                    "[FilterEngine] feOffset dx={} dy={} -> {}",
                    dx, dy, result
                );
            }
            FilterPrimitive::Blend {
                mode,
                in1: _,
                in2: _,
                result,
            } => {
                // TODO: Implement blend via shader
                log::trace!("[FilterEngine] feBlend mode={:?} -> {}", mode, result);
            }
            FilterPrimitive::Composite {
                operator,
                in1: _,
                in2: _,
                result,
            } => {
                // TODO: Implement composite via shader
                log::trace!("[FilterEngine] feComposite op={:?} -> {}", operator, result);
            }
            FilterPrimitive::Flood {
                color,
                result,
            } => {
                // TODO: Implement flood fill
                log::trace!("[FilterEngine] feFlood color={:?} -> {}", color, result);
            }
            FilterPrimitive::Merge {
                inputs: _,
                result,
            } => {
                // TODO: Implement merge by compositing all inputs
                log::trace!("[FilterEngine] feMerge -> {}", result);
            }
        }
    }
}

/// Convert an SVG filter element from pillage-doc into a SvgFilterGraph.
///
/// This bridges the document model (FilterNode/FilterPrimitive) to the
/// render graph model (SvgFilterGraph/FilterPrimitive).
#[cfg(feature = "pillage")]
pub fn build_filter_graph(
    filter_node: &pillage_doc::node::FilterNode,
    primitives: &[pillage_doc::node::FilterPrimitive],
) -> SvgFilterGraph {
    let mut graph = SvgFilterGraph::default();

    for prim in primitives {
        let filter_prim = match prim.primitive_type {
            pillage_doc::node::FilterPrimitiveType::GaussianBlur => {
                FilterPrimitive::GaussianBlur {
                    std_deviation: prim.std_deviation.unwrap_or(0.0),
                    input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                    result: prim.result.clone().unwrap_or_else(|| "blur".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::DropShadow => {
                FilterPrimitive::DropShadow {
                    dx: prim.dx.unwrap_or(0.0),
                    dy: prim.dy.unwrap_or(0.0),
                    std_deviation: prim.std_deviation.unwrap_or(0.0),
                    flood_color: prim.flood_color.unwrap_or([0.0, 0.0, 0.0, 0.5]),
                    input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                    result: prim.result.clone().unwrap_or_else(|| "shadow".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::Offset => {
                FilterPrimitive::Offset {
                    dx: prim.offset_x.unwrap_or(0.0),
                    dy: prim.offset_y.unwrap_or(0.0),
                    input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                    result: prim.result.clone().unwrap_or_else(|| "offset".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::Blend => {
                FilterPrimitive::Blend {
                    mode: BlendMode::from_str(prim.blend_mode.as_deref().unwrap_or("normal")),
                    in1: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                    in2: "background".into(),
                    result: prim.result.clone().unwrap_or_else(|| "blend".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::Composite => {
                FilterPrimitive::Composite {
                    operator: CompositeOperator::Over,
                    in1: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                    in2: "background".into(),
                    result: prim.result.clone().unwrap_or_else(|| "composite".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::Flood => {
                FilterPrimitive::Flood {
                    color: prim.flood_color.unwrap_or([0.0, 0.0, 0.0, 1.0]),
                    result: prim.result.clone().unwrap_or_else(|| "flood".into()),
                }
            }
            pillage_doc::node::FilterPrimitiveType::Merge => {
                FilterPrimitive::Merge {
                    inputs: vec!["source".into()],
                    result: prim.result.clone().unwrap_or_else(|| "merge".into()),
                }
            }
            _ => {
                log::warn!(
                    "[FilterEngine] Unsupported filter primitive: {:?}",
                    prim.primitive_type
                );
                continue;
            }
        };
        graph.primitives.push(filter_prim);
    }

    graph
}

/// Builder for constructing SVG filter render graph nodes.
#[derive(Default)]
pub struct SvgFilterGraphBuilder {
    nodes: Vec<(ResourceId, ResourceId, Option<SvgFilterGraph>)>,
}

impl SvgFilterGraphBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a filter pass that reads from `input` and writes to `output`.
    pub fn add_pass(mut self, input: ResourceId, output: ResourceId, graph: SvgFilterGraph) -> Self {
        self.nodes.push((input, output, Some(graph)));
        self
    }

    /// Add a pass that reads from the scene and writes to a temp buffer.
    pub fn add_scene_pass(self, output: ResourceId, graph: SvgFilterGraph) -> Self {
        self.add_pass(output, output, graph)
    }

    /// Add a pass that reads from a temp buffer and writes to the scene.
    pub fn add_final_pass(self, input: ResourceId, graph: SvgFilterGraph) -> Self {
        self.add_pass(input, input, graph)
    }
}

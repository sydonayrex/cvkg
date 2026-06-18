use crate::material::MaterialValidationConfig;
use crate::material::{MaterialGraph, MaterialSocket};

#[derive(Debug)]
pub enum GeneratorError {
    ChannelClosed,
    UnknownNodeType(String),
    UnknownNode(String),
    ValidationFailed(String),
    Other(String),
}

/// A stable API for AI systems (LLMs, diffusion models) to generate
/// and inject materials or subgraphs into the Kvasir renderer.
pub trait KvasirGenerator: Send + Sync {
    fn generate(&self) -> Result<(), GeneratorError>;
}

/// JSON representation of a Material Graph, allowing AI to author materials safely.
#[derive(Debug, serde::Deserialize)]
pub struct MaterialGraphSpec {
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<EdgeSpec>,
}

#[derive(Debug, serde::Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub kind: String,
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EdgeSpec {
    pub from_node: String,
    pub from_socket: String,
    pub to_node: String,
    pub to_socket: String,
}

impl MaterialGraphSpec {
    pub fn build_graph(&self) -> Result<MaterialGraph, GeneratorError> {
        let mut mat = MaterialGraph::new();
        let mut node_map = std::collections::HashMap::new();

        for node_spec in &self.nodes {
            let key = match node_spec.kind.as_str() {
                "SolidColor" => {
                    let r = node_spec
                        .params
                        .get("r")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32;
                    let g = node_spec
                        .params
                        .get("g")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32;
                    let b = node_spec
                        .params
                        .get("b")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32;
                    let a = node_spec
                        .params
                        .get("a")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32;
                    mat.add_node(crate::material::MaterialOp::ConstantColor { r, g, b, a })
                }
                "Output" => {
                    // special node just to mark output
                    // since Output is an operation, wait, MaterialGraph sets output by id
                    u32::MAX
                }
                kind => return Err(GeneratorError::UnknownNodeType(kind.to_string())),
            };
            if key != u32::MAX {
                node_map.insert(node_spec.id.clone(), key);
            } else {
                // If this is the Output spec node, it means whatever it connects to is the output.
                // Wait, it's easier to just find the edge going into Output.
            }
        }

        // Output node discovery
        let output_edge = self.edges.iter().find(|e| e.to_node == "Output");
        if let Some(edge) = output_edge {
            let from = node_map
                .get(&edge.from_node)
                .ok_or_else(|| GeneratorError::UnknownNode(edge.from_node.clone()))?;
            mat.set_output(*from);
        } else {
            return Err(GeneratorError::ValidationFailed(
                "No edge to Output node".into(),
            ));
        }

        for edge in &self.edges {
            if edge.to_node == "Output" {
                continue;
            }
            let from = node_map
                .get(&edge.from_node)
                .ok_or_else(|| GeneratorError::UnknownNode(edge.from_node.clone()))?;
            let to = node_map
                .get(&edge.to_node)
                .ok_or_else(|| GeneratorError::UnknownNode(edge.to_node.clone()))?;

            mat.connect(*from, MaterialSocket::Color, *to, MaterialSocket::Color);
        }

        mat.validate_with_config(&MaterialValidationConfig { max_nodes: 32, max_edges: 64 })
            .map_err(|e| GeneratorError::ValidationFailed(e.to_string()))?;

        Ok(mat)
    }
}

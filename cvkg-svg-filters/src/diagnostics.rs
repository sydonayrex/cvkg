use crate::graph::{FilterGraph, FilterInput};

// =============================================================================
// P2-30: Node-Level Filter Diagnostics
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct FilterDiagnostic {
    pub severity: DiagnosticSeverity,
    pub node_index: usize,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct FilterDiagnostics {
    messages: Vec<FilterDiagnostic>,
}

impl FilterDiagnostics {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }

    pub fn error(&mut self, node_index: usize, message: impl Into<String>) {
        self.messages.push(FilterDiagnostic {
            severity: DiagnosticSeverity::Error,
            node_index,
            message: message.into(),
        });
    }

    pub fn warning(&mut self, node_index: usize, message: impl Into<String>) {
        self.messages.push(FilterDiagnostic {
            severity: DiagnosticSeverity::Warning,
            node_index,
            message: message.into(),
        });
    }

    pub fn info(&mut self, node_index: usize, message: impl Into<String>) {
        self.messages.push(FilterDiagnostic {
            severity: DiagnosticSeverity::Info,
            node_index,
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.messages.iter().any(|m| m.severity == DiagnosticSeverity::Error)
    }

    pub fn messages(&self) -> &[FilterDiagnostic] {
        &self.messages
    }

    pub fn for_node(&self, node_index: usize) -> Vec<&FilterDiagnostic> {
        self.messages.iter().filter(|m| m.node_index == node_index).collect()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

// =============================================================================
// P2-31: Filter Graph Visualization
// =============================================================================

#[derive(Clone, Debug)]
pub struct FilterNodeView {
    pub index: usize,
    pub result_name: String,
    pub inputs: Vec<String>,
    pub kind: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct FilterEdgeView {
    pub from: usize,
    pub to: usize,
    pub resource: String,
}

#[derive(Clone, Debug)]
pub struct FilterGraphView {
    pub nodes: Vec<FilterNodeView>,
    pub edges: Vec<FilterEdgeView>,
}

impl FilterGraphView {
    pub fn from_graph(graph: &FilterGraph) -> Self {
        let nodes: Vec<FilterNodeView> = graph
            .nodes()
            .iter()
            .map(|node| FilterNodeView {
                index: node.index,
                result_name: node.result_name.clone(),
                inputs: node.inputs.iter().map(|i| format!("{:?}", i)).collect(),
                kind: format!("{:?}", node.kind),
                label: if node.result_name.is_empty() {
                    format!("{:?}", node.kind)
                } else {
                    node.result_name.clone()
                },
            })
            .collect();

        let mut edges = Vec::new();
        for (i, node) in graph.nodes().iter().enumerate() {
            for input in &node.inputs {
                if let FilterInput::Reference(name) = input {
                    for (j, other) in graph.nodes().iter().enumerate() {
                        if other.result_name == *name {
                            edges.push(FilterEdgeView {
                                from: j,
                                to: i,
                                resource: name.clone(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        Self { nodes, edges }
    }

    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n  \"nodes\": [\n");
        for (i, node) in self.nodes.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!(
                "    {{\"index\": {}, \"name\": \"{}\", \"kind\": \"{}\"}}",
                node.index, node.label, node.kind
            ));
        }
        json.push_str("\n  ],\n  \"edges\": [\n");
        for (i, edge) in self.edges.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!(
                "    {{\"from\": {}, \"to\": {}, \"resource\": \"{}\"}}",
                edge.from, edge.to, edge.resource
            ));
        }
        json.push_str("\n  ]\n}");
        json
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph filter_graph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n");

        for node in &self.nodes {
            dot.push_str(&format!(
                "  {} [label=\"{}\"];\n",
                node.index, node.label
            ));
        }

        for edge in &self.edges {
            dot.push_str(&format!(
                "  {} -> {} [label=\"{}\"];\n",
                edge.from, edge.to, edge.resource
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_diagnostics_new() {
        let diag = FilterDiagnostics::new();
        assert!(!diag.has_errors());
        assert!(diag.messages().is_empty());
    }

    #[test]
    fn filter_diagnostics_error() {
        let mut diag = FilterDiagnostics::new();
        diag.error(0, "missing input");
        assert!(diag.has_errors());
        assert_eq!(diag.messages().len(), 1);
    }

    #[test]
    fn filter_graph_view_to_json() {
        let view = FilterGraphView {
            nodes: vec![FilterNodeView {
                index: 0,
                result_name: "blur".to_string(),
                inputs: vec!["SourceGraphic".to_string()],
                kind: "GaussianBlur".to_string(),
                label: "blur".to_string(),
            }],
            edges: vec![],
        };
        let json = view.to_json();
        assert!(json.contains("blur"));
    }
}

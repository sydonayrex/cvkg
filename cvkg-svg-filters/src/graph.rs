use std::collections::{HashMap, VecDeque};
use crate::types::{FilterError, ResolvedInput};
use crate::validators::{LightingValidator, TurbulenceValidator};

// ── Filter Graph ─────────────────────────────────────────────────────────────

/// A node in the filter DAG.
#[derive(Debug)]
pub struct FilterNode {
    /// Index into the primitives array.
    pub index: usize,
    /// The result name (for `in`/`result` references).
    pub result_name: String,
    /// Input slots: which other nodes (or special inputs) feed into this one.
    pub inputs: Vec<FilterInput>,
    /// The primitive kind.
    pub kind: usvg::filter::Kind,
    /// Filter subregion (may be different from the overall filter rect).
    pub rect: usvg::NonZeroRect,
}

/// Resolved input reference for a filter node.
#[derive(Debug, Clone)]
pub enum FilterInput {
    /// The original source graphic.
    SourceGraphic,
    /// The source graphic's alpha channel.
    SourceAlpha,
    /// The backdrop image (rendered behind the element).
    BackdropImage,
    /// The backdrop image's alpha channel.
    BackdropAlpha,
    /// Output of another filter primitive by result name.
    Reference(String),
}

/// Directed acyclic graph of filter primitives.
pub struct FilterGraph {
    /// Nodes in topological evaluation order.
    pub nodes: Vec<FilterNode>,
    /// Map from result name -> node index.
    pub name_to_index: HashMap<String, usize>,
}

impl FilterGraph {
    /// Build a `FilterGraph` from a `usvg::filter::Filter`.
    pub fn from_usvg_filter(filter: &usvg::filter::Filter) -> Result<Self, FilterError> {
        let primitives = filter.primitives();
        let mut nodes = Vec::with_capacity(primitives.len());
        let mut name_to_index: HashMap<String, usize> = HashMap::new();

        // First pass: create nodes and build name map.
        for (i, prim) in primitives.iter().enumerate() {
            let result_name = prim.result().to_string();
            let inputs = Self::resolve_inputs(prim.kind());
            let kind = prim.kind().clone();
            let rect = prim.rect();

            // Validate lighting and turbulence parameters
            match &kind {
                usvg::filter::Kind::DiffuseLighting(dl) => {
                    LightingValidator::validate_diffuse_lighting(
                        dl.surface_scale(),
                        dl.diffuse_constant(),
                        None,
                    )?;
                }
                usvg::filter::Kind::SpecularLighting(sl) => {
                    LightingValidator::validate_specular_lighting(
                        sl.surface_scale(),
                        sl.specular_constant(),
                        sl.specular_exponent(),
                    )?;
                }
                usvg::filter::Kind::Turbulence(t) => {
                    TurbulenceValidator::validate_turbulence(
                        t.base_frequency_x().get(),
                        t.base_frequency_y().get(),
                        t.num_octaves() as i32,
                        t.seed(),
                        t.stitch_tiles(),
                    )?;
                }
                _ => {}
            }

            if !result_name.is_empty() {
                name_to_index.insert(result_name.clone(), i);
            }

            nodes.push(FilterNode {
                index: i,
                result_name,
                inputs,
                kind,
                rect,
            });
        }

        // Topological sort: Kahn's algorithm.
        let sorted = Self::topological_sort(&nodes)?;

        Ok(FilterGraph {
            nodes: sorted,
            name_to_index,
        })
    }

    /// Return the nodes in evaluation order.
    pub fn nodes(&self) -> &[FilterNode] {
        &self.nodes
    }

    /// Resolve a `FilterInput` to either a special source or a node index.
    pub fn resolve_input(&self, input: &FilterInput) -> Result<ResolvedInput, FilterError> {
        match input {
            FilterInput::SourceGraphic => Ok(ResolvedInput::SourceGraphic),
            FilterInput::SourceAlpha => Ok(ResolvedInput::SourceAlpha),
            FilterInput::BackdropImage => Ok(ResolvedInput::BackdropImage),
            FilterInput::BackdropAlpha => Ok(ResolvedInput::BackdropAlpha),
            FilterInput::Reference(name) => {
                if let Some(&idx) = self.name_to_index.get(name) {
                    Ok(ResolvedInput::NodeIndex(idx))
                } else {
                    // Check if it's a special input that was stored as a reference.
                    match name.as_str() {
                        "SourceGraphic" => Ok(ResolvedInput::SourceGraphic),
                        "SourceAlpha" => Ok(ResolvedInput::SourceAlpha),
                        "BackgroundImage" => Ok(ResolvedInput::BackdropImage),
                        "BackgroundAlpha" => Ok(ResolvedInput::BackdropAlpha),
                        _ => Err(FilterError::UnresolvedInput(name.clone())),
                    }
                }
            }
        }
    }

    /// Resolve the `usvg::Input` enum to our `FilterInput`.
    fn resolve_inputs(kind: &usvg::filter::Kind) -> Vec<FilterInput> {
        match kind {
            usvg::filter::Kind::Blend(blend) => {
                vec![
                    Self::input_to_filter_input(blend.input1()),
                    Self::input_to_filter_input(blend.input2()),
                ]
            }
            usvg::filter::Kind::ColorMatrix(cm) => {
                vec![Self::input_to_filter_input(cm.input())]
            }
            usvg::filter::Kind::ComponentTransfer(ct) => {
                vec![Self::input_to_filter_input(ct.input())]
            }
            usvg::filter::Kind::Composite(comp) => {
                vec![
                    Self::input_to_filter_input(comp.input1()),
                    Self::input_to_filter_input(comp.input2()),
                ]
            }
            usvg::filter::Kind::ConvolveMatrix(cm) => {
                vec![Self::input_to_filter_input(cm.input())]
            }
            usvg::filter::Kind::DiffuseLighting(dl) => {
                vec![Self::input_to_filter_input(dl.input())]
            }
            usvg::filter::Kind::DisplacementMap(dm) => {
                vec![
                    Self::input_to_filter_input(dm.input1()),
                    Self::input_to_filter_input(dm.input2()),
                ]
            }
            usvg::filter::Kind::DropShadow(ds) => {
                vec![Self::input_to_filter_input(ds.input())]
            }
            usvg::filter::Kind::Flood(_) => vec![],
            usvg::filter::Kind::GaussianBlur(gb) => {
                vec![Self::input_to_filter_input(gb.input())]
            }
            usvg::filter::Kind::Image(_) => vec![],
            usvg::filter::Kind::Merge(merge) => merge
                .inputs()
                .iter()
                .map(Self::input_to_filter_input)
                .collect(),
            usvg::filter::Kind::Morphology(m) => {
                vec![Self::input_to_filter_input(m.input())]
            }
            usvg::filter::Kind::Offset(o) => {
                vec![Self::input_to_filter_input(o.input())]
            }
            usvg::filter::Kind::SpecularLighting(sl) => {
                vec![Self::input_to_filter_input(sl.input())]
            }
            usvg::filter::Kind::Tile(t) => {
                vec![Self::input_to_filter_input(t.input())]
            }
            usvg::filter::Kind::Turbulence(_) => vec![],
        }
    }

    fn input_to_filter_input(input: &usvg::filter::Input) -> FilterInput {
        match input {
            usvg::filter::Input::SourceGraphic => FilterInput::SourceGraphic,
            usvg::filter::Input::SourceAlpha => FilterInput::SourceAlpha,
            usvg::filter::Input::Reference(name) => match name.as_str() {
                "BackgroundImage" => FilterInput::BackdropImage,
                "BackgroundAlpha" => FilterInput::BackdropAlpha,
                _ => FilterInput::Reference(name.clone()),
            },
        }
    }

    /// Topological sort using Kahn's algorithm.
    pub(crate) fn topological_sort(nodes: &[FilterNode]) -> Result<Vec<FilterNode>, FilterError> {
        let n = nodes.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let name_to_idx: HashMap<String, usize> = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.result_name.is_empty())
            .map(|(i, node)| (node.result_name.clone(), i))
            .collect();

        let mut in_degree = vec![0u32; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (i, node) in nodes.iter().enumerate() {
            for input in &node.inputs {
                if let FilterInput::Reference(name) = input
                    && let Some(&dep_idx) = name_to_idx.get(name)
                {
                    adj[dep_idx].push(i);
                    in_degree[i] += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut sorted = Vec::with_capacity(n);

        while let Some(idx) = queue.pop_front() {
            sorted.push(FilterNode {
                index: nodes[idx].index,
                result_name: nodes[idx].result_name.clone(),
                inputs: nodes[idx].inputs.clone(),
                kind: nodes[idx].kind.clone(),
                rect: nodes[idx].rect,
            });
            for &next in &adj[idx] {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }

        if sorted.len() != n {
            return Err(FilterError::CyclicGraph);
        }

        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn flood_kind() -> usvg::filter::Kind {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
            <defs>
                <filter id="f1">
                    <feFlood flood-color="red" flood-opacity="1"/>
                </filter>
            </defs>
            <rect width="100" height="100" filter="url(#f1)"/>
        </svg>"#;
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
        let root = tree.root();
        find_first_filter_kind(root).expect("should find flood filter in parsed SVG")
    }

    fn find_first_filter_kind(group: &usvg::Group) -> Option<usvg::filter::Kind> {
        for child in group.children() {
            if let usvg::Node::Group(g) = child {
                for filter in g.filters() {
                    if let Some(prim) = filter.primitives().first() {
                        return Some(prim.kind().clone());
                    }
                }
                if let Some(kind) = find_first_filter_kind(g) {
                    return Some(kind);
                }
            }
        }
        None
    }

    #[test]
    fn test_filter_graph_empty() {
        let nodes: Vec<FilterNode> = vec![];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_filter_graph_single_node() {
        let nodes = vec![FilterNode {
            index: 0,
            result_name: "out".to_string(),
            inputs: vec![FilterInput::SourceGraphic],
            kind: flood_kind(),
            rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
        }];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert_eq!(sorted.len(), 1);
    }

    #[test]
    fn test_filter_graph_chain() {
        let nodes = vec![
            FilterNode {
                index: 0,
                result_name: "a".to_string(),
                inputs: vec![FilterInput::SourceGraphic],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
            FilterNode {
                index: 1,
                result_name: "b".to_string(),
                inputs: vec![FilterInput::Reference("a".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
        ];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].result_name, "a");
        assert_eq!(sorted[1].result_name, "b");
    }

    #[test]
    fn test_filter_graph_cycle_detection() {
        let nodes = vec![
            FilterNode {
                index: 0,
                result_name: "a".to_string(),
                inputs: vec![FilterInput::Reference("b".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
            FilterNode {
                index: 1,
                result_name: "b".to_string(),
                inputs: vec![FilterInput::Reference("a".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
        ];
        let result = FilterGraph::topological_sort(&nodes);
        assert!(result.is_err());
    }
}

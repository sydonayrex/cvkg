//! Plain-data docking layout model.
//!
//! The model owns structure and persistence only. Bodies stay with the host
//! through [`PanelBodyHost`], so no view type leaks into serialized state.

use serde::{Deserialize, Serialize};

/// Split axis.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// A leaf panel reference. The body is supplied by the host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Panel {
    pub id: String,
}

/// A tabbed group of panel ids.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tabs {
    pub tabs: Vec<String>,
    pub active: usize,
}

/// A floating panel position.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Floating {
    pub panel_id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A split of two layout nodes.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Split {
    pub axis: Axis,
    pub ratio: f32,
    pub first: Box<Node>,
    pub second: Box<Node>,
}

/// One node in a docking tree.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Node {
    Panel(Panel),
    Tabs(Tabs),
    Floating(Floating),
    Split(Split),
}

impl Node {
    /// Count this node and every child. Tabs count their tab entries too.
    pub fn node_count(&self) -> usize {
        match self {
            Node::Panel(_) | Node::Floating(_) => 1,
            Node::Tabs(tabs) => 1 + tabs.tabs.len(),
            Node::Split(split) => 1 + split.first.node_count() + split.second.node_count(),
        }
    }
}

/// A serializable docking layout.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Layout {
    pub root: Node,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            root: Node::Panel(Panel {
                id: "main".to_string(),
            }),
        }
    }
}

impl Layout {
    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Supplies body-specific data without letting views into the model.
pub trait PanelBodyHost {
    fn panel_title(&self, panel_id: &str) -> Option<String>;
}

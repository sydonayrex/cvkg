use crate::types::{EdgeId, EdgePath, PortId};
use serde::{Deserialize, Serialize};

/// Connection between two ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: EdgeId,
    pub source: PortId,
    pub target: PortId,
    pub path: EdgePath,
    pub selected: bool,
}

impl FlowEdge {
    pub fn new(id: EdgeId, source: PortId, target: PortId) -> Self {
        Self {
            id,
            source,
            target,
            path: EdgePath::Bezier,
            selected: false,
        }
    }
}

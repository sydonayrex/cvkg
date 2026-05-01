use crate::types::{PortId, NodeId, PortPosition, PortDirection, EdgeId};
use serde::{Deserialize, Serialize};

/// Connection port on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowPort {
    pub id: PortId,
    pub node_id: NodeId,
    pub position: PortPosition,
    pub direction: PortDirection,
    pub connections: Vec<EdgeId>,
}

impl FlowPort {
    pub fn new(id: PortId, node_id: NodeId, position: PortPosition, direction: PortDirection) -> Self {
        Self {
            id,
            node_id,
            position,
            direction,
            connections: Vec::new(),
        }
    }
}

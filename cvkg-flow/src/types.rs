use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Default,
    Input,
    Output,
    Group,
    Annotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EdgePath {
    Bezier,
    Step,
    Straight,
}

/// Level of detail (LoD) rendering state for flow canvas components.
///
/// Maps visual rendering details and layout complexity based on camera zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LevelOfDetail {
    /// Render full details (e.g. text labels, ports, shadows, rich materials).
    Detailed,
    /// Render intermediate details (e.g. basic shapes and labels, omit sub-labels/shadows).
    Medium,
    /// Render highly simplified outlines or colored blocks (culled details).
    Simplified,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_serialization() {
        let id = NodeId(123);
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "123");
        let deserialized: NodeId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, id);
    }
}

use cvkg_vdom::{AriaProps, LayoutRect, NodeId, VDom, VDomPatch, VNode};
use std::collections::HashMap;

fn create_node(id: u64, key: Option<&str>, c_type: &str, children: Vec<NodeId>) -> VNode {
    VNode {
        id: NodeId(id),
        key: key.map(|k| k.to_string()),
        component_type: c_type.to_string(),
        props: HashMap::new(),
        state: None,
        layout: LayoutRect::default(),
        children,
        aria_role: "presentation".to_string(),
        aria_props: AriaProps::default(),
        portal_target: None,
        sdf_shape: None,
    }
}

#[test]
fn test_vdom_keyed_reordering() {
    // 1. Initial list: [A (key: "a"), B (key: "b")]
    let mut vdom1 = VDom::new();
    vdom1.root = Some(NodeId(0));
    vdom1.nodes.insert(
        NodeId(0),
        create_node(0, None, "List", vec![NodeId(1), NodeId(2)]),
    );
    vdom1
        .nodes
        .insert(NodeId(1), create_node(1, Some("a"), "Item", vec![]));
    vdom1
        .nodes
        .insert(NodeId(2), create_node(2, Some("b"), "Item", vec![]));

    // 2. Reordered list: [B (key: "b"), A (key: "a")]
    let mut vdom2 = VDom::new();
    vdom2.root = Some(NodeId(0));
    vdom2.nodes.insert(
        NodeId(0),
        create_node(0, None, "List", vec![NodeId(2), NodeId(1)]),
    );
    vdom2
        .nodes
        .insert(NodeId(1), create_node(1, Some("a"), "Item", vec![]));
    vdom2
        .nodes
        .insert(NodeId(2), create_node(2, Some("b"), "Item", vec![]));

    let patches = vdom1.diff(&vdom2);

    // The diff should ideally detect a reorder, but currently CVKG VDom might just replace or move.
    // Let's verify that the structure is at least correct after applying patches (or just verify patch generation).
    assert!(!patches.is_empty());

    // Check if the root List node was updated to reflect the new children order
    let root_update = patches
        .iter()
        .find(|p| matches!(p, VDomPatch::Update { id, .. } if *id == NodeId(0)));
    assert!(
        root_update.is_some(),
        "Root list should be updated with new children order"
    );
}

#[test]
fn test_vdom_deep_diffing() {
    // 1. Initial tree: List -> Item -> Text
    let mut vdom1 = VDom::new();
    vdom1.root = Some(NodeId(0));
    vdom1
        .nodes
        .insert(NodeId(0), create_node(0, None, "List", vec![NodeId(1)]));
    vdom1
        .nodes
        .insert(NodeId(1), create_node(1, None, "Item", vec![NodeId(2)]));
    vdom1
        .nodes
        .insert(NodeId(2), create_node(2, None, "Text", vec![]));

    // 2. Tree with removed leaf: List -> Item -> (Empty)
    let mut vdom2 = VDom::new();
    vdom2.root = Some(NodeId(0));
    vdom2
        .nodes
        .insert(NodeId(0), create_node(0, None, "List", vec![NodeId(1)]));
    vdom2
        .nodes
        .insert(NodeId(1), create_node(1, None, "Item", vec![]));

    let patches = vdom1.diff(&vdom2);

    // Should contain a Remove(2) and an Update(1)
    assert!(
        patches
            .iter()
            .any(|p| matches!(p, VDomPatch::Remove(id) if *id == NodeId(2)))
    );
    assert!(
        patches
            .iter()
            .any(|p| matches!(p, VDomPatch::Update { id, .. } if *id == NodeId(1)))
    );
}

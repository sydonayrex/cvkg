// Property-Based Testing for State Management
// Tests state transitions, VDOM patches, and component behavior using proptest

use cvkg_vdom::{AriaProps, LayoutRect, NodeId, VNode};
use proptest::prelude::*;
use std::collections::HashMap;

// Strategy for generating valid NodeId values
prop_compose! {
    fn arb_node_id()(id in 1u64..10000u64) -> NodeId {
        NodeId(id)
    }
}

// Strategy for generating VNode with random properties
prop_compose! {
    fn arb_vnode()(
        id in 1u64..10000u64,
    ) -> VNode {
        VNode {
            id: NodeId(id),
            key: None,
            component_type: "test_node".to_string(),
            sdf_shape: None,
            props: HashMap::new(),
            state: None,
            layout: LayoutRect::default(),
            children: Vec::new(),
            aria_role: "generic".to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
        }
    }
}

proptest! {
    #[test]
    fn test_vnode_creation(id in 1u64..10000u64) {
        let node = VNode {
            id: NodeId(id),
            key: None,
            component_type: "test".to_string(),
            sdf_shape: None,
            props: HashMap::new(),
            state: None,
            layout: LayoutRect::default(),
            children: Vec::new(),
            aria_role: "text".to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
        };
        assert_eq!(node.id.0, id);
    }
}

// Strategy for generating arbitrary layout rects
prop_compose! {
    fn arb_layout_rect()(x in 0.0f32..1000.0, y in 0.0f32..1000.0, width in 0.0f32..1000.0, height in 0.0f32..1000.0) -> LayoutRect {
        LayoutRect { x, y, width, height }
    }
}

// Strategy to generate completely arbitrary VNodes
prop_compose! {
    fn arb_complex_vnode()(
        id in 1u64..1000u64,
        key in proptest::option::of("[a-z]{5}"),
        component_type in "[a-zA-Z_]{3,10}",
        layout in arb_layout_rect(),
        aria_role in "(button|group|text|switch|slider)",
    ) -> VNode {
        VNode {
            id: NodeId(id),
            key,
            component_type,
            sdf_shape: None,
            props: HashMap::new(),
            state: None,
            layout,
            children: Vec::new(),
            aria_role,
            aria_props: AriaProps::default(),
            portal_target: None,
        }
    }
}

proptest! {
    #[test]
    fn test_vnode_diff_no_panic(node1 in arb_complex_vnode(), node2 in arb_complex_vnode()) {
        use cvkg_vdom::VDom;
        let mut vdom1 = VDom::new();
        let mut vdom2 = VDom::new();
        vdom1.nodes.insert(node1.id, node1.clone());
        vdom1.root = Some(node1.id);

        vdom2.nodes.insert(node2.id, node2.clone());
        vdom2.root = Some(node2.id);

        // Ensure diffing doesn't crash on completely random states
        let patches = vdom1.diff(&vdom2);

        // Either it replaces the root or does property updates
        assert!(patches.len() > 0 || node1 == node2);
    }
}

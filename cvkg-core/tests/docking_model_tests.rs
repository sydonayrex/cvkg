use cvkg_core::docking::{Axis, Floating, Layout, Node, Split, Tabs};

#[test]
fn layout_round_trips_through_json_with_identical_node_counts() {
    let layout = Layout {
        root: Node::Split(Split {
            axis: Axis::Horizontal,
            ratio: 0.6,
            first: Box::new(Node::Tabs(Tabs {
                tabs: vec!["editor".into(), "preview".into()],
                active: 1,
            })),
            second: Box::new(Node::Floating(Floating {
                panel_id: "logs".into(),
                x: 12.0,
                y: 24.0,
                width: 200.0,
                height: 120.0,
            })),
        }),
    };

    let json = layout.to_json().expect("serialize layout");
    let restored = Layout::from_json(&json).expect("deserialize layout");

    assert_eq!(restored.node_count(), layout.node_count());
    assert_eq!(restored, layout);
}

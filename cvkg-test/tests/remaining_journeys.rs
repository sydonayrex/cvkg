use cvkg_anim::{RunicEmitter, SpringParams, SpringSolver};
use cvkg_core::KvasirId;
use cvkg_core::{Alignment, Distribution, LayoutCache, LayoutView, Rect, Size, SizeProposal};
use cvkg_flow::port::FlowPort;
use cvkg_flow::types::{PortDirection, PortId, PortPosition};
use cvkg_flow::{FlowEdge, FlowGraph, FlowNode};
use cvkg_layout::HStack;
use cvkg_vdom::{AriaProps, LayoutRect, VDom, VDomPatch, VNode};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_journey_anim_runic_emitter() {
    // 1. Create a Sleipnir solver (RK4)
    let params = SpringParams::snappy();
    let mut solver = SpringSolver::new(params, 1.0, 0.0);

    // 2. Advance time and verify convergence
    let dt = 0.016;
    let mut last_x = 0.0;

    for _ in 0..10 {
        let x = solver.tick(dt);
        assert!(x >= last_x);
        last_x = x;
    }

    // 3. Check RunicEmitter
    let mut emitter = RunicEmitter::new(Rect::new(0.0, 0.0, 100.0, 100.0));
    emitter.update(Duration::from_millis(500)); // Should spawn some particles if spawn_rate=10

    // RunicEmitter spawn logic is based on spawn_timer and interval
    assert!(!emitter.particles.is_empty());
}

#[test]
fn test_journey_layout_flex_distribution() {
    struct TestFlexView {
        weight: f32,
    }
    impl LayoutView for TestFlexView {
        fn size_that_fits(
            &self,
            proposal: SizeProposal,
            _: &[&dyn LayoutView],
            _: &mut LayoutCache,
        ) -> Size {
            Size {
                width: proposal.width.unwrap_or(10.0),
                height: proposal.height.unwrap_or(10.0),
            }
        }
        fn place_subviews(&self, _: Rect, _: &mut [&mut dyn LayoutView], _: &mut LayoutCache) {}
        fn flex_weight(&self) -> f32 {
            self.weight
        }
    }

    struct Fixed50;
    impl LayoutView for Fixed50 {
        fn size_that_fits(
            &self,
            _: SizeProposal,
            _: &[&dyn LayoutView],
            _: &mut LayoutCache,
        ) -> Size {
            Size {
                width: 50.0,
                height: 50.0,
            }
        }
        fn place_subviews(&self, _: Rect, _: &mut [&mut dyn LayoutView], _: &mut LayoutCache) {}
        fn flex_weight(&self) -> f32 {
            0.0
        }
    }

    let v1_fixed = Fixed50;
    let v2 = TestFlexView { weight: 1.0 };
    let v3 = TestFlexView { weight: 1.0 };

    let views_fixed: Vec<&dyn LayoutView> = vec![&v1_fixed, &v2, &v3];
    let mut cache = LayoutCache::new();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 310.0,
        height: 100.0,
    };

    let rects = HStack::compute_layout(
        10.0,
        Alignment::Center,
        Distribution::Leading,
        bounds,
        &views_fixed,
        &mut cache,
    );

    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].width, 50.0);
    assert_eq!(rects[1].width, 120.0); // (310 - 50 - 20) / 2
    assert_eq!(rects[2].width, 120.0);
}

#[test]
fn test_journey_vdom_patch_lifecycle() {
    let mut vdom = VDom::new();

    let node = VNode {
        id: KvasirId(1),
        key: None,
        component_type: "div".to_string(),
        sdf_shape: None,
        props: HashMap::new(),
        state: None,
        layout: LayoutRect::default(),
        children: Vec::new(),
        aria_role: "group".to_string(),
        aria_props: AriaProps::default(),
        portal_target: None,
    };

    // 1. Initial State
    vdom.apply_patches(vec![VDomPatch::Create(node)]);
    assert_eq!(vdom.nodes.len(), 1);

    // 2. Update State
    vdom.apply_patches(vec![VDomPatch::Update {
        id: KvasirId(1),
        sdf_shape: None,
        props: None,
        layout: Some(LayoutRect {
            x: 10.0,
            y: 10.0,
            width: 200.0,
            height: 200.0,
        }),
        aria_props: None,
        aria_role: None,
        children: None,
        handlers: None,
    }]);

    // 3. Removal
    vdom.apply_patches(vec![VDomPatch::Remove(KvasirId(1))]);
    assert_eq!(vdom.nodes.len(), 0);
}

#[test]
fn test_journey_flow_graph_interaction() {
    let mut graph = FlowGraph::new();

    // 1. Build a simple graph
    let mut n1 = FlowNode::new(KvasirId(1), "Input", (0.0, 0.0));
    n1.add_port(FlowPort::new(
        PortId(1),
        KvasirId(1),
        PortPosition::Right,
        PortDirection::Output,
    ));

    let mut n2 = FlowNode::new(KvasirId(2), "Output", (200.0, 0.0));
    n2.add_port(FlowPort::new(
        PortId(2),
        KvasirId(2),
        PortPosition::Left,
        PortDirection::Input,
    ));

    graph.add_node(n1);
    graph.add_node(n2);
    graph.add_edge(FlowEdge::new(1, KvasirId(1), 0, KvasirId(2), 0));

    // 2. Verify graph topology
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    // 3. Verify lookup
    let node = graph.get_node_by_port(PortId(1)).unwrap();
    assert_eq!(node.id, KvasirId(1));
}

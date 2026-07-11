use cvkg_core::KvasirId;
use cvkg_vdom::{AriaProps, LayoutRect, VDom, VDomPatch, VNode};
use std::collections::HashMap;

/// Helper: create a minimal VNode with only the fields we care about.
fn make_node(id: KvasirId, layout: LayoutRect, children: Vec<KvasirId>) -> VNode {
    VNode {
        id,
        key: None,
        component_type: "div".to_string(),
        props: HashMap::new(),
        state: None,
        layout,
        children,
        aria_role: "div".to_string(),
        aria_props: AriaProps::default(),
        portal_target: None,
        world_space: None,
        theme_override: None,
        color_palette: u16::MAX,
        sdf_shape: None,
        companions: HashMap::new(),
    }
}

/// Build a 3-level tree: root → row → label.
/// Returns (vdom, root_id, row_id, label_id).
fn build_three_level_tree() -> (VDom, KvasirId, KvasirId, KvasirId) {
    let root_id = KvasirId::new();
    let row_id = KvasirId::new();
    let label_id = KvasirId::new();

    let root_node = make_node(root_id, LayoutRect { x: 0.0, y: 0.0, width: 400.0, height: 300.0 }, vec![row_id]);
    let row_node = make_node(row_id, LayoutRect { x: 10.0, y: 20.0, width: 200.0, height: 50.0 }, vec![label_id]);
    let label_node = make_node(label_id, LayoutRect { x: 5.0, y: 5.0, width: 80.0, height: 20.0 }, vec![]);

    let mut vdom = VDom::new();
    vdom.root = Some(root_id);
    vdom.parents.insert(row_id, root_id);
    vdom.parents.insert(label_id, row_id);
    vdom.nodes.insert(root_id, root_node);
    vdom.nodes.insert(row_id, row_node);
    vdom.nodes.insert(label_id, label_node);

    (vdom, root_id, row_id, label_id)
}

// ── Test 1: Moving root's layout changes child's world_rect but not stored layout ──

#[test]
fn move_root_changes_child_world_rect_but_not_stored_layout() {
    let (mut vdom, root_id, _row_id, label_id) = build_three_level_tree();

    // Before move: label world_rect = (10+5, 20+5) = (15, 25)
    let before = vdom.world_rect(label_id).unwrap();
    assert_eq!((before.x, before.y), (15.0, 25.0));

    // Capture label's stored layout before
    let label_before = vdom.nodes.get(&label_id).unwrap().layout;

    // Move root by (30, 40) via Update patch
    let patch = VDomPatch::Update {
        id: root_id,
        layout: Some(LayoutRect { x: 30.0, y: 40.0, width: 400.0, height: 300.0 }),
        props: None,
        aria_props: None,
        aria_role: None,
        children: None,
        handlers: None,
        sdf_shape: None,
        world_space: None,
        theme_override: None,
        color_palette: None,
    };
    vdom.apply_patches(vec![patch]);

    // After move: label world_rect should shift by (30, 40) → (45, 65)
    let after = vdom.world_rect(label_id).unwrap();
    assert_eq!((after.x, after.y), (45.0, 65.0));

    // Label's own stored layout must be byte-identical (local offset unchanged)
    let label_after = vdom.nodes.get(&label_id).unwrap().layout;
    assert_eq!(label_before, label_after);
}

// ── Test 2: Diff returns exactly one Update for root move, not per-descendant ──

#[test]
fn diff_returns_exactly_one_update_for_root_move() {
    let (mut old_vdom, root_id, row_id, label_id) = build_three_level_tree();

    // Move root by (30, 40) in the existing vdom (no new IDs)
    old_vdom.nodes.insert(root_id, make_node(root_id, LayoutRect { x: 30.0, y: 40.0, width: 400.0, height: 300.0 }, vec![row_id]));

    // Build a fresh tree with the original layout
    let new_vdom = {
        // Build a fresh tree with the original layout using the same IDs
        let mut v2 = VDom::new();
        v2.root = Some(root_id);
        v2.parents.insert(row_id, root_id);
        v2.parents.insert(label_id, row_id);
        v2.nodes.insert(root_id, make_node(root_id, LayoutRect { x: 0.0, y: 0.0, width: 400.0, height: 300.0 }, vec![row_id]));
        v2.nodes.insert(row_id, make_node(row_id, LayoutRect { x: 10.0, y: 20.0, width: 200.0, height: 50.0 }, vec![label_id]));
        v2.nodes.insert(label_id, make_node(label_id, LayoutRect { x: 5.0, y: 5.0, width: 80.0, height: 20.0 }, vec![]));
        v2
    };

    let patches = old_vdom.diff(&new_vdom);

    // Count Update patches — should be exactly 1 (root only)
    let updates: Vec<_> = patches.iter().filter(|p| matches!(p, VDomPatch::Update { .. })).collect();
    assert_eq!(updates.len(), 1, "expected exactly 1 Update patch for root move, got {}", updates.len());

    // Confirm the single update is for the root
    match updates[0] {
        VDomPatch::Update { id, .. } => assert_eq!(*id, root_id),
        _ => panic!("expected Update patch"),
    }
}

// ── Test 3: Spring animating parent — child tracks without patch ──

#[test]
fn spring_animating_parent_child_tracks_without_patch() {
    use cvkg_vdom::physics::Spring;

    let (mut vdom, root_id, _row_id, label_id) = build_three_level_tree();

    let initial_label_world = vdom.world_rect(label_id).unwrap();

    // Create a spring with stiffness=300.0, damping=20.0 (critically damped-ish)
    let spring = Spring::new(cvkg_core::Rect { x: 0.0, y: 0.0, width: 400.0, height: 300.0 }, 300.0, 20.0);
    spring.target.set(cvkg_core::Rect { x: 50.0, y: 60.0, width: 400.0, height: 300.0 });

    // Tick a few times to advance the spring (dt=0.016 = ~60fps)
    for _ in 0..10 {
        spring.tick(0.016);
        let current = spring.current.get();
        let node = vdom.nodes.get_mut(&root_id).unwrap();
        node.layout.x = current.x;
        node.layout.y = current.y;
    }

    // The label's world_rect should have moved (parent moved)
    let label_world = vdom.world_rect(label_id).unwrap();
    assert!(
        label_world.x > initial_label_world.x || label_world.y > initial_label_world.y,
        "label world_rect should have tracked parent movement"
    );

    // No patch was applied to the label — its stored layout is unchanged
    let label_stored = vdom.nodes.get(&label_id).unwrap().layout;
    assert_eq!((label_stored.x, label_stored.y), (5.0, 5.0), "label's stored local offset must be unchanged");
}

// ── Test 4: validate_node_sync detects drift and reports clean ──

#[test]
fn validate_node_sync_detects_drift_and_reports_clean() {
    let (vdom, root_id, row_id, label_id) = build_three_level_tree();
    let mut scene = cvkg_scene::SceneGraph::new();
    scene.root = Some(root_id);

    // Helper to create a scene VNode from a vdom node
    let make_scene_node = |id: KvasirId, layout: LayoutRect, children: Vec<KvasirId>| {
        cvkg_scene::VNode {
            id,
            component_type: "div".to_string(),
            children,
            local_rect: cvkg_core::Rect { x: layout.x, y: layout.y, width: layout.width, height: layout.height },
            world_rect: cvkg_core::Rect { x: layout.x, y: layout.y, width: layout.width, height: layout.height },
            is_dirty: false,
            layer_id: 0,
            z_index: 0.0,
            spatial_cells: Vec::new(),
            is_3d: false,
            position_3d: [0.0; 3],
            rotation_3d: [0.0, 0.0, 0.0, 1.0],
            scale_3d: [1.0; 3],
        }
    };

    let root_layout = vdom.nodes.get(&root_id).unwrap().layout;
    let row_layout = vdom.nodes.get(&row_id).unwrap().layout;
    let label_layout = vdom.nodes.get(&label_id).unwrap().layout;

    scene.nodes.insert(root_id, make_scene_node(root_id, root_layout, vec![row_id]));
    scene.nodes.insert(row_id, make_scene_node(row_id, row_layout, vec![label_id]));
    scene.nodes.insert(label_id, make_scene_node(label_id, label_layout, vec![]));

    // Should report no drift
    let result = vdom.validate_sync(&scene);
    assert!(result.is_ok(), "should report no drift: {:?}", result.err());

    // Now desync: change scene's local_rect (validate_node_sync compares local rects)
    if let Some(snode) = scene.nodes.get_mut(&root_id) {
        snode.local_rect.x += 100.0;
    }

    // Should report drift
    let result = vdom.validate_sync(&scene);
    assert!(result.is_err(), "should detect drift");
}

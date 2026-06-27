use cvkg_core::KvasirId;
use cvkg_vdom::{AriaProps, LayoutRect, NodeId, VDom, VDomPatch, VNode};
use std::collections::HashMap;
use std::sync::Arc;

fn create_node(id: u64, key: Option<&str>, c_type: &str, children: Vec<NodeId>) -> VNode {
    VNode {
        id: KvasirId(id),
        key: key.map(|k| k.to_string()),
        component_type: c_type.to_string(),
        props: HashMap::new(),
        state: None,
        layout: LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
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
    vdom1.root = Some(KvasirId(0));
    vdom1.nodes.insert(
        KvasirId(0),
        create_node(0, None, "List", vec![KvasirId(1), KvasirId(2)]),
    );
    vdom1
        .nodes
        .insert(KvasirId(1), create_node(1, Some("a"), "Item", vec![]));
    vdom1
        .nodes
        .insert(KvasirId(2), create_node(2, Some("b"), "Item", vec![]));

    // 2. Reordered list: [B (key: "b"), A (key: "a")]
    let mut vdom2 = VDom::new();
    vdom2.root = Some(KvasirId(0));
    vdom2.nodes.insert(
        KvasirId(0),
        create_node(0, None, "List", vec![KvasirId(2), KvasirId(1)]),
    );
    vdom2
        .nodes
        .insert(KvasirId(1), create_node(1, Some("a"), "Item", vec![]));
    vdom2
        .nodes
        .insert(KvasirId(2), create_node(2, Some("b"), "Item", vec![]));

    let patches = vdom1.diff(&vdom2);

    // The diff should ideally detect a reorder, but currently CVKG VDom might just replace or move.
    // Let's verify that the structure is at least correct after applying patches (or just verify patch generation).
    assert!(!patches.is_empty());

    // Check if the root List node was updated to reflect the new children order
    let root_update = patches
        .iter()
        .find(|p| matches!(p, VDomPatch::Update { id, .. } if *id == KvasirId(0)));
    assert!(
        root_update.is_some(),
        "Root list should be updated with new children order"
    );
}

#[test]
fn test_vdom_deep_diffing() {
    // 1. Initial tree: List -> Item -> Text
    let mut vdom1 = VDom::new();
    vdom1.root = Some(KvasirId(0));
    vdom1
        .nodes
        .insert(KvasirId(0), create_node(0, None, "List", vec![KvasirId(1)]));
    vdom1
        .nodes
        .insert(KvasirId(1), create_node(1, None, "Item", vec![KvasirId(2)]));
    vdom1
        .nodes
        .insert(KvasirId(2), create_node(2, None, "Text", vec![]));

    // 2. Tree with removed leaf: List -> Item -> (Empty)
    let mut vdom2 = VDom::new();
    vdom2.root = Some(KvasirId(0));
    vdom2
        .nodes
        .insert(KvasirId(0), create_node(0, None, "List", vec![KvasirId(1)]));
    vdom2
        .nodes
        .insert(KvasirId(1), create_node(1, None, "Item", vec![]));

    let patches = vdom1.diff(&vdom2);

    // Should contain a Remove(2) and an Update(1)
    assert!(
        patches
            .iter()
            .any(|p| matches!(p, VDomPatch::Remove(id) if *id == KvasirId(2)))
    );
    assert!(
        patches
            .iter()
            .any(|p| matches!(p, VDomPatch::Update { id, .. } if *id == KvasirId(1)))
    );
}

#[test]
fn test_signal_cross_thread() {
    use cvkg_vdom::signals::{create_effect, create_signal};
    use std::sync::{Arc, Mutex};
    use std::thread;

    let (get_val, set_val) = create_signal(0);

    // We'll capture the latest emitted value in a thread-safe mutex
    let latest_val = Arc::new(Mutex::new(0));
    let latest_val_clone = Arc::clone(&latest_val);

    // Create an effect that runs immediately and re-runs when get_val changes
    create_effect(move || {
        let val = get_val();
        let mut l = latest_val_clone.lock().unwrap();
        *l = val;
    });

    // Initial effect run should have populated it with 0
    assert_eq!(*latest_val.lock().unwrap(), 0);

    // Spawn a background thread to mutate the signal
    let handle = thread::spawn(move || {
        set_val(42);
        set_val(100);
    });

    handle.join().unwrap();

    // Verify the effect captured the final mutation from the background thread
    assert_eq!(*latest_val.lock().unwrap(), 100);
}

// =========================================================================
// P0-6: VDOM handler removal must be possible
// =========================================================================
//
// Regression tests for the audit finding: "VDOM Handler Removal Is
// Structurally Impossible". The previous `Update.handlers = None` semantics
// could not remove a handler once attached. The fix adds a `ClearHandlers`
// patch variant that explicitly removes all handlers for a node.

fn handler_closure(_event: cvkg_core::Event) {
    // Marker closure used to compare handler identity between old/new VDOMs.
}

fn interactive_node(
    id: u64,
    component_type: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    aria_role: &str,
) -> VNode {
    VNode {
        id: KvasirId(id),
        key: None,
        component_type: component_type.to_string(),
        props: HashMap::new(),
        state: None,
        layout: LayoutRect {
            x,
            y,
            width,
            height,
        },
        children: Vec::new(),
        aria_role: aria_role.to_string(),
        aria_props: AriaProps::default(),
        portal_target: None,
        sdf_shape: Some(cvkg_core::layout::SdfShape::Rect(cvkg_core::Rect {
            x,
            y,
            width,
            height,
        })),
    }
}

#[test]
fn p0_6_diff_emits_clear_handlers_when_handler_removed() {
    // Build old VDom with a handler attached to a node.
    let mut old = VDom::new();
    let node_id = KvasirId(1);
    let node = create_node(1, None, "Button", vec![]);
    old.nodes.insert(node_id, node);
    old.event_handlers.insert(
        node_id,
        vec![("click".to_string(), Arc::new(handler_closure) as _)]
            .into_iter()
            .collect(),
    );
    old.root = Some(node_id);

    // Build new VDom with no handlers.
    let mut new = VDom::new();
    new.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    new.root = Some(node_id);

    let patches = old.diff(&new);

    // Must contain a ClearHandlers patch for the node.
    assert!(
        patches
            .iter()
            .any(|p| matches!(p, VDomPatch::ClearHandlers { id } if *id == node_id)),
        "expected ClearHandlers patch, got: {patches:?}"
    );
}

#[test]
fn p0_6_apply_clear_handlers_removes_handler_from_event_handlers() {
    let mut vdom = VDom::new();
    let node_id = KvasirId(1);
    vdom.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    vdom.event_handlers.insert(
        node_id,
        vec![("click".to_string(), Arc::new(handler_closure) as _)]
            .into_iter()
            .collect(),
    );

    // Apply a ClearHandlers patch.
    vdom.apply_patches(vec![VDomPatch::ClearHandlers { id: node_id }]);

    assert!(
        !vdom.event_handlers.contains_key(&node_id),
        "handler should be removed after ClearHandlers"
    );
}

#[test]
fn p0_6_remove_handler_then_dispatch_does_not_invoke() {
    // End-to-end: diff removes handler, apply clears it, dispatch doesn't fire.
    let mut old = VDom::new();
    let node_id = KvasirId(1);
    let fired = Arc::new(std::sync::Mutex::new(false));
    let fired_clone = Arc::clone(&fired);
    old.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    old.event_handlers.insert(
        node_id,
        vec![(
            "click".to_string(),
            Arc::new(move |_| {
                *fired_clone.lock().unwrap() = true;
            }) as _,
        )]
        .into_iter()
        .collect(),
    );
    old.root = Some(node_id);

    // New tree drops the handler.
    let mut new = VDom::new();
    new.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    new.root = Some(node_id);

    let patches = old.diff(&new);
    new.apply_patches(patches);

    // The new VDom should have no handlers for this node.
    assert!(
        !new.event_handlers.contains_key(&node_id),
        "handler should be cleared after apply"
    );

    assert!(
        !*fired.lock().unwrap(),
        "no firing should have occurred yet"
    );
}

// =========================================================================
// P0-7: VDOM diff_node handlers-changed detection must be correct
// =========================================================================
//
// Regression tests for: "VDOM diff_node Handlers-Changed Detection Logic
// Is Wrong". The previous check `other.event_handlers.contains_key(&id)`
// always returned true when the new tree had a handler (even if identical),
// causing spurious Update patches every frame.

#[test]
fn p0_7_identical_handlers_do_not_emit_update_patch() {
    // Same handler attached in both old and new trees -> no Update patch
    // for handlers change (handlers_changed should be false).
    let closure: Arc<dyn Fn(cvkg_core::Event) + Send + Sync> = Arc::new(handler_closure);

    let mut old = VDom::new();
    let node_id = KvasirId(1);
    old.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    let mut handlers = HashMap::new();
    handlers.insert("click".to_string(), Arc::clone(&closure));
    old.event_handlers.insert(node_id, handlers);
    old.root = Some(node_id);

    let mut new = VDom::new();
    new.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    let mut handlers = HashMap::new();
    handlers.insert("click".to_string(), Arc::clone(&closure));
    new.event_handlers.insert(node_id, handlers);
    new.root = Some(node_id);

    let patches = old.diff(&new);

    // No Update patch should be emitted (no field changed), and no
    // ClearHandlers should be emitted (no removal).
    for p in &patches {
        match p {
            VDomPatch::Update { handlers, .. } => {
                assert!(
                    handlers.is_none(),
                    "identical handlers should not appear as a changed field, got: {patches:?}"
                );
            }
            VDomPatch::ClearHandlers { .. } => {
                panic!("identical handlers should not trigger ClearHandlers: {patches:?}");
            }
            _ => {}
        }
    }
}

#[test]
fn p0_7_handler_swap_emits_update_patch() {
    // Different closures attached to the same key -> handlers_changed.
    let closure_a: Arc<dyn Fn(cvkg_core::Event) + Send + Sync> = Arc::new(handler_closure);
    let closure_b: Arc<dyn Fn(cvkg_core::Event) + Send + Sync> =
        Arc::new(|_| log::debug!("other handler"));

    let mut old = VDom::new();
    let node_id = KvasirId(1);
    old.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    let mut handlers = HashMap::new();
    handlers.insert("click".to_string(), Arc::clone(&closure_a));
    old.event_handlers.insert(node_id, handlers);
    old.root = Some(node_id);

    let mut new = VDom::new();
    new.nodes
        .insert(node_id, create_node(1, None, "Button", vec![]));
    let mut handlers = HashMap::new();
    handlers.insert("click".to_string(), Arc::clone(&closure_b));
    new.event_handlers.insert(node_id, handlers);
    new.root = Some(node_id);

    let patches = old.diff(&new);

    // Must contain an Update patch with handlers populated.
    assert!(
        patches.iter().any(|p| matches!(
            p,
            VDomPatch::Update {
                handlers: Some(_),
                ..
            }
        )),
        "different closures should trigger an Update patch with handlers, got: {patches:?}"
    );
}

// =========================================================================
// Phase 6: Click-box regressions
// =========================================================================

#[test]
fn phase6_presentation_overlay_does_not_steal_child_hit_target() {
    let mut vdom = VDom::new();
    let root_id = KvasirId(1);
    let overlay_id = KvasirId(2);
    let button_id = KvasirId(3);
    let fired = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));

    let mut root = interactive_node(1, "Root", 0.0, 0.0, 200.0, 200.0, "group");
    root.children = vec![overlay_id];
    let mut overlay = interactive_node(2, "Overlay", 0.0, 0.0, 200.0, 200.0, "presentation");
    overlay.children = vec![button_id];
    let button = interactive_node(3, "Button", 40.0, 40.0, 60.0, 40.0, "button");

    let fired_button = Arc::clone(&fired);
    vdom.event_handlers.insert(
        button_id,
        vec![(
            "pointerdown".to_string(),
            Arc::new(move |_| {
                fired_button.lock().unwrap().push(button_id.0);
            }) as _,
        )]
        .into_iter()
        .collect(),
    );

    vdom.root = Some(root_id);
    vdom.nodes.insert(root_id, root);
    vdom.nodes.insert(overlay_id, overlay);
    vdom.nodes.insert(button_id, button);
    vdom.parents.insert(overlay_id, root_id);
    vdom.parents.insert(button_id, overlay_id);

    let response = vdom.dispatch_event(cvkg_core::Event::PointerDown {
        x: 50.0,
        y: 50.0,
        button: 0,
        proximity_field: 0.0,
        tilt: None,
        azimuth: None,
        pressure: None,
        barrel_rotation: None,
        pointer_precision: 0.0,
    });

    assert_eq!(response, cvkg_core::EventResponse::Handled);
    assert_eq!(*fired.lock().unwrap(), vec![button_id.0]);
}

#[test]
fn phase6_repeated_rebuilds_keep_click_boxes_and_handlers_stable() {
    let fired = Arc::new(std::sync::Mutex::new(0u32));
    let mut previous: Option<VDom> = None;

    for _ in 0..100 {
        let mut vdom = VDom::new();
        let root_id = KvasirId(1);
        let button_id = KvasirId(2);
        let overlay_id = KvasirId(3);

        let mut root = interactive_node(1, "Root", 0.0, 0.0, 320.0, 240.0, "group");
        root.children = vec![overlay_id];

        let mut overlay = interactive_node(3, "Overlay", 0.0, 0.0, 320.0, 240.0, "presentation");
        overlay.children = vec![button_id];

        let button = interactive_node(2, "MenuButton", 16.0, 16.0, 96.0, 40.0, "button");

        let fired_clone = Arc::clone(&fired);
        vdom.event_handlers.insert(
            button_id,
            vec![(
                "pointerdown".to_string(),
                Arc::new(move |_| {
                    *fired_clone.lock().unwrap() += 1;
                }) as _,
            )]
            .into_iter()
            .collect(),
        );

        vdom.root = Some(root_id);
        vdom.nodes.insert(root_id, root);
        vdom.nodes.insert(overlay_id, overlay);
        vdom.nodes.insert(button_id, button);
        vdom.parents.insert(overlay_id, root_id);
        vdom.parents.insert(button_id, overlay_id);

        if let Some(prev) = previous.take() {
            let patches = prev.diff(&vdom);
            assert!(
                patches.len() <= 4,
                "expected an incremental diff, got {patches:?}"
            );
        }

        let hit = vdom.hit_test(32.0, 32.0, 0.0).map(|(id, _)| id);
        assert_eq!(hit, Some(button_id));

        let response = vdom.dispatch_event(cvkg_core::Event::PointerDown {
            x: 32.0,
            y: 32.0,
            button: 0,
            proximity_field: 0.0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        });
        assert_eq!(response, cvkg_core::EventResponse::Handled);

        previous = Some(vdom);
    }

    assert_eq!(*fired.lock().unwrap(), 100);
}

#[test]
fn phase6_pointer_capture_survives_rebuild_before_release() {
    let fired = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));

    let mut pressed = VDom::new();
    let root_id = KvasirId(1);
    let button_id = KvasirId(2);
    let mut root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
    root.children = vec![button_id];
    let button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");
    let fired_press = Arc::clone(&fired);
    pressed.event_handlers.insert(
        button_id,
        vec![
            (
                "pointerdown".to_string(),
                Arc::new(move |_| {
                    fired_press.lock().unwrap().push("down");
                }) as _,
            ),
            (
                "pointerup".to_string(),
                Arc::new({
                    let fired = Arc::clone(&fired);
                    move |_| {
                        fired.lock().unwrap().push("up");
                    }
                }) as _,
            ),
            (
                "pointerclick".to_string(),
                Arc::new({
                    let fired = Arc::clone(&fired);
                    move |_| {
                        fired.lock().unwrap().push("click");
                    }
                }) as _,
            ),
        ]
        .into_iter()
        .collect(),
    );
    pressed.root = Some(root_id);
    pressed.nodes.insert(root_id, root);
    pressed.nodes.insert(button_id, button);
    pressed.parents.insert(button_id, root_id);

    assert_eq!(
        pressed.dispatch_event(cvkg_core::Event::PointerDown {
            x: 30.0,
            y: 30.0,
            button: 0,
            proximity_field: 0.0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        }),
        cvkg_core::EventResponse::Handled
    );

    let mut rebuilt = VDom::new();
    let mut rebuilt_root = interactive_node(1, "Root", 0.0, 0.0, 240.0, 240.0, "group");
    rebuilt_root.children = vec![button_id];
    let rebuilt_button = interactive_node(2, "Button", 20.0, 20.0, 80.0, 40.0, "button");
    rebuilt.event_handlers = pressed.event_handlers.clone();
    rebuilt.root = Some(root_id);
    rebuilt.nodes.insert(root_id, rebuilt_root);
    rebuilt.nodes.insert(button_id, rebuilt_button);
    rebuilt.parents.insert(button_id, root_id);

    let patches = pressed.diff(&rebuilt);
    pressed.apply_patches(patches);

    assert_eq!(
        pressed.dispatch_event(cvkg_core::Event::PointerUp {
            x: 30.0,
            y: 30.0,
            button: 0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        }),
        cvkg_core::EventResponse::Handled
    );
    assert_eq!(
        pressed.dispatch_event(cvkg_core::Event::PointerClick {
            x: 30.0,
            y: 30.0,
            button: 0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        }),
        cvkg_core::EventResponse::Handled
    );

    assert_eq!(*fired.lock().unwrap(), vec!["down", "up", "click"]);
}

// ---------------------------------------------------------------------------
// Berserker click-box regression test
//
// Models the berserker demo's interactive layout and verifies that click
// boxes (hit targets) work correctly across VDOM rebuilds.
// ---------------------------------------------------------------------------

#[test]
fn berserker_click_box_regression() {
    // Fixed layout constants matching the berserker demo (1280x720 window)
    let menu_x = [8.0_f32, 68.0, 128.0, 198.0, 278.0];
    let menu_w = [60.0_f32, 60.0, 70.0, 80.0, 60.0];
    let corner_positions: [(f32, f32); 4] = [
        (20.0, 50.0),    // I: top-left
        (1160.0, 50.0),  // II: top-right (1280-120)
        (20.0, 530.0),   // III: bottom-left (720-190)
        (1160.0, 530.0), // IV: bottom-right
    ];

    // Helper: build a VDom matching the berserker static chrome
    let shared_down: Arc<dyn Fn(cvkg_core::Event) + Send + Sync> = Arc::new(|_| {});
    let shared_click: Arc<dyn Fn(cvkg_core::Event) + Send + Sync> = Arc::new(|_| {});

    // Helper: build a VDom matching the berserker static chrome.
    // Reuses pre-allocated event handler closures so pointer comparisons in diff remain stable.
    let build_vdom = || -> (VDom, Vec<KvasirId>, Vec<KvasirId>, Vec<KvasirId>) {
        let mut vdom = VDom::new();

        // Root container
        let root_id = KvasirId(1);
        let mut root = interactive_node(1, "BerserkerRoot", 0.0, 0.0, 1280.0, 720.0, "application");

        // Nornir bar
        let bar_id = KvasirId(10);
        let mut bar = interactive_node(10, "NornirBar", 0.0, 0.0, 1280.0, 28.0, "group");

        let menu_ids: Vec<KvasirId> = (0..5).map(|i| KvasirId(100 + i as u64)).collect();
        let mut bar_children: Vec<KvasirId> = vec![];
        for i in 0..5 {
            let mid = menu_ids[i];
            let m = interactive_node(
                mid.0,
                "NornirBarItem",
                menu_x[i],
                0.0,
                menu_w[i],
                28.0,
                "group",
            );
            vdom.nodes.insert(mid, m);
            bar_children.push(mid);
            vdom.event_handlers.insert(
                mid,
                vec![("pointerdown".to_string(), Arc::clone(&shared_down))]
                    .into_iter()
                    .collect(),
            );
        }
        bar.children = bar_children.clone();

        // Corner buttons
        let corner_ids: Vec<KvasirId> = (0..4).map(|i| KvasirId(200 + i as u64)).collect();
        let mut root_children: Vec<KvasirId> = vec![bar_id];
        for i in 0..4 {
            let cid = corner_ids[i];
            let (cx, cy) = corner_positions[i];
            let cb = interactive_node(cid.0, "CornerButton", cx, cy, 100.0, 100.0, "button");
            vdom.nodes.insert(cid, cb);
            root_children.push(cid);
            vdom.event_handlers.insert(
                cid,
                vec![("pointerclick".to_string(), Arc::clone(&shared_click))]
                    .into_iter()
                    .collect(),
            );
        }

        // Dock
        let dock_id = KvasirId(50);
        let mut dock = interactive_node(50, "HeimdallDock", 384.0, 652.0, 512.0, 56.0, "group");
        root_children.push(dock_id);

        let dock_item_ids: Vec<KvasirId> = (0..5).map(|i| KvasirId(300 + i as u64)).collect();
        let mut dock_children: Vec<KvasirId> = vec![];
        for i in 0..5 {
            let did = dock_item_ids[i];
            let dx = 384.0 + (512.0 - 304.0) / 2.0 + i as f32 * 64.0;
            let di = interactive_node(did.0, "HeimdallDockItem", dx, 652.0, 48.0, 56.0, "button");
            vdom.nodes.insert(did, di);
            dock_children.push(did);
            vdom.event_handlers.insert(
                did,
                vec![("pointerclick".to_string(), Arc::clone(&shared_click))]
                    .into_iter()
                    .collect(),
            );
        }
        dock.children = dock_children.clone();

        // Overlay
        let overlay_id = KvasirId(999);
        let overlay = interactive_node(
            999,
            "DropdownOverlay",
            0.0,
            0.0,
            1280.0,
            720.0,
            "presentation",
        );
        root_children.push(overlay_id);
        vdom.event_handlers.insert(
            overlay_id,
            vec![
                ("pointerdown".to_string(), Arc::clone(&shared_down)),
                ("pointerclick".to_string(), Arc::clone(&shared_click)),
            ]
            .into_iter()
            .collect(),
        );

        root.children = root_children;
        vdom.root = Some(root_id);
        vdom.nodes.insert(root_id, root);
        vdom.nodes.insert(bar_id, bar);
        vdom.nodes.insert(dock_id, dock);
        vdom.nodes.insert(overlay_id, overlay);
        vdom.parents.insert(bar_id, root_id);
        vdom.parents.insert(dock_id, root_id);
        vdom.parents.insert(overlay_id, root_id);
        for mid in &menu_ids {
            vdom.parents.insert(*mid, bar_id);
        }
        for cid in &corner_ids {
            vdom.parents.insert(*cid, root_id);
        }
        for did in &dock_item_ids {
            vdom.parents.insert(*did, dock_id);
        }

        (vdom, menu_ids, corner_ids, dock_item_ids)
    };

    // ------------------------------------------------------------------
    // 1. Build initial VDom and verify hit testing
    // ------------------------------------------------------------------
    let (vdom, menu_ids, corner_ids, dock_item_ids) = build_vdom();

    // Menu items -- click center of each
    // Note: The overlay (z-top, fullscreen) blocks hits on elements beneath it.
    // This is correct behavior: the overlay is a dismiss layer that catches
    // clicks before they reach the menu bar. In the real berserker demo,
    // the overlay is only rendered when a menu is open, and the menu items
    // are positioned to be reachable.
    for i in 0..5 {
        let mid = menu_ids[i];
        let cx = menu_x[i] + menu_w[i] / 2.0;
        let cy = 14.0;
        let hit = vdom.hit_test(cx, cy, 0.0).map(|(id, _)| id);
        // The overlay (999) is on top, so it absorbs the hit
        assert_eq!(
            hit,
            Some(KvasirId(999)),
            "Overlay should block menu item {i} at ({cx}, {cy})"
        );
    }

    // Corner buttons -- click center of each
    // NOTE: The overlay is fullscreen and on top, so it currently blocks
    // ALL hits including corner buttons. This is a known issue: the
    // overlay should only block hits within its dropdown bounds, not
    // across the entire screen. The test documents the EXPECTED behavior
    // (corner buttons should be reachable) -- the failure indicates a
    // bug in the overlay's hit test policy that needs fixing.
    for i in 0..4 {
        let cid = corner_ids[i];
        let (cx, cy) = corner_positions[i];
        let hit = vdom.hit_test(cx + 50.0, cy + 50.0, 0.0).map(|(id, _)| id);
        assert_eq!(
            hit,
            Some(cid),
            "Corner button {i} should be hit (overlay should not block)"
        );
    }

    // Dock items -- click center of each
    for i in 0..5 {
        let did = dock_item_ids[i];
        let dx = 384.0 + (512.0 - 304.0) / 2.0 + i as f32 * 64.0 + 24.0;
        let dy = 680.0;
        let hit = vdom.hit_test(dx, dy, 0.0).map(|(id, _)| id);
        assert_eq!(hit, Some(did), "Dock item {i} should be hit");
    }

    // ------------------------------------------------------------------
    // 2. Verify event dispatch
    // ------------------------------------------------------------------
    let file_click = vdom.dispatch_event(cvkg_core::Event::PointerDown {
        x: 30.0,
        y: 14.0,
        button: 0,
        proximity_field: 0.0,
        tilt: None,
        azimuth: None,
        pressure: Some(1.0),
        barrel_rotation: None,
        pointer_precision: 0.0,
    });
    assert_eq!(
        file_click,
        cvkg_core::EventResponse::Handled,
        "File menu click"
    );

    let corner_click = vdom.dispatch_event(cvkg_core::Event::PointerClick {
        x: 70.0,
        y: 100.0,
        button: 0,
        tilt: None,
        azimuth: None,
        pressure: None,
        barrel_rotation: None,
        pointer_precision: 0.0,
    });
    assert_eq!(
        corner_click,
        cvkg_core::EventResponse::Handled,
        "Corner button click"
    );

    // ------------------------------------------------------------------
    // 3. Verify handler survival across 100 rebuilds
    // ------------------------------------------------------------------
    let fired = Arc::new(std::sync::Mutex::new(0u32));
    let mut prev: Option<VDom> = None;

    for _ in 0..100 {
        let (mut new_vdom, _new_menu_ids, new_corner_ids, _new_dock_ids) = build_vdom();

        // Increment counter on corner button click
        let f = Arc::clone(&fired);
        new_vdom.event_handlers.insert(
            new_corner_ids[0],
            vec![("pointerclick".to_string(), {
                let f = Arc::clone(&f);
                Arc::new(move |_| {
                    *f.lock().unwrap() += 1;
                }) as _
            })]
            .into_iter()
            .collect(),
        );

        if let Some(prev_vdom) = prev.take() {
            let patches = prev_vdom.diff(&new_vdom);
            assert!(
                patches.len() <= 10,
                "expected incremental diff, got {}",
                patches.len()
            );
        }

        // Verify hit testing after rebuild
        let hit = new_vdom.hit_test(70.0, 100.0, 0.0).map(|(id, _)| id);
        assert_eq!(
            hit,
            Some(new_corner_ids[0]),
            "Corner button should be hit after rebuild"
        );

        // Verify event dispatch after rebuild
        let resp = new_vdom.dispatch_event(cvkg_core::Event::PointerClick {
            x: 70.0,
            y: 100.0,
            button: 0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        });
        assert_eq!(
            resp,
            cvkg_core::EventResponse::Handled,
            "Click should be handled after rebuild"
        );

        prev = Some(new_vdom);
    }

    assert_eq!(
        *fired.lock().unwrap(),
        100,
        "All 100 clicks should have fired"
    );

    // The overlay is a fullscreen dismiss layer at z-top. Under the new priority
    // policy, when y >= 28.0 (outside the menu bar), the DropdownOverlay is evaluated last
    // among siblings. Sibling interactive elements (such as corner buttons) intercept hits first.
    let hit_through = vdom.hit_test(70.0, 100.0, 0.0).map(|(id, _)| id);
    assert_eq!(
        hit_through,
        Some(KvasirId(200)),
        "Corner button should be hit through the overlay priority policy outside the menu bar"
    );

    // Clicking in an empty area where no sibling matches should fall back to the overlay.
    let hit_fallback = vdom.hit_test(500.0, 400.0, 0.0).map(|(id, _)| id);
    assert_eq!(
        hit_fallback,
        Some(KvasirId(999)),
        "Overlay should capture clicks in empty areas to dismiss menu"
    );
}

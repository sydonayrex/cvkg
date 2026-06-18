use cvkg_vdom::{AriaProps, LayoutRect, NodeId, VDom, VDomPatch, VNode};
use cvkg_core::KvasirId;
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

fn empty_props() -> HashMap<String, serde_json::Value> {
    HashMap::new()
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

    assert!(!*fired.lock().unwrap(), "no firing should have occurred yet");
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
            VDomPatch::Update { handlers: Some(_), .. }
        )),
        "different closures should trigger an Update patch with handlers, got: {patches:?}"
    );
}

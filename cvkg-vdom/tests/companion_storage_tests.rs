use cvkg_core::{Companion, FocusableCompanion, A11yCompanion, FocusState, Rect, Renderer};
use cvkg_vdom::VNodeRenderer;

#[test]
fn test_vnoderenderer_stores_multiple_companions() {
    let mut renderer = VNodeRenderer::new();
    let rect = Rect::new(0.0, 0.0, 200.0, 50.0);
    let companions: Vec<Box<dyn Companion>> = vec![
        Box::new(FocusableCompanion { state: FocusState::Unfocused, tab_index: 0 }),
        Box::new(A11yCompanion { role: "button".into(), label: "Submit".into(), description: "".into(), disabled: false }),
    ];

    renderer.push_vnode_with_companions(rect, "MyButton", companions);

    // Companion must be retrievable from the current VNode
    let companion = renderer.current_companion();
    assert!(companion.is_some());
    // HashMap doesn't preserve order, so just verify we can retrieve a companion
    let type_name = companion.unwrap().type_name();
    assert!(type_name == "FocusableCompanion" || type_name == "A11yCompanion");
}

#[test]
fn test_vnoderenderer_companion_missing_returns_none() {
    let mut renderer = VNodeRenderer::new();
    let rect = Rect::new(0.0, 0.0, 200.0, 50.0);

    // Push a VNode with no companions.
    renderer.push_vnode(rect, "Empty");

    let companion = renderer.current_companion();
    assert!(companion.is_none());
}
use cvkg_core::{ActionId, ActionRegistry, FocusHandle, KeyBinding, KeyContextStack, KeyShortcut};

#[test]
fn binding_resolves_in_the_active_key_context() {
    let focus = ActionId::new("PaletteFocus");
    let binding = KeyBinding::new(
        KeyShortcut::parse("Ctrl+K").expect("valid shortcut"),
        focus.clone(),
        Some("Palette"),
    );
    let registry = ActionRegistry::new(vec![binding]);
    let shortcut = KeyShortcut::parse("Ctrl+K").expect("valid shortcut");

    let mut contexts = KeyContextStack::new();
    contexts.push("Palette");
    assert_eq!(registry.resolve(&contexts, &shortcut), Some(focus));

    contexts.pop();
    assert_eq!(registry.resolve(&contexts, &shortcut), None);
}

#[test]
fn cloned_focus_handles_share_an_identity() {
    let handle = FocusHandle::new();
    let clone = handle.clone();
    assert_eq!(handle.id(), clone.id());
    assert_ne!(handle.id(), FocusHandle::new().id());
}

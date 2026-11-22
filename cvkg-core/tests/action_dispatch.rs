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
    let modifiers = cvkg_core::KeyModifiers {
        ctrl: true,
        ..cvkg_core::KeyModifiers::default()
    };

    let mut contexts = KeyContextStack::new();
    contexts.push("Palette");
    assert_eq!(registry.resolve(&contexts, "k", modifiers), Some(focus));

    contexts.pop();
    assert_eq!(registry.resolve(&contexts, "k", modifiers), None);
}

#[test]
fn cloned_focus_handles_share_an_identity() {
    let handle = FocusHandle::new();
    let clone = handle.clone();
    assert_eq!(handle.id(), clone.id());
    assert_ne!(handle.id(), FocusHandle::new().id());
}

use cvkg_core::{
    ActionId, ActionRegistry, KeyBinding, KeyContextStack, KeyModifiers, KeyShortcut,
    dispatch_key_event,
};
use std::cell::RefCell;

#[test]
fn dispatcher_resolves_an_action_and_calls_the_focused_handler() {
    let focus = ActionId::new("PaletteFocus");
    let registry = ActionRegistry::new(vec![KeyBinding::new(
        KeyShortcut::parse("Ctrl+K").expect("valid shortcut"),
        focus,
        Some("Palette"),
    )]);
    let mut contexts = KeyContextStack::new();
    contexts.push("Palette");
    let seen = RefCell::new(Vec::new());

    let result = dispatch_key_event(
        &registry,
        &contexts,
        "k",
        KeyModifiers {
            ctrl: true,
            ..KeyModifiers::default()
        },
        |key, modifiers| {
            seen.borrow_mut().push((key.to_string(), modifiers.ctrl));
            true
        },
    );

    assert_eq!(result.action, Some(ActionId::new("PaletteFocus")));
    assert!(result.consumed);
    assert_eq!(seen.borrow().as_slice(), &[("k".to_string(), true)]);
}

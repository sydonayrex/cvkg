/// Compile-time test: the expanded macro must produce a companion_states() method.
/// This test verifies the macro expansion, not runtime behavior.

use cvkg_macros::{view_component, require};
use cvkg_core::{View, Companion, FocusableCompanion, A11yCompanion, EmptyView};

#[view_component]
#[require(FocusableCompanion, A11yCompanion)]
fn MyButton(label: String) {
    EmptyView
}

#[view_component]
fn PlainButton(label: String) {
    EmptyView
}

#[test]
fn test_view_component_with_require_produces_companion_states() {
    let btn = MyButton("Click".into());
    let companions = btn.companion_states();
    assert_eq!(companions.len(), 2);
    assert_eq!(companions[0].type_name(), "FocusableCompanion");
    assert_eq!(companions[1].type_name(), "A11yCompanion");
}

#[test]
fn test_view_component_without_require_returns_empty() {
    let btn = PlainButton("Go".into());
    let companions = btn.companion_states();
    assert!(companions.is_empty());
}
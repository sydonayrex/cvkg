//! P1-5 input parity: public-surface tests for the shared shortcut foundation.
//!
//! These exercise the API the way a native or web front-end crate would: via
//! `cvkg_core`'s public exports only. Note that `cvkg-core/Cargo.toml` sets
//! `autotests = false`, so this target must be declared with a `[[test]]`
//! entry before `cargo test --test input_parity` can discover it.

use cvkg_core::{
    KeyModifiers, KeyShortcut, ShortcutParseError, ShortcutPlatform, normalize_key_name,
};

#[test]
fn native_physical_code_and_web_logical_key_share_one_shortcut() {
    let shortcut = KeyShortcut::parse("Ctrl+Shift+S").expect("valid shortcut");

    // winit reports physical codes plus a ModifiersState.
    let native_mods = KeyModifiers::from_pressed_keys(["ControlLeft", "ShiftLeft"]);
    assert!(shortcut.matches("KeyS", native_mods));

    // The web reports logical keys plus modifier booleans.
    let web_mods = KeyModifiers {
        ctrl: true,
        shift: true,
        ..KeyModifiers::default()
    };
    assert!(shortcut.matches("s", web_mods));
}

#[test]
fn primary_modifier_normalises_command_and_control() {
    let shortcut = KeyShortcut::parse("Ctrl+P").expect("valid shortcut");
    let meta_p = KeyModifiers {
        meta: true,
        ..KeyModifiers::default()
    };
    let ctrl_p = KeyModifiers {
        ctrl: true,
        ..KeyModifiers::default()
    };

    assert!(!shortcut.matches("p", meta_p));
    assert!(shortcut.matches_on("p", meta_p, ShortcutPlatform::Apple));
    assert!(shortcut.matches_on("p", ctrl_p, ShortcutPlatform::Other));
    assert_eq!(KeyModifiers::primary(ShortcutPlatform::Apple), meta_p);
    assert_eq!(KeyModifiers::primary(ShortcutPlatform::Other), ctrl_p);
}

#[test]
fn modifier_aliases_and_key_names_are_canonicalised() {
    let cmd = KeyShortcut::parse("Cmd+Option+Escape").expect("valid shortcut");
    assert!(cmd.modifiers.meta);
    assert!(cmd.modifiers.alt);
    assert_eq!(cmd.key, "Escape");
    assert_eq!(normalize_key_name("Esc").as_deref(), Some("Escape"));
    assert_eq!(normalize_key_name("OSRight"), None);
}

#[test]
fn malformed_input_is_reported_without_panicking() {
    assert_eq!(KeyShortcut::parse(""), Err(ShortcutParseError::Empty));
    assert!(matches!(
        KeyShortcut::parse("Alt+Shift"),
        Err(ShortcutParseError::MissingKey(_))
    ));
    assert!(matches!(
        KeyShortcut::parse("Ctrl+Control+K"),
        Err(ShortcutParseError::ConflictingModifiers(_, _))
    ));
    assert!(matches!(
        KeyShortcut::parse("A+B"),
        Err(ShortcutParseError::MultipleKeys(_))
    ));
    assert!(matches!(
        KeyShortcut::parse("Ctrl++K"),
        Err(ShortcutParseError::MalformedSeparator(_))
    ));
}

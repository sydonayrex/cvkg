//! Tests for sensitivity, invert, and chord behavior.

use cvkg_inputs::{ActionMap, ActionConfig, Binding};
use cvkg_inputs::backend::InputEvent;
use cvkg_inputs::DeviceId;

#[test]
fn test_sensitivity_scales_axis() {
    let mut map = ActionMap::new();
    map.bind("move_x", Binding::Axis(0));
    map.set_sensitivity("move_x", 0.5);

    let result = map.evaluate_axis("move_x", 0.8);
    assert!(
        (result.unwrap() - 0.4).abs() < f32::EPSILON,
        "expected 0.4, got {:?}",
        result
    );
}

#[test]
fn test_sensitivity_clamps_to_one() {
    let mut map = ActionMap::new();
    map.bind("move_x", Binding::Axis(0));
    map.set_sensitivity("move_x", 2.0);

    let result = map.evaluate_axis("move_x", 0.8);
    assert!(
        (result.unwrap() - 1.0).abs() < f32::EPSILON,
        "expected clamp to 1.0, got {:?}",
        result
    );
}

#[test]
fn test_invert_flips_sign() {
    let mut map = ActionMap::new();
    map.bind("move_x", Binding::Axis(0));
    map.set_invert("move_x", true);

    let result = map.evaluate_axis("move_x", 0.5);
    assert!(
        (result.unwrap() - (-0.5)).abs() < f32::EPSILON,
        "expected -0.5, got {:?}",
        result
    );
}

#[test]
fn test_invert_and_sensitivity_combined() {
    let mut map = ActionMap::new();
    map.bind("move_x", Binding::Axis(0));
    map.set_sensitivity("move_x", 0.5);
    map.set_invert("move_x", true);

    let result = map.evaluate_axis("move_x", 0.8);
    assert!(
        (result.unwrap() - (-0.4)).abs() < f32::EPSILON,
        "expected -0.4, got {:?}",
        result
    );
}

#[test]
fn test_evaluate_axis_returns_none_for_button_binding() {
    let mut map = ActionMap::new();
    map.bind("jump", Binding::Button(0));

    let result = map.evaluate_axis("jump", 0.5);
    assert!(result.is_none());
}

#[test]
fn test_evaluate_axis_returns_none_for_unknown_action() {
    let map = ActionMap::new();
    let result = map.evaluate_axis("nonexistent", 0.5);
    assert!(result.is_none());
}

#[test]
fn test_bind_with_config() {
    let mut map = ActionMap::new();
    let config = ActionConfig {
        binding: Binding::Axis(0),
        sensitivity: 0.75,
        invert: true,
    };
    map.bind_with_config("steer", config);

    let result = map.evaluate_axis("steer", 1.0);
    assert!(
        (result.unwrap() - (-0.75)).abs() < f32::EPSILON,
        "expected -0.75, got {:?}",
        result
    );
}

#[test]
fn test_action_config_default() {
    let config = ActionConfig::default();
    assert_eq!(config.sensitivity, 1.0);
    assert!(!config.invert);
}

#[test]
fn test_chord_requires_both_buttons() {
    let mut map = ActionMap::new();
    map.bind(
        "combo",
        Binding::Chord(vec![Binding::Button(0), Binding::Button(1)]),
    );

    // Only first button
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 0,
        pressure: 1.0,
    });
    assert!(
        events.is_empty(),
        "chord should not trigger with one button"
    );

    // Only second button
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 1,
        pressure: 1.0,
    });
    assert!(
        events.is_empty(),
        "chord should not trigger with one button"
    );
}

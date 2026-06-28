//! Property-based tests for deadzone math and action map.
//! Uses proptest to generate 1000s of random inputs.

use proptest::prelude::*;
use cvkg_inputs::deadzone;

// Strategy: finite f32 in [-1.0, 1.0]
fn finite_f32() -> impl Strategy<Value = f32> {
    -1.0f32..=1.0f32
}

// Strategy: threshold in [0.0, 1.0]
fn threshold() -> impl Strategy<Value = f32> {
    0.0f32..=1.0f32
}

proptest! {

    /// Deadzone output is always in [-1.0, 1.0] for finite inputs.
    #[test]
    fn prop_deadzone_output_bounded(
        value in finite_f32(),
        threshold in threshold(),
    ) {
        let result = deadzone::apply(value, threshold);
        prop_assert!(result >= -1.0 && result <= 1.0, "deadzone out of bounds: {}", result);
    }

    /// Applying deadzone to a value already at 0.0 keeps it at 0.0.
    #[test]
    fn prop_deadzone_zero_stays_zero(
        threshold in threshold(),
    ) {
        let result = deadzone::apply(0.0, threshold);
        prop_assert_eq!(result, 0.0);
    }

    /// Deadzone preserves sign (or maps to zero).
    #[test]
    fn prop_deadzone_preserves_sign(
        value in finite_f32(),
        threshold in threshold(),
    ) {
        let result = deadzone::apply(value, threshold);
        if value > 0.0 {
            prop_assert!(result >= 0.0, "positive input gave negative output");
        } else if value < 0.0 {
            prop_assert!(result <= 0.0, "negative input gave positive output");
        }
    }

    /// Deadzone is monotonic: larger input gives larger (or equal) output.
    #[test]
    fn prop_deadzone_monotonic_positive(
        threshold in threshold(),
    ) {
        let step = 0.01f32;
        let mut prev = deadzone::apply(0.0, threshold);
        for i in 1..=100 {
            let value = (i as f32) * step;
            let result = deadzone::apply(value, threshold);
            prop_assert!(result >= prev - f32::EPSILON, "not monotonic at {}: {} -> {}", value, prev, result);
            prev = result;
        }
    }

    /// Radial deadzone output magnitude is always <= 1.0 (unit circle).
    #[test]
    fn prop_radial_deadzone_output_in_unit_circle(
        x in finite_f32(),
        y in finite_f32(),
        threshold in threshold(),
    ) {
        let (ox, oy) = deadzone::radial(x, y, threshold);
        let output_mag = (ox * ox + oy * oy).sqrt();
        prop_assert!(output_mag <= 1.0 + f32::EPSILON, "radial output outside unit circle: {}", output_mag);
    }

    /// Radial deadzone output is always in unit circle.
    #[test]
    fn prop_radial_output_bounded(
        x in finite_f32(),
        y in finite_f32(),
        threshold in threshold(),
    ) {
        let (ox, oy) = deadzone::radial(x, y, threshold);
        prop_assert!(ox >= -1.0 && ox <= 1.0, "ox out of bounds: {}", ox);
        prop_assert!(oy >= -1.0 && oy <= 1.0, "oy out of bounds: {}", oy);
    }

    /// Radial deadzone preserves direction (ratio y/x).
    #[test]
    fn prop_radial_preserves_direction(
        x in 0.2f32..=1.0f32,
        y in 0.2f32..=1.0f32,
        threshold in 0.0f32..0.15f32,
    ) {
        let (ox, oy) = deadzone::radial(x, y, threshold);
        if ox.abs() > 0.01 {
            let input_ratio = y / x;
            let output_ratio = oy / ox;
            prop_assert!(
                (input_ratio - output_ratio).abs() < 0.1,
                "direction changed: {} -> {}",
                input_ratio, output_ratio
            );
        }
    }

    /// ActionMap evaluate never panics on any event.
    #[test]
    fn prop_action_map_evaluate_no_panic(
        action_name in "[a-z]{1,10}",
        button_idx in 0u32..17,
    ) {
        use cvkg_inputs::{ActionMap, Binding};
        use cvkg_inputs::backend::InputEvent;
        use cvkg_inputs::DeviceId;

        let mut map = ActionMap::new();
        map.bind(&action_name, Binding::Button(button_idx));

        let event = InputEvent::GamepadButton {
            device: DeviceId(0),
            button: button_idx,
            pressure: 1.0,
        };
        let results = map.evaluate(&event);
        prop_assert!(!results.is_empty(), "bound action should trigger");
        prop_assert_eq!(&results[0], &action_name);
    }

    /// ActionMap with no bindings always returns empty.
    #[test]
    fn prop_action_map_empty_always_empty(
        button_idx in 0u32..17,
    ) {
        use cvkg_inputs::ActionMap;
        use cvkg_inputs::backend::InputEvent;
        use cvkg_inputs::DeviceId;

        let map = ActionMap::new();
        let event = InputEvent::GamepadButton {
            device: DeviceId(0),
            button: button_idx,
            pressure: 1.0,
        };
        let results = map.evaluate(&event);
        prop_assert!(results.is_empty());
    }
}

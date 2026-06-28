//! Integration tests: cvkg-inputs ↔ cvkg_core::Event conversion.

use cvkg_inputs::backend::{from_cvkg_event, into_cvkg_event, InputEvent};
use cvkg_inputs::DeviceId;

#[test]
fn test_into_cvkg_event_gamepad_connected() {
    let input_event = InputEvent::GamepadConnected(DeviceId(42));
    let cvkg_event = into_cvkg_event(&input_event).unwrap();
    match cvkg_event {
        cvkg_core::Event::GamepadConnected { id, name } => {
            assert_eq!(id, 42);
            assert!(name.contains("42"));
        }
        other => panic!("expected GamepadConnected, got {other:?}"),
    }
}

#[test]
fn test_into_cvkg_event_gamepad_axis() {
    let input_event = InputEvent::GamepadAxis {
        device: DeviceId(1),
        axis: 0,
        value: 0.75,
    };
    let cvkg_event = into_cvkg_event(&input_event).unwrap();
    match cvkg_event {
        cvkg_core::Event::GamepadAxis { id, axis, value } => {
            assert_eq!(id, 1);
            assert_eq!(axis, 0);
            assert!((value - 0.75).abs() < f32::EPSILON);
        }
        other => panic!("expected GamepadAxis, got {other:?}"),
    }
}

#[test]
fn test_from_cvkg_event_gamepad_button() {
    let cvkg_event = cvkg_core::Event::GamepadButton {
        id: 7,
        button: 3,
        pressure: 1.0,
    };
    let input_event = from_cvkg_event(&cvkg_event).unwrap();
    match input_event {
        InputEvent::GamepadButton { device, button, pressure } => {
            assert_eq!(device, DeviceId(7));
            assert_eq!(button, 3);
            assert!((pressure - 1.0).abs() < f32::EPSILON);
        }
        other => panic!("expected GamepadButton, got {other:?}"),
    }
}

#[test]
fn test_from_cvkg_event_unsupported_returns_none() {
    // FileDrop is not supported in the reverse mapping
    let cvkg_event = cvkg_core::Event::FileDrop {
        x: 0.0,
        y: 0.0,
        path: "/tmp/test".into(),
    };
    let result = from_cvkg_event(&cvkg_event);
    assert!(result.is_none());
}

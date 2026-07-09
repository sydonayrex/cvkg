//! Integration tests: cvkg-inputs ↔ cvkg_core::Event conversion.

use cvkg_inputs::DeviceId;
use cvkg_inputs::backend::{InputEvent, from_cvkg_event, into_cvkg_event};

#[test]
fn test_into_cvkg_event_gamepad_connected() {
    let input_event = InputEvent::GamepadConnected(DeviceId(42));
    let state = cvkg_inputs::InputState::default();
    let cvkg_event = into_cvkg_event(&input_event, &state).unwrap();
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
    let state = cvkg_inputs::InputState::default();
    let cvkg_event = into_cvkg_event(&input_event, &state).unwrap();
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
fn test_into_cvkg_event_mouse_button_uses_absolute_position() {
    let mut state = cvkg_inputs::InputState::default();
    state.mouse.x = 123.0;
    state.mouse.y = 456.0;
    let ev = InputEvent::MouseButton {
        button: 0,
        pressed: true,
    };
    let cv = into_cvkg_event(&ev, &state).unwrap();
    match cv {
        cvkg_core::Event::PointerDown { x, y, button, .. } => {
            assert_eq!(button, 0);
            assert!((x - 123.0).abs() < f32::EPSILON);
            assert!((y - 456.0).abs() < f32::EPSILON);
        }
        other => panic!("expected PointerDown, got {other:?}"),
    }
}

#[test]
fn test_into_cvkg_event_mouse_move_uses_absolute_not_delta() {
    let mut state = cvkg_inputs::InputState::default();
    state.mouse.x = 50.0;
    state.mouse.y = 60.0;
    let ev = InputEvent::MouseMove { dx: 5.0, dy: 6.0 };
    let cv = into_cvkg_event(&ev, &state).unwrap();
    match cv {
        cvkg_core::Event::PointerMove { x, y, .. } => {
            // Must be the absolute position from state, NOT the delta.
            assert!((x - 50.0).abs() < f32::EPSILON, "x={x}");
            assert!((y - 60.0).abs() < f32::EPSILON, "y={y}");
        }
        other => panic!("expected PointerMove, got {other:?}"),
    }
}

#[test]
fn test_into_cvkg_event_keydown_carries_modifiers() {
    let mut state = cvkg_inputs::InputState::default();
    state.keyboard.press("Shift");
    state.keyboard.press("s");
    let ev = InputEvent::KeyDown("s".to_string());
    let cv = into_cvkg_event(&ev, &state).unwrap();
    match cv {
        cvkg_core::Event::KeyDown { key, modifiers } => {
            assert_eq!(key, "s");
            assert!(modifiers.shift);
            assert!(!modifiers.ctrl);
            assert!(!modifiers.alt);
            assert!(!modifiers.meta);
        }
        other => panic!("expected KeyDown, got {other:?}"),
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
        InputEvent::GamepadButton {
            device,
            button,
            pressure,
        } => {
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

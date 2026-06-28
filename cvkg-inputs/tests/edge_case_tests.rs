//! Additional edge-case and error-handling tests.

use cvkg_inputs::backend::{InputBackend, InputEvent, NoopBackend};
use cvkg_inputs::error::InputError;
use cvkg_inputs::{
    ActionMap, Binding, DeviceId, GamepadAxis, GamepadButton, GamepadState, InputState,
    KeyboardState, MouseButton, MouseState, TouchPoint, TouchState,
};

#[test]
fn test_input_state_apply_event_gamepad_disconnect() {
    let mut state = InputState::new();
    state.apply_event(&InputEvent::GamepadConnected(DeviceId(1)));
    assert!(state.gamepads[&DeviceId(1)].connected);

    state.apply_event(&InputEvent::GamepadDisconnected(DeviceId(1)));
    assert!(!state.gamepads[&DeviceId(1)].connected);
}

#[test]
fn test_input_state_apply_event_disconnect_unknown_gamepad() {
    let mut state = InputState::new();
    state.apply_event(&InputEvent::GamepadDisconnected(DeviceId(99)));
}

#[test]
fn test_input_state_apply_event_axis_unknown_device() {
    let mut state = InputState::new();
    state.apply_event(&InputEvent::GamepadAxis {
        device: DeviceId(99),
        axis: 0,
        value: 0.5,
    });
}

#[test]
fn test_input_state_apply_event_button_press_release() {
    let mut state = InputState::new();
    state.apply_event(&InputEvent::GamepadConnected(DeviceId(1)));

    state.apply_event(&InputEvent::GamepadButton {
        device: DeviceId(1),
        button: 0,
        pressure: 1.0,
    });
    assert!(state.gamepads[&DeviceId(1)].button_pressed(GamepadButton::South));

    state.apply_event(&InputEvent::GamepadButton {
        device: DeviceId(1),
        button: 0,
        pressure: 0.0,
    });
    assert!(!state.gamepads[&DeviceId(1)].button_pressed(GamepadButton::South));
}

#[test]
fn test_input_state_apply_event_key_press_release() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::KeyDown("Space".into()));
    assert!(state.keyboard.is_pressed("Space"));

    state.apply_event(&InputEvent::KeyUp("Space".into()));
    assert!(!state.keyboard.is_pressed("Space"));
}

#[test]
fn test_input_state_apply_event_mouse_wheel_accumulates() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::MouseWheel { dx: 0.0, dy: 1.0 });
    state.apply_event(&InputEvent::MouseWheel { dx: 0.0, dy: 1.0 });

    assert!((state.mouse.wheel_dy - 2.0).abs() < f32::EPSILON);
}

#[test]
fn test_input_state_apply_event_touch_cancel_clears_all() {
    let mut state = InputState::new();

    use cvkg_inputs::backend::TouchEvent;
    state.apply_event(&InputEvent::Touch(TouchEvent::Down { id: 0, x: 10.0, y: 20.0 }));
    state.apply_event(&InputEvent::Touch(TouchEvent::Down { id: 1, x: 30.0, y: 40.0 }));
    assert_eq!(state.touch.active_count(), 2);

    state.apply_event(&InputEvent::Touch(TouchEvent::Cancel));
    assert_eq!(state.touch.active_count(), 0);
}

#[test]
fn test_input_error_display() {
    let err = InputError::LockPoisoned;
    assert_eq!(format!("{err}"), "input lock poisoned");

    let err = InputError::BackendInit("test".into());
    assert_eq!(format!("{err}"), "backend init failed: test");

    let err = InputError::DeviceDisconnected(DeviceId(5));
    assert!(format!("{err}").contains("disconnected"));
}

#[test]
fn test_noop_backend_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<NoopBackend>();
}

#[test]
fn test_action_map_set_sensitivity_unknown_action() {
    let mut map = ActionMap::new();
    map.set_sensitivity("nonexistent", 0.5);
}

#[test]
fn test_action_map_set_invert_unknown_action() {
    let mut map = ActionMap::new();
    map.set_invert("nonexistent", true);
}

#[test]
fn test_gamepad_state_axis_default_zero() {
    let state = GamepadState::new();
    assert_eq!(state.axis(GamepadAxis::LeftStickX), 0.0);
}

#[test]
fn test_gamepad_state_button_pressure_default_zero() {
    let state = GamepadState::new();
    assert_eq!(state.button_pressure(GamepadButton::South), 0.0);
    assert!(!state.button_pressed(GamepadButton::South));
}

#[test]
fn test_keyboard_state_release_unknown_key() {
    let mut state = KeyboardState::new();
    state.release("nonexistent");
}

#[test]
fn test_mouse_state_default() {
    let state = MouseState::new();
    assert_eq!(state.x, 0.0);
    assert_eq!(state.y, 0.0);
    assert!(!state.button_pressed(MouseButton::Left));
}

#[test]
fn test_touch_state_operations() {
    let mut state = TouchState::new();

    state.points.insert(0, TouchPoint { id: 0, x: 100.0, y: 200.0, pressure: 0.5 });
    assert_eq!(state.active_count(), 1);

    let point = state.get(0).unwrap();
    assert!((point.x - 100.0).abs() < f32::EPSILON);
    assert!(state.get(999).is_none());
}

#[test]
fn test_device_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(DeviceId(1));
    set.insert(DeviceId(2));
    set.insert(DeviceId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

//! Tests for ActionMap and Binding types.
//! Red-phase: these test the action mapping system.

use cvkg_inputs::{ActionMap, Binding, DeviceId, GamepadAxis, InputEvent, InputState, MouseButton};

#[test]
fn test_action_map_new_is_empty() {
    let map = ActionMap::new();
    let events = map.evaluate(&InputEvent::KeyDown("a".into()));
    assert!(events.is_empty());
}

#[test]
fn test_bind_single_button() {
    let mut map = ActionMap::new();
    map.bind("jump", Binding::Button(0));

    // Button 0 pressed → "jump" triggered
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 0,
        pressure: 1.0,
    });
    assert_eq!(events, vec!["jump"]);

    // Button 1 pressed → nothing
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 1,
        pressure: 1.0,
    });
    assert!(events.is_empty());
}

#[test]
fn test_bind_axis() {
    let mut map = ActionMap::new();
    map.bind("move_x", Binding::Axis(0));

    let events = map.evaluate(&InputEvent::GamepadAxis {
        device: DeviceId(0),
        axis: 0,
        value: 0.5,
    });
    assert_eq!(events, vec!["move_x"]);

    let events = map.evaluate(&InputEvent::GamepadAxis {
        device: DeviceId(0),
        axis: 1,
        value: 0.5,
    });
    assert!(events.is_empty());
}

#[test]
fn test_bind_axis_range() {
    let mut map = ActionMap::new();
    map.bind("trigger_pulled", Binding::AxisRange(4, 0.5, 1.0));

    // Axis 4 at 0.75 → triggered
    let events = map.evaluate(&InputEvent::GamepadAxis {
        device: DeviceId(0),
        axis: 4,
        value: 0.75,
    });
    assert_eq!(events, vec!["trigger_pulled"]);

    // Axis 4 at 0.2 → not triggered
    let events = map.evaluate(&InputEvent::GamepadAxis {
        device: DeviceId(0),
        axis: 4,
        value: 0.2,
    });
    assert!(events.is_empty());

    // Axis 4 at 0.5 → triggered (boundary inclusive)
    let events = map.evaluate(&InputEvent::GamepadAxis {
        device: DeviceId(0),
        axis: 4,
        value: 0.5,
    });
    assert_eq!(events, vec!["trigger_pulled"]);
}

#[test]
fn test_bind_multiple_actions() {
    let mut map = ActionMap::new();
    map.bind("jump", Binding::Button(0));
    map.bind("fire", Binding::Button(1));
    map.bind("pause", Binding::Button(9));

    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 0,
        pressure: 1.0,
    });
    assert_eq!(events, vec!["jump"]);

    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 1,
        pressure: 1.0,
    });
    assert_eq!(events, vec!["fire"]);

    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 9,
        pressure: 1.0,
    });
    assert_eq!(events, vec!["pause"]);
}

#[test]
fn test_button_release_not_triggered() {
    let mut map = ActionMap::new();
    map.bind("jump", Binding::Button(0));

    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 0,
        pressure: 0.0,
    });
    assert!(events.is_empty());
}

#[test]
fn test_chord_binding() {
    let mut map = ActionMap::new();
    map.bind(
        "combo",
        Binding::Chord(vec![Binding::Button(0), Binding::Button(1)]),
    );

    // Only first button → not triggered
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 0,
        pressure: 1.0,
    });
    assert!(events.is_empty());

    // Second button alone → not triggered
    let events = map.evaluate(&InputEvent::GamepadButton {
        device: DeviceId(0),
        button: 1,
        pressure: 1.0,
    });
    assert!(events.is_empty());
}

#[test]
fn test_input_state_apply_event_gamepad_connected() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::GamepadConnected(DeviceId(1)));

    assert!(state.gamepads.contains_key(&DeviceId(1)));
    assert!(state.gamepads[&DeviceId(1)].connected);
}

#[test]
fn test_input_state_apply_event_gamepad_axis() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::GamepadConnected(DeviceId(1)));
    state.apply_event(&InputEvent::GamepadAxis {
        device: DeviceId(1),
        axis: 0,
        value: 0.75,
    });

    assert!(
        (state.gamepads[&DeviceId(1)].axis(GamepadAxis::LeftStickX) - 0.75).abs() < f32::EPSILON
    );
}

#[test]
fn test_input_state_apply_event_keyboard() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::KeyDown("Space".into()));
    assert!(state.keyboard.is_pressed("Space"));

    state.apply_event(&InputEvent::KeyUp("Space".into()));
    assert!(!state.keyboard.is_pressed("Space"));
}

#[test]
fn test_input_state_apply_event_mouse() {
    let mut state = InputState::new();

    state.apply_event(&InputEvent::MouseButton {
        button: 0,
        pressed: true,
    });
    assert!(state.mouse.button_pressed(MouseButton::Left));

    state.apply_event(&InputEvent::MouseButton {
        button: 0,
        pressed: false,
    });
    assert!(!state.mouse.button_pressed(MouseButton::Left));
}

#[test]
fn test_input_state_apply_event_touch() {
    use cvkg_inputs::backend::TouchEvent;
    let mut state = InputState::new();

    state.apply_event(&InputEvent::Touch(TouchEvent::Down {
        id: 0,
        x: 100.0,
        y: 200.0,
    }));
    assert_eq!(state.touch.active_count(), 1);
    assert!((state.touch.get(0).unwrap().x - 100.0).abs() < f32::EPSILON);

    state.apply_event(&InputEvent::Touch(TouchEvent::Move {
        id: 0,
        x: 150.0,
        y: 250.0,
    }));
    assert!((state.touch.get(0).unwrap().x - 150.0).abs() < f32::EPSILON);

    state.apply_event(&InputEvent::Touch(TouchEvent::Up { id: 0 }));
    assert_eq!(state.touch.active_count(), 0);
}

#[test]
fn test_input_state_clone() {
    let mut state = InputState::new();
    state.apply_event(&InputEvent::KeyDown("A".into()));

    let clone = state.clone();
    assert!(clone.keyboard.is_pressed("A"));
}

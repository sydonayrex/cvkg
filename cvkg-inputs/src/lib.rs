//! cvkg-inputs — HID interconnect for CVKG.
//!
//! Provides gamepad, keyboard, mouse, and touch input backends
//! with a unified event type and action mapping system.

#![warn(missing_docs)]

pub mod backend;
pub mod error;

mod action;
mod gamepad;
mod keyboard;
mod mouse;
mod platform;
mod touch;

pub use backend::{InputBackend, InputEvent};
pub use error::InputError;
pub use action::{ActionConfig, ActionMap, Binding};
pub use action::deadzone;
pub use gamepad::{GamepadAxis, GamepadButton, GamepadState};
pub use keyboard::{Key, KeyboardState};
pub use mouse::{MouseButton, MouseState};
pub use touch::{TouchPoint, TouchState};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Unique identifier for an input device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub u64);

/// Central aggregate state for all input devices.
///
/// Clone-friendly: wraps `Arc<RwLock<...>>` internally so it can be
/// shared with the VDOM render thread.
#[derive(Clone, Debug, Default)]
pub struct InputState {
    /// Connected gamepads, keyed by device ID.
    pub gamepads: HashMap<DeviceId, GamepadState>,
    /// Current keyboard state.
    pub keyboard: KeyboardState,
    /// Current mouse state.
    pub mouse: MouseState,
    /// Current touch state.
    pub touch: TouchState,
    /// Action mapping configuration.
    pub action_map: ActionMap,
}

impl InputState {
    /// Creates a new empty `InputState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies an input event to the state.
    pub fn apply_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::GamepadConnected(id) => {
                let mut state = GamepadState::new();
                state.connected = true;
                self.gamepads.insert(*id, state);
            }
            InputEvent::GamepadDisconnected(id) => {
                if let Some(state) = self.gamepads.get_mut(id) {
                    state.connected = false;
                }
            }
            InputEvent::GamepadAxis { device, axis, value } => {
                if let Some(state) = self.gamepads.get_mut(device) {
                    // Map raw axis index to standard axis
                    let axis = match axis {
                        0 => GamepadAxis::LeftStickX,
                        1 => GamepadAxis::LeftStickY,
                        2 => GamepadAxis::RightStickX,
                        3 => GamepadAxis::RightStickY,
                        4 => GamepadAxis::LeftTrigger,
                        5 => GamepadAxis::RightTrigger,
                        other => GamepadAxis::Raw(*other),
                    };
                    state.axes.insert(axis, *value);
                }
            }
            InputEvent::GamepadButton { device, button, pressure } => {
                if let Some(state) = self.gamepads.get_mut(device) {
                    let button = match button {
                        0 => GamepadButton::South,
                        1 => GamepadButton::East,
                        2 => GamepadButton::West,
                        3 => GamepadButton::North,
                        4 => GamepadButton::LeftBumper,
                        5 => GamepadButton::RightBumper,
                        6 => GamepadButton::LeftTrigger,
                        7 => GamepadButton::RightTrigger,
                        8 => GamepadButton::Select,
                        9 => GamepadButton::Start,
                        10 => GamepadButton::LeftStick,
                        11 => GamepadButton::RightStick,
                        12 => GamepadButton::DpadUp,
                        13 => GamepadButton::DpadDown,
                        14 => GamepadButton::DpadLeft,
                        15 => GamepadButton::DpadRight,
                        16 => GamepadButton::Home,
                        other => GamepadButton::Raw(*other),
                    };
                    if *pressure > 0.0 {
                        state.buttons.insert(button, *pressure);
                    } else {
                        state.buttons.remove(&button);
                    }
                }
            }
            InputEvent::KeyDown(key) => {
                self.keyboard.press(key.clone());
            }
            InputEvent::KeyUp(key) => {
                self.keyboard.release(key);
            }
            InputEvent::MouseMove { dx, dy } => {
                self.mouse.x += dx;
                self.mouse.y += dy;
            }
            InputEvent::MouseButton { button, pressed } => {
                let button = match button {
                    0 => MouseButton::Left,
                    1 => MouseButton::Right,
                    2 => MouseButton::Middle,
                    3 => MouseButton::Back,
                    4 => MouseButton::Forward,
                    other => MouseButton::Raw(*other),
                };
                if *pressed {
                    self.mouse.pressed.insert(button);
                } else {
                    self.mouse.pressed.remove(&button);
                }
            }
            InputEvent::MouseWheel { dx, dy } => {
                self.mouse.wheel_dx += dx;
                self.mouse.wheel_dy += dy;
            }
            InputEvent::Touch(touch_event) => match touch_event {
                backend::TouchEvent::Down { id, x, y } => {
                    self.touch.points.insert(*id, TouchPoint { id: *id, x: *x, y: *y, pressure: 1.0 });
                }
                backend::TouchEvent::Move { id, x, y } => {
                    if let Some(point) = self.touch.points.get_mut(id) {
                        point.x = *x;
                        point.y = *y;
                    }
                }
                backend::TouchEvent::Up { id } => {
                    self.touch.points.remove(id);
                }
                backend::TouchEvent::Cancel => {
                    self.touch.points.clear();
                }
            },
        }
    }
}

/// The input system owns backends and aggregates their events into [`InputState`].
pub struct InputSystem {
    backends: Vec<Box<dyn InputBackend>>,
    state: Arc<RwLock<InputState>>,
}

impl InputSystem {
    /// Creates a new `InputSystem` with no backends.
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            state: Arc::new(RwLock::new(InputState::new())),
        }
    }

    /// Adds a backend to the system.
    pub fn add_backend(&mut self, backend: Box<dyn InputBackend>) {
        self.backends.push(backend);
    }

    /// Polls all backends and updates the shared state.
    pub fn poll(&mut self) -> Result<(), InputError> {
        let mut state = self.state.write().map_err(|_| InputError::LockPoisoned)?;
        for backend in &mut self.backends {
            let events = backend.poll();
            for event in events {
                state.apply_event(&event);
            }
        }
        Ok(())
    }

    /// Returns a clone of the shared state handle.
    pub fn state(&self) -> Arc<RwLock<InputState>> {
        self.state.clone()
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_new_is_empty() {
        let state = InputState::new();
        assert!(state.gamepads.is_empty());
    }

    #[test]
    fn test_input_system_new_has_no_backends() {
        let sys = InputSystem::new();
        assert_eq!(sys.backends.len(), 0);
    }

    #[test]
    fn test_device_id_equality() {
        let a = DeviceId(1);
        let b = DeviceId(1);
        let c = DeviceId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}

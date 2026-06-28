//! Input backend trait and event types.

mod noop_backend;
pub use noop_backend::NoopBackend;

#[cfg(feature = "gilrs")]
mod gilrs_backend;
#[cfg(feature = "gilrs")]
pub use gilrs_backend::GilrsBackend;

#[cfg(feature = "evdev")]
mod evdev_backend;
#[cfg(feature = "evdev")]
pub use evdev_backend::EvdevBackend;

use crate::error::InputError;

/// Converts a cvkg-inputs `InputEvent` into a `cvkg_core::Event`.
#[inline]
pub fn into_cvkg_event(event: &InputEvent) -> Option<cvkg_core::Event> {
    match event {
        InputEvent::GamepadConnected(id) => Some(cvkg_core::Event::GamepadConnected {
            id: id.0,
            name: format!("gamepad-{}", id.0),
        }),
        InputEvent::GamepadDisconnected(id) => Some(cvkg_core::Event::GamepadDisconnected { id: id.0 }),
        InputEvent::GamepadAxis { device, axis, value } => Some(cvkg_core::Event::GamepadAxis {
            id: device.0,
            axis: *axis,
            value: *value,
        }),
        InputEvent::GamepadButton { device, button, pressure } => Some(cvkg_core::Event::GamepadButton {
            id: device.0,
            button: *button,
            pressure: *pressure,
        }),
        InputEvent::KeyDown(key) => Some(cvkg_core::Event::KeyDown {
            key: key.clone(),
            modifiers: cvkg_core::KeyModifiers::default(),
        }),
        InputEvent::KeyUp(key) => Some(cvkg_core::Event::KeyUp {
            key: key.clone(),
            modifiers: cvkg_core::KeyModifiers::default(),
        }),
        InputEvent::MouseMove { dx, dy } => Some(cvkg_core::Event::PointerMove {
            x: *dx,
            y: *dy,
            proximity_field: 0.0,
            tilt: None,
            azimuth: None,
            pressure: None,
            barrel_rotation: None,
            pointer_precision: 0.0,
        }),
        _ => None,
    }
}

/// Converts a cvkg_core::Event into a cvkg-inputs `InputEvent` (reverse mapping).
#[inline]
pub fn from_cvkg_event(event: &cvkg_core::Event) -> Option<InputEvent> {
    match event {
        cvkg_core::Event::GamepadConnected { id, .. } => {
            Some(InputEvent::GamepadConnected(crate::DeviceId(*id)))
        }
        cvkg_core::Event::GamepadDisconnected { id } => {
            Some(InputEvent::GamepadDisconnected(crate::DeviceId(*id)))
        }
        cvkg_core::Event::GamepadAxis { id, axis, value } => Some(InputEvent::GamepadAxis {
            device: crate::DeviceId(*id),
            axis: *axis,
            value: *value,
        }),
        cvkg_core::Event::GamepadButton {
            id,
            button,
            pressure,
        } => Some(InputEvent::GamepadButton {
            device: crate::DeviceId(*id),
            button: *button,
            pressure: *pressure,
        }),
        _ => None,
    }
}

/// Trait for pluggable input backends.
///
/// Each backend produces [`InputEvent`]s that the [`InputSystem`](crate::InputSystem)
/// aggregates into [`InputState`](crate::InputState).
///
/// Backends must be `Send` (to transfer ownership to the poll thread) but
/// not necessarily `Sync` (gilrs::Gilrs is !Sync).
pub trait InputBackend: Send {
    /// Returns the human-readable name of this backend.
    fn name(&self) -> &str;

    /// Polls for new events. Called once per frame.
    fn poll(&mut self) -> Vec<InputEvent>;

    /// Sets force-feedback rumble on a device (if supported).
    fn set_rumble(
        &mut self,
        device: DeviceId,
        weak: f32,
        strong: f32,
    ) -> Result<(), InputError>;
}

/// Unified input event type. Converted from backend-native events.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A gamepad was connected.
    GamepadConnected(DeviceId),
    /// A gamepad was disconnected.
    GamepadDisconnected(DeviceId),
    /// A gamepad axis moved.
    GamepadAxis {
        /// Device ID.
        device: DeviceId,
        /// Axis identifier.
        axis: u32,
        /// Axis value in [-1.0, 1.0].
        value: f32,
    },
    /// A gamepad button was pressed or released.
    GamepadButton {
        /// Device ID.
        device: DeviceId,
        /// Button identifier.
        button: u32,
        /// Pressure in [0.0, 1.0].
        pressure: f32,
    },
    /// A key was pressed.
    KeyDown(String),
    /// A key was released.
    KeyUp(String),
    /// Mouse moved (relative).
    MouseMove {
        /// Relative X delta.
        dx: f32,
        /// Relative Y delta.
        dy: f32,
    },
    /// A mouse button was pressed or released.
    MouseButton {
        /// Button index.
        button: u32,
        /// True if pressed.
        pressed: bool,
    },
    /// Mouse wheel scrolled.
    MouseWheel {
        /// Horizontal delta.
        dx: f32,
        /// Vertical delta.
        dy: f32,
    },
    /// Touch event.
    Touch(TouchEvent),
}

/// Touch-specific event.
#[derive(Debug, Clone, PartialEq)]
pub enum TouchEvent {
    /// A touch point began.
    Down {
        /// Touch point ID.
        id: u64,
        /// X coordinate.
        x: f32,
        /// Y coordinate.
        y: f32,
    },
    /// A touch point moved.
    Move {
        /// Touch point ID.
        id: u64,
        /// X coordinate.
        x: f32,
        /// Y coordinate.
        y: f32,
    },
    /// A touch point ended.
    Up {
        /// Touch point ID.
        id: u64,
    },
    /// All touch points cancelled.
    Cancel,
}

/// Re-export DeviceId for convenience.
pub type DeviceId = crate::DeviceId;

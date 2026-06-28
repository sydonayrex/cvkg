//! Gilrs-based cross-platform gamepad backend.

use crate::backend::{InputBackend, InputEvent};
use crate::error::InputError;
use crate::DeviceId;

/// Backend using the `gilrs` crate for cross-platform gamepad support.
///
/// Feature-gated behind the `gilrs` feature (default).
#[cfg(feature = "gilrs")]
pub struct GilrsBackend {
    gilrs: gilrs::Gilrs,
}

#[cfg(feature = "gilrs")]
impl GilrsBackend {
    /// Creates a new GilrsBackend, initializing gilrs.
    pub fn new() -> Result<Self, InputError> {
        let gilrs = gilrs::Gilrs::new().map_err(|e| InputError::BackendInit(e.to_string()))?;
        Ok(Self { gilrs })
    }
}

#[cfg(feature = "gilrs")]
impl InputBackend for GilrsBackend {
    fn name(&self) -> &str {
        "gilrs"
    }

    fn poll(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                gilrs::EventType::Connected => {
                    events.push(InputEvent::GamepadConnected(DeviceId(id_into_u64(event.id))));
                }
                gilrs::EventType::Disconnected => {
                    events.push(InputEvent::GamepadDisconnected(DeviceId(id_into_u64(event.id))));
                }
                gilrs::EventType::AxisChanged(axis, value, _code) => {
                    let axis_index = gilrs_axis_index(axis);
                    events.push(InputEvent::GamepadAxis {
                        device: DeviceId(id_into_u64(event.id)),
                        axis: axis_index,
                        value,
                    });
                }
                gilrs::EventType::ButtonChanged(button, value, _code) => {
                    let button_index = gilrs_button_index(button);
                    events.push(InputEvent::GamepadButton {
                        device: DeviceId(id_into_u64(event.id)),
                        button: button_index,
                        pressure: value,
                    });
                }
                _ => {}
            }
        }
        events
    }

    fn set_rumble(&mut self, device: DeviceId, weak: f32, strong: f32) -> Result<(), InputError> {
        use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Repeat};
        use std::time::Duration;

        // Convert deviceId to gilrs GamepadId
        // We need to find the matching gamepad from our connected gamepads
        // Since GamepadId can't be constructed directly, we iterate connected gamepads
        // and match by index
        let target_idx = device.0 as usize;
        let mut found = false;

        for (id, gamepad) in self.gilrs.gamepads() {
            if usize::from(id) == target_idx {
                if gamepad.is_ff_supported() {
                    let strong_magnitude = ((strong * 65535.0) as u16).max(1);
                    let weak_magnitude = ((weak * 65535.0) as u16).max(1);

                    let effect = EffectBuilder::new()
                        .add_effect(BaseEffect {
                            kind: BaseEffectType::Strong {
                                magnitude: strong_magnitude,
                            },
                            scheduling: Replay {
                                play_for: Duration::from_secs(30).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .add_effect(BaseEffect {
                            kind: BaseEffectType::Weak {
                                magnitude: weak_magnitude,
                            },
                            scheduling: Replay {
                                play_for: Duration::from_secs(30).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .repeat(Repeat::Infinitely)
                        .add_gamepad(&gamepad)
                        .finish(&mut self.gilrs)
                        .map_err(|e| InputError::Platform(format!("rumble creation failed: {e}")))?;

                    effect.play().map_err(|e| {
                        InputError::Platform(format!("rumble play failed: {e}"))
                    })?;
                }
                found = true;
                break;
            }
        }

        if !found {
            return Err(InputError::DeviceDisconnected(device));
        }

        Ok(())
    }
}

#[cfg(feature = "gilrs")]
fn id_into_u64(id: gilrs::GamepadId) -> u64 {
    usize::from(id) as u64
}

#[cfg(feature = "gilrs")]
fn gilrs_axis_index(axis: gilrs::Axis) -> u32 {
    use gilrs::Axis::*;
    match axis {
        LeftStickX => 0,
        LeftStickY => 1,
        RightStickX => 2,
        RightStickY => 3,
        LeftZ => 4,
        RightZ => 5,
        DPadX => 6,
        DPadY => 7,
        Unknown => 8,
    }
}

#[cfg(feature = "gilrs")]
fn gilrs_button_index(button: gilrs::Button) -> u32 {
    use gilrs::Button::*;
    match button {
        South => 0,
        East => 1,
        West => 2,
        North => 3,
        LeftTrigger => 4,
        RightTrigger => 5,
        LeftTrigger2 => 6,
        RightTrigger2 => 7,
        Select => 8,
        Start => 9,
        LeftThumb => 10,
        RightThumb => 11,
        DPadUp => 12,
        DPadDown => 13,
        DPadLeft => 14,
        DPadRight => 15,
        Mode => 16,
        _ => 17,
    }
}

#[cfg(not(feature = "gilrs"))]
pub struct GilrsBackend;

#[cfg(not(feature = "gilrs"))]
impl GilrsBackend {
    /// Fails immediately because gilrs feature is disabled.
    pub fn new() -> Result<Self, InputError> {
        Err(InputError::BackendInit(
            "gilrs feature not enabled".into(),
        ))
    }
}

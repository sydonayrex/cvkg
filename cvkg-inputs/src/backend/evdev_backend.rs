//! Linux evdev backend for raw HID access.
//!
//! Feature-gated behind `evdev`. Linux-only.

use crate::backend::{InputBackend, InputEvent};
use crate::error::InputError;
use crate::platform::HidDeviceInfo;
use crate::DeviceId;

#[cfg(all(feature = "evdev", target_os = "linux"))]
pub struct EvdevBackend {
    devices: Vec<evdev::Device>,
    device_ids: Vec<DeviceId>,
}

#[cfg(all(feature = "evdev", target_os = "linux"))]
impl EvdevBackend {
    /// Creates a new EvdevBackend scanning /dev/input/.
    pub fn new() -> Result<Self, InputError> {
        let mut devices = Vec::new();
        let mut device_ids = Vec::new();

        // Enumerate /dev/input/event* devices
        let paths = glob::glob("/dev/input/event*")
            .map_err(|e| InputError::BackendInit(format!("glob failed: {e}")))?;

        for (idx, path) in paths.enumerate() {
            let path = path.map_err(|e| {
                InputError::BackendInit(format!("read dir failed: {e}"))
            })?;
            if let Ok(device) = evdev::Device::open(&path) {
                device_ids.push(DeviceId(idx as u64));
                devices.push(device);
            }
        }

        Ok(Self { devices, device_ids })
    }

    /// Returns info about all enumerated HID devices.
    pub fn enumerate(&self) -> Vec<HidDeviceInfo> {
        self.devices
            .iter()
            .zip(&self.device_ids)
            .filter_map(|(dev, id)| {
                Some(HidDeviceInfo {
                    path: format!("/dev/input/event{}", id.0),
                    name: dev.name().unwrap_or("Unknown").to_string(),
                    vendor_id: dev.input_id().vendor(),
                    product_id: dev.input_id().product(),
                })
            })
            .collect()
    }
}

#[cfg(all(feature = "evdev", target_os = "linux"))]
impl InputBackend for EvdevBackend {
    fn name(&self) -> &str {
        "evdev"
    }

    fn poll(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();

        for (dev, id) in self.devices.iter_mut().zip(&self.device_ids) {
            while let Ok(batch) = dev.fetch_events() {
                for event in batch {
                    match event.event_type() {
                        evdev::EventType::KEY => {
                            let key = evdev::Key::new(event.code());
                            let key_name = format!("{key:?}");
                            match event.value() {
                                0 => events.push(InputEvent::KeyUp(key_name)),
                                1 => events.push(InputEvent::KeyDown(key_name)),
                                2 => {
                                    // Key repeat — treat as key down
                                    events.push(InputEvent::KeyDown(key_name));
                                }
                                _ => {}
                            }
                        }
                        evdev::EventType::RELATIVE => {
                            match event.code() {
                                0 => {
                                    // REL_X
                                    events.push(InputEvent::MouseMove {
                                        dx: event.value() as f32,
                                        dy: 0.0,
                                    });
                                }
                                1 => {
                                    // REL_Y
                                    events.push(InputEvent::MouseMove {
                                        dx: 0.0,
                                        dy: event.value() as f32,
                                    });
                                }
                                8 => {
                                    // REL_WHEEL
                                    events.push(InputEvent::MouseWheel {
                                        dx: 0.0,
                                        dy: event.value() as f32,
                                    });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                break; // Only read once per poll
            }
        }

        events
    }

    fn set_rumble(
        &mut self,
        _device: DeviceId,
        _weak: f32,
        _strong: f32,
    ) -> Result<(), InputError> {
        // Evdev rumble requires FF_* events — not implemented yet
        Ok(())
    }
}

// Stub when evdev is enabled but not on Linux
#[cfg(all(feature = "evdev", not(target_os = "linux")))]
pub struct EvdevBackend;

#[cfg(all(feature = "evdev", not(target_os = "linux")))]
impl EvdevBackend {
    /// Fails because evdev is only available on Linux.
    pub fn new() -> Result<Self, InputError> {
        Err(InputError::BackendInit(
            "evdev backend is only available on Linux".into(),
        ))
    }
}

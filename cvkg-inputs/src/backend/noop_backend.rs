//! Noop backend for testing and headless environments.

use crate::backend::{InputBackend, InputEvent};
use crate::error::InputError;
use crate::DeviceId;

/// A backend that never produces events.
///
/// Used as a fallback when no real input system is available.
pub struct NoopBackend;

impl NoopBackend {
    /// Creates a new noop backend.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBackend for NoopBackend {
    fn name(&self) -> &str {
        "noop"
    }

    fn poll(&mut self) -> Vec<InputEvent> {
        Vec::new()
    }

    fn set_rumble(
        &mut self,
        _device: DeviceId,
        _weak: f32,
        _strong: f32,
    ) -> Result<(), InputError> {
        // No-op: pretend success
        Ok(())
    }
}

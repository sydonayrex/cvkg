//! Input error types.

use std::fmt;

/// Errors that can occur in the input system.
#[derive(Debug, Clone, PartialEq)]
pub enum InputError {
    /// A mutex or rwlock was poisoned by a panicking thread.
    LockPoisoned,
    /// A backend failed to initialize.
    BackendInit(String),
    /// A device is not connected.
    DeviceDisconnected(DeviceId),
    /// HID device not found.
    DeviceNotFound(String),
    /// Platform-specific error.
    Platform(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockPoisoned => write!(f, "input lock poisoned"),
            Self::BackendInit(msg) => write!(f, "backend init failed: {msg}"),
            Self::DeviceDisconnected(id) => write!(f, "device {:?} disconnected", id),
            Self::DeviceNotFound(path) => write!(f, "device not found: {path}"),
            Self::Platform(msg) => write!(f, "platform error: {msg}"),
        }
    }
}

impl std::error::Error for InputError {}

/// Re-export for internal use.
pub type DeviceId = crate::DeviceId;

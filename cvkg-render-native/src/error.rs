//! Error types for the native rendering backend.
//!
//! Covers failures in window creation, GPU initialization, VDom diffing,
//! window lifecycle, and event loop operations.

use std::fmt;

/// Errors that can occur in the native rendering backend.
#[derive(Debug)]
pub enum NativeError {
    /// Window creation failed (e.g., display server unavailable).
    WindowCreation(String),
    /// GPU initialization failed (e.g., driver issues).
    GpuInit(String),
    /// VDom diff produced no patches but a rebuild was expected. This is a bug.
    DiffEmpty,
    /// A window was destroyed but events are still being dispatched.
    WindowDestroyed(winit::window::WindowId),
    /// An error occurred in the event loop.
    EventLoop(String),
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeError::WindowCreation(msg) => write!(
                f,
                "Window creation failed: {msg}. Check display server connection."
            ),
            NativeError::GpuInit(msg) => write!(
                f,
                "GPU initialization failed: {msg}. Verify drivers and GPU availability."
            ),
            NativeError::DiffEmpty => write!(
                f,
                "VDom diff produced no patches but rebuild was expected. This is a bug."
            ),
            NativeError::WindowDestroyed(id) => write!(
                f,
                "Window {id:?} was destroyed but events are still being dispatched."
            ),
            NativeError::EventLoop(msg) => write!(f, "Event loop error: {msg}"),
        }
    }
}

impl std::error::Error for NativeError {}

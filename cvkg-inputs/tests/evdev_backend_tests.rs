//! Tests for the evdev backend (Linux-only).
//! These tests are skipped on non-Linux platforms or when evdev feature is disabled.

#![cfg(all(target_os = "linux", feature = "evdev"))]

use cvkg_inputs::backend::{InputBackend, InputEvent};

#[test]
fn test_evdev_backend_creation() {
    let backend = cvkg_inputs::backend::EvdevBackend::new();
    assert!(backend.is_ok());
}

#[test]
fn test_evdev_backend_name() {
    if let Ok(backend) = cvkg_inputs::backend::EvdevBackend::new() {
        assert_eq!(backend.name(), "evdev");
    }
}

#[test]
fn test_evdev_enumerate_returns_devices() {
    if let Ok(backend) = cvkg_inputs::backend::EvdevBackend::new() {
        let devices = backend.enumerate();
        for dev in &devices {
            assert!(!dev.path.is_empty());
        }
    }
}

#[test]
fn test_evdev_poll_no_panic() {
    if let Ok(mut backend) = cvkg_inputs::backend::EvdevBackend::new() {
        let events = backend.poll();
        for event in events {
            match event {
                InputEvent::KeyDown(_) | InputEvent::KeyUp(_) => {}
                InputEvent::MouseMove { .. } => {}
                _ => {}
            }
        }
    }
}

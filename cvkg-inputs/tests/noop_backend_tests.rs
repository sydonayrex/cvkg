//! Tests for the noop backend.

use cvkg_inputs::backend::{InputBackend, InputEvent, NoopBackend};
use cvkg_inputs::DeviceId;

#[test]
fn test_noop_backend_name() {
    let backend = NoopBackend::new();
    assert_eq!(backend.name(), "noop");
}

#[test]
fn test_noop_backend_poll_always_empty() {
    let mut backend = NoopBackend::new();
    let events = backend.poll();
    assert!(events.is_empty());
    // Multiple polls still empty
    let events2 = backend.poll();
    assert!(events2.is_empty());
}

#[test]
fn test_noop_backend_rumble_always_ok() {
    let mut backend = NoopBackend::new();
    let result = backend.set_rumble(DeviceId(0), 1.0, 1.0);
    assert!(result.is_ok());
}

#[test]
fn test_noop_backend_default() {
    let backend: NoopBackend = Default::default();
    assert_eq!(backend.name(), "noop");
}

#[test]
fn test_input_system_with_noop_backend() {
    use cvkg_inputs::InputSystem;
    let mut system = InputSystem::new();
    system.add_backend(Box::new(NoopBackend::new()));

    let result = system.poll();
    assert!(result.is_ok());
}

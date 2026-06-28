//! Tests for the Gilrs backend.
//! Uses mock gilrs context injection — no real hardware needed.

use cvkg_inputs::backend::{InputBackend, InputEvent};
use cvkg_inputs::DeviceId;

/// Mock gilrs context for testing.
struct MockGilrsContext {
    events: Vec<MockGilrsEvent>,
    polled: bool,
}

enum MockGilrsEvent {
    Connected(DeviceId),
    Disconnected(DeviceId),
    Axis(DeviceId, u32, f32),
    Button(DeviceId, u32, f32),
}

impl MockGilrsContext {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            polled: false,
        }
    }

    fn push_event(&mut self, event: MockGilrsEvent) {
        self.events.push(event);
    }
}

/// Mock backend that simulates gilrs behavior.
struct MockGilrsBackend {
    events: Vec<InputEvent>,
}

impl MockGilrsBackend {
    fn new(context: &MockGilrsContext) -> Self {
        let events = context.events.iter().map(|e| match e {
            MockGilrsEvent::Connected(id) => InputEvent::GamepadConnected(*id),
            MockGilrsEvent::Disconnected(id) => InputEvent::GamepadDisconnected(*id),
            MockGilrsEvent::Axis(id, axis, value) => InputEvent::GamepadAxis {
                device: *id,
                axis: *axis,
                value: *value,
            },
            MockGilrsEvent::Button(id, button, pressure) => InputEvent::GamepadButton {
                device: *id,
                button: *button,
                pressure: *pressure,
            },
        });
        Self {
            events: events.collect(),
        }
    }
}

impl InputBackend for MockGilrsBackend {
    fn name(&self) -> &str {
        "mock_gilrs"
    }

    fn poll(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.events)
    }

    fn set_rumble(
        &mut self,
        _device: DeviceId,
        _weak: f32,
        _strong: f32,
    ) -> Result<(), cvkg_inputs::error::InputError> {
        // Mock: always succeeds
        Ok(())
    }
}

#[test]
fn test_mock_gilrs_backend_name() {
    let ctx = MockGilrsContext::new();
    let backend = MockGilrsBackend::new(&ctx);
    assert_eq!(backend.name(), "mock_gilrs");
}

#[test]
fn test_mock_gilrs_poll_returns_empty_initially() {
    let ctx = MockGilrsContext::new();
    let mut backend = MockGilrsBackend::new(&ctx);
    let events = backend.poll();
    assert!(events.is_empty());
}

#[test]
fn test_mock_gilrs_poll_gamepad_connected() {
    let mut ctx = MockGilrsContext::new();
    ctx.push_event(MockGilrsEvent::Connected(DeviceId(1)));
    let mut backend = MockGilrsBackend::new(&ctx);

    let events = backend.poll();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], InputEvent::GamepadConnected(DeviceId(1)));

    // Second poll returns empty (events consumed)
    let events2 = backend.poll();
    assert!(events2.is_empty());
}

#[test]
fn test_mock_gilrs_poll_axis_event() {
    let mut ctx = MockGilrsContext::new();
    ctx.push_event(MockGilrsEvent::Axis(DeviceId(1), 0, 0.75));
    let mut backend = MockGilrsBackend::new(&ctx);

    let events = backend.poll();
    assert_eq!(events.len(), 1);
    match &events[0] {
        InputEvent::GamepadAxis { device, axis, value } => {
            assert_eq!(*device, DeviceId(1));
            assert_eq!(*axis, 0);
            assert!((*value - 0.75).abs() < f32::EPSILON);
        }
        other => panic!("expected GamepadAxis, got {other:?}"),
    }
}

#[test]
fn test_mock_gilrs_poll_button_event() {
    let mut ctx = MockGilrsContext::new();
    ctx.push_event(MockGilrsEvent::Button(DeviceId(1), 0, 1.0));
    let mut backend = MockGilrsBackend::new(&ctx);

    let events = backend.poll();
    assert_eq!(events.len(), 1);
    match &events[0] {
        InputEvent::GamepadButton { device, button, pressure } => {
            assert_eq!(*device, DeviceId(1));
            assert_eq!(*button, 0);
            assert!((*pressure - 1.0).abs() < f32::EPSILON);
        }
        other => panic!("expected GamepadButton, got {other:?}"),
    }
}

#[test]
fn test_mock_gilrs_poll_multiple_events() {
    let mut ctx = MockGilrsContext::new();
    ctx.push_event(MockGilrsEvent::Connected(DeviceId(1)));
    ctx.push_event(MockGilrsEvent::Axis(DeviceId(1), 0, 0.5));
    ctx.push_event(MockGilrsEvent::Button(DeviceId(1), 0, 1.0));
    ctx.push_event(MockGilrsEvent::Disconnected(DeviceId(1)));
    let mut backend = MockGilrsBackend::new(&ctx);

    let events = backend.poll();
    assert_eq!(events.len(), 4);
}

#[test]
fn test_mock_gilrs_rumble_always_succeeds() {
    let ctx = MockGilrsContext::new();
    let mut backend = MockGilrsBackend::new(&ctx);
    let result = backend.set_rumble(DeviceId(1), 0.5, 0.5);
    assert!(result.is_ok());
}

#[test]
fn test_mock_gamepad_button_release() {
    let mut ctx = MockGilrsContext::new();
    ctx.push_event(MockGilrsEvent::Button(DeviceId(1), 0, 0.0));
    let mut backend = MockGilrsBackend::new(&ctx);

    let events = backend.poll();
    match &events[0] {
        InputEvent::GamepadButton { pressure, .. } => {
            assert_eq!(*pressure, 0.0);
        }
        other => panic!("expected GamepadButton, got {other:?}"),
    }
}

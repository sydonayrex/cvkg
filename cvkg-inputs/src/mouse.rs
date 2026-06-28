/// Mouse state.

/// Standard mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button (wheel press).
    Middle,
    /// Side button (back).
    Back,
    /// Forward side button.
    Forward,
    /// Raw button index.
    Raw(u32),
}

/// Current mouse state.
#[derive(Debug, Clone, Default)]
pub struct MouseState {
    /// Currently pressed mouse buttons.
    pub pressed: std::collections::HashSet<MouseButton>,
    /// Absolute X position.
    pub x: f32,
    /// Absolute Y position.
    pub y: f32,
    /// Wheel delta since last poll.
    pub wheel_dx: f32,
    /// Wheel delta since last poll.
    pub wheel_dy: f32,
}

impl MouseState {
    /// Creates a new mouse state at origin.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the given button is pressed.
    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }
}

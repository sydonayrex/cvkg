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
    /// Accumulated wheel-X delta this poll cycle. Reset by `take_wheel_deltas`.
    pub wheel_dx: f32,
    /// Accumulated wheel-Y delta this poll cycle. Reset by `take_wheel_deltas`.
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

    /// Returns the current wheel deltas and resets them to zero.
    ///
    /// Consumers MUST call this once per poll cycle to read fresh
    /// per-frame wheel deltas instead of monotonically-accumulating sums.
    pub fn take_wheel_deltas(&mut self) -> (f32, f32) {
        let dx = self.wheel_dx;
        let dy = self.wheel_dy;
        self.wheel_dx = 0.0;
        self.wheel_dy = 0.0;
        (dx, dy)
    }
}

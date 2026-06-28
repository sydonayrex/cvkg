/// Gamepad state, button, and axis types.
use std::collections::HashMap;

/// Standard gamepad buttons (Xbox/PS/Switch layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// A / Cross (bottom face button)
    South,
    /// B / Circle (right face button)
    East,
    /// X / Square (left face button)
    West,
    /// Y / Triangle (top face button)
    North,
    /// Left bumper.
    LeftBumper,
    /// Right bumper.
    RightBumper,
    /// Left trigger (digital).
    LeftTrigger,
    /// Right trigger (digital).
    RightTrigger,
    /// Left stick press.
    LeftStick,
    /// Right stick press.
    RightStick,
    /// D-pad up.
    DpadUp,
    /// D-pad down.
    DpadDown,
    /// D-pad left.
    DpadLeft,
    /// D-pad right.
    DpadRight,
    /// Start/Options button.
    Start,
    /// Select/Share button.
    Select,
    /// Home/Guide button.
    Home,
    /// Raw button index for non-standard buttons.
    Raw(u32),
}

/// Standard gamepad axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// Left stick X.
    LeftStickX,
    /// Left stick Y.
    LeftStickY,
    /// Right stick X.
    RightStickX,
    /// Right stick Y.
    RightStickY,
    /// Left trigger (analog).
    LeftTrigger,
    /// Right trigger (analog).
    RightTrigger,
    /// Raw axis index for non-standard axes.
    Raw(u32),
}

/// State of a single gamepad.
#[derive(Debug, Clone, Default)]
pub struct GamepadState {
    /// Currently pressed buttons.
    pub buttons: HashMap<GamepadButton, f32>,
    /// Current axis values.
    pub axes: HashMap<GamepadAxis, f32>,
    /// Whether this gamepad is currently connected.
    pub connected: bool,
}

impl GamepadState {
    /// Creates a new disconnected gamepad state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current value of an axis, or 0.0 if not set.
    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }

    /// Returns whether a button is currently pressed.
    pub fn button_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.get(&button).map_or(false, |v| *v > 0.0)
    }

    /// Returns the pressure on a button, or 0.0 if not pressed.
    pub fn button_pressure(&self, button: GamepadButton) -> f32 {
        self.buttons.get(&button).copied().unwrap_or(0.0)
    }
}

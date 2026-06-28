//! Action mapping system.

use crate::backend::InputEvent;
use std::collections::HashMap;

/// A binding from a physical input to an action.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Binding {
    /// A button press.
    Button(u32),
    /// An axis value.
    Axis(u32),
    /// An axis range trigger (activates when axis is in [min, max]).
    AxisRange(u32, f32, f32),
    /// A chord (all sub-bindings must be active).
    Chord(Vec<Binding>),
}

/// Per-action configuration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionConfig {
    /// Binding for this action.
    pub binding: Binding,
    /// Axis sensitivity multiplier (default 1.0).
    pub sensitivity: f32,
    /// Invert axis direction.
    pub invert: bool,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            binding: Binding::Button(0),
            sensitivity: 1.0,
            invert: false,
        }
    }
}

impl From<Binding> for ActionConfig {
    fn from(binding: Binding) -> Self {
        Self {
            binding,
            ..Default::default()
        }
    }
}

/// Maps abstract action names to physical input bindings.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionMap {
    bindings: HashMap<String, ActionConfig>,
}

impl ActionMap {
    /// Creates an empty action map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds an action name to a physical input with default config.
    pub fn bind(&mut self, action: impl Into<String>, binding: Binding) {
        self.bindings.insert(action.into(), binding.into());
    }

    /// Binds an action name to a physical input with custom config.
    pub fn bind_with_config(&mut self, action: impl Into<String>, config: ActionConfig) {
        self.bindings.insert(action.into(), config);
    }

    /// Sets the sensitivity multiplier for an action.
    pub fn set_sensitivity(&mut self, action: &str, sensitivity: f32) {
        if let Some(config) = self.bindings.get_mut(action) {
            config.sensitivity = sensitivity;
        }
    }

    /// Sets the invert flag for an action.
    pub fn set_invert(&mut self, action: &str, invert: bool) {
        if let Some(config) = self.bindings.get_mut(action) {
            config.invert = invert;
        }
    }

    /// Evaluates an input event against all bindings.
    /// Returns the names of actions that were triggered.
    pub fn evaluate(&self, event: &InputEvent) -> Vec<String> {
        let mut triggered = Vec::new();
        for (name, config) in &self.bindings {
            if Self::matches(&config.binding, event) {
                triggered.push(name.clone());
            }
        }
        triggered
    }

    /// Evaluates an axis event and returns the adjusted value (sensitivity + invert applied).
    pub fn evaluate_axis(&self, action: &str, raw_value: f32) -> Option<f32> {
        let config = self.bindings.get(action)?;
        match config.binding {
            Binding::Axis(_) | Binding::AxisRange(_, _, _) => {
                let mut value = raw_value * config.sensitivity;
                if config.invert {
                    value = -value;
                }
                Some(value.clamp(-1.0, 1.0))
            }
            _ => None,
        }
    }

    /// Checks if a binding matches an event.
    fn matches(binding: &Binding, event: &InputEvent) -> bool {
        match (binding, event) {
            (Binding::Button(btn), InputEvent::GamepadButton { button, pressure, .. }) => {
                *btn == *button && *pressure > 0.0
            }
            (Binding::Axis(axis), InputEvent::GamepadAxis { axis: a, .. }) => axis == a,
            (
                Binding::AxisRange(axis, min, max),
                InputEvent::GamepadAxis { axis: a, value, .. },
            ) => axis == a && *value >= *min && *value <= *max,
            (Binding::Chord(subs), event) => subs.iter().all(|s| Self::matches(s, event)),
            _ => false,
        }
    }
}

/// Deadzone math utilities.
pub mod deadzone {
    /// Applies a linear deadzone to a single axis value.
    ///
    /// Values within `threshold` of 0.0 are clamped to 0.0.
    /// Values at full deflection (±1.0) are preserved.
    pub fn apply(value: f32, threshold: f32) -> f32 {
        if value.abs() <= threshold {
            0.0
        } else {
            // Rescale so that threshold maps to 0.0 and 1.0 maps to 1.0
            let sign = value.signum();
            let magnitude = (value.abs() - threshold) / (1.0 - threshold);
            sign * magnitude.clamp(0.0, 1.0)
        }
    }

    /// Applies a radial deadzone to a 2D stick.
    ///
    /// If the stick magnitude is below `threshold`, returns (0.0, 0.0).
    /// Otherwise rescales so the output fills the full circle.
    pub fn radial(x: f32, y: f32, threshold: f32) -> (f32, f32) {
        // NaN passthrough
        if x.is_nan() || y.is_nan() {
            return (f32::NAN, f32::NAN);
        }
        let mag = (x * x + y * y).sqrt();
        if mag <= threshold || mag == 0.0 {
            (0.0, 0.0)
        } else {
            // Rescale: threshold maps to 0.0, 1.0 maps to 1.0
            // Ensure output never exceeds unit circle
            let new_mag = ((mag - threshold) / (1.0 - threshold)).min(1.0);
            let scale = new_mag / mag;
            (x * scale, y * scale)
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;

    #[test]
    fn test_serde_roundtrip_json() {
        let mut map = ActionMap::new();
        map.bind("jump", Binding::Button(0));
        map.bind("fire", Binding::Button(1));
        map.bind_with_config(
            "steer",
            ActionConfig {
                binding: Binding::Axis(0),
                sensitivity: 0.5,
                invert: true,
            },
        );

        let json = serde_json::to_string(&map).expect("serialize");
        let deserialized: ActionMap = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(map, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_toml() {
        let mut map = ActionMap::new();
        map.bind("pause", Binding::Button(9));

        let toml_str = toml::to_string(&map).expect("serialize to toml");
        let deserialized: ActionMap = toml::from_str(&toml_str).expect("deserialize from toml");

        assert_eq!(map, deserialized);
    }
}

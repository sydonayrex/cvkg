/// Touch state and gesture types.

use std::collections::HashMap;

/// A single touch point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchPoint {
    /// Unique touch point ID.
    pub id: u64,
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Pressure in [0.0, 1.0].
    pub pressure: f32,
}

/// Current touch state.
#[derive(Debug, Clone, Default)]
pub struct TouchState {
    /// Active touch points, keyed by ID.
    pub points: HashMap<u64, TouchPoint>,
}

impl TouchState {
    /// Creates a new empty touch state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of active touch points.
    pub fn active_count(&self) -> usize {
        self.points.len()
    }

    /// Returns a touch point by ID.
    pub fn get(&self, id: u64) -> Option<&TouchPoint> {
        self.points.get(&id)
    }
}

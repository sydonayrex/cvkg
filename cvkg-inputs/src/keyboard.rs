/// Keyboard state and key types.

use std::collections::HashSet;

/// A keyboard key identifier (string-based for flexibility).
pub type Key = String;

/// Current keyboard state.
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    /// Set of currently pressed keys.
    pub pressed: HashSet<Key>,
}

impl KeyboardState {
    /// Creates a new empty keyboard state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the given key is currently pressed.
    pub fn is_pressed(&self, key: &str) -> bool {
        self.pressed.contains(key)
    }

    /// Marks a key as pressed.
    pub fn press(&mut self, key: impl Into<Key>) {
        self.pressed.insert(key.into());
    }

    /// Marks a key as released.
    pub fn release(&mut self, key: &str) {
        self.pressed.remove(key);
    }
}

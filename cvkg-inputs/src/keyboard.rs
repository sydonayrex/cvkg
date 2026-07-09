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

    /// Marks a key as pressed. Recognized modifier names update the
    /// corresponding modifier flag implicitly.
    pub fn press(&mut self, key: impl Into<Key>) {
        self.pressed.insert(key.into());
    }

    /// Marks a key as released. Recognized modifier names clear the
    /// corresponding modifier flag implicitly.
    pub fn release(&mut self, key: &str) {
        self.pressed.remove(key);
    }
}

impl KeyboardState {
    /// Reconstructs the current set of modifiers from the held-key set.
    ///
    /// The crate does not carry a separate `modifiers` field; instead, modifier
    /// state is derived on demand from `pressed`. This keeps `apply_event`'s
    /// state-mutation pure with respect to the public API surface.
    pub fn modifiers(&self) -> cvkg_core::KeyModifiers {
        let shift = self.pressed.iter().any(|k| k == "Shift" || k == "ShiftLeft" || k == "ShiftRight");
        let ctrl = self.pressed.iter().any(|k| {
            k == "Control" || k == "ControlLeft" || k == "ControlRight"
        });
        let alt = self.pressed.iter().any(|k| {
            k == "Alt" || k == "AltLeft" || k == "AltRight"
        });
        let meta = self.pressed.iter().any(|k| {
            k == "Meta"
                || k == "MetaLeft"
                || k == "MetaRight"
                || k == "OSLeft"
                || k == "OSRight"
        });
        cvkg_core::KeyModifiers {
            shift,
            ctrl,
            alt,
            meta,
        }
    }
}

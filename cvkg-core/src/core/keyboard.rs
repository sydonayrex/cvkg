// =============================================================================
// KEYBOARD NAVIGATION
// =============================================================================

/// Modifier keys for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A keyboard shortcut binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShortcut {
    pub key: String,
    pub modifiers: KeyModifiers,
    pub description: String,
}

impl KeyShortcut {
    pub fn new(key: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: KeyModifiers::default(),
            description: desc.into(),
        }
    }
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers.ctrl = true;
        self
    }
    pub fn with_shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }
    pub fn with_alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }
    pub fn with_meta(mut self) -> Self {
        self.modifiers.meta = true;
        self
    }
}


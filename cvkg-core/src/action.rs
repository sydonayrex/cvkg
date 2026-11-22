//! Typed actions and context-aware key bindings.
//!
//! A key binding maps one `KeyShortcut` to one action inside a key context.
//! The action dispatcher resolves the most specific active context first.

use crate::{KeyContextStack, KeyShortcut};

/// A typed action name. The name is stable across native and web targets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionId(&'static str);

impl ActionId {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// A shortcut bound to an action inside an optional key context.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyBinding {
    pub shortcut: KeyShortcut,
    pub action: ActionId,
    pub context: Option<&'static str>,
}

impl KeyBinding {
    pub fn new(shortcut: KeyShortcut, action: ActionId, context: Option<&'static str>) -> Self {
        Self {
            shortcut,
            action,
            context,
        }
    }
}

/// A registry of bindings resolved against an active context stack.
pub struct ActionRegistry {
    bindings: Vec<KeyBinding>,
}

impl ActionRegistry {
    pub fn new(bindings: Vec<KeyBinding>) -> Self {
        Self { bindings }
    }

    pub fn resolve(&self, contexts: &KeyContextStack, shortcut: &KeyShortcut) -> Option<ActionId> {
        for context in contexts.stack.iter().rev() {
            if let Some(binding) = self
                .bindings
                .iter()
                .find(|binding| binding.context == Some(*context) && binding.shortcut == *shortcut)
            {
                return Some(binding.action);
            }
        }
        self.bindings
            .iter()
            .find(|binding| binding.context.is_none() && binding.shortcut == *shortcut)
            .map(|binding| binding.action)
    }
}

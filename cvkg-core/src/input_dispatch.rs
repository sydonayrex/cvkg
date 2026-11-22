//! Shared keyboard dispatch for native and web input paths.
//!
//! The dispatcher resolves a typed action from the active key context and
//! then offers the event to the focused view handler.

use crate::{ActionId, ActionRegistry, KeyContextStack, KeyModifiers};

/// Outcome of one keyboard dispatch.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyDispatch {
    pub action: Option<ActionId>,
    pub consumed: bool,
}

/// Resolve an action and call the focused handler for one keydown event.
pub fn dispatch_key_event(
    registry: &ActionRegistry,
    contexts: &KeyContextStack,
    key: &str,
    modifiers: KeyModifiers,
    mut focused_handler: impl FnMut(&str, KeyModifiers) -> bool,
) -> KeyDispatch {
    let action = registry.resolve(contexts, key, modifiers);
    let handled = focused_handler(key, modifiers);
    KeyDispatch {
        action,
        consumed: action.is_some() || handled,
    }
}

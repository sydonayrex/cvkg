use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// =============================================================================
// FOCUS MANAGEMENT
// =============================================================================

/// Unique ID for a focusable element.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FocusableId(String);

impl FocusableId {
    /// Returns the inner string representation of the focusable ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FocusableId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl From<String> for FocusableId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Focus trap for confining Tab navigation (e.g., modals).
#[derive(Debug, Clone)]
pub struct FocusTrap {
    pub id: FocusableId,
    pub order: Vec<FocusableId>,
    pub wrap: bool,
}

impl FocusTrap {
    pub fn new(id: impl Into<FocusableId>, order: Vec<FocusableId>) -> Self {
        Self {
            id: id.into(),
            order,
            wrap: true,
        }
    }
}

/// Manages focus order, Tab/Shift+Tab navigation, and focus traps.
#[derive(Debug, Default)]
pub struct FocusManager {
    order: Vec<FocusableId>,
    order_set: HashSet<FocusableId>,
    focused: Option<FocusableId>,
    traps: Vec<FocusTrap>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: impl Into<FocusableId>) {
        let id = id.into();
        if self.order_set.insert(id.clone()) {
            self.order.push(id);
        }
    }

    pub fn unregister(&mut self, id: &FocusableId) {
        if self.order_set.remove(id) {
            self.order.retain(|x| x != id);
        }
        if self.focused.as_ref() == Some(id) {
            self.focused = None;
        }
    }

    pub fn focused(&self) -> Option<&FocusableId> {
        self.focused.as_ref()
    }

    pub fn focus(&mut self, id: impl Into<FocusableId>) -> bool {
        let id = id.into();
        if self.order.contains(&id) || self.traps.iter().any(|t| t.order.contains(&id)) {
            self.focused = Some(id);
            true
        } else {
            false
        }
    }

    pub fn focus_next(&mut self) -> Option<&FocusableId> {
        let order = self.effective_order();
        if order.is_empty() {
            return None;
        }
        let idx = self
            .focused
            .as_ref()
            .and_then(|f| order.iter().position(|x| x == f));
        let next = match idx {
            Some(i) if i + 1 < order.len() => &order[i + 1],
            _ => &order[0],
        };
        self.focused = Some(next.clone());
        self.focused.as_ref()
    }

    pub fn focus_prev(&mut self) -> Option<&FocusableId> {
        let order = self.effective_order();
        if order.is_empty() {
            return None;
        }
        let idx = self
            .focused
            .as_ref()
            .and_then(|f| order.iter().position(|x| x == f));
        let prev = match idx {
            Some(i) if i > 0 => &order[i - 1],
            _ => &order[order.len() - 1],
        };
        self.focused = Some(prev.clone());
        self.focused.as_ref()
    }

    pub fn push_trap(&mut self, trap: FocusTrap) -> FocusableId {
        let id = trap.id.clone();
        self.traps.push(trap);
        id
    }

    pub fn pop_trap(&mut self) {
        self.traps.pop();
    }
    pub fn trap_count(&self) -> usize {
        self.traps.len()
    }

    fn effective_order(&self) -> &[FocusableId] {
        self.traps
            .last()
            .map(|t| t.order.as_slice())
            .unwrap_or(&self.order)
    }
}

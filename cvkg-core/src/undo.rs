//! Undo/redo manager.
//!
//! Extracted from lib.rs (P1-13).

use std::sync::Arc;

/// A single action group representing an undo/redo step.
pub struct UndoGroup {
    /// Descriptive label of the action (e.g. "Type", "Delete").
    pub label: String,
    /// Time when the action was recorded, in seconds.
    pub timestamp: f32,
    /// Closure to revert the action.
    pub undo: Arc<dyn Fn() + Send + Sync>,
    /// Closure to re-apply the action.
    pub redo: Arc<dyn Fn() + Send + Sync>,
}

impl Clone for UndoGroup {
    /// Clone the undo/redo group. The closures are shared via Arc.
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            timestamp: self.timestamp,
            undo: Arc::clone(&self.undo),
            redo: Arc::clone(&self.redo),
        }
    }
}

impl std::fmt::Debug for UndoGroup {
    /// Debug format helper to avoid printing closures.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoGroup")
            .field("label", &self.label)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

/// Unified manager for undo and redo stacks.
/// Supports grouping of actions, max undo depth clamping, and coalescing.
pub struct UndoManager {
    /// History stack of undo/redo groups.
    pub(crate) stack: Vec<UndoGroup>,
    /// Current position/index in the stack.
    pub(crate) position: usize,
    /// Maximum allowed undo steps before discarding oldest.
    max_depth: usize,
    /// Time window in seconds to coalesce consecutive actions of the same type.
    coalesce_window: f32,
}

impl Default for UndoManager {
    /// Create a default UndoManager with a depth of 100 and a 0.5s coalesce window.
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            position: 0,
            max_depth: 100,
            coalesce_window: 0.5,
        }
    }
}

impl Clone for UndoManager {
    /// Clone the undo manager, preserving stacks and position.
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            position: self.position,
            max_depth: self.max_depth,
            coalesce_window: self.coalesce_window,
        }
    }
}

impl std::fmt::Debug for UndoManager {
    /// Debug format helper for UndoManager.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoManager")
            .field("stack_len", &self.stack.len())
            .field("position", &self.position)
            .field("max_depth", &self.max_depth)
            .field("coalesce_window", &self.coalesce_window)
            .finish()
    }
}

impl UndoManager {
    /// Create a new UndoManager with custom settings.
    pub fn new(max_depth: usize, coalesce_window: f32) -> Self {
        Self {
            stack: Vec::new(),
            position: 0,
            max_depth,
            coalesce_window,
        }
    }

    /// Push a new undo/redo group to the stack, clearing any forward redo history.
    pub fn push(
        &mut self,
        label: &str,
        undo: impl Fn() + Send + Sync + 'static,
        redo: impl Fn() + Send + Sync + 'static,
    ) {
        if self.position < self.stack.len() {
            self.stack.truncate(self.position);
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f32();

        self.stack.push(UndoGroup {
            label: label.to_string(),
            timestamp,
            undo: Arc::new(undo),
            redo: Arc::new(redo),
        });

        if self.stack.len() > self.max_depth {
            self.stack.remove(0);
        }
        self.position = self.stack.len();
    }

    /// Perform the undo action if possible, moving the position back.
    /// Returns the undo closure to be executed outside of any state lock.
    pub fn undo(&mut self) -> Option<Arc<dyn Fn() + Send + Sync>> {
        if self.can_undo() {
            self.position -= 1;
            Some(Arc::clone(&self.stack[self.position].undo))
        } else {
            None
        }
    }

    /// Perform the redo action if possible, moving the position forward.
    /// Returns the redo closure to be executed outside of any state lock.
    pub fn redo(&mut self) -> Option<Arc<dyn Fn() + Send + Sync>> {
        if self.can_redo() {
            let group = &self.stack[self.position];
            self.position += 1;
            Some(Arc::clone(&group.redo))
        } else {
            None
        }
    }

    /// Returns true if there is an action that can be undone.
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Returns true if there is an action that can be redone.
    pub fn can_redo(&self) -> bool {
        self.position < self.stack.len()
    }

    /// Clear all undo/redo history.
    pub fn clear(&mut self) {
        self.stack.clear();
        self.position = 0;
    }

    /// Push a new coalesceable action. If the last action in the stack matches the label,
    /// is within the coalesce window, and the position is at the end of the stack, their undo/redo
    /// functions will be combined instead of creating a new group.
    pub fn push_coalesceable(
        &mut self,
        label: &str,
        undo: impl Fn() + Send + Sync + 'static,
        redo: impl Fn() + Send + Sync + 'static,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f32();

        if self.position == self.stack.len() && !self.stack.is_empty() {
            let last_idx = self.stack.len() - 1;
            let last = &self.stack[last_idx];
            if last.label == label && (now - last.timestamp).abs() <= self.coalesce_window {
                let old_undo = Arc::clone(&last.undo);
                let old_redo = Arc::clone(&last.redo);
                let new_undo = Arc::new(undo);
                let new_redo = Arc::new(redo);

                self.stack[last_idx].undo = Arc::new(move || {
                    new_undo();
                    old_undo();
                });
                self.stack[last_idx].redo = Arc::new(move || {
                    old_redo();
                    new_redo();
                });
                self.stack[last_idx].timestamp = now;
                return;
            }
        }

        self.push(label, undo, redo);
    }
}

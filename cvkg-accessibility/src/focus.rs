//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! Focus management — keyboard tab order and directional navigation.
//!
//! # Why this exists
//! Correct keyboard navigation is required for WCAG 2.1 Level AA compliance.
//! The `FocusManager` owns the canonical tab order and implements forward/
//! backward cycling so that the platform event handler never needs to
//! know about the internal tree structure.

use cvkg_core::KvasirId;
use serde::{Deserialize, Serialize};

/// Direction of programmatic focus movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusDirection {
    /// Move to the next focusable node in tab order (Tab key).
    Forward,
    /// Move to the previous focusable node in tab order (Shift+Tab).
    Backward,
    /// Jump directly to the first focusable node.
    First,
    /// Jump directly to the last focusable node.
    Last,
}

/// The outcome of a focus-move operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusResult {
    /// Focus successfully moved to the given node.
    Focused(KvasirId),
    /// No focusable nodes are registered — nothing to focus.
    NoFocusableNode,
    /// Already at the boundary (first/last) and wrapping is disabled.
    AlreadyAtBoundary,
}

/// Keyboard navigation model.
///
/// `FocusManager` owns the ordered list of focusable node IDs (the tab order)
/// and tracks which node currently holds focus. It deliberately does NOT hold
/// a reference to the `AccessibilityTree` — the tree is the source of truth
/// for node metadata, but focus traversal only needs the ordered ID list.
///
/// The manager wraps at boundaries: pressing Tab on the last node moves focus
/// to the first node, and Shift+Tab on the first node moves to the last. This
/// matches standard browser and desktop AT behavior.
pub struct FocusManager {
    /// ID of the currently focused node, or `None` if nothing is focused.
    focused: Option<KvasirId>,
    /// Ordered list of focusable node IDs (the canonical tab order).
    tab_order: Vec<KvasirId>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new `FocusManager` with no focused node and an empty tab order.
    pub fn new() -> Self {
        Self {
            focused: None,
            tab_order: Vec::new(),
        }
    }

    /// Replace the tab order with a new ordered list of focusable node IDs.
    ///
    /// The order determines Tab/Shift+Tab cycling. Callers should call this
    /// whenever `AccessibilityTree::version()` changes.
    pub fn set_tab_order(&mut self, order: Vec<KvasirId>) {
        // Validate that the currently focused node still exists in the new order.
        // If it was removed, clear focus to avoid a dangling focus state.
        if let Some(focused) = self.focused {
            if !order.contains(&focused) {
                self.focused = None;
            }
        }
        self.tab_order = order;
    }

    /// Return the ID of the currently focused node, if any.
    pub fn focused(&self) -> Option<KvasirId> {
        self.focused
    }

    /// Move focus in the given direction, returning the result.
    ///
    /// `Forward` and `Backward` wrap at the list boundaries.
    /// `First` and `Last` unconditionally jump to the respective ends.
    ///
    /// # Contract
    /// Returns `NoFocusableNode` if `tab_order` is empty.
    /// Returns `Focused(id)` and updates `self.focused` on success.
    pub fn move_focus(&mut self, direction: FocusDirection) -> FocusResult {
        if self.tab_order.is_empty() {
            return FocusResult::NoFocusableNode;
        }

        let new_id = match direction {
            FocusDirection::First => self.tab_order[0],
            FocusDirection::Last => *self.tab_order.last().unwrap(),
            FocusDirection::Forward => {
                let idx = self
                    .focused
                    .and_then(|f| self.tab_order.iter().position(|&id| id == f))
                    .map(|i| (i + 1) % self.tab_order.len())
                    .unwrap_or(0);
                self.tab_order[idx]
            }
            FocusDirection::Backward => {
                let len = self.tab_order.len();
                let idx = self
                    .focused
                    .and_then(|f| self.tab_order.iter().position(|&id| id == f))
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(len - 1);
                self.tab_order[idx]
            }
        };

        self.focused = Some(new_id);
        FocusResult::Focused(new_id)
    }

    /// Directly focus a specific node by ID.
    ///
    /// Returns `NoFocusableNode` if the given ID is not in the current tab
    /// order (i.e., the node is not focusable or doesn't exist in the tree).
    pub fn focus(&mut self, id: KvasirId) -> FocusResult {
        if self.tab_order.contains(&id) {
            self.focused = Some(id);
            FocusResult::Focused(id)
        } else {
            FocusResult::NoFocusableNode
        }
    }

    /// Clear focus without moving to another node.
    ///
    /// Use when a dialog closes, the window loses activation, or the user
    /// clicks into a non-focusable area.
    pub fn blur(&mut self) {
        self.focused = None;
    }

    /// Rebuild the tab order from the focusable nodes in an `AccessibilityTree`.
    ///
    /// The order is derived from `tree.focusable_nodes()`. Because `HashMap`
    /// iteration order is arbitrary, the IDs are sorted by their raw `u64`
    /// value to produce a stable, deterministic tab order. In a real
    /// production integration the caller would impose a document-order sort
    /// based on visual position or DOM tree depth instead.
    pub fn rebuild_from_tree(&mut self, tree: &super::tree::AccessibilityTree) {
        let mut ids: Vec<KvasirId> = tree.focusable_nodes().iter().map(|n| n.id).collect();
        // Sort by raw id value for a stable, deterministic order.
        // Real document-order sorting would use layout position (x, y).
        ids.sort_by_key(|id| id.0);
        self.set_tab_order(ids);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{AccessNode, AccessibilityTree, SemanticRole};
    use cvkg_core::{KvasirId, layout::Rect};

    fn make_rect() -> Rect {
        Rect::new(0.0, 0.0, 100.0, 30.0)
    }

    /// Build a tree with `count` Button nodes, returning their IDs sorted ascending.
    fn make_tree_with_buttons(count: u8) -> (AccessibilityTree, Vec<KvasirId>) {
        let mut tree = AccessibilityTree::new();
        let mut ids = Vec::new();
        for _ in 0..count {
            let id = KvasirId::new();
            tree.insert(AccessNode::new(id, SemanticRole::Button, make_rect()));
            ids.push(id);
        }
        ids.sort_by_key(|id| id.0);
        (tree, ids)
    }

    #[test]
    fn new_manager_has_no_focus() {
        let fm = FocusManager::new();
        assert!(fm.focused().is_none());
    }

    #[test]
    fn move_focus_forward_cycles() {
        let (tree, ids) = make_tree_with_buttons(3);
        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);

        // Forward from nothing → first
        let r = fm.move_focus(FocusDirection::Forward);
        assert_eq!(r, FocusResult::Focused(ids[0]));

        // Forward → second
        let r = fm.move_focus(FocusDirection::Forward);
        assert_eq!(r, FocusResult::Focused(ids[1]));

        // Forward → third
        let r = fm.move_focus(FocusDirection::Forward);
        assert_eq!(r, FocusResult::Focused(ids[2]));

        // Forward from last → wraps to first
        let r = fm.move_focus(FocusDirection::Forward);
        assert_eq!(r, FocusResult::Focused(ids[0]));
    }

    #[test]
    fn move_focus_backward_cycles() {
        let (tree, ids) = make_tree_with_buttons(3);
        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);

        // Backward from nothing → last
        let r = fm.move_focus(FocusDirection::Backward);
        assert_eq!(r, FocusResult::Focused(ids[2]));

        // Backward → second
        let r = fm.move_focus(FocusDirection::Backward);
        assert_eq!(r, FocusResult::Focused(ids[1]));

        // Backward → first
        let r = fm.move_focus(FocusDirection::Backward);
        assert_eq!(r, FocusResult::Focused(ids[0]));

        // Backward from first → wraps to last
        let r = fm.move_focus(FocusDirection::Backward);
        assert_eq!(r, FocusResult::Focused(ids[2]));
    }

    #[test]
    fn move_focus_first_and_last() {
        let (tree, ids) = make_tree_with_buttons(4);
        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);

        assert_eq!(fm.move_focus(FocusDirection::First), FocusResult::Focused(ids[0]));
        assert_eq!(fm.move_focus(FocusDirection::Last), FocusResult::Focused(ids[3]));
    }

    #[test]
    fn move_focus_empty_returns_no_focusable() {
        let mut fm = FocusManager::new();
        assert_eq!(fm.move_focus(FocusDirection::Forward), FocusResult::NoFocusableNode);
    }

    #[test]
    fn focus_direct_sets_focused() {
        let (tree, ids) = make_tree_with_buttons(3);
        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);

        let r = fm.focus(ids[1]);
        assert_eq!(r, FocusResult::Focused(ids[1]));
        assert_eq!(fm.focused(), Some(ids[1]));
    }

    #[test]
    fn focus_unknown_id_returns_no_focusable() {
        let mut fm = FocusManager::new();
        let unknown = KvasirId::new();
        let r = fm.focus(unknown);
        assert_eq!(r, FocusResult::NoFocusableNode);
    }

    #[test]
    fn blur_clears_focus() {
        let (tree, ids) = make_tree_with_buttons(2);
        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);
        fm.focus(ids[0]);
        assert!(fm.focused().is_some());
        fm.blur();
        assert!(fm.focused().is_none());
    }

    #[test]
    fn rebuild_from_tree_sets_tab_order() {
        let mut tree = AccessibilityTree::new();
        let btn_id = KvasirId::new();
        let txt_id = KvasirId::new();
        // Add a non-focusable node — it must be excluded from tab order
        let label_id = KvasirId::new();
        tree.insert(AccessNode::new(btn_id, SemanticRole::Button, make_rect()));
        tree.insert(AccessNode::new(txt_id, SemanticRole::TextInput, make_rect()));
        tree.insert(AccessNode::new(label_id, SemanticRole::Generic, make_rect()));

        let mut fm = FocusManager::new();
        fm.rebuild_from_tree(&tree);

        // 2 focusable nodes (button + textinput), not 3
        assert_eq!(fm.tab_order.len(), 2);
        assert!(!fm.tab_order.contains(&label_id));
    }

    #[test]
    fn set_tab_order_clears_stale_focus() {
        let id_a = KvasirId::new();
        let id_b = KvasirId::new();
        let mut fm = FocusManager::new();
        fm.set_tab_order(vec![id_a, id_b]);
        fm.focus(id_b);
        assert_eq!(fm.focused(), Some(id_b));

        // Remove id_b from the order — focus should be cleared
        fm.set_tab_order(vec![id_a]);
        assert!(fm.focused().is_none());
    }
}

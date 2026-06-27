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

//! Accessibility tree — unified node model and tree container.
//!
//! # Why this exists
//! cvkg-vdom's `A11yNodeEntry` was a display-only snapshot for the inspector.
//! `AccessibilityTree` is the **authoritative** platform-facing tree that drives
//! real AT (assistive technology) integration. It maps `KvasirId` → `AccessNode`
//! so that the VDOM, scene graph, and platform bridge all share a single source
//! of truth without cross-crate circular dependencies.

use cvkg_core::{KvasirId, layout::Rect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic roles understood by CVKG's accessibility layer.
///
/// This enum is richer than the string roles stored in `VNode::aria_role`
/// (which must interop with accesskit's stringly-typed API). Having a proper
/// enum here lets the compiler enforce exhaustiveness in match arms and
/// prevents typo-class bugs when constructing the tree programmatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRole {
    Button,
    Checkbox,
    ComboBox,
    Dialog,
    Form,
    Grid,
    Heading,
    Image,
    Link,
    List,
    ListItem,
    Menu,
    MenuItem,
    Navigation,
    ProgressBar,
    Radio,
    RadioGroup,
    Scrollbar,
    Separator,
    Slider,
    SpinButton,
    Switch,
    Tab,
    Table,
    TabList,
    TabPanel,
    TextInput,
    Toolbar,
    Tooltip,
    Tree,
    TreeItem,
    Application,
    Banner,
    Main,
    Region,
    /// Catch-all for roles that don't map to a specific semantic concept.
    Generic,
}

impl SemanticRole {
    /// Returns `true` if this role is normally keyboard-focusable.
    ///
    /// This mirrors `VNode::is_focusable` but operates on the typed enum so
    /// `FocusManager::rebuild_from_tree` can determine tab-stop eligibility
    /// without string matching.
    pub fn is_focusable(self) -> bool {
        matches!(
            self,
            SemanticRole::Button
                | SemanticRole::Checkbox
                | SemanticRole::ComboBox
                | SemanticRole::Radio
                | SemanticRole::Scrollbar
                | SemanticRole::Slider
                | SemanticRole::SpinButton
                | SemanticRole::Switch
                | SemanticRole::Tab
                | SemanticRole::TextInput
        )
    }
}

/// A single node in the platform accessibility tree.
///
/// `AccessNode` is a flattened, serializable representation of a UI element
/// intended to be handed to the platform AT bridge (e.g., AccessKit, macOS
/// NSAccessibility, MSAA). It is **separate** from `VNode` to avoid coupling
/// the render path to the a11y path and to allow the tree to outlive a
/// single VDOM diffing cycle.
///
/// # Contract
/// - `id` uniquely identifies this node within the tree (same `KvasirId` as
///   the corresponding `VNode`).
/// - Nodes whose `is_hidden` is `true` MUST NOT appear in focus navigation.
/// - `children` holds ordered child IDs; the tree is the authoritative
///   parent→child mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessNode {
    /// Globally unique identifier matching the corresponding VNode.
    pub id: KvasirId,
    /// Semantic role of this node.
    pub role: SemanticRole,
    /// Human-readable accessible label (screen reader narration).
    pub label: Option<String>,
    /// Extended description surfaced by screen readers on demand.
    pub description: Option<String>,
    /// Current value string (e.g. "65%" for a slider).
    pub value: Option<String>,
    /// Whether this node currently has input focus.
    pub is_focused: bool,
    /// Whether this node can accept user interaction.
    pub is_enabled: bool,
    /// Whether this node is hidden from assistive technologies (Section 508 omission).
    pub is_hidden: bool,
    /// Visual bounding rect in logical pixels (same coordinate space as the renderer).
    pub bounds: Rect,
    /// Ordered list of child node IDs.
    pub children: Vec<KvasirId>,
    /// Current numeric value (e.g. slider position, progress amount).
    pub numeric_value: Option<f32>,
    /// Minimum allowed numeric value.
    pub min_numeric_value: Option<f32>,
    /// Maximum allowed numeric value.
    pub max_numeric_value: Option<f32>,
}

impl AccessNode {
    /// Create a minimal `AccessNode` with sane defaults.
    ///
    /// All optional fields start as `None`; use the builder methods to
    /// populate them. `is_enabled` defaults to `true` and `is_hidden` to
    /// `false` because most nodes in the tree are visible and interactive.
    pub fn new(id: KvasirId, role: SemanticRole, bounds: Rect) -> Self {
        Self {
            id,
            role,
            label: None,
            description: None,
            value: None,
            is_focused: false,
            is_enabled: true,
            is_hidden: false,
            bounds,
            children: Vec::new(),
            numeric_value: None,
            min_numeric_value: None,
            max_numeric_value: None,
        }
    }

    /// Set the accessible label (builder pattern).
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the extended description (builder pattern).
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the current value string (builder pattern).
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Mark this node as hidden from assistive technologies (builder pattern).
    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.is_hidden = true;
        self
    }

    /// Mark this node as disabled / non-interactive (builder pattern).
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.is_enabled = false;
        self
    }
}

/// The authoritative platform-facing accessibility tree.
///
/// # Why a separate tree?
/// Merging the a11y tree with the VDOM would create a circular dependency
/// between cvkg-vdom and any AT integration layer. By keeping this tree
/// in its own crate, the VDOM can push patches upward and the bridge can
/// pull a snapshot downward without either knowing about the other.
///
/// # Version counter
/// Every structural mutation (insert, remove, set_root) increments `version`.
/// Callers can compare `version` across frames to detect whether a full
/// AT-tree sync is required, avoiding redundant platform round-trips.
pub struct AccessibilityTree {
    /// Flattened map of all live accessibility nodes.
    nodes: HashMap<KvasirId, AccessNode>,
    /// ID of the root node, if any.
    root: Option<KvasirId>,
    /// Monotonic counter incremented on every structural change.
    version: u64,
}

impl Default for AccessibilityTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityTree {
    /// Create an empty `AccessibilityTree`.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            version: 0,
        }
    }

    /// Insert or replace a node in the tree.
    ///
    /// If a node with the same `id` already exists it is replaced wholesale.
    /// The version counter is always incremented so callers can detect the
    /// change even when updating an existing node.
    pub fn insert(&mut self, node: AccessNode) {
        self.nodes.insert(node.id, node);
        self.version += 1;
    }

    /// Remove a node by ID, returning it if it existed.
    ///
    /// Does not recursively remove children — the caller is responsible for
    /// cleaning up orphaned child nodes to avoid dangling references.
    pub fn remove(&mut self, id: KvasirId) -> Option<AccessNode> {
        let removed = self.nodes.remove(&id);
        if removed.is_some() {
            // Clear root if the root node was removed.
            if self.root == Some(id) {
                self.root = None;
            }
            self.version += 1;
        }
        removed
    }

    /// Look up an immutable reference to a node by ID.
    pub fn get(&self, id: KvasirId) -> Option<&AccessNode> {
        self.nodes.get(&id)
    }

    /// Look up a mutable reference to a node by ID.
    ///
    /// NOTE: Callers that mutate a node via this reference should call
    /// `AccessibilityTree::bump_version` afterward if they want consumers to
    /// detect the change. (In-place field mutations do NOT auto-increment.)
    pub fn get_mut(&mut self, id: KvasirId) -> Option<&mut AccessNode> {
        self.nodes.get_mut(&id)
    }

    /// Manually increment the version counter.
    ///
    /// Use this after in-place node mutations via `get_mut` so that AT bridge
    /// consumers can detect the change through a version comparison.
    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Designate a node as the tree root.
    ///
    /// Setting the root increments the version counter because the AT bridge
    /// must re-announce the root on every structural change.
    pub fn set_root(&mut self, id: KvasirId) {
        if !self.nodes.contains_key(&id) {
            log::warn!(
                "AccessibilityTree::set_root: node {:?} does not exist in the tree",
                id
            );
            return;
        }
        self.root = Some(id);
        self.version += 1;
    }

    /// Return an immutable reference to the root node, if one is set.
    pub fn root(&self) -> Option<&AccessNode> {
        self.root.and_then(|id| self.nodes.get(&id))
    }

    /// Return the current version counter value.
    ///
    /// This value is monotonically increasing and reflects the number of
    /// structural mutations since the tree was created.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Return the total number of nodes currently in the tree.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Collect all nodes that are eligible for keyboard focus.
    ///
    /// A node is considered focusable when:
    /// - `is_enabled` is `true`
    /// - `is_hidden` is `false`
    /// - `role.is_focusable()` returns `true`
    ///
    /// The returned slice is **not ordered** (HashMap iteration order). Call
    /// `FocusManager::rebuild_from_tree` which applies the correct tab order.
    pub fn focusable_nodes(&self) -> Vec<&AccessNode> {
        self.nodes
            .values()
            .filter(|n| n.is_enabled && !n.is_hidden && n.role.is_focusable())
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::KvasirId;

    fn make_rect() -> Rect {
        Rect::new(0.0, 0.0, 100.0, 30.0)
    }

    #[test]
    fn insert_and_get() {
        let mut tree = AccessibilityTree::new();
        let id = KvasirId::new();
        let node = AccessNode::new(id, SemanticRole::Button, make_rect()).with_label("Click me");
        tree.insert(node);

        let found = tree.get(id).expect("node should exist");
        assert_eq!(found.label.as_deref(), Some("Click me"));
        assert_eq!(found.role, SemanticRole::Button);
    }

    #[test]
    fn version_increments_on_insert() {
        let mut tree = AccessibilityTree::new();
        assert_eq!(tree.version(), 0);

        let id = KvasirId::new();
        tree.insert(AccessNode::new(id, SemanticRole::Generic, make_rect()));
        assert_eq!(tree.version(), 1);

        tree.insert(AccessNode::new(id, SemanticRole::Generic, make_rect()));
        assert_eq!(tree.version(), 2);
    }

    #[test]
    fn version_increments_on_remove() {
        let mut tree = AccessibilityTree::new();
        let id = KvasirId::new();
        tree.insert(AccessNode::new(id, SemanticRole::Button, make_rect()));
        let v = tree.version();

        let removed = tree.remove(id);
        assert!(removed.is_some());
        assert_eq!(tree.version(), v + 1);
        assert_eq!(tree.node_count(), 0);
    }

    #[test]
    fn remove_nonexistent_does_not_increment() {
        let mut tree = AccessibilityTree::new();
        let v = tree.version();
        let removed = tree.remove(KvasirId::new());
        assert!(removed.is_none());
        assert_eq!(
            tree.version(),
            v,
            "version must not change on a no-op remove"
        );
    }

    #[test]
    fn set_root_and_root_accessor() {
        let mut tree = AccessibilityTree::new();
        let id = KvasirId::new();
        tree.insert(AccessNode::new(id, SemanticRole::Application, make_rect()));
        tree.set_root(id);

        let root = tree.root().expect("root should be set");
        assert_eq!(root.id, id);
    }

    #[test]
    fn remove_root_clears_root() {
        let mut tree = AccessibilityTree::new();
        let id = KvasirId::new();
        tree.insert(AccessNode::new(id, SemanticRole::Application, make_rect()));
        tree.set_root(id);
        tree.remove(id);
        assert!(tree.root().is_none());
    }

    #[test]
    fn focusable_nodes_filters_correctly() {
        let mut tree = AccessibilityTree::new();

        // Focusable: Button, enabled, visible
        let btn_id = KvasirId::new();
        tree.insert(AccessNode::new(btn_id, SemanticRole::Button, make_rect()));

        // Non-focusable: Generic role
        let generic_id = KvasirId::new();
        tree.insert(AccessNode::new(
            generic_id,
            SemanticRole::Generic,
            make_rect(),
        ));

        // Hidden button — must be excluded
        let hidden_id = KvasirId::new();
        tree.insert(AccessNode::new(hidden_id, SemanticRole::Button, make_rect()).hidden());

        // Disabled button — must be excluded
        let disabled_id = KvasirId::new();
        tree.insert(AccessNode::new(disabled_id, SemanticRole::Button, make_rect()).disabled());

        let focusable = tree.focusable_nodes();
        assert_eq!(focusable.len(), 1);
        assert_eq!(focusable[0].id, btn_id);
    }

    #[test]
    fn builder_methods() {
        let id = KvasirId::new();
        let node = AccessNode::new(id, SemanticRole::Slider, make_rect())
            .with_label("Volume")
            .with_description("Master volume control")
            .with_value("75%");

        assert_eq!(node.label.as_deref(), Some("Volume"));
        assert_eq!(node.description.as_deref(), Some("Master volume control"));
        assert_eq!(node.value.as_deref(), Some("75%"));
        assert!(node.is_enabled);
        assert!(!node.is_hidden);
    }

    #[test]
    fn semantic_role_is_focusable() {
        assert!(SemanticRole::Button.is_focusable());
        assert!(SemanticRole::TextInput.is_focusable());
        assert!(SemanticRole::Slider.is_focusable());
        assert!(!SemanticRole::Generic.is_focusable());
        assert!(!SemanticRole::Dialog.is_focusable());
        assert!(!SemanticRole::Navigation.is_focusable());
    }
}

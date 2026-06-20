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

//! Virtual DOM implementation for CVKG

pub mod animated;
pub mod physics;
pub mod signals;
use cvkg_core::Renderer;
pub use cvkg_core::KvasirId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A map of event names to their corresponding thread-safe handlers.
pub type EventHandlerMap = HashMap<String, Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>;

/// A map of node IDs to their respective event handler maps.
pub type NodeEventHandlerMap = HashMap<NodeId, EventHandlerMap>;

/// A unique identifier for a node within the Virtual DOM tree.
///
/// # Crosscrate identity (crosscrate.md Finding #2)
///
/// Type alias for [`cvkg_core::KvasirId`] so that VDOM nodes, scene nodes,
/// and flow nodes share the same identity type. Allocates via `KvasirId::new()`.
pub type NodeId = KvasirId;

/// Represents the computed layout bounds of a component in the Virtual DOM.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LayoutRect {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Width of the bounds
    pub width: f32,
    /// Height of the bounds
    pub height: f32,
}

/// A single node in the accessibility tree, extracted from the VDOM.
///
/// Used by `A11yInspector` to display the real accessibility tree structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNodeEntry {
    /// ARIA role (e.g., "button", "group", "slider")
    pub role: String,
    /// Accessible label for the node
    pub label: String,
    /// Current value display (e.g., "65%" for sliders)
    pub value: Option<String>,
    /// Whether the node is currently focused
    pub focused: bool,
    /// Whether the node is enabled
    pub enabled: bool,
    /// Tree depth for indentation
    pub depth: u32,
}

/// Accessibility ARIA properties for the DOM shadow tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AriaProps {
    /// Screen reader accessible label (audio narration)
    pub label: Option<String>,
    /// Screen reader extended description (audio narration context)
    pub description: Option<String>,
    /// Value for input fields (screen reader readout)
    pub value: Option<String>,
    /// Whether the element is disabled
    pub disabled: bool,
    /// Whether the element is hidden from screen readers (Section 508 omission)
    pub hidden: bool,
    /// ARIA aria-valuenow for slider/progress/meter roles
    pub aria_valuenow: Option<f32>,
    /// ARIA aria-valuemin for slider/progress/meter roles
    pub aria_valuemin: Option<f32>,
    /// ARIA aria-valuemax for slider/progress/meter roles
    pub aria_valuemax: Option<f32>,
}

/// A node in the Virtual DOM tree representing a component instance.
#[derive(Clone, Serialize, Deserialize)]
pub struct VNode {
    /// Unique identifier for this node instance
    pub id: NodeId,
    /// Optional key for keyed list diffing
    pub key: Option<String>,
    /// String representation of the CVKG component type (e.g. "Text", "Button")
    pub component_type: String,
    /// Serialized view properties
    pub props: HashMap<String, serde_json::Value>,
    /// Serialized internal state, captured for Inspector debugging
    pub state: Option<HashMap<String, serde_json::Value>>,
    /// The computed layout bounds of this node
    pub layout: LayoutRect,
    /// Node IDs of the children
    pub children: Vec<NodeId>,
    /// Standard ARIA role string (e.g. "button", "group")
    pub aria_role: String,
    /// Standard ARIA properties
    pub aria_props: AriaProps,
    /// Optional portal target. If set, this node's children render into the target ID.
    pub portal_target: Option<NodeId>,
    /// Vili SDF Shape for precise hit testing
    pub sdf_shape: Option<cvkg_core::layout::SdfShape>,
}

impl PartialEq for VNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.key == other.key
            && self.component_type == other.component_type
            && self.props == other.props
            && self.state == other.state
            && self.layout == other.layout
            && self.children == other.children
            && self.aria_role == other.aria_role
            && self.aria_props == other.aria_props
            && self.sdf_shape == other.sdf_shape
    }
}

impl std::fmt::Debug for VNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VNode")
            .field("id", &self.id)
            .field("key", &self.key)
            .field("component_type", &self.component_type)
            .field("props", &self.props)
            .field("state", &self.state)
            .field("layout", &self.layout)
            .field("children", &self.children)
            .field("aria_role", &self.aria_role)
            .field("aria_props", &self.aria_props)
            .field("sdf_shape", &self.sdf_shape)
            .finish_non_exhaustive()
    }
}

impl VNode {
    /// Convert this VNode to an AccessKit node for accessibility tree generation.
    pub fn to_accesskit_node(&self) -> accesskit::Node {
        let mut node = accesskit::Node::new(match self.aria_role.as_str() {
            // All 53 AriaRole variants mapped to AccessKit Role equivalents
            "alert" => accesskit::Role::Alert,
            "alertdialog" => accesskit::Role::AlertDialog,
            "article" => accesskit::Role::Article,
            "banner" => accesskit::Role::Banner,
            "button" => accesskit::Role::Button,
            "checkbox" => accesskit::Role::CheckBox,
            "columnheader" => accesskit::Role::ColumnHeader,
            "combobox" => accesskit::Role::ComboBox,
            "complementary" => accesskit::Role::Complementary,
            "contentinfo" => accesskit::Role::ContentInfo,
            "dialog" => accesskit::Role::Dialog,
            "form" => accesskit::Role::Form,
            "grid" => accesskit::Role::Grid,
            "gridcell" => accesskit::Role::GridCell,
            "heading" => accesskit::Role::Heading,
            "img" => accesskit::Role::Image,
            "link" => accesskit::Role::Link,
            "list" => accesskit::Role::List,
            "listbox" => accesskit::Role::ListBox,
            "listitem" => accesskit::Role::ListItem,
            "main" => accesskit::Role::Main,
            "menu" => accesskit::Role::Menu,
            "menubar" => accesskit::Role::MenuBar,
            "menuitem" => accesskit::Role::MenuItem,
            "menuitemcheckbox" => accesskit::Role::MenuItemCheckBox,
            "menuitemradio" => accesskit::Role::MenuItemRadio,
            "navigation" => accesskit::Role::Navigation,
            "none" => accesskit::Role::GenericContainer,
            "note" => accesskit::Role::Note,
            "option" => accesskit::Role::ListBoxOption,
            "presentation" => accesskit::Role::GenericContainer,
            "progressbar" => accesskit::Role::ProgressIndicator,
            "radio" => accesskit::Role::RadioButton,
            "radiogroup" => accesskit::Role::RadioGroup,
            "region" => accesskit::Role::Region,
            "row" => accesskit::Role::Row,
            "rowgroup" => accesskit::Role::RowGroup,
            "rowheader" => accesskit::Role::RowHeader,
            "search" => accesskit::Role::Search,
            "separator" => accesskit::Role::Splitter,
            "slider" => accesskit::Role::Slider,
            "spinbutton" => accesskit::Role::SpinButton,
            "status" => accesskit::Role::Status,
            "switch" => accesskit::Role::Switch,
            "tab" => accesskit::Role::Tab,
            "table" => accesskit::Role::Table,
            "tablist" => accesskit::Role::TabList,
            "tabpanel" => accesskit::Role::TabPanel,
            "textbox" => accesskit::Role::TextInput,
            "toolbar" => accesskit::Role::Toolbar,
            "tooltip" => accesskit::Role::Tooltip,
            "tree" => accesskit::Role::Tree,
            "treeitem" => accesskit::Role::TreeItem,

            // Non-ARIA utility roles used by the codebase
            "text" => accesskit::Role::Label,
            "group" => accesskit::Role::Group,
            "window" => accesskit::Role::Window,
            "password" => accesskit::Role::TextInput,
            "application" => accesskit::Role::Application,
            "colorwell" => accesskit::Role::ColorWell,

            _ => accesskit::Role::Unknown,
        });

        if self.aria_role == "password" {
            // Note: In some accesskit versions, you might need a specific property for password.
            // For now, we rely on the role mapping or a custom property if available.
        }

        if let Some(label) = &self.aria_props.label {
            node.set_label(label.clone());
        }

        if let Some(desc) = &self.aria_props.description {
            node.set_description(desc.clone());
        }

        if let Some(val) = &self.aria_props.value {
            node.set_value(val.clone());
        }

        // Expose ARIA slider/progress/meter numeric values to accesskit
        if let Some(now) = self.aria_props.aria_valuenow {
            node.set_numeric_value(now as f64);
        }
        if let Some(min) = self.aria_props.aria_valuemin {
            node.set_min_numeric_value(min as f64);
        }
        if let Some(max) = self.aria_props.aria_valuemax {
            node.set_max_numeric_value(max as f64);
        }

        if self.aria_props.disabled {
            node.set_disabled();
        }

        if self.aria_props.hidden {
            node.set_hidden();
        }

        node.set_bounds(accesskit::Rect {
            x0: self.layout.x as f64,
            y0: self.layout.y as f64,
            x1: (self.layout.x + self.layout.width) as f64,
            y1: (self.layout.y + self.layout.height) as f64,
        });

        node.set_children(
            self.children
                .iter()
                .map(|id| accesskit::NodeId(id.0))
                .collect::<Vec<_>>(),
        );

        node
    }

    /// Returns true if this node can receive keyboard focus.
    /// Focusable roles include: button, checkbox, radio, slider, tab, spinbutton, combobox, textbox, password, switch, scrollbar.
    pub fn is_focusable(&self) -> bool {
        matches!(
            self.aria_role.as_str(),
            "button" | "checkbox" | "radio" | "slider" | "tab" |
            "spinbutton" | "combobox" | "textbox" | "password" | "switch" | "scrollbar"
        )
    }
}

/// A discrete mutation to the Virtual DOM tree.
#[derive(Clone)]
pub enum VDomPatch {
    /// Create and append a new node
    Create(VNode),
    /// Update properties of an existing node
    Update {
        /// ID of the node to update
        id: NodeId,
        /// Updated properties map
        props: Option<HashMap<String, serde_json::Value>>,
        /// Updated layout
        layout: Option<LayoutRect>,
        /// Updated ARIA properties
        aria_props: Option<AriaProps>,
        /// Updated ARIA role
        aria_role: Option<String>,
        /// Updated children list
        children: Option<Vec<NodeId>>,
        /// Updated event handlers
        handlers: Option<EventHandlerMap>,
        /// Updated SDF shape
        sdf_shape: Option<cvkg_core::layout::SdfShape>,
    },
    /// Remove an existing node
    Remove(NodeId),
    /// Replace an existing node completely with a new one
    Replace {
        /// ID of the node being replaced
        id: NodeId,
        /// The new node to substitute
        node: VNode,
    },
    /// Move a keyed node to a new position within its parent
    Move {
        /// ID of the node being moved
        id: NodeId,
        /// The new index position
        new_index: usize,
    },
    /// Update the root node ID
    SetRoot(Option<NodeId>),
    /// Clear all event handlers attached to a node.
    ///
    /// Without this variant, `Update { handlers: None }` cannot remove
    /// handlers -- the apply step interprets `None` as "leave handlers
    /// unchanged". Emitted when the new tree has no handlers for a node
    /// but the old tree did.
    ClearHandlers {
        /// ID of the node whose handlers should be cleared.
        id: NodeId,
    },
}

impl std::fmt::Debug for VDomPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Create(node) => f.debug_tuple("Create").field(node).finish(),
            Self::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => f
                .debug_struct("Update")
                .field("id", id)
                .field("props", props)
                .field("layout", layout)
                .field("aria_props", aria_props)
                .field("aria_role", aria_role)
                .field("children", children)
                .field("handlers_count", &handlers.as_ref().map(|h| h.len()))
                .field("sdf_shape", sdf_shape)
                .finish(),
            Self::Remove(id) => f.debug_tuple("Remove").field(id).finish(),
            Self::Replace { id, node } => f
                .debug_struct("Replace")
                .field("id", id)
                .field("node", node)
                .finish(),
            Self::Move { id, new_index } => f
                .debug_struct("Move")
                .field("id", id)
                .field("new_index", new_index)
                .finish(),
            Self::SetRoot(id) => f.debug_tuple("SetRoot").field(id).finish(),
            Self::ClearHandlers { id } => f
                .debug_struct("ClearHandlers")
                .field("id", id)
                .finish(),
        }
    }
}

impl serde::Serialize for VDomPatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStructVariant;
        match self {
            Self::Create(node) => {
                serializer.serialize_newtype_variant("VDomPatch", 0, "Create", node)
            }
            Self::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => {
                let mut state = serializer.serialize_struct_variant("VDomPatch", 1, "Update", 8)?;
                state.serialize_field("id", id)?;
                state.serialize_field("props", props)?;
                state.serialize_field("layout", layout)?;
                state.serialize_field("aria_props", aria_props)?;
                state.serialize_field("aria_role", aria_role)?;
                state.serialize_field("children", children)?;
                state.serialize_field(
                    "handlers",
                    &handlers
                        .as_ref()
                        .map(|h| h.keys().cloned().collect::<Vec<String>>()),
                )?;
                state.serialize_field("sdf_shape", sdf_shape)?;
                state.end()
            }
            Self::Remove(id) => serializer.serialize_newtype_variant("VDomPatch", 2, "Remove", id),
            Self::Replace { id, node } => {
                let mut state =
                    serializer.serialize_struct_variant("VDomPatch", 3, "Replace", 2)?;
                state.serialize_field("id", id)?;
                state.serialize_field("node", node)?;
                state.end()
            }
            Self::Move { id, new_index } => {
                let mut state = serializer.serialize_struct_variant("VDomPatch", 4, "Move", 2)?;
                state.serialize_field("id", id)?;
                state.serialize_field("new_index", new_index)?;
                state.end()
            }
            Self::SetRoot(id) => {
                serializer.serialize_newtype_variant("VDomPatch", 5, "SetRoot", id)
            }
            Self::ClearHandlers { id } => {
                serializer.serialize_newtype_variant("VDomPatch", 6, "ClearHandlers", id)
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for VDomPatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum VDomPatchInternal {
            Create(VNode),
            Update {
                id: NodeId,
                props: Option<HashMap<String, serde_json::Value>>,
                layout: Option<LayoutRect>,
                aria_props: Option<AriaProps>,
                aria_role: Option<String>,
                children: Option<Vec<NodeId>>,
                handlers: Option<Vec<String>>,
                sdf_shape: Option<cvkg_core::layout::SdfShape>,
            },
            Remove(NodeId),
            Replace {
                id: NodeId,
                node: VNode,
            },
            Move {
                id: NodeId,
                new_index: usize,
            },
            SetRoot(Option<NodeId>),
        }

        let internal = VDomPatchInternal::deserialize(deserializer)?;
        Ok(match internal {
            VDomPatchInternal::Create(n) => VDomPatch::Create(n),
            VDomPatchInternal::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers,
                sdf_shape,
            } => VDomPatch::Update {
                id,
                props,
                layout,
                aria_props,
                aria_role,
                children,
                handlers: handlers.map(|keys| {
                    let mut map: EventHandlerMap = HashMap::new();
                    for key in keys {
                        // Handlers are serialized as key names only;
                        // on deserialization we create placeholder entries.
                        // The actual handler closures cannot be serialized.
                        map.insert(
                            key,
                            std::sync::Arc::new(|_| log::warn!("Cannot invoke serialized handler")),
                        );
                    }
                    map
                }),
                sdf_shape,
            },
            VDomPatchInternal::Remove(id) => VDomPatch::Remove(id),
            VDomPatchInternal::Replace { id, node } => VDomPatch::Replace { id, node },
            VDomPatchInternal::Move { id, new_index } => VDomPatch::Move { id, new_index },
            VDomPatchInternal::SetRoot(id) => VDomPatch::SetRoot(id),
        })
    }
}

/// The root container for the Virtual DOM state.
pub struct VDom {
    /// The root node ID
    pub root: Option<NodeId>,
    /// Flattened map of all nodes currently in the VDOM
    pub nodes: HashMap<NodeId, VNode>,
    /// Parent mapping for O(1) event bubbling
    pub parents: HashMap<NodeId, NodeId>,
    /// Currently focused node for keyboard events
    pub focused_node: std::sync::Mutex<Option<NodeId>>,
    /// Currently captured node for pointer events
    pub captured_node: std::sync::Mutex<Option<NodeId>>,
    /// Currently hovered node for pointer events
    pub hovered_node: std::sync::Mutex<Option<NodeId>>,
    /// Centralized event handlers for efficient delegation
    pub event_handlers: NodeEventHandlerMap,
}

impl Default for VDom {
    fn default() -> Self {
        Self::new()
    }
}

impl VDom {
    /// Create a new empty VDom.
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: HashMap::new(),
            parents: HashMap::new(),
            focused_node: std::sync::Mutex::new(None),
            captured_node: std::sync::Mutex::new(None),
            hovered_node: std::sync::Mutex::new(None),
            event_handlers: HashMap::new(),
        }
    }

    /// Build a VDom tree from a view by performing a virtual render pass.
    pub fn build<V: cvkg_core::View>(view: &V, rect: cvkg_core::Rect) -> Self {
        let mut renderer = VNodeRenderer::new();
        view.render(&mut renderer, rect);
        renderer.into_vdom()
    }

    /// Phase 4.4: Prepare this VDom to receive a new frame's nodes by clearing
    /// data while retaining allocated capacity in all HashMaps.
    ///
    /// Call this at the start of a frame rebuild (before rebuilding) to reuse the
    /// previous frame's heap allocations. This avoids repeated allocator churn
    /// when the VDom is rebuilt every frame.
    pub fn clear_and_retain_capacity(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.parents.clear();
        self.event_handlers.clear();
        // Note: focused_node, captured_node, hovered_node are Mutex<Option<NodeId>>
        // and are not cleared here -- they carry state across frames.
    }

    /// Apply a set of patches to the host's DOM environment.
    pub fn apply_to_dom(&self, patches: &[VDomPatch]) {
        // This is a bridge to the platform-specific accessibility tree (ShieldWall).
        log::debug!("Applying {} patches to host ShieldWall", patches.len());
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => log::debug!("ShieldWall: Create node {}", node.id.0),
                VDomPatch::Update { id, .. } => log::debug!("ShieldWall: Update node {}", id.0),
                VDomPatch::Remove(id) => log::debug!("ShieldWall: Remove node {}", id.0),
                VDomPatch::Replace { id, .. } => log::debug!("ShieldWall: Replace node {}", id.0),
                VDomPatch::Move { id, .. } => log::debug!("ShieldWall: Move node {}", id.0),
                VDomPatch::SetRoot(id) => log::debug!("ShieldWall: SetRoot {:?}", id),
                VDomPatch::ClearHandlers { id } => {
                    log::debug!("ShieldWall: ClearHandlers for node {}", id.0)
                }
            }
        }
    }

    /// Check if the VDOM and a SceneGraph are in perfect synchronization.
    /// Returns Err if corruption is detected, signaling a full rebuild is required.
    pub fn validate_sync(&self, scene: &cvkg_scene::SceneGraph) -> Result<(), String> {
        let _span = tracing::info_span!("vdom_validate_sync").entered();

        // 1. Root parity
        match (self.root, scene.root) {
            (None, None) => return Ok(()),
            (Some(vr), Some(sr)) if vr.0 == sr.0 => {}
            _ => return Err("Root node mismatch".to_string()),
        }

        // 2. Node count parity (approximate check for performance)
        if self.nodes.len() != scene.nodes.len() {
            return Err(format!(
                "Node count mismatch: VDom({}) vs SceneGraph({})",
                self.nodes.len(),
                scene.nodes.len()
            ));
        }

        // 3. Hierarchical Consistency Check (DFS)
        if let Some(root_id) = self.root {
            self.validate_node_sync(root_id, scene)?;
        }

        Ok(())
    }

    fn validate_node_sync(&self, id: NodeId, scene: &cvkg_scene::SceneGraph) -> Result<(), String> {
        let vnode = self
            .nodes
            .get(&id)
            .ok_or_else(|| format!("Node {} missing in VDom", id.0))?;
        let snode = scene
            .nodes
            .get(&cvkg_core::KvasirId(id.0))
            .ok_or_else(|| format!("Node {} missing in SceneGraph", id.0))?;

        // Check child count and IDs
        if vnode.children.len() != snode.children.len() {
            return Err(format!("Child count mismatch for node {}", id.0));
        }

        for (v_child, s_child) in vnode.children.iter().zip(snode.children.iter()) {
            if v_child.0 != s_child.0 {
                return Err(format!(
                    "Child ID mismatch in node {}: {} != {}",
                    id.0, v_child.0, s_child.0
                ));
            }
            self.validate_node_sync(*v_child, scene)?;
        }

        // Check visual bounds (within tolerance)
        let tolerance = 0.5;
        if (vnode.layout.x - snode.world_rect.x).abs() > tolerance
            || (vnode.layout.y - snode.world_rect.y).abs() > tolerance
        {
            return Err(format!("Spatial drift detected in node {}", id.0));
        }

        Ok(())
    }
}

/// A single decorative draw command collected into a batch.
///
/// Phase 4.1: Instead of creating individual VNodes for every decorative
/// draw call (fill_rect, fill_rounded_rect, etc.), consecutive decorative
/// operations are collected into a batch and emitted as a single VNode.
/// This reduces VDOM node count and allocation pressure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecorativeCmd {
    /// The draw command type.
    pub cmd_type: String,
    /// The bounding rect.
    pub rect: LayoutRect,
    /// Serialized command-specific properties (radius, color, width, etc.).
    pub props: HashMap<String, serde_json::Value>,
}

/// A specialized renderer that captures the component hierarchy as a Virtual DOM.
pub struct VNodeRenderer {
    nodes: HashMap<NodeId, VNode>,
    event_handlers: NodeEventHandlerMap,
    next_id: u64,
    stack: Vec<NodeId>,
    clip_stack: Vec<cvkg_core::Rect>,
    root: Option<NodeId>,
    /// Phase 4.1: Accumulated decorative commands since last flush.
    decorative_batch: Vec<DecorativeCmd>,
    /// Phase 4.1: The node ID of the current batch VNode, if any.
    batch_node_id: Option<NodeId>,
    /// Phase 2 fix: long-lived text engine, constructed once per VNodeRenderer.
    /// Avoids re-parsing fonts + spawning a thread on every text shape call.
    text_engine: cvkg_runic_text::RunicTextEngine,
}

impl Default for VNodeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl VNodeRenderer {
    /// Create a new VNodeRenderer.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            event_handlers: HashMap::new(),
            next_id: 1,
            stack: Vec::new(),
            clip_stack: Vec::new(),
            root: None,
            decorative_batch: Vec::new(),
            batch_node_id: None,
            text_engine: {
                let mut engine = cvkg_runic_text::RunicTextEngine::new_light();
                engine.load_font_data(include_bytes!("../../cvkg-runic-text/Fonts/Jupiteroid.ttf").to_vec());
                engine
            },
        }
    }

    /// Convert the captured nodes into a VDom instance.
    pub fn into_vdom(mut self) -> VDom {
        // Phase 4.1: Flush any remaining decorative batch before finalizing.
        self.flush_decorative_batch();
        log::debug!("[VDOM] Built VDOM with {} nodes", self.nodes.len());
        let mut parents = HashMap::new();
        for (id, node) in &self.nodes {
            for child_id in &node.children {
                parents.insert(*child_id, *id);
            }
        }
        VDom {
            root: self.root,
            nodes: self.nodes,
            parents,
            focused_node: std::sync::Mutex::new(None),
            captured_node: std::sync::Mutex::new(None),
            hovered_node: std::sync::Mutex::new(None),
            event_handlers: self.event_handlers,
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = KvasirId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Generate a stable NodeId from a node's key and component type.
    /// Nodes with the same (component_type, key) pair will get the same NodeId
    /// across rebuilds, ensuring event targeting survives VDOM diff/patch cycles.
    fn stable_id_for(&self, component_type: &str, key: Option<&str>) -> Option<NodeId> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let key = key?;
        let mut hasher = DefaultHasher::new();
        component_type.hash(&mut hasher);
        key.hash(&mut hasher);
        // Use high bit range to avoid collisions with sequential counter (which starts at 1)
        let hash = hasher.finish();
        Some(KvasirId(0x8000_0000_0000_0000 | (hash & 0x7FFF_FFFF_FFFF_FFFF)))
    }

    fn add_node(&mut self, mut node: VNode) -> NodeId {
        // Use stable ID derived from (component_type, key) when available.
        // This ensures the same logical node gets the same NodeId across rebuilds,
        // which is critical for event targeting (click boxes) to survive VDOM diff/patch.
        if let Some(stable_id) = self.stable_id_for(&node.component_type, node.key.as_deref()) {
            node.id = stable_id;
        }
        let id = node.id;
        log::trace!(
            "[VDOM] Adding node {:?} ({}): {:?}",
            id,
            node.component_type,
            node.layout
        );
        if let Some(parent_id) = self.stack.last() {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.push(id);
            }
        } else if self.root.is_none() {
            self.root = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    /// Phase 4.1: Flush the accumulated decorative batch as a single VNode.
    fn flush_decorative_batch(&mut self) {
        if self.decorative_batch.is_empty() {
            return;
        }
        if let Some(batch_id) = self.batch_node_id {
            if let Some(node) = self.nodes.get_mut(&batch_id) {
                node.props.insert(
                    "commands".to_string(),
                    serde_json::to_value(&self.decorative_batch).unwrap_or_default(),
                );
            }
        }
        self.decorative_batch.clear();
        self.batch_node_id = None;
    }

    /// Phase 4.1: Begin a new decorative batch. Called by decorative draw methods.
    fn begin_decorative(&mut self, rect: cvkg_core::Rect) {
        if self.batch_node_id.is_none() {
            let id = self.next_id();
            self.batch_node_id = Some(id);
            let batch_node = VNode {
                id,
                key: None,
                component_type: "Primitive::DecorativeBatch".to_string(),
                props: HashMap::new(),
                state: None,
                layout: LayoutRect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
                children: Vec::new(),
                aria_role: "presentation".to_string(),
                aria_props: AriaProps::default(),
                portal_target: None,
                sdf_shape: None,
            };
            if let Some(parent_id) = self.stack.last() {
                if let Some(parent) = self.nodes.get_mut(parent_id) {
                    parent.children.push(id);
                }
            } else if self.root.is_none() {
                self.root = Some(id);
            }
            self.nodes.insert(id, batch_node);
        }
    }

    /// Phase 4.1: Expand the current batch node's bounding rect.
    fn expand_batch_rect(&mut self, rect: cvkg_core::Rect) {
        if let Some(batch_id) = self.batch_node_id {
            if let Some(node) = self.nodes.get_mut(&batch_id) {
                let new_left = node.layout.x.min(rect.x);
                let new_top = node.layout.y.min(rect.y);
                let new_right = (node.layout.x + node.layout.width).max(rect.x + rect.width);
                let new_bottom = (node.layout.y + node.layout.height).max(rect.y + rect.height);
                node.layout.x = new_left;
                node.layout.y = new_top;
                node.layout.width = new_right - new_left;
                node.layout.height = new_bottom - new_top;
            }
        }
    }

    /// Phase 4.1: Push a decorative command and update the batch node.
    fn push_decorative_cmd(&mut self, cmd_type: &str, rect: cvkg_core::Rect, props: HashMap<String, serde_json::Value>) {
        self.expand_batch_rect(rect);
        self.decorative_batch.push(DecorativeCmd {
            cmd_type: cmd_type.to_string(),
            rect: LayoutRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            },
            props,
        });
    }
}

impl cvkg_core::ElapsedTime for VNodeRenderer {
    fn delta_time(&self) -> f32 {
        0.0 // VDOM capture is static, delta_time is irrelevant but required by trait
    }

    fn elapsed_time(&self) -> f32 {
        0.0
    }
}

impl cvkg_core::Renderer for VNodeRenderer {
    fn fill_rect(&mut self, rect: cvkg_core::Rect, _color: [f32; 4]) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        self.push_decorative_cmd("fill_rect", rect, HashMap::new());
    }



    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        // Phase 2 fix: use the long-lived text engine instead of creating a new one per call.
        // This avoids re-parsing fonts + spawning a thread on every text shape call.
        self.text_engine.shape_layout(spans, max_width, align, overflow).ok()
    }

    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        // Phase 4.1: Flush decorative batch before creating a text node.
        self.flush_decorative_batch();
        let id = self.next_id();
        let mut props = HashMap::new();
        let text = shaped.spans.iter().map(|s| s.text.as_str()).collect::<Vec<&str>>().join("");
        props.insert("text".to_string(), serde_json::Value::String(text.clone()));
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Text".to_string(),
            props,
            state: None,
            layout: LayoutRect { x, y, width: shaped.width, height: shaped.height },
            children: Vec::new(),
            aria_role: "text".to_string(),
            aria_props: AriaProps { label: Some(text), ..Default::default() },
            portal_target: None,
            sdf_shape: None,
        });
    }

    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
        // Phase 4.1: Flush decorative batch before starting a new named component.
        self.flush_decorative_batch();
        let id = self.next_id();
        let role = match name {
            "CornerButton" => "button",
            "BerserkerRoot" => "application",
            _ => "group",
        };

        self.add_node(VNode {
            id,
            key: None,
            component_type: name.to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            },
            children: Vec::new(),
            aria_role: role.to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
            sdf_shape: None,
        });
        self.stack.push(id);
    }

    fn pop_vnode(&mut self) {
        self.stack.pop();
    }

    // Standard renderer methods can be implemented as stubs or as specific VNodes if needed.
    fn fill_rounded_rect(&mut self, rect: cvkg_core::Rect, radius: f32, _color: [f32; 4]) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        self.push_decorative_cmd("fill_rounded_rect", rect, props);
    }

    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4]) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        self.push_decorative_cmd("fill_ellipse", rect, HashMap::new());
    }

    fn draw_3d_cube(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], _rotation: [f32; 3]) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        self.push_decorative_cmd("draw_3d_cube", rect, HashMap::new());
    }

    fn stroke_rect(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_rect", rect, props);
    }

    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        _color: [f32; 4],
        width: f32,
    ) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_rounded_rect", rect, props);
    }

    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("stroke_ellipse", rect, props);
    }

    fn set_sdf_shape(&mut self, shape: cvkg_core::layout::SdfShape) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.sdf_shape = Some(shape);
        }
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], width: f32) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        let rect = cvkg_core::Rect {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x1 - x2).abs(),
            height: (y1 - y2).abs(),
        };
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("x1".to_string(), serde_json::to_value(x1).unwrap());
        props.insert("y1".to_string(), serde_json::to_value(y1).unwrap());
        props.insert("x2".to_string(), serde_json::to_value(x2).unwrap());
        props.insert("y2".to_string(), serde_json::to_value(y2).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.push_decorative_cmd("draw_line", rect, props);
    }

    fn draw_texture(&mut self, _id: u32, _rect: cvkg_core::Rect) {}

    fn draw_image(&mut self, name: &str, rect: cvkg_core::Rect) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("src".to_string(), serde_json::to_value(name).unwrap());
        self.push_decorative_cmd("draw_image", rect, props);
    }

    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    fn push_clip_rect(&mut self, rect: cvkg_core::Rect) {
        self.clip_stack.push(rect);
    }
    fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }
    fn current_clip_rect(&self) -> cvkg_core::Rect {
        self.clip_stack
            .last()
            .copied()
            .unwrap_or(cvkg_core::Rect::new(-10000.0, -10000.0, 20000.0, 20000.0))
    }
    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }
    fn push_opacity(&mut self, _opacity: f32) {}
    fn pop_opacity(&mut self) {}
    fn bifrost(&mut self, _rect: cvkg_core::Rect, _blur: f32, _sat: f32, _op: f32) {}
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}

    fn mjolnir_shatter(&mut self, rect: cvkg_core::Rect, pieces: u32, force: f32, color: [f32; 4]) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("pieces".to_string(), serde_json::to_value(pieces).unwrap());
        props.insert("force".to_string(), serde_json::to_value(force).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_decorative_cmd("mjolnir_shatter", rect, props);
    }

    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    fn draw_linear_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert(
            "start_color".to_string(),
            serde_json::to_value(start_color).unwrap(),
        );
        props.insert(
            "end_color".to_string(),
            serde_json::to_value(end_color).unwrap(),
        );
        props.insert("angle".to_string(), serde_json::to_value(angle).unwrap());
        self.push_decorative_cmd("draw_linear_gradient", rect, props);
    }

    fn draw_radial_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert(
            "inner_color".to_string(),
            serde_json::to_value(inner_color).unwrap(),
        );
        props.insert(
            "outer_color".to_string(),
            serde_json::to_value(outer_color).unwrap(),
        );
        self.push_decorative_cmd("draw_radial_gradient", rect, props);
    }

    fn gungnir(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32, intensity: f32) {
        // Phase 4.1: Batch decorative draw calls into a single VNode.
        self.begin_decorative(rect);
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert(
            "intensity".to_string(),
            serde_json::to_value(intensity).unwrap(),
        );
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_decorative_cmd("gungnir", rect, props);
    }

    fn set_aria_role(&mut self, role: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_role = role.to_string();
        }
    }

    fn set_aria_label(&mut self, label: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.label = Some(label.to_string());
        }
    }

    fn set_aria_valuemin(&mut self, min: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuemin = Some(min);
        }
    }

    fn set_aria_valuemax(&mut self, max: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuemax = Some(max);
        }
    }

    fn set_aria_valuenow(&mut self, now: f32) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.aria_props.aria_valuenow = Some(now);
        }
    }

    fn register_shared_element(&mut self, _id: &str, _rect: cvkg_core::Rect) {}

    fn set_key(&mut self, key: &str) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.key = Some(key.to_string());
        }
    }

    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        // Phase 4.1: Flush decorative batch before registering a handler.
        self.flush_decorative_batch();
        if let Some(node_id) = self.stack.last() {
            log::trace!(
                "[VDOM] Registering handler '{}' on node {:?}",
                event_type,
                node_id
            );
            self.event_handlers
                .entry(*node_id)
                .or_default()
                .insert(event_type.to_string(), handler);
        }
    }
}
impl VDom {
    /// Query the accessibility tree from the VDOM.
    ///
    /// Traverses the VDOM tree from the root, collecting all nodes with
    /// ARIA roles and labels into a flat list suitable for display in
    /// the A11yInspector.
    pub fn query_accessibility_tree(
        &self,
        root: Option<NodeId>,
    ) -> Vec<crate::A11yNodeEntry> {
        let mut result = Vec::new();
        if let Some(root_id) = root {
            self.collect_a11y_nodes(root_id, 0, &mut result);
        }
        result
    }

    /// Recursively collect A11y nodes from the VDOM tree.
    fn collect_a11y_nodes(
        &self,
        id: NodeId,
        depth: u32,
        result: &mut Vec<crate::A11yNodeEntry>,
    ) {
        if let Some(node) = self.nodes.get(&id) {
            // Only include nodes that have meaningful ARIA roles
            // (skip "presentation" and "none" which are structural only)
            if node.aria_role != "presentation" && node.aria_role != "none" {
                let label = node.aria_props.label.clone().unwrap_or_default();
                let value = node.aria_props.aria_valuenow.map(|v| {
                    let min = node.aria_props.aria_valuemin.unwrap_or(0.0);
                    let max = node.aria_props.aria_valuemax.unwrap_or(100.0);
                    let pct = if max > min {
                        ((v - min) / (max - min) * 100.0).round() as u32
                    } else {
                        0
                    };
                    format!("{}%", pct)
                });

                let focused = *self
                    .focused_node
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    == Some(id);

                result.push(crate::A11yNodeEntry {
                    role: node.aria_role.clone(),
                    label,
                    value,
                    focused,
                    enabled: !node.aria_props.disabled,
                    depth,
                });
            }

            // Recurse into children
            for child_id in &node.children {
                self.collect_a11y_nodes(*child_id, depth + 1, result);
            }
        }
    }

    /// Mutate the Virtual DOM state by applying a sequence of patches.
    #[tracing::instrument(skip(self, patches))]
    pub fn apply_patches(&mut self, patches: Vec<VDomPatch>) {
        if !patches.is_empty() {
            println!("VDom: Applying {} patches", patches.len());
        }
        let _span = tracing::info_span!("vdom_apply_patches").entered();
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => {
                    for child_id in &node.children {
                        self.parents.insert(*child_id, node.id);
                    }
                    self.nodes.insert(node.id, node);
                }
                VDomPatch::Update {
                    id,
                    props,
                    layout,
                    aria_props,
                    aria_role,
                    children,
                    handlers,
                    sdf_shape,
                } => {
                    if let Some(node) = self.nodes.get_mut(&id) {
                        if let Some(p) = props {
                            node.props = p;
                        }
                        if let Some(l) = layout {
                            node.layout = l;
                        }
                        if let Some(ap) = aria_props {
                            node.aria_props = ap;
                        }
                        if let Some(ar) = aria_role {
                            node.aria_role = ar;
                        }
                        if let Some(c) = children {
                            // Update children and parents map
                            for child_id in &node.children {
                                self.parents.remove(child_id);
                            }
                            node.children = c;
                            for child_id in &node.children {
                                self.parents.insert(*child_id, id);
                            }
                        }
                        if let Some(h) = handlers {
                            self.event_handlers.insert(id, h);
                        }
                        if let Some(s) = sdf_shape {
                            node.sdf_shape = Some(s);
                        }
                    }
                }
                VDomPatch::Remove(id) => {
                    if let Some(node) = self.nodes.remove(&id) {
                        for child_id in &node.children {
                            self.parents.remove(child_id);
                        }
                    }
                    self.parents.remove(&id);
                }
                VDomPatch::Replace { id, node } => {
                    let is_root = self.root == Some(id);
                    let new_id = node.id;

                    // Cleanup old children from parents map
                    if let Some(old_node) = self.nodes.get(&id) {
                        for child_id in &old_node.children {
                            self.parents.remove(child_id);
                        }
                    }
                    for child_id in &node.children {
                        self.parents.insert(*child_id, new_id);
                    }

                    // Update nodes map. We use the new_id as the key to keep it consistent.
                    self.nodes.remove(&id);
                    self.nodes.insert(new_id, node);

                    if is_root {
                        self.root = Some(new_id);
                    }

                    // Migrate capture and focus state
                    if let Ok(mut capture) = self.captured_node.lock()
                        && *capture == Some(id)
                    {
                        *capture = Some(new_id);
                    }
                    if let Ok(mut focus) = self.focused_node.lock()
                        && *focus == Some(id)
                    {
                        *focus = Some(new_id);
                    }
                }
                VDomPatch::Move { id, new_index } => {
                    if let Some(&p_id) = self.parents.get(&id)
                        && let Some(parent) = self.nodes.get_mut(&p_id)
                        && let Some(old_pos) = parent.children.iter().position(|&x| x == id)
                    {
                        parent.children.remove(old_pos);
                        let target_pos = new_index.min(parent.children.len());
                        parent.children.insert(target_pos, id);
                    }
                }
                VDomPatch::SetRoot(id) => {
                    self.root = id;
                }
                VDomPatch::ClearHandlers { id } => {
                    // P0-6 fix: explicitly clear handlers for this node so a
                    // removed `on_click` doesn't ghost-fire later. Without
                    // this variant, the Update.handlers=None path leaves
                    // the old handler attached forever.
                    self.event_handlers.remove(&id);
                }
            }
        }
    }

    /// Compute the difference between this VDom and another.
    ///
    /// Generates a minimal sequence of `VDomPatch` instructions to transition
    /// the host accessibility DOM from `self` to `other`.
    #[tracing::instrument(skip(self, other))]
    pub fn diff(&self, other: &VDom) -> Vec<VDomPatch> {
        let _span = tracing::info_span!("vdom_diff").entered();
        let mut patches = Vec::new();

        // Handle root changes
        match (self.root.as_ref(), other.root.as_ref()) {
            (None, None) => return patches,
            (None, Some(new_root_id)) => {
                if let Some(new_node) = other.nodes.get(new_root_id) {
                    patches.push(VDomPatch::Create(new_node.clone()));
                    patches.push(VDomPatch::SetRoot(Some(*new_root_id)));
                }
            }
            (Some(old_root_id), None) => {
                patches.push(VDomPatch::Remove(*old_root_id));
                patches.push(VDomPatch::SetRoot(None));
            }
            (Some(old_root_id), Some(new_root_id)) => {
                if old_root_id != new_root_id {
                    if let Some(new_node) = other.nodes.get(new_root_id) {
                        patches.push(VDomPatch::Replace {
                            id: *old_root_id,
                            node: new_node.clone(),
                        });
                        patches.push(VDomPatch::SetRoot(Some(*new_root_id)));
                    }
                } else {
                    self.diff_node(*old_root_id, *new_root_id, other, &mut patches);
                }
            }
        }

        patches
    }

    fn diff_node(
        &self,
        old_id: NodeId,
        new_id: NodeId,
        other: &VDom,
        patches: &mut Vec<VDomPatch>,
    ) {
        let old_node = match self.nodes.get(&old_id) {
            Some(n) => n,
            None => return,
        };
        let new_node = match other.nodes.get(&new_id) {
            Some(n) => n,
            None => return,
        };

        // If components are completely different types or have different keys, replace.
        if old_node.component_type != new_node.component_type || old_node.key != new_node.key {
            patches.push(VDomPatch::Replace {
                id: old_id,
                node: new_node.clone(),
            });
            return;
        }

        // If props, layout, aria_props, or children changed, emit an Update
        let props_changed = old_node.props != new_node.props;
        let layout_changed = old_node.layout != new_node.layout;
        let aria_props_changed = old_node.aria_props != new_node.aria_props;
        let aria_role_changed = old_node.aria_role != new_node.aria_role;
        let children_changed = old_node.children != new_node.children;
        let sdf_shape_changed = old_node.sdf_shape != new_node.sdf_shape;

        // P0-7 fix: compare old vs new handler maps directly. The previous
        // check `other.event_handlers.contains_key(&new_id)` always returned
        // true when the new tree had a handler (even if identical), causing
        // spurious Update patches every frame, AND it returned false when
        // the new tree had no handler (preventing handler removal, see P0-6).
        let old_handlers = self.event_handlers.get(&new_id);
        let new_handlers = other.event_handlers.get(&new_id);
        // `Arc<dyn Fn>` doesn't implement PartialEq, so we compare by:
        //   1) presence/absence (Option layer)
        //   2) key set equality
        //   3) Arc pointer identity per key (same Arc instance => same closure)
        let handlers_changed = match (old_handlers, new_handlers) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(a), Some(b)) => {
                a.len() != b.len()
                    || a.keys().any(|k| {
                        b.get(k)
                            .map_or(true, |bv| !std::sync::Arc::ptr_eq(a.get(k).unwrap(), bv))
                    })
            }
        };
        // P0-6 fix: detect handler removal explicitly. Without this, a button
        // whose `on_click` is dropped retains the old handler forever (ghost
        // click bug + memory leak).
        let handlers_removed = old_handlers.is_some() && new_handlers.is_none();

        if props_changed
            || layout_changed
            || aria_props_changed
            || aria_role_changed
            || children_changed
            || sdf_shape_changed
            || handlers_changed
        {
            patches.push(VDomPatch::Update {
                id: old_id,
                props: if props_changed {
                    Some(new_node.props.clone())
                } else {
                    None
                },
                layout: if layout_changed {
                    Some(new_node.layout)
                } else {
                    None
                },
                aria_props: if aria_props_changed {
                    Some(new_node.aria_props.clone())
                } else {
                    None
                },
                aria_role: if aria_role_changed {
                    Some(new_node.aria_role.clone())
                } else {
                    None
                },
                children: if children_changed {
                    Some(new_node.children.clone())
                } else {
                    None
                },
                handlers: other.event_handlers.get(&new_id).cloned(),
                sdf_shape: if sdf_shape_changed {
                    new_node.sdf_shape
                } else {
                    None
                },
            });
        }

        // P0-6 fix: emit ClearHandlers when handlers were removed. The
        // Update patch above is skipped if no other field changed, so a
        // pure handler removal would otherwise produce no patch at all,
        // leaving the old handler attached forever.
        if handlers_removed {
            patches.push(VDomPatch::ClearHandlers { id: old_id });
        }

        // High-fidelity Keyed Child Diffing
        // Enterprise-Grade Keyed Child Diffing (LIS-based)
        let old_children = &old_node.children;
        let new_children = &new_node.children;

        // 1. Map old children by key for fast lookup
        let mut old_keyed: HashMap<String, (usize, NodeId)> = HashMap::new();
        for (i, id) in old_children.iter().enumerate() {
            if let Some(node) = self.nodes.get(id)
                && let Some(key) = &node.key
            {
                old_keyed.insert(key.clone(), (i, *id));
            }
        }

        // 2. Identify moves and updates
        let mut last_index = 0;
        let mut source_indices = vec![-1; new_children.len()];
        let mut moved = false;

        for (i, new_child_id) in new_children.iter().enumerate() {
            let new_child = match other.nodes.get(new_child_id) {
                Some(n) => n,
                None => continue,
            };

            if let Some(key) = &new_child.key {
                if let Some((old_idx, old_child_id)) = old_keyed.remove(key) {
                    source_indices[i] = old_idx as i32;
                    self.diff_node(old_child_id, *new_child_id, other, patches);
                    if old_idx < last_index {
                        moved = true;
                    } else {
                        last_index = old_idx;
                    }
                } else {
                    patches.push(VDomPatch::Create(new_child.clone()));
                }
            } else if i < old_children.len() {
                self.diff_node(old_children[i], *new_child_id, other, patches);
            } else {
                patches.push(VDomPatch::Create(new_child.clone()));
            }
        }

        // 3. Apply moves using LIS to minimize mutations
        if moved {
            let lis = self.calculate_lis(&source_indices);
            let mut lis_idx = lis.len() as i32 - 1;
            for i in (0..new_children.len()).rev() {
                if source_indices[i] != -1 {
                    if lis_idx >= 0 && lis[lis_idx as usize] == i as i32 {
                        lis_idx -= 1;
                    } else {
                        patches.push(VDomPatch::Move {
                            id: new_children[i],
                            new_index: i,
                        });
                    }
                }
            }
        }

        // 4. Cleanup remaining old keyed nodes
        for (_, (_, id)) in old_keyed {
            patches.push(VDomPatch::Remove(id));
        }

        // 5. Cleanup excess unkeyed old children
        if old_children.len() > new_children.len() {
            for id in old_children.iter().skip(new_children.len()) {
                if self.nodes.get(id).is_some_and(|n| n.key.is_none()) {
                    patches.push(VDomPatch::Remove(*id));
                }
            }
        }
    }

    /// Calculate the Longest Increasing Subsequence indices
    fn calculate_lis(&self, arr: &[i32]) -> Vec<i32> {
        let n = arr.len();
        if n == 0 {
            return Vec::new();
        }

        let mut p = vec![0; n];
        let mut m = vec![0; n + 1];
        let mut l = 0;

        for i in 0..n {
            if arr[i] == -1 {
                continue;
            }

            let mut low = 1;
            let mut high = l;
            while low <= high {
                let mid = (low + high) / 2;
                if arr[m[mid] as usize] < arr[i] {
                    low = mid + 1;
                } else {
                    high = mid - 1;
                }
            }

            let new_l = low;
            p[i] = m[new_l - 1];
            m[new_l] = i as i32;

            if new_l > l {
                l = new_l;
            }
        }

        let mut res = vec![0; l];
        let mut k = m[l];
        for i in (0..l).rev() {
            res[i] = k;
            k = p[k as usize];
        }
        res
    }

    fn sdf_distance(
        shape: Option<&cvkg_core::layout::SdfShape>,
        layout: &LayoutRect,
        x: f32,
        y: f32,
    ) -> f32 {
        let shape =
            shape
                .copied()
                .unwrap_or(cvkg_core::layout::SdfShape::Rect(cvkg_core::layout::Rect {
                    x: layout.x,
                    y: layout.y,
                    width: layout.width,
                    height: layout.height,
                }));
        match shape {
            cvkg_core::layout::SdfShape::Rect(r) => {
                let dx = (r.x - x).max(x - (r.x + r.width)).max(0.0);
                let dy = (r.y - y).max(y - (r.y + r.height)).max(0.0);
                if dx == 0.0 && dy == 0.0 {
                    let in_x = (x - r.x).min(r.x + r.width - x);
                    let in_y = (y - r.y).min(r.y + r.height - y);
                    -in_x.min(in_y)
                } else {
                    (dx * dx + dy * dy).sqrt()
                }
            }
            cvkg_core::layout::SdfShape::RoundedRect { rect: r, radius } => {
                let hw = r.width / 2.0;
                let hh = r.height / 2.0;
                let cx = r.x + hw;
                let cy = r.y + hh;
                let dx = (x - cx).abs() - hw + radius;
                let dy = (y - cy).abs() - hh + radius;
                dx.max(0.0).hypot(dy.max(0.0)) + dx.max(dy).min(0.0) - radius
            }
            cvkg_core::layout::SdfShape::Circle { center, radius } => {
                ((x - center[0]).powi(2) + (y - center[1]).powi(2)).sqrt() - radius
            }
        }
    }

    /// Perform hit testing to find the front-most node at the given coordinates.
    ///
    /// WHY: Looks up node at coordinates.
    /// CONTRACT: Uses pointer_precision to define the maximum expansion radius for hit matching.
    pub fn hit_test(&self, x: f32, y: f32, pointer_precision: f32) -> Option<(NodeId, f32)> {
        self.root
            .and_then(|root_id| self.hit_test_recursive(root_id, x, y, pointer_precision))
    }

    /// Perform recursive hit-testing down the VDOM tree.
    ///
    /// WHY: Returns the matched node ID and its proximity score.
    /// CONTRACT: Prioritizes direct hits (proximity == 1.0, meaning the coordinates fall within the node's layout)
    /// over proximity-based target expansion hits to prevent outer targets from blocking focused elements.
    /// Dynamic bounds expansion is scaled using the provided `pointer_precision` metric.
    fn hit_test_recursive(
        &self,
        node_id: NodeId,
        x: f32,
        y: f32,
        pointer_precision: f32,
    ) -> Option<(NodeId, f32)> {
        let node = self.nodes.get(&node_id)?;

        let dist = Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y);

        // Scale proximity limit based on the precision of the pointer device.
        let proximity_limit = pointer_precision.max(0.0);
        let proximity = if dist <= 0.0 {
            1.0
        } else if proximity_limit > 0.0 {
            (1.0 - (dist / proximity_limit)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if proximity > 0.0 {
            // Search children in reverse (front-to-back) to maintain proper Z-ordering.
            let mut best_child_hit: Option<(NodeId, f32)> = None;
            for child_id in node.children.iter().rev() {
                if let Some((hit, hit_prox)) =
                    self.hit_test_recursive(*child_id, x, y, pointer_precision)
                {
                    // Direct hit (point inside child's SDF): return immediately
                    if hit_prox >= 1.0 {
                        return Some((hit, hit_prox));
                    }
                    // Track best partial hit among children
                    if best_child_hit.is_none() || hit_prox > best_child_hit.unwrap().1 {
                        best_child_hit = Some((hit, hit_prox));
                    }
                }
            }

            // If any child matched (even partially), prefer the best child hit
            // over the parent. This prevents the root from swallowing clicks
            // when children cover the click point but lack event handlers.
            if let Some(bh) = best_child_hit {
                return Some(bh);
            }

            // No child matched at all -- return this node if it's interactive
            if dist <= 0.0 || self.event_handlers.contains_key(&node_id) {
                return Some((node_id, proximity));
            }
        }

        None
    }

    /// Dispatch an event to the VDOM by performing a hit test and calling the handler.
    pub fn dispatch_event(&self, mut event: cvkg_core::Event) -> cvkg_core::EventResponse {
        let _span = tracing::info_span!("vdom_dispatch_event").entered();
        let event_name = event.name();

        log::trace!("[VDOM] DISPATCH: {} (root={:?})", event_name, self.root);

        let captured_target = self.captured_node.lock().ok().and_then(|captured| *captured);
        let target_id = match event {
            cvkg_core::Event::PointerDown { x, y, .. }
            | cvkg_core::Event::PointerUp { x, y, .. }
            | cvkg_core::Event::PointerMove { x, y, .. }
            | cvkg_core::Event::PointerClick { x, y, .. }
            | cvkg_core::Event::PointerWheel { x, y, .. }
            | cvkg_core::Event::PointerDoubleClick { x, y, .. }
            | cvkg_core::Event::DragStart { x, y, .. }
            | cvkg_core::Event::DragMove { x, y, .. }
            | cvkg_core::Event::DragEnd { x, y, .. }
            | cvkg_core::Event::FileDrop { x, y, .. } => {
                let use_capture = matches!(
                    event,
                    cvkg_core::Event::PointerUp { .. }
                        | cvkg_core::Event::PointerClick { .. }
                        | cvkg_core::Event::DragMove { .. }
                        | cvkg_core::Event::DragEnd { .. }
                ) && captured_target.is_some();

                let (id, proximity) = if use_capture {
                    let captured = captured_target;
                    log::trace!("[VDOM] Using captured target for {}: {:?}", event_name, captured);
                    (captured, 1.0)
                } else {
                    log::trace!(
                        "[VDOM] Hit testing at ({}, {}) with precision {}",
                        x,
                        y,
                        event.pointer_precision()
                    );
                    let (id, proximity) = match self.hit_test(x, y, event.pointer_precision()) {
                        Some((i, p)) => (Some(i), p),
                        None => (None, 0.0),
                    };
                    log::trace!("[VDOM] Hit test result: {:?}, proximity: {}", id, proximity);
                    (id, proximity)
                };

                if let cvkg_core::Event::PointerMove {
                    ref mut proximity_field,
                    ..
                } = event
                {
                    *proximity_field = proximity;
                }
                if let cvkg_core::Event::PointerDown {
                    ref mut proximity_field,
                    ..
                } = event
                {
                    *proximity_field = proximity;
                }

                // Update focus/capture/hover state
                if let cvkg_core::Event::PointerDown { .. } = event {
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = id;
                    }
                    if let Ok(mut capture) = self.captured_node.lock() {
                        *capture = id;
                    }
                }
                if let cvkg_core::Event::PointerUp { .. } = event
                    && let Ok(mut capture) = self.captured_node.lock()
                {
                    *capture = None;
                }

                // Handle hover transitions
                if let cvkg_core::Event::PointerMove { .. } = event {
                    let old_hover = if let Ok(mut hover) = self.hovered_node.lock() {
                        let prev = *hover;
                        *hover = id;
                        prev
                    } else {
                        None
                    };

                    if old_hover != id {
                        if let Some(old_id) = old_hover {
                            self.bubble_event(old_id, &cvkg_core::Event::PointerLeave);
                        }
                        if let Some(new_id) = id {
                            self.bubble_event(new_id, &cvkg_core::Event::PointerEnter);
                        }
                    }
                }

                id
            }
            cvkg_core::Event::FocusIn | cvkg_core::Event::FocusOut => {
                // Focus events dispatch to the currently focused node
                self.focused_node.lock().ok().and_then(|f| *f)
            }
            _ => {
                // Focus-based dispatch for keyboard and clipboard events
                self.focused_node.lock().ok().and_then(|f| *f)
            }
        };

        if let Some(id) = target_id {
            log::trace!(
                "[VDOM] Dispatching {} to node {:?} ({})",
                event_name,
                id,
                self.nodes
                    .get(&id)
                    .map(|n| n.component_type.as_str())
                    .unwrap_or("UNKNOWN")
            );
            self.bubble_event_response(id, event)
        } else {
            log::trace!("[VDOM] No hit for event {} at {:?}", event_name, event);
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Dispatch an event directly to a specific node and bubble upward from there.
    ///
    /// WHY: Native pointer sequences can preserve the original press target across
    /// rebuilds, preventing click-box drift when the tree changes mid-interaction.
    /// CONTRACT: The target must refer to a live node in this VDOM; otherwise the
    /// event is ignored.
    pub fn dispatch_event_to_target(
        &self,
        target: NodeId,
        event: cvkg_core::Event,
    ) -> cvkg_core::EventResponse {
        if self.nodes.contains_key(&target) {
            self.bubble_event_response(target, event)
        } else {
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Bubble an event up the tree from a target node to the root.
    ///
    /// Walks the parent chain from `target` to root, calling any matching
    /// event handlers at each level. Returns `true` if at least one handler
    /// processed the event, `false` if it bubbled to the root unhandled.
    pub fn bubble_event(&self, target: NodeId, event: &cvkg_core::Event) -> bool {
        let event_name = event.name();
        let mut current_id = target;
        let mut processed = false;

        loop {
            if let Some(handlers) = self.event_handlers.get(&current_id)
                && let Some(handler) = handlers.get(event_name)
            {
                handler(event.clone());
                processed = true;
            }

            if let Some(parent_id) = self.parents.get(&current_id) {
                current_id = *parent_id;
            } else {
                break;
            }
        }

        processed
    }

    /// Internal helper that dispatches and converts to EventResponse.
    fn bubble_event_response(
        &self,
        mut current_id: NodeId,
        event: cvkg_core::Event,
    ) -> cvkg_core::EventResponse {
        let event_name = event.name();
        let mut processed = false;

        loop {
            if let Some(handlers) = self.event_handlers.get(&current_id) {
                log::trace!("[VDOM] Checking node {:?} for handlers", current_id);
                if let Some(handler) = handlers.get(event_name) {
                    log::debug!(
                        "[VDOM] Executing handler for '{}' on node {:?}",
                        event_name,
                        current_id
                    );
                    handler(event.clone());
                    processed = true;
                }
            }

            if let Some(parent_id) = self.parents.get(&current_id) {
                current_id = *parent_id;
            } else {
                break;
            }
        }

        if processed {
            cvkg_core::EventResponse::Handled
        } else {
            cvkg_core::EventResponse::Ignored
        }
    }

    /// Set the focused node by ID.
    pub fn focus_node(&self, id: NodeId) {
        if let Ok(mut focus) = self.focused_node.lock() {
            *focus = Some(id);
        }
    }

    /// Clear the current focus.
    pub fn blur_node(&self) {
        if let Ok(mut focus) = self.focused_node.lock() {
            *focus = None;
        }
    }

    /// Build the document-order (DFS pre-order) list of node IDs.
    fn build_focus_order(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        if let Some(root_id) = self.root {
            self.dfs_pre_order(root_id, &mut order);
        }
        order
    }

    fn dfs_pre_order(&self, node_id: NodeId, order: &mut Vec<NodeId>) {
        order.push(node_id);
        if let Some(node) = self.nodes.get(&node_id) {
            for child_id in &node.children {
                self.dfs_pre_order(*child_id, order);
            }
        }
    }

    /// Move focus to the next node in document order (DFS pre-order).
    pub fn focus_next(&self) {
        let order = self.build_focus_order();
        if order.is_empty() {
            return;
        }

        let current = self.focused_node.lock().ok().and_then(|f| *f);
        match current {
            None => {
                if let Ok(mut focus) = self.focused_node.lock() {
                    *focus = Some(order[0]);
                }
            }
            Some(cur_id) => {
                if let Some(pos) = order.iter().position(|id| *id == cur_id) {
                    let next = (pos + 1) % order.len();
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = Some(order[next]);
                    }
                }
            }
        }
    }

    /// Move focus to the previous node in document order (DFS pre-order).
    pub fn focus_prev(&self) {
        let order = self.build_focus_order();
        if order.is_empty() {
            return;
        }

        let current = self.focused_node.lock().ok().and_then(|f| *f);
        match current {
            None => {
                if let Ok(mut focus) = self.focused_node.lock() {
                    *focus = Some(order[order.len() - 1]);
                }
            }
            Some(cur_id) => {
                if let Some(pos) = order.iter().position(|id| *id == cur_id) {
                    let prev = if pos == 0 { order.len() - 1 } else { pos - 1 };
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = Some(order[prev]);
                    }
                }
            }
        }
    }

    /// Build a complete AccessKit tree update from the current VDOM state.
    ///
    /// Generates a `TreeUpdate` containing all nodes with proper parent-child
    /// relationships, roles, labels, descriptions, bounds, and states.
    pub fn build_accesskit_tree(&self) -> accesskit::TreeUpdate {
        let mut nodes: Vec<(accesskit::NodeId, accesskit::Node)> = Vec::new();

        if let Some(root_id) = self.root {
            self.build_accesskit_node(root_id, &mut nodes);
        }

        accesskit::TreeUpdate {
            nodes,
            tree: Some(accesskit::Tree::new(accesskit::NodeId(0))),
            focus: self
                .focused_node
                .lock()
                .ok()
                .and_then(|f| *f)
                .map(|id| accesskit::NodeId(id.0))
                .unwrap_or(accesskit::NodeId(0)),
            tree_id: accesskit::TreeId::ROOT,
        }
    }

    fn build_accesskit_node(
        &self,
        node_id: NodeId,
        output: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) {
        if let Some(node) = self.nodes.get(&node_id) {
            let mut ak_node = accesskit::Node::new(match node.aria_role.as_str() {
                // All 53 AriaRole variants mapped to AccessKit Role equivalents
                "alert" => accesskit::Role::Alert,
                "alertdialog" => accesskit::Role::AlertDialog,
                "article" => accesskit::Role::Article,
                "banner" => accesskit::Role::Banner,
                "button" => accesskit::Role::Button,
                "checkbox" => accesskit::Role::CheckBox,
                "columnheader" => accesskit::Role::ColumnHeader,
                "combobox" => accesskit::Role::ComboBox,
                "complementary" => accesskit::Role::Complementary,
                "contentinfo" => accesskit::Role::ContentInfo,
                "dialog" => accesskit::Role::Dialog,
                "form" => accesskit::Role::Form,
                "grid" => accesskit::Role::Grid,
                "gridcell" => accesskit::Role::GridCell,
                "heading" => accesskit::Role::Heading,
                "img" => accesskit::Role::Image,
                "link" => accesskit::Role::Link,
                "list" => accesskit::Role::List,
                "listbox" => accesskit::Role::ListBox,
                "listitem" => accesskit::Role::ListItem,
                "main" => accesskit::Role::Main,
                "menu" => accesskit::Role::Menu,
                "menubar" => accesskit::Role::MenuBar,
                "menuitem" => accesskit::Role::MenuItem,
                "menuitemcheckbox" => accesskit::Role::MenuItemCheckBox,
                "menuitemradio" => accesskit::Role::MenuItemRadio,
                "navigation" => accesskit::Role::Navigation,
                "none" => accesskit::Role::GenericContainer,
                "note" => accesskit::Role::Note,
                "option" => accesskit::Role::ListBoxOption,
                "presentation" => accesskit::Role::GenericContainer,
                "progressbar" => accesskit::Role::ProgressIndicator,
                "radio" => accesskit::Role::RadioButton,
                "radiogroup" => accesskit::Role::RadioGroup,
                "region" => accesskit::Role::Region,
                "row" => accesskit::Role::Row,
                "rowgroup" => accesskit::Role::RowGroup,
                "rowheader" => accesskit::Role::RowHeader,
                "search" => accesskit::Role::Search,
                "separator" => accesskit::Role::Splitter,
                "slider" => accesskit::Role::Slider,
                "spinbutton" => accesskit::Role::SpinButton,
                "status" => accesskit::Role::Status,
                "switch" => accesskit::Role::Switch,
                "tab" => accesskit::Role::Tab,
                "table" => accesskit::Role::Table,
                "tablist" => accesskit::Role::TabList,
                "tabpanel" => accesskit::Role::TabPanel,
                "textbox" => accesskit::Role::TextInput,
                "toolbar" => accesskit::Role::Toolbar,
                "tooltip" => accesskit::Role::Tooltip,
                "tree" => accesskit::Role::Tree,
                "treeitem" => accesskit::Role::TreeItem,

                // Non-ARIA utility roles used by the codebase
                "text" => accesskit::Role::Label,
                "group" => accesskit::Role::Group,
                "window" => accesskit::Role::Window,
                "password" => accesskit::Role::TextInput,
                "application" => accesskit::Role::Application,
                "colorwell" => accesskit::Role::ColorWell,

                _ => accesskit::Role::Unknown,
            });

            if let Some(label) = &node.aria_props.label {
                ak_node.set_label(label.clone());
            }

            if let Some(desc) = &node.aria_props.description {
                ak_node.set_description(desc.clone());
            }

            if let Some(val) = &node.aria_props.value {
                ak_node.set_value(val.clone());
            }

            // Expose ARIA slider/progress/meter numeric values to accesskit
            if let Some(now) = node.aria_props.aria_valuenow {
                ak_node.set_numeric_value(now as f64);
            }
            if let Some(min) = node.aria_props.aria_valuemin {
                ak_node.set_min_numeric_value(min as f64);
            }
            if let Some(max) = node.aria_props.aria_valuemax {
                ak_node.set_max_numeric_value(max as f64);
            }

            if node.aria_props.disabled {
                ak_node.set_disabled();
            }

            if node.aria_props.hidden {
                ak_node.set_hidden();
            }

            ak_node.set_bounds(accesskit::Rect {
                x0: node.layout.x as f64,
                y0: node.layout.y as f64,
                x1: (node.layout.x + node.layout.width) as f64,
                y1: (node.layout.y + node.layout.height) as f64,
            });

            let child_ids: Vec<accesskit::NodeId> = node
                .children
                .iter()
                .map(|id| accesskit::NodeId(id.0))
                .collect();
            ak_node.set_children(child_ids);

            output.push((accesskit::NodeId(node_id.0), ak_node));

            for child_id in &node.children {
                self.build_accesskit_node(*child_id, output);
            }
        }
    }
}

/// State management hook primitive for the Virtual DOM.
///
/// Returns a tuple containing:
/// 1. The current state value of type `T`.
/// 2. A setter `Arc<dyn Fn(T)>` to update the state.
pub fn use_state<T: Clone + Send + Sync + 'static>(
    id_hash: u64,
    default: T,
) -> (T, std::sync::Arc<dyn Fn(T) + Send + Sync>) {
    let current = {
        let s = cvkg_core::load_system_state();
        match s.get_component_state::<T>(id_hash) {
            Some(arc_val) => arc_val.read().unwrap().clone(),
            None => {
                cvkg_core::update_system_state(|s| {
                    let mut s = s.clone();
                    s.set_component_state(id_hash, default.clone());
                    s
                });
                default.clone()
            }
        }
    };

    let setter = std::sync::Arc::new(move |new_val: T| {
        cvkg_core::update_system_state(move |s| {
            let mut s = s.clone();
            s.set_component_state(id_hash, new_val.clone());
            s
        });
    });

    (current, setter)
}

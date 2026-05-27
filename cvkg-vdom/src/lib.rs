//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! Virtual DOM implementation for CVKG

use cvkg_core::Renderer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A map of event names to their corresponding thread-safe handlers.
pub type EventHandlerMap = HashMap<String, Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>;

/// A map of node IDs to their respective event handler maps.
pub type NodeEventHandlerMap = HashMap<NodeId, EventHandlerMap>;

/// A unique identifier for a node within the Virtual DOM tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

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
            "button" => accesskit::Role::Button,
            "checkbox" => accesskit::Role::CheckBox,
            "text" => accesskit::Role::Label,
            "group" => accesskit::Role::Group,
            "window" => accesskit::Role::Window,
            "textbox" => accesskit::Role::TextInput,
            "password" => accesskit::Role::TextInput, // We'll set the password property below
            "switch" => accesskit::Role::Switch,
            "slider" => accesskit::Role::Slider,
            "spinbutton" => accesskit::Role::SpinButton,
            "combobox" => accesskit::Role::ComboBox,
            "grid" => accesskit::Role::Grid,
            "colorwell" => accesskit::Role::ColorWell,
            _ => accesskit::Role::GenericContainer,
        });

        if self.aria_role == "password" {
            // Note: In some accesskit versions, you might need a specific property for password.
            // For now, we rely on the role mapping or a custom property if available.
        }

        if let Some(label) = &self.aria_props.label {
            node.set_label(label.clone());
        }

        if let Some(desc) = &self.aria_props.description {
            node.set_value(desc.clone()); // Or description if supported, value is typically read
        }

        if let Some(val) = &self.aria_props.value {
            node.set_value(val.clone());
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

    /// Apply a set of patches to the host's DOM environment.
    pub fn apply_to_dom(&self, patches: &[VDomPatch]) {
        // This is a bridge to the platform-specific accessibility tree (ShieldWall).
        log::info!("Applying {} patches to host ShieldWall", patches.len());
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => log::debug!("ShieldWall: Create node {}", node.id.0),
                VDomPatch::Update { id, .. } => log::debug!("ShieldWall: Update node {}", id.0),
                VDomPatch::Remove(id) => log::debug!("ShieldWall: Remove node {}", id.0),
                VDomPatch::Replace { id, .. } => log::debug!("ShieldWall: Replace node {}", id.0),
                VDomPatch::Move { id, .. } => log::debug!("ShieldWall: Move node {}", id.0),
                VDomPatch::SetRoot(id) => log::debug!("ShieldWall: SetRoot {:?}", id),
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
            .get(&cvkg_scene::NodeId(id.0))
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

/// A specialized renderer that captures the component hierarchy as a Virtual DOM.
pub struct VNodeRenderer {
    nodes: HashMap<NodeId, VNode>,
    event_handlers: NodeEventHandlerMap,
    next_id: u64,
    stack: Vec<NodeId>,
    clip_stack: Vec<cvkg_core::Rect>,
    root: Option<NodeId>,
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
        }
    }

    /// Convert the captured nodes into a VDom instance.
    pub fn into_vdom(self) -> VDom {
        log::info!("[VDOM] Built VDOM with {} nodes", self.nodes.len());
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
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn add_node(&mut self, node: VNode) -> NodeId {
        let id = node.id;
        log::info!(
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
        let id = self.next_id();
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Rect".to_string(),
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
        });
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, _color: [f32; 4]) {
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );
        let (w, h) = self.measure_text(text, size);
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Text".to_string(),
            props,
            state: None,
            layout: LayoutRect {
                x,
                y,
                width: w,
                height: h,
            },
            children: Vec::new(),
            aria_role: "text".to_string(),
            aria_props: AriaProps {
                label: Some(text.to_string()),
                ..Default::default()
            },
            portal_target: None,
            sdf_shape: None,
        });
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        // VDOM capture only needs rough bounds for layout
        (text.len() as f32 * size * 0.6, size)
    }

    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
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
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::RoundedRect".to_string(),
            props,
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
        });
    }

    fn fill_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4]) {
        let id = self.next_id();
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Ellipse".to_string(),
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
        });
    }

    fn draw_3d_cube(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], _rotation: [f32; 3]) {
        let id = self.next_id();
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Cube3D".to_string(),
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
        });
    }

    fn stroke_rect(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::StrokeRect".to_string(),
            props,
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
        });
    }

    fn stroke_rounded_rect(
        &mut self,
        rect: cvkg_core::Rect,
        radius: f32,
        _color: [f32; 4],
        width: f32,
    ) {
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::StrokeRoundedRect".to_string(),
            props,
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
        });
    }

    fn stroke_ellipse(&mut self, rect: cvkg_core::Rect, _color: [f32; 4], width: f32) {
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::StrokeEllipse".to_string(),
            props,
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
        });
    }

    fn set_sdf_shape(&mut self, shape: cvkg_core::layout::SdfShape) {
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.sdf_shape = Some(shape);
        }
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], width: f32) {
        let mut props = HashMap::new();
        props.insert("x1".to_string(), serde_json::to_value(x1).unwrap());
        props.insert("y1".to_string(), serde_json::to_value(y1).unwrap());
        props.insert("x2".to_string(), serde_json::to_value(x2).unwrap());
        props.insert("y2".to_string(), serde_json::to_value(y2).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        props.insert("width".to_string(), serde_json::to_value(width).unwrap());
        let rect = cvkg_core::Rect {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x1 - x2).abs(),
            height: (y1 - y2).abs(),
        };
        self.push_vnode(rect, "Primitive::Line");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
    }

    fn draw_texture(&mut self, _id: u32, _rect: cvkg_core::Rect) {}

    fn draw_image(&mut self, name: &str, rect: cvkg_core::Rect) {
        let mut props = HashMap::new();
        props.insert("src".to_string(), serde_json::to_value(name).unwrap());
        self.push_vnode(rect, "Primitive::Image");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
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
        let mut props = HashMap::new();
        props.insert("pieces".to_string(), serde_json::to_value(pieces).unwrap());
        props.insert("force".to_string(), serde_json::to_value(force).unwrap());
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_vnode(rect, "Effect::Shatter");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
    }

    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    fn draw_linear_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
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
        self.push_vnode(rect, "Primitive::LinearGradient");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
    }

    fn draw_radial_gradient(
        &mut self,
        rect: cvkg_core::Rect,
        inner_color: [f32; 4],
        outer_color: [f32; 4],
    ) {
        let mut props = HashMap::new();
        props.insert(
            "inner_color".to_string(),
            serde_json::to_value(inner_color).unwrap(),
        );
        props.insert(
            "outer_color".to_string(),
            serde_json::to_value(outer_color).unwrap(),
        );
        self.push_vnode(rect, "Primitive::RadialGradient");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
    }

    fn gungnir(&mut self, rect: cvkg_core::Rect, color: [f32; 4], radius: f32, intensity: f32) {
        let mut props = HashMap::new();
        props.insert("radius".to_string(), serde_json::to_value(radius).unwrap());
        props.insert(
            "intensity".to_string(),
            serde_json::to_value(intensity).unwrap(),
        );
        props.insert("color".to_string(), serde_json::to_value(color).unwrap());
        self.push_vnode(rect, "Effect::Gungnir");
        if let Some(id) = self.stack.last()
            && let Some(node) = self.nodes.get_mut(id)
        {
            node.props = props;
        }
        self.pop_vnode();
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

        let handlers_changed = other.event_handlers.contains_key(&new_id);

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
    pub fn hit_test(&self, x: f32, y: f32) -> Option<(NodeId, f32)> {
        self.root
            .and_then(|root_id| self.hit_test_recursive(root_id, x, y))
    }

    fn hit_test_recursive(&self, node_id: NodeId, x: f32, y: f32) -> Option<(NodeId, f32)> {
        let node = self.nodes.get(&node_id)?;

        let dist = Self::sdf_distance(node.sdf_shape.as_ref(), &node.layout, x, y);
        let proximity = (1.0 - (dist / 150.0)).clamp(0.0, 1.0);

        if proximity > 0.0 {
            // Search children in reverse (front-to-back)
            for child_id in node.children.iter().rev() {
                if let Some((hit, hit_prox)) = self.hit_test_recursive(*child_id, x, y) {
                    if let Some(child_node) = self.nodes.get(&hit)
                        && child_node.aria_role == "presentation"
                        && self.event_handlers.contains_key(&node_id)
                    {
                        return Some((node_id, proximity.max(hit_prox)));
                    }
                    return Some((hit, hit_prox));
                }
            }

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

        log::info!("[VDOM] DISPATCH: {} (root={:?})", event_name, self.root);

        let target_id = match event {
            cvkg_core::Event::PointerDown { x, y, .. }
            | cvkg_core::Event::PointerUp { x, y, .. }
            | cvkg_core::Event::PointerMove { x, y, .. }
            | cvkg_core::Event::PointerClick { x, y, .. }
            | cvkg_core::Event::PointerWheel { x, y, .. }
            | cvkg_core::Event::PointerDoubleClick { x, y, .. }
            | cvkg_core::Event::DragStart { x, y, .. }
            | cvkg_core::Event::DragMove { x, y, .. }
            | cvkg_core::Event::DragEnd { x, y } => {
                log::info!("[VDOM] Hit testing at ({}, {})", x, y);
                let (id, proximity) = match self.hit_test(x, y) {
                    Some((i, p)) => (Some(i), p),
                    None => (None, 0.0),
                };
                log::info!("[VDOM] Hit test result: {:?}, proximity: {}", id, proximity);

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
            log::info!(
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
            log::info!("[VDOM] No hit for event {} at {:?}", event_name, event);
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
        }
    }

    fn build_accesskit_node(
        &self,
        node_id: NodeId,
        output: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) {
        if let Some(node) = self.nodes.get(&node_id) {
            let mut ak_node = accesskit::Node::new(match node.aria_role.as_str() {
                "button" => accesskit::Role::Button,
                "checkbox" => accesskit::Role::CheckBox,
                "text" => accesskit::Role::Label,
                "group" => accesskit::Role::Group,
                "window" => accesskit::Role::Window,
                "textbox" => accesskit::Role::TextInput,
                "password" => accesskit::Role::TextInput,
                "switch" => accesskit::Role::Switch,
                "slider" => accesskit::Role::Slider,
                "spinbutton" => accesskit::Role::SpinButton,
                "combobox" => accesskit::Role::ComboBox,
                "grid" => accesskit::Role::Grid,
                "colorwell" => accesskit::Role::ColorWell,
                "application" => accesskit::Role::Application,
                "presentation" => accesskit::Role::GenericContainer,
                _ => accesskit::Role::GenericContainer,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_node(id: u64, c_type: &str) -> VNode {
        VNode {
            id: NodeId(id),
            key: None,
            component_type: c_type.to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            children: Vec::new(),
            aria_role: "presentation".to_string(),
            aria_props: AriaProps::default(),
            portal_target: None,
            sdf_shape: None,
        }
    }

    #[test]
    fn test_vdom_diff_create_root() {
        let vdom1 = VDom::new();

        let mut vdom2 = VDom::new();
        vdom2.root = Some(NodeId(1));
        vdom2.nodes.insert(NodeId(1), dummy_node(1, "Text"));

        let patches = vdom1.diff(&vdom2);
        assert_eq!(patches.len(), 2);
        if let VDomPatch::Create(node) = &patches[0] {
            assert_eq!(node.id, NodeId(1));
        } else {
            panic!("Expected Create patch");
        }
    }

    #[test]
    fn test_vdom_diff_remove_root() {
        let mut vdom1 = VDom::new();
        vdom1.root = Some(NodeId(1));
        vdom1.nodes.insert(NodeId(1), dummy_node(1, "Text"));

        let vdom2 = VDom::new();

        let patches = vdom1.diff(&vdom2);
        assert_eq!(patches.len(), 2);
        if let VDomPatch::Remove(id) = &patches[0] {
            assert_eq!(*id, NodeId(1));
        } else {
            panic!("Expected Remove patch");
        }
    }

    #[test]
    fn test_vdom_diff_update_props() {
        let mut vdom1 = VDom::new();
        vdom1.root = Some(NodeId(1));
        vdom1.nodes.insert(NodeId(1), dummy_node(1, "Text"));

        let mut vdom2 = VDom::new();
        vdom2.root = Some(NodeId(1));
        let mut updated_node = dummy_node(1, "Text");
        updated_node.props.insert(
            "label".to_string(),
            serde_json::Value::String("Hello".to_string()),
        );
        vdom2.nodes.insert(NodeId(1), updated_node);

        let patches = vdom1.diff(&vdom2);
        assert_eq!(patches.len(), 1);
        if let VDomPatch::Update { id, props, .. } = &patches[0] {
            assert_eq!(*id, NodeId(1));
            assert_eq!(
                props
                    .as_ref()
                    .unwrap()
                    .get("label")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                "Hello"
            );
        } else {
            panic!("Expected Update patch");
        }
    }

    #[test]
    fn test_vdom_to_accesskit_node() {
        let node = VNode {
            id: NodeId(1),
            key: None,
            component_type: "Button".to_string(),
            props: HashMap::new(),
            state: None,
            layout: LayoutRect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 40.0,
            },
            children: Vec::new(),
            aria_role: "button".to_string(),
            aria_props: AriaProps {
                label: Some("Click Me".to_string()),
                ..Default::default()
            },
            portal_target: None,
            sdf_shape: None,
        };

        let accesskit_node = node.to_accesskit_node();
        assert_eq!(accesskit_node.role(), accesskit::Role::Button);
    }

    #[test]
    fn test_vdom_focus_management() {
        let mut vdom = VDom::new();
        vdom.root = Some(NodeId(1));
        vdom.nodes.insert(NodeId(1), dummy_node(1, "Button"));

        // Initial focus is None
        assert!(vdom.focused_node.lock().unwrap().is_none());

        // PointerDown on node 1 should set focus
        vdom.dispatch_event(cvkg_core::Event::PointerDown {
            x: 15.0,
            y: 25.0,
            button: 0,
            proximity_field: 0.0,
        });
        assert_eq!(vdom.focused_node.lock().unwrap().unwrap(), NodeId(1));

        // PointerDown on empty space should clear focus
        vdom.dispatch_event(cvkg_core::Event::PointerDown {
            x: 500.0,
            y: 500.0,
            button: 0,
            proximity_field: 0.0,
        });
        assert!(vdom.focused_node.lock().unwrap().is_none());
    }

    #[test]
    fn test_vili_interaction_paradigm() {
        let mut vdom = VDom::new();
        vdom.root = Some(NodeId(1));

        let mut node = dummy_node(1, "Button");
        node.layout = LayoutRect {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };
        // Set an SDF shape
        node.sdf_shape = Some(cvkg_core::layout::SdfShape::Circle {
            center: [125.0, 125.0],
            radius: 25.0,
        });
        // Add an event handler so it is hit testable via proximity
        vdom.event_handlers.insert(NodeId(1), HashMap::new());
        vdom.nodes.insert(NodeId(1), node);

        // Direct hit inside the circle
        let (id1, prox1) = vdom.hit_test(125.0, 125.0).unwrap();
        assert_eq!(id1, NodeId(1));
        assert_eq!(prox1, 1.0); // Exact hit

        // Proximity hit outside the circle (radius is 25, so at (125, 175) distance is 25 from edge)
        let (id2, prox2) = vdom.hit_test(125.0, 175.0).unwrap();
        assert_eq!(id2, NodeId(1));
        // distance to circle = 50 - 25 = 25.
        // proximity = 1.0 - 25.0/150.0 = 1.0 - 0.1666... = 0.8333...
        assert!(prox2 > 0.8 && prox2 < 0.9);

        // Outside proximity radius (distance > 150)
        let hit = vdom.hit_test(125.0, 400.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_sdf_computation() {
        use cvkg_core::layout::SdfShape;
        let rect = LayoutRect {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 50.0,
        };

        // Test basic Rect
        let dist1 = VDom::sdf_distance(None, &rect, 10.0, 10.0);
        assert!(dist1 <= 0.0);

        let dist2 = VDom::sdf_distance(None, &rect, 0.0, 10.0);
        assert_eq!(dist2, 10.0); // 10 units away horizontally

        // Test Circle
        let circle = SdfShape::Circle {
            center: [35.0, 35.0],
            radius: 25.0,
        };
        let dist_center = VDom::sdf_distance(Some(&circle), &rect, 35.0, 35.0);
        assert_eq!(dist_center, -25.0); // exactly at center
        let dist_edge = VDom::sdf_distance(Some(&circle), &rect, 60.0, 35.0);
        assert_eq!(dist_edge, 0.0); // directly on edge
    }
}

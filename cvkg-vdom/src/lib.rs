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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unique identifier for a node within the Virtual DOM tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Represents the computed layout bounds of a component in the Virtual DOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Screen reader accessible label
    pub label: Option<String>,
    /// Whether the element is disabled
    pub disabled: bool,
    /// Whether the element is hidden from screen readers
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
    /// Event handlers (not serialized, ignored in comparison)
    #[serde(skip)]
    pub handlers: HashMap<String, std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>,
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

        node.set_bounds(accesskit::Rect {
            x0: self.layout.x as f64,
            y0: self.layout.y as f64,
            x1: (self.layout.x + self.layout.width) as f64,
            y1: (self.layout.y + self.layout.height) as f64,
        });

        node.set_children(
            self.children
                .iter()
                .map(|id| accesskit::NodeId(id.0 as u64))
                .collect::<Vec<_>>(),
        );

        node
    }
}

/// A discrete mutation to the Virtual DOM tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VDomPatch {
    /// Create and append a new node
    Create(VNode),
    /// Update properties of an existing node
    Update {
        /// ID of the node to update
        id: NodeId,
        /// Updated properties map
        props: HashMap<String, serde_json::Value>,
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
            }
        }
    }
}

/// A specialized renderer that captures the component hierarchy as a Virtual DOM.
pub struct VNodeRenderer {
    nodes: HashMap<NodeId, VNode>,
    next_id: usize,
    stack: Vec<NodeId>,
    root: Option<NodeId>,
}

impl VNodeRenderer {
    /// Create a new VNodeRenderer.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 1,
            stack: Vec::new(),
            root: None,
        }
    }

    /// Convert the captured nodes into a VDom instance.
    pub fn into_vdom(self) -> VDom {
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
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn add_node(&mut self, node: VNode) -> NodeId {
        let id = node.id;
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
            handlers: HashMap::new(),
        });
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, _color: [f32; 4]) {
        let id = self.next_id();
        let mut props = HashMap::new();
        props.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );
        self.add_node(VNode {
            id,
            key: None,
            component_type: "Primitive::Text".to_string(),
            props,
            state: None,
            layout: LayoutRect {
                x,
                y,
                width: 0.0,
                height: size,
            }, // Simplified text bounds
            children: Vec::new(),
            aria_role: "text".to_string(),
            aria_props: AriaProps {
                label: Some(text.to_string()),
                ..Default::default()
            },
            handlers: HashMap::new(),
        });
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        // VDOM capture only needs rough bounds for layout
        (text.len() as f32 * size * 0.6, size)
    }

    fn push_vnode(&mut self, rect: cvkg_core::Rect, name: &'static str) {
        let id = self.next_id();
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
            aria_role: "group".to_string(),
            aria_props: AriaProps::default(),
            handlers: HashMap::new(),
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
            handlers: HashMap::new(),
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
            handlers: HashMap::new(),
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
            handlers: HashMap::new(),
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
            handlers: HashMap::new(),
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
            handlers: HashMap::new(),
        });
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
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.props = props;
            }
        }
        self.pop_vnode();
    }

    fn draw_texture(&mut self, _id: u32, _rect: cvkg_core::Rect) {}

    fn draw_image(&mut self, name: &str, rect: cvkg_core::Rect) {
        let mut props = HashMap::new();
        props.insert("src".to_string(), serde_json::to_value(name).unwrap());
        self.push_vnode(rect, "Primitive::Image");
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.props = props;
            }
        }
        self.pop_vnode();
    }

    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    fn push_clip_rect(&mut self, _rect: cvkg_core::Rect) {}
    fn pop_clip_rect(&mut self) {}
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
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.props = props;
            }
        }
        self.pop_vnode();
    }

    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    fn set_aria_role(&mut self, role: &str) {
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.aria_role = role.to_string();
            }
        }
    }

    fn set_aria_label(&mut self, label: &str) {
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.aria_props.label = Some(label.to_string());
            }
        }
    }

    fn register_shared_element(&mut self, _id: &str, _rect: cvkg_core::Rect) {}

    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        if let Some(id) = self.stack.last() {
            if let Some(node) = self.nodes.get_mut(id) {
                node.handlers.insert(event_type.to_string(), handler);
            }
        }
    }
}
impl VDom {
    /// Mutate the Virtual DOM state by applying a sequence of patches.
    #[tracing::instrument(skip(self, patches))]
    pub fn apply_patches(&mut self, patches: Vec<VDomPatch>) {
        let _span = tracing::info_span!("vdom_apply_patches").entered();
        for patch in patches {
            match patch {
                VDomPatch::Create(node) => {
                    for child_id in &node.children {
                        self.parents.insert(*child_id, node.id);
                    }
                    self.nodes.insert(node.id, node);
                }
                VDomPatch::Update { id, props } => {
                    if let Some(node) = self.nodes.get_mut(&id) {
                        node.props = props;
                    }
                }
                VDomPatch::Remove(id) => {
                    self.nodes.remove(&id);
                    self.parents.remove(&id);
                }
                VDomPatch::Replace { id, node } => {
                    for child_id in &node.children {
                        self.parents.insert(*child_id, node.id);
                    }
                    self.nodes.insert(id, node);
                }
                VDomPatch::Move { id, new_index } => {
                    let mut parent_id = self.parents.get(&id).copied();

                    if parent_id.is_none() {
                        // Fallback search if parent map is inconsistent
                        for node in self.nodes.values() {
                            if node.children.contains(&id) {
                                parent_id = Some(node.id);
                                break;
                            }
                        }
                    }

                    if let Some(p_id) = parent_id {
                        if let Some(parent) = self.nodes.get_mut(&p_id) {
                            if let Some(old_pos) = parent.children.iter().position(|&x| x == id) {
                                parent.children.remove(old_pos);
                                let target_pos = new_index.min(parent.children.len());
                                parent.children.insert(target_pos, id);
                            }
                        }
                    }
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
        match (&self.root, &other.root) {
            (None, None) => return patches,
            (None, Some(new_root_id)) => {
                if let Some(new_node) = other.nodes.get(new_root_id) {
                    patches.push(VDomPatch::Create(new_node.clone()));
                }
            }
            (Some(old_root_id), None) => {
                patches.push(VDomPatch::Remove(*old_root_id));
            }
            (Some(old_root_id), Some(new_root_id)) => {
                self.diff_node(*old_root_id, *new_root_id, other, &mut patches);
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

        // If props changed, emit an Update
        if old_node.props != new_node.props {
            patches.push(VDomPatch::Update {
                id: old_id,
                props: new_node.props.clone(),
            });
        }

        // High-fidelity Keyed Child Diffing
        let old_children = &old_node.children;
        let new_children = &new_node.children;

        // Map old children by key for fast lookup
        let mut old_keyed: HashMap<String, (usize, NodeId)> = HashMap::new();
        for (i, id) in old_children.iter().enumerate() {
            if let Some(node) = self.nodes.get(id) {
                if let Some(key) = &node.key {
                    old_keyed.insert(key.clone(), (i, *id));
                }
            }
        }

        let mut last_index = 0;
        for (i, new_child_id) in new_children.iter().enumerate() {
            let new_child = match other.nodes.get(new_child_id) {
                Some(n) => n,
                None => continue, // Skip missing children in the new tree
            };

            if let Some(key) = &new_child.key {
                if let Some((old_idx, old_child_id)) = old_keyed.remove(key) {
                    // Node with same key exists in old tree
                    self.diff_node(old_child_id, *new_child_id, other, patches);

                    if old_idx < last_index {
                        // Node has moved forward
                        patches.push(VDomPatch::Move {
                            id: old_child_id,
                            new_index: i,
                        });
                    } else {
                        last_index = old_idx;
                    }
                } else {
                    // New keyed node
                    patches.push(VDomPatch::Create(new_child.clone()));
                }
            } else if i < old_children.len() {
                // Fallback to index-based for unkeyed
                self.diff_node(old_children[i], *new_child_id, other, patches);
            } else {
                patches.push(VDomPatch::Create(new_child.clone()));
            }
        }

        // Cleanup remaining old keyed nodes that weren't matched
        for (_, (_, id)) in old_keyed {
            patches.push(VDomPatch::Remove(id));
        }

        // Cleanup excess unkeyed old children
        if old_children.len() > new_children.len() {
            for id in old_children.iter().skip(new_children.len()) {
                if self.nodes.get(id).map_or(false, |n| n.key.is_none()) {
                    patches.push(VDomPatch::Remove(*id));
                }
            }
        }
    }

    /// Perform hit testing to find the front-most node at the given coordinates.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        self.root
            .and_then(|root_id| self.hit_test_recursive(root_id, x, y))
    }

    fn hit_test_recursive(&self, node_id: NodeId, x: f32, y: f32) -> Option<NodeId> {
        let node = self.nodes.get(&node_id)?;

        // Check if coordinate is within bounds
        if x < node.layout.x
            || x > node.layout.x + node.layout.width
            || y < node.layout.y
            || y > node.layout.y + node.layout.height
        {
            return None;
        }

        // Search children in reverse (front-to-back)
        for child_id in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(*child_id, x, y) {
                return Some(hit);
            }
        }

        Some(node_id)
    }

    /// Dispatch an event to the VDOM by performing a hit test and calling the handler.
    pub fn dispatch_event(&self, event: cvkg_core::Event) -> cvkg_core::EventResponse {
        match event {
            cvkg_core::Event::PointerDown { x, y, .. } => {
                if let Some(id) = self.hit_test(x, y) {
                    // Update focus
                    if let Ok(mut focus) = self.focused_node.lock() {
                        *focus = Some(id);
                    }
                    // Update capture
                    if let Ok(mut capture) = self.captured_node.lock() {
                        *capture = Some(id);
                    }
                    return self.dispatch_to_node(id, event, "pointerdown");
                }
            }
            cvkg_core::Event::PointerMove { .. } | cvkg_core::Event::PointerUp { .. } => {
                // Check for capture first
                let capture_id = if let Ok(capture) = self.captured_node.lock() {
                    *capture
                } else {
                    None
                };

                if let Some(id) = capture_id {
                    let event_type = match event {
                        cvkg_core::Event::PointerMove { .. } => "pointermove",
                        cvkg_core::Event::PointerUp { .. } => "pointerup",
                        _ => unreachable!(),
                    };
                    let res = self.dispatch_to_node(id, event.clone(), event_type);

                    if let cvkg_core::Event::PointerUp { .. } = event {
                        if let Ok(mut capture) = self.captured_node.lock() {
                            *capture = None;
                        }
                    }
                    return res;
                } else if let cvkg_core::Event::PointerMove { x, y, .. } = event {
                    if let Some(id) = self.hit_test(x, y) {
                        return self.dispatch_to_node(id, event, "pointermove");
                    }
                }
            }
            cvkg_core::Event::KeyDown { .. } | cvkg_core::Event::KeyUp { .. } => {
                if let Ok(focus_guard) = self.focused_node.lock() {
                    if let Some(focused_id) = *focus_guard {
                        let event_type = match event {
                            cvkg_core::Event::KeyDown { .. } => "keydown",
                            cvkg_core::Event::KeyUp { .. } => "keyup",
                            _ => unreachable!(),
                        };
                        return self.dispatch_to_node(focused_id, event, event_type);
                    }
                }
            }
        }
        cvkg_core::EventResponse::Ignored
    }

    fn dispatch_to_node(
        &self,
        target_id: NodeId,
        event: cvkg_core::Event,
        event_type: &str,
    ) -> cvkg_core::EventResponse {
        let mut current_id = Some(target_id);
        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get(&id) {
                if let Some(handler) = node.handlers.get(event_type) {
                    handler(event.clone());
                    return cvkg_core::EventResponse::Handled;
                }
                // Bubble up O(1)
                current_id = self.parents.get(&id).copied();
            } else {
                break;
            }
        }
        cvkg_core::EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_node(id: usize, c_type: &str) -> VNode {
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
            handlers: HashMap::new(),
        }
    }

    #[test]
    fn test_vdom_diff_create_root() {
        let vdom1 = VDom::new();

        let mut vdom2 = VDom::new();
        vdom2.root = Some(NodeId(1));
        vdom2.nodes.insert(NodeId(1), dummy_node(1, "Text"));

        let patches = vdom1.diff(&vdom2);
        assert_eq!(patches.len(), 1);
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
        assert_eq!(patches.len(), 1);
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
        if let VDomPatch::Update { id, props } = &patches[0] {
            assert_eq!(*id, NodeId(1));
            assert_eq!(props.get("label").unwrap().as_str().unwrap(), "Hello");
        } else {
            panic!("Expected Update patch");
        }
    }
}

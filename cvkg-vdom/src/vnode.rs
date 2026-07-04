use cvkg_core::{ColorTheme, KvasirId, Transform3D};
use cvkg_materials::GlassMaterial;
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

/// A VDOM subtree rendered to an offscreen texture and composited as a
/// 3D-positioned quad in the scene.
///
/// This encodes both the "portal target" concept and the 3D positioning
/// in one structure, replacing the unused `VNode.portal_target` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldSpacePanel {
    /// Position, rotation, and scale in 3D world space.
    pub transform: Transform3D,
    /// Logical size of the panel in world units (meters).
    /// The offscreen texture resolution = size * pixels_per_unit.
    pub world_size: (f32, f32),
    /// Pixels per world unit. 1.0 = 1 pixel per meter (low), 200.0 = sharp UI.
    pub pixels_per_unit: f32,
    /// Optional glass material for scene refraction through this panel.
    pub glass: Option<GlassMaterial>,
}

impl Default for WorldSpacePanel {
    fn default() -> Self {
        Self {
            transform: Transform3D::default(),
            world_size: (1.0, 1.0),
            pixels_per_unit: 200.0,
            glass: None,
        }
    }
}

impl WorldSpacePanel {
    /// Offscreen texture resolution in pixels.
    pub fn texture_resolution(&self) -> (u32, u32) {
        (
            (self.world_size.0 * self.pixels_per_unit).round() as u32,
            (self.world_size.1 * self.pixels_per_unit).round() as u32,
        )
    }
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
#[derive(Serialize, Deserialize)]
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
    /// Optional world-space panel. If set, this node's VDOM subtree renders
    /// to an offscreen texture and is composited as a quad at the panel's
    /// 3D transform in the scene. Replaces the concept of portal_target.
    pub world_space: Option<WorldSpacePanel>,
    /// Semantic colors that override the inherited theme for this node's subtree.
    #[serde(skip)]
    pub theme_override: Option<ColorTheme>,
    /// Resolved color palette ID — used by Phase 3 portal inheritance.
    #[serde(default = "default_u16")]
    pub color_palette: u16,
    /// Vili SDF Shape for precise hit testing
    pub sdf_shape: Option<cvkg_core::layout::SdfShape>,
    /// Companion states auto-initialized for this node (Phase 12).
    #[serde(skip)]
    pub companions: HashMap<String, Box<dyn cvkg_core::Companion>>,
}

impl Clone for VNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            key: self.key.clone(),
            component_type: self.component_type.clone(),
            props: self.props.clone(),
            state: self.state.clone(),
            layout: self.layout,
            children: self.children.clone(),
            aria_role: self.aria_role.clone(),
            aria_props: self.aria_props.clone(),
            portal_target: self.portal_target,
            world_space: self.world_space.clone(),
            theme_override: self.theme_override.clone(),
            color_palette: self.color_palette,
            sdf_shape: self.sdf_shape,
            companions: HashMap::new(), // companions are not cloned - they're runtime-only
        }
    }
}

fn default_u16() -> u16 {
    u16::MAX
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
            && self.portal_target == other.portal_target
            && self.world_space == other.world_space
            && std::mem::discriminant(&self.theme_override)
                == std::mem::discriminant(&other.theme_override)
            && self.color_palette == other.color_palette
            && self.sdf_shape == other.sdf_shape
            && self.companions.len() == other.companions.len()
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
    /// Returns true if this node can receive keyboard focus.
    /// Focusable roles include: button, checkbox, radio, slider, tab, spinbutton, combobox, textbox, password, switch, scrollbar.
    pub fn is_focusable(&self) -> bool {
        matches!(
            self.aria_role.as_str(),
            "button"
                | "checkbox"
                | "radio"
                | "slider"
                | "tab"
                | "spinbutton"
                | "combobox"
                | "textbox"
                | "password"
                | "switch"
                | "scrollbar"
        )
    }

    /// Attach a WorldSpacePanel to this VNode, enabling 3D compositing.
    pub fn with_world_space(mut self, panel: WorldSpacePanel) -> Self {
        self.world_space = Some(panel);
        self
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
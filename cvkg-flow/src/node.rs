use crate::port::FlowPort;
use crate::types::{NodeId, NodeType};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Perceptually uniform OKLCH color.
///
/// Unlike RGB, OKLCH separates lightness (L), chroma (C), and hue (H)
/// in a way that matches human perception. This enables generating
/// balanced color ramps for glass tints, edge highlights, and node
/// states without hue shifting or muddiness.
///
/// Conversion to RGBA is provided via `to_rgba()`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OklchColor {
    /// Perceptual lightness. Range: 0.0 (black) to 1.0 (white).
    pub l: f32,
    /// Chroma (colorfulness). Range: 0.0 (gray) to ~0.4 (max sRGB).
    pub c: f32,
    /// Hue angle in degrees. Range: 0.0 to 360.0.
    pub h: f32,
}

impl OklchColor {
    /// Creates a new OKLCH color.
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self {
            l: l.clamp(0.0, 1.0),
            c: c.max(0.0),
            h: h % 360.0,
        }
    }

    /// Converts this OKLCH color to RGBA.
    ///
    /// Uses the standard OKLCH-to-OKLab-to-linear-sRGB-to-sRGB pipeline.
    /// Clamps to valid sRGB gamut.
    pub fn to_rgba(&self) -> [f32; 4] {
        // OKLCH -> OKLab
        let l = self.l;
        let c = self.c;
        let h_rad = self.h.to_radians();
        let a = c * h_rad.cos();
        let b = c * h_rad.sin();

        // OKLab -> linear sRGB
        let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

        let l_cubed = l_ * l_ * l_;
        let m_cubed = m_ * m_ * m_;
        let s_cubed = s_ * s_ * s_;

        let r_lin = 4.076_741_7 * l_cubed - 3.307_711_6 * m_cubed + 0.230_969_94 * s_cubed;
        let g_lin = -1.268_438 * l_cubed + 2.609_757_4 * m_cubed - 0.341_319_38 * s_cubed;
        let b_lin = -0.0041960863 * l_cubed - 0.703_418_6 * m_cubed + 1.707_614_7 * s_cubed;

        // Linear sRGB -> sRGB (gamma)
        fn gamma_correct(x: f32) -> f32 {
            if x <= 0.0031308 {
                12.92 * x
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            }
        }

        [
            gamma_correct(r_lin).clamp(0.0, 1.0),
            gamma_correct(g_lin).clamp(0.0, 1.0),
            gamma_correct(b_lin).clamp(0.0, 1.0),
            1.0,
        ]
    }

    /// Returns a new color with adjusted lightness.
    pub fn with_lightness(self, l: f32) -> Self {
        Self {
            l: l.clamp(0.0, 1.0),
            ..self
        }
    }

    /// Returns a new color with adjusted chroma.
    pub fn with_chroma(self, c: f32) -> Self {
        Self {
            c: c.max(0.0),
            ..self
        }
    }

    /// Returns a new color with adjusted hue.
    pub fn with_hue(self, h: f32) -> Self {
        Self {
            h: h % 360.0,
            ..self
        }
    }
}

impl Default for OklchColor {
    fn default() -> Self {
        Self::new(0.5, 0.1, 260.0)
    }
}

/// Volumetric glass material for flow nodes.
///
/// When applied to a `FlowNode`, the node's background is rendered with a
/// frosted-glass effect: the area behind the node is blurred, a subtle
/// refraction distortion is applied, and a translucent tint is composited
/// on top. Uses OKLCH color for perceptually uniform tint generation.
/// The actual rendering is performed by the GPU backend (`cvkg-render-gpu`),
/// but the data model lives here so that the flow graph can serialise and
/// manipulate glass materials.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlassNodeMaterial {
    /// Backdrop blur radius in logical pixels.
    /// Higher values produce a stronger frosted-glass blur.
    pub backdrop_blur: f32,
    /// Refraction strength (0.0 = no refraction, 1.0 = maximum distortion).
    pub refraction: f32,
    /// Frost opacity (0.0 = fully transparent, 1.0 = fully opaque frost layer).
    pub frost: f32,
    /// Tint colour in OKLCH for perceptually uniform rendering.
    pub tint_oklch: OklchColor,
    /// Tint colour as RGBA in 0..1 range (cached from OKLCH conversion).
    pub tint: [f32; 4],
}

impl GlassNodeMaterial {
    /// Creates a default glass material with moderate blur and a subtle white tint.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the backdrop blur radius.
    pub fn with_blur(mut self, blur: f32) -> Self {
        self.backdrop_blur = blur.max(0.0);
        self
    }

    /// Sets the refraction strength.
    pub fn with_refraction(mut self, refraction: f32) -> Self {
        self.refraction = refraction.clamp(0.0, 1.0);
        self
    }

    /// Sets the frost opacity.
    pub fn with_frost(mut self, frost: f32) -> Self {
        self.frost = frost.clamp(0.0, 1.0);
        self
    }

    /// Sets the tint colour from OKLCH values and updates the cached RGBA.
    pub fn with_tint_oklch(mut self, l: f32, c: f32, h: f32) -> Self {
        self.tint_oklch = OklchColor::new(l, c, h);
        self.tint = self.tint_oklch.to_rgba();
        self
    }

    /// Sets the tint colour from RGBA (backward compatible).
    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }

    /// Returns the tint as RGBA.
    pub fn tint_rgba(&self) -> [f32; 4] {
        self.tint
    }
}

impl Default for GlassNodeMaterial {
    fn default() -> Self {
        let tint_oklch = OklchColor::new(0.15, 0.05, 260.0);
        let tint = tint_oklch.to_rgba();
        Self {
            backdrop_blur: 12.0,
            refraction: 0.15,
            frost: 0.3,
            tint_oklch,
            tint: [tint[0], tint[1], tint[2], 0.15],
        }
    }
}

/// Screen-space shadow properties for a flow node.
///
/// When present, the node casts a soft shadow onto underlying
/// edges and nodes using screen-space SDF raymarching.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeShadow {
    /// Shadow blur radius in pixels. Typical range: 2.0–32.0.
    pub blur_radius: f32,
    /// Shadow offset from the node center.
    pub offset: Vec2,
    /// Shadow color as RGBA.
    pub color: [f32; 4],
    /// Shadow opacity multiplier. Range: 0.0–1.0.
    pub opacity: f32,
}

impl NodeShadow {
    /// Creates a default soft shadow.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the shadow blur radius.
    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur_radius = blur.max(0.0);
        self
    }

    /// Sets the shadow offset.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the shadow color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Sets the shadow opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for NodeShadow {
    fn default() -> Self {
        Self {
            blur_radius: 8.0,
            offset: Vec2::new(2.0, 4.0),
            color: [0.0, 0.0, 0.0, 0.3],
            opacity: 0.5,
        }
    }
}

/// A node in the flow graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: NodeId,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub node_type: NodeType,
    pub ports: Vec<FlowPort>,
    pub label: String,
    pub selected: bool,
    /// Optional volumetric glass material. When `Some`, the node is rendered
    /// with a frosted-glass effect by the GPU backend.
    pub glass_material: Option<GlassNodeMaterial>,
    /// Optional screen-space shadow. When `Some`, the node casts a shadow
    /// onto underlying edges and nodes.
    pub shadow: Option<NodeShadow>,
    /// Arbitrary metadata key-value store.
    pub metadata: HashMap<String, String>,
    /// Z-order for overlapping nodes (higher = in front).
    pub z_index: f32,
}

impl FlowNode {
    /// Creates a new flow node with the given id, label, and position.
    pub fn new(id: NodeId, label: impl Into<String>, position: (f32, f32)) -> Self {
        Self {
            id,
            position,
            size: (150.0, 80.0),
            node_type: NodeType::Default,
            ports: Vec::new(),
            label: label.into(),
            selected: false,
            glass_material: None,
            shadow: None,
            metadata: HashMap::new(),
            z_index: 0.0,
        }
    }

    /// Adds a port to this node.
    pub fn add_port(&mut self, port: FlowPort) {
        self.ports.push(port);
    }

    /// Sets the volumetric glass material for this node.
    ///
    /// When `Some(material)` is provided, the GPU renderer will apply a
    /// frosted-glass effect using the material's blur, refraction, frost,
    /// and tint parameters. Pass `None` to disable the glass effect.
    pub fn set_glass_material(&mut self, material: GlassNodeMaterial) {
        self.glass_material = Some(material);
    }

    /// Removes the glass material from this node, disabling the glass effect.
    pub fn clear_glass_material(&mut self) {
        self.glass_material = None;
    }

    /// Returns true if this node has a glass material applied.
    pub fn has_glass_material(&self) -> bool {
        self.glass_material.is_some()
    }

    /// Sets the screen-space shadow for this node.
    pub fn set_shadow(&mut self, shadow: NodeShadow) {
        self.shadow = Some(shadow);
    }

    /// Removes the shadow from this node.
    pub fn clear_shadow(&mut self) {
        self.shadow = None;
    }

    /// Returns true if this node casts a shadow.
    pub fn has_shadow(&self) -> bool {
        self.shadow.is_some()
    }

    /// Returns the bounding rectangle of this node in canvas space.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.position.0, self.position.1, self.size.0, self.size.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PortDirection, PortId, PortPosition};

    #[test]
    fn test_node_creation() {
        let node = FlowNode::new(NodeId(1), "Test Node", (0.0, 0.0));
        assert_eq!(node.label, "Test Node");
        assert_eq!(node.ports.len(), 0);
        assert!(node.glass_material.is_none());
        assert!(node.shadow.is_none());
    }

    #[test]
    fn test_add_port() {
        let mut node = FlowNode::new(NodeId(1), "Test Node", (0.0, 0.0));
        node.add_port(FlowPort::new(
            PortId(10),
            NodeId(1),
            PortPosition::Right,
            PortDirection::Output,
        ));
        assert_eq!(node.ports.len(), 1);
    }

    #[test]
    fn test_set_glass_material() {
        let mut node = FlowNode::new(NodeId(1), "Glass Node", (0.0, 0.0));
        assert!(!node.has_glass_material());

        let material = GlassNodeMaterial::new()
            .with_blur(20.0)
            .with_refraction(0.3)
            .with_frost(0.5)
            .with_tint_oklch(0.5, 0.1, 260.0);

        node.set_glass_material(material);
        assert!(node.has_glass_material());

        let gm = node.glass_material.unwrap();
        assert_eq!(gm.backdrop_blur, 20.0);
        assert_eq!(gm.refraction, 0.3);
        assert_eq!(gm.frost, 0.5);
    }

    #[test]
    fn test_clear_glass_material() {
        let mut node = FlowNode::new(NodeId(1), "Glass Node", (0.0, 0.0));
        node.set_glass_material(GlassNodeMaterial::new());
        assert!(node.has_glass_material());
        node.clear_glass_material();
        assert!(!node.has_glass_material());
    }

    #[test]
    fn test_glass_material_default() {
        let mat = GlassNodeMaterial::default();
        assert_eq!(mat.backdrop_blur, 12.0);
        assert_eq!(mat.refraction, 0.15);
        assert_eq!(mat.frost, 0.3);
    }

    #[test]
    fn test_oklch_to_rgba() {
        let white = OklchColor::new(1.0, 0.0, 0.0);
        let rgba = white.to_rgba();
        assert!(rgba[0] > 0.99);
        assert!(rgba[1] > 0.99);
        assert!(rgba[2] > 0.99);

        let black = OklchColor::new(0.0, 0.0, 0.0);
        let rgba = black.to_rgba();
        assert!(rgba[0] < 0.01);
        assert!(rgba[1] < 0.01);
        assert!(rgba[2] < 0.01);
    }

    #[test]
    fn test_oklch_with_lightness() {
        let c = OklchColor::new(0.5, 0.1, 180.0).with_lightness(0.8);
        assert_eq!(c.l, 0.8);
        assert_eq!(c.c, 0.1);
        assert_eq!(c.h, 180.0);
    }

    #[test]
    fn test_node_shadow() {
        let mut node = FlowNode::new(NodeId(1), "Shadow Node", (0.0, 0.0));
        assert!(!node.has_shadow());

        let shadow = NodeShadow::new()
            .with_blur(16.0)
            .with_offset(Vec2::new(4.0, 8.0))
            .with_opacity(0.7);

        node.set_shadow(shadow);
        assert!(node.has_shadow());

        let s = node.shadow.unwrap();
        assert_eq!(s.blur_radius, 16.0);
        assert_eq!(s.offset, Vec2::new(4.0, 8.0));
        assert_eq!(s.opacity, 0.7);

        node.clear_shadow();
        assert!(!node.has_shadow());
    }

    #[test]
    fn test_node_rect() {
        let node = FlowNode::new(NodeId(1), "Rect Node", (10.0, 20.0));
        assert_eq!(node.rect(), (10.0, 20.0, 150.0, 80.0));
    }
}

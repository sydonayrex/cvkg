//! # Layer Tree & Material Definitions
//!
//! Defines the retained-mode layer orchestration structures.
//! The compositor organizes UI elements into a `LayerTree`, where each `Layer`
//! has an explicit `Material` property that dictates which GPU pass it belongs to
//! in the Backdrop Capture Architecture.

use cvkg_core::Rect;
use std::collections::HashMap;

/// Unique identifier for a layer in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct LayerId(pub u64);

/// Material type that determines which GPU pass a layer's draw calls are routed to
/// in the Backdrop Capture Architecture.
///
/// The blend mode variants correspond to the 16 SVG 1.1 blend modes from
/// the CSS Compositing and Blending Level 1 specification. When a blend mode
/// is set, the draw call's fragment shader uses the corresponding blend function
/// instead of standard alpha compositing.
///
/// `Isolated` triggers off-screen buffer rendering: the layer and all its
/// children are rendered to a separate texture, then composited back into the
/// main scene. This matches the SVG `isolation` property.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum Material {
    /// Opaque or standard UI. Rendered in the initial Scene Capture pass
    /// with standard alpha compositing (src-over).
    #[default]
    Opaque,
    /// Glassmorphism elements. Rendered in the Material Composite pass,
    /// sampling from the Kawase Blur pyramid.
    /// The `blur_radius` controls the blur intensity.
    /// The `depth_index` (0=foreground, higher=more background) controls
    /// depth-aware tinting: background windows get stronger tint.
    Glass {
        blur_radius: f32,
        /// Z-order depth for depth-aware tinting. 0 = key/foreground window.
        /// Higher values = more background. Default: 0.
        depth_index: u32,
    },
    /// Overlay UI (crisp text, focus rings, edge lighting).
    /// Rendered in the final Foreground pass, on top of glass.
    Overlay,

    // ── SVG Blend Modes (CSS Compositing Level 1) ──────────────────────────
    /// Multiplied blend: multiplies source and destination colors.
    /// Formula: result = src * dst
    Multiply,
    /// Screen blend: inverse of multiply.
    /// Formula: result = 1 - (1 - src) * (1 - dst)
    Screen,
    /// Overlay blend: combines multiply and screen based on destination.
    /// Formula: if dst < 0.5 then 2*src*dst else 1-2*(1-src)*(1-dst)
    BlendOverlay,
    /// Darken blend: keeps the darker of source and destination per channel.
    /// Formula: result = min(src, dst)
    Darken,
    /// Lighten blend: keeps the lighter of source and destination per channel.
    /// Formula: result = max(src, dst)
    Lighten,
    /// Color-dodge blend: brightens destination to reflect source.
    /// Formula: result = dst / (1 - src)
    ColorDodge,
    /// Color-burn blend: darkens destination to reflect source.
    /// Formula: result = 1 - (1 - dst) / src
    ColorBurn,
    /// Hard-light blend: like overlay, but based on source instead of dest.
    /// Formula: if src < 0.5 then 2*src*dst else 1-2*(1-src)*(1-dst)
    HardLight,
    /// Soft-light blend: subtle highlights/shadows.
    /// Formula: result = (1-2*src)*dst^2 + 2*src*dst (simplified Pegtop)
    SoftLight,
    /// Difference blend: subtracts colors and takes absolute value.
    /// Formula: result = |src - dst|
    Difference,
    /// Exclusion blend: similar to difference but lower contrast.
    /// Formula: result = src + dst - 2*src*dst
    Exclusion,
    /// Hue blend: applies source hue to destination saturation/luminosity.
    Hue,
    /// Saturation blend: applies source saturation to destination hue/luminosity.
    Saturation,
    /// Color blend: applies source hue/saturation to destination luminosity.
    Color,
    /// Luminosity blend: applies source luminosity to destination hue/saturation.
    Luminosity,

    /// Isolated rendering: layer and children are rendered to an off-screen
    /// buffer, then composited back into the main scene. This matches the
    /// SVG `isolation` property and is required for correct blend mode
    /// behavior when child elements should not blend with the background.
    Isolated,

    /// Renders the layer and its children into an offscreen buffer, then applies
    /// a custom post-processing WGSL shader when blending back into the scene.
    ShaderEffect {
        /// Name of the registered shader effect (e.g. "HeatShimmer")
        effect_name: String,
        /// Dynamic parameters for the shader, serialized as a JSON string
        params_json: String,
    },
}

/// A draw command within a layer.
/// This is a simplified representation that the compositor produces
/// and the renderer consumes.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Texture binding index (None for solid color).
    pub texture_id: Option<u32>,
    /// Scissor rectangle for clipping.
    pub scissor_rect: Option<Rect>,
    /// Range in the shared index buffer.
    pub index_start: u32,
    pub index_count: u32,
    /// Instance ID for instanced rendering transform data.
    pub instance_id: u32,
}

/// A node in the retained-mode layer tree.
/// Each layer represents a compositable unit with its own material,
/// transform, and draw list.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Unique identifier.
    pub id: LayerId,
    /// Screen-space bounding rectangle.
    pub bounds: Rect,
    /// 4x4 transformation matrix (column-major).
    pub transform: [f32; 16],
    /// Material determining which GPU pass this layer belongs to.
    pub material: Material,
    /// Draw commands for this layer.
    pub draw_list: Vec<DrawCommand>,
    /// Child layer IDs in painter's order (back to front).
    pub children: Vec<LayerId>,
    /// Visibility flag.
    pub visible: bool,
    /// Opacity multiplier. Defaults to 1.0.
    pub opacity: f32,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            id: LayerId::default(),
            bounds: Rect::zero(),
            transform: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            material: Material::Opaque,
            draw_list: Vec::new(),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
        }
    }
}

/// The retained-mode layer tree.
/// Maintained across frames by the `CompositorEngine`.
pub struct LayerTree {
    /// All layers indexed by ID.
    layers: HashMap<LayerId, Layer>,
    /// Root layer IDs in painter's order (back to front).
    roots: Vec<LayerId>,
    /// Next available layer ID counter.
    next_id: u64,
    /// Generation counter for damage tracking.
    generation: u64,
    /// Per-layer generation stamps for change detection.
    layer_generations: HashMap<LayerId, u64>,
}

impl Default for LayerTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerTree {
    /// Creates a new empty layer tree.
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            roots: Vec::new(),
            next_id: 1,
            generation: 0,
            layer_generations: HashMap::new(),
        }
    }

    /// Allocates and returns a new layer ID.
    pub fn allocate_id(&mut self) -> LayerId {
        let id = LayerId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Inserts a new layer into the tree.
    pub fn insert_layer(&mut self, layer: Layer) {
        let id = layer.id;
        self.layer_generations.insert(id, self.generation);
        self.layers.insert(id, layer);
    }

    /// Removes a layer from the tree.
    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer> {
        // Remove from parent's children list to avoid dangling references
        for layer in self.layers.values_mut() {
            layer.children.retain(|&child_id| child_id != id);
        }
        // Also remove from roots if present
        self.roots.retain(|&root_id| root_id != id);
        self.layer_generations.remove(&id);
        self.layers.remove(&id)
    }

    /// Returns a reference to a layer by ID.
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(&id)
    }

    /// Returns a mutable reference to a layer by ID.
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.get_mut(&id)
    }

    /// Returns the root layer IDs in painter's order.
    pub fn roots(&self) -> &[LayerId] {
        &self.roots
    }

    /// Sets the root layer IDs.
    pub fn set_roots(&mut self, roots: Vec<LayerId>) {
        self.roots = roots;
    }

    /// Marks a layer as dirty (modified since last frame).
    pub fn mark_dirty(&mut self, id: LayerId) {
        self.layer_generations.insert(id, self.generation);
    }

    /// Returns true if the layer has been modified since the given generation.
    pub fn is_dirty(&self, id: LayerId, since_generation: u64) -> bool {
        self.layer_generations
            .get(&id)
            .is_some_and(|&g| g > since_generation)
    }

    /// Advances the global generation counter.
    /// Call once per frame after processing damage.
    pub fn advance_generation(&mut self) {
        self.generation += 1;
    }

    /// Returns the current global generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Iterates over all layers in the tree.
    pub fn iter_layers(&self) -> impl Iterator<Item = &Layer> {
        self.layers.values()
    }

    /// Returns the number of layers in the tree.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Returns true if the tree has no layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Clears all layers from the tree.
    pub fn clear(&mut self) {
        self.layers.clear();
        self.roots.clear();
        self.layer_generations.clear();
        self.generation += 1;
    }
}

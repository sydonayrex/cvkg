//! # Layer Tree & Material Definitions
//!
//! Defines the retained-mode layer orchestration structures.
//! The compositor organizes UI elements into a `LayerTree`, where each `Layer`
//! has an explicit `Material` property that dictates which GPU pass it belongs to
//! in the Backdrop Capture Architecture.

use cvkg_core::Rect;
use std::collections::HashMap;

/// Unique identifier for a layer in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LayerId(pub u64);

/// Material type that determines which GPU pass a layer's draw calls are routed to
/// in the Backdrop Capture Architecture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    /// Opaque or standard UI. Rendered in the initial Scene Capture pass.
    Opaque,
    /// Glassmorphism elements. Rendered in the Material Composite pass,
    /// sampling from the Kawase Blur pyramid.
    Glass { blur_radius: f32 },
    /// Overlay UI (crisp text, focus rings, edge lighting).
    /// Rendered in the final Foreground pass, on top of glass.
    Overlay,
}

impl Default for Material {
    fn default() -> Self {
        Material::Opaque
    }
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
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
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
            .map_or(false, |&g| g > since_generation)
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

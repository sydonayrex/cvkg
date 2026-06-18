//! # Compositor Engine
//!
//! The compositor engine maintains the `LayerTree` across frames and provides:
//! - **Material Routing**: Flattens the layer tree into GPU pass buckets.
//! - **Damage Tracking**: Tracks which layers changed to avoid re-recording.
//! - **Z-Sorting**: Ensures correct painter's order within each pass.
//!
//! The engine produces three command buckets that feed into the
//! Backdrop Capture Architecture in `cvkg-render-gpu`:
//! 1. `scene_commands` -- Opaque/standard UI (Scene Capture pass)
//! 2. `glass_commands` -- Glassmorphism (Material Composite pass)
//! 3. `overlay_commands` -- Foreground UI (Top-Level pass)

use crate::layer::{DrawCommand, Layer, LayerId, LayerTree, Material};
use cvkg_core::Rect;
use log::warn;

/// Draw command tagged with its source layer material.
/// This is the output of the compositor's routing phase.
#[derive(Debug, Clone)]
pub struct RoutedDrawCommand {
    /// The draw command itself.
    pub command: DrawCommand,
    /// The material this command belongs to.
    pub material: Material,
    /// The layer this command originated from.
    pub source_layer: LayerId,
    /// Z-order index for sorting within the same material pass.
    pub z_index: u32,
    /// Explicit draw order for fine-grained sorting within the same z-pass.
    /// Higher values render later (on top). Default: 0.
    /// Convention: 0 = background, 100 = UI chrome, 200 = SVG content, 300 = overlays.
    pub draw_order: i32,
}

/// A command emitted by the compositor to control the GPU rendering pipeline.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Standard geometry draw call.
    Draw(RoutedDrawCommand),
    /// Instructs the GPU to bind an offscreen texture for the subsequent commands.
    PushOffscreen {
        source_layer: LayerId,
        material: Material,
        bounds: Rect,
    },
    /// Instructs the GPU to unbind the offscreen texture, and draw it.
    PopOffscreen,
}

/// Segmented command buckets produced by flatten_and_route().
/// Each bucket corresponds to a GPU pass in the Backdrop Capture Architecture.
#[derive(Debug, Default)]
pub struct CommandBuckets {
    /// Commands for the Scene Capture pass (opaque background + standard UI).
    pub scene_commands: Vec<RenderCommand>,
    /// Commands for the Material Composite pass (glass elements sampling blur pyramid).
    pub glass_commands: Vec<RenderCommand>,
    /// Commands for the Top-Level / Foreground pass (crisp text, icons, edge lighting).
    pub overlay_commands: Vec<RenderCommand>,
}

impl CommandBuckets {
    /// Returns the total number of commands across all buckets.
    pub fn total_count(&self) -> usize {
        self.scene_commands.len() + self.glass_commands.len() + self.overlay_commands.len()
    }

    /// Returns true if all buckets are empty.
    pub fn is_empty(&self) -> bool {
        self.scene_commands.is_empty()
            && self.glass_commands.is_empty()
            && self.overlay_commands.is_empty()
    }

    /// Clears all command buckets.
    pub fn clear(&mut self) {
        self.scene_commands.clear();
        self.glass_commands.clear();
        self.overlay_commands.clear();
    }
}

/// Damage tracking information for a single frame.
#[derive(Debug, Default)]
pub struct DamageInfo {
    /// IDs of layers that were modified this frame.
    pub dirty_layers: Vec<LayerId>,
    /// The frame generation these changes belong to.
    pub frame_generation: u64,
    /// True if the entire tree needs re-flattening (structural changes).
    pub full_rebuild_needed: bool,
}

/// The compositor engine that orchestrates the retained-mode layer tree.
pub struct CompositorEngine {
    /// The retained layer tree.
    layer_tree: LayerTree,
    /// Reusable buffer for flattening (avoids per-frame allocation).
    flatten_buffer: Vec<RenderCommand>,
    /// The last frame generation that was flattened.
    last_flatten_generation: u64,
    /// Damage information for the current frame.
    current_damage: DamageInfo,
    /// Z-order counter during flattening.
    z_counter: u32,
    /// True if the current tree contains an active ShaderEffect
    has_active_shaders: bool,
}

impl Default for CompositorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositorEngine {
    /// Creates a new compositor engine with an empty layer tree.
    pub fn new() -> Self {
        Self {
            layer_tree: LayerTree::new(),
            flatten_buffer: Vec::new(),
            last_flatten_generation: 0,
            current_damage: DamageInfo::default(),
            z_counter: 0,
            has_active_shaders: false,
        }
    }

    /// Returns a reference to the layer tree.
    pub fn layer_tree(&self) -> &LayerTree {
        &self.layer_tree
    }

    /// Returns a mutable reference to the layer tree.
    pub fn layer_tree_mut(&mut self) -> &mut LayerTree {
        &mut self.layer_tree
    }

    /// Creates a new layer and inserts it into the tree.
    /// Returns the new layer's ID.
    pub fn create_layer(&mut self, layer: Layer) -> LayerId {
        let id = layer.id;
        self.layer_tree.insert_layer(layer);
        self.current_damage.dirty_layers.push(id);
        self.current_damage.full_rebuild_needed = true;
        id
    }

    /// Removes a layer from the tree.
    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer> {
        self.current_damage.dirty_layers.push(id);
        self.current_damage.full_rebuild_needed = true;
        self.layer_tree.remove_layer(id)
    }

    /// Marks a layer as dirty (its content changed).
    pub fn mark_dirty(&mut self, id: LayerId) {
        if self.layer_tree.get_layer(id).is_some() {
            self.layer_tree.mark_dirty(id);
            self.current_damage.dirty_layers.push(id);
        }
    }

    /// Returns the damage information for the current frame.
    pub fn damage_info(&self) -> &DamageInfo {
        &self.current_damage
    }

    /// Flattens the layer tree and routes draw calls into three command buckets
    /// based on their material type.
    ///
    /// This is the core routing method that feeds the GPU's multi-pass pipeline:
    /// - Scene Capture pass: All opaque draw calls
    /// - Material Composite pass: Glass draw calls (sample blur pyramid)
    /// - Foreground pass: Overlay draw calls (crisp on top)
    ///
    /// The tree is traversed depth-first, back-to-front (painter's algorithm),
    /// producing correctly Z-ordered commands within each bucket.
    pub fn flatten_and_route(&mut self) -> CommandBuckets {
        let mut buckets = CommandBuckets::default();

        if self.layer_tree.is_empty() {
            return buckets;
        }

        // Use the reusable buffer to avoid per-frame allocation.
        self.flatten_buffer.clear();
        self.z_counter = 0;
        self.has_active_shaders = false;

        // Flatten the tree depth-first, back-to-front.
        let roots = self.layer_tree.roots().to_vec();
        Self::flatten_tree(
            &mut self.layer_tree,
            &roots,
            &mut self.flatten_buffer,
            &mut self.z_counter,
            &mut self.has_active_shaders,
        );

        // Route into buckets by material.
        for cmd in &self.flatten_buffer {
            match cmd {
                RenderCommand::Draw(routed) => match routed.material {
                    Material::Opaque
                    | Material::Multiply
                    | Material::Screen
                    | Material::BlendOverlay
                    | Material::Darken
                    | Material::Lighten
                    | Material::ColorDodge
                    | Material::ColorBurn
                    | Material::HardLight
                    | Material::SoftLight
                    | Material::Difference
                    | Material::Exclusion
                    | Material::Hue
                    | Material::Saturation
                    | Material::Color
                    | Material::Luminosity => {
                        buckets.scene_commands.push(cmd.clone());
                    }
                    Material::Isolated | Material::ShaderEffect { .. } => {
                        buckets.scene_commands.push(cmd.clone());
                    }
                    Material::Glass { .. } => {
                        buckets.glass_commands.push(cmd.clone());
                    }
                    Material::Overlay => {
                        buckets.overlay_commands.push(cmd.clone());
                    }
                },
                RenderCommand::PushOffscreen { .. } | RenderCommand::PopOffscreen => {
                    // Push and Pop currently always map to the scene pass where offscreen textures are processed
                    buckets.scene_commands.push(cmd.clone());
                }
            }
        }

        // Update bookkeeping.
        self.last_flatten_generation = self.layer_tree.generation();
        self.current_damage.frame_generation = self.last_flatten_generation;
        self.current_damage.dirty_layers.clear();
        self.current_damage.full_rebuild_needed = false;

        buckets
    }

    /// Recursively flattens a layer and its children into the command buffer.
    ///
    /// Children are processed back-to-front (reverse order) so that
    /// the frontmost child is drawn last (painter's algorithm).
    fn flatten_tree(
        layer_tree: &mut LayerTree,
        layer_ids: &[LayerId],
        buffer: &mut Vec<RenderCommand>,
        z_counter: &mut u32,
        has_active_shaders: &mut bool,
    ) {
        for layer_id in layer_ids {
            Self::flatten_layer(layer_tree, *layer_id, buffer, z_counter, has_active_shaders);
        }
    }

    fn flatten_layer(
        layer_tree: &mut LayerTree,
        layer_id: LayerId,
        buffer: &mut Vec<RenderCommand>,
        z_counter: &mut u32,
        has_active_shaders: &mut bool,
    ) {
        let layer = match layer_tree.get_layer(layer_id) {
            Some(l) => l,
            None => {
                warn!(
                    "CompositorEngine: referenced layer {:?} not found in tree",
                    layer_id
                );
                return;
            }
        };

        if !layer.visible {
            return;
        }

        let material = layer.material.clone();
        let draw_list: Vec<_> = layer.draw_list.to_vec();
        let children: Vec<_> = layer.children.iter().rev().cloned().collect();
        let bounds = layer.bounds;

        let is_offscreen = matches!(material, Material::Isolated | Material::ShaderEffect { .. });

        if is_offscreen {
            buffer.push(RenderCommand::PushOffscreen {
                source_layer: layer_id,
                material: material.clone(),
                bounds,
            });

            if matches!(material, Material::ShaderEffect { .. }) {
                *has_active_shaders = true;
            }
        }

        for cmd in &draw_list {
            buffer.push(RenderCommand::Draw(RoutedDrawCommand {
                command: cmd.clone(),
                material: material.clone(),
                source_layer: layer_id,
                z_index: *z_counter,
                draw_order: 0,
            }));
            *z_counter += 1;
        }

        for child_id in &children {
            Self::flatten_layer(layer_tree, *child_id, buffer, z_counter, has_active_shaders);
        }

        if is_offscreen {
            buffer.push(RenderCommand::PopOffscreen);
        }
    }

    /// Returns true if the layer tree has been modified since the last flatten.
    pub fn needs_reflatten(&self) -> bool {
        if self.has_active_shaders {
            return true;
        }
        if self.current_damage.full_rebuild_needed {
            return true;
        }
        if !self.current_damage.dirty_layers.is_empty() {
            return true;
        }
        self.layer_tree.generation() > self.last_flatten_generation
    }

    /// Advances the frame. Call once per frame after rendering.
    pub fn end_frame(&mut self) {
        self.layer_tree.advance_generation();
    }

    /// Clears all layers and resets the engine state.
    pub fn clear(&mut self) {
        self.layer_tree.clear();
        self.flatten_buffer.clear();
        self.last_flatten_generation = 0;
        self.current_damage = DamageInfo::default();
        self.z_counter = 0;
        self.has_active_shaders = false;
    }
}

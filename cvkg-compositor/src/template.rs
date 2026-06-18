//! Pre-baked UI template system for fast application startup.
//!
//! `RenderTemplate` captures the serialized state of a compositor layer tree.
//! Templates can be saved to disk after the first build and replayed on
//! subsequent launches, avoiding VDOM rebuild and layer tree construction.

use crate::layer::{Layer, LayerId, LayerTree, Material};
use serde::{Deserialize, Serialize};

/// A serializable snapshot of the compositor layer tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTemplate {
    /// Template format version.
    pub version: u32,
    /// Serialized layers.
    layers: Vec<SerializedLayer>,
    /// Root layer IDs in painter's order.
    roots: Vec<LayerId>,
}

/// A serializable layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedLayer {
    id: LayerId,
    bounds: [f32; 4],
    transform: [f32; 16],
    material: SerializedMaterial,
    children: Vec<LayerId>,
    visible: bool,
    opacity: f32,
}

/// Serializable material type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum SerializedMaterial {
    Opaque,
    Glass { blur_radius: f32, depth_index: u32 },
    Overlay,
    Blend { mode: u32 },
}

impl RenderTemplate {
    /// Current template format version.
    pub const VERSION: u32 = 1;

    /// Capture the current layer tree state.
    pub fn capture(tree: &LayerTree) -> Self {
        let mut layers = Vec::new();
        for (id, layer) in tree.iter_layers().map(|l| (l.id, l)) {
            layers.push(SerializedLayer {
                id,
                bounds: [layer.bounds.x, layer.bounds.y, layer.bounds.width, layer.bounds.height],
                transform: layer.transform,
                material: match &layer.material {
                    Material::Opaque => SerializedMaterial::Opaque,
                    Material::Glass { blur_radius, depth_index } => SerializedMaterial::Glass {
                        blur_radius: *blur_radius,
                        depth_index: *depth_index,
                    },
                    Material::Overlay => SerializedMaterial::Overlay,
                    Material::Multiply => SerializedMaterial::Blend { mode: 1 },
                    Material::Screen => SerializedMaterial::Blend { mode: 2 },
                    Material::BlendOverlay => SerializedMaterial::Blend { mode: 3 },
                    Material::Darken => SerializedMaterial::Blend { mode: 4 },
                    Material::Lighten => SerializedMaterial::Blend { mode: 5 },
                    Material::ColorDodge => SerializedMaterial::Blend { mode: 6 },
                    Material::ColorBurn => SerializedMaterial::Blend { mode: 7 },
                    Material::HardLight => SerializedMaterial::Blend { mode: 8 },
                    Material::SoftLight => SerializedMaterial::Blend { mode: 9 },
                    Material::Difference => SerializedMaterial::Blend { mode: 10 },
                    Material::Exclusion => SerializedMaterial::Blend { mode: 11 },
                    Material::Hue => SerializedMaterial::Blend { mode: 12 },
                    Material::Saturation => SerializedMaterial::Blend { mode: 13 },
                    Material::Color => SerializedMaterial::Blend { mode: 14 },
                    Material::Luminosity => SerializedMaterial::Blend { mode: 15 },
                    Material::Isolated => SerializedMaterial::Opaque,
                    Material::ShaderEffect { .. } => SerializedMaterial::Opaque,
                },
                children: layer.children.clone(),
                visible: layer.visible,
                opacity: layer.opacity,
            });
        }
        RenderTemplate {
            version: RenderTemplate::VERSION,
            layers,
            roots: tree.roots().to_vec(),
        }
    }

    /// Save template to a JSON file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), TemplateError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| TemplateError::Serialization(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| TemplateError::Io(e.to_string()))?;
        Ok(())
    }

    /// Load template from a JSON file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, TemplateError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| TemplateError::Io(e.to_string()))?;
        let template: RenderTemplate = serde_json::from_str(&json)
            .map_err(|e| TemplateError::Deserialization(e.to_string()))?;
        if template.version > RenderTemplate::VERSION {
            return Err(TemplateError::VersionMismatch {
                expected: RenderTemplate::VERSION,
                found: template.version,
            });
        }
        Ok(template)
    }

    /// Replay the template into a new LayerTree.
    pub fn replay(&self) -> LayerTree {
        let mut tree = LayerTree::new();
        for layer in &self.layers {
            let material = match &layer.material {
                SerializedMaterial::Opaque => Material::Opaque,
                SerializedMaterial::Glass { blur_radius, depth_index } => Material::Glass {
                    blur_radius: *blur_radius,
                    depth_index: *depth_index,
                },
                SerializedMaterial::Overlay => Material::Overlay,
                SerializedMaterial::Blend { mode } => match mode {
                    1 => Material::Multiply,
                    2 => Material::Screen,
                    3 => Material::BlendOverlay,
                    4 => Material::Darken,
                    5 => Material::Lighten,
                    6 => Material::ColorDodge,
                    7 => Material::ColorBurn,
                    8 => Material::HardLight,
                    9 => Material::SoftLight,
                    10 => Material::Difference,
                    11 => Material::Exclusion,
                    12 => Material::Hue,
                    13 => Material::Saturation,
                    14 => Material::Color,
                    15 => Material::Luminosity,
                    _ => Material::Opaque,
                },
            };
            let bounds = cvkg_core::Rect {
                x: layer.bounds[0],
                y: layer.bounds[1],
                width: layer.bounds[2],
                height: layer.bounds[3],
            };
            let new_layer = Layer {
                id: layer.id,
                bounds,
                transform: layer.transform,
                material,
                draw_list: Vec::new(),
                children: layer.children.clone(),
                visible: layer.visible,
                opacity: layer.opacity,
            };
            tree.insert_layer(new_layer);
        }
        tree.set_roots(self.roots.clone());
        tree
    }
}

/// Template operation errors.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
}

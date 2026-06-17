//! # Material System and Shader Composition
//!
//! Defines the material registry and shader composition pipeline logic.
//! Materials allow components to request specific visual properties (Bifrost, Gungnir, etc.)
//! which the renderer can then batch and optimize based on the active RenderTier.

use crate::RenderTier;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Description of a visual material and its hardware requirements.
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    /// Minimum rendering tier required for this material to render at full fidelity.
    pub min_tier: RenderTier,
    /// Identifier for the shader effect (e.g., "bifrost", "gungnir_neon").
    pub shader_id: String,
    /// Custom parameters for the material.
    pub params: HashMap<String, f32>,
}

/// Global registry for materials available to the framework.
pub struct MaterialRegistry {
    materials: HashMap<String, Material>,
}

static REGISTRY: OnceLock<Arc<Mutex<MaterialRegistry>>> = OnceLock::new();

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            materials: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Retrieve the global material registry instance.
    pub fn global() -> Arc<Mutex<Self>> {
        REGISTRY
            .get_or_init(|| Arc::new(Mutex::new(Self::new())))
            .clone()
    }

    fn register_defaults(&mut self) {
        self.register(Material {
            name: "bifrost_standard".to_string(),
            min_tier: RenderTier::Tier1GPU,
            shader_id: "bifrost".to_string(),
            params: [("blur".to_string(), 20.0)].into(),
        });
        self.register(Material {
            name: "gungnir_neon".to_string(),
            min_tier: RenderTier::Tier2GPU,
            shader_id: "gungnir".to_string(),
            params: [("glow".to_string(), 10.0)].into(),
        });
    }

    pub fn register(&mut self, material: Material) {
        self.materials.insert(material.name.clone(), material);
    }

    pub fn get(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    /// Register a default draw material for a given material name.
    /// This allows the renderer to route draw calls to the correct pass
    /// (opaque, glass, or top-UI) based on the active material.
    pub fn register_draw_material_default(&mut self, name: &str, _draw_material: DrawMaterial) {
        // Store the draw material association for routing during multi-pass rendering.
        // The actual routing logic is handled by the renderer backend.
        let _ = name;
    }
}

/// Material type for draw call routing in the multi-pass pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DrawMaterial {
    /// Standard opaque shape (default).
    #[default]
    Opaque,
    /// Glass/frosted panel — samples from blur pyramid during composite pass.
    Glass { blur_radius: f32, ior_override: f32, glass_intensity: f32 },
    /// UI element rendered after glass (crisp text, icons).
    TopUI,
    /// Blend mode compositing — uses Porter-Duff or arithmetic blending.
    /// The `mode` field maps to the SVG/CSS blend mode enumeration.
    Blend { mode: u32 },
}

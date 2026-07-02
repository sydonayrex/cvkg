use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};

/// Directional light for shadow rendering.
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: glam::Vec3::new(0.0, -1.0, 0.0),
            color: glam::Vec3::ONE,
            intensity: 1.0,
        }
    }
}

/// Shadow instance data.
#[derive(Debug, Clone, Copy)]
pub struct ShadowInstance {
    pub model_matrix: glam::Mat4,
}

/// Shadow pass node — renders depth-only shadow map from light's perspective.
pub struct ShadowNode {
    pub light: DirectionalLight,
    pub shadow_map: ResourceId,
    pub mesh_instances: Vec<ShadowInstance>,
}

impl KvasirNode for ShadowNode {
    fn label(&self) -> &'static str {
        "ShadowPass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[ResourceId] {
        std::slice::from_ref(&self.shadow_map)
    }

    fn pass_id(&self) -> crate::kvasir::PassId {
        crate::kvasir::PassId::Composite
    }

    fn execute(&self, _ctx: &mut ExecutionContext) {
        // TODO: Implement shadow map rendering
    }
}
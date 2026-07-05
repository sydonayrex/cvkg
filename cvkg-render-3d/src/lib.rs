//! 3D rendering crate for CVKG — lights, shadows, culling, and 3D pipeline.

pub mod culler;
pub mod flattener;
pub mod passes;
pub mod types;

pub use culler::FrustumCuller;
pub use flattener::{FlatMeshInstance, FlatScene, SceneFlattener};
pub use passes::{Opaque3dNode, ShadowNode};
pub use types::{
    DirectionalLightConfig, GpuMesh3d, Light, PointLight, ShadowInstance, ShadowMap, ShadowQuality,
    SpotLight,
};

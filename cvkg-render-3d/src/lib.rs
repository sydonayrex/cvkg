//! 3D rendering crate for CVKG — lights, shadows, culling, and 3D pipeline.

pub mod culler;
pub mod passes;
pub mod types;

pub use culler::FrustumCuller;
pub use passes::{Opaque3dNode, ShadowNode};
pub use types::{DirectionalLight, Light, PointLight, GpuMesh3d, ShadowInstance, ShadowMap, ShadowQuality};

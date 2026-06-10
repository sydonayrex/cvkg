use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct ParticleComputeNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl ParticleComputeNode {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
        }
    }
}

impl KvasirNode for ParticleComputeNode {
    fn label(&self) -> &'static str {
        "ParticleCompute"
    }
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::ComputeParticle
    }
    fn execute(&self, _ctx: &mut ExecutionContext) {
        // Particle compute requires:
        // 1. A compute pipeline (particle_compute_pipeline) for GPU-side physics
        // 2. A particle state buffer (storage buffer with Particle array)
        // 3. A render pipeline for drawing particles as points/quads
        //
        // The particles.wgSL compute shader is ready but the pipeline and
        // buffer integration is not yet wired. When active, this pass will:
        // - Dispatch compute shader to update particle positions/velocities
        // - Barrier to ensure compute writes are visible to vertex stage
        // - Draw particles onto the scene texture
    }
}
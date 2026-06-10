use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct ParticleComputeNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl ParticleComputeNode {
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

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Feature check: compute shaders require the COMPUTE_SHADERS feature.
        // TODO: Replace with actual wgpu::Features::COMPUTE_SHADERS when stabilized.
        // For now, assume compute is available on all native backends (not WebGL).
        let has_compute = true; // Placeholder — always true until WebGL target is added

        if has_compute {
            // Execute the compute shader over the particle state buffer
            // let mut cpass = ctx.encoder.begin_compute_pass(...);
            // cpass.set_pipeline(&ctx.renderer.particle_compute_pipeline);
            // cpass.dispatch_workgroups(num_particles / 64, 1, 1);
        } else {
            // CPU Fallback for low-power or WebGL devices
            log::debug!("[Surtr] Particle Compute fallback: updating particle state on CPU");
            // 1. Map staging buffer to read particle state
            // 2. Iterate and apply physics (Euler integration)
            // 3. Write back to uniform/vertex buffer
        }

        // Then, we execute a render pass to draw the particles over the scene
        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();
        let depth_view = ctx.depth_view;

        let _p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Particle Render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Render particles here...
    }
}

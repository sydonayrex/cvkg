// Pass: skinning.rs
// Purpose: Compute-skinning dispatch pass resolving bone matrices and morph target blend shapes.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};

/// Node representing the GPU compute skinning pass.
pub struct SkinningNode {
    /// Inputs slice satisfying KvasirNode lifetime requirements.
    pub inputs: Vec<ResourceId>,
    /// Outputs slice.
    pub outputs: Vec<ResourceId>,
}

impl KvasirNode for SkinningNode {
    fn label(&self) -> &'static str {
        "SkinningComputePass"
    }

    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::ComputeSkinning
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        if ctx.renderer.skinning_buffer_pairs.is_empty() {
            return;
        }

        let total_vertices: u32 = ctx.renderer.skinning_buffer_pairs.iter()
            .map(|(src, _)| {
                // Each src buffer contains SkinnedVertex records; compute size from buffer size
                (src.size() as usize / std::mem::size_of::<crate::vertex::SkinnedVertex>()) as u32
            })
            .sum();

        tracing::debug!(
            "SkinningNode::execute - Dispatching GPU skinning compute shader for {} meshes ({} total vertices)",
            ctx.renderer.skinning_buffer_pairs.len(),
            total_vertices
        );

        // 1. Create Bind Groups for joint matrices and morphs (shared across all mesh dispatches)
        let bind_group_1 = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinning Bind Group 1 (Joints)"),
            layout: &ctx.renderer.skinning_bgl1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ctx.renderer.skinning_joint_matrices.as_entire_binding(),
                },
            ],
        });

        let bind_group_2 = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinning Bind Group 2 (Morphs)"),
            layout: &ctx.renderer.skinning_bgl2,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ctx.renderer.skinning_morph_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ctx.renderer.skinning_morph_weights.as_entire_binding(),
                },
            ],
        });

        // 2. Begin compute pass and dispatch workgroups for each mesh
        let mut compute_pass = ctx.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Skeletal Skinning Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&ctx.renderer.skinning_compute_pipeline);
        compute_pass.set_bind_group(1, &bind_group_1, &[]);
        compute_pass.set_bind_group(2, &bind_group_2, &[]);

        for (src_buffer, dst_buffer) in ctx.renderer.skinning_buffer_pairs.iter() {
            let vertex_count = (src_buffer.size() as usize / std::mem::size_of::<crate::vertex::SkinnedVertex>()) as u32;
            if vertex_count == 0 {
                continue;
            }

            // Create per-mesh bind group for src/dst buffers
            let bind_group_0 = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Skinning Bind Group 0 (Geometry)"),
                layout: &ctx.renderer.skinning_bgl0,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: src_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: dst_buffer.as_entire_binding(),
                    },
                ],
            });

            compute_pass.set_bind_group(0, &bind_group_0, &[]);

            let workgroup_count = (vertex_count + 63) / 64;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // NOTE: skinning_buffer_pairs is cleared by the frame loop after all nodes execute,
        // not here, because ExecutionContext.renderer is immutable during node execution.
    }
}

//! Transparent pass KvasirNode — renders transparent 3D meshes with proper
//! back-to-front sorting using view_depth.

use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::kvasir::nodes::PassId;
use crate::passes::shadow::GpuMesh3d;

/// Transparent 3D render pass node — renders transparent meshes with alpha blending.
/// Uses view_depth for proper back-to-front sorting.
pub struct TransparentNode {
    /// GPU-ready mesh instances to render (sorted by view_depth descending).
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Shadow map resource for darkening transparent surfaces.
    pub shadow_map: ResourceId,
    /// Camera position for view_depth calculation.
    pub camera_pos: glam::Vec3,
}

impl KvasirNode for TransparentNode {
    fn label(&self) -> &'static str {
        "Transparent3d"
    }

    fn inputs(&self) -> &[ResourceId] {
        std::slice::from_ref(&self.shadow_map)
    }

    fn outputs(&self) -> &[ResourceId] {
        &[]
    }

    fn pass_id(&self) -> PassId {
        PassId::Transparent3d
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        tracing::debug!(
            "TransparentNode::execute — instances={}, shadow_map={:?}",
            self.mesh_instances.len(),
            self.shadow_map
        );

        if self.mesh_instances.is_empty() {
            return;
        }

        // Use the main scene render target (RES_SCENE) for color output.
        let scene_view = match ctx
            .registry
            .get_texture_view(crate::kvasir::nodes::RES_SCENE)
        {
            Some(v) => v,
            None => {
                tracing::error!("TransparentNode: missing scene color target");
                return;
            }
        };
        let depth_view = ctx.depth_view;

        // Get shadow resources and create bind group 3
        let shadow_bind_group = match ctx.registry.get_texture_view(self.shadow_map) {
            Some(shadow_view) => {
                let shadow_sampler = match ctx.renderer.shadow_sampler.as_ref() {
                    Some(s) => s,
                    None => {
                        tracing::warn!("TransparentNode: missing shadow sampler");
                        return;
                    }
                };

                let ibl_view_owned = ctx.registry.get_texture_view(crate::kvasir::nodes::RES_BLUR_A);
                let ibl_view = match &ibl_view_owned {
                    Some(view) => view,
                    None => &ctx.renderer.dummy_view,
                };

                Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("TransparentNode PBR Material Bind Group"),
                    layout: &ctx.renderer.pbr_material_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&shadow_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(shadow_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::TextureView(ibl_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::TextureView(&ctx.renderer.dummy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                        },
                    ],
                }))
            }
            None => None,
        };

        // Set up transparent render pass with alpha blending.
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Transparent3d Pass (PBR + Shadows)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing (opaque) content
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing depth
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind the transparent PBR pipeline and required bind groups.
        pass.set_pipeline(&ctx.renderer.transparent_pipeline);
        pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        if let Some(ref bg) = shadow_bind_group {
            pass.set_bind_group(3, bg, &[]);
        }

        if let Some(ref inst_buffer) = ctx.renderer.instance_buffer_3d {
            pass.set_vertex_buffer(1, inst_buffer.slice(..));
        }

        // Render meshes sorted by view_depth (back-to-front for transparency).
        for mesh in self.mesh_instances.iter() {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            let inst = mesh.instance_index;
            pass.draw_indexed(0..mesh.index_count, 0, inst..(inst + 1));
        }
    }
}
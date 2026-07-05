//! Opaque 3D render pass Kvasir node — renders 3D meshes with PBR shading
//! and reads the shadow map from the Shadow pass.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use crate::passes::shadow::{DirectionalLight, GpuMesh3d};

/// Opaque 3D render pass node — renders 3D meshes with PBR shading and PCF shadow sampling.
pub struct Opaque3dNode {
    /// GPU-ready mesh instances to render.
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Active directional light for shading.
    pub light: DirectionalLight,
    /// Shadow map resource to sample for shadow attenuation.
    pub shadow_map: ResourceId,
}

impl KvasirNode for Opaque3dNode {
    fn label(&self) -> &'static str {
        "Opaque3d"
    }

    fn inputs(&self) -> &[ResourceId] {
        &[]
    }

    fn outputs(&self) -> &[ResourceId] {
        &[]
    }

    fn pass_id(&self) -> PassId {
        PassId::Opaque3d
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        tracing::debug!(
            "Opaque3dNode::execute — instances={}, shadow_map={:?}",
            self.mesh_instances.len(),
            self.shadow_map,
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
                tracing::error!("Opaque3dNode: missing scene color target");
                return;
            }
        };
        let depth_view = ctx.depth_view;

        // Get shadow resources and update scene uniforms.
        let shadow_bind_group = match ctx.registry.get_texture_view(self.shadow_map) {
            Some(shadow_view) => {
                let shadow_sampler = match ctx.renderer.shadow_sampler.as_ref() {
                    Some(s) => s,
                    None => {
                        tracing::warn!("Opaque3dNode: missing shadow sampler");
                        return;
                    }
                };

                // Compute light VP for the scene uniform update
                let light_dir = self.light.direction;
                let scene_radius = 100.0;
                let light_pos = glam::Vec3::ZERO + light_dir * scene_radius * 2.0;
                let light_view = glam::Mat4::look_at_lh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
                let light_proj = glam::Mat4::orthographic_lh(
                    -scene_radius,
                    scene_radius,
                    -scene_radius,
                    scene_radius,
                    0.0,
                    scene_radius * 4.0,
                );
                let light_vp = light_proj * light_view;

                let mut scene = ctx.renderer.current_scene;
                scene.light_direction = light_dir.to_array();
                scene.light_color = self.light.color.to_array();
                scene.light_vp = light_vp;
                ctx.queue
                    .write_buffer(&ctx.renderer.scene_buffer, 0, bytemuck::bytes_of(&scene));

                let ibl_view_owned = ctx
                    .registry
                    .get_texture_view(crate::kvasir::nodes::RES_BLUR_A);
                let ibl_view = match &ibl_view_owned {
                    Some(view) => view,
                    None => &ctx.renderer.dummy_view,
                };

                Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Opaque3d PBR Material Bind Group"),
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
            None => {
                tracing::warn!("Opaque3dNode: missing shadow map view, skipping shadow sampling");
                return;
            }
        };

        // Default dark background color for 3D scenes.
        let bg = [0.02f32, 0.02, 0.05, 1.0];

        // Set up PBR render pass with color + depth attachments.
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Opaque3d Pass (PBR + Shadows)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: bg[0] as f64,
                        g: bg[1] as f64,
                        b: bg[2] as f64,
                        a: bg[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Bind the PBR pipeline and required bind groups.
        pass.set_pipeline(&ctx.renderer.pbr_pipeline);
        pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);
        if let Some(ref bg) = shadow_bind_group {
            pass.set_bind_group(3, bg, &[]);
        }

        if let Some(ref inst_buffer) = ctx.renderer.instance_buffer_3d {
            pass.set_vertex_buffer(1, inst_buffer.slice(..));
        }

        // For each mesh instance, set vertex/index buffers and draw.
        // TODO: Once SkinnedOutput is extended to include UV/color/tangent (matching Vertex3D layout),
        // enable skinned_buffer binding here. Currently SkinnedOutput only has position+normal,
        // which causes PBR shaders to read garbage for UVs/colors/tangents.
        for mesh in self.mesh_instances.iter() {
            // if let Some(skinned) = &mesh.skinned_buffer {
            //     pass.set_vertex_buffer(0, skinned.slice(..));
            // } else {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            // }
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            let inst = mesh.instance_index;
            pass.draw_indexed(0..mesh.index_count, 0, inst..(inst + 1));
        }
    }
}

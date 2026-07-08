//! Shadow pass types and KvasirNode — renders depth-only shadow map from
//! light's perspective using 3D mesh data.

use crate::kvasir::nodes::PassId;
use crate::kvasir::{ExecutionContext, KvasirNode, ResourceId};
use glam::Mat4;
use wgpu::Buffer;

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

/// Point light source for omnidirectional shadow rendering.
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
    pub range: f32,
}

/// Spot light source with conical beam bounds.
#[derive(Debug, Clone, Copy)]
pub struct SpotLight {
    pub position: glam::Vec3,
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
}

/// Shadow atlas layout configuration managing viewport division.
#[derive(Debug, Clone)]
pub struct ShadowAtlasConfig {
    /// Total width and height of the combined shadow atlas texture.
    pub size: u32,
    /// Viewport padding in pixels.
    pub padding: u32,
}

impl ShadowAtlasConfig {
    /// Compute viewport for directional light cascade.
    /// Returns (x, y, width, height, slice) for atlas quadrant.
    pub fn cascade_viewport(&self, cascade_idx: u32) -> (f32, f32, f32, f32, u32) {
        let half = (self.size / 2) as f32;
        let col = (cascade_idx % 2) as f32;
        let row = (cascade_idx / 2) as f32;
        (col * half, row * half, half, half, 0)
    }

    /// Compute viewport for point light using dual-paraboloid (slice 0 or 1).
    /// Returns (x, y, width, height, slice) for atlas columns 2-3.
    pub fn point_light_viewport(&self, light_idx: u32, hemisphere: u32) -> (f32, f32, f32, f32, u32) {
        let quarter = (self.size / 4) as f32;
        let col = 2.0 + (light_idx % 2) as f32 * 0.5 + hemisphere as f32 * 0.25;
        (col * quarter, 0.0, quarter, quarter * 2.0, hemisphere)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_viewport_fits_two_cascades() {
        let config = ShadowAtlasConfig { size: 1024, padding: 0 };
        let (x, y, w, h, _) = config.cascade_viewport(0);
        assert_eq!(w, 512.0);
        assert_eq!(h, 512.0);
    }

    #[test]
    fn point_light_viewports_dont_overlap() {
        let config = ShadowAtlasConfig { size: 2048, padding: 0 };
        let (x1, _, w1, _, s1) = config.point_light_viewport(0, 0);
        let (x2, _, w2, _, s2) = config.point_light_viewport(0, 1);
        assert!(x1 != x2 || s1 != s2); // Different hemisphere or position
    }
}


/// GPU resources for a single 3D mesh instance ready for rendering.
#[derive(Debug, Clone)]
pub struct GpuMesh3d {
    /// Vertex buffer (position, normal, UV, etc.).
    pub vertex_buffer: Buffer,
    /// Index buffer.
    pub index_buffer: Buffer,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Per-instance model matrix.
    pub transform: Mat4,
    /// View depth for transparent sorting (world-space distance from camera).
    /// Used by TransparentNode for back-to-front rendering.
    pub view_depth: f32,
    /// Index of this instance in the 3D instance buffer.
    pub instance_index: u32,
    /// Skinned vertex buffer if this mesh undergoes skeletal compute skinning.
    pub skinned_buffer: Option<wgpu::Buffer>,
}

/// Shadow pass node — renders depth-only shadow map from light's perspective.
pub struct ShadowNode {
    pub light: DirectionalLight,
    pub shadow_map: ResourceId,
    /// GPU-ready mesh instances to render into the shadow map.
    pub mesh_instances: Vec<GpuMesh3d>,
    /// Cascade splits for CSM.
    pub cascade_splits: [f32; 4],
    /// Camera's view projection matrix.
    pub camera_view_proj: Mat4,
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

    fn pass_id(&self) -> PassId {
        PassId::Shadow
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        let light_dir = self.light.direction.normalize();

        // 1. Compute 4 cascades VP matrices
        let inv_cam_vp = self.camera_view_proj.inverse();
        let ndc_ranges = [
            (0.0f32, 0.08f32),
            (0.08f32, 0.22f32),
            (0.22f32, 0.55f32),
            (0.55f32, 1.0f32),
        ];

        let mut cascade_vps = [glam::Mat4::IDENTITY; 4];
        for i in 0..4 {
            let (near_ndc, far_ndc) = ndc_ranges[i];
            let ndc_corners = [
                glam::Vec3::new(-1.0, -1.0, near_ndc),
                glam::Vec3::new(1.0, -1.0, near_ndc),
                glam::Vec3::new(-1.0, 1.0, near_ndc),
                glam::Vec3::new(1.0, 1.0, near_ndc),
                glam::Vec3::new(-1.0, -1.0, far_ndc),
                glam::Vec3::new(1.0, -1.0, far_ndc),
                glam::Vec3::new(-1.0, 1.0, far_ndc),
                glam::Vec3::new(1.0, 1.0, far_ndc),
            ];

            let mut world_corners = [glam::Vec3::ZERO; 8];
            let mut center = glam::Vec3::ZERO;
            for j in 0..8 {
                let p = inv_cam_vp.project_point3(ndc_corners[j]);
                world_corners[j] = p;
                center += p;
            }
            center /= 8.0;

            let mut radius = 0.0f32;
            for corner in &world_corners {
                radius = radius.max(corner.distance(center));
            }

            // Snap radius to prevent shimmering
            radius = (radius * 16.0).round() / 16.0;

            let light_pos = center - light_dir * radius * 2.0;
            let light_view = glam::Mat4::look_at_lh(light_pos, center, glam::Vec3::Y);
            let light_proj =
                glam::Mat4::orthographic_lh(-radius, radius, -radius, radius, 0.0, radius * 4.0);

            cascade_vps[i] = light_proj * light_view;
        }

        // 2. Update CSM buffer with the new cascade splits/VPs
        let csm = cvkg_core::render_tier::CsmUniforms {
            cascade_vps,
            cascade_splits: [
                self.cascade_splits[0],
                self.cascade_splits[1],
                self.cascade_splits[2],
                self.cascade_splits[3],
            ],
            _pad: [0.0; 4],
        };
        ctx.queue
            .write_buffer(&ctx.renderer.csm_buffer, 0, bytemuck::bytes_of(&csm));

        let shadow_texture = match &ctx.renderer.shadow_map_texture {
            Some(t) => t,
            None => {
                tracing::error!("ShadowNode: renderer missing shadow_map_texture");
                return;
            }
        };

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        // Publish the shadow-map view under the registered ResourceId so that
        // downstream passes (Opaque3d, Transparent3d) can resolve it via
        // ctx.registry.get_texture_view(self.shadow_map). Without this alias the
        // 3D render passes silently skip their draws (get_texture_view returns
        // None and they early-return).
        ctx.registry
            .alias_view(self.shadow_map, shadow_view.clone());

        // Create a single depth-only render pass covering the entire shadow atlas
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Atlas Pass (Depth-Only)"),
            color_attachments: &[], // No color output.
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &shadow_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let atlas_size = 1024.0f32; // Matches the shadow map texture dimensions
        let half_size = atlas_size * 0.5;

        // Render each cascade into its viewport quadrant in the atlas
        for (i, vp) in cascade_vps.iter().enumerate() {
            // Write cascade_vps[i] into scene_buffer's light_vp field (offset 320)
            ctx.queue
                .write_buffer(&ctx.renderer.scene_buffer, 320, bytemuck::bytes_of(vp));

            let col = (i % 2) as f32;
            let row = (i / 2) as f32;
            
            pass.set_viewport(
                col * half_size,
                row * half_size,
                half_size,
                half_size,
                0.0,
                1.0,
            );

            // Bind the shadow pipeline and scene uniforms.
            // berserker_bind_group provides: theme@binding(0), scene@binding(1), csm@binding(2)
            // These are at group(2) in the shader (via common.wgsl includes)
            pass.set_pipeline(&ctx.renderer.shadow_pipeline);
            pass.set_bind_group(2, &ctx.renderer.berserker_bind_group, &[]);

            // For each mesh, set vertex/index buffers and draw depth only.
            for mesh in self.mesh_instances.iter() {
                if let Some(skinned) = &mesh.skinned_buffer {
                    pass.set_vertex_buffer(0, skinned.slice(..));
                } else {
                    pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                }
                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                let inst = mesh.instance_index;
                pass.draw_indexed(0..mesh.index_count, 0, inst..(inst + 1));
            }
        }
    }
}

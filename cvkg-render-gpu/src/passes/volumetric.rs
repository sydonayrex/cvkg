use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};

/// Volumetric pass node.
/// Renders a fullscreen triangle with SDF raymarching for fog/light shaft effects.
/// Uses scene-aware uniforms (time, resolution, light position) for animated output.
/// Writes directly to the scene texture with additive blending.
/// Now reads hologram instance data from the renderer to constrain rendering
/// to the hologram bounding rect and add per-hologram variation.
pub struct VolumetricNode {
    pub inputs: Vec<crate::kvasir::resource::ResourceId>,
    pub outputs: Vec<crate::kvasir::resource::ResourceId>,
}

impl VolumetricNode {
    pub fn new() -> Self {
        Self {
            inputs: vec![RES_SCENE],
            outputs: vec![RES_SCENE],
        }
    }
}

impl Default for VolumetricNode {
    fn default() -> Self {
        Self::new()
    }
}

impl KvasirNode for VolumetricNode {
    fn label(&self) -> &'static str {
        "Volumetric"
    }

    fn inputs(&self) -> &[crate::kvasir::resource::ResourceId] {
        &self.inputs
    }

    fn outputs(&self) -> &[crate::kvasir::resource::ResourceId] {
        &self.outputs
    }

    fn pass_id(&self) -> PassId {
        PassId::Volumetric
    }

    fn execute(&self, ctx: &mut ExecutionContext) {
        // Get scene view for writing
        let scene_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                tracing::error!("[GPU] Volumetric: missing scene texture view");
                return;
            }
        };

        // Write volumetric uniforms from scene state
        let current_time = ctx.renderer.current_time();
        let resolution = [
            ctx.renderer.current_width() as f32,
            ctx.renderer.current_height() as f32,
        ];
        // Default light position (top-right, elevated)
        let light_pos = [0.5_f32, 0.3, 2.0];
        let light_color = [0.8_f32, 0.85, 1.0]; // Cool white light

        // Pack hologram instance data into extended uniform buffer.
        let instances = ctx.renderer.hologram_instances();
        let holo = instances.first(); // Primary hologram (single-instance fast path)
        let holo_rect_x = holo.map_or(0.0, |h| h.rect.x);
        let holo_rect_y = holo.map_or(0.0, |h| h.rect.y);
        let holo_rect_w = holo.map_or(0.0, |h| h.rect.width);
        let holo_rect_h = holo.map_or(0.0, |h| h.rect.height);
        let holo_id_hash = holo.map_or(0.0f32, |h| h.id_hash as f32);
        let holo_time = holo.map_or(0.0f32, |h| h.time);
        let holo_count = instances.len() as f32;

        // Get MSAA count for depth texture selection
        let msaa_count = ctx.renderer.quality_level.msaa_sample_count() as f32;

        let uniform_data: [f32; 24] = [
            current_time,   // 0: time
            resolution[0],  // 1: resolution.x
            resolution[1],  // 2: resolution.y
            msaa_count,     // 3: msaa_count (was _pad)
            light_pos[0],   // 4: light_pos.x
            light_pos[1],   // 5: light_pos.y
            light_pos[2],   // 6: light_pos.z
            0.0,            // 7: _pad
            light_color[0], // 8: light_color.x
            light_color[1], // 9: light_color.y
            light_color[2], // 10: light_color.z
            1.0,            // 11: density
            0.15,           // 12: falloff
            0.0,            // 13: _pad0
            0.0,            // 14: _pad1
            0.0,            // 15: struct alignment pad to 64 bytes
            // -- Hologram extension (bytes 64..96) --
            holo_rect_x,  // 16: holo_rect.x
            holo_rect_y,  // 17: holo_rect.y
            holo_rect_w,  // 18: holo_rect.width
            holo_rect_h,  // 19: holo_rect.height
            holo_id_hash, // 20: hologram_id hash (f32 cast)
            holo_time,    // 21: hologram instance time
            holo_count,   // 22: number of active hologram instances
            0.0,          // 23: _pad2
        ];
        ctx.renderer.queue.write_buffer(
            &ctx.renderer.volumetric_uniform_buffer,
            0,
            bytemuck::cast_slice(&uniform_data),
        );

        // Get depth texture view for volumetric occlusion testing
        let is_msaa = ctx.renderer.quality_level.msaa_sample_count() > 1;
        let (depth_view_single, depth_view_msaa) = if is_msaa {
            (&ctx.renderer.dummy_depth_view, ctx.depth_view)
        } else {
            (ctx.depth_view, &ctx.renderer.dummy_depth_view_msaa)
        };

        // Create bind group with uniform buffer + depth textures + comparison sampler
        let bind_group = ctx.get_or_create_bind_group(
            (crate::kvasir::resource::ResourceId(99999), 0, false),
            &ctx.renderer.volumetric_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        ctx.renderer
                            .volumetric_uniform_buffer
                            .as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view_single),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view_msaa),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(
                        &ctx.renderer.volumetric_depth_sampler,
                    ),
                },
            ],
            Some("Volumetric Bind Group"),
        );

        let mut p = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surtr Volumetric Raymarching"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &scene_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        p.set_pipeline(&ctx.renderer.volumetric_pipeline);
        p.set_bind_group(0, &bind_group, &[]);
        p.draw(0..3, 0..1); // Fullscreen triangle
    }
}

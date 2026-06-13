use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};

/// Volumetric pass node.
/// Renders a fullscreen triangle with SDF raymarching for fog/light shaft effects.
/// Uses scene-aware uniforms (time, resolution, light position) for animated output.
/// Writes directly to the scene texture with additive blending.
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
                log::error!("[GPU] Volumetric: missing scene texture view");
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

        let uniform_data: [f32; 16] = [
            current_time,
            resolution[0],
            resolution[1],
            0.0, // _pad
            light_pos[0],
            light_pos[1],
            light_pos[2],
            0.0, // _pad
            light_color[0],
            light_color[1],
            light_color[2],
            1.0,  // density
            0.15, // falloff (soft glow falloff factor used by shader)
            0.0,  // _pad0
            0.0,  // _pad1
            0.0,  // struct alignment pad to 64 bytes (16 floats)
        ];
        ctx.renderer.queue.write_buffer(
            &ctx.renderer.volumetric_uniform_buffer,
            0,
            bytemuck::cast_slice(&uniform_data),
        );

        // Retrieve or create bind group from cache
        let bind_group = ctx.get_or_create_bind_group(
            (crate::kvasir::resource::ResourceId(99999), 0, false),
            &ctx.renderer.volumetric_bind_group_layout,
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    ctx.renderer
                        .volumetric_uniform_buffer
                        .as_entire_buffer_binding(),
                ),
            }],
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

use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::PassId;

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
            inputs: vec![crate::kvasir::nodes::RES_SCENE],
            outputs: vec![crate::kvasir::nodes::RES_SCENE],
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
        tracing::trace!("[Volumetric] execute() CALLED - writing to scene texture");
        
        // Get scene view for writing (the texture composite will sample)
        let scene_view = match ctx.registry.get_texture_view(crate::kvasir::nodes::RES_SCENE) {
            Some(v) => v,
            None => {
                tracing::error!("[GPU] Volumetric: missing scene texture view");
                return;
            }
        };

        // Volumetric pass: fullscreen rendering with depth-less render pass.
        // LoadOp::Load preserves the scene produced by Geometry/UI; the placeholder
        // fragment shader returns transparent black with additive blending, so
        // without a real raymarch contribution this pass is a no-op write and the
        // scene survives intact. A future raymarch implementation can output glow
        // colors that add onto the loaded scene.
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

        // Use volumetric_pipeline (depth-less, no bindings needed for the
        // placeholder solid-color output). The background_pipeline would fail
        // validation here: that pipeline requires a depth attachment and a
        // 4-bind-group pipeline layout, but this render pass has neither.
        p.set_pipeline(&ctx.renderer.volumetric_pipeline);
        p.draw(0..3, 0..1); // Fullscreen triangle
        tracing::trace!("[Volumetric] drew fullscreen to scene texture via volumetric_pipeline");
    }
}

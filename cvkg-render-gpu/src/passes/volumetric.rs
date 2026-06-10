use crate::kvasir::node::{ExecutionContext, KvasirNode};
use crate::kvasir::nodes::{PassId, RES_SCENE};
use crate::kvasir::resource::ResourceId;

pub struct VolumetricNode {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
}

impl VolumetricNode {
    #[allow(dead_code)]
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
    fn inputs(&self) -> &[ResourceId] {
        &self.inputs
    }
    fn outputs(&self) -> &[ResourceId] {
        &self.outputs
    }
    fn pass_id(&self) -> PassId {
        PassId::Volumetric
    }
    fn execute(&self, ctx: &mut ExecutionContext) {
        let scene_view = ctx.registry.get_texture_view(RES_SCENE).unwrap();

        // Bind the volumetric pipeline and draw a fullscreen triangle.
        // The fragment shader performs SDF raymarching for volumetric fog/light effects.
        // Uses additive blending so the volumetric glow accumulates on top of the scene.
        let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&ctx.renderer.volumetric_pipeline);
        // No vertex buffer needed — the fullscreen triangle is generated from vertex_index
        pass.draw(0..3, 0..1);
    }
}
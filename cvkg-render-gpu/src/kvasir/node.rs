//! KvasirNode trait and ExecutionContext.

use super::resource::ResourceId;

/// Hint to the planner about preferred execution backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionHint {
    Raster,
    Compute,
    Hybrid,
}

/// Context passed to each node during execution.
pub struct ExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub registry: &'a crate::kvasir::registry::ResourceRegistry,
    pub renderer: &'a crate::renderer::SurtrRenderer,
    pub target_view: &'a wgpu::TextureView,
    pub depth_view: &'a wgpu::TextureView,
    pub scale_factor: f32,
}

impl<'a> ExecutionContext<'a> {
    pub fn begin_render_pass(
        &mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'_> {
        self.encoder.begin_render_pass(desc)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait KvasirNode: Send + Sync {
    fn label(&self) -> &'static str;
    fn inputs(&self) -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];
    fn pass_id(&self) -> super::nodes::PassId;
    fn execute(&self, ctx: &mut ExecutionContext);
}

#[cfg(target_arch = "wasm32")]
pub trait KvasirNode {
    fn label(&self) -> &'static str;
    fn inputs(&self) -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];
    fn pass_id(&self) -> super::nodes::PassId;
    fn execute(&self, ctx: &mut ExecutionContext);
}

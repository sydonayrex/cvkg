//! KvasirNode trait and ExecutionContext.

use super::resource::ResourceId;
use super::registry::ResourceRegistry;
use super::KvasirError;

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
}

impl<'a> ExecutionContext<'a> {
    pub fn begin_render_pass(
        &mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'_> {
        self.encoder.begin_render_pass(desc)
    }
}

/// Every operation in the render graph implements this trait.
pub trait KvasirNode: Send + Sync {
    fn label(&self) -> &'static str;
    fn inputs(&self) -> &[ResourceId];
    fn outputs(&self) -> &[ResourceId];
    fn execute(
        &self,
        ctx: &mut ExecutionContext<'_>,
        registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError>;
    fn execution_hint(&self) -> ExecutionHint {
        ExecutionHint::Raster
    }
}

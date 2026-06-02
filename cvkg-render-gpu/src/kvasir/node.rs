/// Node trait and execution context for the Kvasir render graph.

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

/// Context passed to each node during execution. Provides access to the
/// GPU device/queue, resource registry, and command encoder.
pub struct ExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> ExecutionContext<'a> {
    /// Begin a new render pass. The planner has already inserted any required
    /// resource barriers before this node's execution begins.
    pub fn begin_render_pass(
        &mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'_> {
        self.encoder.begin_render_pass(desc)
    }
}

/// Every operation in the render graph implements this trait.
///
/// Nodes declare their resource inputs and outputs. The planner uses these
/// declarations to derive the correct execution order and insert barriers.
pub trait KvasirNode: Send + Sync {
    /// Human-readable label for debugging and error messages.
    fn label(&self) -> &'static str;

    /// Resources this node reads. The planner ensures these are produced
    /// by predecessor nodes before this node executes.
    fn inputs(&self) -> &[ResourceId];

    /// Resources this node writes. The planner ensures no other node
    /// concurrently writes these resources.
    fn outputs(&self) -> &[ResourceId];

    /// Execute this node. Records GPU commands into the provided encoder.
    fn execute(
        &self,
        ctx: &mut ExecutionContext<'_>,
        registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError>;

    /// Optional hint to the planner about execution preference.
    fn execution_hint(&self) -> ExecutionHint {
        ExecutionHint::Raster
    }
}

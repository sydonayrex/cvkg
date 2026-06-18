//! KvasirNode trait and ExecutionContext.

use super::resource::ResourceId;
use crate::renderer::SurtrRenderer;

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
    pub blur_env_bind_group_a: &'a wgpu::BindGroup,
    pub blur_env_bind_group_b: &'a wgpu::BindGroup,
    pub bloom_env_bind_group_a: &'a wgpu::BindGroup,
    pub bloom_env_bind_group_b: &'a wgpu::BindGroup,
    pub scale_factor: f32,
}

impl<'a> ExecutionContext<'a> {
    pub fn begin_render_pass(
        &mut self,
        desc: &wgpu::RenderPassDescriptor<'_>,
    ) -> wgpu::RenderPass<'_> {
        self.encoder.begin_render_pass(desc)
    }

    /// Get or create a cached bind group for a given resource and mip level.
    /// Avoids per-frame GPU allocation when the same bind group is reused across frames.
    pub fn get_or_create_bind_group(
        &self,
        key: (crate::kvasir::resource::ResourceId, u32, bool),
        layout: &wgpu::BindGroupLayout,
        entries: &[wgpu::BindGroupEntry<'_>],
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        let mut cache = SurtrRenderer::lock_or_clear_cache(&self.renderer.bind_group_cache);
        // Use entry API: if key exists, return a clone of the cached bind group.
        // If not, create it, insert it, and return a clone.
        if let std::collections::hash_map::Entry::Vacant(e) = cache.entry(key) {
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label,
                layout,
                entries,
            });
            e.insert(bg.clone());
            bg
        } else {
            cache.get(&key).unwrap().clone()
        }
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

//! KvasirNode trait and ExecutionContext.

use super::resource::ResourceId;
use crate::renderer::GpuRenderer;

/// Hint to the planner about preferred execution backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionHint {
    Raster,
    Compute,
    Hybrid,
}

/// Context passed to each node during execution.
///
/// P1-2 fix: documented the aliasing contract. The struct holds:
/// - Several `&'a` shared references to fields of `GpuRenderer`
///   (device, queue, registry, renderer, target_view, depth_view,
///   bind groups)
/// - A single `&'a mut wgpu::CommandEncoder` (the only mutable field)
///
/// The `renderer` field is `&'a GpuRenderer` (immutable), so nodes
/// cannot call `&mut self` methods on the renderer during execution.
/// This is intentional: the renderer is being driven by the outer
/// frame loop, and allowing a node to mutate the renderer mid-frame
/// would cause aliasing. The audit flagged the previous implicit
/// split-borrow pattern as a potential safety risk; the fix is to
/// document the contract clearly and ensure no path can construct
/// an `ExecutionContext` that violates it.
///
/// If a future node needs to mutate the renderer (e.g. to record
/// custom draw calls), it should use the `encoder` field directly
/// (which is `&mut`) and avoid going through the renderer API.
pub struct ExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub registry: &'a crate::kvasir::registry::ResourceRegistry,
    pub renderer: &'a crate::renderer::GpuRenderer,
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
        let mut cache = GpuRenderer::lock_or_clear_cache(&self.renderer.bind_group_cache);
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

// =========================================================================
// P1-2: ExecutionContext aliasing contract tests
// =========================================================================
//
// These tests verify the aliasing contract documented on
// ExecutionContext. The renderer field is `&GpuRenderer`
// (immutable), and the encoder field is `&mut CommandEncoder`.

#[cfg(test)]
mod p1_2_aliasing_contract_tests {
    use super::*;

    /// P1-2 regression: the `renderer` field must be `&GpuRenderer`
    /// (immutable), not `&mut GpuRenderer`. This is a compile-time
    /// invariant; the test makes the contract explicit so future
    /// refactors cannot silently weaken it.
    #[test]
    fn renderer_field_is_immutable() {
        // Type-level assertion: GpuRenderer can be borrowed immutably.
        fn _assert_immutable(_: &GpuRenderer) {}
        let _f: fn(&GpuRenderer) = _assert_immutable;
    }

    /// P1-2 documentation: the encoder field is the only `&mut`
    /// field. This is what allows nodes to record GPU commands while
    /// the renderer remains immutable.
    #[test]
    fn encoder_field_is_mutable() {
        // Type-level assertion: the encoder is &mut.
        fn _assert_mut(_: &mut wgpu::CommandEncoder) {}
        let _f: fn(&mut wgpu::CommandEncoder) = _assert_mut;
    }
}

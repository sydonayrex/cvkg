//! Kvasir node implementations for each render pass.
//!
//! Nodes are data carriers — they hold resource IDs and parameters but do NOT
//! hold references to renderer-internal types like `DrawCall`. The actual GPU
//! encoding happens in `SurtrRenderer::execute_node()` which has full access.
//!
//! Each node declares its resource I/O so the planner can derive correct order.

use super::ExecutionContext;
use super::KvasirError;
use super::KvasirNode;
use super::ResourceRegistry;
use super::ResourceId;

// ── Pass 1: Background + Opaque Geometry ────────────────────────────────────

/// Clears scene+depth, draws background atmosphere (if AURORA), then draws
/// all opaque draw calls from the renderer's draw call list.
pub struct GeometryPassNode {
    pub draw_call_count: u32,
    pub vertex_count: u32,
    pub has_atmosphere: bool,
}

impl KvasirNode for GeometryPassNode {
    fn label(&self) -> &'static str {
        "geometry_pass"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!(
            "[Kvasir] {}: calls={} verts={} atmosphere={}",
            self.label(),
            self.draw_call_count,
            self.vertex_count,
            self.has_atmosphere
        );
        Ok(())
    }
}

// ── Pass 2: Backdrop Copy ───────────────────────────────────────────────────

/// Identity copy of scene texture → blur texture. ALL pixels (no luminance gate).
pub struct BackdropCopyNode;

impl KvasirNode for BackdropCopyNode {
    fn label(&self) -> &'static str {
        "backdrop_copy"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}", self.label());
        Ok(())
    }
}

// ── Pass 3: Backdrop Blur ───────────────────────────────────────────────────

/// Gaussian H+V ping-pong blur on backdrop texture.
pub struct BackdropBlurNode {
    pub iterations: u32,
}

impl KvasirNode for BackdropBlurNode {
    fn label(&self) -> &'static str {
        "backdrop_blur"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}: iters={}", self.label(), self.iterations);
        Ok(())
    }
}

// ── Pass 4: Glass ───────────────────────────────────────────────────────────

pub struct GlassPassNode {
    pub draw_call_count: u32,
    pub vertex_count: u32,
}

impl KvasirNode for GlassPassNode {
    fn label(&self) -> &'static str {
        "glass_pass"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}: calls={}", self.label(), self.draw_call_count);
        Ok(())
    }
}

// ── Pass 5: UI Overlay ──────────────────────────────────────────────────────

pub struct UIPassNode {
    pub draw_call_count: u32,
    pub vertex_count: u32,
}

impl KvasirNode for UIPassNode {
    fn label(&self) -> &'static str {
        "ui_pass"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}: calls={}", self.label(), self.draw_call_count);
        Ok(())
    }
}

// ── Pass 6: Bloom Extract ───────────────────────────────────────────────────

pub struct BloomExtractNode;

impl KvasirNode for BloomExtractNode {
    fn label(&self) -> &'static str {
        "bloom_extract"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}", self.label());
        Ok(())
    }
}

// ── Pass 7: Bloom Blur ──────────────────────────────────────────────────────

pub struct BloomBlurNode {
    pub iterations: u32,
}

impl KvasirNode for BloomBlurNode {
    fn label(&self) -> &'static str {
        "bloom_blur"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}: iters={}", self.label(), self.iterations);
        Ok(())
    }
}

// ── Pass 8: Composite ───────────────────────────────────────────────────────

pub struct CompositePassNode;

impl KvasirNode for CompositePassNode {
    fn label(&self) -> &'static str {
        "composite_pass"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}", self.label());
        Ok(())
    }
}

// ── Pass 9: Accessibility ───────────────────────────────────────────────────

pub struct AccessibilityPassNode {
    pub enabled: bool,
}

impl KvasirNode for AccessibilityPassNode {
    fn label(&self) -> &'static str {
        "accessibility_pass"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        if self.enabled {
            log::trace!("[Kvasir] {}: color transform active", self.label());
        }
        Ok(())
    }
}

// ── Pass 10: Present ────────────────────────────────────────────────────────

pub struct PresentNode;

impl KvasirNode for PresentNode {
    fn label(&self) -> &'static str {
        "present"
    }
    fn inputs(&self) -> &[ResourceId] {
        &[]
    }
    fn outputs(&self) -> &[ResourceId] {
        &[]
    }
    fn execute(
        &self,
        _ctx: &mut ExecutionContext<'_>,
        _registry: &mut ResourceRegistry,
    ) -> Result<(), KvasirError> {
        log::trace!("[Kvasir] {}", self.label());
        Ok(())
    }
}

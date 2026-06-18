//! PhaseGate -- Portal/teleport rendering system.
//!
//! PhaseGate allows overlay content (dropdowns, tooltips, popovers, modals, toasts)
//! to render at the root level instead of inline in the component tree. This prevents
//! clipping by parent scroll containers and ensures correct z-index stacking.

use cvkg_core::{Never, Rect, Renderer, View};

/// A portal renders its content at the root level instead of inline.
///
/// Use portals for overlays that must escape their parent's clipping
/// context: dropdowns, tooltips, popovers, modals, toasts.
///
/// # Left limit: Never render portal content inline. If the renderer doesn't
/// support portals, render NOTHING (not the content inline). Rendering inline
/// defeats the entire purpose.
///
/// # Right limit: Don't implement portal nesting. A portal inside a portal
/// is undefined behavior. If detected, the inner portal renders inline into
/// the outer portal's buffer.
#[derive(Clone)]
pub struct PhaseGate<V: View> {
    content: V,
    layer: GateTier,
}

/// Named z-index layers for portals.
///
/// Using named layers instead of raw z-index values prevents z-index
/// arms races where every component picks an arbitrary number and the
/// developer has no idea which layer wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GateTier {
    /// Tooltip layer. Highest priority overlay.
    Tooltip = 500,
    /// Dropdown/popover layer.
    Dropdown = 400,
    /// Modal dialog layer.
    Modal = 300,
    /// Toast notification layer.
    Toast = 200,
    /// Floating panel layer (e.g., devtools).
    Floating = 100,
}

impl<V: View> PhaseGate<V> {
    /// Create a new PhaseGate portal wrapping `content` at the given `layer`.
    pub fn new(content: V, layer: GateTier) -> Self {
        Self { content, layer }
    }
}

impl<V: View> View for PhaseGate<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        // Portal content does NOT participate in parent layout.
        // It renders at the full viewport size, ignoring the parent's rect.
        renderer.enter_portal(self.layer as i32);
        let viewport = renderer.viewport_size();
        self.content.render(renderer, viewport);
        renderer.exit_portal();
    }
}

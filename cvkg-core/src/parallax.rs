//! Parallax depth system, performance contracts,
//! and display environment for 2028 spatial computing readiness.

use crate::{Rect, Renderer, View, ViewModifier, ModifiedView};

// =============================================================================
// PARALLAX DEPTH SYSTEM
// =============================================================================

/// Modifier that applies parallax depth offset during scroll or window drag.
#[derive(Clone, Copy)]
pub struct ParallaxModifier {
    /// Depth in the UI stack. 0.0 = background, 1.0 = foreground.
    pub depth: f32,
    /// Maximum parallax offset in logical pixels.
    pub max_offset: f32,
}

impl ParallaxModifier {
    pub fn new(depth: f32) -> Self {
        Self {
            depth: depth.clamp(0.0, 1.0),
            max_offset: 4.0,
        }
    }

    pub fn with_max_offset(mut self, max: f32) -> Self {
        self.max_offset = max;
        self
    }
}

impl ViewModifier for ParallaxModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let offset_x = self.depth * self.max_offset;
        let offset_y = self.depth * self.max_offset;
        renderer.push_transform([offset_x, offset_y], [1.0, 1.0], 0.0);
    }
}

// =============================================================================
// DISPLAY ENVIRONMENT (2028 Spatial Computing Readiness)
// =============================================================================

/// Hint to the renderer about the target display environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayEnvironment {
    /// Standard flat display (monitor, laptop).
    #[default]
    Flat,
    /// Spatial display (Apple Vision Pro, Meta Quest, etc.).
    /// Elements have physical depth; glass reflects actual environment.
    Spatial,
    /// Head-up display (projected overlay).
    HeadsUp,
}

// =============================================================================
// PERFORMANCE CONTRACTS
// =============================================================================

/// Performance contract for a UI component.
/// Declares render cost and fallback behavior on underpowered hardware.
#[derive(Debug, Clone, Copy)]
pub struct PerformanceContract {
    /// Maximum acceptable render time per frame (microseconds).
    pub max_render_us: u32,
    /// Whether this component uses glass (requires backdrop blur pass).
    pub uses_glass: bool,
    /// Whether this component has ambient animation (requires continuous redraws).
    pub continuous_animation: bool,
    /// GPU tier minimum for full-quality rendering.
    pub min_tier: crate::RenderTier,
    /// Fallback behavior on Tier3 hardware.
    pub tier3_fallback: Tier3Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier3Fallback {
    /// Render as flat opaque surface.
    FlatOpaque,
    /// Render normally but disable effects.
    NoEffects,
    /// Do not render at all.
    Hidden,
}

impl PerformanceContract {
    pub fn chrome_standard() -> Self {
        Self {
            max_render_us: 300,
            uses_glass: true,
            continuous_animation: false,
            min_tier: crate::RenderTier::Tier2GPU,
            tier3_fallback: Tier3Fallback::FlatOpaque,
        }
    }

    pub fn particle_system() -> Self {
        Self {
            max_render_us: 100,
            uses_glass: false,
            continuous_animation: true,
            min_tier: crate::RenderTier::Tier2GPU,
            tier3_fallback: Tier3Fallback::Hidden,
        }
    }

    pub fn standard() -> Self {
        Self {
            max_render_us: 100,
            uses_glass: false,
            continuous_animation: false,
            min_tier: crate::RenderTier::Tier3Fallback,
            tier3_fallback: Tier3Fallback::FlatOpaque,
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallax_modifier_default() {
        let p = ParallaxModifier::new(0.5);
        assert!((p.depth - 0.5).abs() < 0.01);
        assert!((p.max_offset - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_parallax_clamps_depth() {
        let p = ParallaxModifier::new(1.5);
        assert!((p.depth - 1.0).abs() < 0.01);
        let p = ParallaxModifier::new(-0.5);
        assert!((p.depth - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parallax_with_max_offset() {
        let p = ParallaxModifier::new(0.3).with_max_offset(8.0);
        assert!((p.max_offset - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_display_environment_default() {
        let env: DisplayEnvironment = Default::default();
        assert!(matches!(env, DisplayEnvironment::Flat));
    }

    #[test]
    fn test_display_environment_variants() {
        assert!(matches!(DisplayEnvironment::Flat, DisplayEnvironment::Flat));
        assert!(matches!(DisplayEnvironment::Spatial, DisplayEnvironment::Spatial));
        assert!(matches!(DisplayEnvironment::HeadsUp, DisplayEnvironment::HeadsUp));
    }

    #[test]
    fn test_performance_contract_chrome() {
        let c = PerformanceContract::chrome_standard();
        assert_eq!(c.max_render_us, 300);
        assert!(c.uses_glass);
        assert!(!c.continuous_animation);
        assert!(matches!(c.min_tier, crate::RenderTier::Tier2GPU));
        assert!(matches!(c.tier3_fallback, Tier3Fallback::FlatOpaque));
    }

    #[test]
    fn test_performance_contract_particle() {
        let c = PerformanceContract::particle_system();
        assert_eq!(c.max_render_us, 100);
        assert!(!c.uses_glass);
        assert!(c.continuous_animation);
        assert!(matches!(c.tier3_fallback, Tier3Fallback::Hidden));
    }

    #[test]
    fn test_render_tier_ordering() {
        use crate::RenderTier;
        // Tier1GPU (0) > Tier2GPU (1) > Tier3Fallback (2) in capability
        // but the enum values are ordered 0, 1, 2
        assert!((RenderTier::Tier1GPU as u32) < (RenderTier::Tier2GPU as u32));
        assert!((RenderTier::Tier2GPU as u32) < (RenderTier::Tier3Fallback as u32));
    }

    #[test]
    fn test_tier3_fallback_variants() {
        assert!(matches!(
            Tier3Fallback::FlatOpaque,
            Tier3Fallback::FlatOpaque
        ));
        assert!(matches!(Tier3Fallback::NoEffects, Tier3Fallback::NoEffects));
        assert!(matches!(Tier3Fallback::Hidden, Tier3Fallback::Hidden));
    }
}
//! Glass material — frosted glass backdrop effect.
//!
//! Used for panels, popovers, and nodes that render with blur-behind.
//! Based on macOS vibrancy / Windows Acrylic design language.
//!
//! # Relationship to other types
//! `cvkg-flow::GlassNodeMaterial` extends this concept with OKLCH color
//! and flow-graph-specific serialization. `cvkg-core::DrawMaterial::Glass`
//! is a render-pipeline routing tag, not a parameter bundle.
//! `GlassMaterial` is the authoritative parameter bundle consumed by backends.

use serde::{Deserialize, Serialize};

/// Frosted glass material with backdrop blur, frost, and tint.
///
/// # WHY
/// Glass materials require sampling the scene behind the surface,
/// applying a Gaussian blur, and compositing a tinted frost layer.
/// This struct carries all parameters the compositor needs to set up
/// the multi-pass glass pipeline without any GPU or platform-specific code.
///
/// # Contract
/// - `backdrop_blur` is in logical pixels; backends convert to device pixels.
/// - `tint` is RGBA in 0..1 **linear** space (not gamma-corrected).
/// - `frost` opacity modulates a white diffusion layer; 0 = clear, 1 = fully frosted.
/// - `refraction` ∈ [0, 1]; values outside this range are clamped by builder methods.
/// - `roughness` ∈ [0, 1]; 0 = mirror-like, 1 = fully rough (no specular highlights).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlassMaterial {
    /// Backdrop blur radius in logical pixels (0 = no blur).
    pub backdrop_blur: f32,
    /// Refraction strength 0.0–1.0 (0 = no distortion).
    pub refraction: f32,
    /// Frost opacity 0.0–1.0.
    pub frost: f32,
    /// Tint color as RGBA in 0..1 linear space.
    pub tint: [f32; 4],
    /// Surface roughness for specular highlights (0 = mirror, 1 = fully rough).
    pub roughness: f32,
}

impl GlassMaterial {
    /// Create a `GlassMaterial` with default parameters.
    ///
    /// Defaults: 12 px blur, 0.15 refraction, 0.3 frost, clear tint, 0.5 roughness.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the backdrop blur radius in logical pixels.
    ///
    /// Negative values are clamped to 0 (no blur).
    pub fn with_blur(mut self, blur: f32) -> Self {
        self.backdrop_blur = blur.max(0.0);
        self
    }

    /// Set the refraction strength.
    ///
    /// Clamped to [0, 1]. 0 = no distortion, 1 = maximum IOR distortion.
    pub fn with_refraction(mut self, refraction: f32) -> Self {
        self.refraction = refraction.clamp(0.0, 1.0);
        self
    }

    /// Set the frost opacity.
    ///
    /// Clamped to [0, 1]. 0 = fully transparent white layer, 1 = fully opaque frost.
    pub fn with_frost(mut self, frost: f32) -> Self {
        self.frost = frost.clamp(0.0, 1.0);
        self
    }

    /// Set the tint color as RGBA in 0..1 linear space.
    ///
    /// Components are **not** clamped here; backends must handle out-of-range HDR values
    /// if they choose to accept them. Typical usage should pass values in [0, 1].
    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }

    /// Set the surface roughness.
    ///
    /// Clamped to [0, 1]. 0 = mirror-like specular, 1 = fully diffuse.
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }
}

impl Default for GlassMaterial {
    /// Sensible glass defaults: 12 px blur, subtle refraction, light frost, no tint.
    ///
    /// These values mirror `cvkg-flow::GlassNodeMaterial::default()` for parity,
    /// minus the OKLCH tint (which resolves to near-transparent blue — defaulting
    /// to fully transparent here since the canonical material doesn't carry OKLCH).
    fn default() -> Self {
        Self {
            backdrop_blur: 12.0,
            refraction: 0.15,
            frost: 0.3,
            tint: [1.0, 1.0, 1.0, 0.0], // fully transparent tint by default
            roughness: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let m = GlassMaterial::default();
        assert_eq!(m.backdrop_blur, 12.0);
        assert_eq!(m.refraction, 0.15);
        assert_eq!(m.frost, 0.3);
        assert_eq!(m.tint, [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(m.roughness, 0.5);
    }

    #[test]
    fn test_new_equals_default() {
        assert_eq!(GlassMaterial::new(), GlassMaterial::default());
    }

    #[test]
    fn test_builder_with_blur() {
        let m = GlassMaterial::new().with_blur(24.0);
        assert_eq!(m.backdrop_blur, 24.0);
    }

    #[test]
    fn test_builder_clamps_negative_blur() {
        let m = GlassMaterial::new().with_blur(-5.0);
        assert_eq!(m.backdrop_blur, 0.0);
    }

    #[test]
    fn test_builder_with_refraction_clamp() {
        let m = GlassMaterial::new().with_refraction(2.5);
        assert_eq!(m.refraction, 1.0);
        let m2 = GlassMaterial::new().with_refraction(-0.1);
        assert_eq!(m2.refraction, 0.0);
    }

    #[test]
    fn test_builder_with_frost_clamp() {
        let m = GlassMaterial::new().with_frost(1.5);
        assert_eq!(m.frost, 1.0);
    }

    #[test]
    fn test_builder_with_tint() {
        let tint = [0.1, 0.2, 0.3, 0.8];
        let m = GlassMaterial::new().with_tint(tint);
        assert_eq!(m.tint, tint);
    }

    #[test]
    fn test_builder_with_roughness_clamp() {
        let m = GlassMaterial::new().with_roughness(3.0);
        assert_eq!(m.roughness, 1.0);
        let m2 = GlassMaterial::new().with_roughness(-1.0);
        assert_eq!(m2.roughness, 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = GlassMaterial::new()
            .with_blur(20.0)
            .with_refraction(0.3)
            .with_frost(0.5)
            .with_tint([0.9, 0.9, 1.0, 0.2])
            .with_roughness(0.7);

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: GlassMaterial = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_copy_trait() {
        let a = GlassMaterial::new().with_blur(8.0);
        let b = a; // Copy
        assert_eq!(a.backdrop_blur, b.backdrop_blur);
    }
}

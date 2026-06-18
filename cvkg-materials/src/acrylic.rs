//! Acrylic material — blurred background with noise texture.
//!
//! Acrylic (Windows Fluent Design) blurs content behind the surface and adds
//! a subtle noise grain to prevent color banding. Used for secondary surfaces
//! like sidebars, flyouts, and secondary panels.
//!
//! When the platform cannot sample the content behind the surface (e.g., during
//! window drag, on energy-saver mode, or when hardware acceleration is disabled),
//! the `fallback_color` is used instead.

use serde::{Deserialize, Serialize};

/// Acrylic material — blurred background with noise texture.
///
/// # WHY
/// Acrylic (Windows Fluent Design) blurs content behind the surface
/// and adds a subtle noise grain to prevent banding. Used for secondary
/// surfaces like sidebars and secondary panels. This struct is the
/// canonical parameter bundle; backends produce the actual multi-pass
/// blur + noise composite.
///
/// # Contract
/// - `blur_radius` is in logical pixels; backends convert to device pixels.
/// - `tint` is RGBA in 0..1 linear space, applied **after** the blur pass.
/// - `noise_opacity` ∈ [0, 1]; 0 = no grain, ~0.02 = subtle (recommended), 1 = full noise.
/// - `fallback_color` is used when backdrop sampling is unavailable (drag, power-saver mode).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AcrylicMaterial {
    /// Blur radius in logical pixels.
    pub blur_radius: f32,
    /// Tint color applied after blur, in 0..1 linear RGBA.
    pub tint: [f32; 4],
    /// Noise texture opacity (0 = no grain, 0.02 = subtle grain).
    pub noise_opacity: f32,
    /// Fallback color when backdrop sampling is unavailable.
    pub fallback_color: [f32; 4],
}

impl AcrylicMaterial {
    /// Create an `AcrylicMaterial` with default parameters.
    ///
    /// Defaults: 30 px blur, semi-transparent white tint, 0.02 noise opacity,
    /// light gray fallback.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the blur radius in logical pixels.
    ///
    /// Negative values are clamped to 0 (no blur).
    pub fn with_blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius.max(0.0);
        self
    }

    /// Set the tint color as RGBA in 0..1 linear space.
    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }

    /// Set the noise texture opacity.
    ///
    /// Clamped to [0, 1]. Values around 0.02 produce a subtle, natural-looking grain.
    pub fn with_noise_opacity(mut self, opacity: f32) -> Self {
        self.noise_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set the fallback color used when backdrop sampling is unavailable.
    pub fn with_fallback_color(mut self, color: [f32; 4]) -> Self {
        self.fallback_color = color;
        self
    }
}

impl Default for AcrylicMaterial {
    /// Sensible acrylic defaults matching Windows Fluent Design recommendations.
    fn default() -> Self {
        Self {
            blur_radius: 30.0,
            tint: [1.0, 1.0, 1.0, 0.6],       // semi-transparent white
            noise_opacity: 0.02,                 // subtle grain
            fallback_color: [0.95, 0.95, 0.95, 1.0], // light gray
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let m = AcrylicMaterial::default();
        assert_eq!(m.blur_radius, 30.0);
        assert_eq!(m.tint, [1.0, 1.0, 1.0, 0.6]);
        assert_eq!(m.noise_opacity, 0.02);
        assert_eq!(m.fallback_color, [0.95, 0.95, 0.95, 1.0]);
    }

    #[test]
    fn test_new_equals_default() {
        assert_eq!(AcrylicMaterial::new(), AcrylicMaterial::default());
    }

    #[test]
    fn test_builder_with_blur_radius() {
        let m = AcrylicMaterial::new().with_blur_radius(60.0);
        assert_eq!(m.blur_radius, 60.0);
    }

    #[test]
    fn test_builder_clamps_negative_blur() {
        let m = AcrylicMaterial::new().with_blur_radius(-1.0);
        assert_eq!(m.blur_radius, 0.0);
    }

    #[test]
    fn test_builder_with_tint() {
        let tint = [0.2, 0.3, 0.4, 0.7];
        let m = AcrylicMaterial::new().with_tint(tint);
        assert_eq!(m.tint, tint);
    }

    #[test]
    fn test_builder_noise_opacity_clamp() {
        let m = AcrylicMaterial::new().with_noise_opacity(2.0);
        assert_eq!(m.noise_opacity, 1.0);
        let m2 = AcrylicMaterial::new().with_noise_opacity(-0.1);
        assert_eq!(m2.noise_opacity, 0.0);
    }

    #[test]
    fn test_builder_with_fallback_color() {
        let color = [0.5, 0.5, 0.5, 1.0];
        let m = AcrylicMaterial::new().with_fallback_color(color);
        assert_eq!(m.fallback_color, color);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = AcrylicMaterial::new()
            .with_blur_radius(40.0)
            .with_tint([0.8, 0.8, 0.9, 0.5])
            .with_noise_opacity(0.03)
            .with_fallback_color([0.85, 0.85, 0.85, 1.0]);

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: AcrylicMaterial = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_copy_trait() {
        let a = AcrylicMaterial::new().with_blur_radius(50.0);
        let b = a; // Copy
        assert_eq!(a.blur_radius, b.blur_radius);
    }
}

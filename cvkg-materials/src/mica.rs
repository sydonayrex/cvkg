//! Mica material — Windows 11 Mica effect (system wallpaper sampling + tint).
//!
//! Mica creates a sense of place by reflecting the user's desktop wallpaper
//! through application surfaces. It is used for title bars and navigation
//! surfaces in the Windows 11 Fluent Design Language.
//!
//! Backends must sample the system wallpaper via the platform compositor API
//! (e.g., `DwmSetWindowAttribute` on Windows) and blend it according to these
//! parameters. On platforms without wallpaper access, backends fall back to a
//! neutral tinted surface.

use serde::{Deserialize, Serialize};

/// Mica material — system wallpaper sampling with color blending.
///
/// # WHY
/// Mica creates a sense of place by reflecting the user's desktop
/// wallpaper through the app surface. It's used for title bars and
/// navigation surfaces in Windows 11 design language. This struct
/// carries the parameters needed by the platform backend to configure
/// the Mica effect without any platform or GPU code living here.
///
/// # Contract
/// - `tint_opacity` ∈ [0, 1]; builder clamps.
/// - `tint` is RGBA in 0..1 linear space.
/// - `luminosity` ∈ [0, 2]; values > 1 brighten beyond the wallpaper's natural luminance.
/// - `alt_variant` selects the darker "MicaAlt" variant (used for tab bars and
///   surfaces that sit below the primary Mica surface).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MicaMaterial {
    /// Opacity of the tint overlay (0 = fully transparent, 1 = fully opaque).
    pub tint_opacity: f32,
    /// Tint color blended over the sampled wallpaper in 0..1 linear RGBA.
    pub tint: [f32; 4],
    /// Luminosity boost applied to the sampled wallpaper (1.0 = unchanged).
    pub luminosity: f32,
    /// Whether the darker "MicaAlt" variant is active.
    pub alt_variant: bool,
}

impl MicaMaterial {
    /// Create a `MicaMaterial` with default parameters.
    ///
    /// Defaults: 0.5 tint opacity, neutral white tint, 1.0 luminosity, standard (non-alt) variant.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tint overlay opacity.
    ///
    /// Clamped to [0, 1].
    pub fn with_tint_opacity(mut self, opacity: f32) -> Self {
        self.tint_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set the tint color as RGBA in 0..1 linear space.
    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }

    /// Set the luminosity multiplier.
    ///
    /// 1.0 = unchanged wallpaper luminance. Values in [0, 2] are typical;
    /// the builder does not clamp this to allow HDR rendering pipelines to
    /// experiment with super-unity values, but standard usage should stay in [0, 2].
    pub fn with_luminosity(mut self, luminosity: f32) -> Self {
        self.luminosity = luminosity.max(0.0);
        self
    }

    /// Enable the darker "MicaAlt" variant.
    ///
    /// MicaAlt is used for secondary surfaces that sit below the primary Mica
    /// surface (e.g., tab strips, secondary sidebars). It uses a darker blend
    /// to create visual separation from the primary surface.
    pub fn alt(mut self) -> Self {
        self.alt_variant = true;
        self
    }

    /// Disable the MicaAlt variant, reverting to the standard (lighter) Mica.
    pub fn standard(mut self) -> Self {
        self.alt_variant = false;
        self
    }
}

impl Default for MicaMaterial {
    /// Sensible Mica defaults: semi-transparent white tint, natural luminosity, standard variant.
    fn default() -> Self {
        Self {
            tint_opacity: 0.5,
            tint: [1.0, 1.0, 1.0, 1.0],
            luminosity: 1.0,
            alt_variant: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let m = MicaMaterial::default();
        assert_eq!(m.tint_opacity, 0.5);
        assert_eq!(m.tint, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(m.luminosity, 1.0);
        assert!(!m.alt_variant);
    }

    #[test]
    fn test_new_equals_default() {
        assert_eq!(MicaMaterial::new(), MicaMaterial::default());
    }

    #[test]
    fn test_alt_variant() {
        let m = MicaMaterial::new().alt();
        assert!(m.alt_variant);
    }

    #[test]
    fn test_standard_variant() {
        let m = MicaMaterial::new().alt().standard();
        assert!(!m.alt_variant);
    }

    #[test]
    fn test_builder_tint_opacity_clamp() {
        let m = MicaMaterial::new().with_tint_opacity(2.0);
        assert_eq!(m.tint_opacity, 1.0);
        let m2 = MicaMaterial::new().with_tint_opacity(-0.5);
        assert_eq!(m2.tint_opacity, 0.0);
    }

    #[test]
    fn test_builder_with_luminosity() {
        let m = MicaMaterial::new().with_luminosity(1.5);
        assert_eq!(m.luminosity, 1.5);
    }

    #[test]
    fn test_builder_with_tint() {
        let tint = [0.8, 0.85, 1.0, 0.9];
        let m = MicaMaterial::new().with_tint(tint);
        assert_eq!(m.tint, tint);
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = MicaMaterial::new()
            .with_tint_opacity(0.7)
            .with_tint([0.9, 0.92, 1.0, 1.0])
            .with_luminosity(1.2)
            .alt();

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: MicaMaterial = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, restored);
    }

    #[test]
    fn test_copy_trait() {
        let a = MicaMaterial::new().alt();
        let b = a; // Copy
        assert_eq!(a.alt_variant, b.alt_variant);
    }
}

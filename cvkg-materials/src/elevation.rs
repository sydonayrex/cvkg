//! Elevation system — z-order shadow parameters for CVKG surfaces.
//!
//! Elevation communicates the z-order of surfaces by casting appropriate
//! drop shadows. Six discrete levels (0–5) map to well-defined shadow
//! parameters so that all surfaces at the same conceptual elevation cast
//! identical shadows regardless of which backend renders them.
//!
//! Shadow values are derived from Material Design 3 and Fluent Design
//! guidance, adapted for CVKG's linear-light rendering pipeline.

use serde::{Deserialize, Serialize};

/// Elevation level for the CVKG elevation shadow system.
///
/// # WHY
/// Elevation communicates the z-order of surfaces by casting appropriate
/// shadows. The six levels map to well-defined shadow parameters so that
/// all surfaces at the same conceptual elevation cast identical shadows,
/// regardless of which backend renders them. This eliminates ad-hoc shadow
/// values scattered throughout the codebase.
///
/// # Variants
/// | Level  | Typical use                          |
/// |--------|--------------------------------------|
/// | Level0 | Flat surfaces — no shadow            |
/// | Level1 | Cards, chips                         |
/// | Level2 | Drawers, bottom sheets               |
/// | Level3 | Floating panels, navigation drawers  |
/// | Level4 | Modal dialogs                        |
/// | Level5 | Tooltips, context menus              |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ElevationLevel {
    /// Flat surface — no shadow.
    #[default]
    Level0,
    /// Cards, chips.
    Level1,
    /// Drawers, bottom sheets.
    Level2,
    /// Floating panels, navigation drawers.
    Level3,
    /// Modal dialogs.
    Level4,
    /// Tooltips, context menus.
    Level5,
}

/// Shadow parameters derived from an [`ElevationLevel`].
///
/// All values are in logical pixels except `color` (linear RGBA).
/// Backends convert logical pixels to device pixels using the display scale factor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ElevationShadow {
    /// Shadow blur (Gaussian sigma) in logical pixels.
    pub blur_radius: f32,
    /// Vertical offset of the shadow in logical pixels (positive = down).
    pub y_offset: f32,
    /// Shadow spread radius in logical pixels.
    pub spread: f32,
    /// Shadow color as RGBA in 0..1 linear space.
    pub color: [f32; 4],
}

impl ElevationLevel {
    /// Return the [`ElevationShadow`] parameters for this level.
    ///
    /// # WHY
    /// Centralizing shadow values here ensures that every surface at Level N
    /// casts an identical shadow, giving backends a single source of truth
    /// rather than requiring each backend to maintain its own shadow table.
    ///
    /// Returns `None` for `Level0` — flat surfaces cast no shadow.
    ///
    /// # Shadow value derivation
    /// Values follow a roughly quadratic progression inspired by Material Design 3
    /// elevation tokens, adapted for CVKG's linear-light pipeline (colors are
    /// pre-converted to linear space; gamma-space targets should linearize them).
    pub fn shadow(self) -> Option<ElevationShadow> {
        match self {
            ElevationLevel::Level0 => None,
            ElevationLevel::Level1 => Some(ElevationShadow {
                blur_radius: 2.0,
                y_offset: 1.0,
                spread: 0.0,
                color: [0.0, 0.0, 0.0, 0.12],
            }),
            ElevationLevel::Level2 => Some(ElevationShadow {
                blur_radius: 6.0,
                y_offset: 3.0,
                spread: 0.0,
                color: [0.0, 0.0, 0.0, 0.16],
            }),
            ElevationLevel::Level3 => Some(ElevationShadow {
                blur_radius: 12.0,
                y_offset: 6.0,
                spread: 1.0,
                color: [0.0, 0.0, 0.0, 0.20],
            }),
            ElevationLevel::Level4 => Some(ElevationShadow {
                blur_radius: 24.0,
                y_offset: 12.0,
                spread: 2.0,
                color: [0.0, 0.0, 0.0, 0.24],
            }),
            ElevationLevel::Level5 => Some(ElevationShadow {
                blur_radius: 36.0,
                y_offset: 18.0,
                spread: 4.0,
                color: [0.0, 0.0, 0.0, 0.28],
            }),
        }
    }

    /// Return the Z-plane depth for this elevation level.
    ///
    /// # WHY
    /// When compositing surfaces in a 3D scene graph, each elevation level
    /// occupies a distinct Z plane. Higher levels are closer to the viewer.
    /// This method provides the canonical depth so backends don't need to
    /// hard-code Z offsets.
    ///
    /// # Contract
    /// - Returns 0.0 for `Level0` (on the base plane).
    /// - Monotonically increasing: `LevelN.z_depth() < Level(N+1).z_depth()`.
    /// - Values are in logical Z units; the scale is chosen so that 1.0 ≈ 1 dp of
    ///   apparent lift when rendered with a standard perspective camera.
    pub fn z_depth(self) -> f32 {
        match self {
            ElevationLevel::Level0 => 0.0,
            ElevationLevel::Level1 => 1.0,
            ElevationLevel::Level2 => 2.0,
            ElevationLevel::Level3 => 4.0,
            ElevationLevel::Level4 => 8.0,
            ElevationLevel::Level5 => 16.0,
        }
    }

    /// Return all six elevation levels in ascending order (Level0..=Level5).
    ///
    /// Useful for iterating levels to build shadow atlases or lookup tables.
    pub fn all() -> [ElevationLevel; 6] {
        [
            ElevationLevel::Level0,
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level0_has_no_shadow() {
        assert!(ElevationLevel::Level0.shadow().is_none());
    }

    #[test]
    fn test_all_non_zero_levels_have_shadows() {
        for level in [
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ] {
            assert!(level.shadow().is_some(), "{level:?} should have a shadow");
        }
    }

    #[test]
    fn test_shadow_blur_increases_with_level() {
        let shadows: Vec<ElevationShadow> = [
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ]
        .iter()
        .map(|l| l.shadow().unwrap())
        .collect();

        for window in shadows.windows(2) {
            assert!(
                window[1].blur_radius > window[0].blur_radius,
                "blur_radius should increase: {} <= {}",
                window[1].blur_radius,
                window[0].blur_radius
            );
        }
    }

    #[test]
    fn test_shadow_y_offset_increases_with_level() {
        let offsets: Vec<f32> = [
            ElevationLevel::Level1,
            ElevationLevel::Level2,
            ElevationLevel::Level3,
            ElevationLevel::Level4,
            ElevationLevel::Level5,
        ]
        .iter()
        .map(|l| l.shadow().unwrap().y_offset)
        .collect();

        for window in offsets.windows(2) {
            assert!(window[1] > window[0], "y_offset should increase");
        }
    }

    #[test]
    fn test_z_depth_ordering() {
        let levels = ElevationLevel::all();
        for i in 1..levels.len() {
            assert!(
                levels[i].z_depth() > levels[i - 1].z_depth(),
                "{:?} z_depth ({}) must be greater than {:?} z_depth ({})",
                levels[i],
                levels[i].z_depth(),
                levels[i - 1],
                levels[i - 1].z_depth()
            );
        }
    }

    #[test]
    fn test_z_depth_level0_is_zero() {
        assert_eq!(ElevationLevel::Level0.z_depth(), 0.0);
    }

    #[test]
    fn test_default_is_level0() {
        let level: ElevationLevel = Default::default();
        assert_eq!(level, ElevationLevel::Level0);
    }

    #[test]
    fn test_elevation_level_serde_roundtrip() {
        for level in ElevationLevel::all() {
            let json = serde_json::to_string(&level).expect("serialize");
            let restored: ElevationLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(level, restored);
        }
    }

    #[test]
    fn test_elevation_shadow_serde_roundtrip() {
        let shadow = ElevationShadow {
            blur_radius: 12.0,
            y_offset: 6.0,
            spread: 1.0,
            color: [0.0, 0.0, 0.0, 0.2],
        };
        let json = serde_json::to_string(&shadow).expect("serialize");
        let restored: ElevationShadow = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(shadow, restored);
    }

    #[test]
    fn test_shadow_level5_values() {
        let s = ElevationLevel::Level5.shadow().unwrap();
        assert_eq!(s.blur_radius, 36.0);
        assert_eq!(s.y_offset, 18.0);
        assert_eq!(s.spread, 4.0);
    }

    #[test]
    fn test_copy_trait_elevation_level() {
        let a = ElevationLevel::Level3;
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn test_copy_trait_elevation_shadow() {
        let s = ElevationLevel::Level2.shadow().unwrap();
        let t = s; // Copy
        assert_eq!(s.blur_radius, t.blur_radius);
    }

    #[test]
    fn test_all_returns_six_levels() {
        assert_eq!(ElevationLevel::all().len(), 6);
    }
}

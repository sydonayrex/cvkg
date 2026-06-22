/// Effect LOD (Level of Detail) based on active effect count.
/// When many effects are stacked, reduces quality to maintain frame rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectLod {
    /// All effects at full quality.
    Full,
    /// Reduce blur mip levels, disable volumetric.
    Reduced,
    /// Only essential passes (geometry, UI, composite).
    Minimal,
}

impl EffectLod {
    /// Determine LOD from the number of active effects.
    pub fn from_active_count(count: usize) -> Self {
        match count {
            0..=2 => EffectLod::Full,
            3..=4 => EffectLod::Reduced,
            _ => EffectLod::Minimal,
        }
    }

    /// Number of blur mip levels at this LOD.
    pub fn blur_mip_levels(&self) -> u32 {
        match self {
            EffectLod::Full => 7,
            EffectLod::Reduced => 4,
            EffectLod::Minimal => 2,
        }
    }

    /// Whether volumetric effects should be enabled at this LOD.
    pub fn enable_volumetric(&self) -> bool {
        matches!(self, EffectLod::Full)
    }

    /// Whether bloom should be enabled at this LOD.
    pub fn enable_bloom(&self) -> bool {
        !matches!(self, EffectLod::Minimal)
    }
}

#[cfg(test)]
mod p1_28_effect_lod_tests {
    use super::EffectLod;

    #[test]
    fn full_quality_for_few_effects() {
        assert_eq!(EffectLod::from_active_count(0), EffectLod::Full);
        assert_eq!(EffectLod::from_active_count(1), EffectLod::Full);
        assert_eq!(EffectLod::from_active_count(2), EffectLod::Full);
    }

    #[test]
    fn reduced_quality_for_moderate_effects() {
        assert_eq!(EffectLod::from_active_count(3), EffectLod::Reduced);
        assert_eq!(EffectLod::from_active_count(4), EffectLod::Reduced);
    }

    #[test]
    fn minimal_quality_for_many_effects() {
        assert_eq!(EffectLod::from_active_count(5), EffectLod::Minimal);
        assert_eq!(EffectLod::from_active_count(10), EffectLod::Minimal);
    }

    #[test]
    fn blur_mip_levels_scale_with_lod() {
        assert_eq!(EffectLod::Full.blur_mip_levels(), 7);
        assert_eq!(EffectLod::Reduced.blur_mip_levels(), 4);
        assert_eq!(EffectLod::Minimal.blur_mip_levels(), 2);
    }

    #[test]
    fn volumetric_only_at_full() {
        assert!(EffectLod::Full.enable_volumetric());
        assert!(!EffectLod::Reduced.enable_volumetric());
        assert!(!EffectLod::Minimal.enable_volumetric());
    }

    #[test]
    fn bloom_disabled_at_minimal() {
        assert!(EffectLod::Full.enable_bloom());
        assert!(EffectLod::Reduced.enable_bloom());
        assert!(!EffectLod::Minimal.enable_bloom());
    }
}

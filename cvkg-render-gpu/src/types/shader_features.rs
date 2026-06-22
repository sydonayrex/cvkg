/// Shader feature flags that control permutation generation.
/// Each enabled feature adds to the shader permutation count.
/// Use specialization constants to reduce permutations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderFeatureFlags(pub u32);

impl ShaderFeatureFlags {
    pub const NONE: Self = Self(0);
    pub const GLASS: Self = Self(1 << 0);
    pub const BLOOM: Self = Self(1 << 1);
    pub const VOLUMETRIC: Self = Self(1 << 2);
    pub const COLOR_BLIND: Self = Self(1 << 3);
    pub const PARTICLES: Self = Self(1 << 4);
    pub const DROPSHADOW: Self = Self(1 << 5);
    pub const ALL: Self = Self(0x3F);

    /// Returns the number of enabled features (permutation count contribution).
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if the permutation count is within acceptable limits.
    pub fn is_within_permutation_limit(self) -> bool {
        self.count() <= 4
    }

    /// Returns the permutation index for this feature combination.
    pub fn permutation_index(self) -> u32 {
        self.0
    }

    pub fn has(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
}

impl std::ops::BitOr for ShaderFeatureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ShaderFeatureFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod p2_25_tests {
    use super::ShaderFeatureFlags;

    #[test]
    fn shader_feature_flags_default_is_none() {
        let flags = ShaderFeatureFlags::NONE;
        assert_eq!(flags.count(), 0);
        assert!(flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_combine() {
        let flags = ShaderFeatureFlags::GLASS | ShaderFeatureFlags::BLOOM;
        assert_eq!(flags.count(), 2);
        assert!(flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_permutation_limit() {
        let flags = ShaderFeatureFlags::GLASS
            | ShaderFeatureFlags::BLOOM
            | ShaderFeatureFlags::VOLUMETRIC
            | ShaderFeatureFlags::COLOR_BLIND
            | ShaderFeatureFlags::PARTICLES;
        assert_eq!(flags.count(), 5);
        assert!(!flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_permutation_index() {
        let flags = ShaderFeatureFlags::GLASS | ShaderFeatureFlags::BLOOM;
        assert_eq!(flags.permutation_index(), 3); // 1 | 2 = 3
    }
}

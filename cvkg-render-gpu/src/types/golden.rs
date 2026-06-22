// =============================================================================
// P2-29: Golden-Image Test Infrastructure
// =============================================================================

/// Configuration for golden-image comparison tests.
#[derive(Clone, Debug)]
pub struct GoldenImageConfig {
    /// Per-pixel tolerance (0-255).
    pub pixel_tolerance: u8,
    /// Maximum percentage of differing pixels allowed.
    pub max_diff_percent: f32,
    /// Whether to update golden images on mismatch (for CI).
    pub update_on_mismatch: bool,
}

impl Default for GoldenImageConfig {
    fn default() -> Self {
        Self {
            pixel_tolerance: 3,
            max_diff_percent: 0.1,
            update_on_mismatch: false,
        }
    }
}

/// Result of a golden-image comparison.
#[derive(Clone, Debug)]
pub struct GoldenImageResult {
    /// Whether the test passed.
    pub passed: bool,
    /// Percentage of pixels that differed.
    pub diff_percent: f32,
    /// Number of pixels that differed.
    pub diff_count: u64,
    /// Total number of pixels compared.
    pub total_pixels: u64,
}

/// Golden-image comparator for render output validation.
pub struct GoldenImageComparator;

impl GoldenImageComparator {
    /// Compare two RGBA pixel buffers and return the comparison result.
    ///
    /// # Contract
    /// - Both buffers must have the same length. If lengths differ, the test fails with 100% diff.
    /// - If buffers are empty, the test passes with 0% diff.
    /// - Compares RGB channels only, skipping the alpha channel.
    pub fn compare(
        actual: &[u8],
        expected: &[u8],
        config: &GoldenImageConfig,
    ) -> GoldenImageResult {
        if actual.len() != expected.len() {
            return GoldenImageResult {
                passed: false,
                diff_percent: 100.0,
                diff_count: actual.len() as u64 / 4,
                total_pixels: actual.len() as u64 / 4,
            };
        }

        let total_pixels = (actual.len() / 4) as u64;
        if total_pixels == 0 {
            return GoldenImageResult {
                passed: true,
                diff_percent: 0.0,
                diff_count: 0,
                total_pixels: 0,
            };
        }

        let mut diff_count = 0u64;
        for i in 0..(actual.len() / 4) {
            let base = i * 4;
            let mut pixel_differs = false;
            for ch in 0..3 {
                // Compare RGB only (skip alpha)
                if actual[base + ch].abs_diff(expected[base + ch]) > config.pixel_tolerance {
                    pixel_differs = true;
                    break;
                }
            }
            if pixel_differs {
                diff_count += 1;
            }
        }

        let diff_percent = (diff_count as f32 / total_pixels as f32) * 100.0;
        GoldenImageResult {
            passed: diff_percent <= config.max_diff_percent,
            diff_percent,
            diff_count,
            total_pixels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_image_identical() {
        let pixels = vec![255u8; 400]; // 10x10 white
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&pixels, &pixels, &config);
        assert!(result.passed);
        assert_eq!(result.diff_percent, 0.0);
    }

    #[test]
    fn golden_image_detects_difference() {
        let mut actual = vec![255u8; 400];
        let expected = vec![255u8; 400];
        // Change one pixel significantly
        actual[0] = 0;
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(!result.passed);
        assert!(result.diff_percent > 0.0);
    }

    #[test]
    fn golden_image_tolerance() {
        let mut actual = vec![255u8; 400];
        let expected = vec![255u8; 400];
        // Small difference within tolerance
        actual[0] = 253; // Within tolerance of 3
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(result.passed);
    }

    #[test]
    fn golden_image_different_sizes() {
        let actual = vec![255u8; 400];
        let expected = vec![255u8; 800];
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(!result.passed);
        assert_eq!(result.diff_percent, 100.0);
    }

    #[test]
    fn golden_image_empty() {
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&[], &[], &config);
        assert!(result.passed);
    }
}

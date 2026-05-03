/// Utility for comparing images (pixel buffers) to detect visual regressions.
pub struct VisualComparator {
    /// Tolerance for individual pixel differences (0.0 to 1.0)
    pub pixel_tolerance: f32,
    /// Maximum percentage of different pixels allowed (0.0 to 100.0)
    pub total_tolerance_percent: f32,
}

impl Default for VisualComparator {
    fn default() -> Self {
        Self {
            pixel_tolerance: 0.01,
            total_tolerance_percent: 0.05,
        }
    }
}

impl VisualComparator {
    /// Compare two RGBA pixel buffers of the same size.
    /// Returns the percentage of pixels that differ beyond the tolerance.
    pub fn compare(&self, img1: &[u8], img2: &[u8]) -> f32 {
        if img1.len() != img2.len() {
            return 100.0; // Completely different if sizes don't match
        }
        
        if img1.is_empty() {
            return 0.0;
        }

        let mut diff_count = 0;
        let total_pixels = img1.len() / 4;

        for i in 0..total_pixels {
            let base = i * 4;
            let mut pixel_diff = false;
            
            for j in 0..3 { // Check R, G, B
                let v1 = img1[base + j] as f32 / 255.0;
                let v2 = img2[base + j] as f32 / 255.0;
                if (v1 - v2).abs() > self.pixel_tolerance {
                    pixel_diff = true;
                    break;
                }
            }
            
            if pixel_diff {
                diff_count += 1;
            }
        }

        (diff_count as f32 / total_pixels as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_comparator_identical() {
        let img = vec![255; 400]; // 10x10 white
        let comp = VisualComparator::default();
        assert_eq!(comp.compare(&img, &img), 0.0);
    }

    #[test]
    fn test_visual_comparator_different() {
        let img1 = vec![255; 400];
        let mut img2 = vec![255; 400];
        img2[0] = 0; // Change one pixel
        
        let comp = VisualComparator::default();
        let diff = comp.compare(&img1, &img2);
        assert!(diff > 0.0);
        assert!(diff < 2.0); // Only 1% diff
    }
}

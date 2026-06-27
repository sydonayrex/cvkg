use std::path::PathBuf;

/// Native Visual Regression Testing infrastructure.
/// Captures and compares frames to detect platform-specific visual differences.
#[derive(Debug, Clone)]
pub struct VisualRegressionTracker {
    /// Path to directory where reference "golden" images are located.
    reference_dir: PathBuf,
    /// Absolute threshold difference tolerance per pixel component (0 to 255).
    pixel_tolerance: u8,
    /// Percentage threshold of allowed mismatched pixels (0.0 to 100.0).
    max_mismatched_percentage: f64,
}

impl VisualRegressionTracker {
    /// Creates a new `VisualRegressionTracker` with specified reference folder and tolerances.
    pub fn new(
        reference_dir: PathBuf,
        pixel_tolerance: u8,
        max_mismatched_percentage: f64,
    ) -> Self {
        Self {
            reference_dir,
            pixel_tolerance,
            max_mismatched_percentage,
        }
    }

    /// Compares a captured PNG byte buffer against a named golden reference file.
    ///
    /// If the reference image file does not exist, this function writes the captured PNG
    /// as the new reference (acting in recording mode) and returns `true`.
    pub fn verify_frame(&self, test_name: &str, captured_png: &[u8]) -> bool {
        let reference_path = self.reference_dir.join(format!("{}.png", test_name));
        if !reference_path.exists() {
            log::info!(
                "Golden reference for '{}' not found. Recording current capture as reference.",
                test_name
            );
            if let Some(parent) = reference_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&reference_path, captured_png) {
                log::error!("Failed to write golden image: {}", e);
                return false;
            }
            return true;
        }

        // Load reference image
        let ref_img =
            match image::load_from_memory(&std::fs::read(&reference_path).unwrap_or_default()) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    log::error!("Failed to decode reference image: {}", e);
                    return false;
                }
            };

        // Load captured image
        let cap_img = match image::load_from_memory(captured_png) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to decode captured image: {}", e);
                return false;
            }
        };

        if ref_img.dimensions() != cap_img.dimensions() {
            log::warn!(
                "Dimensions mismatch for test '{}': ref {:?}, cap {:?}",
                test_name,
                ref_img.dimensions(),
                cap_img.dimensions()
            );
            return false;
        }

        let (width, height) = ref_img.dimensions();
        let total_pixels = width as f64 * height as f64;
        let mut mismatched_pixels = 0;

        for (x, y, ref_pixel) in ref_img.enumerate_pixels() {
            let cap_pixel = cap_img.get_pixel(x, y);
            let mut pixel_differs = false;
            for c in 0..4 {
                let diff = (ref_pixel[c] as i16 - cap_pixel[c] as i16).abs();
                if diff > self.pixel_tolerance as i16 {
                    pixel_differs = true;
                    break;
                }
            }
            if pixel_differs {
                mismatched_pixels += 1;
            }
        }

        let mismatch_pct = (mismatched_pixels as f64 / total_pixels) * 100.0;
        if mismatch_pct > self.max_mismatched_percentage {
            log::warn!(
                "Visual regression detected in test '{}': {:.2}% mismatched pixels (max allowed {:.2}%)",
                test_name,
                mismatch_pct,
                self.max_mismatched_percentage
            );
            false
        } else {
            true
        }
    }
}

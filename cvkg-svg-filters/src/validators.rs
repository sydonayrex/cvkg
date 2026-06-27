use crate::types::FilterError;

// =============================================================================
// P1-31: Lighting Filter Validation
// =============================================================================

/// Validates lighting filter parameters.
pub struct LightingValidator;

impl LightingValidator {
    pub fn validate_diffuse_lighting(
        surface_scale: f32,
        diffuse_constant: f32,
        kernel_unit_length: Option<(f32, f32)>,
    ) -> Result<(), FilterError> {
        if surface_scale < 0.0 {
            return Err(FilterError::InvalidRegion(surface_scale, 0.0));
        }
        if diffuse_constant < 0.0 {
            return Err(FilterError::InvalidRegion(diffuse_constant, 0.0));
        }
        if let Some((kx, ky)) = kernel_unit_length
            && (kx <= 0.0 || ky <= 0.0)
        {
            return Err(FilterError::InvalidRegion(kx, ky));
        }
        Ok(())
    }

    pub fn validate_specular_lighting(
        surface_scale: f32,
        specular_constant: f32,
        specular_exponent: f32,
    ) -> Result<(), FilterError> {
        if surface_scale < 0.0 {
            return Err(FilterError::InvalidRegion(surface_scale, 0.0));
        }
        if specular_constant < 0.0 {
            return Err(FilterError::InvalidRegion(specular_constant, 0.0));
        }
        if !(1.0..=128.0).contains(&specular_exponent) {
            return Err(FilterError::InvalidRegion(specular_exponent, 1.0));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn validate_light_source(
        light_type: &str,
        azimuth: Option<f32>,
        elevation: Option<f32>,
    ) -> Result<(), FilterError> {
        match light_type {
            "distant" => {
                if let Some(az) = azimuth
                    && !(0.0..=360.0).contains(&az)
                {
                    return Err(FilterError::InvalidRegion(az, 0.0));
                }
                if let Some(el) = elevation
                    && !(-90.0..=90.0).contains(&el)
                {
                    return Err(FilterError::InvalidRegion(el, 0.0));
                }
            }
            "point" | "spot" => {}
            other => {
                return Err(FilterError::UnresolvedInput(format!(
                    "unknown light type: {}",
                    other
                )));
            }
        }
        Ok(())
    }
}

// =============================================================================
// P1-32: Turbulence Filter Validation
// =============================================================================

/// Validates turbulence filter parameters and provides reference values.
pub struct TurbulenceValidator;

impl TurbulenceValidator {
    pub fn validate_turbulence(
        base_frequency_x: f32,
        base_frequency_y: f32,
        num_octaves: i32,
        seed: i32,
        stitch_tiles: bool,
    ) -> Result<(), FilterError> {
        if base_frequency_x < 0.0 || base_frequency_y < 0.0 {
            return Err(FilterError::InvalidRegion(
                base_frequency_x,
                base_frequency_y,
            ));
        }
        if !(1..=8).contains(&num_octaves) {
            return Err(FilterError::InvalidRegion(num_octaves as f32, 1.0));
        }
        if seed < 0 {
            return Err(FilterError::InvalidRegion(seed as f32, 0.0));
        }
        let _ = stitch_tiles;
        Ok(())
    }

    pub fn reference_value(x: f32, y: f32, seed: i32) -> f32 {
        let n = (x * 12.9898 + y * 78.233 + seed as f32 * 0.001).sin();
        let val = (n * 43_758.547).fract();
        val.clamp(0.0, 1.0)
    }
}

// =============================================================================
// P1-37: Glass Effects Compatibility
// =============================================================================

pub struct GlassCompatReference;

impl GlassCompatReference {
    pub const BLUR_RADIUS_RANGE: (f32, f32) = (5.0, 40.0);
    pub const OPACITY_RANGE: (f32, f32) = (0.3, 0.85);
    pub const NOISE_INTENSITY_RANGE: (f32, f32) = (0.01, 0.05);

    pub fn validate_glass_params(blur_radius: f32, opacity: f32) -> Result<(), FilterError> {
        if !(Self::BLUR_RADIUS_RANGE.0..=Self::BLUR_RADIUS_RANGE.1).contains(&blur_radius) {
            return Err(FilterError::InvalidRegion(
                blur_radius,
                Self::BLUR_RADIUS_RANGE.0,
            ));
        }
        if !(Self::OPACITY_RANGE.0..=Self::OPACITY_RANGE.1).contains(&opacity) {
            return Err(FilterError::InvalidRegion(opacity, Self::OPACITY_RANGE.0));
        }
        Ok(())
    }
}

// =============================================================================
// P1-58: Kerning Validation
// =============================================================================

pub struct KerningValidator;

impl KerningValidator {
    pub const KNOWN_PAIRS: &[(&str, &str, f32)] = &[
        ("A", "V", -0.05),
        ("A", "W", -0.04),
        ("T", "o", -0.04),
        ("T", "a", -0.03),
        ("V", "A", -0.05),
        ("W", "A", -0.04),
        ("L", "T", -0.03),
        ("P", "a", -0.02),
    ];

    pub fn validate_kern_pair(left: &str, right: &str, kern_value: f32) -> bool {
        for (l, r, expected) in Self::KNOWN_PAIRS {
            if *l == left && *r == right {
                return kern_value <= 0.0 && kern_value >= *expected * 1.5;
            }
        }
        true
    }
}

// =============================================================================
// P2-33: SVG Filter Browser Parity Testing
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserEngine {
    Chromium,
    Firefox,
    Safari,
}

#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub engine: BrowserEngine,
    pub use_linear_rgb: bool,
    pub blur_approx_coefficient: f32,
    pub color_precision_epsilon: f32,
}

impl BrowserProfile {
    pub fn for_engine(engine: BrowserEngine) -> Self {
        match engine {
            BrowserEngine::Chromium => Self {
                engine,
                use_linear_rgb: true,
                blur_approx_coefficient: 1.0,
                color_precision_epsilon: 0.01,
            },
            BrowserEngine::Firefox => Self {
                engine,
                use_linear_rgb: true,
                blur_approx_coefficient: 0.98,
                color_precision_epsilon: 0.02,
            },
            BrowserEngine::Safari => Self {
                engine,
                use_linear_rgb: false,
                blur_approx_coefficient: 1.02,
                color_precision_epsilon: 0.03,
            },
        }
    }
}

pub struct BrowserParityValidator;

impl BrowserParityValidator {
    pub fn validate_blur_parity(
        std_deviation: f32,
        effective_std_dev: f32,
        profile: &BrowserProfile,
    ) -> bool {
        let expected = std_deviation * profile.blur_approx_coefficient;
        (effective_std_dev - expected).abs() < 0.1
    }

    pub fn validate_color_matrix_parity(
        input_color: [f32; 4],
        effective_output: [f32; 4],
        color_matrix: &[f32; 20],
        profile: &BrowserProfile,
    ) -> bool {
        let rgb = if profile.use_linear_rgb {
            [
                Self::srgb_to_linear(input_color[0]),
                Self::srgb_to_linear(input_color[1]),
                Self::srgb_to_linear(input_color[2]),
            ]
        } else {
            [input_color[0], input_color[1], input_color[2]]
        };
        let a = input_color[3];

        let r_out = color_matrix[0] * rgb[0]
            + color_matrix[1] * rgb[1]
            + color_matrix[2] * rgb[2]
            + color_matrix[3] * a
            + color_matrix[4];
        let g_out = color_matrix[5] * rgb[0]
            + color_matrix[6] * rgb[1]
            + color_matrix[7] * rgb[2]
            + color_matrix[8] * a
            + color_matrix[9];
        let b_out = color_matrix[10] * rgb[0]
            + color_matrix[11] * rgb[1]
            + color_matrix[12] * rgb[2]
            + color_matrix[13] * a
            + color_matrix[14];
        let a_out = color_matrix[15] * rgb[0]
            + color_matrix[16] * rgb[1]
            + color_matrix[17] * rgb[2]
            + color_matrix[18] * a
            + color_matrix[19];

        let mut expected = if profile.use_linear_rgb {
            [
                Self::linear_to_srgb(r_out),
                Self::linear_to_srgb(g_out),
                Self::linear_to_srgb(b_out),
                a_out.clamp(0.0, 1.0),
            ]
        } else {
            [
                r_out.clamp(0.0, 1.0),
                g_out.clamp(0.0, 1.0),
                b_out.clamp(0.0, 1.0),
                a_out.clamp(0.0, 1.0),
            ]
        };

        for c in expected.iter_mut() {
            *c = c.clamp(0.0, 1.0);
        }

        for i in 0..4 {
            if (effective_output[i] - expected[i]).abs() > profile.color_precision_epsilon {
                return false;
            }
        }
        true
    }

    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    fn linear_to_srgb(c: f32) -> f32 {
        if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffuse_lighting_valid() {
        assert!(LightingValidator::validate_diffuse_lighting(1.0, 1.0, None).is_ok());
    }

    #[test]
    fn diffuse_lighting_rejects_negative_surface_scale() {
        assert!(LightingValidator::validate_diffuse_lighting(-1.0, 1.0, None).is_err());
    }

    #[test]
    fn specular_lighting_rejects_bad_exponent() {
        assert!(LightingValidator::validate_specular_lighting(1.0, 1.0, 0.5).is_err());
        assert!(LightingValidator::validate_specular_lighting(1.0, 1.0, 200.0).is_err());
    }

    #[test]
    fn turbulence_valid() {
        assert!(TurbulenceValidator::validate_turbulence(0.05, 0.05, 3, 42, false).is_ok());
    }

    #[test]
    fn turbulence_rejects_negative_frequency() {
        assert!(TurbulenceValidator::validate_turbulence(-0.01, 0.05, 3, 42, false).is_err());
    }

    #[test]
    fn test_browser_profiles() {
        let chrome = BrowserProfile::for_engine(BrowserEngine::Chromium);
        assert!(chrome.use_linear_rgb);
    }

    #[test]
    fn test_blur_parity() {
        let chrome = BrowserProfile::for_engine(BrowserEngine::Chromium);
        assert!(BrowserParityValidator::validate_blur_parity(
            5.0, 5.0, &chrome
        ));
    }
}

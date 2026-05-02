use cvkg_core::Color;

/// Semantic colors for the Berserker Design System
#[derive(Debug, Clone)]
pub struct SemanticColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
}

/// Typography scale for consistent rhythmic text
#[derive(Debug, Clone)]
pub struct TypographyScale {
    pub hero: f32,
    pub h1: f32,
    pub h2: f32,
    pub body: f32,
    pub caption: f32,
    pub code: f32,
}

/// Spacing scale for layout consistency
#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub xs: f32,
    pub s: f32,
    pub m: f32,
    pub l: f32,
    pub xl: f32,
}

/// Motion scale for standardized animation physics
#[derive(Debug, Clone)]
pub struct MotionScale {
    pub snappy: cvkg_anim::SleipnirParams,
    pub fluid: cvkg_anim::SleipnirParams,
    pub heavy: cvkg_anim::SleipnirParams,
    pub bouncy: cvkg_anim::SleipnirParams,
}

/// A resolved Theme instance, providing concrete values for the current mode
#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: SemanticColors,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub motion: MotionScale,
    is_dark: bool,
}

impl Theme {
    /// Create a theme with default Norse tokens
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            colors: SemanticColors {
                primary: Color { r: 0.0, g: 0.8, b: 1.0, a: 1.0 }, // Cyan Glow
                secondary: Color { r: 0.5, g: 0.0, b: 1.0, a: 1.0 }, // Purple Haze
                accent: Color { r: 1.0, g: 0.0, b: 0.4, a: 1.0 }, // Crimson Flash
                background: Color { r: 0.02, g: 0.02, b: 0.05, a: 1.0 }, // Deep Void
                surface: Color { r: 0.08, g: 0.08, b: 0.12, a: 1.0 }, // Obsidian
                error: Color { r: 1.0, g: 0.2, b: 0.2, a: 1.0 },
                warning: Color { r: 1.0, g: 0.8, b: 0.0, a: 1.0 },
                success: Color { r: 0.0, g: 1.0, b: 0.5, a: 1.0 },
                text: Color { r: 0.95, g: 0.95, b: 1.0, a: 1.0 },
                text_dim: Color { r: 0.6, g: 0.6, b: 0.7, a: 1.0 },
            },
            typography: TypographyScale {
                hero: 48.0,
                h1: 32.0,
                h2: 24.0,
                body: 16.0,
                caption: 12.0,
                code: 12.0,
            },
            spacing: SpacingScale {
                xs: 4.0,
                s: 8.0,
                m: 16.0,
                l: 24.0,
                xl: 32.0,
            },
            motion: MotionScale {
                snappy: cvkg_anim::SleipnirParams::snappy(),
                fluid: cvkg_anim::SleipnirParams::fluid(),
                heavy: cvkg_anim::SleipnirParams::heavy(),
                bouncy: cvkg_anim::SleipnirParams::bouncy(),
            },
        }
    }

    /// Check if the theme is currently in dark mode
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Validate the theme against WCAG 2.1 accessibility standards
    /// Returns a list of strings describing any contrast failures
    pub fn validate_accessibility(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Primary on Background (Minimum 4.5:1 for normal text)
        let primary_bg = self.colors.primary.contrast_ratio(&self.colors.background);
        if primary_bg < 4.5 {
            warnings.push(format!("Primary color contrast ratio too low: {:.2}:1 (Background)", primary_bg));
        }

        // Text on Background (Minimum 4.5:1)
        let text_bg = self.colors.text.contrast_ratio(&self.colors.background);
        if text_bg < 4.5 {
            warnings.push(format!("Text color contrast ratio too low: {:.2}:1 (Background)", text_bg));
        }

        // Text on Surface (Minimum 4.5:1)
        let text_surface = self.colors.text.contrast_ratio(&self.colors.surface);
        if text_surface < 4.5 {
            warnings.push(format!("Text color contrast ratio too low: {:.2}:1 (Surface)", text_surface));
        }

        warnings
    }
}

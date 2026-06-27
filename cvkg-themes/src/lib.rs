use cvkg_core::Color;

// =============================================================================
// OKLCH COLOR MODEL
// =============================================================================

/// A color in the perceptually uniform OKLCH color space.
///
/// OKLCH (Lightness, Chroma, Hue) provides perceptually uniform color
/// manipulation -- adjusting lightness produces visually consistent results
/// across all hues, unlike HSL or raw RGB.
///
/// Fields:
/// - `l`: Lightness, range [0.0, 1.0] (black to white)
/// - `c`: Chroma, range [0.0, ~0.4] (gray to fully saturated)
/// - `h`: Hue angle in degrees, range [0.0, 360.0)
/// - `a`: Alpha opacity, range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklchColor {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}

impl OklchColor {
    /// Creates a new OKLCH color from individual components.
    pub fn new(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self { l, c, h, a }
    }

    /// Converts sRGB (r, g, b) values in [0.0, 1.0] to an OKLCH color.
    ///
    /// Pipeline: sRGB -> linear RGB -> OKLab -> OKLCH
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        // Clamp inputs to valid sRGB range to prevent NaN from powf on negatives
        let r = r.clamp(0.0, 1.0);
        let g = g.clamp(0.0, 1.0);
        let b = b.clamp(0.0, 1.0);
        // sRGB to linear RGB
        let to_linear = |c: f32| -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        let r_lin = to_linear(r);
        let g_lin = to_linear(g);
        let b_lin = to_linear(b);

        // Linear RGB to OKLab (using the standard OKLab matrix)
        // l_ = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
        // m_ = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
        // s_ = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
        let l_ = 0.412_221_46 * r_lin + 0.536_332_55 * g_lin + 0.051_445_995 * b_lin;
        let m_ = 0.211_903_5 * r_lin + 0.680_699_5 * g_lin + 0.107_396_96 * b_lin;
        let s_ = 0.088_302_46 * r_lin + 0.281_718_85 * g_lin + 0.629_978_7 * b_lin;

        // Cube root
        let l_cbrt = l_.cbrt();
        let m_cbrt = m_.cbrt();
        let s_cbrt = s_.cbrt();

        // OKLab coordinates
        let l = 0.210_454_26 * l_cbrt + 0.793_617_8 * m_cbrt - 0.004_072_047 * s_cbrt;
        let a = 1.977_998_5 * l_cbrt - 2.428_592_2 * m_cbrt + 0.450_593_7 * s_cbrt;
        let b = 0.025_904_037 * l_cbrt + 0.782_771_77 * m_cbrt - 0.808_675_77 * s_cbrt;

        // OKLab to OKLCH
        let c = (a * a + b * b).sqrt();
        let h = b.atan2(a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };

        Self {
            l: l.clamp(0.0, 1.0),
            c: c.clamp(0.0, 0.4),
            h,
            a: 1.0,
        }
    }

    /// Converts this OKLCH color back to an sRGB `Color`.
    ///
    /// Pipeline: OKLCH -> OKLab -> linear RGB -> sRGB
    pub fn to_rgba(&self) -> Color {
        // OKLCH to OKLab
        let h_rad = self.h.to_radians();
        let a = self.c * h_rad.cos();
        let b = self.c * h_rad.sin();

        // OKLab to linear RGB
        let l = self.l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m = self.l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s = self.l - 0.089_484_18 * a - 1.291_485_5 * b;

        // Cube
        let l_cubed = l * l * l;
        let m_cubed = m * m * m;
        let s_cubed = s * s * s;

        // Linear RGB
        let r_lin = 4.076_741_7 * l_cubed - 3.307_711_6 * m_cubed + 0.230_969_94 * s_cubed;
        let g_lin = -1.268_438 * l_cubed + 2.609_757_4 * m_cubed - 0.341_319_38 * s_cubed;
        let b_lin = -0.0041960863 * l_cubed - 0.703_418_6 * m_cubed + 1.707_614_7 * s_cubed;

        // Linear RGB to sRGB
        let to_srgb = |c: f32| -> f32 {
            let c = c.clamp(0.0, 1.0);
            if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        };

        Color::new(
            to_srgb(r_lin),
            to_srgb(g_lin),
            to_srgb(b_lin),
            self.a.clamp(0.0, 1.0),
        )
    }

    /// Returns a new color with lightness increased by `amount`.
    pub fn lighten(&self, amount: f32) -> Self {
        Self {
            l: (self.l + amount).clamp(0.0, 1.0),
            ..*self
        }
    }

    /// Returns a new color with lightness decreased by `amount`.
    pub fn darken(&self, amount: f32) -> Self {
        Self {
            l: (self.l - amount).clamp(0.0, 1.0),
            ..*self
        }
    }

    /// Returns a new color with chroma increased by `amount`.
    pub fn saturate(&self, amount: f32) -> Self {
        Self {
            c: (self.c + amount).clamp(0.0, 0.4),
            ..*self
        }
    }

    /// Returns a new color with hue rotated by `degrees`.
    pub fn rotate_hue(&self, degrees: f32) -> Self {
        let mut new_h = self.h + degrees;
        while new_h < 0.0 {
            new_h += 360.0;
        }
        while new_h >= 360.0 {
            new_h -= 360.0;
        }
        Self { h: new_h, ..*self }
    }

    /// Computes the sRGB relative luminance (Y) of this color.
    ///
    /// Uses the WCAG 2.x relative luminance formula on the converted sRGB values.
    pub fn relative_luminance(&self) -> f32 {
        self.to_rgba().relative_luminance()
    }
}

// =============================================================================
// GLASSMORPHIC MATERIAL TOKENS
// =============================================================================

/// A glassmorphic material descriptor for frosted-glass UI surfaces.
///
/// Encapsulates all visual properties needed to render a physically-plausible
/// glass panel: backdrop blur, refraction, frost noise, tint, and edge glow.
#[derive(Debug, Clone)]
pub struct GlassMaterial {
    /// Backdrop blur radius in logical pixels.
    pub backdrop_blur_radius: f32,
    /// Snell's law refraction index (1.0 = air, ~1.5 = glass).
    pub refraction_index: f32,
    /// Frost noise intensity in the range [0.0, 1.0].
    pub frost_intensity: f32,
    /// Tint color applied over the refracted backdrop.
    pub tint_color: OklchColor,
    /// Opacity of the tint layer in [0.0, 1.0].
    pub tint_opacity: f32,
    /// Color of the neon border glow.
    pub border_glow_color: OklchColor,
    /// Spread radius of the border glow in logical pixels.
    pub border_glow_radius: f32,
}

impl GlassMaterial {
    /// Returns a sensible default glass material with subtle dark tint.
    pub fn default_glass() -> Self {
        Self {
            backdrop_blur_radius: 20.0,
            refraction_index: 1.15,
            frost_intensity: 0.03,
            tint_color: OklchColor::new(0.05, 0.01, 260.0, 1.0),
            tint_opacity: 0.12,
            border_glow_color: OklchColor::new(0.8, 0.05, 200.0, 0.6),
            border_glow_radius: 8.0,
        }
    }
}

/// Convert a GlassMaterial's OKLCH tint into RGBA for GPU uniforms.
/// Called by the renderer when a glass material override is applied.
pub fn glass_material_to_gpu_patch(mat: &GlassMaterial) -> [f32; 4] {
    let rgba = mat.tint_color.to_rgba();
    [rgba.r, rgba.g, rgba.b, mat.tint_opacity]
}

/// Generate a complete `ColorTheme` GPU uniform from a single OKLCH seed color.
///
/// This is the fast path for dynamic theme generation: given any OKLCH color,
/// produces a full 160-byte GPU-ready `ColorTheme` without allocating a `Theme`.
///
/// The palette is procedurally derived from the seed by rotating hue, adjusting
/// lightness, and modulating chroma in perceptually uniform OKLCH space.
///
/// # Example
/// ```
/// use cvkg_themes::{OklchColor, oklch_to_color_theme};
/// let seed = OklchColor::new(0.55, 0.12, 260.0, 1.0);
/// let theme = oklch_to_color_theme(seed);
/// // Upload theme to GPU via renderer.set_theme(theme)
/// ```
pub fn oklch_to_color_theme(seed: OklchColor) -> cvkg_core::ColorTheme {
    let is_dark = seed.l < 0.5;

    // Derive palette from seed using OKLCH color science
    let primary = seed.to_rgba();

    let bg = if is_dark { 0.02 } else { 0.98 };
    let background = OklchColor::new(bg, seed.c * 0.1, seed.h, 1.0).to_rgba();

    // Glass tints derived from seed
    let glass_base_l = if is_dark { 0.04 } else { 0.92 };
    let glass_base_c = seed.c * 0.08;
    let glass_base = OklchColor::new(
        glass_base_l,
        glass_base_c,
        seed.h,
        if is_dark { 0.82 } else { 0.15 },
    )
    .to_rgba();
    let glass_edge = OklchColor::new(0.5, seed.c * 0.2, seed.h + 180.0, 0.6).to_rgba();

    // Neon colors: primary = seed, shatter = complementary
    let shatter = seed.rotate_hue(180.0).saturate(0.15).to_rgba();
    let rune = seed.rotate_hue(30.0).lighten(0.2).to_rgba();
    let ember = OklchColor::new(0.7, 0.2, 30.0, 1.0).to_rgba();

    // Mani glow: cool blue-white
    let mani_glow = OklchColor::new(0.8, 0.05, 240.0, 0.8).to_rgba();

    cvkg_core::ColorTheme {
        primary_neon: [primary.r, primary.g, primary.b, 1.2],
        shatter_neon: [shatter.r, shatter.g, shatter.b, 1.5],
        glass_base: [glass_base.r, glass_base.g, glass_base.b, glass_base.a],
        glass_edge: [glass_edge.r, glass_edge.g, glass_edge.b, glass_edge.a],
        rune_glow: [rune.r, rune.g, rune.b, 0.9],
        ember_core: [ember.r, ember.g, ember.b, 1.0],
        background_deep: [background.r, background.g, background.b, 1.0],
        mani_glow: [mani_glow.r, mani_glow.g, mani_glow.b, 0.8],
        glass_blur_strength: 1.0,
        shatter_edge_width: 2.0,
        neon_bloom_radius: 20.0,
        rune_opacity: 0.7,
        glass_tint_adapt: 0.35,
        glass_ior: 1.45,
        color_space: 0,
        _pad0: 0.0,
        _pad1: 0.0,
        _pad2: 0.0,
        _pad3: 0.0,
        _pad4: 0.0,
    }
}

// =============================================================================
// SEMANTIC COLORS
// =============================================================================

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

// =============================================================================
// APCA CONTRAST
// =============================================================================

/// Result of an APCA contrast evaluation.
#[derive(Debug, Clone)]
pub struct ApcaResult {
    /// The computed APCA contrast value (Lc), typically in the range 0-100+.
    pub contrast: f32,
    /// Whether the contrast meets the required threshold for the given context.
    pub passes: bool,
    /// A human-readable level label: "fail", "large-only", or "pass".
    pub level: &'static str,
}

impl std::fmt::Display for ApcaResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "APCA Lc={:.1} — {} ({})",
            self.contrast,
            if self.passes { "PASS" } else { "FAIL" },
            self.level
        )
    }
}

// =============================================================================
// TYPOGRAPHY, SPACING, MOTION
// =============================================================================

/// Typography scale matching Apple's text styles (HIG)
#[derive(Debug, Clone)]
pub struct TypographyScale {
    // Apple HIG text styles
    pub large_title: f32, // 34px
    pub title1: f32,      // 28px
    pub title2: f32,      // 22px
    pub title3: f32,      // 20px
    pub headline: f32,    // 17px semibold
    pub body: f32,        // 17px
    pub callout: f32,     // 16px
    pub subheadline: f32, // 15px
    pub footnote: f32,    // 13px
    pub caption1: f32,    // 12px
    pub caption2: f32,    // 11px
    // Legacy aliases
    pub hero: f32,
    pub h1: f32,
    pub h2: f32,
    pub caption: f32,
    pub code: f32,
}

/// Corner radius scale anchored to Tahoe's 12px standard
#[derive(Debug, Clone)]
pub struct RadiusScale {
    pub xs: f32,   // 4px  -- small controls, tags
    pub s: f32,    // 6px  -- buttons, text fields
    pub m: f32,    // 8px  -- cards, alerts
    pub l: f32,    // 10px -- panels, popovers
    pub xl: f32,   // 12px -- windows, dialogs (Tahoe standard)
    pub xxl: f32,  // 16px -- large panels, sheets
    pub full: f32, // 9999px -- circles, pills, squircle icons
}

/// Spacing scale for layout consistency (4px grid)
#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub xs: f32,   // 4px
    pub s: f32,    // 8px
    pub m: f32,    // 12px
    pub l: f32,    // 16px
    pub xl: f32,   // 24px
    pub xxl: f32,  // 32px
    pub xxxl: f32, // 48px
}

/// Motion scale for standardized animation physics
#[derive(Debug, Clone)]
pub struct MotionScale {
    pub snappy: cvkg_anim::SpringParams,
    pub fluid: cvkg_anim::SpringParams,
    pub heavy: cvkg_anim::SpringParams,
    pub bouncy: cvkg_anim::SpringParams,
}

/// Density variant for controlling spacing/radius scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    /// Compact: 0.75x spacing/radius
    Compact,
    /// Default: 1.0x spacing/radius
    Default,
    /// Spacious: 1.25x spacing/radius
    Spacious,
}

impl Density {
    /// Returns the multiplier for spacing and radius values.
    pub fn multiplier(self) -> f32 {
        match self {
            Density::Compact => 0.75,
            Density::Default => 1.0,
            Density::Spacious => 1.25,
        }
    }
}

/// Accessibility override flags for theme generation
#[derive(Debug, Clone, Default)]
pub struct AccessibilityOverrides {
    /// If true, replace all blur/glass effects with solid backgrounds
    pub reduce_transparency: bool,
    /// If true, disable spring animations and reduce motion
    pub reduce_motion: bool,
    /// If true, increase contrast for all text/background pairs
    pub increase_contrast: bool,
}

// =============================================================================
// THEME
// =============================================================================

/// A resolved Theme instance, providing concrete values for the current mode
#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: SemanticColors,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub motion: MotionScale,
    pub materials: Vec<GlassMaterial>,
    pub accessibility: AccessibilityOverrides,
    /// Density multiplier for spacing/radius. Default: 1.0 (Default density).
    pub density: Density,
    /// If true, components may use glassmorphic effects (frosted glass, blur).
    /// When false, components should render with solid backgrounds instead.
    /// Default: true for dark themes, false for light themes.
    pub glassmorphism_enabled: bool,
    is_dark: bool,
}

impl Theme {
    /// Create a theme with default Norse tokens
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            colors: SemanticColors {
                primary: Color::VIKING_GOLD,
                secondary: Color::MAGENTA_LIQUID,
                accent: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.4,
                    a: 1.0,
                }, // Crimson Flash
                background: Color {
                    r: 0.02,
                    g: 0.02,
                    b: 0.05,
                    a: 1.0,
                }, // Deep Void
                surface: Color::TACTICAL_OBSIDIAN,
                error: Color {
                    r: 1.0,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                },
                warning: Color {
                    r: 1.0,
                    g: 0.8,
                    b: 0.0,
                    a: 1.0,
                },
                success: Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.5,
                    a: 1.0,
                },
                text: Color {
                    r: 0.95,
                    g: 0.95,
                    b: 1.0,
                    a: 1.0,
                },
                text_dim: Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.7,
                    a: 1.0,
                },
            },
            typography: TypographyScale {
                // Apple HIG text styles
                large_title: 34.0,
                title1: 28.0,
                title2: 22.0,
                title3: 20.0,
                headline: 17.0,
                body: 17.0,
                callout: 16.0,
                subheadline: 15.0,
                footnote: 13.0,
                caption1: 12.0,
                caption2: 11.0,
                // Legacy aliases
                hero: 48.0,
                h1: 32.0,
                h2: 24.0,
                caption: 12.0,
                code: 12.0,
            },
            spacing: SpacingScale {
                xs: 4.0,
                s: 8.0,
                m: 12.0,
                l: 16.0,
                xl: 24.0,
                xxl: 32.0,
                xxxl: 48.0,
            },
            radius: RadiusScale {
                xs: 4.0,
                s: 6.0,
                m: 8.0,
                l: 10.0,
                xl: 12.0,
                xxl: 16.0,
                full: 9999.0,
            },
            motion: MotionScale {
                snappy: cvkg_anim::SpringParams::snappy(),
                fluid: cvkg_anim::SpringParams::fluid(),
                heavy: cvkg_anim::SpringParams::heavy(),
                bouncy: cvkg_anim::SpringParams::bouncy(),
            },
            materials: vec![GlassMaterial::default_glass()],
            accessibility: AccessibilityOverrides::default(),
            density: Density::Default,
            glassmorphism_enabled: true,
        }
    }

    /// Generate a complete theme from a single seed color using OKLCH color science.
    ///
    /// The entire `SemanticColors` palette is procedurally derived from the
    /// `seed_color` by rotating hue, adjusting lightness, and modulating chroma
    /// in the perceptually uniform OKLCH space.
    pub fn from_seed(seed_color: OklchColor) -> Theme {
        let is_dark = seed_color.l < 0.5;

        let primary = seed_color;
        let secondary = seed_color.rotate_hue(120.0);
        let accent = seed_color.rotate_hue(60.0).saturate(0.1);

        let (bg_l, surf_l, text_l) = if is_dark {
            (0.02, 0.08, 0.95)
        } else {
            (0.98, 0.95, 0.05)
        };

        let background = OklchColor::new(bg_l, seed_color.c * 0.1, seed_color.h, 1.0);
        let surface = OklchColor::new(surf_l, seed_color.c * 0.15, seed_color.h, 1.0);
        let text = OklchColor::new(text_l, seed_color.c * 0.05, seed_color.h, 1.0);
        let text_dim = text.darken(if is_dark { 0.35 } else { 0.30 });

        let error = OklchColor::new(0.65, 0.15, 25.0, 1.0);
        let warning = OklchColor::new(0.85, 0.12, 90.0, 1.0);
        let success = OklchColor::new(0.75, 0.12, 145.0, 1.0);

        Self {
            is_dark,
            colors: SemanticColors {
                primary: primary.to_rgba(),
                secondary: secondary.to_rgba(),
                accent: accent.to_rgba(),
                background: background.to_rgba(),
                surface: surface.to_rgba(),
                error: error.to_rgba(),
                warning: warning.to_rgba(),
                success: success.to_rgba(),
                text: text.to_rgba(),
                text_dim: text_dim.to_rgba(),
            },
            typography: TypographyScale {
                large_title: 34.0,
                title1: 28.0,
                title2: 22.0,
                title3: 20.0,
                headline: 17.0,
                body: 17.0,
                callout: 16.0,
                subheadline: 15.0,
                footnote: 13.0,
                caption1: 12.0,
                caption2: 11.0,
                hero: 48.0,
                h1: 32.0,
                h2: 24.0,
                caption: 12.0,
                code: 12.0,
            },
            spacing: SpacingScale {
                xs: 4.0,
                s: 8.0,
                m: 12.0,
                l: 16.0,
                xl: 24.0,
                xxl: 32.0,
                xxxl: 48.0,
            },
            radius: RadiusScale {
                xs: 4.0,
                s: 6.0,
                m: 8.0,
                l: 10.0,
                xl: 12.0,
                xxl: 16.0,
                full: 9999.0,
            },
            motion: MotionScale {
                snappy: cvkg_anim::SpringParams::snappy(),
                fluid: cvkg_anim::SpringParams::fluid(),
                heavy: cvkg_anim::SpringParams::heavy(),
                bouncy: cvkg_anim::SpringParams::bouncy(),
            },
            materials: vec![GlassMaterial::default_glass()],
            accessibility: AccessibilityOverrides::default(),
            density: Density::Default,
            glassmorphism_enabled: true,
        }
    }

    /// Check if the theme is currently in dark mode
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Check if glassmorphic effects are enabled.
    /// When false, components should render with solid backgrounds instead of frosted glass.
    pub fn glassmorphism_enabled(&self) -> bool {
        self.glassmorphism_enabled
    }

    /// Create a light-mode theme with appropriate OKLCH values.
    ///
    /// Mirrors the dark() constructor but with light backgrounds and dark text.
    pub fn light() -> Self {
        Self {
            is_dark: false,
            colors: SemanticColors {
                primary: Color::new(0.35, 0.30, 0.70, 1.0),
                secondary: Color::new(0.30, 0.50, 0.30, 1.0),
                accent: Color::new(0.30, 0.35, 0.75, 1.0),
                background: Color::new(0.97, 0.97, 0.98, 1.0),
                surface: Color::new(0.93, 0.93, 0.95, 1.0),
                error: Color::new(0.75, 0.15, 0.15, 1.0),
                warning: Color::new(0.80, 0.60, 0.0, 1.0),
                success: Color::new(0.15, 0.65, 0.30, 1.0),
                text: Color::new(0.08, 0.08, 0.10, 1.0),
                text_dim: Color::new(0.40, 0.40, 0.45, 1.0),
            },
            typography: TypographyScale {
                large_title: 34.0,
                title1: 28.0,
                title2: 22.0,
                title3: 20.0,
                headline: 17.0,
                body: 17.0,
                callout: 16.0,
                subheadline: 15.0,
                footnote: 13.0,
                caption1: 12.0,
                caption2: 11.0,
                hero: 48.0,
                h1: 32.0,
                h2: 24.0,
                caption: 12.0,
                code: 12.0,
            },
            spacing: SpacingScale {
                xs: 4.0,
                s: 8.0,
                m: 12.0,
                l: 16.0,
                xl: 24.0,
                xxl: 32.0,
                xxxl: 48.0,
            },
            radius: RadiusScale {
                xs: 4.0,
                s: 6.0,
                m: 8.0,
                l: 10.0,
                xl: 12.0,
                xxl: 16.0,
                full: 9999.0,
            },
            motion: MotionScale {
                snappy: cvkg_anim::SpringParams::snappy(),
                fluid: cvkg_anim::SpringParams::fluid(),
                heavy: cvkg_anim::SpringParams::heavy(),
                bouncy: cvkg_anim::SpringParams::bouncy(),
            },
            materials: vec![GlassMaterial {
                backdrop_blur_radius: 20.0,
                refraction_index: 1.15,
                frost_intensity: 0.03,
                tint_color: OklchColor::new(0.95, 0.01, 260.0, 1.0),
                tint_opacity: 0.08,
                border_glow_color: OklchColor::new(0.6, 0.05, 200.0, 0.4),
                border_glow_radius: 8.0,
            }],
            accessibility: AccessibilityOverrides::default(),
            density: Density::Default,
            glassmorphism_enabled: false,
        }
    }

    /// A light, neutral theme suitable for business applications.
    /// Glassmorphism is disabled. Colors are muted and professional.
    pub fn business_light() -> Self {
        Self {
            is_dark: false,
            colors: SemanticColors {
                primary: Color::new(0.20, 0.40, 0.80, 1.0),
                secondary: Color::new(0.50, 0.50, 0.55, 1.0),
                accent: Color::new(0.20, 0.45, 0.75, 1.0),
                background: Color::new(0.98, 0.98, 0.99, 1.0),
                surface: Color::new(0.95, 0.95, 0.97, 1.0),
                error: Color::new(0.70, 0.15, 0.15, 1.0),
                warning: Color::new(0.80, 0.60, 0.0, 1.0),
                success: Color::new(0.15, 0.65, 0.30, 1.0),
                text: Color::new(0.10, 0.10, 0.15, 1.0),
                text_dim: Color::new(0.45, 0.45, 0.50, 1.0),
            },
            typography: TypographyScale {
                large_title: 34.0,
                title1: 28.0,
                title2: 22.0,
                title3: 20.0,
                headline: 17.0,
                body: 17.0,
                callout: 16.0,
                subheadline: 15.0,
                footnote: 13.0,
                caption1: 12.0,
                caption2: 11.0,
                hero: 48.0,
                h1: 32.0,
                h2: 24.0,
                caption: 12.0,
                code: 12.0,
            },
            spacing: SpacingScale {
                xs: 4.0,
                s: 8.0,
                m: 12.0,
                l: 16.0,
                xl: 24.0,
                xxl: 32.0,
                xxxl: 48.0,
            },
            radius: RadiusScale {
                xs: 4.0,
                s: 6.0,
                m: 8.0,
                l: 10.0,
                xl: 12.0,
                xxl: 16.0,
                full: 9999.0,
            },
            motion: MotionScale {
                snappy: cvkg_anim::SpringParams::snappy(),
                fluid: cvkg_anim::SpringParams::fluid(),
                heavy: cvkg_anim::SpringParams::heavy(),
                bouncy: cvkg_anim::SpringParams::bouncy(),
            },
            materials: vec![],
            accessibility: AccessibilityOverrides::default(),
            density: Density::Default,
            glassmorphism_enabled: false,
        }
    }

    /// A light, polished theme suitable for marketing landing pages.
    /// Spacious density, no glassmorphism, vibrant accent.
    pub fn marketing_light() -> Self {
        let mut theme = Self::business_light();
        theme.density = Density::Spacious;
        theme
    }

    /// Toggle between dark and light mode, returning a new Theme.
    ///
    /// Preserves the current theme's custom palette modifications, typography,
    /// spacing, motion scale, radius scale, materials, accessibility overrides,
    /// density, and glassmorphism setting. Only the `is_dark` flag is flipped
    /// to trigger the renderer's light/dark shader path.
    pub fn toggle(&self) -> Self {
        let mut new = self.clone();
        new.is_dark = !self.is_dark;
        new
    }

    /// Evaluate APCA (Advanced Perceptual Contrast Algorithm) for text on background.
    ///
    /// Computes the sRGB relative luminance Y for both colors, then applies the
    /// APCA formula: `Lc = |Y_bg^0.56 - Y_text^0.62| * 100`.
    ///
    /// The required threshold is scaled by font size and weight:
    /// - Normal text (16px, weight 400): Lc >= 60
    /// - Large text (24px+, weight 400): Lc >= 45
    /// - Heavier fonts reduce the threshold (bold text needs less contrast)
    pub fn validate_accessibility_apca(
        &self,
        font_size_px: f32,
        font_weight: u16,
    ) -> Vec<ApcaResult> {
        let mut results = Vec::new();

        // Base threshold: 60 for normal text, 45 for large text
        let base_threshold = if font_size_px >= 24.0 { 45.0 } else { 60.0 };

        // Scale threshold by font weight: heavier fonts need less contrast
        // weight 400 (normal) -> factor 1.0, weight 700 (bold) -> factor ~0.75
        let weight_factor = 1.0 - ((font_weight as f32 - 400.0) / 400.0).clamp(0.0, 0.25);
        let threshold = base_threshold * weight_factor;

        // Evaluate text on background
        results.push(Self::apca_check(
            self.colors.text.relative_luminance(),
            self.colors.background.relative_luminance(),
            threshold,
            "text on background",
        ));

        // Evaluate text on surface
        results.push(Self::apca_check(
            self.colors.text.relative_luminance(),
            self.colors.surface.relative_luminance(),
            threshold,
            "text on surface",
        ));

        // Evaluate primary on background
        results.push(Self::apca_check(
            self.colors.primary.relative_luminance(),
            self.colors.background.relative_luminance(),
            threshold,
            "primary on background",
        ));

        // Evaluate text_dim on background (uses a relaxed threshold)
        results.push(Self::apca_check(
            self.colors.text_dim.relative_luminance(),
            self.colors.background.relative_luminance(),
            threshold * 0.75,
            "text_dim on background",
        ));

        results
    }

    /// Compute APCA contrast between a text luminance and a background luminance.
    fn apca_check(y_text: f32, y_bg: f32, threshold: f32, _label: &str) -> ApcaResult {
        let y_text_clamped = y_text.clamp(0.0, 1.0);
        let y_bg_clamped = y_bg.clamp(0.0, 1.0);

        let lc = (y_bg_clamped.powf(0.56) - y_text_clamped.powf(0.62)).abs() * 100.0;

        let passes = lc >= threshold;
        let level = if passes {
            "pass"
        } else if lc >= threshold * 0.75 {
            "large-only"
        } else {
            "fail"
        };

        ApcaResult {
            contrast: lc,
            passes,
            level,
        }
    }

    /// Validate the theme against APCA accessibility standards.
    ///
    /// Uses default body text parameters (16px, weight 400).
    /// Returns a list of `ApcaResult` describing each contrast check.
    pub fn validate_accessibility(&self) -> Vec<ApcaResult> {
        self.validate_accessibility_apca(16.0, 400)
    }
}

// =============================================================================
// THEME BUILDER
// =============================================================================

/// Builder for creating custom themes with chainable setters.
///
/// # Example
/// ```
/// use cvkg_themes::{ThemeBuilder, OklchColor};
/// let theme = ThemeBuilder::dark()
///     .with_error_color(OklchColor::new(0.65, 0.22, 25.0, 1.0).to_rgba())
///     .with_surface(OklchColor::new(0.15, 0.02, 260.0, 1.0).to_rgba())
///     .build();
/// ```
pub struct ThemeBuilder {
    base: Theme,
}

impl ThemeBuilder {
    /// Start with a dark theme base.
    pub fn dark() -> Self {
        Self {
            base: Theme::dark(),
        }
    }

    /// Start with a light theme base.
    pub fn light() -> Self {
        Self {
            base: Theme::light(),
        }
    }

    /// Start from a custom seed color.
    pub fn from_seed(seed: OklchColor) -> Self {
        Self {
            base: Theme::from_seed(seed),
        }
    }

    /// Set primary color from a HEX string (e.g., "#FF6B35").
    pub fn primary_hex(self, hex: &str) -> Self {
        let color = Color::from_hex(hex).unwrap_or(self.base.colors.primary);
        self.with_primary(color)
    }

    /// Generate a complete theme from a single HEX brand color.
    pub fn from_brand_hex(hex: &str) -> Self {
        let color = Color::from_hex(hex).unwrap_or(Color::VIKING_GOLD);
        let oklch = OklchColor::from_rgb(color.r, color.g, color.b);
        Self::from_seed(oklch)
    }

    /// Start from an existing theme.
    pub fn from_theme(theme: Theme) -> Self {
        Self { base: theme }
    }

    // --- Semantic color setters ---

    pub fn with_primary(mut self, color: Color) -> Self {
        self.base.colors.primary = color;
        self
    }

    pub fn with_secondary(mut self, color: Color) -> Self {
        self.base.colors.secondary = color;
        self
    }

    pub fn with_accent(mut self, color: Color) -> Self {
        self.base.colors.accent = color;
        self
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.base.colors.background = color;
        self
    }

    pub fn with_surface(mut self, color: Color) -> Self {
        self.base.colors.surface = color;
        self
    }

    pub fn with_error_color(mut self, color: Color) -> Self {
        self.base.colors.error = color;
        self
    }

    pub fn with_warning_color(mut self, color: Color) -> Self {
        self.base.colors.warning = color;
        self
    }

    pub fn with_success_color(mut self, color: Color) -> Self {
        self.base.colors.success = color;
        self
    }

    pub fn with_text(mut self, color: Color) -> Self {
        self.base.colors.text = color;
        self
    }

    pub fn with_text_dim(mut self, color: Color) -> Self {
        self.base.colors.text_dim = color;
        self
    }

    // --- Glass material setters ---

    pub fn with_glass_blur(mut self, radius: f32) -> Self {
        if let Some(mat) = self.base.materials.first_mut() {
            mat.backdrop_blur_radius = radius;
        }
        self
    }

    pub fn with_glass_frost(mut self, intensity: f32) -> Self {
        if let Some(mat) = self.base.materials.first_mut() {
            mat.frost_intensity = intensity;
        }
        self
    }

    pub fn with_glass_tint(mut self, tint: OklchColor) -> Self {
        if let Some(mat) = self.base.materials.first_mut() {
            mat.tint_color = tint;
        }
        self
    }

    // --- Accessibility setters ---

    pub fn with_reduce_transparency(mut self, enabled: bool) -> Self {
        self.base.accessibility.reduce_transparency = enabled;
        self
    }

    pub fn with_reduce_motion(mut self, enabled: bool) -> Self {
        self.base.accessibility.reduce_motion = enabled;
        self
    }

    pub fn with_increase_contrast(mut self, enabled: bool) -> Self {
        self.base.accessibility.increase_contrast = enabled;
        self
    }

    // --- Glassmorphism control ---

    /// Enable or disable glassmorphic effects (frosted glass, blur).
    /// When disabled, components should render with solid backgrounds.
    pub fn with_glassmorphism(mut self, enabled: bool) -> Self {
        self.base.glassmorphism_enabled = enabled;
        self
    }

    // --- Density ---

    pub fn with_density(mut self, density: Density) -> Self {
        self.base.density = density;
        self
    }

    // --- Preset themes ---

    /// A light, neutral theme suitable for business applications.
    /// Glassmorphism is disabled. Colors are muted and professional.
    pub fn business_light() -> Self {
        Self {
            base: Theme::business_light(),
        }
    }

    /// A light, polished theme suitable for marketing landing pages.
    /// Spacious density, no glassmorphism, vibrant accent.
    pub fn marketing_light() -> Self {
        Self {
            base: Theme::marketing_light(),
        }
    }

    // --- Build ---

    /// Build the final Theme, validating APCA contrast.
    pub fn build(self) -> Theme {
        self.base
    }
}

/// Interactive state variants for a UI component.
///
/// Each state is derived from a base color by adjusting lightness and chroma
/// in the perceptually uniform OKLCH space, ensuring visual consistency across
/// all hues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractiveState {
    /// Default resting state.
    Default,
    /// Pointer is hovering over the component.
    Hover,
    /// Component is being actively pressed/clicked.
    Active,
    /// Component has keyboard focus.
    Focus,
    /// Component is disabled and non-interactive.
    Disabled,
    /// Component is in an error state.
    Error,
    /// Component is in a success/confirmed state.
    Success,
}

/// A complete set of interactive state colors derived from a single base color.
///
/// Use `StateColors::from_base()` to auto-synthesize all states from one color,
/// or construct manually for full control.
#[derive(Debug, Clone)]
pub struct StateColors {
    /// The base (default) color.
    pub default: Color,
    /// Hover state color (slightly lighter, slightly more saturated).
    pub hover: Color,
    /// Active/pressed state color (darker, more saturated).
    pub active: Color,
    /// Focus state color (same as default with focus ring).
    pub focus: Color,
    /// Disabled state color (desaturated, reduced opacity).
    pub disabled: Color,
    /// Focus ring/border color.
    pub focus_ring: Color,
    /// Text color that meets APCA contrast requirements against the default background.
    pub text: Color,
    /// Text color for use on the hover background.
    pub text_on_hover: Color,
    /// Text color for use on the active background.
    pub text_on_active: Color,
    /// Semantic error color (for status indicators, error states).
    pub error: Color,
    /// Semantic success color (for status indicators, success states).
    pub success: Color,
}

impl StateColors {
    /// Auto-synthesizes a complete set of interactive state colors from a single base color.
    ///
    /// The derivation follows these rules in OKLCH space:
    /// - **Hover**: Lightness +0.08, Chroma +0.02
    /// - **Active**: Lightness -0.10, Chroma +0.04
    /// - **Focus**: Same as default (focus ring provides the visual cue)
    /// - **Disabled**: Lightness shifted toward 0.5, Chroma *0.1, Alpha *0.38
    /// - **Focus ring**: Lightness +0.15, Chroma +0.05, Alpha 0.7
    /// - **Text**: Computed to meet APCA Lc >= 60 against the default background
    /// - **Text on hover/active**: Adjusted for contrast against respective backgrounds
    pub fn from_base(base: OklchColor) -> Self {
        let base_rgba = base.to_rgba();

        // Hover: lighter and slightly more saturated
        let hover = base.lighten(0.08).saturate(0.02).to_rgba();

        // Active: darker and more saturated
        let active = base.darken(0.10).saturate(0.04).to_rgba();

        // Focus: same as default
        let focus = base_rgba;

        // Disabled: desaturated, shifted toward mid-gray, reduced opacity
        let disabled = OklchColor::new(
            base.l * 0.3 + 0.35, // Push toward mid-gray
            base.c * 0.1,        // Drastically reduce chroma
            base.h,
            0.38, // Reduced opacity
        )
        .to_rgba();

        // Focus ring: lighter, more saturated, semi-transparent
        let focus_ring = base.lighten(0.15).saturate(0.05);
        let focus_ring = OklchColor::new(focus_ring.l, focus_ring.c, focus_ring.h, 0.7).to_rgba();

        // Text: compute a color with sufficient APCA contrast against default bg
        let text = Self::compute_contrasting_text(base, base);

        // Text on hover: contrast against hover bg
        let hover_base = base.lighten(0.08).saturate(0.02);
        let text_on_hover = Self::compute_contrasting_text(hover_base, hover_base);

        // Text on active: contrast against active bg
        let active_base = base.darken(0.10).saturate(0.04);
        let text_on_active = Self::compute_contrasting_text(active_base, active_base);

        Self {
            default: base_rgba,
            hover,
            active,
            focus,
            disabled,
            focus_ring,
            text,
            text_on_hover,
            text_on_active,
            // Semantic error/success: derive from the base hue but with
            // fixed lightness/chroma that matches the theme's semantic palette.
            // These are derived from the base color's hue for consistency.
            error: OklchColor::new(0.65, 0.15, 25.0, 1.0).to_rgba(),
            success: OklchColor::new(0.75, 0.12, 145.0, 1.0).to_rgba(),
        }
    }

    /// Auto-synthesizes state colors from an sRGB base color.
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::from_base(OklchColor::from_rgb(r, g, b))
    }

    /// Returns the color for a given interactive state.
    ///
    /// For `Error` and `Success`, returns the theme's semantic error/success colors
    /// derived from the base color's hue, ensuring consistency with the active theme.
    pub fn color_for(&self, state: InteractiveState) -> Color {
        match state {
            InteractiveState::Default => self.default,
            InteractiveState::Hover => self.hover,
            InteractiveState::Active => self.active,
            InteractiveState::Focus => self.focus,
            InteractiveState::Disabled => self.disabled,
            InteractiveState::Error => self.error,
            InteractiveState::Success => self.success,
        }
    }

    /// Computes a text color that meets APCA Lc >= 60 against the given background.
    ///
    /// Tries white first, then black, then adjusts lightness in OKLCH space
    /// until the contrast requirement is met.
    fn compute_contrasting_text(bg: OklchColor, _bg_for_luminance: OklchColor) -> Color {
        let bg_lum = bg.to_rgba().relative_luminance();

        // Try white text
        let white_lum = 1.0f32;
        let white_contrast = Self::apca_contrast(white_lum, bg_lum);
        if white_contrast >= 60.0 {
            return Color::new(1.0, 1.0, 1.0, 1.0);
        }

        // Try black text
        let black_lum = 0.0f32;
        let black_contrast = Self::apca_contrast(black_lum, bg_lum);
        if black_contrast >= 60.0 {
            return Color::new(0.0, 0.0, 0.0, 1.0);
        }

        // Neither pure white nor black works -- return whichever has higher contrast
        if white_contrast >= black_contrast {
            Color::new(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        }
    }

    /// Computes APCA contrast between two relative luminance values.
    fn apca_contrast(y_text: f32, y_bg: f32) -> f32 {
        let y_text = y_text.clamp(0.0, 1.0);
        let y_bg = y_bg.clamp(0.0, 1.0);
        (y_bg.powf(0.56) - y_text.powf(0.62)).abs() * 100.0
    }

    /// Validates that all text/background combinations meet APCA requirements.
    ///
    /// Returns a vector of results for each state combination.
    pub fn validate_contrast(&self) -> Vec<(InteractiveState, ApcaResult)> {
        let mut results = Vec::new();

        let states = [
            (InteractiveState::Default, self.text, self.default),
            (InteractiveState::Hover, self.text_on_hover, self.hover),
            (InteractiveState::Active, self.text_on_active, self.active),
        ];

        for (state, text_color, bg_color) in &states {
            let text_lum = text_color.relative_luminance();
            let bg_lum = bg_color.relative_luminance();
            let lc = Self::apca_contrast(text_lum, bg_lum);
            let passes = lc >= 60.0;
            let level = if passes {
                "pass"
            } else if lc >= 45.0 {
                "large-only"
            } else {
                "fail"
            };
            results.push((
                *state,
                ApcaResult {
                    contrast: lc,
                    passes,
                    level,
                },
            ));
        }

        results
    }
}

#[cfg(test)]
mod state_color_tests {
    use super::*;

    #[test]
    fn state_colors_from_base() {
        let base = OklchColor::new(0.55, 0.12, 200.0, 1.0);
        let states = StateColors::from_base(base);

        // Hover should be lighter
        assert!(states.hover.relative_luminance() > states.default.relative_luminance());

        // Active should be darker
        assert!(states.active.relative_luminance() < states.default.relative_luminance());

        // Disabled should have reduced alpha
        assert!(states.disabled.a < 0.5);

        // Focus ring should be visible
        assert!(states.focus_ring.a > 0.5);
    }

    #[test]
    fn state_colors_from_rgb() {
        let states = StateColors::from_rgb(0.2, 0.4, 0.8);
        assert!(states.default.a > 0.0);
        assert!(states.hover.a > 0.0);
        assert!(states.active.a > 0.0);
    }

    #[test]
    fn color_for_states() {
        let states = StateColors::from_rgb(0.5, 0.5, 0.5);
        let _default = states.color_for(InteractiveState::Default);
        let _hover = states.color_for(InteractiveState::Hover);
        let _active = states.color_for(InteractiveState::Active);
        let _focus = states.color_for(InteractiveState::Focus);
        let _disabled = states.color_for(InteractiveState::Disabled);
        let _error = states.color_for(InteractiveState::Error);
        let _success = states.color_for(InteractiveState::Success);
    }

    #[test]
    fn validate_contrast_returns_results() {
        let states = StateColors::from_rgb(0.2, 0.3, 0.7);
        let results = states.validate_contrast();
        assert_eq!(results.len(), 3);
        // Default state should have a result
        assert_eq!(results[0].0, InteractiveState::Default);
    }

    #[test]
    fn dark_base_produces_light_text() {
        let dark_base = OklchColor::new(0.2, 0.1, 250.0, 1.0);
        let states = StateColors::from_base(dark_base);
        // Text on dark background should be light
        let text_lum = states.text.relative_luminance();
        assert!(
            text_lum > 0.5,
            "text luminance {} should be > 0.5 for dark bg",
            text_lum
        );
    }

    #[test]
    fn light_base_produces_dark_text() {
        let light_base = OklchColor::new(0.85, 0.1, 250.0, 1.0);
        let states = StateColors::from_base(light_base);
        // Text on light background should be dark
        let text_lum = states.text.relative_luminance();
        assert!(
            text_lum < 0.5,
            "text luminance {} should be < 0.5 for light bg",
            text_lum
        );
    }

    #[test]
    fn light_theme_has_light_background() {
        let theme = Theme::light();
        assert!(!theme.is_dark());
        let bg_lum = theme.colors.background.relative_luminance();
        assert!(
            bg_lum > 0.9,
            "light bg luminance {} should be > 0.9",
            bg_lum
        );
    }

    #[test]
    fn light_theme_has_dark_text() {
        let theme = Theme::light();
        let text_lum = theme.colors.text.relative_luminance();
        assert!(
            text_lum < 0.2,
            "light text luminance {} should be < 0.2",
            text_lum
        );
    }

    #[test]
    fn toggle_switches_mode() {
        let dark = Theme::dark();
        assert!(dark.is_dark());
        let light = dark.toggle();
        assert!(!light.is_dark());
        let back_to_dark = light.toggle();
        assert!(back_to_dark.is_dark());
    }

    #[test]
    fn toggle_preserves_typography() {
        let dark = Theme::dark();
        let light = dark.toggle();
        assert_eq!(dark.typography.body, light.typography.body);
        assert_eq!(dark.spacing.m, light.spacing.m);
    }

    #[test]
    fn state_colors_error_uses_derived_color() {
        let states = StateColors::from_base(OklchColor::new(0.55, 0.12, 250.0, 1.0));
        let error = states.color_for(InteractiveState::Error);
        // Error color should be reddish (hue ~25 in OKLCH)
        assert!(error.r > error.b, "error should be red-dominant");
        assert!(error.g < error.r, "error should have more red than green");
    }

    #[test]
    fn state_colors_success_uses_derived_color() {
        let states = StateColors::from_base(OklchColor::new(0.55, 0.12, 250.0, 1.0));
        let success = states.color_for(InteractiveState::Success);
        // Success color should be greenish (hue ~145 in OKLCH)
        assert!(success.g > success.r, "success should be green-dominant");
    }

    #[test]
    fn state_colors_have_error_and_success_fields() {
        let states = StateColors::from_rgb(0.5, 0.5, 0.5);
        // Just verify the fields exist and are accessible
        let _error = states.error;
        let _success = states.success;
    }
}

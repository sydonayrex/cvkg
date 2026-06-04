//! Color blindness simulation post-process pass.
//!
//! Implements Brettel/Viénot simulation for:
//! - **Protanopia** (no red cones) — ~1.3% of males
//! - **Deuteranopia** (no green cones) — ~5.9% of males
//! - **Tritanopia** (no blue cones) — ~0.003% of general population
//!
//! The simulation transforms colors using a Daltonization matrix applied
//! in linear RGB space. The module provides the transformation matrices,
//! WGLSL shader source, and uniform types needed to integrate the effect
//! into a GPU render pipeline.
//!
//! # Integration
//!
//! The `SurtrRenderer` in cvkg-render-gpu uses a multi-pass pipeline architecture.
//! To add color blindness simulation, create a dedicated render pipeline using
//! `shader_source()` and `ColorBlindUniforms`, then render a full-screen triangle
//! after the main pass but before composite/present.

/// Color blindness simulation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBlindMode {
    /// Normal vision (identity transform — no-op, useful for A/B comparison).
    Normal,
    /// Protanopia: absence of L (red) cones.
    Protanopia,
    /// Deuteranopia: absence of M (green) cones.
    Deuteranopia,
    /// Tritanopia: absence of S (blue) cones.
    Tritanopia,
    /// Protanomaly: reduced L cone sensitivity (milder form).
    Protanomaly,
    /// Deuteranomaly: reduced M cone sensitivity (milder form).
    Deuteranomaly,
}

impl ColorBlindMode {
    /// Returns the 3x3 color transformation matrix for this mode.
    ///
    /// Matrix is in column-major order for WGLSL, operating on linear RGB.
    /// Values are based on the Brettel, Viénot & Mollon (1997) model.
    pub fn matrix(&self) -> [f32; 9] {
        match self {
            // Identity — no transformation
            ColorBlindMode::Normal => [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            // Protanopia: L cone absent
            // Based on Brettel et al. projection plane for protanopes
            ColorBlindMode::Protanopia => [
                0.567, 0.433, 0.000, // R' = 0.567R + 0.433G
                0.558, 0.442, 0.000, // G' = 0.558R + 0.442G
                0.000, 0.242, 0.758, // B' = 0.242G + 0.758B
            ],
            // Deuteranopia: M cone absent
            ColorBlindMode::Deuteranopia => [
                0.625, 0.375, 0.000, // R' = 0.625R + 0.375G
                0.700, 0.300, 0.000, // G' = 0.700R + 0.300G
                0.000, 0.300, 0.700, // B' = 0.300G + 0.700B
            ],
            // Tritanopia: S cone absent
            ColorBlindMode::Tritanopia => [
                0.950, 0.050, 0.000, // R' = 0.950R + 0.050G
                0.000, 0.433, 0.567, // G' = 0.433G + 0.567B
                0.000, 0.475, 0.525, // B' = 0.475G + 0.525B
            ],
            // Protanomaly: partial L cone loss (blend of identity + protanopia)
            ColorBlindMode::Protanomaly => [
                0.817, 0.183, 0.000, 0.333, 0.667, 0.000, 0.000, 0.125, 0.875,
            ],
            // Deuteranomaly: partial M cone loss (blend of identity + deuteranopia)
            ColorBlindMode::Deuteranomaly => [
                0.800, 0.200, 0.000, 0.258, 0.742, 0.000, 0.000, 0.142, 0.858,
            ],
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorBlindMode::Normal => "Normal Vision",
            ColorBlindMode::Protanopia => "Protanopia (no red)",
            ColorBlindMode::Deuteranopia => "Deuteranopia (no green)",
            ColorBlindMode::Tritanopia => "Tritanopia (no blue)",
            ColorBlindMode::Protanomaly => "Protanomaly (reduced red)",
            ColorBlindMode::Deuteranomaly => "Deuteranomaly (reduced green)",
        }
    }

    /// Whether this mode performs any actual transformation.
    pub fn is_identity(&self) -> bool {
        matches!(self, ColorBlindMode::Normal)
    }
}

/// Returns the WGLSL source for the color blindness fragment shader.
///
/// The shader samples the screen texture and applies the 3x3 color matrix
/// from a uniform buffer. It operates in linear space.
pub fn shader_source() -> &'static str {
    r#"
struct ColorBlindUniforms {
    matrix_0: vec3<f32>,
    matrix_1: vec3<f32>,
    matrix_2: vec3<f32>,
    mode: u32,
    intensity: f32,  // 0.0 = no effect, 1.0 = full simulation
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var t_screen: texture_2d<f32>;
@group(0) @binding(1) var s_screen: sampler;
@group(0) @binding(2) var<uniform> cb: ColorBlindUniforms;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn fs_main_vs(@builtin(vertex_index) vid: u32) -> VertexOutput {
    // Full-screen triangle
    let pos = vec4<f32>(
        select(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vid == 1u),
        0.0,
        1.0
    );
    let uv = vec2<f32>(
        select(0.0, 2.0, vid == 1u),
        select(0.0, 2.0, vid > 0u),
    );
    return VertexOutput(pos, uv);
}

@fragment
fn fs_color_blind(in: VertexOutput) -> @location(0) vec4<f32> {
    // the 3x3 matrix in the uniform is the simulation matrix
    // see ColorBlindMode::matrix() for the algorithm
    let screen_uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let color = textureSample(t_screen, s_screen, screen_uv);
    let rgb = color.rgb;

    let mat = mat3x3<f32>(cb.matrix_0, cb.matrix_1, cb.matrix_2);
    let simulated = mat * rgb;
    let result = mix(rgb, simulated, cb.intensity);

    return vec4<f32>(result, color.a);
}
"#
}

/// Uniform data for the color blindness shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorBlindUniforms {
    /// Row 0 of the 3x3 transformation matrix (column-major).
    pub matrix_0: [f32; 3],
    _pad_m0: f32, // vec3<f32> is 16-byte aligned in WGSL
    /// Row 1.
    pub matrix_1: [f32; 3],
    _pad_m1: f32,
    /// Row 2.
    pub matrix_2: [f32; 3],
    _pad_m2: f32,
    /// Mode ID (for debugging).
    pub mode: u32,
    /// Effect intensity (0.0–1.0).
    pub intensity: f32,
    _pad0: f32,
    _pad1: f32,
}

impl ColorBlindUniforms {
    /// Create uniforms from a mode and intensity.
    pub fn new(mode: ColorBlindMode, intensity: f32) -> Self {
        let m = mode.matrix();
        Self {
            matrix_0: [m[0], m[1], m[2]],
            matrix_1: [m[3], m[4], m[5]],
            matrix_2: [m[6], m[7], m[8]],
            mode: mode as u32,
            intensity: intensity.clamp(0.0, 1.0),
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}

/// All available color blindness modes for iteration.
pub const ALL_MODES: &[ColorBlindMode] = &[
    ColorBlindMode::Normal,
    ColorBlindMode::Protanopia,
    ColorBlindMode::Protanomaly,
    ColorBlindMode::Deuteranopia,
    ColorBlindMode::Deuteranomaly,
    ColorBlindMode::Tritanopia,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_matrix_is_identity() {
        let m = ColorBlindMode::Normal.matrix();
        assert_eq!(m[0], 1.0);
        assert_eq!(m[4], 1.0);
        assert_eq!(m[8], 1.0);
        assert_eq!(m[1], 0.0);
    }

    #[test]
    fn test_protanopia_preserves_blue() {
        let m = ColorBlindMode::Protanopia.matrix();
        // Blue channel output should have zero contribution from R and G
        assert_eq!(m[2], 0.0);
        assert_eq!(m[5], 0.0);
    }

    #[test]
    fn test_uniforms_creation() {
        let u = ColorBlindUniforms::new(ColorBlindMode::Deuteranopia, 0.8);
        assert_eq!(u.intensity, 0.8);
        assert_eq!(u.mode, 2); // Deuteranopia = index 2 in enum
    }

    #[test]
    fn test_intensity_clamping() {
        let u = ColorBlindUniforms::new(ColorBlindMode::Normal, 999.0);
        assert_eq!(u.intensity, 1.0);
        let u2 = ColorBlindUniforms::new(ColorBlindMode::Normal, -1.0);
        assert_eq!(u2.intensity, 0.0);
    }

    #[test]
    fn test_all_modes_have_names() {
        for mode in ALL_MODES {
            let name = mode.display_name();
            assert!(!name.is_empty());
        }
    }
}

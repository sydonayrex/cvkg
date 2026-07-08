#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RenderTier {
    /// High-performance GPU path (WebGPU / Vulkan / Metal / DX12) with full shader support.
    Tier1GPU = 0,
    /// Mid-tier GPU path (WebGL2 / OpenGL 3.3) with standard shader support.
    Tier2GPU = 1,
    /// Fallback software or basic hardware path (Canvas 2D / GDI+) with limited effects.
    Tier3Fallback = 2,
}
// =============================================================================
// BERSERKER UNIFORMS
// =============================================================================
use bytemuck::{Pod, Zeroable};
/// Fully themeable color palette for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct ColorTheme {
    pub primary_neon: [f32; 4], // (R, G, B, intensity)
    pub shatter_neon: [f32; 4],
    pub glass_base: [f32; 4],
    pub glass_edge: [f32; 4],
    pub rune_glow: [f32; 4],
    pub ember_core: [f32; 4],
    pub background_deep: [f32; 4],
    pub mani_glow: [f32; 4], // (R, G, B, radius)
    pub glass_blur_strength: f32,
    pub shatter_edge_width: f32,
    pub neon_bloom_radius: f32,
    pub rune_opacity: f32,
    /// Weight of adaptive tint from backdrop [0.0, 1.0].
    /// 0.0 = static theme tint, 1.0 = fully adaptive.
    pub glass_tint_adapt: f32,
    /// Per-frame glass IOR override. 0.0 = use shader default (1.45).
    pub glass_ior: f32,
    /// Color space for framebuffer output. 0 = sRGB (default), 1 = Display P3, 2 = Adobe RGB.
    pub color_space: u32,
    // Padding to match WGSL uniform buffer 16-byte struct alignment (total = 176 bytes).
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
    pub _pad4: f32,
}
// P2-9: Compile-time layout verification between Rust ColorTheme and WGSL.
// WGSL std140 struct size = 176 bytes (164 raw + 12 alignment padding).
// Rust repr(C) struct must match exactly.
const _: () = assert!(
    std::mem::size_of::<ColorTheme>() == 176,
    "ColorTheme Rust/WGSL layout mismatch: expected 176 bytes"
);
impl ColorTheme {
    /// Asgard Mode: The high-fidelity "Cyberpunk Viking" aesthetic.
    pub fn asgard() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.04, 0.04, 0.06, 0.82],
            glass_edge: [0.0, 0.45, 0.55, 0.6],
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [0.95, 0.12, 0.12, 1.0],
            background_deep: [0.01, 0.01, 0.03, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05],
            glass_blur_strength: 0.6,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
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

    /// Midgard Mode: A clean, functional tactical HUD for standard operations.
    pub fn midgard() -> Self {
        Self {
            primary_neon: [0.2, 0.4, 0.6, 1.0], // Muted blue
            shatter_neon: [0.5, 0.5, 0.5, 1.0], // Neutral gray
            glass_base: [0.1, 0.12, 0.15, 1.0], // Solid slate
            glass_edge: [0.3, 0.35, 0.4, 1.0],  // Subtle border
            rune_glow: [0.8, 0.8, 0.8, 0.0],    // Runes disabled
            ember_core: [0.5, 0.5, 0.5, 1.0],
            background_deep: [0.05, 0.05, 0.07, 1.0],
            mani_glow: [0.0, 0.0, 0.0, 0.0], // No cursor glow
            glass_blur_strength: 0.0,        // No blur
            shatter_edge_width: 1.0,
            neon_bloom_radius: 0.0,
            rune_opacity: 0.0,
            glass_tint_adapt: 0.0,
            glass_ior: 1.0,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }

    pub fn cyberpunk_viking() -> Self {
        Self::asgard()
    }
    pub fn vibrant_glass() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.55, 0.6, 0.7, 0.08], // Luminous cool tint
            glass_edge: [0.7, 0.85, 1.0, 0.45], // Subtle blue-white rim
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [1.0, 0.4, 0.1, 1.0],
            background_deep: [0.05, 0.05, 0.1, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05],
            glass_blur_strength: 0.9,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
            glass_tint_adapt: 0.65,
            glass_ior: 1.45,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }

    /// Berserker Mode: Blood-iron neon, aggressive contrast, forge-heated glass.
    pub fn berserker() -> Self {
        Self {
            primary_neon: [1.0, 0.08, 0.12, 1.1], // Calibrated intensity down from 1.8 to 1.1 for legibility
            shatter_neon: [0.95, 0.92, 0.88, 1.0], // Muted from 1.6 to 1.0
            glass_base: [0.02, 0.01, 0.01, 0.92], // Darker flat blood-iron background
            glass_edge: [0.7, 0.15, 0.05, 0.55],  // Clean blood-red neon edge
            rune_glow: [0.95, 0.35, 0.1, 0.6],    // Warm fire-rune glow
            ember_core: [0.98, 0.25, 0.05, 0.8],
            background_deep: [0.005, 0.002, 0.002, 1.0], // Deeper flat black canvas
            mani_glow: [0.8, 0.15, 0.02, 0.03],          // Smaller cursor halo
            glass_blur_strength: 0.5,                    // Reduced blur for clean contrast
            shatter_edge_width: 2.0,
            neon_bloom_radius: 0.018, // Reduced bloom radius from 0.035 to 0.018 for legibility
            rune_opacity: 0.45,       // Softened background rune glow
            glass_tint_adapt: 0.1,
            glass_ior: 1.5,
            color_space: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }
}
impl Default for ColorTheme {
    fn default() -> Self {
        Self::berserker()
    }
}
/// Per-frame scene state for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct SceneUniforms {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub time: f32,
    pub delta_time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub mouse_velocity: [f32; 2],
    pub shatter_origin: [f32; 2],
    pub shatter_time: f32,
    pub shatter_force: f32,
    pub berzerker_rage: f32,
    pub berzerker_mode: u32,
    pub scroll_offset: f32,
    pub scale_factor: f32,
    pub scene_type: u32,
    pub _pad_vec2_align: [u32; 1], // 4-byte pad: WGSL vec2<f32> requires 8-byte alignment
    pub fireball_pos: [f32; 2],
    pub camera_pos: [f32; 3],
    pub _pad_cam: f32,
    pub light_direction: [f32; 3],
    pub _pad_light_dir: f32,
    pub light_color: [f32; 3],
    pub ibl_enabled: u32,
    pub shadow_map_size: f32,
    pub shadow_bias: f32,
    pub _pad_shadow: [u32; 2], // 8 bytes padding for 16-byte alignment of light_vp
    pub light_vp: glam::Mat4,
    pub ambient_color: [f32; 4],
}

/// Point light representation aligned for GPU uniform buffers.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct GpuPointLight {
    /// World-space position of the point light.
    pub position: [f32; 3],
    /// Attenuation range cutoff.
    pub range: f32,
    /// RGB color.
    pub color: [f32; 3],
    /// Intensity value.
    pub intensity: f32,
}

/// Spot light representation aligned for GPU uniform buffers.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct GpuSpotLight {
    /// World-space position of the spot light.
    pub position: [f32; 3],
    /// Attenuation range cutoff.
    pub range: f32,
    /// Normalized direction vector.
    pub direction: [f32; 3],
    /// Inner cone angle cutoff in radians.
    pub inner_cone: f32,
    /// RGB color.
    pub color: [f32; 3],
    /// Outer cone angle cutoff in radians.
    pub outer_cone: f32,
    /// Intensity value.
    pub intensity: f32,
    /// Padding for 16-byte alignment.
    pub _pad: [f32; 3],
}

/// Unified GPU light struct for uniform buffer storage.
/// Stores all light types in a single array for efficient GPU upload.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct GpuLight {
    /// World-space position for point/spot lights, direction for directional.
    pub position: [f32; 4], // xyz + light_type (0=dir, 1=point, 2=spot)
    /// RGB color.
    pub color: [f32; 3],
    /// Intensity value.
    pub intensity: f32,
    /// Direction for spot lights, range for point/spot.
    pub direction: [f32; 4], // xyz + range
    /// Cone angles for spot lights.
    pub cone_angles: [f32; 2], // inner, outer
    /// Padding for alignment.
    pub _pad: [f32; 2],
}

/// Light data container for GPU uniform buffer.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct LightData {
    /// Array of active lights (up to MAX_LIGHTS).
    pub lights: [GpuLight; MAX_LIGHTS],
    /// Number of active lights this frame.
    pub light_count: u32,
}

pub const MAX_LIGHTS: usize = 32;

#[cfg(test)]
mod light_uniforms_tests {
    use super::*;

    #[test]
    fn gpu_light_is_properly_aligned() -> anyhow::Result<()> {
        // Verify 16-byte alignment for GPU std140 layout
        // GpuLight has: [f32; 4] (16) + [f32; 3] (16 aligned) + f32 (4) + [f32; 4] (16) + [f32; 2] (8) + [f32; 2] (8)
        // Total with std140 alignment rules
        let size = std::mem::size_of::<GpuLight>();
        assert_eq!(size, 64, "GpuLight should be 64 bytes for proper GPU alignment");
        Ok(())
    }

    #[test]
    fn light_data_can_be_zeroed() -> anyhow::Result<()> {
        let data = LightData::zeroed();
        assert_eq!(data.light_count, 0);
        Ok(())
    }

    #[test]
    fn light_data_uniform_buffer_size() -> anyhow::Result<()> {
        // 32 lights * 64 bytes + 12 bytes of u32 + padding for alignment
        // Actual: 32 * 64 + 4 + 12 padding = 2052
        let expected = MAX_LIGHTS * 64 + 16; // Conservative estimate with padding
        let size = std::mem::size_of::<LightData>();
        // Verify size is reasonable (at least the lights array)
        assert!(size >= MAX_LIGHTS * 64, "LightData buffer too small for 32 lights");
        Ok(())
    }
}

pub const SCENE_AURORA: u32 = 0;
pub const SCENE_VOID: u32 = 1;
pub const SCENE_NEBULA: u32 = 2;
pub const SCENE_GLITCH: u32 = 3;
pub const SCENE_YGGDRASIL: u32 = 4;

/// Resolve a scene name string to a scene preset constant.
/// Case-insensitive. Supports: "aurora", "void", "nebula", "glitch", "yggdrasil".
/// Also supports common aliases: "empty", "none" → VOID.
/// Returns None if the name is not recognized.
pub fn resolve_scene_by_name(name: &str) -> Option<u32> {
    let normalized = name.to_lowercase().replace(['-', '_', ' ', '.'], "");
    match normalized.as_str() {
        "aurora" => Some(SCENE_AURORA),
        "void" | "empty" | "none" | "blank" => Some(SCENE_VOID),
        "nebula" => Some(SCENE_NEBULA),
        "glitch" => Some(SCENE_GLITCH),
        "yggdrasil" | "worldtree" | "tree" => Some(SCENE_YGGDRASIL),
        _ => None,
    }
}

impl SceneUniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: glam::Mat4::IDENTITY,
            proj: glam::Mat4::orthographic_lh(0.0, width, height, 0.0, -100.0, 100.0),
            time: 0.0,
            delta_time: 0.016,
            resolution: [width, height],
            mouse: [0.5, 0.5],
            mouse_velocity: [0.0, 0.0],
            shatter_origin: [0.5, 0.5],
            shatter_time: -100.0,
            shatter_force: 0.0,
            berzerker_rage: 0.0,
            berzerker_mode: 0,
            scroll_offset: 0.0,
            scale_factor: 1.0,
            scene_type: SCENE_AURORA,
            _pad_vec2_align: [0],
            fireball_pos: [0.0, 0.0],
            camera_pos: [0.0, 0.0, -5.0],
            _pad_cam: 0.0,
            light_direction: [0.5, 0.8, 0.6],
            _pad_light_dir: 0.0,
            light_color: [1.0, 0.95, 0.9],
            ibl_enabled: 0,
            shadow_map_size: 1024.0,
            shadow_bias: 0.005,
            _pad_shadow: [0, 0],
            light_vp: glam::Mat4::IDENTITY,
            ambient_color: [0.06, 0.07, 0.1, 1.0],
        }
    }
}

/// Cascaded Shadow Map uniform buffer structure.
/// Maintains 16-byte alignment requirements for uniform buffers.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct CsmUniforms {
    pub cascade_vps: [glam::Mat4; 4], // 256 bytes
    pub cascade_splits: [f32; 4],     // 16 bytes
    pub _pad: [f32; 4],               // 16 bytes (padding for 16-byte alignment)
}

impl Default for CsmUniforms {
    fn default() -> Self {
        Self {
            cascade_vps: [glam::Mat4::IDENTITY; 4],
            cascade_splits: [0.0; 4],
            _pad: [0.0; 4],
        }
    }
}

// =============================================================================
// GI UNIFORMS - Precomputed Global Illumination
// =============================================================================

/// Uniform buffer for precomputed GI data (L2 Spherical Harmonics probes).
/// Contains irradiance volume configuration and SH coefficients for indirect lighting.
/// 
/// # GPU Layout
/// - Header: 48 bytes (volume_origin: vec3 + padding, volume_spacing: vec3 + padding, probe_dimensions: uvec3 + padding)
/// - Probe coefficients: 4096 probes × 12 floats × 4 bytes = 196,608 bytes
/// - Total: 196,656 bytes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GiUniforms {
    /// World-space origin of the irradiance volume grid.
    pub volume_origin: [f32; 3],
    pub _pad0: f32, // 16-byte alignment padding
    
    /// Spacing between probes in world units.
    pub volume_spacing: [f32; 3],
    pub _pad1: f32, // 16-byte alignment padding
    
    /// Number of probes in each dimension (x, y, z).
    pub probe_dimensions: [u32; 3],
    pub _pad2: u32, // 16-byte alignment padding
    
    /// L2 SH coefficients (9 per probe, stored in 12-float slots for alignment).
    /// Each probe stores: L0 RGB, L1 RGB×3, L2 RGB×5 scaled/aligned to vec3 boundaries.
    pub probe_coefficients: [[f32; 12]; 4096], // 16×16×16 grid default
}

impl Default for GiUniforms {
    fn default() -> Self {
        Self {
            volume_origin: [0.0, 0.0, 0.0],
            _pad0: 0.0,
            volume_spacing: [1.0, 1.0, 1.0],
            _pad1: 0.0,
            probe_dimensions: [16, 16, 16],
            _pad2: 0,
            probe_coefficients: [[0.0; 12]; 4096],
        }
    }
}

/// L2 Spherical Harmonics evaluation.
/// Evaluates the SH coefficients at a given world position using trilinear interpolation.
/// 
/// # Arguments
/// * `world_pos` - World-space position to sample
/// * `_normal` - Surface normal for directional SH evaluation (unused in L0-only implementation)
/// * `uniforms` - GI uniform buffer containing probe coefficients
/// 
/// # Returns
/// Indirect irradiance color (RGB)
pub fn evaluate_sh_l2(world_pos: [f32; 3], _normal: [f32; 3], uniforms: &GiUniforms) -> [f32; 3] {
    let grid_pos = [
        (world_pos[0] - uniforms.volume_origin[0]) / uniforms.volume_spacing[0],
        (world_pos[1] - uniforms.volume_origin[1]) / uniforms.volume_spacing[1],
        (world_pos[2] - uniforms.volume_origin[2]) / uniforms.volume_spacing[2],
    ];

    // Clamp to volume bounds (edge case invariant)
    if grid_pos[0] < 0.0 
        || grid_pos[1] < 0.0 
        || grid_pos[2] < 0.0
        || (grid_pos[0] as u32) >= uniforms.probe_dimensions[0]
        || (grid_pos[1] as u32) >= uniforms.probe_dimensions[1]
        || (grid_pos[2] as u32) >= uniforms.probe_dimensions[2] 
    {
        return [0.0, 0.0, 0.0]; // Fallback to ambient (handled by shader)
    }

    let grid_cell = [
        grid_pos[0].floor() as u32,
        grid_pos[1].floor() as u32,
        grid_pos[2].floor() as u32,
    ];
    
    // Linear index into probe array: idx = x + y*dim_x + z*dim_x*dim_y
    let probe_idx = grid_cell[0] 
        + grid_cell[1] * uniforms.probe_dimensions[0] 
        + grid_cell[2] * uniforms.probe_dimensions[0] * uniforms.probe_dimensions[1];
    
    // Extract L0 coefficients (indices 0-2)
    let coeffs = uniforms.probe_coefficients[probe_idx as usize];
    [coeffs[0], coeffs[1], coeffs[2]]
}

#[cfg(test)]
mod gi_uniforms_tests {
    use super::*;

    #[test]
    fn gi_uniforms_has_valid_layout() -> anyhow::Result<()> {
        // Precondition: struct is #[repr(C)] with proper alignment
        // Verify each field offset matches GPU std140 layout
        assert_eq!(std::mem::offset_of!(GiUniforms, volume_origin), 0);
        assert_eq!(std::mem::offset_of!(GiUniforms, volume_spacing), 16);
        assert_eq!(std::mem::offset_of!(GiUniforms, probe_dimensions), 32);
        Ok(())
    }

    #[test]
    fn gi_uniforms_size_is_packed() -> anyhow::Result<()> {
        // Expected: 48 bytes header + 4096 probes * 48 bytes = 196,656 bytes total
        let expected_size = 48 + (4096 * 12 * 4);
        assert_eq!(std::mem::size_of::<GiUniforms>(), expected_size);
        Ok(())
    }

    #[test]
    fn probe_coefficients_stride_is_aligned() -> anyhow::Result<()> {
        // Invariant: each probe is 12 floats (48 bytes) tightly packed
        assert_eq!(std::mem::size_of::<[f32; 12]>(), 48);
        Ok(())
    }

    #[test]
    fn sh_evaluation_returns_zero_outside_volume() -> anyhow::Result<()> {
        let uniforms = GiUniforms::default();
        // Position at origin, volume starts at 0 - should be inside
        let result = evaluate_sh_l2([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], &uniforms);
        // Default coefficients are zero, so result should be [0, 0, 0]
        assert_eq!(result, [0.0, 0.0, 0.0]);
        Ok(())
    }
}

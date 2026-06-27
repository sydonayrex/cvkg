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
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
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
            glass_base: [0.02, 0.01, 0.01, 0.92],  // Darker flat blood-iron background
            glass_edge: [0.7, 0.15, 0.05, 0.55],   // Clean blood-red neon edge
            rune_glow: [0.95, 0.35, 0.1, 0.6],     // Warm fire-rune glow
            ember_core: [0.98, 0.25, 0.05, 0.8],
            background_deep: [0.005, 0.002, 0.002, 1.0], // Deeper flat black canvas
            mani_glow: [0.8, 0.15, 0.02, 0.03],    // Smaller cursor halo
            glass_blur_strength: 0.5,              // Reduced blur for clean contrast
            shatter_edge_width: 2.0,
            neon_bloom_radius: 0.018,              // Reduced bloom radius from 0.035 to 0.018 for legibility
            rune_opacity: 0.45,                    // Softened background rune glow
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
    pub _pad: [f32; 4], // Align to 224 bytes (struct align 16 from Mat4)
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
            _pad: [0.0; 4],
        }
    }
}

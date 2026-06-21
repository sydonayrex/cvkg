//! P1-1 (phase 6): SurtrConfig -- the renderer's tunable parameters.
//!
//! Extracted from types.rs to give the configuration its own
//! module. The struct and its impl blocks were previously defined
//! at the top of cvkg-render-gpu/src/renderer.rs.

use std::num::NonZeroUsize;

/// Configurable parameters for the GpuRenderer.
///
/// The 5220-line GpuRenderer monolith hardcoded six LRU cache
/// sizes plus the Mega-Heim atlas dimensions. This struct
/// extracts those tunables so they can be adjusted at runtime via
/// `GpuRenderer::set_config()` (e.g., to use a low_vram preset
/// after detecting a device with limited VRAM).
#[derive(Debug, Clone)]
pub struct SurtrConfig {
    /// Capacity of the text glyph cache (keyed by glyph hash).
    pub text_cache_capacity: NonZeroUsize,
    /// Capacity of the SVG tessellated model cache.
    pub svg_cache_capacity: NonZeroUsize,
    /// Capacity of the parsed usvg::Tree cache.
    pub svg_trees_capacity: NonZeroUsize,
    /// Capacity of the shared element (Lottie) cache.
    pub shared_elements_capacity: NonZeroUsize,
    /// Capacity of the image UV registry.
    pub image_uv_capacity: NonZeroUsize,
    /// Capacity of the texture name -> handle registry.
    pub texture_registry_capacity: NonZeroUsize,
    /// Width of the Mega-Heim shared texture atlas.
    pub mega_heim_width: u32,
    /// Height of the Mega-Heim shared texture atlas.
    pub mega_heim_height: u32,
}

impl Default for SurtrConfig {
    fn default() -> Self {
        // Defaults match the previously hardcoded values so
        // behavior is preserved.
        Self {
            text_cache_capacity: NonZeroUsize::new(8192).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(512).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(512).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(1024).unwrap(),
            image_uv_capacity: NonZeroUsize::new(256).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(31).unwrap(),
            mega_heim_width: 4096,
            mega_heim_height: 4096,
        }
    }
}

impl SurtrConfig {
    /// Low-VRAM preset: smaller caches and atlas for mobile GPUs.
    pub fn low_vram() -> Self {
        Self {
            text_cache_capacity: NonZeroUsize::new(2048).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(128).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(128).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(256).unwrap(),
            image_uv_capacity: NonZeroUsize::new(64).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(15).unwrap(),
            mega_heim_width: 2048,
            mega_heim_height: 2048,
        }
    }

    /// High-end preset: larger caches and atlas for desktop GPUs.
    pub fn high_end() -> Self {
        Self {
            text_cache_capacity: NonZeroUsize::new(16384).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(1024).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(1024).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(4096).unwrap(),
            image_uv_capacity: NonZeroUsize::new(1024).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(127).unwrap(),
            mega_heim_width: 8192,
            mega_heim_height: 8192,
        }
    }

    /// Total VRAM cost of the Mega-Heim texture in bytes
    /// (RGBA8 = 4 bytes/pixel).
    pub fn mega_heim_vram_bytes(&self) -> u64 {
        self.mega_heim_width as u64
            * self.mega_heim_height as u64
            * 4
    }
}

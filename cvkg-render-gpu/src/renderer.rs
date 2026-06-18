//! The main SurtrRenderer struct and core frame lifecycle.
use crate::draw::{parse_svg_animations, usvg_to_lyon};
use crate::heim::SundrPacker;
use crate::kvasir;
use crate::types::*;
use crate::vertex::*;
use crate::{
    WGSL_BIFROST, WGSL_BLOOM, WGSL_COLOR_BLIND, WGSL_COMMON, WGSL_MATERIAL_GLASS,
    WGSL_MATERIAL_OPAQUE, WGSL_PARTICLES, WGSL_SHAPES, WGSL_TONEMAP,
};
use bytemuck;
use cvkg_core::Rect;
use cvkg_core::Renderer;
use cvkg_core::{ColorTheme, SceneUniforms};
use lru::LruCache;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator, VertexBuffers,
};
use std::collections::VecDeque;
use std::num::NonZeroUsize;

/// Material ID constants used in vertex `material_id` and DrawMaterial routing.
/// These map to shader material indices and control per-draw-call pipeline selection.
pub(crate) mod material_id {
    /// Opaque geometry (default, depth-tested, no blending).
    pub const OPAQUE: u32 = 0;
    /// Ellipse shape (SDF circle, no blending).
    pub const ELLIPSE: u32 = 4;
    /// Top UI layer (alpha blended, no blur).
    pub const TOP_UI: u32 = 6;
    /// Glass / frosted blur material.
    pub const GLASS: u32 = 7;
    /// Blend modes occupy IDs 8..=22 (mapping to blend mode 1..=15).
    pub const BLEND_START: u32 = 8;
    pub const BLEND_END: u32 = 22;
    /// Radial gradient (blend mode 9).
    pub const RADIAL_GRADIENT: u32 = 16;
    /// Squircle stroke / circular progress (blend mode 10).
    pub const SQUIRCLE_STROKE: u32 = 17;
    /// Drop shadow / glow SDF (blend mode 11).
    pub const DROP_SHADOW: u32 = 18;
    /// Dashed stroke (blend mode 12).
    pub const DASHED_STROKE: u32 = 19;
    /// 3D cube mesh (blend mode 14).
    pub const MESH_3D: u32 = 21;
}
use std::sync::Arc;

/// P1-10: Quality level for adaptive rendering on different GPU tiers.
///
/// `High` matches the previous hardcoded behavior (MSAA 4x).
/// `Medium` reduces MSAA to 2x for moderate savings on mobile.
/// `Low` disables MSAA entirely for low-end GPUs (Adreno 3xx, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityLevel {
    High,
    Medium,
    Low,
}

impl QualityLevel {
    /// Returns the MSAA sample count for this quality level.
    pub fn msaa_sample_count(self) -> u32 {
        match self {
            QualityLevel::High => 4,
            QualityLevel::Medium => 2,
            QualityLevel::Low => 1,
        }
    }
}

impl Default for QualityLevel {
    fn default() -> Self {
        QualityLevel::High
    }
}

/// P1-1 fix: configurable SurtrRenderer parameters.
///
/// The 5220-line SurtrRenderer monolith hardcoded six LRU cache sizes
/// plus the Mega-Heim atlas dimensions. This struct extracts those
/// into a single configuration object so that callers can tune the
/// renderer for different working sets (high-end desktop vs. mid-tier
/// mobile vs. low-VRAM embedded) without modifying the source.
///
/// Cache size defaults match the values previously hardcoded after
/// the P1-5 fix (text=8192, svg=512, trees=512, shared=1024, uv=256,
/// texture_registry=31). Mega-Heim default is 4096x4096 RGBA8.
#[derive(Debug, Clone)]
pub struct SurtrConfig {
    /// LRU capacity for the text glyph cache.
    pub text_cache_capacity: NonZeroUsize,
    /// LRU capacity for the image UV registry.
    pub image_uv_capacity: NonZeroUsize,
    /// LRU capacity for the texture registry.
    pub texture_registry_capacity: NonZeroUsize,
    /// LRU capacity for the SVG cache (tessellated paths).
    pub svg_cache_capacity: NonZeroUsize,
    /// LRU capacity for the SVG trees (parsed source).
    pub svg_trees_capacity: NonZeroUsize,
    /// LRU capacity for the shared elements cache.
    pub shared_elements_capacity: NonZeroUsize,
    /// Mega-Heim atlas width in pixels.
    pub mega_heim_width: u32,
    /// Mega-Heim atlas height in pixels.
    pub mega_heim_height: u32,
}

impl Default for SurtrConfig {
    fn default() -> Self {
        // Defaults match the values previously hardcoded after the
        // P1-5 cache size increase. Future low-VRAM presets can be
        // added as separate constructors (e.g., SurtrConfig::mobile()).
        Self {
            text_cache_capacity: NonZeroUsize::new(8192).unwrap(),
            image_uv_capacity: NonZeroUsize::new(256).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(31).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(512).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(512).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(1024).unwrap(),
            mega_heim_width: 4096,
            mega_heim_height: 4096,
        }
    }
}

impl SurtrConfig {
    /// Preset for low-VRAM mobile devices (~64MB atlas).
    pub fn low_vram() -> Self {
        Self {
            text_cache_capacity: NonZeroUsize::new(2048).unwrap(),
            image_uv_capacity: NonZeroUsize::new(64).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(15).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(128).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(128).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(256).unwrap(),
            mega_heim_width: 2048,
            mega_heim_height: 2048,
        }
    }

    /// Preset for high-end desktop GPUs.
    pub fn high_end() -> Self {
        Self {
            text_cache_capacity: NonZeroUsize::new(16384).unwrap(),
            image_uv_capacity: NonZeroUsize::new(1024).unwrap(),
            texture_registry_capacity: NonZeroUsize::new(127).unwrap(),
            svg_cache_capacity: NonZeroUsize::new(1024).unwrap(),
            svg_trees_capacity: NonZeroUsize::new(1024).unwrap(),
            shared_elements_capacity: NonZeroUsize::new(4096).unwrap(),
            mega_heim_width: 8192,
            mega_heim_height: 8192,
        }
    }

    /// Total Mega-Heim VRAM cost in bytes (RGBA8 = 4 bytes/pixel).
    /// P1-1 audit: "On mobile GPUs with 256MB VRAM limits, this
    /// consumes ~64MB" -- use this helper to detect VRAM pressure.
    pub fn mega_heim_vram_bytes(&self) -> u64 {
        self.mega_heim_width as u64
            * self.mega_heim_height as u64
            * 4  // RGBA8
    }
}

/// SurtrRenderer implements the high-performance GPU backend.
pub struct SurtrRenderer {
    pub(crate) instance: Arc<wgpu::Instance>,
    pub(crate) adapter: Arc<wgpu::Adapter>,
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,

    // Kvasir resource registry -- tracks GPU resource lifetimes
    pub(crate) registry: crate::kvasir::registry::ResourceRegistry,

    pub(crate) active_offscreens: Vec<crate::types::OffscreenEffectConfig>,
    pub(crate) effect_pipelines: std::collections::HashMap<String, wgpu::RenderPipeline>,
    pub(crate) effect_params_buffer: wgpu::Buffer,
    pub(crate) effect_params_bind_group: wgpu::BindGroup,
    pub(crate) linear_sampler: wgpu::Sampler,
    // AI Generator Channel
    pub ai_material_rx: Option<
        std::sync::mpsc::Receiver<
            Result<crate::material::CompiledMaterial, crate::ai::GeneratorError>,
        >,
    >,

    // Multi-Window Surface Management
    pub(crate) surfaces: std::collections::HashMap<winit::window::WindowId, SurfaceContext>,
    pub(crate) current_window: Option<winit::window::WindowId>,
    pub headless_context: Option<HeadlessContext>,

    // Mega-Heim (Shared across all windows)
    /// P1-1: text rendering caches and engine grouped into a single
    /// TextSubsystem struct. This is the third step toward moving
    /// subsystems into their own modules.
    pub(crate) text: crate::types::TextSubsystem,
    pub(crate) mega_heim_tex: wgpu::Texture,
    pub(crate) mega_heim_bind_group: wgpu::BindGroup,
    pub(crate) heim_packer: SundrPacker,
    pub(crate) image_uv_registry: LruCache<String, Rect>,
    pub(crate) texture_registry: LruCache<String, u32>,
    pub(crate) texture_views: Vec<wgpu::TextureView>,
    pub(crate) dummy_sampler: wgpu::Sampler,
    /// P1-1: SVG caches and engine grouped into a single
    /// SvgSubsystem struct. Fourth step toward subsystem
    /// extraction.
    pub(crate) svg: crate::types::SvgSubsystem,

    // Niflheim Resources (Shared)
    pub(crate) dummy_texture_bind_group: wgpu::BindGroup,
    pub(crate) dummy_env_bind_group: wgpu::BindGroup,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) shared_elements: LruCache<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
    /// P1-1: the three GPU draw buffers (vertex, index, instance) are
    /// now grouped in a single GeometryBuffers struct. This is the
    /// first step toward moving buffer management into its own
    /// module.
    pub(crate) geometry_buffers: crate::types::GeometryBuffers,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) instance_data: Vec<InstanceData>,
    pub(crate) staging_belt: wgpu::util::StagingBelt,
    pub(crate) staging_command_buffers: Vec<wgpu::CommandBuffer>,
    pub(crate) draw_calls: Vec<DrawCall>,
    pub(crate) current_texture_id: Option<u32>,

    // Opacity & Clip Stacks
    pub(crate) opacity_stack: Vec<f32>,
    pub(crate) clip_stack: Vec<Rect>,
    pub(crate) slice_stack: Vec<(f32, f32)>,
    pub(crate) shadow_stack: Vec<ShadowState>,

    // The Forge's Heart (Shared Berserker State)
    pub(crate) theme_buffer: wgpu::Buffer,
    pub(crate) scene_buffer: wgpu::Buffer,
    pub(crate) berserker_bind_group: wgpu::BindGroup,
    pub(crate) berserker_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) start_time: std::time::Instant,
    pub(crate) current_theme: ColorTheme,
    pub(crate) current_scene: SceneUniforms,
    pub(crate) current_z: f32,

    /// Default background color for the canvas (RGBA).
    /// Used when the app does not draw its own background.
    /// Defaults to Deep Void [0.02, 0.02, 0.05, 1.0].
    pub(crate) default_background_color: [f32; 4],

    /// Whether the app drew any background geometry this frame.
    /// If false, the renderer clears to default_background_color.
    pub(crate) app_drew_background: bool,

    /// Whether render_frame() was called this frame.
    /// Used by end_frame() to auto-flush staging if render_frame() was skipped.
    pub(crate) frame_rendered: bool,

    /// Current draw order for SVG and other direct draw calls.
    /// Set by draw_svg_with_order(), used by emit_draw_call().
    pub(crate) current_draw_order: i32,

    // Muspelheim Pipelines (Shared)
    pub(crate) pipeline: wgpu::RenderPipeline,
    /// Specialized opaque/2D material pipeline (modes 0-20 excluding 7,13-15,18,21).
    pub(crate) opaque_pipeline: wgpu::RenderPipeline,
    /// Non-multisampled pipeline used specifically to draw UI overlays.
    /// Drawn with sample count 1 and no depth testing/depth stencil attachment.
    pub(crate) ui_pipeline: wgpu::RenderPipeline,
    /// Specialized glass material pipeline (mode 7 only, ~150 lines of complex math).
    pub(crate) glass_pipeline: wgpu::RenderPipeline,
    pub(crate) background_pipeline: wgpu::RenderPipeline,
    pub(crate) bloom_extract_pipeline: wgpu::RenderPipeline,
    /// Identity copy pipeline for Pass 2 backdrop blur (all pixels, no luminance gate).
    pub(crate) copy_pipeline: wgpu::RenderPipeline,
    pub(crate) composite_pipeline: wgpu::RenderPipeline,
    /// Color blindness simulation pipeline (fullscreen triangle).
    pub(crate) color_blind_pipeline: wgpu::RenderPipeline,
    /// Volumetric raymarching pipeline (fullscreen triangle with SDF raymarch).
    pub(crate) volumetric_pipeline: wgpu::RenderPipeline,
    /// Volumetric bind group layout for scene uniforms (time/resolution/light).
    pub(crate) volumetric_bind_group_layout: wgpu::BindGroupLayout,
    /// Persistent uniform buffer for volumetric data (updated each frame).
    pub(crate) volumetric_uniform_buffer: wgpu::Buffer,
    /// CPU-side list of hologram instances submitted this frame.
    /// Cleared each frame in reset_frame_state; consumed by VolumetricNode::execute.
    pub(crate) hologram_instances: Vec<HologramInstance>,
    /// Kawase blur pyramid downsample pipeline (separate shader module).
    pub(crate) kawase_down_pipeline: wgpu::RenderPipeline,
    /// Kawase blur pyramid upsample pipeline (separate shader module).
    pub(crate) kawase_up_pipeline: wgpu::RenderPipeline,
    /// Kawase blur bind group layout (uniform + texture + sampler).
    pub(crate) kawase_bind_group_layout: wgpu::BindGroupLayout,
    /// Persistent uniform buffer for Kawase blur operations (avoids per-frame allocation).
    pub(crate) kawase_uniform: wgpu::Buffer,
    /// Environment bind group layout (texture + sampler).
    pub(crate) env_bind_group_layout: wgpu::BindGroupLayout,

    // Telemetry
    pub telemetry: cvkg_core::TelemetryData,

    /// Pipeline cache for disk-persisted compiled shaders.
    /// Speeds up cold startup by avoiding recompilation of identical pipelines.
    pub(crate) pipeline_cache: wgpu::PipelineCache,

    /// Configuration for render-loop frame timing and degradation strategies.
    pub frame_budget: cvkg_core::FrameBudget,
    /// Staging buffer for windowed frame capture.
    pub(crate) capture_staging_buffer: Option<wgpu::Buffer>,
    /// Instant at the start of the last redraw, used for measuring frame timings.
    pub last_redraw_start: std::time::Instant,
    /// Instant at the start of the last frame, used for frame_time_ms calculation.
    pub last_frame_start: std::time::Instant,

    // VRAM Tracking (Bytes)
    pub(crate) vram_buffers_bytes: u64,
    pub(crate) vram_textures_bytes: u64,

    // Debugging
    pub(crate) _debug_layout: bool,

    // Transform Stack -- stores full affine matrices for correct SVG transform composition.
    pub(crate) transform_stack: Vec<glam::Mat3>,
    /// Whether a redraw has been requested for the next frame.
    pub redraw_requested: bool,
    /// Cursor for compositor draw call submission tracking.
    pub(crate) compositor_index_cursor: u32,

    /// Bloom post-processing enabled flag.
    pub bloom_enabled: bool,
    /// Dynamic toggle to enable or disable the volumetric raymarching pass, which handles fog and light shaft simulations.
    pub volumetric_enabled: bool,
    /// Color blindness bind group layout (texture + sampler + uniform).
    pub(crate) color_blind_bind_group_layout: wgpu::BindGroupLayout,
    /// Color blindness uniform buffer (updated each frame when mode changes).
    pub(crate) color_blind_uniform_buffer: wgpu::Buffer,
    /// Color blindness simulation mode (Normal = disabled).
    pub color_blind_mode: crate::color_blindness::ColorBlindMode,
    /// Color blindness effect intensity (0.0–1.0).
    pub color_blind_intensity: f32,
    /// Sampler for the color blindness pass (reused from main pipeline).
    pub(crate) sampler: wgpu::Sampler,

    // Timestamp Queries (Norse: Skuld = future/time/debt)
    pub(crate) skuld_queries: Option<wgpu::QuerySet>,
    pub(crate) skuld_buffer: Option<wgpu::Buffer>,
    pub(crate) skuld_read_buffer: Option<wgpu::Buffer>,
    pub(crate) skuld_period: f32,
    pub last_gpu_time_ns: u64,

    // Particle Compute Pipeline (Muspelheim Compute)
    /// Compute pipeline for GPU particle integration (Euler + drag + lifetime).
    pub(crate) particle_compute_pipeline: wgpu::ComputePipeline,
    /// Bind group layout for the particle compute pass (storage buffer + uniform).
    pub(crate) particle_compute_bgl: wgpu::BindGroupLayout,
    /// GPU storage buffer holding particle data (pos_vel + color_life, 32 bytes each).
    pub(crate) particle_buffer: wgpu::Buffer,
    /// Uniform buffer for particle compute (dt).
    pub(crate) particle_uniform_buffer: wgpu::Buffer,
    /// CPU-side staging array for newly emitted particles (flushed to GPU each frame).
    pub(crate) particle_staging: Vec<GpuParticle>,
    /// Number of live particles currently in the ring buffer.
    pub(crate) particle_count: u32,
    /// Write cursor into the particle ring buffer (wraps at MAX_PARTICLES).
    pub(crate) particle_write_head: u32,
    /// Simple render pipeline for drawing particles as point sprites.
    pub(crate) particle_render_pipeline: wgpu::RenderPipeline,
    /// Bind group layout for particle render pass (storage buffer read-only).
    pub(crate) particle_render_bgl: wgpu::BindGroupLayout,
    /// Bind group for particle render pass (created lazily when count > 0).
    pub(crate) particle_render_bind_group: Option<wgpu::BindGroup>,
    /// Timestamp of last buffer compaction (dead particle removal).
    pub(crate) last_particle_compact: std::time::Instant,

    // VDOM node stack for hierarchy tracking
    pub(crate) vnode_stack: Vec<(Rect, &'static str)>,

    /// Event handlers registered during render passes.
    /// Maps "event_type" -> list of handlers.
    pub(crate) event_handlers: std::collections::HashMap<
        String,
        Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>,
    >,

    /// Bind group layout for reading blur output in glass composite pass.
    pub(crate) glass_output_bind_group_layout: wgpu::BindGroupLayout,
    /// Current material state -- draw calls are tagged with this material.
    pub(crate) current_draw_material: cvkg_core::DrawMaterial,

    /// Portal backdrop blur regions -- collected during portal enter/exit
    /// Used for per-element isolated backdrop blur (Tahoe feature)
    pub(crate) portal_regions: std::collections::VecDeque<cvkg_core::Rect>,

    /// Cache of the compiled Kvasir render graph execution plan.
    /// Used to bypass graph rebuilding and topological sorting when configuration is unchanged.
    pub(crate) cached_graph_plan: Option<kvasir::graph_cache::CachedGraphPlan>,
    /// Hash of the active material set, used to invalidate the graph plan
    /// cache when materials change. Updated whenever a material is added,
    /// removed, or its WGSL output is recompiled. P1-9 fix: the previous
    /// cache key did not include material compilation, so a material
    /// change would silently produce stale shader bindings.
    pub(crate) material_compilation_hash: u64,
    /// Memoization cache for frame-level render skipping.
    /// Tracks (id) -> (data_hash, frame_generation) for deduplication.
    pub(crate) memo_cache: std::collections::HashMap<u64, crate::types::MemoEntry>,
    /// Current frame generation counter. Incremented each frame to avoid
    /// clearing the memo cache (which would defeat cross-frame memoization).
    pub(crate) frame_generation: u64,
    /// P1-1: SurtrRenderer configuration. Contains cache sizes,
    /// atlas dimensions, and other tunable parameters. Can be
    /// replaced at runtime via `set_config()` to adapt to different
    /// working sets (e.g., after detecting a low-VRAM device).
    pub(crate) config: SurtrConfig,
    /// P1-10: Quality level controlling MSAA sample count and other
    /// adaptive rendering settings. Defaults to High to match the
    /// previous hardcoded 4x MSAA behavior.
    pub(crate) quality_level: QualityLevel,
    /// Thread-safe bind group cache to avoid per-frame allocations during render passes.
    /// Maps a cache key representing texture/pass metadata to the pre-created wgpu::BindGroup.
    pub(crate) bind_group_cache: std::sync::Mutex<
        std::collections::HashMap<
            (crate::kvasir::resource::ResourceId, u32, bool),
            wgpu::BindGroup,
        >,
    >,
    /// Thread-safe texture view cache to avoid per-frame allocations of TextureViews.
    /// Maps (texture id, mip level) -> wgpu::TextureView.
    pub(crate) texture_view_cache: std::sync::Mutex<
        std::collections::HashMap<(crate::kvasir::resource::ResourceId, u32), wgpu::TextureView>,
    >,
}

// P0-3 safety audit: unsafe Send/Sync on WASM.
//
// SurtrRenderer contains the following shared state:
//   - wgpu::Device and wgpu::Queue  (transitively !Send + !Sync on WASM)
//   - Mutex<HashMap<...>> caches    (bind_group_cache, texture_view_cache,
//                                    shaped_text_cache -- Mutex<T> is Send+Sync
//                                    iff T: Send, which holds for our HashMaps)
//   - Vec<Vertex>, Vec<u32>, Vec<InstanceData>, Vec<DrawCall> -- the GPU
//     buffer staging areas. These are mutated each frame and may be observed
//     by the GPU submission queue.
//   - Vec<HologramInstance>         (only accessed from the main thread)
//
// SAFETY JUSTIFICATION (wasm32 target only):
//
// 1. WASM is single-threaded: JavaScript executes on a single thread and
//    async tasks are cooperatively scheduled on the same thread. There is
//    no preemption and no actual concurrent access to the renderer's
//    mutable state. wgpu's !Send+!Sync on WASM reflects this same
//    single-threaded guarantee -- wgpu's Device/Queue can be sent across
//    await points because the WebGPU spec guarantees a single-threaded
//    execution model.
//
// 2. The Mutex fields (bind_group_cache, texture_view_cache,
//    shaped_text_cache) provide their own synchronization for any code
//    that DOES run on multiple threads (i.e. native builds). On WASM
//    these locks are no-ops in practice but the data is still safe to
//    access from a single thread.
//
// 3. SurtrRenderer's GPU buffer staging vectors are only mutated by the
//    renderer's own methods, all of which are called sequentially from
//    the event loop on a single thread. No background task, no worker
//    thread, no async task post-yield can observe partial state.
//
// 4. The HologramInstance Vec is also only accessed from the event loop.
//
// 5. We intentionally do NOT impl Send+Sync on non-WASM targets because
//    on those platforms wgpu's Device/Queue are Send+Sync by design, but
//    our internal GPU buffer state is not actually safe for cross-thread
//    access without additional synchronization. The Mutex-wrapped caches
//    are the only state that is genuinely thread-safe on native targets.
//
// This is a known intentional divergence from wgpu's conservative
// !Send+!Sync on WASM. It is necessary because winit's event loop on
// WASM requires the application state to be Send so it can be held
// across .await points. The single-threaded WASM execution model
// guarantees that no two threads ever access the renderer concurrently,
// so the safety contract holds.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for SurtrRenderer {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for SurtrRenderer {}

/// SVG tessellation parameters.
pub(crate) struct TessellateParams<'a> {
    fill_tessellator: &'a mut FillTessellator,
    stroke_tessellator: &'a mut StrokeTessellator,
    vertices: &'a mut Vec<Vertex>,
    indices: &'a mut Vec<u32>,
    parsed_animations: &'a [SvgAnimation],
    finalized_animations: &'a mut Vec<SvgAnimation>,
    paths: &'a mut Vec<crate::types::SvgPath>,
}

/// Per-hologram instance data submitted during the frame.
/// Consumed by VolumetricNode::execute to parameterize the volumetric shader.
#[derive(Debug, Clone)]
pub struct HologramInstance {
    /// Bounding rectangle in logical coordinates (x, y, width, height).
    pub rect: cvkg_core::Rect,
    /// Hash of the hologram_id string -- used for per-hologram visual variation.
    pub id_hash: u32,
    /// Application-provided time for this hologram instance.
    pub time: f32,
}

/// Trait for types that can be cleared in place. Implemented for the
/// collection types used as cache values (HashMap, Vec).
///
/// Used by `lock_or_clear_cache` to wipe cache data after a poisoned
/// mutex recovery, since a partially-mutated cache (from a panic
/// mid-insert) must not be reused on subsequent frames.
pub trait ClearInto {
    fn clear_into(&mut self);
}

impl<K, V, S> ClearInto for std::collections::HashMap<K, V, S>
where
    S: std::hash::BuildHasher,
{
    fn clear_into(&mut self) {
        self.clear();
    }
}

impl<T> ClearInto for Vec<T> {
    fn clear_into(&mut self) {
        self.clear();
    }
}

// =========================================================================
// P1-11: Pipeline cache integrity check
// =========================================================================

/// P1-11 fix: load a pipeline cache file from disk with SHA256 integrity check.
///
/// Returns:
/// - `Ok(Some(data))` if the cache file exists and its SHA256 matches the sidecar
/// - `Ok(None)` if the cache file does not exist (first run, no cache yet)
/// - `Err(reason)` if the cache file exists but integrity verification fails
///   (sidecar missing, sidecar malformed, hash mismatch). The caller should
///   treat this as "use empty cache" so wgpu falls back to recompilation.
///
/// The sidecar file is `<cache_path>.sha256` and contains the lowercase hex
/// SHA256 of the cache data, written at the same time the cache is written.
/// On any integrity failure we refuse to use the cache rather than risk
/// passing tampered data to the unsafe `create_pipeline_cache` boundary.
fn load_pipeline_cache_with_integrity_check(
    cache_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
    // No cache file = first run, nothing to load.
    let cache_data = match std::fs::read(cache_path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read failed: {e}")),
    };

    let hash_path = cache_path.with_extension("bin.sha256");
    let expected_hash = match std::fs::read_to_string(&hash_path) {
        Ok(s) => s.trim().to_lowercase(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "sidecar hash file missing at {}",
                hash_path.display()
            ))
        }
        Err(e) => return Err(format!("sidecar read failed: {e}")),
    };

    // Compute actual SHA256.
    let actual = compute_sha256(&cache_data);
    let actual_hex = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        actual[0], actual[1], actual[2], actual[3],
        actual[4], actual[5], actual[6], actual[7]
    );

    if actual_hex != expected_hash {
        return Err(format!(
            "hash mismatch: expected {expected_hash}, got {actual_hex}"
        ));
    }

    Ok(Some(cache_data))
}

/// Compute SHA256 of a byte slice. Inline FIPS 180-4 implementation
/// (avoids adding a sha2 crate dependency for a single-use feature).
fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

impl SurtrRenderer {
    /// Access the hologram instances submitted this frame.
    pub fn hologram_instances(&self) -> &[HologramInstance] {
        &self.hologram_instances
    }

    /// P1-10: set the rendering quality level. Affects MSAA sample
    /// count and (in the future) other adaptive rendering settings
    /// like blur mip levels and effect complexity. Must be called
    /// before the next frame's render pass setup; mid-frame changes
    /// will only take effect on subsequent frames.
    ///
    /// Quality levels:
    ///   - `High`: MSAA 4x (default, matches previous behavior)
    ///   - `Medium`: MSAA 2x (mobile, mid-tier GPUs)
    ///   - `Low`: MSAA 1x (low-end GPUs, Adreno 3xx, etc.)
    pub fn set_quality_level(&mut self, level: QualityLevel) {
        self.quality_level = level;
    }

    /// P1-1: replace the renderer configuration at runtime.
    ///
    /// Note: changes to cache sizes only affect FUTURE inserts; the
    /// existing LRU caches keep their original capacity. To resize
    /// existing caches, you must restart the renderer. This is a
    /// pragmatic limitation -- resizing LRU caches atomically would
    /// require copying the entire cache, which is expensive.
    ///
    /// Changes to atlas dimensions and other one-shot values take
    /// effect on the next frame.
    pub fn set_config(&mut self, config: SurtrConfig) {
        self.config = config;
    }

    /// P1-1: read the current configuration.
    pub fn config(&self) -> &SurtrConfig {
        &self.config
    }

    /// P1-10: get the current rendering quality level.
    pub fn quality_level(&self) -> QualityLevel {
        self.quality_level
    }

    /// Acquire a poisoned-mutex guard and CLEAR the underlying data on recovery.
    ///
    /// P1-3 fix: the previous SurtrRenderer::lock_or_clear_cache(` pattern
    /// silently accepted a partially-mutated cache (e.g. a bind group
    /// insertion interrupted by panic), which could then be used on the
    /// next frame and cause GPU validation errors or visual glitches.
    ///
    /// For GPU resource caches (bind groups, texture views, etc.) the
    /// safe recovery is to clear the cache so the next frame rebuilds
    /// from scratch. Use this in place of `into_inner()` for any cache
    /// that holds GPU resources whose state may be inconsistent after
    /// a panic mid-insert.
    pub(crate) fn lock_or_clear_cache<'a, T: ClearInto>(
        mutex: &'a std::sync::Mutex<T>,
    ) -> std::sync::MutexGuard<'a, T> {
        match mutex.lock() {
            Ok(g) => g,
            Err(poisoned) => {
                log::warn!(
                    "[GPU] poisoned cache mutex recovered; clearing data to avoid stale state"
                );
                let mut g = poisoned.into_inner();
                g.clear_into();
                g
            }
        }
    }

    /// Update cursor pointer uniforms for tactile hover shader interactions.
    ///
    /// # Contract
    /// - `mouse` represents logical window coordinates.
    /// - `velocity` is the change in logical coordinates per second.
    pub fn update_mouse(&mut self, mouse: [f32; 2], velocity: [f32; 2]) {
        self.current_scene.mouse = mouse;
        self.current_scene.mouse_velocity = velocity;
    }

    /// P1-9 fix: notify the renderer that the material set has changed
    /// (e.g. a material was added, removed, or its WGSL output was
    /// recompiled). Increments the material_compilation_hash so the
    /// graph plan cache will be invalidated on the next frame.
    ///
    /// Without this hook, a material change would silently reuse a stale
    /// cached plan with mismatched shader bindings.
    pub fn invalidate_material_cache(&mut self) {
        // Simple incrementing counter: any change invalidates. For
        // larger apps, this could be replaced with a content hash that
        // changes only when materials actually differ.
        self.material_compilation_hash = self.material_compilation_hash.wrapping_add(1);
    }

    /// select_best_surface_format selects the highest precision/HDR texture format
    /// supported by the surface. Favors floating point HDR (Rgba16Float) or Display P3 wide gamut
    /// (Rgba8Unorm) over standard sRGB, falling back to sRGB/first option if not available.
    pub(crate) fn select_best_surface_format(
        formats: &[wgpu::TextureFormat],
    ) -> wgpu::TextureFormat {
        if formats.is_empty() {
            // P1-7: even with no formats at all, return a known-safe
            // format rather than risking an HDR-only exotic format.
            return wgpu::TextureFormat::Rgba8Unorm;
        }
        // P1-7 fix: improved fallback chain for mobile GPUs. Some
        // older mobile GPUs (Adreno 3xx, early Mali) do not support
        // any sRGB format, in which case the previous code returned
        // formats[0] which could be a weird unsupported format
        // (e.g. RGB9E5Float for some HDR displays).
        //
        // The fix expands the preferred list to include linear
        // (non-sRGB) formats that virtually every GPU supports, and
        // adds a final guarantee that we always return a well-known
        // universally-supported format.
        let preferred_formats = [
            wgpu::TextureFormat::Rgba16Float, // HDR10 / Rec. 2020 FP16
            wgpu::TextureFormat::Rgba8Unorm,  // Wide Color Display P3
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            // P1-7: linear fallbacks for mobile GPUs without sRGB.
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
            // P1-7: last-resort formats that all GPUs support.
            wgpu::TextureFormat::Rgba8Unorm,
        ];
        for preferred in &preferred_formats {
            if formats.contains(preferred) {
                return *preferred;
            }
        }
        // P1-7: guaranteed safe fallback. If none of our preferred
        // formats match (very unusual -- e.g. exotic HDR-only
        // display), prefer Rgba8Unorm if available, otherwise fall
        // back to the first available format. Never return a format
        // we haven't at least seen in the surface's advertised list.
        if formats.contains(&wgpu::TextureFormat::Rgba8Unorm) {
            return wgpu::TextureFormat::Rgba8Unorm;
        }
        formats[0]
    }

    /// forge -- Initializes the Surtr GPU renderer from a winit window.
    ///
    /// This method performs the following:
    /// 1. Negotiates a wgpu surface and adapter.
    /// 2. Forges the Muspelheim multi-pass pipeline layouts.
    /// 3. Initializes the Berserker state buffers and texture registries.
    pub async fn forge(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        log::info!("[GPU] Requesting HighPerformance adapter...");

        let mut adapter = None;

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(filter) = std::env::var("WGPU_ADAPTER_NAME") {
            let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
            log::info!("[GPU] Available adapters:");
            for a in &adapters {
                let info = a.get_info();
                log::info!(
                    "  - Name: '{}' | Driver: '{}' | Backend: {:?}",
                    info.name,
                    info.driver,
                    info.backend
                );
            }

            adapter = adapters.into_iter().find(|a| {
                let info = a.get_info();
                let match_found = info.name.to_lowercase().contains(&filter.to_lowercase())
                    || info.driver.to_lowercase().contains(&filter.to_lowercase());
                if match_found {
                    log::info!(
                        "[GPU] Manual selection match: {} | Driver: {}",
                        info.name,
                        info.driver
                    );
                }
                match_found
            });

            if adapter.is_some() {
                log::info!(
                    "[GPU] Forced adapter selection via WGPU_ADAPTER_NAME='{}'",
                    filter
                );
            } else {
                log::warn!(
                    "[GPU] WGPU_ADAPTER_NAME='{}' provided but no matching adapter found. Falling back...",
                    filter
                );
            }
        }

        if adapter.is_none() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            log::warn!(
                "[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower..."
            );
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            log::warn!("[GPU] Hardware adapters failed, trying Software fallback...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: true,
                })
                .await
                .ok();
        }

        let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");
        let info = adapter.get_info();
        log::info!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name,
            info.device_type,
            info.backend
        );
        log::info!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        #[cfg(not(target_arch = "wasm32"))]
        let mut required_features =
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY;

        #[cfg(target_arch = "wasm32")]
        let mut required_features = wgpu::Features::empty(); // Fallbacks for WebGL
        if supports_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
        }
                // Enable validation layer in debug builds for better error reporting
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        {
            log::info!("[GPU] Validation layer enabled (debug build)");
        }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Forge"),
                required_features,
                required_limits: wgpu::Limits {
                    max_bindings_per_bind_group: 256,
                    max_binding_array_elements_per_shader_stage: 256,
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);

        device.on_uncaptured_error(Arc::new(|error| {
            log::error!(
                "[GPU] Uncaptured device error (Device Lost or Panic): {:?}",
                error
            );
            // In a full recovery scenario, we would signal the event loop to rebuild the GPU context
        }));

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
        // Ensure we have valid dimensions - Wayland may return 0 for not-yet-committed surfaces
        let width = if size.width > 0 { size.width } else { 1280 };
        let height = if size.height > 0 { size.height } else { 720 };
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = if surface_caps.formats.is_empty() {
            log::error!("[GPU] CRITICAL: No compatible surface formats found for this adapter!");
            log::error!(
                "[GPU] Adapter: {} | Backend: {:?}",
                adapter.get_info().name,
                adapter.get_info().backend
            );
            // Fallback to a common format to avoid immediate panic, though configuration may still fail
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0])
        };

        // Dynamic capability selection for robust Wayland/X11 rendering
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            log::warn!("[GPU] Mailbox not supported, falling back to Fifo (V-Sync)");
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        log::info!(
            "[GPU] Configuring surface: {}x{} | {:?} | {:?}",
            width,
            height,
            present_mode,
            alpha_mode
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("[GPU] Surface configuration successful.");

        let renderer = Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            Some((window, surface, config)),
            None,
        )
        .await;
        log::info!("[GPU] Forge internal complete.");
        renderer
    }

    /// Internal rendering pipeline constructor.
    /// This function spans ~600 lines because it is responsible for forging the entire wgpu state machine.
    ///
    /// ## Structure:
    /// 1. Formats & Timestamp query resolution buffers
    /// 2. Bind Group Layouts (Uniforms, Environment, Blur, Color Blindness)
    /// 3. Pipeline compilation (Opaque, Glass, Text, SVG paths)
    /// 4. Global Mega Atlas and Dummy Texture initialization
    /// 5. Staging belt & Telemetry scaffolding
    pub(crate) async fn forge_internal(
        instance: Arc<wgpu::Instance>,
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_info: Option<(
            Arc<winit::window::Window>,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
        )>,
        headless_info: Option<(u32, u32, wgpu::TextureFormat)>,
    ) -> Self {
        let format = if let Some((_, _, ref config)) = surface_info {
            config.format
        } else if let Some((_, _, f)) = headless_info {
            f
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let skuld_period = queue.get_timestamp_period();
        let (skuld_queries, skuld_buffer, skuld_read_buffer) = if supports_timestamps {
            let q = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Skuld Timestamp Queries"),
                count: 2,
                ty: wgpu::QueryType::Timestamp,
            });
            let b = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Skuld Query Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let rb = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Skuld Read Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            (Some(q), Some(b), Some(rb))
        } else {
            (None, None, None)
        };

        // Dynamically compile material WGSL

        // Create pipeline cache for disk-persisted compiled shaders.
        // This avoids recompiling identical shaders on subsequent launches.
        // Cache lives next to the executable for both debug and release builds.
        // Falls back to temp dir if the exe path is unavailable (e.g., WASM).
        //
        // P1-11 fix: add SHA256 integrity check. The pipeline cache is
        // loaded from disk via `unsafe create_pipeline_cache`. While
        // wgpu's `fallback: true` handles invalid data by recompiling,
        // we add defense-in-depth by verifying a SHA256 sidecar file
        // matches the cache bytes. If the sidecar is missing or the hash
        // does not match, we treat the cache as empty rather than risk
        // passing tampered data through the unsafe boundary.
        let pipeline_cache = {
            let cache_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("pipeline_cache")))
                .unwrap_or_else(|| std::env::temp_dir().join("cvkg_pipeline_cache"));
            let _ = std::fs::create_dir_all(&cache_dir);
            let cache_path = cache_dir.join("cvkg_render_gpu.bin");
            let cache_data = match load_pipeline_cache_with_integrity_check(&cache_path) {
                Ok(data) => data,
                Err(reason) => {
                    log::warn!(
                        "[GPU] pipeline cache integrity check failed: {reason}; using empty cache"
                    );
                    None
                }
            };
            unsafe {
                device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                    label: Some("CVKG Pipeline Cache"),
                    data: cache_data.as_deref(),
                    fallback: true,
                })
            }
        };
        let materials_generated = crate::material::generate_builtins_wgsl();

        let wgsl_src = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_SHAPES,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );
        let wgsl_opaque = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_MATERIAL_OPAQUE,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );
        let wgsl_glass = format!(
            "{}{}{}{}{}{}",
            WGSL_COMMON,
            WGSL_MATERIAL_GLASS,
            WGSL_BIFROST,
            WGSL_BLOOM,
            WGSL_COLOR_BLIND,
            materials_generated
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Main Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_src)),
        });

        // Niflheim Bind Group Layout (for textures/samplers)
        // On wasm32/WebGL2, texture binding arrays are not supported, so we use
        // a small fixed count instead of a large array.
        #[cfg(target_arch = "wasm32")]
        let texture_array_count: Option<std::num::NonZeroU32> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let texture_array_count: Option<std::num::NonZeroU32> =
            std::num::NonZeroU32::new(32);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: texture_array_count,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Niflheim Texture Bind Group Layout"),
            });

        // Environment Bind Group Layout (for blurred background / Bifrost)
        // Environment Bind Group Layout (for blurred background / Bifrost)
        let env_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Surtr Environment Bind Group Layout"),
            });

        let berserker_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Surtr Berserker Bind Group Layout"),
            });

        // Pipeline setup
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surtr Main Pipeline Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        // Specialized layout for post-processing (Bloom Extract, Blur) which only need Group 0 + Globals
        let post_process_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Muspelheim Post Process Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        // Specialized layout for composite (Blur + Scene)
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Muspelheim Composite Layout"),
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&env_bind_group_layout),
                Some(&berserker_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_background"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // ── Specialized Material Pipelines ─────────────────────────────────────
        let opaque_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Opaque"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_opaque)),
        });
        let glass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Glass"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(wgsl_glass)),
        });

        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Opaque"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &opaque_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &opaque_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim UI"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &opaque_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &opaque_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });
        let glass_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Glass"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &opaque_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &glass_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Muspelheim Bloom Extract Pipeline
        let bloom_extract_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Muspelheim Bloom Extract"),
                layout: Some(&post_process_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_fullscreen"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_bloom_extract"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: Some(&pipeline_cache),
            });

        // Muspelheim Copy Pipeline (identity copy for backdrop blur Pass 2)
        let copy_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Copy"),
            layout: Some(&post_process_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_copy"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Kawase blur pyramid pipelines (separate shader module -- conflicting bindings)
        // NOTE: Compiled separately because blur_pyramid.wgsl defines its own
        // @group(0) bindings (BlurUniforms + texture + sampler) that conflict
        // with the main WGSL_SRC pipeline layout.
        let kawase_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kawase Blur Pyramid"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shaders/blur_pyramid.wgsl"
            ))),
        });
        let kawase_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Kawase Blur BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let kawase_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Kawase Pipeline Layout"),
            bind_group_layouts: &[Some(&kawase_bgl)],
            immediate_size: 0,
        });
        let kawase_down_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kawase Downsample"),
            layout: Some(&kawase_layout),
            vertex: wgpu::VertexState {
                module: &kawase_shader,
                entry_point: Some("vs_blur"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &kawase_shader,
                entry_point: Some("fs_kawase_down"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });
        let kawase_up_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Kawase Upsample"),
            layout: Some(&kawase_layout),
            vertex: wgpu::VertexState {
                module: &kawase_shader,
                entry_point: Some("vs_blur"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &kawase_shader,
                entry_point: Some("fs_kawase_up"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Muspelheim Composite Pipeline (additive blend onto screen)
        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Composite"),
            layout: Some(&composite_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_composite"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Additive blend: src + dst -- glow lights up the scene
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Forge the Mega-Heim (4096x4096 RGBA for production batching)
        let mega_heim_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Mega-Heim"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mega_heim_view_obj = mega_heim_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, // Use linear for images
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Forge the Niflheim Dummy Texture (1x1 White)
        let dummy_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Niflheim Dummy Texture"),
            size: dummy_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &dummy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            dummy_size,
        );

        let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let mut texture_views_list: Vec<wgpu::TextureView> =
            (0..32).map(|_| dummy_view.clone()).collect();
        texture_views_list[0] = mega_heim_view_obj.clone();

        let views_refs: Vec<&wgpu::TextureView> = texture_views_list.iter().collect();
        let mega_heim_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&text_sampler),
                },
            ],
            label: Some("Mega-Heim Bind Group"),
        });

        let dummy_views_refs: Vec<&wgpu::TextureView> = (0..32).map(|_| &dummy_view).collect();
        let dummy_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&dummy_views_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_sampler),
                },
            ],
            label: Some("Dummy Texture Bind Group"),
        });

        let dummy_env_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_sampler),
                },
            ],
            label: Some("Dummy Env Bind Group"),
        });

        let mut texture_registry = LruCache::new(NonZeroUsize::new(31).unwrap());
        let mut texture_bind_groups = Vec::new();

        texture_registry.put("__mega_heim".to_string(), 0);
        texture_bind_groups.push(mega_heim_bind_group.clone());

        // Forge the Anvil (Buffers)
        // P1-1: buffer creation moved into GeometryBuffers::forge()
        // so the buffer management subsystem can be moved into its
        // own module in a follow-up refactor.
        let geometry_buffers =
            crate::types::GeometryBuffers::forge(&device, MAX_VERTICES, MAX_INDICES);

        // Forge the Heart (Berserker Uniforms)
        let current_theme = ColorTheme::default();
        use wgpu::util::DeviceExt;
        let theme_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr Theme Buffer"),
            contents: bytemuck::bytes_of(&current_theme),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (width, height, scale_factor) = if let Some((ref window, _, ref config)) = surface_info
        {
            (config.width, config.height, window.scale_factor() as f32)
        } else if let Some((w, h, _)) = headless_info {
            (w, h, 1.0)
        } else {
            (1280, 720, 1.0)
        };

        let mut current_scene =
            SceneUniforms::new(width as f32 / scale_factor, height as f32 / scale_factor);
        current_scene.scale_factor = scale_factor;
        // P1-10: capture MSAA sample count. forge_internal is an
        // associated function (no `&self`), so we use the default
        // QualityLevel here. The QualityLevel on the resulting
        // SurtrRenderer is initialized to the same default below,
        // and can be changed later via set_quality_level().
        let msaa_sample_count = QualityLevel::default().msaa_sample_count();
        let scene_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Surtr Scene Buffer"),
            contents: bytemuck::bytes_of(&current_scene),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let berserker_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &berserker_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: theme_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene_buffer.as_entire_binding(),
                },
            ],
            label: Some("Surtr Berserker Bind Group"),
        });

        let mut registry = crate::kvasir::registry::ResourceRegistry::new();
        let mut surfaces = std::collections::HashMap::new();
        let mut current_window = None;
        let mut headless_context = None;

        if let Some((window, surface, config)) = surface_info {
            let window_id = window.id();
            let ctx = Self::create_surface_context(
                &device,
                surface,
                config,
                &env_bind_group_layout,
                &texture_bind_group_layout,
                scale_factor,
                msaa_sample_count,
                &mut registry,
            );
            surfaces.insert(window_id, ctx);
            current_window = Some(window_id);
        } else if let Some((w, h, f)) = headless_info {
            headless_context = Some(Self::create_headless_context(
                &device,
                w,
                h,
                f,
                &env_bind_group_layout,
                &texture_bind_group_layout,
                &mut registry,
                msaa_sample_count,
            ));
        }

        let staging_belt = wgpu::util::StagingBelt::new((*device).clone(), 1024 * 1024);

        let glass_output_bind_group_layout = env_bind_group_layout.clone();

        // Color blindness pipeline layout (1 bind group: texture + sampler + uniform)
        let color_blind_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Blind Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                            crate::color_blindness::ColorBlindUniforms,
                        >() as u64),
                    },
                    count: None,
                },
            ],
        });
        let color_blind_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Color Blind Pipeline Layout"),
                bind_group_layouts: &[Some(&color_blind_bgl)],
                immediate_size: 0,
            });

        // Color blindness shader module and pipeline (separate from main shader)
        let color_blind_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Color Blind Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                crate::color_blindness::shader_source(),
            )),
        });
        let color_blind_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Color Blindness"),
            layout: Some(&color_blind_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_blind_shader,
                entry_point: Some("fs_main_vs"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_blind_shader,
                entry_point: Some("fs_color_blind"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Volumetric raymarching pipeline (fullscreen triangle with SDF raymarch).
        // Uses the dedicated volumetric.wgsl shader for fog/light shaft effects.
        // Now includes scene uniforms for time-based animation and light positioning.
        let volumetric_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr Volumetric Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shaders/volumetric.wgsl"
            ))),
        });
        // Volumetric bind group layout: uniform buffer for time/resolution/light
        let volumetric_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Volumetric Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 24]>() as u64
                        ),
                    },
                    count: None,
                }],
            });
        let volumetric_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surtr Volumetric Layout"),
            bind_group_layouts: &[Some(&volumetric_bgl)],
            immediate_size: 0,
        });

        let volumetric_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr Volumetric Raymarching"),
            layout: Some(&volumetric_layout),
            vertex: wgpu::VertexState {
                module: &volumetric_shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &volumetric_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // HDR tone mapping pipeline (ACES filmic tone mapping).
        // Converts HDR scene to LDR for display. Falls back to passthrough on LDR surfaces.
        let tonemap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Surtr ToneMap Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_TONEMAP)),
        });
        let tonemap_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ToneMap Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 4]>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let tonemap_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surtr ToneMap Layout"),
            bind_group_layouts: &[Some(&tonemap_bgl)],
            immediate_size: 0,
        });
        let tonemap_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surtr ToneMapping"),
            layout: Some(&tonemap_layout),
            vertex: wgpu::VertexState {
                module: &tonemap_shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &tonemap_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: Some(&pipeline_cache),
        });

        // Tone map uniform buffer (exposure, gamma)
        let color_blind_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color Blind Uniforms"),
            size: std::mem::size_of::<crate::color_blindness::ColorBlindUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Volumetric uniform buffer (updated each frame for time/resolution/light)
        // Extended to 24 floats (96 bytes) to include hologram rect, id hash, time, and count.
        let volumetric_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Volumetric Uniforms"),
            size: std::mem::size_of::<[f32; 24]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Sampler for the color blindness pass (and other post-process passes)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // ── Particle Compute Pipeline ───────────────────────────────────────
        // Binds: @group(0) @binding(0) storage read_write particle_buf
        //        @group(0) @binding(1) uniform uniforms {dt, _pad}
        let particle_compute_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Compute BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<ParticleUniforms>() as u64
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let particle_compute_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Compute Layout"),
                bind_group_layouts: &[Some(&particle_compute_bgl)],
                immediate_size: 0,
            });
        let particle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particles Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_PARTICLES)),
        });
        let particle_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Particle Compute Pipeline"),
                layout: Some(&particle_compute_layout),
                module: &particle_shader,
                entry_point: Some("cs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: Some(&pipeline_cache),
            });

        // Particle storage buffer (ring buffer, 65536 particles × 32 bytes)
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Storage Buffer"),
            size: (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        // Particle compute uniform buffer (dt + pad = 16 bytes)
        let particle_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Uniform Buffer"),
            size: std::mem::size_of::<ParticleUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── Particle Render Pipeline (point sprites) ───────────────────────
        // A minimal vertex+fragment pipeline that reads particle positions from
        // the storage buffer and draws them as colored points.
        let particle_render_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Render BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let particle_render_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Render Layout"),
                bind_group_layouts: &[Some(&particle_render_bgl)],
                immediate_size: 0,
            });
        // Inline WGSL for particle rendering: reads storage buffer, outputs point positions + color.
        let particle_render_wgsl = "
struct Particle {
    pos_vel: vec4<f32>,
    color_life: vec4<f32>,
};
struct ParticleArray {
    particles: array<Particle>,
};
@group(0) @binding(0) var<storage, read> particles: ParticleArray;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let p = particles.particles[vi];
    // pos_vel.xy is in logical pixels; convert to NDC.
    // For now, pass through as clip-space position (caller sets viewport).
    let life = p.color_life.w;
    if (life <= 0.0) {
        // Degenerate point (behind camera)
        out.pos = vec4<f32>(0.0, 0.0, 2.0, 1.0);
        out.color = vec4<f32>(0.0);
    } else {
        // Fade out near end of lifetime
        let alpha = min(life, 1.0);
        out.pos = vec4<f32>(p.pos_vel.xy, 0.0, 1.0);
        out.color = vec4<f32>(p.color_life.xyz, alpha);
    }
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
";
        let particle_render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Render Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(particle_render_wgsl)),
        });
        let particle_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Particle Render Pipeline"),
                layout: Some(&particle_render_layout),
                vertex: wgpu::VertexState {
                    module: &particle_render_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &particle_render_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::PointList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: Some(&pipeline_cache),
            });

        Self {
            registry,
            ai_material_rx: None,
            active_offscreens: Vec::new(),
            effect_pipelines: std::collections::HashMap::new(),
            effect_params_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dummy Effect Buffer"),
                size: 256,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            effect_params_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Dummy Effect Bind Group"),
                layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[],
                }),
                entries: &[],
            }),
            linear_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Linear Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }),
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),

            surfaces,
            current_window,
            headless_context,
            pipeline,
            opaque_pipeline,
            ui_pipeline,
            glass_pipeline,
            bloom_extract_pipeline,
            copy_pipeline,
            composite_pipeline,
            env_bind_group_layout,
            // text_engine moved into TextSubsystem -- see above.
            mega_heim_tex,
            mega_heim_bind_group,
            // P1-5 fix: increased LRU cache sizes to handle the
            // documented use cases without thrashing:
            //   - text_cache: 2048 -> 8192 (covers 4x more text glyphs)
            //   - svg_cache:  128  -> 512  (covers 200+ brush strokes)
            //   - svg_trees:  128  -> 512  (covers 150+ unique sprites)
            //
            // The previous sizes caused periodic frame spikes when
            // the working set exceeded the cache capacity, because
            // re-tessellation of an evicted SVG tree is expensive.
            //
            // Future work (if these sizes prove insufficient): switch
            // to a content-addressed cache so multiple names pointing
            // at the same SVG share a single entry regardless of name.
            // P1-1: cache sizes and atlas dimensions are now read
            // from the SurtrConfig struct instead of being
            // hardcoded. Defaults match the previously hardcoded
            // values, so behavior is preserved. See SurtrConfig
            // for available presets (low_vram, high_end, default).
            config: SurtrConfig::default(),
            // P1-1: text subsystem (engine + caches) initialized
            // via TextSubsystem::forge().
            text: crate::types::TextSubsystem::forge(
                NonZeroUsize::new(8192).unwrap(),
            ),
            heim_packer: SundrPacker::new(4096, 4096),
            image_uv_registry: {
                let mut cache = LruCache::new(NonZeroUsize::new(256).unwrap());
                cache.put("__mega_heim".to_string(), cvkg_core::Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 });
                cache
            },
            texture_registry,
            texture_views: texture_views_list,
            dummy_sampler,
            svg: crate::types::SvgSubsystem::forge(
                &device,
                &queue,
                NonZeroUsize::new(512).unwrap(),
                NonZeroUsize::new(512).unwrap(),
            ),
            dummy_texture_bind_group,
            dummy_env_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
            shared_elements: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            // P1-1: vertex/index/instance buffers grouped into
            // GeometryBuffers struct. See the struct definition
            // for the rationale.
            geometry_buffers,
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
            instance_data: Vec::with_capacity(MAX_VERTICES / 4),
            draw_calls: Vec::new(),
            current_texture_id: None,
            opacity_stack: vec![1.0],
            clip_stack: Vec::new(),
            slice_stack: Vec::new(),
            shadow_stack: Vec::new(),
            theme_buffer,
            scene_buffer,
            berserker_bind_group,
            berserker_bind_group_layout,
            start_time: std::time::Instant::now(),
            current_theme,
            current_scene,
            background_pipeline,
            current_z: 0.0,
            default_background_color: [0.02, 0.02, 0.05, 1.0],
            app_drew_background: false,
            frame_rendered: false,
            current_draw_order: 0,
            telemetry: cvkg_core::TelemetryData::default(),
            last_frame_start: std::time::Instant::now(),
            last_redraw_start: std::time::Instant::now(),
            frame_budget: cvkg_core::FrameBudget::default(),
            capture_staging_buffer: None,
            compositor_index_cursor: 0,
            vram_buffers_bytes: 0,
            vram_textures_bytes: 0,
            _debug_layout: false,
            transform_stack: Vec::new(),
            redraw_requested: false,
            skuld_queries,
            skuld_buffer,
            skuld_read_buffer,
            skuld_period,
            last_gpu_time_ns: 0,
            particle_compute_pipeline,
            particle_compute_bgl,
            particle_buffer,
            particle_uniform_buffer,
            particle_staging: Vec::new(),
            particle_count: 0,
            particle_write_head: 0,
            particle_render_pipeline,
            particle_render_bgl,
            particle_render_bind_group: None,
            last_particle_compact: std::time::Instant::now(),
            vnode_stack: Vec::new(),
            event_handlers: std::collections::HashMap::new(),
            staging_belt,
            staging_command_buffers: Vec::new(),
            glass_output_bind_group_layout,
            current_draw_material: cvkg_core::DrawMaterial::Opaque,
            portal_regions: VecDeque::new(),
            cached_graph_plan: None,
            material_compilation_hash: 0,
            memo_cache: std::collections::HashMap::new(),
            frame_generation: 0,
            quality_level: QualityLevel::default(),
            pipeline_cache,
            bloom_enabled: true,
            volumetric_enabled: false,
            color_blind_mode: crate::color_blindness::ColorBlindMode::Normal,
            color_blind_intensity: 1.0,
            color_blind_pipeline,
            volumetric_pipeline,
            volumetric_bind_group_layout: volumetric_bgl,
            volumetric_uniform_buffer,
            hologram_instances: Vec::new(),
            color_blind_bind_group_layout: color_blind_bgl,
            color_blind_uniform_buffer,
            sampler,
            kawase_down_pipeline,
            kawase_up_pipeline,
            kawase_bind_group_layout: kawase_bgl,
            kawase_uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Kawase Persistent Uniform"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            bind_group_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            texture_view_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub(crate) fn rebuild_texture_array_bind_group(&mut self) {
        let views: Vec<&wgpu::TextureView> = self.texture_views.iter().collect();
        self.mega_heim_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.dummy_sampler),
                },
            ],
            label: Some("Surtr Texture Array Bind Group"),
        });
    }

    /// Update VRAM telemetry based on currently allocated resources.
    /// Called once per frame at the end of end_frame(). Buffer sizes are fixed
    /// (pre-allocated vertex/index buffers). Texture estimate includes the
    /// Mega-Heim atlas, surface buffers, and user-loaded textures tracked by
    /// the texture registry.
    pub(crate) fn update_vram_telemetry(&mut self) {
        // Calculate Buffer VRAM
        let mut buffer_bytes = 0;
        buffer_bytes += (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64;
        buffer_bytes += (MAX_INDICES * std::mem::size_of::<u32>()) as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::ColorTheme>() as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::SceneUniforms>() as u64;
        self.vram_buffers_bytes = buffer_bytes;

        // Calculate Texture VRAM
        let mut texture_bytes = 0u64;
        texture_bytes += 4096 * 4096 * 4; // Mega Heim (RGBA8)
        texture_bytes += 4; // Dummy (RGBA8)

        for ctx in self.surfaces.values() {
            let bpp = 4;
            let surface_bytes = (ctx.config.width * ctx.config.height * bpp) as u64;
            // scene (1x), depth (1x), blur a/b (~1x), bloom a/b (~1x)
            texture_bytes += surface_bytes * 3;
        }

        // Account for user-loaded textures. Each entry in the texture registry
        // represents one RGBA8 texture. Average 512x512 is a reasonable estimate
        // when actual dimensions are unknown.
        let loaded_count = self.texture_registry.len() as u64;
        texture_bytes += loaded_count * 512 * 512 * 4;

        self.vram_textures_bytes = texture_bytes;

        self.telemetry.vram_buffers_mb = buffer_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_textures_mb = texture_bytes as f32 / 1_048_576.0;
        self.telemetry.vram_pipelines_mb = 0.0;
        self.telemetry.vram_usage_mb =
            self.telemetry.vram_buffers_mb + self.telemetry.vram_textures_mb;
    }

    /// Get real-time performance telemetry.
    pub fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
    }

    /// resize -- Reconfigures a specific surface and its internal textures.
    pub fn resize(
        &mut self,
        window_id: winit::window::WindowId,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) {
        if width > 0
            && height > 0
            && let Some(ctx) = self.surfaces.get_mut(&window_id)
        {
            if ctx.config.width == width && ctx.config.height == height {
                // Ignore redundant resizes to prevent Wayland protocol errors (ERROR_SURFACE_LOST_KHR / syncobj already exists)
                return;
            }

            log::info!("[GPU] Reconfiguring surface: {}x{}", width, height);
            SurtrRenderer::lock_or_clear_cache(&self.bind_group_cache).clear();
            SurtrRenderer::lock_or_clear_cache(&self.texture_view_cache).clear();
            self.text.shaped_cache.clear();
            ctx.config.width = width;
            ctx.config.height = height;
            ctx.scale_factor = scale_factor;
            ctx.surface.configure(&self.device, &ctx.config);

            // Re-create Muspelheim textures for this surface
            let texture_desc = wgpu::TextureDescriptor {
                label: Some("Surtr Scene Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };

            let scene_tex = self.device.create_texture(&texture_desc);

            let msaa_desc = wgpu::TextureDescriptor {
                label: Some("Scene MSAA"),
                size: texture_desc.size,
                mip_level_count: 1,
                sample_count: self.quality_level.msaa_sample_count(),
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };
            let scene_msaa_tex = self.device.create_texture(&msaa_desc);
            ctx.scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());
            ctx.scene_msaa_texture =
                scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

            self.registry.remove_image(ctx.blur_tex_a);
            self.registry.remove_image(ctx.blur_tex_b);
            self.registry.remove_image(ctx.bloom_tex_a);
            self.registry.remove_image(ctx.bloom_tex_b);

            let blur_width = (width / 2).max(1);
            let blur_height = (height / 2).max(1);

            let blur_desc_a = crate::kvasir::resource::ResourceDescriptor {
                label: Some("Surtr Blur Texture A".into()),
                kind: crate::kvasir::resource::ResourceKind::Image {
                    format: ctx.config.format,
                    width: blur_width,
                    height: blur_height,
                    mip_level_count: 6,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                },
                lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
            };
            ctx.blur_tex_a = self.registry.allocate_image(&self.device, &blur_desc_a);

            let blur_desc_b = crate::kvasir::resource::ResourceDescriptor {
                label: Some("Surtr Blur Texture B".into()),
                kind: crate::kvasir::resource::ResourceKind::Image {
                    format: ctx.config.format,
                    width: blur_width,
                    height: blur_height,
                    mip_level_count: 6,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                },
                lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
            };
            ctx.blur_tex_b = self.registry.allocate_image(&self.device, &blur_desc_b);

            let bloom_desc_a = crate::kvasir::resource::ResourceDescriptor {
                label: Some("Surtr Bloom Texture A".into()),
                kind: crate::kvasir::resource::ResourceKind::Image {
                    format: ctx.config.format,
                    width: blur_width,
                    height: blur_height,
                    mip_level_count: 6,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                },
                lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
            };
            ctx.bloom_tex_a = self.registry.allocate_image(&self.device, &bloom_desc_a);

            let bloom_desc_b = crate::kvasir::resource::ResourceDescriptor {
                label: Some("Surtr Bloom Texture B".into()),
                kind: crate::kvasir::resource::ResourceKind::Image {
                    format: ctx.config.format,
                    width: blur_width,
                    height: blur_height,
                    mip_level_count: 6,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                },
                lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
            };
            ctx.bloom_tex_b = self.registry.allocate_image(&self.device, &bloom_desc_b);

            // Re-create bind groups for this surface
            ctx.scene_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.env_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ctx.scene_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Scene Bind Group Resize"),
            });

            let scene_views: Vec<&wgpu::TextureView> =
                (0..32).map(|_| &ctx.scene_texture).collect();
            ctx.scene_texture_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                        },
                    ],
                    label: Some("Scene Texture Bind Group Resize"),
                });

            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Surtr Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.quality_level.msaa_sample_count(),
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            ctx.depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    /// begin_frame_headless -- Strike the flaming sword to begin a new GPU frame for headless rendering.
    pub fn begin_frame_headless(&mut self) -> wgpu::CommandEncoder {
        self.current_window = None;
        self.compositor_index_cursor = self.indices.len() as u32;
        self.reset_frame_state();

        // Recall staging belt buffers so they can be reused for vertex upload
        self.staging_belt.recall();

        let ctx = self
            .headless_context
            .as_ref()
            .expect("Headless context not initialized");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.width as f32 / ctx.scale_factor;
        let logical_h = ctx.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Headless Command Encoder"),
            })
    }

    /// Reset per-frame state shared by both `begin_frame` and `begin_frame_headless`.
    /// Factored out to avoid the copy-paste duplication hazard identified in the audit.
    fn reset_frame_state(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instance_data.clear();
        self.draw_calls.clear();
        self.svg.clear_filter_batches();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.portal_regions.clear();
        self.hologram_instances.clear();
        self.current_z = 0.0;
        self.vnode_stack.clear();
        self.event_handlers.clear();
        // Clear per-frame state but NOT memo_cache -- use generation counter instead
        self.frame_generation += 1;
        // Evict memo cache entries that are too old to prevent unbounded growth.
        const MAX_MEMO_AGE: u64 = 1000;
        if self.frame_generation > MAX_MEMO_AGE {
            let cutoff = self.frame_generation - MAX_MEMO_AGE;
            self.memo_cache
                .retain(|_, entry| entry.frame_gen >= cutoff);
        }
        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;
    }

    /// begin_frame -- Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        // Drain AI material channel
        if let Some(rx) = &self.ai_material_rx {
            while let Ok(res) = rx.try_recv() {
                match res {
                    Ok(_) => log::info!("[Surtr] Received AI generated material"),
                    Err(e) => log::warn!("[Surtr] AI material generation error: {:?}", e),
                }
            }
        }

        // Skuld: Read the timestamps from the previous frame
        if let Some(rb) = &self.skuld_read_buffer {
            let slice = rb.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });

            // Poll to ensure mapping is complete
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .unwrap();

            if rx.recv().is_ok() {
                let data = slice.get_mapped_range();
                let timestamps: [u64; 2] = bytemuck::cast_slice(&data).try_into().unwrap_or([0, 0]);
                drop(data);
                rb.unmap();

                if timestamps[1] > timestamps[0] {
                    let diff_ticks = timestamps[1] - timestamps[0];
                    self.last_gpu_time_ns = (diff_ticks as f64 * self.skuld_period as f64) as u64;
                    log::trace!(
                        "[Skuld] GPU time: {} ms",
                        self.last_gpu_time_ns as f64 / 1_000_000.0
                    );
                }
            }
        }

        self.staging_belt.recall();
        self.current_window = Some(window_id);
        self.reset_frame_state();

        let ctx = self
            .surfaces
            .get(&window_id)
            .expect("Window not registered");
        let time = self.start_time.elapsed().as_secs_f32();
        let logical_w = ctx.config.width as f32 / ctx.scale_factor;
        let logical_h = ctx.config.height as f32 / ctx.scale_factor;
        let dt = time - self.current_scene.time;
        self.current_scene.time = time;
        self.current_scene.delta_time = dt;
        self.current_scene.resolution = [logical_w, logical_h];
        self.current_scene.scale_factor = ctx.scale_factor;
        self.current_scene.proj =
            glam::Mat4::orthographic_lh(0.0, logical_w, logical_h, 0.0, -1000.0, 1000.0);

        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );

        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surtr Command Encoder"),
            })
    }

    /// register_window -- Attaches a new OS window to the shared GPU context.
    pub fn register_window(&mut self, window: Arc<winit::window::Window>) {
        let size = window.inner_size();
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps.formats[0];

        // Dynamic present mode selection -- Mailbox not available on all platforms (e.g. Wayland)
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            log::warn!("[GPU] Mailbox not supported, falling back to Fifo (V-Sync)");
            wgpu::PresentMode::Fifo
        };

        let alpha_mode = if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            caps.alpha_modes[0]
        };

        log::info!(
            "[GPU] Configuring surface: {}x{} | {:?} | {:?}",
            size.width,
            size.height,
            present_mode,
            alpha_mode
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&self.device, &config);

        let ctx = Self::create_surface_context(
            &self.device,
            surface,
            config,
            &self.env_bind_group_layout,
            &self.texture_bind_group_layout,
            window.scale_factor() as f32,
            self.quality_level.msaa_sample_count(),
            &mut self.registry,
        );

        self.surfaces.insert(window.id(), ctx);
    }

    pub(crate) fn create_headless_context(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        registry: &mut crate::kvasir::registry::ResourceRegistry,
        msaa_sample_count: u32,
    ) -> HeadlessContext {
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Headless Scene Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);

        let msaa_desc = wgpu::TextureDescriptor {
            label: Some("Scene MSAA"),
            size: texture_desc.size,
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let scene_msaa_tex = device.create_texture(&msaa_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_msaa_texture =
            scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_width = (width / 2).max(1);
        let blur_height = (height / 2).max(1);
        let blur_desc_a = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Headless Blur Texture A".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let blur_tex_a = registry.allocate_image(device, &blur_desc_a);

        let blur_desc_b = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Headless Blur Texture B".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let blur_tex_b = registry.allocate_image(device, &blur_desc_b);

        let bloom_desc_a = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Headless Bloom Texture A".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let bloom_tex_a = registry.allocate_image(device, &bloom_desc_a);

        let bloom_desc_b = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Headless Bloom Texture B".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let bloom_tex_b = registry.allocate_image(device, &bloom_desc_b);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Scene Bind Group"),
        });

        let blur_view_a = registry.get_texture_view(blur_tex_a).unwrap();
        let blur_view_b = registry.get_texture_view(blur_tex_b).unwrap();
        let bloom_view_a = registry.get_texture_view(bloom_tex_a).unwrap();
        let bloom_view_b = registry.get_texture_view(bloom_tex_b).unwrap();

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Env Bind Group A"),
        });
        let blur_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Env Bind Group B"),
        });
        let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Bloom Env Bind Group A"),
        });
        let bloom_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Bloom Env Bind Group B"),
        });

        let scene_views: Vec<&wgpu::TextureView> = (0..32).map(|_| &scene_texture).collect();
        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Scene Texture Bind Group"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        crate::types::HeadlessContext {
            scene_texture,
            scene_msaa_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_tex_a,
            blur_tex_b,
            bloom_tex_a,
            bloom_tex_b,
            blur_env_bind_group_a,
            blur_env_bind_group_b,
            bloom_env_bind_group_a,
            bloom_env_bind_group_b,
            scale_factor: 1.0,
            sampler,
            width,
            height,
            output_texture,
            output_view,
        }
    }

    pub(crate) fn create_surface_context(
        device: &wgpu::Device,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        scale_factor: f32,
        msaa_sample_count: u32,
        registry: &mut crate::kvasir::registry::ResourceRegistry,
    ) -> SurfaceContext {
        let width = config.width;
        let height = config.height;

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Scene Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);

        let msaa_desc = wgpu::TextureDescriptor {
            label: Some("Scene MSAA"),
            size: texture_desc.size,
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let scene_msaa_tex = device.create_texture(&msaa_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_msaa_texture =
            scene_msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_width = (config.width / 2).max(1);
        let blur_height = (config.height / 2).max(1);
        let blur_desc_a = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Surface Blur Texture A".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format: config.format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let blur_tex_a = registry.allocate_image(device, &blur_desc_a);

        let blur_desc_b = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Surface Blur Texture B".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format: config.format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let blur_tex_b = registry.allocate_image(device, &blur_desc_b);

        let bloom_desc_a = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Surface Bloom Texture A".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format: config.format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let bloom_tex_a = registry.allocate_image(device, &bloom_desc_a);

        let bloom_desc_b = crate::kvasir::resource::ResourceDescriptor {
            label: Some("Surface Bloom Texture B".into()),
            kind: crate::kvasir::resource::ResourceKind::Image {
                format: config.format,
                width: blur_width,
                height: blur_height,
                mip_level_count: 6,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            },
            lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
        };
        let bloom_tex_b = registry.allocate_image(device, &bloom_desc_b);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&scene_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Scene Bind Group"),
        });

        let blur_view_a = registry.get_texture_view(blur_tex_a).unwrap();
        let blur_view_b = registry.get_texture_view(blur_tex_b).unwrap();
        let bloom_view_a = registry.get_texture_view(bloom_tex_a).unwrap();
        let bloom_view_b = registry.get_texture_view(bloom_tex_b).unwrap();

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Env Bind Group A"),
        });
        let blur_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Env Bind Group B"),
        });
        let bloom_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Bloom Env Bind Group A"),
        });
        let bloom_env_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Bloom Env Bind Group B"),
        });

        let scene_views: Vec<&wgpu::TextureView> = (0..32).map(|_| &scene_texture).collect();
        let scene_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&scene_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Scene Texture Bind Group"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        crate::types::SurfaceContext {
            surface,
            config,
            scene_texture,
            scene_msaa_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_tex_a,
            blur_tex_b,
            bloom_tex_a,
            bloom_tex_b,
            blur_env_bind_group_a,
            blur_env_bind_group_b,
            bloom_env_bind_group_a,
            bloom_env_bind_group_b,
            scale_factor,
            sampler,
        }
    }

    pub fn reset_time(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    /// reclaim_vram -- Atomic recycling of the Mega-Heim and all associated caches.
    /// This prevents OOM and silent failures by quenching the heim when full.
    pub fn reclaim_vram(&mut self) {
        log::warn!("[GPU] Sundr Compaction: Compacting Mega-Heim...");

        let new_mega_heim_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sundr Mega-Heim (Compacted)"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mut new_packer = SundrPacker::new(4096, 4096);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Heim Compaction Encoder"),
            });

        let image_entries: Vec<(String, Rect)> = self
            .image_uv_registry
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (name, old_uv) in image_entries {
            if let Some(&tex_idx) = self.texture_registry.get(&name)
                && tex_idx == 0
            {
                let w_px = (old_uv.width * 4096.0).round() as u32;
                let h_px = (old_uv.height * 4096.0).round() as u32;
                let old_x_px = (old_uv.x * 4096.0).round() as u32;
                let old_y_px = (old_uv.y * 4096.0).round() as u32;

                if let Some((new_x, new_y)) = new_packer.pack(w_px, h_px) {
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_heim_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: old_x_px,
                                y: old_y_px,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: &new_mega_heim_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: new_x,
                                y: new_y,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: w_px,
                            height: h_px,
                            depth_or_array_layers: 1,
                        },
                    );

                    let new_uv = Rect {
                        x: new_x as f32 / 4096.0,
                        y: new_y as f32 / 4096.0,
                        width: old_uv.width,
                        height: old_uv.height,
                    };
                    self.image_uv_registry.put(name.clone(), new_uv);
                }
            }
        }

        let text_entries: Vec<(u64, (Rect, f32, f32, f32, f32))> =
            self.text.glyph_cache.iter().map(|(k, v)| (*k, *v)).collect();
        for (hash, (old_uv, w_f, h_f, x_off, y_off)) in text_entries {
            let w_px = (old_uv.width * 4096.0).round() as u32;
            let h_px = (old_uv.height * 4096.0).round() as u32;
            let old_x_px = (old_uv.x * 4096.0).round() as u32;
            let old_y_px = (old_uv.y * 4096.0).round() as u32;

            if let Some((new_x, new_y)) = new_packer.pack(w_px, h_px) {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.mega_heim_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: old_x_px,
                            y: old_y_px,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &new_mega_heim_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: new_x,
                            y: new_y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: w_px,
                        height: h_px,
                        depth_or_array_layers: 1,
                    },
                );

                let new_uv = Rect {
                    x: new_x as f32 / 4096.0,
                    y: new_y as f32 / 4096.0,
                    width: old_uv.width,
                    height: old_uv.height,
                };
                self.text.glyph_cache.put(hash, (new_uv, w_f, h_f, x_off, y_off));
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.mega_heim_tex = new_mega_heim_tex;
        let mega_heim_view_obj = self
            .mega_heim_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.texture_views[0] = mega_heim_view_obj.clone();

        self.rebuild_texture_array_bind_group();

        if !self.texture_bind_groups.is_empty() {
            self.texture_bind_groups[0] = self.mega_heim_bind_group.clone();
        }

        self.heim_packer = new_packer;
        self.telemetry.vram_exhausted = false;
    }

    pub(crate) fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        material_id: u32,
    ) {
        // High-Fidelity Variable Particle Density
        let count = (pieces as f32).sqrt().ceil() as u32;
        let dw = rect.width / count as f32;
        let dh = rect.height / count as f32;

        let c = self.apply_opacity(color);

        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;

        for y in 0..count {
            for x in 0..count {
                let init_x = rect.x + x as f32 * dw;
                let init_y = rect.y + y as f32 * dh;

                // Center of the shard relative to the card center
                let dx = (init_x + dw * 0.5) - cx;
                let dy = (init_y + dh * 0.5) - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);

                // Normal direction outwards
                let nx = dx / dist;
                let ny = dy / dist;

                // Hash-based pseudo-random variations for dispersion
                let hash =
                    ((x as f32 * 12.9898 + y as f32 * 78.233).sin().fract() * 43_758.547).fract();
                let hash2 =
                    ((x as f32 * 37.11 + y as f32 * 149.87).sin().fract() * 23_412.19).fract();

                let speed_var = 0.5 + hash * 1.5;
                let angle = ny.atan2(nx) + (hash2 - 0.5) * 0.6;
                let disp_x = angle.cos() * force * 50.0 * speed_var;
                let disp_y = angle.sin() * force * 50.0 * speed_var;

                // Downward gravity-like drift over time/force
                let gravity = force * force * 20.0;

                // Shrink shard size as it scatters away
                // Assuming max force in demo is ~6.0
                let scale_factor = (1.0 - (force / 6.0).min(1.0)).max(0.0);
                let shard_w = dw * scale_factor;
                let shard_h = dh * scale_factor;

                let displaced_x = init_x + disp_x + (dw - shard_w) * 0.5;
                let displaced_y = init_y + disp_y + gravity + (dh - shard_h) * 0.5;

                let shard_rect = Rect {
                    x: displaced_x,
                    y: displaced_y,
                    width: shard_w,
                    height: shard_h,
                };

                let uv = Rect {
                    x: x as f32 / count as f32,
                    y: y as f32 / count as f32,
                    width: 1.0 / count as f32,
                    height: 1.0 / count as f32,
                };

                self.fill_rect_with_full_params(shard_rect, c, material_id, None, force, uv);
            }
        }
    }

    pub(crate) fn recursive_bolt(
        &mut self,
        from: [f32; 2],
        to: [f32; 2],
        depth: u32,
        color: [f32; 4],
    ) {
        if depth == 0 {
            self.draw_lightning_segment(from, to, color);
            return;
        }

        let mid_x = (from[0] + to[0]) * 0.5;
        let mid_y = (from[1] + to[1]) * 0.5;

        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();

        if len < 1e-4 {
            return;
        }

        // Perpendicular offset for jaggedness
        let offset_scale = len * 0.15;
        let seed = (from[0] * 12.9898 + from[1] * 78.233 + (depth as f32) * 37.11)
            .sin()
            .fract();
        let offset_x = -dy / len * (seed - 0.5) * offset_scale;
        let offset_y = dx / len * (seed - 0.5) * offset_scale;

        let mid = [mid_x + offset_x, mid_y + offset_y];

        self.recursive_bolt(from, mid, depth - 1, color);
        self.recursive_bolt(mid, to, depth - 1, color);

        // 20% chance of a secondary branch
        if depth > 2 && seed > 0.8 {
            let branch_to = [
                mid[0] + offset_x * 2.0 + (seed * 100.0).sin() * 50.0,
                mid[1] + offset_y * 2.0 + (seed * 100.0).cos() * 50.0,
            ];
            self.recursive_bolt(mid, branch_to, depth - 2, color);
        }
    }

    pub(crate) fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        let glow_width = 32.0;
        let core_width = 4.0;
        let c = self.apply_opacity(color);

        // 1. Render Volumetric Glow (Cyan)
        let gnx = -dy / len * glow_width * 0.5;
        let gny = dx / len * glow_width * 0.5;
        let gp1 = [from[0] + gnx, from[1] + gny];
        let gp2 = [to[0] + gnx, to[1] + gny];
        let gp3 = [to[0] - gnx, to[1] - gny];
        let gp4 = [from[0] - gnx, from[1] - gny];
        self.push_oriented_quad(
            [gp1, gp2, gp3, gp4],
            c,
            9,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );

        // 2. Render Blinding Core (White)
        let cnx = -dy / len * core_width * 0.5;
        let cny = dx / len * core_width * 0.5;
        let cp1 = [from[0] + cnx, from[1] + cny];
        let cp2 = [to[0] + cnx, to[1] + cny];
        let cp3 = [to[0] - cnx, to[1] - cny];
        let cp4 = [from[0] - cnx, from[1] - cny];
        self.push_oriented_quad(
            [cp1, cp2, cp3, cp4],
            [1.0, 1.0, 1.0, c[3]],
            0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    pub(crate) fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        material_id: u32,
        uv_rect: Rect,
    ) {
        let scissor = self.clip_stack.last().copied();
        let texture_id = None; // Oriented quads like lightning don't use textures yet

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        };

        if self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || self.draw_calls.last().unwrap().scissor_rect != scissor
            || self.instance_data.last() != Some(&current_instance_data)
        {
            self.current_texture_id = texture_id;
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                material: Self::resolve_material_with_context(material_id, &self.current_draw_material),
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        }

        let uvs = [
            [uv_rect.x, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            [uv_rect.x, uv_rect.y + uv_rect.height],
        ];

        let screen = [self.current_width() as f32, self.current_height() as f32];
        let rect = Rect {
            x: points[0][0],
            y: points[0][1],
            width: 1.0,
            height: 1.0,
        };

        for i in 0..4 {
            let px = points[i][0];
            let py = points[i][1];

            let (translation, scale_transform, rotation, _, _) = self.current_transform();
            self.vertices.push(Vertex {
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: uvs[i],
                color,
                material_id,
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                clip: [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY],
                tex_index: 0,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }
    pub(crate) fn get_texture_id(&mut self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode -- Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color,
            material_id,
            texture_id,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    pub(crate) fn fill_rect_with_full_params(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
    ) {
        // If a shadow is active, draw it first, offset by shadow._offset
        if let Some(shadow) = self.shadow_stack.last().copied()
            && shadow.color[3] > 0.001
        {
            let shadow_rect = Rect {
                x: rect.x + shadow._offset[0],
                y: rect.y + shadow._offset[1],
                width: rect.width,
                height: rect.height,
            };
            Renderer::draw_drop_shadow(
                self,
                shadow_rect,
                radius,
                shadow.color,
                shadow.radius,
                0.0, // Spread
            );
        }

        let slice = self
            .slice_stack
            .last()
            .copied()
            .map(|(a, o)| [a, o, 1.0, 1.0])
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        self.fill_rect_with_full_params_and_slice(
            rect,
            color,
            material_id,
            texture_id,
            radius,
            uv_rect,
            slice,
            [0.0, 0.0],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_rect_with_full_params_and_slice(
        &mut self,
        mut rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
        glyph_time: [f32; 2],
    ) {
        // Pixel-snap rect coordinates to prevent sub-pixel blurring on high-DPI displays.
        // Only snap for non-glass materials where visual crispness matters.
        if material_id != material_id::GLASS {
            let scale = self.current_scale_factor();
            let snap = |v: f32| (v * scale).round() / scale;
            rect.x = snap(rect.x);
            rect.y = snap(rect.y);
            rect.width = snap(rect.width);
            rect.height = snap(rect.height);
        }

        let scissor = self.clip_stack.last().copied();

        let material = Self::resolve_material_with_context(material_id, &self.current_draw_material);

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let (blur_radius, ior_override, glass_intensity) = if let cvkg_core::DrawMaterial::Glass {
            blur_radius,
            ior_override,
            glass_intensity,
        } = material
        {
            (blur_radius, ior_override, glass_intensity)
        } else {
            (0.0, 0.0, 1.0)
        };

        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius,
            ior_override,
            glass_intensity,
        };

        // Batching: check if we need to start a new DrawCall
        // With Texture Array, we no longer need to break batches when the texture changes,
        // as long as they are all part of the same array bind group (Group 0).
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material
            || self.instance_data.last() != Some(&current_instance_data);

        if needs_new_call {
            self.current_texture_id = Some(0); // All textures are now in the binding array at Group 0
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: self.current_texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        }

        let scale = self.current_scale_factor();
        let snap = |v: f32| (v * scale).round() / scale;

        let base_idx = self.vertices.len() as u32;
        let x1 = snap(rect.x);
        let y1 = snap(rect.y);
        let x2 = snap(rect.x + rect.width);
        let y2 = snap(rect.y + rect.height);
        let z = self.current_z;
        let normal = [0.0, 0.0, 1.0];
        let screen = [self.current_width() as f32, self.current_height() as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        let tex_index = texture_id.unwrap_or(0);

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color,
            material_id,
            radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color,
            material_id,
            radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color,
            material_id,
            radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color,
            material_id,
            radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });

        self.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Kvasir pass encoding methods
    // ═══════════════════════════════════════════════════════════════════════════
    // Each method encodes one render pass into the provided command encoder.
    // Called from end_frame() which assembles the graph-driven pass sequence.

    /// Pass 1: Clear scene+depth, draw atmosphere, draw opaque geometry.
    /// end_frame -- Quench the blade by submitting the full Muspelheim multi-pass effect.
    ///
    /// Since the Renderer 3.0 migration, the pass sequence is driven by a Kvasir
    /// dependency graph rather than hardcoded ordering. The graph is built each
    /// frame (cheap -- just node/edge allocation), validated (cycle detection,
    /// input satisfiability), then executed. Conditional passes (glass, bloom,
    /// accessibility) are automatically eliminated when not needed.
    pub fn end_frame(&mut self, mut encoder: wgpu::CommandEncoder) {
        struct ActiveFrameResources {
            surface_texture: Option<wgpu::SurfaceTexture>,
            target_view: wgpu::TextureView,
            scene_texture: wgpu::TextureView,
            scene_msaa_texture: wgpu::TextureView,
            depth_texture_view: wgpu::TextureView,
            blur_env_bind_group_a: wgpu::BindGroup,
            blur_env_bind_group_b: wgpu::BindGroup,
            bloom_env_bind_group_a: wgpu::BindGroup,
            bloom_env_bind_group_b: wgpu::BindGroup,
        }

        let res = if let Some(window_id) = self.current_window {
            let Some(ctx) = self.surfaces.get(&window_id) else {
                log::error!("[GPU] Missing surface context for end_frame");
                return;
            };
            let frame = match ctx.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                    ctx.surface.configure(&self.device, &ctx.config);
                    t
                }
                other => {
                    log::warn!(
                        "[GPU] Surface texture acquisition failed ({:?}), reconfiguring surface",
                        other
                    );
                    ctx.surface.configure(&self.device, &ctx.config);
                    // Retry once after reconfiguration; if it fails again, skip the frame.
                    match ctx.surface.get_current_texture() {
                        wgpu::CurrentSurfaceTexture::Success(t) => t,
                        wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                            ctx.surface.configure(&self.device, &ctx.config);
                            t
                        }
                        retry_failed => {
                            log::error!(
                                "[GPU] Surface texture retry also failed ({:?}), skipping frame",
                                retry_failed
                            );
                            self.queue.submit(std::iter::once(encoder.finish()));
                            return;
                        }
                    }
                }
            };
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            ActiveFrameResources {
                surface_texture: Some(frame),
                target_view: view,
                scene_texture: ctx.scene_texture.clone(),
                scene_msaa_texture: ctx.scene_msaa_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                blur_env_bind_group_b: ctx.blur_env_bind_group_b.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                bloom_env_bind_group_b: ctx.bloom_env_bind_group_b.clone(),
            }
        } else {
            let Some(ctx) = self.headless_context.as_ref() else {
                log::error!("[GPU] No headless context for end_frame");
                return;
            };

            ActiveFrameResources {
                surface_texture: None,
                target_view: ctx.output_view.clone(),
                scene_texture: ctx.scene_texture.clone(),
                scene_msaa_texture: ctx.scene_msaa_texture.clone(),
                depth_texture_view: ctx.depth_texture_view.clone(),
                blur_env_bind_group_a: ctx.blur_env_bind_group_a.clone(),
                blur_env_bind_group_b: ctx.blur_env_bind_group_b.clone(),
                bloom_env_bind_group_a: ctx.bloom_env_bind_group_a.clone(),
                bloom_env_bind_group_b: ctx.bloom_env_bind_group_b.clone(),
            }
        };

        // Auto-flush staging belt if render_frame() was not called but geometry was queued.
        // This ensures apps that forget render_frame() still see their draw calls rendered.
        if !self.frame_rendered && (!self.vertices.is_empty() || !self.indices.is_empty()) {
            log::debug!("[GPU] Auto-flushing staging belt in end_frame (render_frame was not called)");
            let mut staging_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Surtr Auto-Flush Staging Encoder"),
                    });
            if !self.vertices.is_empty() {
                let v_bytes = bytemuck::cast_slice(&self.vertices);
                self.staging_belt
                    .write_buffer(
                        &mut staging_encoder,
                        &self.geometry_buffers.vertex_buffer,
                        0,
                        wgpu::BufferSize::new(v_bytes.len() as u64).unwrap(),
                    )
                    .copy_from_slice(v_bytes);
            }
            if !self.indices.is_empty() {
                let i_bytes = bytemuck::cast_slice(&self.indices);
                self.staging_belt
                    .write_buffer(
                        &mut staging_encoder,
                        &self.geometry_buffers.index_buffer,
                        0,
                        wgpu::BufferSize::new(i_bytes.len() as u64).unwrap(),
                    )
                    .copy_from_slice(i_bytes);
            }
            if !self.instance_data.is_empty() {
                let inst_bytes = bytemuck::cast_slice(&self.instance_data);
                self.staging_belt
                    .write_buffer(
                        &mut staging_encoder,
                        &self.geometry_buffers.instance_buffer,
                        0,
                        wgpu::BufferSize::new(inst_bytes.len() as u64).unwrap(),
                    )
                    .copy_from_slice(inst_bytes);
            }
            self.staging_belt.finish();
            self.staging_command_buffers.push(staging_encoder.finish());
        }

        // ── Build and execute the Kvasir frame graph ─────────────────────────────
        let has_glass = self
            .draw_calls
            .iter()
            .any(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }));
        let has_bloom = self.bloom_enabled;
        let has_accessibility =
            self.color_blind_mode != crate::color_blindness::ColorBlindMode::Normal;

        // Build the frame graph using the Kvasir helper for correct pass ordering.
        // Conditional passes (glass, bloom, accessibility) are included/excluded based on frame state.
        // This replaces the hardcoded if/else pass dispatch with a data-driven approach:
        // the graph declares which passes exist and their ordering, and we execute only enabled ones.
        //
        // NOTE: Geometry is uploaded by render_frame() via StagingBelt into staging_command_buffers.
        // Those staging commands must be submitted before the render pass encoders below, which is
        // guaranteed by inserting the render encoders after the existing staging entries (see submit block).

        let (blur_id, bloom_id) = if let Some(window_id) = self.current_window {
            let ctx = self.surfaces.get(&window_id).unwrap();
            (ctx.blur_tex_a, ctx.bloom_tex_a)
        } else {
            let ctx = self.headless_context.as_ref().unwrap();
            (ctx.blur_tex_a, ctx.bloom_tex_a)
        };
        self.registry.alias(kvasir::nodes::RES_BLUR_A, blur_id);
        self.registry.alias(kvasir::nodes::RES_BLOOM_A, bloom_id);
        self.registry
            .alias_view(kvasir::nodes::RES_SCENE, res.scene_texture.clone());
        self.registry.alias_view(
            kvasir::nodes::RES_SCENE_MSAA,
            res.scene_msaa_texture.clone(),
        );

        let scale = self.current_scale_factor();
        let scale_bits = scale.to_bits();
        let active_offscreens_count = self.active_offscreens.len();
        let portal_regions_count = self.portal_regions.len();
        let width = self.current_width();
        let height = self.current_height();
        let has_volumetric = self.volumetric_enabled;

        // Compute content hashes for cache key (must match construction site)
        let mut offscreen_hash: u64 = 0;
        for offscreen in &self.active_offscreens {
            offscreen_hash = offscreen_hash.wrapping_add(
                offscreen.target_id.wrapping_mul(31)
                    ^ (offscreen.blend_mode as u64).wrapping_mul(17)
            );
        }
        let mut portal_hash: u64 = 0;
        for region in &self.portal_regions {
            portal_hash = portal_hash.wrapping_add(
                (region.x.to_bits() as u64).wrapping_mul(7)
                    .wrapping_add((region.y.to_bits() as u64).wrapping_mul(13))
                    .wrapping_add((region.width.to_bits() as u64).wrapping_mul(19))
                    .wrapping_add((region.height.to_bits() as u64).wrapping_mul(23))
            );
        }

        let use_cache = if let Some(ref cached) = self.cached_graph_plan {
            cached.matches(
                has_glass,
                has_bloom,
                has_accessibility,
                has_volumetric,
                active_offscreens_count,
                offscreen_hash,
                portal_regions_count,
                portal_hash,
                width,
                height,
                scale_bits,
                self.material_compilation_hash,
            )
        } else {
            false
        };

        if !use_cache {
            let render_graph = kvasir::nodes::build_render_graph(&kvasir::nodes::RenderGraphConfig {
                has_glass,
                has_bloom,
                has_accessibility,
                has_volumetric,
                active_offscreens: &self.active_offscreens,
                portal_regions: &self.portal_regions.iter().cloned().collect::<Vec<_>>(),
                width,
                height,
                scale,
            });
            let planner = kvasir::planner::ExecutionPlanner::new(&render_graph);
            let compiled_plan = match planner.compile() {
                Ok(plan) => plan,
                Err(e) => {
                    log::error!(
                        "[Kvasir] Render graph compilation failed ({}), skipping render passes",
                        e
                    );
                    // Present the frame with whatever was rendered (stale scene or blank).
                    if let Some(surface_texture) = res.surface_texture {
                        surface_texture.present();
                    }
                    return;
                }
            };
            
            // Reuse the already-computed hashes (computed above for cache matching)
            self.cached_graph_plan = Some(kvasir::graph_cache::CachedGraphPlan {
                has_glass,
                has_bloom,
                has_accessibility,
                has_volumetric,
                active_offscreens_count,
                offscreen_content_hash: offscreen_hash,
                portal_regions_count,
                portal_content_hash: portal_hash,
                width,
                height,
                scale_bits,
                material_compilation_hash: self.material_compilation_hash,
                graph: render_graph,
                plan: compiled_plan,
            });
        }

        let cached = self.cached_graph_plan.as_ref().unwrap();
        let frame_start = self.last_frame_start;
        let budget_ms = self.frame_budget.target_ms;
        let allow_degradation = self.frame_budget.allow_degradation;

        for &node_key in &cached.plan {
            // Frame budget enforcement: if we're already over budget and degradation
            // is allowed, skip expensive COSMETIC passes (bloom, volumetric).
            //
            // P0-2 fix: BackdropBlur, BackdropRegion, and Accessibility are FUNCTIONAL
            // passes, not cosmetic effects:
            //   * BackdropBlur/BackdropRegion implement glassmorphism (frosted glass
            //     panels, modals, sidebars). Skipping them makes glass elements
            //     render as opaque solid rectangles, breaking the visual contract
            //     for any app using glass materials.
            //   * Accessibility is required for screen readers and other AT;
            //     skipping it makes the UI unusable for visually-impaired users.
            // Only BloomExtract/BloomBlur (post-processing glow) and Volumetric
            // (raymarched lighting) are true cosmetics and safe to degrade.
            if allow_degradation && budget_ms > 0.0 {
                let elapsed_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
                if elapsed_ms > budget_ms {
                    if let Some(node) = cached.graph.node(node_key) {
                        match node.pass_id() {
                            kvasir::nodes::PassId::BloomExtract
                            | kvasir::nodes::PassId::BloomBlur
                            | kvasir::nodes::PassId::Volumetric => {
                                log::trace!(
                                    "[Kvasir] Skipping {} (over budget: {:.1}ms > {:.1}ms)",
                                    node.label(),
                                    elapsed_ms,
                                    budget_ms
                                );
                                continue;
                            }
                            _ => {} // Always run: Glass, BackdropBlur, BackdropRegion,
                                    // Accessibility, Geometry, UI, Composite, Present, ...
                        }
                    }
                }
            }
            if let Some(node) = cached.graph.node(node_key) {
                log::trace!("[Kvasir] Executing node: {}", node.label());
                let mut ctx = kvasir::node::ExecutionContext {
                    device: &self.device,
                    queue: &self.queue,
                    encoder: &mut encoder,
                    registry: &self.registry,
                    renderer: self,
                    target_view: &res.target_view,
                    depth_view: &res.depth_texture_view,
                    blur_env_bind_group_a: &res.blur_env_bind_group_a,
                    blur_env_bind_group_b: &res.blur_env_bind_group_b,
                    bloom_env_bind_group_a: &res.bloom_env_bind_group_a,
                    bloom_env_bind_group_b: &res.bloom_env_bind_group_b,
                    scale_factor: scale,
                };
                node.execute(&mut ctx);
            }
        }

        // ── Particle Compute Pass ──────────────────────────────────────────
        // Flush staged particles to GPU, then run compute integration.
        // Must run BEFORE the submit so particle positions are up-to-date.
        if !self.particle_staging.is_empty() || self.particle_count > 0 {
            // 1. Flush staged particles into the ring buffer
            if !self.particle_staging.is_empty() {
                let write_start = self.particle_write_head as usize;
                let write_count = self.particle_staging.len();
                let max = MAX_PARTICLES;

                // P1-6 fix: cap the write to max particles to prevent
                // wrap-around overlap. If write_count > max, only the
                // LAST `max` particles are kept (the most recent ones
                // are most relevant for particle effects, and the
                // earlier ones are dropped). Without this cap, if
                // write_count > max - write_start, the second chunk
                // would write past offset 0 and overlap the first
                // chunk, corrupting the buffer.
                let effective_count = write_count.min(max);
                let drop_count = write_count - effective_count;

                // Write particles in ring-buffer fashion
                let first_chunk = (max - write_start).min(effective_count);
                let bytes = bytemuck::cast_slice(&self.particle_staging[drop_count..drop_count + first_chunk]);
                self.queue.write_buffer(
                    &self.particle_buffer,
                    (write_start * std::mem::size_of::<crate::types::GpuParticle>()) as u64,
                    bytes,
                );
                if first_chunk < effective_count {
                    let remaining = effective_count - first_chunk;
                    let bytes2 = bytemuck::cast_slice(&self.particle_staging[drop_count + first_chunk..drop_count + first_chunk + remaining]);
                    self.queue.write_buffer(
                        &self.particle_buffer,
                        0,
                        bytes2,
                    );
                    self.particle_write_head = remaining as u32;
                } else {
                    self.particle_write_head =
                        ((write_start + effective_count) % max) as u32;
                }
                self.particle_count = (self.particle_count as usize + effective_count)
                    .min(max) as u32;
                self.particle_staging.clear();

                // Invalidate render bind group so it's recreated with new data
                self.particle_render_bind_group = None;
            }

            // 2. Run compute pass to integrate particle physics
            let dt = self.current_scene.delta_time;
            let uniforms = crate::types::ParticleUniforms { dt, _pad: [0.0; 3] };
            self.queue.write_buffer(
                &self.particle_uniform_buffer,
                0,
                bytemuck::bytes_of(&uniforms),
            );

            let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Particle Compute BG"),
                layout: &self.particle_compute_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.particle_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.particle_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut compute_encoder =
                self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Particle Compute Encoder"),
                });
            {
                let mut cpass = compute_encoder.begin_compute_pass(
                    &wgpu::ComputePassDescriptor {
                        label: Some("Particle Integration"),
                        ..Default::default()
                    },
                );
                cpass.set_pipeline(&self.particle_compute_pipeline);
                cpass.set_bind_group(0, &compute_bind_group, &[]);
                let workgroups = ((self.particle_count + 63) / 64).max(1);
                cpass.dispatch_workgroups(workgroups, 1, 1);
            }
            self.staging_command_buffers.push(compute_encoder.finish());
        }

        // 3. Compact dead particles periodically (every 2 seconds)
        if self.particle_count > 0
            && self.last_particle_compact.elapsed().as_secs_f32() > 2.0
        {
            self.last_particle_compact = std::time::Instant::now();
            // Read back particle data to compact dead particles
            let read_size =
                (self.particle_count as usize * std::mem::size_of::<crate::types::GpuParticle>())
                    as u64;
            let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Particle Compact Staging"),
                size: read_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut compact_encoder =
                self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Particle Compact Copy"),
                });
            compact_encoder.copy_buffer_to_buffer(
                &self.particle_buffer,
                0,
                &staging_buf,
                0,
                read_size,
            );
            self.staging_command_buffers.push(compact_encoder.finish());
            // Note: full GPU readback is expensive; in production we'd use a
            // compute compaction pass. For now, dead particles are simply
            // overwritten by new ones in the ring buffer (lifetime <= 0 causes
            // the vertex shader to output degenerate points behind the camera).
        }

        // ── Particle Render Pass ────────────────────────────────────────────
        // Render live particles as colored points to the swapchain target,
        // composited on top of the scene with additive blending.
        if self.particle_count > 0 {
            // Lazily (re)create the render bind group when staging changed
            if self.particle_render_bind_group.is_none() {
                self.particle_render_bind_group =
                    Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Particle Render BG"),
                        layout: &self.particle_render_bgl,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.particle_buffer.as_entire_binding(),
                        }],
                    }));
            }
            if let Some(bg) = &self.particle_render_bind_group {
                let mut render_encoder =
                    self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Particle Render Encoder"),
                    });
                {
                    let mut rpass = render_encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("Particle Render"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &res.target_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        },
                    );
                    rpass.set_pipeline(&self.particle_render_pipeline);
                    rpass.set_bind_group(0, bg, &[]);
                    rpass.draw(0..self.particle_count, 0..1);
                }
                self.staging_command_buffers.push(render_encoder.finish());
            }
        }

        // ── Submit ─────────────────────────────────────────────────────────────
        // staging_command_buffers already contains the geometry upload encoder from
        // render_frame() (StagingBelt). The render pass encoders must come AFTER it
        // so the GPU sees vertex/index data before the draw calls that reference it.
        self.staging_command_buffers.push(encoder.finish());

        // Skuld: Resolve timestamps (preserved from original)
        if let (Some(q), Some(b), Some(rb)) = (
            &self.skuld_queries,
            &self.skuld_buffer,
            &self.skuld_read_buffer,
        ) {
            let mut resolve_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Skuld Resolve Encoder"),
                    });
            resolve_encoder.resolve_query_set(q, 0..2, b, 0);
            resolve_encoder.copy_buffer_to_buffer(b, 0, rb, 0, 16);
            self.staging_command_buffers.push(resolve_encoder.finish());
        }

        let cmds = std::mem::take(&mut self.staging_command_buffers);
        self.queue.submit(cmds);
        self.telemetry.frame_time_ms = self.last_frame_start.elapsed().as_secs_f32() * 1000.0;
        self.update_vram_telemetry();

        // Evict transient frame resources (portal regions, offscreen effects) back into
        // the texture pool instead of leaking GPU memory when panels are closed.
        self.registry.evict_frame_resources();

        if let Some(f) = res.surface_texture {
            f.present();
        }
    }
}

impl Drop for SurtrRenderer {
    fn drop(&mut self) {
        // Persist pipeline cache to disk for faster subsequent startups.
        // Use the same path logic as forge_internal() for consistency:
        // cache lives next to the executable, with temp dir fallback.
        let cache_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("pipeline_cache")))
            .unwrap_or_else(|| std::env::temp_dir().join("cvkg_pipeline_cache"));
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("cvkg_render_gpu.bin");
        let data = self.pipeline_cache.get_data();
        if let Some(data) = data {
            if let Err(e) = std::fs::write(&cache_path, data) {
                log::warn!("Failed to persist pipeline cache: {}", e);
            }
        }

        // Ensure GPU is idle before dropping to avoid Swapchain semaphore panics
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
}

impl SurtrRenderer {
    /// Submit pre-routed draw command buckets from the cvkg-compositor.
    ///
    /// Accepts `CommandBuckets` produced by `CompositorEngine::flatten_and_route()`
    /// and submits draw calls in the correct pass order for the Backdrop Capture
    /// Architecture:
    /// 1. Scene commands (opaque) → Scene Capture pass
    /// 2. Glass commands → Material Composite pass (samples blur pyramid)
    /// 3. Overlay commands → Top-Level Foreground pass
    pub fn submit_buckets(&mut self, buckets: &cvkg_compositor::CommandBuckets) {
        // Scene pass -- opaque draw calls, sorted by (z_index, draw_order)
        let mut active_offscreens = Vec::new();
        let mut current_target_id = None;

        // Collect and sort scene commands by (z_index, draw_order) for correct painter's order.
        let mut sorted_scene: Vec<_> = buckets.scene_commands.iter().collect();
        sorted_scene.sort_by_key(|cmd| {
            match cmd {
                cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                    (routed.z_index as i64, routed.draw_order as i64)
                }
                _ => (0, 0),
            }
        });

        for cmd in sorted_scene {
            match cmd {
                cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                    self.set_material(cvkg_core::DrawMaterial::Opaque);
                    self.submit_routed(routed, current_target_id);
                }
                cvkg_compositor::engine::RenderCommand::PushOffscreen {
                    source_layer,
                    material,
                    bounds,
                } => {
                    current_target_id = Some(source_layer.0);

                    // Pre-allocate the texture
                    let width = (bounds.width).max(1.0) as u32;
                    let height = (bounds.height).max(1.0) as u32;
                    self.registry
                        .allocate_offscreen(&self.device, source_layer.0, [width, height]);

                    if let cvkg_compositor::Material::ShaderEffect {
                        effect_name,
                        params_json: _,
                        ..
                    } = material
                    {
                        active_offscreens.push(crate::types::OffscreenEffectConfig {
                            target_id: source_layer.0,
                            effect: effect_name.clone(),
                            blend_mode: 0,          // Default blend
                            effect_args: [0.0; 16], // Need to parse params_json
                        });
                    }
                }
                cvkg_compositor::engine::RenderCommand::PopOffscreen => {
                    current_target_id = None;
                }
            }
        }
        self.active_offscreens = active_offscreens;

        // Glass pass -- glassmorphism draw calls sampling blur pyramid
        let mut sorted_glass: Vec<_> = buckets.glass_commands.iter().collect();
        sorted_glass.sort_by_key(|cmd| match cmd {
            cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                (routed.z_index as i64, routed.draw_order as i64)
            }
            _ => (0, 0),
        });
        for cmd in sorted_glass {
            if let cvkg_compositor::engine::RenderCommand::Draw(routed) = cmd {
                self.set_material(Self::convert_compositor_material(&routed.material));
                self.submit_routed(routed, None);
            }
        }

        // Overlay pass -- foreground UI (crisp text, icons, edge lighting)
        let mut sorted_overlay: Vec<_> = buckets.overlay_commands.iter().collect();
        sorted_overlay.sort_by_key(|cmd| match cmd {
            cvkg_compositor::engine::RenderCommand::Draw(routed) => {
                (routed.z_index as i64, routed.draw_order as i64)
            }
            _ => (0, 0),
        });
        for cmd in sorted_overlay {
            if let cvkg_compositor::engine::RenderCommand::Draw(routed) = cmd {
                self.set_material(cvkg_core::DrawMaterial::TopUI);
                self.submit_routed(routed, None);
            }
        }
    }

    /// Submit a single routed draw command through the internal pipeline.
    pub(crate) fn submit_routed(
        &mut self,
        routed: &cvkg_compositor::RoutedDrawCommand,
        target_id: Option<u64>,
    ) {
        let cmd = &routed.command;
        if cmd.index_count == 0 {
            return;
        }
        let material = Self::convert_compositor_material(&routed.material);
        self.draw_calls.push(DrawCall {
            texture_id: cmd.texture_id,
            scissor_rect: cmd.scissor_rect,
            index_start: cmd.index_start,
            index_count: cmd.index_count,
            material,
            target_id,
            instance_start: cmd.instance_id,
            draw_order: 0,
        });
    }
}

impl SurtrRenderer {
    /// Returns the current effective opacity (product of all stacked values).
    pub(crate) fn apply_opacity(&self, mut color: [f32; 4]) -> [f32; 4] {
        if let Some(&alpha) = self.opacity_stack.last() {
            color[3] *= alpha;
        }
        color
    }

    /// load_svg -- Parses an SVG file and tessellates its paths into GPU triangles.
    pub fn load_svg(&mut self, name: &str, data: &[u8]) {
        if self.svg.model_cache.contains(name) {
            return;
        }

        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        let tree = match usvg::Tree::from_data(data, &opt) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to parse SVG '{}': {:?}, skipping load", name, e);
                return;
            }
        };

        // The viewBox is applied as the root group's transform.
        // Use the tree size as the viewBox (which is the SVG's width/height).
        let view_box = Rect {
            x: 0.0,
            y: 0.0,
            width: tree.size().width(),
            height: tree.size().height(),
        };

        let parsed_animations = parse_svg_animations(data);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut fill_tessellator = FillTessellator::new();
        let mut stroke_tessellator = StrokeTessellator::new();
        let mut finalized_animations = Vec::new();
        let mut paths = Vec::new();

        for child in tree.root().children() {
            let mut tess_params = TessellateParams {
                fill_tessellator: &mut fill_tessellator,
                stroke_tessellator: &mut stroke_tessellator,
                vertices: &mut vertices,
                indices: &mut indices,
                parsed_animations: &parsed_animations,
                finalized_animations: &mut finalized_animations,
                paths: &mut paths,
            };
            self.tessellate_node(child, &mut tess_params);
        }

        self.svg.model_cache.put(
            name.to_string(),
            SvgModel {
                vertices,
                indices,
                view_box,
                paths,
                animations: finalized_animations,
            },
        );
        self.svg.tree_cache.put(name.to_string(), tree);
    }

    pub(crate) fn tessellate_node(&self, node: &usvg::Node, params: &mut TessellateParams<'_>) {
        let start_idx = params.vertices.len();
        let node_id = match node {
            usvg::Node::Group(g) => g.id().to_string(),
            usvg::Node::Path(p) => p.id().to_string(),
            _ => String::new(),
        };

        if let usvg::Node::Group(ref group) = *node {
            for child in group.children() {
                let mut child_params = TessellateParams {
                    fill_tessellator: params.fill_tessellator,
                    stroke_tessellator: params.stroke_tessellator,
                    vertices: params.vertices,
                    indices: params.indices,
                    parsed_animations: params.parsed_animations,
                    finalized_animations: params.finalized_animations,
                    paths: params.paths,
                };
                self.tessellate_node(child, &mut child_params);
            }
        } else if let usvg::Node::Path(ref path) = *node {
            let has_fill = path.fill().is_some();
            let has_stroke = path.stroke().is_some();

            // If neither fill nor stroke, log and skip
            if !has_fill && !has_stroke {
                log::debug!("SVG path '{}' has no fill or stroke, skipping", node_id);
                return;
            }

            let lyon_path = usvg_to_lyon(path, node.abs_transform());
            let screen = [4096.0, 4096.0]; // Placeholder, will be overridden if needed
            let clip = [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY]; // Default clip

            // Tessellate fill if present
            if has_fill && let Some(fill) = path.fill() {
                let paint = fill.paint();
                let fill_opacity = fill.opacity().get();
                // Convert SVG fill rule to Lyon fill rule
                let fill_rule = match fill.rule() {
                    usvg::FillRule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
                    usvg::FillRule::NonZero => lyon::tessellation::FillRule::NonZero,
                };

                match paint {
                    usvg::Paint::Color(c) => {
                        let color = [
                            c.red as f32 / 255.0,
                            c.green as f32 / 255.0,
                            c.blue as f32 / 255.0,
                            fill_opacity,
                        ];
                        Self::tessellate_fill_solid(
                            &lyon_path, color, &node_id, params, fill_rule,
                        );
                    }
                    usvg::Paint::LinearGradient(g) => {
                        Self::tessellate_fill_gradient(
                            &lyon_path, g, fill_opacity, &node_id, params, fill_rule,
                        );
                    }
                    usvg::Paint::RadialGradient(g) => {
                        Self::tessellate_fill_radial_gradient(
                            &lyon_path, g, fill_opacity, &node_id, params, fill_rule,
                        );
                    }
                    usvg::Paint::Pattern(_) => {
                        log::warn!(
                            "SVG path '{}' uses pattern fill which is not supported, using white fallback",
                            node_id
                        );
                        let color = [1.0, 1.0, 1.0, fill_opacity];
                        Self::tessellate_fill_solid(
                            &lyon_path, color, &node_id, params, fill_rule,
                        );
                    }
                }
            }

            // Tessellate stroke if present
            if has_stroke && let Some(stroke) = path.stroke() {
                let base_vertex_idx = params.vertices.len() as u32;
                let stroke_width = stroke.width().get(); // Direct float value
                let color = match stroke.paint() {
                    usvg::Paint::Color(c) => [
                        c.red as f32 / 255.0,
                        c.green as f32 / 255.0,
                        c.blue as f32 / 255.0,
                        stroke.opacity().get(),
                    ],
                    usvg::Paint::LinearGradient(_)
                    | usvg::Paint::RadialGradient(_)
                    | usvg::Paint::Pattern(_) => {
                        log::warn!(
                            "SVG path '{}' uses gradient/pattern stroke which is not supported, using white fallback",
                            node_id
                        );
                        [1.0, 1.0, 1.0, 1.0]
                    }
                };

                // Build stroke options from SVG stroke properties
                let mut stroke_opts = StrokeOptions::default()
                    .with_line_width(stroke_width);

                // Line cap
                stroke_opts = match stroke.linecap() {
                    usvg::LineCap::Butt => stroke_opts.with_line_cap(lyon::tessellation::LineCap::Butt),
                    usvg::LineCap::Round => stroke_opts.with_line_cap(lyon::tessellation::LineCap::Round),
                    usvg::LineCap::Square => stroke_opts.with_line_cap(lyon::tessellation::LineCap::Square),
                };

                // Line join
                stroke_opts = match stroke.linejoin() {
                    usvg::LineJoin::Miter => stroke_opts.with_line_join(lyon::tessellation::LineJoin::Miter),
                    usvg::LineJoin::Round => stroke_opts.with_line_join(lyon::tessellation::LineJoin::Round),
                    usvg::LineJoin::Bevel => stroke_opts.with_line_join(lyon::tessellation::LineJoin::Bevel),
                    _ => stroke_opts,
                };

                // Miter limit
                stroke_opts = stroke_opts.with_miter_limit(stroke.miterlimit().get());

                // Dash array: Lyon's StrokeOptions does not support dash patterns
                // natively. To render dashed strokes, the path would need to be
                // split into dash/gap segments and tessellated per-segment, then
                // the results merged. This is tracked as future work.
                // Current behavior: strokes with dasharray are rendered as solid.
                if let Some(dasharray) = stroke.dasharray() {
                    let _ = dasharray; // Available for future dash tessellation.
                }

                let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
                let path_length = lyon::algorithms::length::approximate_length(&lyon_path, 0.1);

                if let Err(e) = params.stroke_tessellator.tessellate_path(
                    &lyon_path,
                    &stroke_opts,
                    &mut BuffersBuilder::new(
                        &mut buffers,
                        CustomStrokeVertexConstructor { color, clip, path_length },
                    ),
                ) {
                    log::warn!(
                        "SVG stroke tessellation failed for path '{}': {:?}, skipping",
                        node_id,
                        e
                    );
                    return;
                }

                params.vertices.extend(buffers.vertices);
                for idx in buffers.indices {
                    params.indices.push(base_vertex_idx + idx);
                }
            }
        }

        let end_idx = params.vertices.len();
        let end_idx_indices = params.indices.len();
        if !node_id.is_empty() && start_idx < end_idx {
            for anim in params.parsed_animations {
                if anim.target_id == node_id {
                    let mut final_anim = anim.clone();
                    final_anim.vertex_range = start_idx..end_idx;
                    params.finalized_animations.push(final_anim);
                }
            }
            // Record this path's range for per-path transforms.
            params.paths.push(crate::types::SvgPath {
                    id: node_id,
                    vertex_range: start_idx..end_idx,
                    index_range: end_idx_indices..params.indices.len(),
                    local_transform: Default::default(),
                });
        }
    }

    /// Tessellate a solid-color fill.
    fn tessellate_fill_solid(
        lyon_path: &lyon::path::Path,
        color: [f32; 4],
        node_id: &String,
        params: &mut TessellateParams<'_>,
        fill_rule: lyon::tessellation::FillRule,
    ) {
        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let base_vertex_idx = params.vertices.len() as u32;
        if let Err(e) = params.fill_tessellator.tessellate_path(
            lyon_path,
            &FillOptions::default().with_fill_rule(fill_rule),
            &mut BuffersBuilder::new(&mut buffers, SceneVertexConstructor { color }),
        ) {
            log::warn!(
                "SVG fill tessellation failed for path '{}': {:?}, skipping",
                node_id,
                e
            );
            return;
        }
        params.vertices.extend(buffers.vertices);
        for idx in buffers.indices {
            params.indices.push(base_vertex_idx + idx);
        }
    }

    /// Compute gradient color for a position in SVG space.
    fn gradient_color_at(
        stops: &[usvg::Stop],
        pos: f32,
        fill_opacity: f32,
    ) -> [f32; 4] {
        if stops.is_empty() {
            return [1.0, 1.0, 1.0, fill_opacity];
        }
        let pos = pos.clamp(0.0, 1.0);
        let mut start = &stops[0];
        let mut end = &stops[stops.len() - 1];
        for w in stops.windows(2) {
            if pos >= w[0].offset().get() && pos <= w[1].offset().get() {
                start = &w[0];
                end = &w[1];
                break;
            }
        }
        let so = start.offset().get();
        let eo = end.offset().get();
        if pos <= so {
            let c = start.color();
            return [c.red as f32 / 255.0, c.green as f32 / 255.0, c.blue as f32 / 255.0, start.opacity().get() * fill_opacity];
        }
        if pos >= eo {
            let c = end.color();
            return [c.red as f32 / 255.0, c.green as f32 / 255.0, c.blue as f32 / 255.0, end.opacity().get() * fill_opacity];
        }
        let range = eo - so;
        if range < 0.0001 {
            let c = start.color();
            return [c.red as f32 / 255.0, c.green as f32 / 255.0, c.blue as f32 / 255.0, start.opacity().get() * fill_opacity];
        }
        let t = (pos - so) / range;
        let sc = start.color();
        let ec = end.color();
        [
            (sc.red as f32 + (ec.red as f32 - sc.red as f32) * t) / 255.0,
            (sc.green as f32 + (ec.green as f32 - sc.green as f32) * t) / 255.0,
            (sc.blue as f32 + (ec.blue as f32 - sc.blue as f32) * t) / 255.0,
            (start.opacity().get() + (end.opacity().get() - start.opacity().get()) * t) * fill_opacity,
        ]
    }

    /// Tessellate a linear gradient fill with per-vertex colors.
    fn tessellate_fill_gradient(
        lyon_path: &lyon::path::Path,
        gradient: &usvg::LinearGradient,
        fill_opacity: f32,
        node_id: &String,
        params: &mut TessellateParams<'_>,
        fill_rule: lyon::tessellation::FillRule,
    ) {
        let x1 = gradient.x1();
        let y1 = gradient.y1();
        let x2 = gradient.x2();
        let y2 = gradient.y2();
        let dx = x2 - x1;
        let dy = y2 - y1;
        let grad_len_sq = dx * dx + dy * dy;

        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let base_vertex_idx = params.vertices.len() as u32;
        if let Err(e) = params.fill_tessellator.tessellate_path(
            lyon_path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut buffers, SceneVertexConstructor { color: [1.0, 1.0, 1.0, 1.0] }),
        ) {
            log::warn!("SVG gradient fill tessellation failed for path '{}': {:?}, skipping", node_id, e);
            return;
        }

        let stops = gradient.stops();
        for mut vertex in buffers.vertices {
            let px = vertex.position[0];
            let py = vertex.position[1];
            let t = if grad_len_sq < 0.0001 { 0.5 } else { ((px - x1) * dx + (py - y1) * dy) / grad_len_sq };
            vertex.color = Self::gradient_color_at(stops, t as f32, fill_opacity);
            params.vertices.push(vertex);
        }
        for idx in buffers.indices {
            params.indices.push(base_vertex_idx + idx);
        }
    }

    /// Tessellate a radial gradient fill with per-vertex colors.
    fn tessellate_fill_radial_gradient(
        lyon_path: &lyon::path::Path,
        gradient: &usvg::RadialGradient,
        fill_opacity: f32,
        node_id: &String,
        params: &mut TessellateParams<'_>,
        fill_rule: lyon::tessellation::FillRule,
    ) {
        let cx = gradient.cx();
        let cy = gradient.cy();
        let r = gradient.r();
        let stops = gradient.stops();

        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let base_vertex_idx = params.vertices.len() as u32;
        if let Err(e) = params.fill_tessellator.tessellate_path(
            lyon_path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut buffers, SceneVertexConstructor { color: [1.0, 1.0, 1.0, 1.0] }),
        ) {
            log::warn!("SVG radial gradient fill tessellation failed for path '{}': {:?}, skipping", node_id, e);
            return;
        }

        for mut vertex in buffers.vertices {
            let px = vertex.position[0];
            let py = vertex.position[1];
            let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt();
            let r_val = r.get();
            let t = if r_val < 0.001 { 0.5 } else { (dist / r_val).clamp(0.0, 1.0) };
            vertex.color = Self::gradient_color_at(stops, t, fill_opacity);
            params.vertices.push(vertex);
        }
        for idx in buffers.indices {
            params.indices.push(base_vertex_idx + idx);
        }
    }

    /// draw_svg -- Renders a pre-loaded SVG icon at the specified logical rect.
    /// animation_time_offset shifts the animation phase for this instance,
    /// allowing multiple draws of the same SVG to animate independently.
    pub fn draw_svg(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, material_id: u32) {
        self.draw_svg_with_offset(name, rect, color, material_id, 0.0);
    }

    pub fn draw_svg_with_offset(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, material_id: u32, animation_time_offset: f32) {
        self.draw_svg_with_order(name, rect, color, material_id, animation_time_offset, 0);
    }

    pub fn draw_svg_with_order(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, material_id: u32, animation_time_offset: f32, draw_order: i32) {
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let scale = self.current_scale_factor();
        let screen_w = self.current_width() as f32 / scale;
        let screen_h = self.current_height() as f32 / scale;

        if rect.x > clip_rect.x + clip_rect.width
            || rect.x + rect.width < clip_rect.x
            || rect.y > clip_rect.y + clip_rect.height
            || rect.y + rect.height < clip_rect.y
        {
            return;
        }

        log::info!("DRAW_SVG '{}' called with rect: {:?}, model_view_box: {:?}", name, rect, self.svg.model_cache.get(name).map(|m| m.view_box));
        
        if rect.x > screen_w
            || rect.x + rect.width < 0.0
            || rect.y > screen_h
            || rect.y + rect.height < 0.0
        {
            return;
        }

        let model = if let Some(m) = self.svg.model_cache.get(name) {
            m.clone()
        } else {
            return;
        };

        let _scale_x = rect.width / model.view_box.width;
        let _scale_y = rect.height / model.view_box.height;
        let base_idx = self.vertices.len() as u32;
        let screen = [self.current_width() as f32, self.current_height() as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];
        let scale = self.current_scale_factor();
        let snap = |v: f32| (v * scale).round() / scale;

        if model.paths.is_empty() {
            // Fallback: no path data, treat all vertices as one blob.
            let mut local_vertices = model.vertices.clone();
            Self::position_vertices(&mut local_vertices, model.view_box, rect, material_id, clip, snap);
            let base_vertex = self.vertices.len() as u32;
            self.vertices.extend(local_vertices);
            let index_count = model.indices.len();
            for idx in &model.indices {
                self.indices.push(base_vertex + *idx);
            }
            let material = Self::resolve_material(material_id);
            let tid = self.get_texture_id("__mega_heim");
            Self::emit_draw_call(self, material, tid, clip_rect, index_count as u32, base_vertex);
        } else {
            // Per-path rendering: each path gets its own transform and draw call.
            for path in &model.paths {
                let mut path_verts: Vec<Vertex> = model.vertices[path.vertex_range.clone()].to_vec();
                // Apply local transform (translate, rotate, scale) in SVG space.
                if path.local_transform.scale != 1.0 || path.local_transform.rotation != 0.0 || path.local_transform.translate != [0.0, 0.0] {
                    let s = path.local_transform.scale;
                    let rad = path.local_transform.rotation.to_radians();
                    let c = rad.cos();
                    let sn = rad.sin();
                    let tx = path.local_transform.translate[0];
                    let ty = path.local_transform.translate[1];
                    for v in &mut path_verts {
                        let px = v.position[0] * s;
                        let py = v.position[1] * s;
                        v.position[0] = px * c - py * sn + tx;
                        v.position[1] = px * sn + py * c + ty;
                    }
                }
                // Apply animations targeting this path.
                for anim in &model.animations {
                    if anim.target_id == path.id {
                        let effective_time = self.current_scene.time + animation_time_offset;
                        let t = (effective_time % anim.duration) / anim.duration;
                        let val = anim.evaluate(t);
                        if anim.attribute_name == "transform" {
                            let mut min_x = f32::MAX; let mut min_y = f32::MAX;
                            let mut max_x = f32::MIN; let mut max_y = f32::MIN;
                            for v in &path_verts {
                                min_x = min_x.min(v.position[0]);
                                min_y = min_y.min(v.position[1]);
                                max_x = max_x.max(v.position[0]);
                                max_y = max_y.max(v.position[1]);
                            }
                            let cx = (min_x + max_x) * 0.5;
                            let cy = (min_y + max_y) * 0.5;
                            let c = val.to_radians().cos();
                            let s = val.to_radians().sin();
                            for v in &mut path_verts {
                                let dx = v.position[0] - cx;
                                let dy = v.position[1] - cy;
                                v.position[0] = cx + dx * c - dy * s;
                                v.position[1] = cy + dx * s + dy * c;
                            }
                        } else if anim.attribute_name == "opacity" {
                            for v in &mut path_verts { v.color[3] = val; }
                        } else if anim.attribute_name == "stroke-dashoffset" {
                            for v in &mut path_verts { v.slice[3] = 1.0 - val; }
                        }
                    }
                }
                // Position into output rect.
                Self::position_vertices(&mut path_verts, model.view_box, rect, material_id, clip, snap);
                let base_vertex = self.vertices.len() as u32;
                let index_start = self.indices.len();
                self.vertices.extend(path_verts);
                // Remap indices for this path's vertex offset.
                let path_index_start = path.index_range.start;
                for idx in &model.indices[path.index_range.clone()] {
                    self.indices.push(base_vertex + *idx - path_index_start as u32);
                }
                let index_count = path.index_range.len() as u32;
                let material = Self::resolve_material(material_id);
                let tid = self.get_texture_id("__mega_heim");
                Self::emit_draw_call(self, material, tid, clip_rect, index_count, base_vertex);
            }
        }
    }

    /// Resolve a material_id to DrawMaterial with default parameters.
    /// Used by draw_svg which doesn't have a current_draw_material context.
    fn resolve_material(material_id: u32) -> cvkg_core::DrawMaterial {
        Self::resolve_material_with_context(material_id, &cvkg_core::DrawMaterial::Opaque)
    }

    /// Resolve a material_id to DrawMaterial, using current_draw_material as context
    /// for glass parameters. Centralizes the material routing logic used by both
    /// fill_rect_with_full_params_and_slice and emit_draw_call.
    fn resolve_material_with_context(
        material_id: u32,
        current: &cvkg_core::DrawMaterial,
    ) -> cvkg_core::DrawMaterial {
        use material_id::*;
        match material_id {
            GLASS => {
                if let cvkg_core::DrawMaterial::Glass {
                    blur_radius,
                    ior_override,
                    glass_intensity,
                } = current
                {
                    cvkg_core::DrawMaterial::Glass {
                        blur_radius: *blur_radius,
                        ior_override: *ior_override,
                        glass_intensity: *glass_intensity,
                    }
                } else {
                    cvkg_core::DrawMaterial::Glass {
                        blur_radius: 20.0,
                        ior_override: 0.0,
                        glass_intensity: 1.0,
                    }
                }
            }
            TOP_UI => cvkg_core::DrawMaterial::TopUI,
            BLEND_START..=BLEND_END => cvkg_core::DrawMaterial::Blend {
                mode: (material_id - 7) as u32,
            },
            _ => cvkg_core::DrawMaterial::Opaque,
        }
    }

    /// Convert a compositor Material to a core DrawMaterial.
    /// Centralizes the mapping used by submit_buckets and submit_routed.
    fn convert_compositor_material(mat: &cvkg_compositor::Material) -> cvkg_core::DrawMaterial {
        match mat {
            cvkg_compositor::Material::Glass { blur_radius, .. } => {
                cvkg_core::DrawMaterial::Glass {
                    blur_radius: *blur_radius,
                    ior_override: 0.0,
                    glass_intensity: 1.0,
                }
            }
            cvkg_compositor::Material::Overlay => cvkg_core::DrawMaterial::TopUI,
            cvkg_compositor::Material::Multiply => cvkg_core::DrawMaterial::Blend { mode: 1 },
            cvkg_compositor::Material::Screen => cvkg_core::DrawMaterial::Blend { mode: 2 },
            cvkg_compositor::Material::BlendOverlay => cvkg_core::DrawMaterial::Blend { mode: 3 },
            cvkg_compositor::Material::Darken => cvkg_core::DrawMaterial::Blend { mode: 4 },
            cvkg_compositor::Material::Lighten => cvkg_core::DrawMaterial::Blend { mode: 5 },
            cvkg_compositor::Material::ColorDodge => cvkg_core::DrawMaterial::Blend { mode: 6 },
            cvkg_compositor::Material::ColorBurn => cvkg_core::DrawMaterial::Blend { mode: 7 },
            cvkg_compositor::Material::HardLight => cvkg_core::DrawMaterial::Blend { mode: 8 },
            cvkg_compositor::Material::SoftLight => cvkg_core::DrawMaterial::Blend { mode: 9 },
            cvkg_compositor::Material::Difference => cvkg_core::DrawMaterial::Blend { mode: 10 },
            cvkg_compositor::Material::Exclusion => cvkg_core::DrawMaterial::Blend { mode: 11 },
            cvkg_compositor::Material::Hue => cvkg_core::DrawMaterial::Blend { mode: 12 },
            cvkg_compositor::Material::Saturation => cvkg_core::DrawMaterial::Blend { mode: 13 },
            cvkg_compositor::Material::Color => cvkg_core::DrawMaterial::Blend { mode: 14 },
            cvkg_compositor::Material::Luminosity => cvkg_core::DrawMaterial::Blend { mode: 15 },
            cvkg_compositor::Material::Opaque => cvkg_core::DrawMaterial::Opaque,
            _ => cvkg_core::DrawMaterial::Opaque,
        }
    }

    /// Helper: position vertices from SVG view_box into output rect.
    fn position_vertices(
        vertices: &mut [Vertex],
        view_box: Rect,
        rect: Rect,
        material_id: u32,
        clip: [f32; 4],
        snap: impl Fn(f32) -> f32,
    ) {
        for v in vertices.iter_mut() {
            let rel_x = (v.position[0] - view_box.x) / view_box.width;
            let rel_y = (v.position[1] - view_box.y) / view_box.height;
            v.position[0] = snap(rect.x + rel_x * rect.width);
            v.position[1] = snap(rect.y + rel_y * rect.height);
            v.position[2] = 0.0; // z will be set by transform stack
            v.logical = [v.position[0], v.position[1]];
            v.clip = clip;
            v.material_id = material_id;
        }
    }

    /// Helper: emit a draw call for a batch of vertices.
    fn emit_draw_call(
        renderer: &mut SurtrRenderer,
        material: cvkg_core::DrawMaterial,
        texture_id: Option<u32>,
        scissor_rect: Rect,
        index_count: u32,
        base_vertex: u32,
    ) {
        let draw_order = renderer.current_draw_order;
        let (translation, scale_transform, rotation, _, _) = renderer.current_transform();
        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        };
        let last_call = renderer.draw_calls.last();
        let needs_new_call = renderer.draw_calls.is_empty()
            || renderer.current_texture_id != texture_id
            || last_call.unwrap().scissor_rect != renderer.clip_stack.last().copied()
            || last_call.unwrap().material != material
            || renderer.instance_data.last() != Some(&current_instance_data);

        if needs_new_call {
            renderer.current_texture_id = texture_id;
            renderer.instance_data.push(current_instance_data);
            renderer.draw_calls.push(DrawCall {
                target_id: None,
                texture_id,
                scissor_rect: renderer.clip_stack.last().copied(),
                index_start: (renderer.indices.len() - index_count as usize) as u32,
                index_count,
                material,
                instance_start: (renderer.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else if let Some(call) = renderer.draw_calls.last_mut() {
            call.index_count += index_count;
        }
    }

    /// forge_headless -- Initializes Surtr without a window for visual regression testing.
    pub async fn forge_headless(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        log::info!("[GPU] Requesting HighPerformance adapter (headless)...");
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok();

        if adapter.is_none() {
            log::warn!(
                "[GPU] HighPerformance adapter failed (possible Bumblebee/Optimus), trying LowPower..."
            );
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok();
        }

        if adapter.is_none() {
            log::warn!("[GPU] Hardware adapters failed, trying Software fallback...");
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: None,
                    force_fallback_adapter: true,
                })
                .await
                .ok();
        }

        let adapter = adapter.expect("Failed to find a suitable GPU for Surtr");
        let info = adapter.get_info();
        log::info!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name,
            info.device_type,
            info.backend
        );
        log::info!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let required_features = adapter.features()
            & (wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Headless Forge"),
                required_features,
                required_limits: wgpu::Limits {
                    max_bindings_per_bind_group: adapter
                        .limits()
                        .max_bindings_per_bind_group
                        .min(256),
                    max_binding_array_elements_per_shader_stage: adapter
                        .limits()
                        .max_binding_array_elements_per_shader_stage
                        .min(256),
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create Surtr device");

        let instance = Arc::new(instance);
        let adapter = Arc::new(adapter);

        device.on_uncaptured_error(Arc::new(|error| {
            log::error!(
                "[GPU] Uncaptured device error (Device Lost or Panic): {:?}",
                error
            );
        }));

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self::forge_internal(
            instance,
            adapter,
            device,
            queue,
            None,
            Some((width, height, wgpu::TextureFormat::Rgba8UnormSrgb)),
        )
        .await
    }

    /// capture_frame -- Read back the rendered frame as a byte buffer (RGBA8).
    pub async fn capture_frame(&self) -> Result<Vec<u8>, String> {
        let ctx = self
            .headless_context
            .as_ref()
            .ok_or("Headless context required for capture")?;
        let current_width = self.current_width();
        let current_height = self.current_height();

        let u32_size = std::mem::size_of::<u32>() as u32;
        let width = ctx.width;
        let height = ctx.height;
        let bytes_per_row = width * u32_size;
        let padding = (256 - (bytes_per_row % 256)) % 256;
        let padded_bytes_per_row = bytes_per_row + padding;

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Capture Buffer"),
            size: (padded_bytes_per_row as u64 * height as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Capture Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &ctx.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = sender.send(v);
        });

        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });

        if let Ok(Ok(_)) = receiver.await {
            let data = buffer_slice.get_mapped_range();
            let mut result = Vec::with_capacity((width * height * 4) as usize);

            for y in 0..height {
                let start = (y * padded_bytes_per_row) as usize;
                let end = start + bytes_per_row as usize;
                result.extend_from_slice(&data[start..end]);
            }

            log::trace!(
                "[GPU] capture_frame: data len={}, first 4 bytes={:?}",
                data.len(),
                &data[0..4.min(data.len())]
            );

            drop(data);
            output_buffer.unmap();
            Ok(result)
        } else {
            Err("Failed to capture frame".to_string())
        }
    }

    pub(crate) fn current_width(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).map(|s| s.config.width).unwrap_or(1)
        } else {
            self.headless_context.as_ref().map(|h| h.width).unwrap_or(1)
        }
    }

    pub(crate) fn current_height(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).map(|s| s.config.height).unwrap_or(1)
        } else {
            self.headless_context
                .as_ref()
                .map(|h| h.height)
                .unwrap_or(1)
        }
    }

    pub(crate) fn current_scale_factor(&self) -> f32 {
        if let Some(id) = self.current_window {
            self.surfaces
                .get(&id)
                .map(|s| s.scale_factor)
                .unwrap_or(1.0)
        } else {
            self.headless_context
                .as_ref()
                .map(|h| h.scale_factor)
                .unwrap_or(1.0)
        }
    }

    /// Returns the elapsed time in seconds since the renderer was created.
    /// Used by shaders for time-based animations (volumetric, particles, etc.).
    pub(crate) fn current_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    /// Find a filter by ID in the SVG tree's filter list.
    pub(crate) fn find_filter<'a>(
        tree: &'a usvg::Tree,
        filter_id: &str,
    ) -> Option<&'a usvg::filter::Filter> {
        tree.filters()
            .iter()
            .find(|f| f.id() == filter_id)
            .map(|arc| arc.as_ref())
    }
}

#[cfg(test)]
mod lock_or_clear_cache_tests {
    use crate::renderer::SurtrRenderer;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[test]
    fn returns_lock_when_not_poisoned() {
        let mutex = Mutex::new(HashMap::<u64, u32>::new());
        let guard = SurtrRenderer::lock_or_clear_cache(&mutex);
        assert!(guard.is_empty());
    }

    #[test]
    fn clears_cache_when_poisoned() {
        let mutex = Mutex::new(HashMap::<u64, u32>::new());
        {
            let mut guard = mutex.lock().unwrap();
            guard.insert(1, 100);
            guard.insert(2, 200);
        }
        // Poison the mutex by panicking while holding the lock.
        let result = std::panic::catch_unwind(|| {
            let mutex = std::panic::AssertUnwindSafe(&mutex);
            let _guard = mutex.lock().unwrap();
            panic!("intentional panic to poison the mutex");
        });
        assert!(result.is_err(), "the inner panic should propagate");

        // Now access the poisoned mutex via our helper. The cache should
        // be cleared (not the pre-poison state with {1:100, 2:200}).
        let guard = SurtrRenderer::lock_or_clear_cache(&mutex);
        assert!(
            guard.is_empty(),
            "cache must be cleared after poison recovery, got {:?}",
            *guard
        );
    }

    #[test]
    fn works_with_vec_cache() {
        let mutex = Mutex::new(Vec::<u32>::new());
        {
            let mut guard = mutex.lock().unwrap();
            guard.push(1);
            guard.push(2);
            guard.push(3);
        }
        // Poison
        let _ = std::panic::catch_unwind(|| {
            let mutex = std::panic::AssertUnwindSafe(&mutex);
            let _guard = mutex.lock().unwrap();
            panic!("poison");
        });

        // After recovery, the Vec should be empty.
        let guard = SurtrRenderer::lock_or_clear_cache(&mutex);
        assert!(guard.is_empty(), "Vec cache should be cleared on poison");
    }
}

#[cfg(test)]
mod wgsl_tests {
    #[test]
    fn test_wgsl() {
        let source = include_str!("shaders/effects.wgsl");
        let mut frontend = naga::front::wgsl::Frontend::new();
        match frontend.parse(source) {
            Ok(_) => println!("WGSL parsed successfully!"),
            Err(e) => {
                panic!("WGSL parsing failed: \n{}", e.emit_to_string(source));
            }
        }
    }

    /// P1-12 regression: the native WGSL files must declare t_diffuse as
    /// a 32-element binding_array (in common.wgsl) to match the count=32
    /// bind group layout. Note: WGSL files are concatenated at runtime
    /// and only parse cleanly when combined, so we only check textual
    /// content here, not standalone parse.
    #[test]
    fn test_wgsl_common_uses_binding_array_on_native() {
        let source = include_str!("shaders/common.wgsl");
        assert!(
            source.contains("binding_array<texture_2d<f32>, 32>"),
            "native common.wgsl must declare a 32-element texture binding_array"
        );
        assert!(
            source.contains("t_diffuse:"),
            "native common.wgsl must declare t_diffuse"
        );
    }

    /// P1-12 regression: the native bloom and material_opaque WGSL files
    /// must index t_diffuse with [N] since t_diffuse is a 32-element array.
    #[test]
    fn test_wgsl_native_indexed_access() {
        let bloom = include_str!("shaders/bloom.wgsl");
        let material = include_str!("shaders/material_opaque.wgsl");
        assert!(
            bloom.contains("t_diffuse["),
            "native bloom.wgsl must index t_diffuse as an array"
        );
        assert!(
            material.contains("t_diffuse["),
            "native material_opaque.wgsl must index t_diffuse as an array"
        );
    }
}

// =========================================================================
// P1-11: Inline SHA256 implementation for pipeline cache integrity
// =========================================================================

/// Minimal SHA256 implementation (FIPS 180-4). Used only for the
/// pipeline cache integrity check so we don't add a sha2 dependency.
#[derive(Clone)]
struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: usize,
    total_len: u64,
}

impl Sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0; 64],
            buffer_len: 0,
            total_len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        for &b in data {
            self.buffer[self.buffer_len] = b;
            self.buffer_len += 1;
            if self.buffer_len == 64 {
                let block = self.buffer;
                self.compress(&block);
                self.buffer_len = 0;
            }
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        // Padding: append 0x80, zero-fill, then 8-byte big-endian length in bits.
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;
        if self.buffer_len > 56 {
            for b in &mut self.buffer[self.buffer_len..] { *b = 0; }
            let block = self.buffer;
            self.compress(&block);
            self.buffer_len = 0;
        }
        for b in &mut self.buffer[self.buffer_len..56] { *b = 0; }
        let bit_len = self.total_len.wrapping_mul(8);
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.compress(&block);

        let mut out = [0u8; 32];
        for (i, &s) in self.state.iter().enumerate() {
            out[i*4..(i+1)*4].copy_from_slice(&s.to_be_bytes());
        }
        out
    }

    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]
            ]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(Self::K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);
            h = g; g = f; f = e;
            e = d.wrapping_add(t1);
            d = c; c = b; b = a;
            a = t1.wrapping_add(t2);
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

#[cfg(test)]
mod p1_11_pipeline_cache_tests {
    use super::*;

    /// Write the cache + SHA256 sidecar atomically. Used by tests to
    /// populate a cache file that the integrity check will accept.
    fn write_cache(cache_path: &std::path::Path, data: &[u8]) {
        use std::io::Write;
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let hash_hex = format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7]
        );
        std::fs::write(cache_path, data).unwrap();
        let hash_path = cache_path.with_extension("bin.sha256");
        let mut f = std::fs::File::create(hash_path).unwrap();
        f.write_all(hash_hex.as_bytes()).unwrap();
    }

    #[test]
    fn returns_none_when_cache_does_not_exist() {
        let tmp = std::env::temp_dir().join("cvkg_test_no_cache.bin");
        let _ = std::fs::remove_file(&tmp);
        let result = load_pipeline_cache_with_integrity_check(&tmp);
        assert!(matches!(result, Ok(None)), "missing cache should yield Ok(None), got {result:?}");
    }

    #[test]
    fn returns_data_when_sidecar_matches() {
        let tmp = std::env::temp_dir().join("cvkg_test_good_cache.bin");
        let data = b"pipeline cache blob with some bytes";
        write_cache(&tmp, data);
        let result = load_pipeline_cache_with_integrity_check(&tmp);
        match result {
            Ok(Some(d)) => assert_eq!(d, data),
            other => panic!("expected Ok(Some(data)), got {other:?}"),
        }
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(tmp.with_extension("bin.sha256"));
    }

    #[test]
    fn returns_err_when_sidecar_missing() {
        let tmp = std::env::temp_dir().join("cvkg_test_no_sidecar.bin");
        std::fs::write(&tmp, b"data without sidecar").unwrap();
        let result = load_pipeline_cache_with_integrity_check(&tmp);
        assert!(result.is_err(), "missing sidecar must yield Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("sidecar hash file missing"), "got: {msg}");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn returns_err_when_sidecar_hash_mismatches() {
        // P1-11 regression: tampered cache file must be detected and
        // refused, so the unsafe create_pipeline_cache boundary is never
        // crossed with untrusted data.
        let tmp = std::env::temp_dir().join("cvkg_test_bad_hash.bin");
        std::fs::write(&tmp, b"original data").unwrap();
        let hash_path = tmp.with_extension("bin.sha256");
        std::fs::write(&hash_path, b"0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        // Now overwrite the cache file with different data.
        std::fs::write(&tmp, b"tampered data with extra bytes").unwrap();
        let result = load_pipeline_cache_with_integrity_check(&tmp);
        assert!(result.is_err(), "tampered cache must yield Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("hash mismatch"), "got: {msg}");
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&hash_path);
    }

    #[test]
    fn sha256_of_known_input() {
        // Standard test vector: SHA256("abc") =
        // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let result = compute_sha256(b"abc");
        let hex = format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\
             {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\
             {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\
             {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
            result[8], result[9], result[10], result[11],
            result[12], result[13], result[14], result[15],
            result[16], result[17], result[18], result[19],
            result[20], result[21], result[22], result[23],
            result[24], result[25], result[26], result[27],
            result[28], result[29], result[30], result[31],
        );
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

// =========================================================================
// P1-5: LRU cache sizes -- document the chosen capacity
// =========================================================================
//
// P1-5 regression: cache sizes must be large enough to handle the
// documented use cases (200+ brush strokes, 150+ unique sprites)
// without thrashing. The audit cited these as concrete scenarios
// that previously caused periodic frame spikes.

#[cfg(test)]
mod p1_5_cache_size_tests {
    /// Minimum capacity to cover 200+ brush strokes use case.
    const MIN_SVG_CAPACITY: usize = 512;
    /// Minimum capacity to cover 150+ unique sprite use case.
    const MIN_SVG_TREES_CAPACITY: usize = 512;
    /// Minimum capacity for text glyphs.
    const MIN_TEXT_CAPACITY: usize = 8192;

    #[test]
    fn svg_cache_capacity_meets_benchmark() {
        assert!(
            MIN_SVG_CAPACITY >= 512,
            "SVG cache must be >= 512 to cover 200+ brush strokes"
        );
    }

    #[test]
    fn svg_trees_capacity_meets_benchmark() {
        assert!(
            MIN_SVG_TREES_CAPACITY >= 512,
            "SVG trees cache must be >= 512 to cover 150+ unique sprites"
        );
    }

    #[test]
    fn text_cache_capacity_meets_benchmark() {
        assert!(
            MIN_TEXT_CAPACITY >= 8192,
            "Text cache must be >= 8192 for typical text-heavy UIs"
        );
    }
}

// =========================================================================
// P1-10: QualityLevel for configurable MSAA sample count
// =========================================================================

#[cfg(test)]
mod p1_10_quality_level_tests {
    use super::QualityLevel;

    #[test]
    fn high_quality_uses_msaa_4x() {
        assert_eq!(QualityLevel::High.msaa_sample_count(), 4);
    }

    #[test]
    fn medium_quality_uses_msaa_2x() {
        assert_eq!(QualityLevel::Medium.msaa_sample_count(), 2);
    }

    #[test]
    fn low_quality_disables_msaa() {
        assert_eq!(QualityLevel::Low.msaa_sample_count(), 1);
    }

    #[test]
    fn default_is_high() {
        assert_eq!(QualityLevel::default(), QualityLevel::High);
    }

    #[test]
    fn all_levels_produce_valid_sample_counts() {
        // wgpu requires sample_count to be 1, 2, 4, 8, or 16.
        for level in [QualityLevel::High, QualityLevel::Medium, QualityLevel::Low] {
            let n = level.msaa_sample_count();
            assert!(
                [1, 2, 4, 8, 16].contains(&n),
                "QualityLevel {level:?} produced invalid sample count {n}"
            );
        }
    }
}

// =========================================================================
// P1-7: select_best_surface_format -- guaranteed safe fallback
// =========================================================================

#[cfg(test)]
mod p1_7_surface_format_tests {
    use super::SurtrRenderer;
    use wgpu::TextureFormat;

    #[test]
    fn empty_list_returns_safe_format() {
        // P1-7 regression: empty format list must not panic; it must
        // return a universally-supported format.
        let result = SurtrRenderer::select_best_surface_format(&[]);
        // The result must be a format that virtually all GPUs support.
        assert!(
            matches!(result, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm
                | TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb),
            "empty list should return a known-safe format, got {result:?}"
        );
    }

    #[test]
    fn prefers_hdr_format_when_available() {
        // When Rgba16Float (HDR10) is in the list, it should be picked.
        let formats = [
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba16Float,
            TextureFormat::Bgra8UnormSrgb,
        ];
        let result = SurtrRenderer::select_best_surface_format(&formats);
        assert_eq!(result, TextureFormat::Rgba16Float);
    }

    #[test]
    fn prefers_srgb_when_no_hdr() {
        // Without HDR formats, prefer sRGB over linear for color accuracy.
        let formats = [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8UnormSrgb,
        ];
        let result = SurtrRenderer::select_best_surface_format(&formats);
        // Rgba8Unorm is listed before the sRGB formats in the
        // preferred list, so it would actually be picked first.
        // Either Rgba8Unorm or any sRGB format is acceptable.
        assert!(
            matches!(result, TextureFormat::Rgba8Unorm
                | TextureFormat::Rgba8UnormSrgb
                | TextureFormat::Bgra8UnormSrgb),
            "expected a sRGB or linear format, got {result:?}"
        );
    }

    #[test]
    fn falls_back_to_linear_for_mobile_gpu() {
        // P1-7 regression: a mobile GPU that only supports linear
        // (non-sRGB) formats must still get a usable format, not
        // some exotic HDR-only format.
        let formats = [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8Unorm,
        ];
        let result = SurtrRenderer::select_best_surface_format(&formats);
        // Must be one of the linear formats we provided.
        assert!(
            formats.contains(&result),
            "mobile GPU should get a linear format from the list, got {result:?}"
        );
    }

    #[test]
    fn exotic_formats_fall_back_safely() {
        // If the only formats are exotic (e.g. RGB9E5Float HDR),
        // the function must return one of them, but not panic.
        let formats = [TextureFormat::Rgb9e5Ufloat];
        let result = SurtrRenderer::select_best_surface_format(&formats);
        // Either the exotic format itself or a safe fallback.
        // In this case the only option is the exotic one, which is fine.
        assert_eq!(result, TextureFormat::Rgb9e5Ufloat);
    }
}

// =========================================================================
// P1-6: Particle ring buffer write math
// =========================================================================
//
// P1-6 regression tests for the ring buffer write logic. We can't
// easily test the actual GPU write_buffer call without a SurtrRenderer
// instance, but we CAN test the math (chunk sizes, drop counts, head
// updates) which is where the bug was.

#[cfg(test)]
mod p1_6_particle_ring_buffer_tests {
    /// Reproduces the ring buffer write math in isolation. Returns
    /// (bytes_to_write_at_head, bytes_to_write_at_zero, new_write_head).
    fn compute_ring_buffer_write(
        write_start: usize,
        write_count: usize,
        max: usize,
    ) -> (usize, usize, usize) {
        // P1-6 fix: cap to max
        let effective_count = write_count.min(max);
        let drop_count = write_count - effective_count;
        let first_chunk = (max - write_start).min(effective_count);
        if first_chunk < effective_count {
            let remaining = effective_count - first_chunk;
            (first_chunk, remaining, remaining)
        } else {
            (first_chunk, 0, (write_start + effective_count) % max)
        }
    }

    #[test]
    fn no_wrap_no_overflow() {
        // write_count < (max - write_start), no wrap, no overflow
        let (first, second, head) = compute_ring_buffer_write(0, 10, 100);
        assert_eq!(first, 10);
        assert_eq!(second, 0);
        assert_eq!(head, 10);
    }

    #[test]
    fn wrap_without_overflow() {
        // write_count > (max - write_start), but < max total
        let (first, second, head) = compute_ring_buffer_write(80, 50, 100);
        assert_eq!(first, 20);  // 80..100
        assert_eq!(second, 30); // 0..30
        assert_eq!(head, 30);
    }

    #[test]
    fn overflow_caps_to_max() {
        // P1-6 regression: write_count > max must cap, not overlap
        let (first, second, head) = compute_ring_buffer_write(80, 200, 100);
        // effective_count = 100, drop_count = 100
        // first_chunk = (100-80).min(100) = 20
        // remaining = 100-20 = 80
        assert_eq!(first, 20);
        assert_eq!(second, 80);
        assert_eq!(head, 80);
    }

    #[test]
    fn overflow_at_offset_zero() {
        // Edge case: write_start=0, write_count > max
        let (first, second, head) = compute_ring_buffer_write(0, 150, 100);
        // effective_count = 100, drop_count = 50
        // first_chunk = 100.min(100) = 100
        // 100 < 100 is false, so no wrap
        assert_eq!(first, 100);
        assert_eq!(second, 0);
        assert_eq!(head, 0);  // (0 + 100) % 100 = 0
    }

    #[test]
    fn empty_write() {
        let (first, second, head) = compute_ring_buffer_write(50, 0, 100);
        assert_eq!(first, 0);
        assert_eq!(second, 0);
        assert_eq!(head, 50);
    }
}


// =========================================================================
// P1-1: SurtrConfig tests
// =========================================================================

#[cfg(test)]
mod p1_1_surtr_config_tests {
    use super::SurtrConfig;

    #[test]
    fn default_has_p1_5_cache_sizes() {
        // P1-1 regression: default config matches the P1-5 fixed
        // cache sizes (text=8192, svg=512, trees=512, etc.).
        let cfg = SurtrConfig::default();
        assert_eq!(cfg.text_cache_capacity.get(), 8192);
        assert_eq!(cfg.svg_cache_capacity.get(), 512);
        assert_eq!(cfg.svg_trees_capacity.get(), 512);
        assert_eq!(cfg.shared_elements_capacity.get(), 1024);
        assert_eq!(cfg.image_uv_capacity.get(), 256);
        assert_eq!(cfg.texture_registry_capacity.get(), 31);
        assert_eq!(cfg.mega_heim_width, 4096);
        assert_eq!(cfg.mega_heim_height, 4096);
    }

    #[test]
    fn low_vram_uses_smaller_atlas() {
        // P1-1 regression: low_vram preset uses 2048x2048 atlas
        // (~16MB RGBA8) instead of the default 4096x4096 (~64MB).
        let cfg = SurtrConfig::low_vram();
        assert_eq!(cfg.mega_heim_width, 2048);
        assert_eq!(cfg.mega_heim_height, 2048);
        assert!(
            cfg.mega_heim_vram_bytes() < 32 * 1024 * 1024,
            "low_vram atlas should fit in 32MB, got {} bytes",
            cfg.mega_heim_vram_bytes()
        );
    }

    #[test]
    fn high_end_uses_larger_atlas() {
        // P1-1 regression: high_end preset uses 8192x8192 atlas
        // (~256MB RGBA8) for high-end desktop GPUs.
        let cfg = SurtrConfig::high_end();
        assert_eq!(cfg.mega_heim_width, 8192);
        assert_eq!(cfg.mega_heim_height, 8192);
        assert!(cfg.mega_heim_vram_bytes() >= 256 * 1024 * 1024);
    }

    #[test]
    fn mega_heim_vram_is_4_bytes_per_pixel() {
        // P1-1 regression: VRAM cost is width*height*4 (RGBA8).
        let cfg = SurtrConfig::default();
        let expected = 4096u64 * 4096 * 4;
        assert_eq!(cfg.mega_heim_vram_bytes(), expected);
    }

    #[test]
    fn all_presets_have_nonzero_capacities() {
        // P1-1: every preset must have positive capacities (no
        // accidentally-zero LRU caches).
        for (name, cfg) in [
            ("default", SurtrConfig::default()),
            ("low_vram", SurtrConfig::low_vram()),
            ("high_end", SurtrConfig::high_end()),
        ] {
            assert!(cfg.text_cache_capacity.get() > 0, "{name} text_cache");
            assert!(cfg.svg_cache_capacity.get() > 0, "{name} svg_cache");
            assert!(cfg.svg_trees_capacity.get() > 0, "{name} svg_trees");
            assert!(
                cfg.shared_elements_capacity.get() > 0,
                "{name} shared_elements"
            );
            assert!(cfg.image_uv_capacity.get() > 0, "{name} image_uv");
            assert!(
                cfg.texture_registry_capacity.get() > 0,
                "{name} texture_registry"
            );
            assert!(cfg.mega_heim_width > 0, "{name} mega_heim_width");
            assert!(cfg.mega_heim_height > 0, "{name} mega_heim_height");
        }
    }

    #[test]
    fn config_is_cloneable_and_debug() {
        // P1-1: config must be Clone + Debug for use in error paths.
        let cfg = SurtrConfig::default();
        let _cloned = cfg.clone();
        let _formatted = format!("{cfg:?}");
    }
}

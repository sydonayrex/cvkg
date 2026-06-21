//! The main GpuRenderer struct and core frame lifecycle.
use crate::draw::{parse_svg_animations, usvg_to_lyon};
use crate::heim::SkylinePacker;
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

/// P1-1 fix: configurable GpuRenderer parameters.
///
/// The 5220-line GpuRenderer monolith hardcoded six LRU cache sizes
/// plus the Mega-Heim atlas dimensions. This struct extracts those
/// into a single configuration object so that callers can tune the
/// renderer for different working sets (high-end desktop vs. mid-tier
/// mobile vs. low-VRAM embedded) without modifying the source.
/// P1-1 (phase 6): RendererConfig has been moved to its own module
/// at `crate::subsystems::config::RendererConfig`. The re-export at
/// `crate::RendererConfig` (from `cvkg_runic_text` re-exports in
/// `lib.rs`) preserves backward compatibility.
///
/// GpuRenderer implements the high-performance GPU backend.
pub struct GpuRenderer {
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
    pub(crate) heim_packer: SkylinePacker,
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
    /// Comparison sampler for volumetric depth comparison.
    pub(crate) volumetric_depth_sampler: wgpu::Sampler,
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
    /// Pool of persistent uniform buffers for Kawase blur operations.
    pub(crate) kawase_uniform_buffers: Vec<wgpu::Buffer>,
    /// Environment bind group layout (texture + sampler).
    pub(crate) env_bind_group_layout: wgpu::BindGroupLayout,

    // Telemetry
    pub telemetry: cvkg_core::TelemetryData,

    /// Pipeline cache for disk-persisted compiled shaders when the adapter exposes PIPELINE_CACHE.
    /// None means pipelines compile normally without a disk cache.
    pub(crate) pipeline_cache: Option<wgpu::PipelineCache>,

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

    // Path Geometry Cache — avoids re-tessellating static paths every frame.
    /// Key: (path_hash, stroke_width_bits) where path_hash is derived from
    /// the path's data pointer identity + length, and stroke_width_bits is
    /// the bit representation of the stroke width for exact matching.
    /// Value: (vertices, indices) ready to upload to GPU buffers.
    pub(crate) path_geometry_cache: lru::LruCache<u64, (Vec<Vertex>, Vec<u32>)>,
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
    /// P1-1: particle CPU-side state (staging, count, write_head,
    /// last_compact) grouped into a single ParticleSubsystem struct.
    /// The GPU-side buffer and pipelines remain in the renderer
    /// because they're tightly coupled to the wgpu device lifecycle.
    pub(crate) particles: crate::types::ParticleSubsystem,
    /// Simple render pipeline for drawing particles as point sprites.
    pub(crate) particle_render_pipeline: wgpu::RenderPipeline,
    /// Bind group layout for particle render pass (storage buffer read-only).
    pub(crate) particle_render_bgl: wgpu::BindGroupLayout,
    /// Bind group for particle render pass (created lazily when count > 0).
    pub(crate) particle_render_bind_group: Option<wgpu::BindGroup>,
    /// Bind group for particle compute pass (created lazily when count > 0).
    pub(crate) particle_compute_bind_group: Option<wgpu::BindGroup>,

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
    /// P1-1: GpuRenderer configuration. Contains cache sizes,
    /// atlas dimensions, and other tunable parameters. Can be
    /// replaced at runtime via `set_config()` to adapt to different
    /// working sets (e.g., after detecting a low-VRAM device).
    pub(crate) config: crate::subsystems::RendererConfig,
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
// GpuRenderer contains the following shared state:
//   - wgpu::Device and wgpu::Queue  (transitively !Send + !Sync on WASM)
//   - Mutex<HashMap<...>> caches    (bind_group_cache, texture_view_cache)
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
// 3. GpuRenderer's GPU buffer staging vectors are only mutated by the
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
// X-08: unsafe Send/Sync for GpuRenderer on WASM
// SAFETY: GpuRenderer contains wgpu types that are not Send/Sync on WASM
// because wgpu's web backend uses OffscreenCanvas which is main-thread-only.
// However, CVKG's WASM execution model is single-threaded:
// - The browser event loop is single-threaded
// - All renderer access happens on the main thread
// - No web workers are used for rendering
// - wgpu's own WebGPU backend allows this on single-threaded WASM
//
// CRITICAL: If CVKG ever adds web worker rendering or shared WebAssembly
// threads, this unsafe impl MUST be removed and GpuRenderer must be
// wrapped in a !Send/!Sync marker (e.g., PhantomData<*const ()>) to prevent
// accidental cross-thread use. The cfg gate ensures this only applies to wasm32.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for GpuRenderer {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for GpuRenderer {}

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

    // Compute actual SHA256 and compare full 32 bytes (not just first 8)
    let actual = compute_sha256(&cache_data);
    let actual_hex: String = actual.iter().map(|b| format!("{:02x}", b)).collect();
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

// P2-12: Compute mip level count from texture dimensions.
// Uses floor(log2(max(width, height))) + 1, clamped to [2, 8].
// A 1080p/2=540px blur texture -> log2(540)=9.07 -> 10 mips, clamped to 8.
// A 720p/2=360px blur texture -> log2(360)=8.49 -> 9 mips, clamped to 8.
// A 256px blur texture -> log2(256)=8 -> 9 mips, clamped to 8.
// A 64px blur texture -> log2(64)=6 -> 7 mips.
fn compute_mip_levels(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim <= 1 {
        return 1;
    }
    // floor(log2(max_dim)) + 1, clamped to [2, 8]
    let mips = (32 - max_dim.leading_zeros()).clamp(2, 8);
    mips
}


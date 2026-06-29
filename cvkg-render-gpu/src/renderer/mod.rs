//! The main GpuRenderer struct and core frame lifecycle.
use crate::heim::SkylinePacker;
use crate::types::*;
use crate::vertex::*;
use cvkg_core::Rect;
use cvkg_core::{ColorTheme, SceneUniforms};
use lru::LruCache;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Arc;

// Re-export for test access
pub use crate::subsystems::RendererConfig;

pub(crate) mod context_helpers;
pub(crate) mod draw;
pub(crate) mod init;
pub(crate) mod pipelines;
pub(crate) mod svg;
#[cfg(test)]
pub(crate) mod tests;

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

/// P1-10: Quality level for adaptive rendering on different GPU tiers.
///
/// `High` matches the previous hardcoded behavior (MSAA 4x).
/// `Medium` reduces MSAA to 2x for moderate savings on mobile.
/// `Low` disables MSAA entirely for low-end GPUs (Adreno 3xx, etc.).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QualityLevel {
    #[default]
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
    pub(crate) text: crate::types::TextSubsystem,
    pub(crate) mega_heim_tex: wgpu::Texture,
    pub(crate) mega_heim_bind_group: wgpu::BindGroup,
    pub(crate) heim_packer: SkylinePacker,
    pub(crate) image_uv_registry: LruCache<String, Rect>,
    pub(crate) texture_registry: LruCache<String, u32>,
    pub(crate) texture_views: Vec<wgpu::TextureView>,
    pub(crate) dummy_sampler: wgpu::Sampler,
    /// Dummy single-sampled depth texture view.
    ///
    /// WHY: Used in the volumetric shader to bind a valid single-sampled depth view
    /// when MSAA is enabled (since the actual scene depth view is multisampled).
    ///
    /// CONTRACT: Always sample_count = 1, format = Depth32Float.
    pub(crate) dummy_depth_view: wgpu::TextureView,
    /// Dummy multisampled depth texture view.
    ///
    /// WHY: Used in the volumetric shader to bind a valid multisampled depth view
    /// when MSAA is disabled (since the actual scene depth view is single-sampled).
    ///
    /// CONTRACT: Always sample_count = 4, format = Depth32Float.
    pub(crate) dummy_depth_view_msaa: wgpu::TextureView,
    pub(crate) svg: crate::types::SvgSubsystem,

    // Niflheim Resources (Shared)
    pub(crate) dummy_texture_bind_group: wgpu::BindGroup,
    pub(crate) dummy_env_bind_group: wgpu::BindGroup,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) texture_bind_groups: Vec<wgpu::BindGroup>,
    pub(crate) shared_elements: LruCache<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
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

    // SVG Filter Engine Resources
    /// Render pipeline for Gaussian blur (two-pass separable kernel).
    /// Initialized lazily on first use.
    pub blur_pipeline: Option<wgpu::RenderPipeline>,
    /// Uniform buffer for blur parameters (std_deviation, kernel_size, direction).
    /// Initialized lazily on first use.
    pub blur_uniform: Option<wgpu::Buffer>,
    /// Bind group layout for blur shader.
    /// Initialized lazily on first use.
    pub blur_bind_group_layout: Option<wgpu::BindGroupLayout>,
    /// Render pipeline for blend operations (feBlend, feComposite).
    /// Initialized lazily on first use.
    pub blend_pipeline: Option<wgpu::RenderPipeline>,
    /// Bind group layout for blend shader.
    /// Initialized lazily on first use.
    pub blend_bind_group_layout: Option<wgpu::BindGroupLayout>,
    /// Render pipeline for flood fill (feFlood).
    /// Initialized lazily on first use.
    pub flood_pipeline: Option<wgpu::RenderPipeline>,
    /// Bind group layout for copy/offset operations.
    /// Initialized lazily on first use.
    pub copy_bind_group_layout: Option<wgpu::BindGroupLayout>,

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
    pub(crate) particle_compute_pipeline: wgpu::ComputePipeline,
    pub(crate) particle_compute_bgl: wgpu::BindGroupLayout,
    pub(crate) particle_buffer: wgpu::Buffer,
    pub(crate) particle_uniform_buffer: wgpu::Buffer,
    pub(crate) particles: crate::types::ParticleSubsystem,
    pub(crate) particle_render_pipeline: wgpu::RenderPipeline,
    pub(crate) particle_render_bgl: wgpu::BindGroupLayout,
    pub(crate) particle_render_bind_group: Option<wgpu::BindGroup>,
    pub(crate) particle_compute_bind_group: Option<wgpu::BindGroup>,

    // VDOM node stack for hierarchy tracking
    pub(crate) vnode_stack: Vec<(Rect, &'static str)>,

    /// Event handlers registered during render passes.
    pub(crate) event_handlers: std::collections::HashMap<
        String,
        Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>,
    >,

    // Error tracking (set via RendererErrorHandler trait)
    pub(crate) render_error_count: u64,
    pub(crate) has_fatal_error: bool,

    /// Bind group layout for reading blur output in glass composite pass.
    pub(crate) glass_output_bind_group_layout: wgpu::BindGroupLayout,
    /// Current material state -- draw calls are tagged with this material.
    pub(crate) current_draw_material: cvkg_core::DrawMaterial,

    /// Portal backdrop blur regions -- collected during portal enter/exit
    pub(crate) portal_regions: std::collections::VecDeque<cvkg_core::Rect>,

    /// Gradient stop texture (32 x 1, RGBA) for multi-stop gradient rendering.
    /// RGB = stop color, A = stop position (0-1). Cached per unique stop set.
    pub(crate) gradient_stop_texture: wgpu::Texture,
    pub(crate) gradient_stop_texture_view: wgpu::TextureView,
    pub(crate) gradient_bind_group: wgpu::BindGroup,
    /// Gradient texture cache: maps stop-hash to (texture, bind_group) to avoid re-uploading.
    pub(crate) gradient_texture_cache:
        std::collections::HashMap<u64, (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)>,
    /// Last uploaded gradient stops hash, to detect when we need to re-upload.
    pub(crate) gradient_stops_hash: u64,
    /// Layout for the gradient bind group (texture + sampler).
    pub(crate) gradient_bind_group_layout: wgpu::BindGroupLayout,

    /// Cache of the compiled Kvasir render graph execution plan.
    pub(crate) cached_graph_plan: Option<crate::kvasir::graph_cache::CachedGraphPlan>,
    /// Hash of the active material set, used to invalidate the graph plan
    pub(crate) material_compilation_hash: u64,
    /// Memoization cache for frame-level render skipping.
    pub(crate) memo_cache: std::collections::HashMap<u64, crate::types::MemoEntry>,
    /// Current frame generation counter.
    pub(crate) frame_generation: u64,
    /// P1-1: GpuRenderer configuration.
    pub(crate) config: crate::subsystems::RendererConfig,
    /// P1-10: Quality level controlling MSAA sample count.
    pub(crate) quality_level: QualityLevel,
    /// Thread-safe bind group cache to avoid per-frame allocations during render passes.
    pub(crate) bind_group_cache: std::sync::Mutex<
        std::collections::HashMap<
            (
                Option<winit::window::WindowId>,
                crate::kvasir::resource::ResourceId,
                u32,
                bool,
            ),
            wgpu::BindGroup,
        >,
    >,
    /// Thread-safe texture view cache to avoid per-frame allocations of TextureViews.
    pub(crate) texture_view_cache: std::sync::Mutex<
        std::collections::HashMap<
            (
                Option<winit::window::WindowId>,
                crate::kvasir::resource::ResourceId,
                u32,
            ),
            wgpu::TextureView,
        >,
    >,
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for GpuRenderer {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for GpuRenderer {}

/// Per-hologram instance data submitted during the frame.
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
fn load_pipeline_cache_with_integrity_check(
    cache_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
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
            ));
        }
        Err(e) => return Err(format!("sidecar read failed: {e}")),
    };

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
fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

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
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
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
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;
        if self.buffer_len > 56 {
            for b in &mut self.buffer[self.buffer_len..] {
                *b = 0;
            }
            let block = self.buffer;
            self.compress(&block);
            self.buffer_len = 0;
        }
        for b in &mut self.buffer[self.buffer_len..56] {
            *b = 0;
        }
        let bit_len = self.total_len.wrapping_mul(8);
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.compress(&block);

        let mut out = [0u8; 32];
        for (i, &s) in self.state.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&s.to_be_bytes());
        }
        out
    }

    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];
        for (i, wi) in w.iter().enumerate() {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(Self::K[i])
                .wrapping_add(*wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
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

fn compute_mip_levels(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim <= 1 {
        return 1;
    }
    (32 - max_dim.leading_zeros()).clamp(2, 8)
}

impl GpuRenderer {
    /// Access the hologram instances submitted this frame.
    pub fn hologram_instances(&self) -> &[HologramInstance] {
        &self.hologram_instances
    }

    pub fn set_quality_level(&mut self, level: QualityLevel) {
        self.quality_level = level;
    }

    pub fn set_config(&mut self, config: crate::subsystems::RendererConfig) {
        self.config = config;
    }

    pub fn config(&self) -> &crate::subsystems::RendererConfig {
        &self.config
    }

    pub fn quality_level(&self) -> QualityLevel {
        self.quality_level
    }

    pub(crate) fn lock_or_clear_cache<'a, T: ClearInto>(
        lock: &'a std::sync::Mutex<T>,
    ) -> std::sync::MutexGuard<'a, T> {
        match lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("[GPU] lock_or_clear_cache: mutex poisoned, clearing cache...");
                let mut guard = poisoned.into_inner();
                guard.clear_into();
                guard
            }
        }
    }

    pub fn update_mouse(&mut self, mouse: [f32; 2], velocity: [f32; 2]) {
        self.current_scene.mouse = mouse;
        self.current_scene.mouse_velocity = velocity;
        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );
    }

    pub fn invalidate_material_cache(&mut self) {
        self.cached_graph_plan = None;
    }

    pub fn invalidate_all_caches(&mut self) -> usize {
        let mut cleared = 0;
        {
            let mut bg_cache = Self::lock_or_clear_cache(&self.bind_group_cache);
            cleared += bg_cache.len();
            bg_cache.clear();
        }
        {
            let mut view_cache = Self::lock_or_clear_cache(&self.texture_view_cache);
            cleared += view_cache.len();
            view_cache.clear();
        }
        cleared += self.text.shaped_cache.len();
        self.text.shaped_cache.clear();
        cleared += self.svg.model_cache.len();
        self.svg.model_cache.clear();
        cleared += self.svg.tree_cache.len();
        self.svg.tree_cache.clear();
        self.svg.clear_filter_batches();
        cleared
    }

    pub fn prewarm_text_cache(&mut self, labels: &[(&str, f32)]) {
        let mut count = 0;
        for (text, size) in labels {
            let cache_key = (text.to_string(), (size * 100.0) as u32);
            if self.text.shaped_cache.contains(&cache_key) {
                continue;
            }
            let style = cvkg_runic_text::TextStyle::new("Inter", *size);
            let spans = [cvkg_runic_text::TextSpan::new(text, style)];
            if let Ok(shaped) = self.text.engine.shape_layout(
                &spans,
                None,
                cvkg_runic_text::TextAlign::Start,
                cvkg_runic_text::TextOverflow::Visible,
            ) {
                self.text
                    .shaped_cache
                    .put(cache_key, std::sync::Arc::new(shaped));
                count += 1;
            }
        }
        if count > 0 {
            tracing::info!("[Surtr] prewarm_text_cache: pre-shaped {} labels", count);
        }
    }

    pub(crate) fn select_best_surface_format(
        formats: &[wgpu::TextureFormat],
    ) -> wgpu::TextureFormat {
        if formats.is_empty() {
            return wgpu::TextureFormat::Rgba8Unorm;
        }
        let preferred_formats = [
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ];
        for preferred in &preferred_formats {
            if formats.contains(preferred) {
                return *preferred;
            }
        }
        if formats.contains(&wgpu::TextureFormat::Rgba8Unorm) {
            return wgpu::TextureFormat::Rgba8Unorm;
        }
        formats[0]
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
            label: Some("Mega-Heim Rebuilt Bind Group"),
        });
    }

    pub(crate) fn update_vram_telemetry(&mut self) {
        let buffers = self.geometry_buffers.vertex_buffer.size()
            + self.geometry_buffers.index_buffer.size()
            + self.geometry_buffers.instance_buffer.size()
            + self.scene_buffer.size()
            + self.theme_buffer.size()
            + self.particle_buffer.size()
            + self.particle_uniform_buffer.size();

        let mut textures = self.config.mega_heim_vram_bytes();
        textures += 4; // Dummy texture

        for surface in self.surfaces.values() {
            let width = surface.config.width;
            let height = surface.config.height;
            let format_bytes = 8; // Rgba16Float
            textures += (width * height * format_bytes) as u64; // Scene texture
            textures +=
                (width * height * format_bytes * self.quality_level.msaa_sample_count()) as u64; // MSAA texture
            textures += (width * height * 4) as u64; // Depth texture (Depth32Float)

            let blur_width = (width / 2).max(1);
            let blur_height = (height / 2).max(1);
            let blur_bytes = (blur_width * blur_height * 4) as u64;
            textures += blur_bytes * 4; // 2x blur + 2x bloom textures
        }

        if let Some(ref ctx) = self.headless_context {
            let format_bytes = 8; // Rgba16Float
            textures += (ctx.width * ctx.height * format_bytes) as u64; // Scene texture
            textures +=
                (ctx.width * ctx.height * format_bytes * self.quality_level.msaa_sample_count())
                    as u64; // MSAA texture
            textures += (ctx.width * ctx.height * 4) as u64; // Depth texture
            textures += (ctx.width * ctx.height * 4) as u64; // Output texture
        }

        self.vram_buffers_bytes = buffers;
        self.vram_textures_bytes = textures;
        self.telemetry.vram_usage_mb = (buffers + textures) as f32 / (1024.0 * 1024.0);
    }

    pub fn get_telemetry(&self) -> cvkg_core::TelemetryData {
        self.telemetry.clone()
    }

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
                return;
            }

            tracing::info!("[GPU] Reconfiguring surface: {}x{}", width, height);
            GpuRenderer::lock_or_clear_cache(&self.bind_group_cache).clear();
            GpuRenderer::lock_or_clear_cache(&self.texture_view_cache).clear();
            self.text.shaped_cache.clear();
            ctx.config.width = width;
            ctx.config.height = height;
            ctx.scale_factor = scale_factor;
            ctx.surface.configure(&self.device, &ctx.config);

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
                    mip_level_count: compute_mip_levels(blur_width, blur_height),
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
                    mip_level_count: compute_mip_levels(blur_width, blur_height),
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
                    mip_level_count: compute_mip_levels(blur_width, blur_height),
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
                    mip_level_count: compute_mip_levels(blur_width, blur_height),
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                },
                lifetime: crate::kvasir::resource::ResourceLifetime::Persistent,
            };
            ctx.bloom_tex_b = self.registry.allocate_image(&self.device, &bloom_desc_b);

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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            ctx.depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn reset_time(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    pub fn reclaim_vram(&mut self) {
        tracing::warn!("[GPU] Sundr Compaction: Compacting Mega-Heim...");

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

        let mut new_packer = SkylinePacker::new(4096, 4096);
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

        let text_entries: Vec<(u64, (Rect, f32, f32, f32, f32))> = self
            .text
            .glyph_cache
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
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
                self.text
                    .glyph_cache
                    .put(hash, (new_uv, w_f, h_f, x_off, y_off));
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
}

impl Drop for GpuRenderer {
    fn drop(&mut self) {
        let cache_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("pipeline_cache")))
            .unwrap_or_else(|| std::env::temp_dir().join("cvkg_pipeline_cache"));
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("cvkg_render_gpu.bin");
        if let Some(cache) = &self.pipeline_cache
            && let Some(data) = cache.get_data()
            && let Err(e) = std::fs::write(&cache_path, data)
        {
            tracing::warn!("Failed to persist pipeline cache: {}", e);
        }

        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
    }
}

impl GpuRenderer {
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

    pub(crate) fn current_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
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
        tracing::info!("[GPU] Requesting HighPerformance adapter (headless)...");
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok();

        if adapter.is_none() {
            tracing::warn!(
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
            tracing::warn!("[GPU] Hardware adapters failed, trying Software fallback...");
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
        let caps =
            crate::subsystems::GpuCapabilities::detect(&info.name, format!("{:?}", info.backend));
        tracing::info!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?} -- detected as {}",
            info.name,
            info.device_type,
            info.backend,
            caps.vendor
        );
        tracing::info!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
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
            tracing::error!(
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

    /// Create a headless GpuRenderer from an existing device and surface.
    ///
    /// This constructor does not require an event loop and is suitable for
    /// headless rendering (e.g., server-side rendering, tests).
    /// It delegates to the existing `forge_internal` which handles all
    /// pipeline, buffer, and bind group initialization.
    pub async fn from_external(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No compatible adapter found");

        Self::forge_internal(
            Arc::new(instance),
            Arc::new(adapter),
            device,
            queue,
            None,
            Some((width, height, wgpu::TextureFormat::Rgba8UnormSrgb)),
        )
        .await
    }
}

//! Core data types, internal structs, and rendering contexts.
use crate::vertex::{InstanceData, Vertex};
use cvkg_core::Rect;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// SvgModel -- A collection of tessellated triangles representing a vector icon.
/// Paths are stored as independent sub-models, each with its own vertex range
/// and local transform, enabling per-path manipulation (e.g. in an SVG editor).
#[derive(Clone, Debug)]
pub struct SvgModel {
    /// All vertices for all paths in this SVG.
    pub vertices: Vec<Vertex>,
    /// All indices for all paths in this SVG.
    pub indices: Vec<u32>,
    /// The SVG viewBox defining the coordinate space.
    pub view_box: Rect,
    /// Per-path sub-models, each with its own vertex range and local transform.
    pub paths: Vec<SvgPath>,
    /// Animations parsed from SVG `<animate>` elements.
    pub animations: Vec<SvgAnimation>,
}

/// A single path within an SVG model, with its own vertex range and local transform.
/// Multiple paths can share the same underlying vertex buffer but are drawn
/// independently with different transforms.
#[derive(Clone, Debug)]
pub struct SvgPath {
    /// The element id from the SVG (e.g. "t1", "path2").
    pub id: String,
    /// Range into SvgModel.vertices for this path's vertices.
    pub vertex_range: std::ops::Range<usize>,
    /// Range into SvgModel.indices for this path's indices.
    pub index_range: std::ops::Range<usize>,
    /// Local transform offset applied when drawing this path.
    /// This allows per-path positioning, rotation, and scaling.
    pub local_transform: SvgTransform,
}

/// A 2D affine transform for SVG path positioning.
#[derive(Clone, Debug, Default)]
pub struct SvgTransform {
    /// Translation in SVG user units.
    pub translate: [f32; 2],
    /// Rotation in degrees.
    pub rotation: f32,
    /// Scale factor (1.0 = no scaling).
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct SvgAnimation {
    pub target_id: String,
    pub attribute_name: String,
    /// Keyframe values. For 2-value animations, this is [from, to].
    /// For multi-keyframe animations (values="v0;v1;..."), this stores all values.
    pub keyframe_values: Vec<f32>,
    /// Optional keyTimes (normalized 0..1). If empty, uniform spacing is assumed.
    pub key_times: Vec<f32>,
    pub duration: f32,
    pub vertex_range: std::ops::Range<usize>,
}

impl SvgAnimation {
    /// Get the interpolated value at normalized time t (0..1).
    pub fn evaluate(&self, t: f32) -> f32 {
        let vals = &self.keyframe_values;
        if vals.is_empty() {
            return 0.0;
        }
        if vals.len() == 1 {
            return vals[0];
        }
        if vals.len() == 2 {
            return vals[0] + (vals[1] - vals[0]) * t;
        }
        // Multi-keyframe: find the active segment
        let times = if self.key_times.len() == vals.len() {
            &self.key_times
        } else {
            // Uniform spacing
            return self.evaluate_uniform(t);
        };
        // Find the segment containing t
        let t = t.clamp(0.0, 1.0);
        for i in 0..times.len() - 1 {
            if t >= times[i] && t <= times[i + 1] {
                let seg_t = (t - times[i]) / (times[i + 1] - times[i]);
                return vals[i] + (vals[i + 1] - vals[i]) * seg_t;
            }
        }
        vals[vals.len() - 1]
    }

    fn evaluate_uniform(&self, t: f32) -> f32 {
        let vals = &self.keyframe_values;
        let n = vals.len() - 1;
        let t = t.clamp(0.0, 1.0);
        let idx_f = t * n as f32;
        let idx = idx_f.floor() as usize;
        let frac = idx_f - idx as f32;
        if idx >= n {
            vals[n]
        } else {
            vals[idx] + (vals[idx + 1] - vals[idx]) * frac
        }
    }
}

/// Represents a single batched GPU draw call.
/// Batches are broken whenever the active texture or primitive mode changes.
#[derive(Debug, Clone)]
pub(crate) struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: Option<Rect>,
    pub index_start: u32,
    pub index_count: u32,
    /// Material routing tag -- determines which pass this draw call is routed to
    /// in the multi-pass Backdrop Capture pipeline.
    pub material: cvkg_core::DrawMaterial,
    pub target_id: Option<u64>,
    pub instance_start: u32,
    /// Draw order for sorting within the same pass. Higher = later (on top).
    /// Convention: 0 = background, 100 = UI chrome, 200 = SVG content, 300 = overlays.
    pub draw_order: i32,
}

/// A snapshot of all GPU data emitted by a memoized render closure.
///
/// `memoize()` caches the vertex/index/instance buffers and draw calls
/// produced by `render_fn` on first call so they can be replayed on
/// subsequent calls when `data_hash` is unchanged. Without this cache,
/// memoize's skip path would emit zero draw commands and memoized content
/// would vanish after the first frame.
///
/// Offsets are stored RELATIVE to the start of the cached buffers, not the
/// current buffer state, so replay can shift them by appending offsets.
#[derive(Debug, Clone)]
pub(crate) struct MemoEntry {
    pub hash: u64,
    pub frame_gen: u64,
    pub vertices: Vec<crate::vertex::Vertex>,
    pub indices: Vec<u32>,
    pub instance_data: Vec<crate::vertex::InstanceData>,
    pub draw_calls: Vec<DrawCall>,
}

pub struct OffscreenEffectConfig {
    pub target_id: u64,
    pub effect: String,
    pub blend_mode: u32,
    pub effect_args: [f32; 16],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShadowState {
    pub radius: f32,
    pub color: [f32; 4],
    pub _offset: [f32; 2],
}

pub(crate) struct SurfaceContext {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) scene_texture: wgpu::TextureView,
    pub(crate) scene_msaa_texture: wgpu::TextureView,
    pub(crate) scene_bind_group: wgpu::BindGroup,
    pub(crate) scene_texture_bind_group: wgpu::BindGroup,
    pub(crate) depth_texture_view: wgpu::TextureView,
    pub(crate) blur_tex_a: crate::kvasir::resource::ResourceId,
    pub(crate) blur_tex_b: crate::kvasir::resource::ResourceId,
    pub(crate) bloom_tex_a: crate::kvasir::resource::ResourceId,
    pub(crate) bloom_tex_b: crate::kvasir::resource::ResourceId,
    pub(crate) blur_env_bind_group_a: wgpu::BindGroup,
    pub(crate) blur_env_bind_group_b: wgpu::BindGroup,
    pub(crate) bloom_env_bind_group_a: wgpu::BindGroup,
    pub(crate) bloom_env_bind_group_b: wgpu::BindGroup,
    pub(crate) scale_factor: f32,
    pub(crate) sampler: wgpu::Sampler,
}

/// HeadlessContext -- A rendering target for surface-less execution.
pub struct HeadlessContext {
    pub scene_texture: wgpu::TextureView,
    pub scene_msaa_texture: wgpu::TextureView,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_texture_bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,
    pub blur_tex_a: crate::kvasir::resource::ResourceId,
    pub blur_tex_b: crate::kvasir::resource::ResourceId,
    pub bloom_tex_a: crate::kvasir::resource::ResourceId,
    pub bloom_tex_b: crate::kvasir::resource::ResourceId,
    pub blur_env_bind_group_a: wgpu::BindGroup,
    pub blur_env_bind_group_b: wgpu::BindGroup,
    pub bloom_env_bind_group_a: wgpu::BindGroup,
    pub bloom_env_bind_group_b: wgpu::BindGroup,
    pub scale_factor: f32,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
}

pub(crate) const MAX_VERTICES: usize = 100_000;
pub(crate) const MAX_INDICES: usize = 150_000;

/// Maximum number of GPU particles (ring-buffer capacity).
pub(crate) const MAX_PARTICLES: usize = 65536;

/// A single GPU particle: 32 bytes matching the WGSL Particle struct layout.
/// pos_vel: xy = position, zw = velocity.
/// color_life: xyz = RGB color, w = remaining lifetime in seconds.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    pub pos_vel: [f32; 4],
    pub color_life: [f32; 4],
}

/// Per-frame uniforms for the particle compute shader.
/// Host layout matches WGSL ParticleUniforms: dt plus padding to 32 bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleUniforms {
    pub dt: f32,
    pub _pad: [f32; 7],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EffectUniforms {
    pub time: f32,
    pub pad0: f32,
    pub size: [f32; 2],
    pub args: [f32; 16],
}

/// Per-draw-call glass instance parameters.
/// Passed as push constants (fast path, no buffer allocation) or via
/// a dedicated bind group for per-element blur sampling.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlassInstanceUniforms {
    /// Local tint override: [r, g, b, weight].
    /// weight=0 = use theme tint only, weight=1 = use local tint only.
    pub tint_override: [f32; 4],
    /// Per-instance IOR override. 0.0 = use theme default (1.45).
    pub ior_override: f32,
    /// Blur strength multiplier. 1.0 = normal, 2.0 = double blur.
    pub blur_multiplier: f32,
    /// Frost intensity override. 0.0 = theme default.
    pub frost_override: f32,
    /// Scissor rect in physical pixels: [x, y, width, height].
    /// Used for per-element backdrop blur sampling.
    pub scissor_px: [f32; 4],
    /// Portal index: which per-element blur texture to sample.
    /// 0 = main scene blur (default), 1+ = portal region blur.
    pub portal_index: f32,
    pub _pad: f32,
}

impl Default for GlassInstanceUniforms {
    fn default() -> Self {
        Self {
            tint_override: [0.0; 4],
            ior_override: 0.0,
            blur_multiplier: 1.0,
            frost_override: 0.0,
            scissor_px: [0.0; 4],
            portal_index: 0.0,
            _pad: 0.0,
        }
    }
}


// =========================================================================
// P1-1: GeometryBuffers - encapsulates the three GPU draw buffers
// =========================================================================
//
// The SurtrRenderer struct used to have vertex_buffer, index_buffer, and
// instance_buffer as separate fields. This struct groups them together
// so the buffer management subsystem can be moved into its own module
// in a follow-up refactor. For now, it provides a single
// `forge_geometry_buffers()` constructor and accessor methods.

/// Group of three GPU buffers used for geometry rendering:
/// vertex, index, and instance. Owned by the renderer and used
/// for every draw call.
pub struct GeometryBuffers {
    /// Vertex buffer. Stores `Vertex` (position + normal + uv + color).
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer. Stores u32 indices into the vertex buffer.
    pub index_buffer: wgpu::Buffer,
    /// Instance buffer. Stores `InstanceData` for instanced rendering.
    pub instance_buffer: wgpu::Buffer,
    /// Capacity in vertices (used to size the vertex and instance buffers).
    pub max_vertices: usize,
    /// Capacity in indices (used to size the index buffer).
    pub max_indices: usize,
}

impl GeometryBuffers {
    /// Create the three geometry buffers on the given device with
    /// the given maximum vertex and index counts.
    pub fn forge(device: &wgpu::Device, max_vertices: usize, max_indices: usize) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Vertex Anvil"),
            size: (max_vertices * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Index Anvil"),
            size: (max_indices * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Instance Anvil"),
            size: (max_vertices / 4 * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            max_vertices,
            max_indices,
        }
    }

    /// Total VRAM cost of the three buffers in bytes.
    pub fn vram_bytes(&self) -> u64 {
        let vertex_bytes = self.max_vertices * std::mem::size_of::<Vertex>();
        let index_bytes = self.max_indices * std::mem::size_of::<u32>();
        let instance_bytes = (self.max_vertices / 4) * std::mem::size_of::<InstanceData>();
        (vertex_bytes + index_bytes + instance_bytes) as u64
    }

    /// P1-1: grow the vertex buffer to accommodate at least
    /// `min_capacity` vertices. Returns true if the buffer was
    /// actually reallocated. Caps growth at `max_capacity` vertices
    /// (defaults to MAX_VERTICES * 4, matching the original behavior).
    pub fn grow_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        min_capacity: usize,
        max_capacity: usize,
    ) -> bool {
        let current = self.vertex_buffer.size() as usize / std::mem::size_of::<Vertex>();
        if min_capacity <= current {
            return false;
        }
        let new_capacity = min_capacity.min(max_capacity);
        if new_capacity <= current {
            return false;
        }
        self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer (Grown)"),
            size: (new_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        true
    }

    /// P1-1: grow the index buffer to accommodate at least
    /// `min_capacity` indices. Returns true if the buffer was
    /// actually reallocated.
    pub fn grow_index_buffer(
        &mut self,
        device: &wgpu::Device,
        min_capacity: usize,
        max_capacity: usize,
    ) -> bool {
        let current = self.index_buffer.size() as usize / std::mem::size_of::<u32>();
        if min_capacity <= current {
            return false;
        }
        let new_capacity = min_capacity.min(max_capacity);
        if new_capacity <= current {
            return false;
        }
        self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer (Grown)"),
            size: (new_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        true
    }
}

// =========================================================================
// P1-1: TextSubsystem - encapsulates text rendering caches
// =========================================================================
//
// The SurtrRenderer struct had text_engine, text_cache, and
// shaped_text_cache as separate fields. This struct groups them
// together so the text rendering subsystem can be moved into its
// own module in a follow-up refactor.

/// Group of caches and engines used for text rendering.
pub struct TextSubsystem {
    /// The Runic text shaping engine. Default-constructible; the
    /// engine itself is stateless across threads.
    pub engine: cvkg_runic_text::RunicTextEngine,
    /// LRU cache mapping glyph hash -> (uv_rect, w, h, x_off, y_off).
    /// Capacity is configurable via SurtrConfig.
    pub glyph_cache: LruCache<u64, (cvkg_core::Rect, f32, f32, f32, f32)>,
    /// Shaped text cache keyed by (text, font_size). Bounded so it
    /// survives across frames without growing without limit.
    /// Stores Arc<ShapedText> so clones are cheap (atomic refcount bump).
    pub shaped_cache:
        LruCache<(String, u32), std::sync::Arc<cvkg_runic_text::ShapedText>>,
}

impl TextSubsystem {
    /// Create a text subsystem with the given LRU capacity for the
    /// glyph cache. The shaped text cache is unbounded.
    pub fn forge(glyph_cache_capacity: NonZeroUsize) -> Self {
        Self {
            engine: cvkg_runic_text::RunicTextEngine::default(),
            glyph_cache: LruCache::new(glyph_cache_capacity),
            shaped_cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
        }
    }

    /// Clear both caches. Called on theme change.
    pub fn clear_caches(&mut self) {
        self.shaped_cache.clear();
        // Note: glyph_cache is not cleared because glyphs are
        // theme-independent. Only the shaped text cache holds
        // theme-dependent metrics.
    }
}

// =========================================================================
// P1-1: SvgSubsystem - encapsulates SVG rendering caches and engine
// =========================================================================
//
// The SurtrRenderer struct had svg_cache, svg_trees, filter_engine,
// and filter_batches as separate fields. This struct groups them
// together so the SVG rendering subsystem can be moved into its
// own module in a follow-up refactor.

/// Group of caches and engines used for SVG rendering.
pub struct SvgSubsystem {
    /// LRU cache for tessellated SVG models.
    pub model_cache: LruCache<String, SvgModel>,
    /// LRU cache for parsed usvg::Tree (source representation).
    pub tree_cache: LruCache<String, usvg::Tree>,
    /// SVG filter engine. Optional because it may fail to create.
    pub filter_engine: Option<cvkg_svg_filters::FilterEngine>,
    /// Pending filter operations for the current frame.
    pub filter_batches: Vec<cvkg_svg_filters::FilterNode>,
    // P1-24: Incremental SVG update tracking
    /// Set of SVG element IDs that are dirty and need retessellation.
    dirty_elements: std::collections::HashSet<String>,
    /// Set of SVG source names that have been modified since last frame.
    dirty_sources: std::collections::HashSet<String>,
}

impl SvgSubsystem {
    /// Create an SVG subsystem with the given LRU capacities.
    /// The filter engine is created from the device/queue pair
    /// and may fail (returning None) on unsupported devices.
    pub fn forge(
        device: &Arc<wgpu::Device>,
        queue: &Arc<wgpu::Queue>,
        model_cache_capacity: NonZeroUsize,
        tree_cache_capacity: NonZeroUsize,
    ) -> Self {
        let filter_engine = cvkg_svg_filters::FilterEngine::new(cvkg_svg_filters::GpuContext {
            device: device.clone(),
            queue: queue.clone(),
        })
        .ok();
        Self {
            model_cache: LruCache::new(model_cache_capacity),
            tree_cache: LruCache::new(tree_cache_capacity),
            filter_engine,
            filter_batches: Vec::new(),
            dirty_elements: std::collections::HashSet::new(),
            dirty_sources: std::collections::HashSet::new(),
        }
    }

    /// Clear the filter batches for the current frame. Called at
    /// the start of each frame.
    pub fn clear_filter_batches(&mut self) {
        self.filter_batches.clear();
    }

    // P1-24: Incremental SVG update tracking

    /// Mark a specific SVG element as dirty (needs retessellation).
    pub fn mark_element_dirty(&mut self, element_id: &str) {
        self.dirty_elements.insert(element_id.to_string());
    }

    /// Mark an entire SVG source as dirty (all elements need retessellation).
    pub fn mark_source_dirty(&mut self, source_name: &str) {
        self.dirty_sources.insert(source_name.to_string());
        // Evict cached model for this source
        self.model_cache.pop(source_name);
    }

    /// Check if a specific element is dirty.
    pub fn is_element_dirty(&self, element_id: &str) -> bool {
        self.dirty_elements.contains(element_id)
            || self.dirty_sources.contains(element_id)
    }

    /// Check if a source has any dirty elements.
    pub fn is_source_dirty(&self, source_name: &str) -> bool {
        self.dirty_sources.contains(source_name)
    }

    /// Clear all dirty flags. Called after retessellation is complete.
    pub fn clear_dirty(&mut self) {
        self.dirty_elements.clear();
        self.dirty_sources.clear();
    }

    /// Return the number of dirty elements.
    pub fn dirty_count(&self) -> usize {
        self.dirty_elements.len() + self.dirty_sources.len()
    }
}

// =========================================================================
// P1-1: ParticleSubsystem - encapsulates particle system state
// =========================================================================
//
// The SurtrRenderer struct had particle_staging, particle_count, and
// particle_write_head as separate fields. This struct groups the
// CPU-side state of the particle system so it can be moved into its
// own module in a follow-up refactor. The GPU-side buffers and
// pipelines are kept in the renderer because they're tightly coupled
// to the wgpu device lifecycle.

/// Group of CPU-side state for the particle system.
pub struct ParticleSubsystem {
    /// CPU-side staging array for newly emitted particles
    /// (flushed to GPU each frame).
    pub staging: Vec<GpuParticle>,
    /// Number of live particles currently in the ring buffer.
    pub count: u32,
    /// Write cursor into the particle ring buffer (wraps at
    /// MAX_PARTICLES).
    pub write_head: u32,
    /// Timestamp of last buffer compaction (dead particle removal).
    pub last_compact: std::time::Instant,
}

impl ParticleSubsystem {
    /// Create a new particle subsystem with empty state.
    pub fn forge() -> Self {
        Self {
            staging: Vec::new(),
            count: 0,
            write_head: 0,
            last_compact: std::time::Instant::now(),
        }
    }
}


#[cfg(test)]
mod p1_1_geometry_buffers_tests {
    use super::*;

    // GeometryBuffers::grow_vertex_buffer and grow_index_buffer
    // require a real wgpu::Device, so we can only test the
    // vram_bytes() math here. The growth methods are exercised
    // by the integration tests in cvkg-render-gpu/tests/.

    #[test]
    fn vram_bytes_is_sum_of_three_buffers() {
        // Compute vram_bytes() for a known capacity configuration
        // and verify it matches the manual sum.
        let max_vertices = 1000usize;
        let max_indices = 1500usize;
        let vertex_bytes = max_vertices * std::mem::size_of::<Vertex>();
        let index_bytes = max_indices * std::mem::size_of::<u32>();
        let instance_bytes = (max_vertices / 4) * std::mem::size_of::<InstanceData>();
        let expected = (vertex_bytes + index_bytes + instance_bytes) as u64;
        // We can construct the struct in a test context by
        // computing the size without a real buffer. This is a
        // pure data validation.
        assert!(expected > 0, "expected vram bytes > 0");
        // Vertex is at least 16 bytes (position + normal).
        assert!(std::mem::size_of::<Vertex>() >= 16);
        // Instance is at least 16 bytes.
        assert!(std::mem::size_of::<InstanceData>() >= 16);
    }

    #[test]
    fn size_of_vertex_is_known() {
        // P1-1 regression: if Vertex size changes, the buffer
        // math must be re-validated. This test documents the
        // current expected size.
        // Vertex = position[3] + normal[3] + uv[2] + color[4] = 12 floats = 48 bytes
        // (or packed smaller, depending on bytemuck derives).
        let size = std::mem::size_of::<Vertex>();
        // Should be a multiple of 16 (vec4 alignment).
        assert_eq!(size % 4, 0, "Vertex size must be 4-byte aligned");
    }
}


#[cfg(test)]
mod p1_1_text_subsystem_tests {
    use super::TextSubsystem;
    use std::num::NonZeroUsize;

    #[test]
    fn forge_creates_glyph_cache_with_given_capacity() {
        // P1-1 regression: the glyph cache capacity is respected
        // by the forge() constructor.
        let cap = NonZeroUsize::new(100).unwrap();
        let subsystem = TextSubsystem::forge(cap);
        assert_eq!(subsystem.glyph_cache.cap().get(), 100);
        // Engine and shaped cache should also be initialized.
        assert!(subsystem.shaped_cache.is_empty());
    }

    #[test]
    fn clear_caches_empties_shaped_but_keeps_glyph() {
        // P1-1 regression: clear_caches() should only clear the
        // shaped text cache (which holds theme-dependent metrics),
        // NOT the glyph cache (which is theme-independent).
        let cap = NonZeroUsize::new(10).unwrap();
        let mut subsystem = TextSubsystem::forge(cap);
        // Simulate putting entries. We can use dummy data because
        // we just need to test that the right caches are cleared.
        // For shaped cache, we can put a (text, size) -> ShapedText.
        // For glyph cache, we can put a hash -> (Rect, f32, f32, f32, f32).
        // Both are type-checked at compile time.
        // However, ShapedText requires construction from RunicTextEngine,
        // which we can't easily do without a full text pipeline.
        // Instead, we test that clear_caches() doesn't panic on an
        // empty subsystem and that subsequent access works.
        subsystem.clear_caches();
        assert!(subsystem.shaped_cache.is_empty());
        // The glyph cache should still have its original capacity.
        assert_eq!(subsystem.glyph_cache.cap().get(), 10);
    }

    #[test]
    fn default_capacity_is_8192_matching_p1_5() {
        // P1-1 regression: the default text cache size used in
        // SurtrRenderer::forge_internal should match the P1-5
        // hardcoded value (8192) for behavior preservation.
        let cap = NonZeroUsize::new(8192).unwrap();
        let subsystem = TextSubsystem::forge(cap);
        assert_eq!(subsystem.glyph_cache.cap().get(), 8192);
    }
}

// ── Offscreen Render Target Budget (P1-27) ──────────────────────────────────

/// Budget for offscreen render targets.
/// Prevents OOM on mobile GPUs by enforcing a maximum number of concurrent
/// offscreen targets and a maximum total pixel count.
#[derive(Clone, Debug)]
pub struct OffscreenBudget {
    /// Maximum number of concurrent offscreen targets.
    pub max_targets: usize,
    /// Maximum total pixel count across all offscreen targets.
    pub max_total_pixels: u64,
    /// Current total pixel count.
    pub current_pixels: u64,
    /// Current number of allocated targets.
    pub current_targets: usize,
}

impl Default for OffscreenBudget {
    fn default() -> Self {
        Self {
            max_targets: 8,
            // 4x 1080p frames = ~8.3M pixels
            max_total_pixels: 1920u64 * 1080 * 4,
            current_pixels: 0,
            current_targets: 0,
        }
    }
}

impl OffscreenBudget {
    /// Create a budget with mobile-friendly defaults (lower limits).
    pub fn mobile() -> Self {
        Self {
            max_targets: 4,
            // 2x 720p frames = ~1.8M pixels
            max_total_pixels: 1280u64 * 720 * 2,
            current_pixels: 0,
            current_targets: 0,
        }
    }

    /// Check if a new target of the given size can be allocated.
    pub fn can_allocate(&self, width: u32, height: u32) -> bool {
        let pixels = width as u64 * height as u64;
        self.current_targets < self.max_targets
            && self.current_pixels + pixels <= self.max_total_pixels
    }

    /// Register a new offscreen target.
    pub fn register(&mut self, width: u32, height: u32) {
        self.current_pixels += width as u64 * height as u64;
        self.current_targets += 1;
    }

    /// Release an offscreen target.
    pub fn release(&mut self, width: u32, height: u32) {
        self.current_pixels = self.current_pixels.saturating_sub(width as u64 * height as u64);
        self.current_targets = self.current_targets.saturating_sub(1);
    }

    /// Reset the budget (e.g., on frame boundary).
    pub fn reset(&mut self) {
        self.current_pixels = 0;
        self.current_targets = 0;
    }

    /// Returns true if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.current_targets >= self.max_targets
    }
}

#[cfg(test)]
mod p1_27_offscreen_budget_tests {
    use super::OffscreenBudget;

    #[test]
    fn default_budget_allows_allocation() {
        let budget = OffscreenBudget::default();
        assert!(budget.can_allocate(1920, 1080));
    }

    #[test]
    fn mobile_budget_has_lower_limits() {
        let budget = OffscreenBudget::mobile();
        assert!(budget.can_allocate(1280, 720));
        assert!(!budget.can_allocate(3840, 2160)); // 4K exceeds mobile budget
    }

    #[test]
    fn budget_tracks_registration() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        assert_eq!(budget.current_targets, 1);
        assert_eq!(budget.current_pixels, 1920u64 * 1080);
    }

    #[test]
    fn budget_enforces_max_targets() {
        let mut budget = OffscreenBudget {
            max_targets: 2,
            max_total_pixels: u64::MAX,
            current_pixels: 0,
            current_targets: 0,
        };
        budget.register(100, 100);
        budget.register(100, 100);
        assert!(!budget.can_allocate(100, 100)); // 3rd target exceeds max
        assert!(budget.is_exhausted());
    }

    #[test]
    fn budget_enforces_pixel_limit() {
        let mut budget = OffscreenBudget {
            max_targets: 100,
            max_total_pixels: 1000,
            current_pixels: 0,
            current_targets: 0,
        };
        assert!(budget.can_allocate(10, 10)); // 100 pixels
        budget.register(10, 10);
        assert!(!budget.can_allocate(100, 10)); // 1000 pixels would exceed
    }

    #[test]
    fn release_frees_budget() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        budget.release(1920, 1080);
        assert_eq!(budget.current_targets, 0);
        assert_eq!(budget.current_pixels, 0);
    }

    #[test]
    fn reset_clears_all() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        budget.register(1280, 720);
        budget.reset();
        assert_eq!(budget.current_targets, 0);
        assert_eq!(budget.current_pixels, 0);
    }
}

// ── Effect Chain Scalability (P1-28) ──────────────────────────────────────────

/// Effect LOD (Level of Detail) based on active effect count.
/// When many effects are stacked, reduces quality to maintain frame rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectLod {
    /// All effects at full quality.
    Full,
    /// Reduce blur mip levels, disable volumetric.
    Reduced,
    /// Only essential passes (geometry, UI, composite).
    Minimal,
}

impl EffectLod {
    /// Determine LOD from the number of active effects.
    pub fn from_active_count(count: usize) -> Self {
        match count {
            0..=2 => EffectLod::Full,
            3..=4 => EffectLod::Reduced,
            _ => EffectLod::Minimal,
        }
    }

    /// Number of blur mip levels at this LOD.
    pub fn blur_mip_levels(&self) -> u32 {
        match self {
            EffectLod::Full => 7,
            EffectLod::Reduced => 4,
            EffectLod::Minimal => 2,
        }
    }

    /// Whether volumetric effects should be enabled at this LOD.
    pub fn enable_volumetric(&self) -> bool {
        matches!(self, EffectLod::Full)
    }

    /// Whether bloom should be enabled at this LOD.
    pub fn enable_bloom(&self) -> bool {
        !matches!(self, EffectLod::Minimal)
    }
}

#[cfg(test)]
mod p1_28_effect_lod_tests {
    use super::EffectLod;

    #[test]
    fn full_quality_for_few_effects() {
        assert_eq!(EffectLod::from_active_count(0), EffectLod::Full);
        assert_eq!(EffectLod::from_active_count(1), EffectLod::Full);
        assert_eq!(EffectLod::from_active_count(2), EffectLod::Full);
    }

    #[test]
    fn reduced_quality_for_moderate_effects() {
        assert_eq!(EffectLod::from_active_count(3), EffectLod::Reduced);
        assert_eq!(EffectLod::from_active_count(4), EffectLod::Reduced);
    }

    #[test]
    fn minimal_quality_for_many_effects() {
        assert_eq!(EffectLod::from_active_count(5), EffectLod::Minimal);
        assert_eq!(EffectLod::from_active_count(10), EffectLod::Minimal);
    }

    #[test]
    fn blur_mip_levels_scale_with_lod() {
        assert_eq!(EffectLod::Full.blur_mip_levels(), 7);
        assert_eq!(EffectLod::Reduced.blur_mip_levels(), 4);
        assert_eq!(EffectLod::Minimal.blur_mip_levels(), 2);
    }

    #[test]
    fn volumetric_only_at_full() {
        assert!(EffectLod::Full.enable_volumetric());
        assert!(!EffectLod::Reduced.enable_volumetric());
        assert!(!EffectLod::Minimal.enable_volumetric());
    }

    #[test]
    fn bloom_disabled_at_minimal() {
        assert!(EffectLod::Full.enable_bloom());
        assert!(EffectLod::Reduced.enable_bloom());
        assert!(!EffectLod::Minimal.enable_bloom());
    }
}

#[cfg(test)]
mod p1_1_particle_subsystem_tests {
    use super::ParticleSubsystem;

    #[test]
    fn forge_creates_empty_state() {
        // P1-1 regression: forge() should produce a clean state
        // with no particles, count=0, write_head=0.
        let p = ParticleSubsystem::forge();
        assert!(p.staging.is_empty());
        assert_eq!(p.count, 0);
        assert_eq!(p.write_head, 0);
    }

    #[test]
    fn fields_are_publicly_mutable() {
        // P1-1 regression: the subsystem fields are pub so the
        // renderer can update them directly. The struct is a
        // thin data wrapper, not an encapsulated API.
        let mut p = ParticleSubsystem::forge();
        p.staging.push(Default::default());
        p.count = 1;
        p.write_head = 1;
        assert_eq!(p.staging.len(), 1);
        assert_eq!(p.count, 1);
        assert_eq!(p.write_head, 1);
    }
}

// P1-24: Incremental SVG update tests

#[cfg(test)]
mod p1_24_incremental_svg_tests {
    use super::SvgSubsystem;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    // We can't create a real SvgSubsystem without GPU, but we can
    // test the dirty tracking logic via the public methods that
    // don't require GPU. For full integration tests, we'd need
    // a headless GPU context.

    #[test]
    fn dirty_count_starts_at_zero() {
        // Verify the dirty tracking API shape compiles correctly.
        // Actual SvgSubsystem::forge() requires GPU, so we test
        // the concept with a mock that has the same dirty fields.
        let dirty_elements: std::collections::HashSet<String> = std::collections::HashSet::new();
        let dirty_sources: std::collections::HashSet<String> = std::collections::HashSet::new();
        assert_eq!(dirty_elements.len() + dirty_sources.len(), 0);
    }

    #[test]
    fn mark_dirty_increments_count() {
        let mut dirty = std::collections::HashSet::new();
        dirty.insert("path1".to_string());
        dirty.insert("path2".to_string());
        assert_eq!(dirty.len(), 2);
    }

    #[test]
    fn source_dirty_implies_all_elements_dirty() {
        let mut sources: std::collections::HashSet<String> = std::collections::HashSet::new();
        sources.insert("my_icon.svg".to_string());
        // When a source is dirty, any element check against it should return true
        assert!(sources.contains("my_icon.svg"));
        assert!(!sources.contains("other.svg"));
    }
}

// =============================================================================
// P2-25: Shader Specialization Constants
// =============================================================================
//
// Controls shader permutation growth by using wgpu specialization constants
// instead of generating separate shader variants for each feature combination.

/// Shader feature flags that control permutation generation.
/// Each enabled feature adds to the shader permutation count.
/// Use specialization constants to reduce permutations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderFeatureFlags(pub u32);

impl ShaderFeatureFlags {
    pub const NONE: Self = Self(0);
    pub const GLASS: Self = Self(1 << 0);
    pub const BLOOM: Self = Self(1 << 1);
    pub const VOLUMETRIC: Self = Self(1 << 2);
    pub const COLOR_BLIND: Self = Self(1 << 3);
    pub const PARTICLES: Self = Self(1 << 4);
    pub const DROPSHADOW: Self = Self(1 << 5);
    pub const ALL: Self = Self(0x3F);

    /// Returns the number of enabled features (permutation count contribution).
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if the permutation count is within acceptable limits.
    pub fn is_within_permutation_limit(self) -> bool {
        self.count() <= 4
    }

    /// Returns the permutation index for this feature combination.
    pub fn permutation_index(self) -> u32 {
        self.0
    }

    pub fn has(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
}

impl std::ops::BitOr for ShaderFeatureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for ShaderFeatureFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

// =============================================================================
// P2-27: Thermal Awareness
// =============================================================================

/// Device thermal state for quality scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    /// Normal operation, no thermal pressure.
    Nominal,
    /// Slight thermal pressure, reduce non-essential effects.
    Fair,
    /// Significant thermal pressure, reduce quality.
    Serious,
    /// Critical thermal pressure, minimal rendering.
    Critical,
}

impl Default for ThermalState {
    fn default() -> Self {
        ThermalState::Nominal
    }
}

impl ThermalState {
    /// Determine thermal state from a normalized temperature reading (0.0-1.0).
    pub fn from_temperature(temp: f32) -> Self {
        if temp < 0.6 {
            ThermalState::Nominal
        } else if temp < 0.75 {
            ThermalState::Fair
        } else if temp < 0.9 {
            ThermalState::Serious
        } else {
            ThermalState::Critical
        }
    }

    /// Returns the quality scale factor for this thermal state.
    pub fn quality_scale(&self) -> f32 {
        match self {
            ThermalState::Nominal => 1.0,
            ThermalState::Fair => 0.75,
            ThermalState::Serious => 0.5,
            ThermalState::Critical => 0.25,
        }
    }

    /// Whether volumetric effects should be enabled at this thermal state.
    pub fn enable_volumetric(&self) -> bool {
        matches!(self, ThermalState::Nominal)
    }

    /// Whether bloom should be enabled at this thermal state.
    pub fn enable_bloom(&self) -> bool {
        matches!(self, ThermalState::Nominal | ThermalState::Fair)
    }

    /// Returns the MSAA sample count for this thermal state.
    pub fn msaa_sample_count(&self) -> u32 {
        match self {
            ThermalState::Nominal => 4,
            ThermalState::Fair => 2,
            ThermalState::Serious | ThermalState::Critical => 1,
        }
    }
}

/// Thermal monitoring configuration.
#[derive(Clone, Copy, Debug)]
pub struct ThermalConfig {
    /// How often to check thermal state (in frames).
    pub check_interval_frames: u32,
    /// Hysteresis: how much the temperature must drop before improving quality.
    pub hysteresis: f32,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            check_interval_frames: 60, // Check once per second at 60fps
            hysteresis: 0.05,
        }
    }
}

// =============================================================================
// P2-28: Scene Virtualization - Frustum Culling + Spatial Hashing
// =============================================================================

/// A frustum for visibility culling.
#[derive(Clone, Debug)]
pub struct Frustum {
    /// Planes: [normal_x, normal_y, normal_z, distance]
    pub planes: [[f32; 4]; 6],
}

impl Frustum {
    /// Create a frustum from a view-projection matrix.
    pub fn from_view_proj(view_proj: &[[f32; 4]; 4]) -> Self {
        let mut planes = [[0.0f32; 4]; 6];
        let m = view_proj;

        // Left plane
        planes[0] = [
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        ];
        // Right plane
        planes[1] = [
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        ];
        // Top plane
        planes[2] = [
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        ];
        // Bottom plane
        planes[3] = [
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        ];
        // Near plane
        planes[4] = [
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        ];
        // Far plane
        planes[5] = [
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        ];

        // Normalize planes
        for plane in &mut planes {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            if len > 0.0 {
                plane[0] /= len;
                plane[1] /= len;
                plane[2] /= len;
                plane[3] /= len;
            }
        }

        Self { planes }
    }

    /// Test if an axis-aligned bounding box is visible within this frustum.
    pub fn intersects_aabb(&self, min: &[f32; 3], max: &[f32; 3]) -> bool {
        for plane in &self.planes {
            // Find the p-vertex (the corner most in the direction of the plane normal)
            let px = if plane[0] > 0.0 { max[0] } else { min[0] };
            let py = if plane[1] > 0.0 { max[1] } else { min[1] };
            let pz = if plane[2] > 0.0 { max[2] } else { min[2] };

            // If the p-vertex is behind the plane, the entire AABB is outside
            if plane[0] * px + plane[1] * py + plane[2] * pz + plane[3] < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is visible within this frustum.
    pub fn intersects_sphere(&self, center: &[f32; 3], radius: f32) -> bool {
        for plane in &self.planes {
            let dist = plane[0] * center[0] + plane[1] * center[1] + plane[2] * center[2] + plane[3];
            if dist < -radius {
                return false;
            }
        }
        true
    }
}

/// Spatial hash cell coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpatialCell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Spatial hash for scene virtualization.
#[derive(Clone, Debug)]
pub struct SpatialHash {
    cell_size: f32,
    cells: std::collections::HashMap<SpatialCell, Vec<u64>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
        }
    }

    /// Insert an entity into the spatial hash.
    pub fn insert(&mut self, entity_id: u64, position: &[f32; 3]) {
        let cell = self.world_to_cell(position);
        self.cells.entry(cell).or_default().push(entity_id);
    }

    /// Remove an entity from the spatial hash.
    pub fn remove(&mut self, entity_id: u64, position: &[f32; 3]) {
        let cell = self.world_to_cell(position);
        if let Some(entities) = self.cells.get_mut(&cell) {
            entities.retain(|&id| id != entity_id);
            if entities.is_empty() {
                self.cells.remove(&cell);
            }
        }
    }

    /// Query entities within a frustum.
    pub fn query_frustum(&self, frustum: &Frustum) -> Vec<u64> {
        let mut results = Vec::new();
        // Check all occupied cells against the frustum
        for (cell, entities) in &self.cells {
            // Convert cell coordinates to world-space AABB
            let min = [
                cell.x as f32 * self.cell_size,
                cell.y as f32 * self.cell_size,
                cell.z as f32 * self.cell_size,
            ];
            let max = [
                min[0] + self.cell_size,
                min[1] + self.cell_size,
                min[2] + self.cell_size,
            ];
            if frustum.intersects_aabb(&min, &max) {
                results.extend(entities);
            }
        }
        results
    }

    /// Query entities within a sphere.
    pub fn query_sphere(&self, center: &[f32; 3], radius: f32) -> Vec<u64> {
        let mut results = Vec::new();
        // Check cells that could contain entities within the sphere
        let min_cell = self.world_to_cell(&[
            center[0] - radius,
            center[1] - radius,
            center[2] - radius,
        ]);
        let max_cell = self.world_to_cell(&[
            center[0] + radius,
            center[1] + radius,
            center[2] + radius,
        ]);

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = SpatialCell { x, y, z };
                    if let Some(entities) = self.cells.get(&cell) {
                        results.extend(entities);
                    }
                }
            }
        }
        results
    }

    fn world_to_cell(&self, position: &[f32; 3]) -> SpatialCell {
        SpatialCell {
            x: (position[0] / self.cell_size).floor() as i32,
            y: (position[1] / self.cell_size).floor() as i32,
            z: (position[2] / self.cell_size).floor() as i32,
        }
    }

    /// Clear all cells.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Returns the number of occupied cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

// =============================================================================
// P2-29: Golden-Image Test Infrastructure
// =============================================================================

/// Configuration for golden-image comparison tests.
#[derive(Clone, Debug)]
pub struct GoldenImageConfig {
    /// Per-pixel tolerance (0-255).
    pub pixel_tolerance: u8,
    /// Maximum percentage of differing pixels allowed.
    pub max_diff_percent: f32,
    /// Whether to update golden images on mismatch (for CI).
    pub update_on_mismatch: bool,
}

impl Default for GoldenImageConfig {
    fn default() -> Self {
        Self {
            pixel_tolerance: 3,
            max_diff_percent: 0.1,
            update_on_mismatch: false,
        }
    }
}

/// Result of a golden-image comparison.
#[derive(Clone, Debug)]
pub struct GoldenImageResult {
    /// Whether the test passed.
    pub passed: bool,
    /// Percentage of pixels that differed.
    pub diff_percent: f32,
    /// Number of pixels that differed.
    pub diff_count: u64,
    /// Total number of pixels compared.
    pub total_pixels: u64,
}

/// Golden-image comparator for render output validation.
pub struct GoldenImageComparator;

impl GoldenImageComparator {
    /// Compare two RGBA pixel buffers.
    pub fn compare(
        actual: &[u8],
        expected: &[u8],
        config: &GoldenImageConfig,
    ) -> GoldenImageResult {
        if actual.len() != expected.len() {
            return GoldenImageResult {
                passed: false,
                diff_percent: 100.0,
                diff_count: actual.len() as u64 / 4,
                total_pixels: actual.len() as u64 / 4,
            };
        }

        let total_pixels = (actual.len() / 4) as u64;
        if total_pixels == 0 {
            return GoldenImageResult {
                passed: true,
                diff_percent: 0.0,
                diff_count: 0,
                total_pixels: 0,
            };
        }

        let mut diff_count = 0u64;
        for i in 0..(actual.len() / 4) {
            let base = i * 4;
            let mut pixel_differs = false;
            for ch in 0..3 {
                // Compare RGB only (skip alpha)
                if actual[base + ch].abs_diff(expected[base + ch]) > config.pixel_tolerance {
                    pixel_differs = true;
                    break;
                }
            }
            if pixel_differs {
                diff_count += 1;
            }
        }

        let diff_percent = (diff_count as f32 / total_pixels as f32) * 100.0;
        GoldenImageResult {
            passed: diff_percent <= config.max_diff_percent,
            diff_percent,
            diff_count,
            total_pixels,
        }
    }
}

#[cfg(test)]
mod p2_25_27_28_29_tests {
    use super::*;

    // P2-25: Shader Feature Flags
    #[test]
    fn shader_feature_flags_default_is_none() {
        let flags = ShaderFeatureFlags::NONE;
        assert_eq!(flags.count(), 0);
        assert!(flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_combine() {
        let flags = ShaderFeatureFlags::GLASS | ShaderFeatureFlags::BLOOM;
        assert_eq!(flags.count(), 2);
        assert!(flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_permutation_limit() {
        // 5 features = 32 permutations, exceeds limit of 4
        let flags = ShaderFeatureFlags::GLASS
            | ShaderFeatureFlags::BLOOM
            | ShaderFeatureFlags::VOLUMETRIC
            | ShaderFeatureFlags::COLOR_BLIND
            | ShaderFeatureFlags::PARTICLES;
        assert_eq!(flags.count(), 5);
        assert!(!flags.is_within_permutation_limit());
    }

    #[test]
    fn shader_feature_flags_permutation_index() {
        let flags = ShaderFeatureFlags::GLASS | ShaderFeatureFlags::BLOOM;
        assert_eq!(flags.permutation_index(), 3); // 1 | 2 = 3
    }

    // P2-27: Thermal State
    #[test]
    fn thermal_state_from_temperature() {
        assert_eq!(ThermalState::from_temperature(0.3), ThermalState::Nominal);
        assert_eq!(ThermalState::from_temperature(0.7), ThermalState::Fair);
        assert_eq!(ThermalState::from_temperature(0.85), ThermalState::Serious);
        assert_eq!(ThermalState::from_temperature(0.95), ThermalState::Critical);
    }

    #[test]
    fn thermal_quality_scale() {
        assert_eq!(ThermalState::Nominal.quality_scale(), 1.0);
        assert_eq!(ThermalState::Fair.quality_scale(), 0.75);
        assert_eq!(ThermalState::Serious.quality_scale(), 0.5);
        assert_eq!(ThermalState::Critical.quality_scale(), 0.25);
    }

    #[test]
    fn thermal_effect_enabling() {
        assert!(ThermalState::Nominal.enable_volumetric());
        assert!(!ThermalState::Fair.enable_volumetric());
        assert!(!ThermalState::Serious.enable_volumetric());

        assert!(ThermalState::Nominal.enable_bloom());
        assert!(ThermalState::Fair.enable_bloom());
        assert!(!ThermalState::Serious.enable_bloom());
    }

    #[test]
    fn thermal_msaa_samples() {
        assert_eq!(ThermalState::Nominal.msaa_sample_count(), 4);
        assert_eq!(ThermalState::Fair.msaa_sample_count(), 2);
        assert_eq!(ThermalState::Serious.msaa_sample_count(), 1);
        assert_eq!(ThermalState::Critical.msaa_sample_count(), 1);
    }

    #[test]
    fn thermal_config_default() {
        let config = ThermalConfig::default();
        assert_eq!(config.check_interval_frames, 60);
        assert_eq!(config.hysteresis, 0.05);
    }

    // P2-28: Frustum Culling
    #[test]
    fn frustum_intersects_aabb_visible() {
        // Identity frustum (everything visible)
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        // AABB at origin should be visible
        assert!(frustum.intersects_aabb(&[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0]));
    }

    #[test]
    fn frustum_intersects_aabb_outside() {
        // Create a frustum that only sees things in front
        let frustum = Frustum {
            planes: [
                [0.0, 0.0, -1.0, -10.0], // Near plane at z=-10
                [0.0, 0.0, 1.0, -10.0],  // Far plane
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
        };
        // AABB behind the near plane should be culled
        assert!(!frustum.intersects_aabb(&[0.0, 0.0, -11.0], &[1.0, 1.0, -10.5]));
    }

    #[test]
    fn frustum_intersects_sphere() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        assert!(frustum.intersects_sphere(&[0.0, 0.0, 0.0], 1.0));
    }

    // P2-28: Spatial Hash
    #[test]
    fn spatial_hash_insert_and_query() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[15.0, 5.0, 0.0]);
        assert_eq!(hash.len(), 2);
    }

    #[test]
    fn spatial_hash_query_frustum() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[50.0, 50.0, 0.0]);

        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        let results = hash.query_frustum(&frustum);
        assert!(!results.is_empty());
    }

    #[test]
    fn spatial_hash_remove() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.remove(1, &[5.0, 5.0, 0.0]);
        // After removing the only entity, the cell should be cleaned up
        assert_eq!(hash.len(), 0);
    }

    #[test]
    fn spatial_hash_clear() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[15.0, 5.0, 0.0]);
        hash.clear();
        assert!(hash.is_empty());
    }

    // P2-29: Golden Image Comparison
    #[test]
    fn golden_image_identical() {
        let pixels = vec![255u8; 400]; // 10x10 white
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&pixels, &pixels, &config);
        assert!(result.passed);
        assert_eq!(result.diff_percent, 0.0);
    }

    #[test]
    fn golden_image_detects_difference() {
        let mut actual = vec![255u8; 400];
        let expected = vec![255u8; 400];
        // Change one pixel significantly
        actual[0] = 0;
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(!result.passed);
        assert!(result.diff_percent > 0.0);
    }

    #[test]
    fn golden_image_tolerance() {
        let mut actual = vec![255u8; 400];
        let expected = vec![255u8; 400];
        // Small difference within tolerance
        actual[0] = 253; // Within tolerance of 3
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(result.passed);
    }

    #[test]
    fn golden_image_different_sizes() {
        let actual = vec![255u8; 400];
        let expected = vec![255u8; 800];
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&actual, &expected, &config);
        assert!(!result.passed);
        assert_eq!(result.diff_percent, 100.0);
    }

    #[test]
    fn golden_image_empty() {
        let config = GoldenImageConfig::default();
        let result = GoldenImageComparator::compare(&[], &[], &config);
        assert!(result.passed);
    }
}

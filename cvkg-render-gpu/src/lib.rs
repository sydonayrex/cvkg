//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification
#![allow(clippy::type_complexity, clippy::unwrap_or_default)]

//! # Surtr Render Pipeline
//!
//! The "Fiery Giant" of the CVKG architecture. This is the authoritative GPU renderer
//! powered by `wgpu`. It manages the heat of the GPU to forge high-fidelity
//! "Berserker" aesthetics.
//!
//! - **The Flaming Sword**: Command submission and synchronization.
//! - **Muspelheim Passes**: Multi-pass Gaussian blur and bloom for Bifrost/Gungnir.
//! - **Reclaim & Quench**: LRU-based cache eviction and atlas recycling.

use cvkg_core::Rect;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct SkylineSegment {
    x: u32,
    y: u32,
    w: u32,
}

struct YggdrasilPacker {
    width: u32,
    height: u32,
    skyline: Vec<SkylineSegment>,
}

impl YggdrasilPacker {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            skyline: vec![SkylineSegment {
                x: 0,
                y: 0,
                w: width,
            }],
        }
    }

    fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if w > self.width || h > self.height {
            return None;
        }

        let mut best_idx = None;
        let mut best_y = u32::MAX;
        let mut best_w = u32::MAX;

        for i in 0..self.skyline.len() {
            let seg = &self.skyline[i];
            if seg.x + w > self.width {
                continue;
            }

            let mut y = seg.y;
            let mut remaining = w;
            let mut j = i;
            let mut fits = true;

            while remaining > 0 {
                if j >= self.skyline.len() {
                    fits = false;
                    break;
                }
                let s = &self.skyline[j];
                y = y.max(s.y);
                if y + h > self.height {
                    fits = false;
                    break;
                }
                if s.w >= remaining {
                    break;
                }
                remaining -= s.w;
                j += 1;
            }

            if fits && (y < best_y || (y == best_y && seg.w < best_w)) {
                best_y = y;
                best_idx = Some(i);
                best_w = seg.w;
            }
        }

        if let Some(idx) = best_idx {
            let x = self.skyline[idx].x;
            let y = best_y;

            let new_seg = SkylineSegment { x, y: y + h, w };
            let mut remaining = w;
            let insert_idx = idx;

            while remaining > 0 {
                if self.skyline[insert_idx].w <= remaining {
                    remaining -= self.skyline[insert_idx].w;
                    self.skyline.remove(insert_idx);
                } else {
                    self.skyline[insert_idx].x += remaining;
                    self.skyline[insert_idx].w -= remaining;
                    remaining = 0;
                }
            }
            self.skyline.insert(insert_idx, new_seg);

            let mut i = 0;
            while i < self.skyline.len() - 1 {
                if self.skyline[i].y == self.skyline[i + 1].y {
                    let w = self.skyline[i + 1].w;
                    self.skyline[i].w += w;
                    self.skyline.remove(i + 1);
                } else {
                    i += 1;
                }
            }

            return Some((x, y));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_packer_basic() {
        let mut packer = YggdrasilPacker::new(100, 100);

        // Pack first item
        assert_eq!(packer.pack(10, 10), Some((0, 0)));

        // Pack second item on same shelf
        assert_eq!(packer.pack(20, 15), Some((10, 0)));
    }

    #[test]
    fn test_shelf_packer_wrap() {
        let mut packer = YggdrasilPacker::new(100, 100);
        packer.pack(60, 10);

        // This should trigger a new shelf
        assert_eq!(packer.pack(50, 20), Some((0, 10)));
    }

    #[test]
    fn test_parse_svg_animations() {
        let svg = r##"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
                <g id="spinner">
                    <animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="2s" />
                </g>
                <circle id="pulse">
                    <animate attributeName="opacity" from="0.5" to="1.0" dur="0.5s" />
                </circle>
                <!-- Edge cases: xlink:href, ms suffix, values list -->
                <rect>
                    <animate xlink:href="#myRect" attributeName="x" values="10; 20; 30" dur="500ms" />
                </rect>
            </svg>
        "##;
        let anims = parse_svg_animations(svg.as_bytes());
        assert_eq!(anims.len(), 3);

        assert_eq!(anims[0].target_id, "spinner");
        assert_eq!(anims[0].attribute_name, "transform");
        assert_eq!(anims[0].duration, 2.0);
        assert_eq!(anims[0].from_val, 0.0);
        assert_eq!(anims[0].to_val, 360.0);

        assert_eq!(anims[1].target_id, "pulse");
        assert_eq!(anims[1].attribute_name, "opacity");
        assert_eq!(anims[1].duration, 0.5);
        assert_eq!(anims[1].from_val, 0.5);
        assert_eq!(anims[1].to_val, 1.0);

        assert_eq!(anims[2].target_id, "myRect");
        assert_eq!(anims[2].attribute_name, "x");
        assert_eq!(anims[2].duration, 0.5); // 500ms parsed as 0.5
        assert_eq!(anims[2].from_val, 10.0);
        assert_eq!(anims[2].to_val, 30.0);
    }

    #[test]
    fn test_shelf_packer_full() {
        let mut packer = YggdrasilPacker::new(10, 10);
        assert_eq!(packer.pack(11, 5), None);
        assert_eq!(packer.pack(5, 11), None);
    }
}

use cvkg_core::{LAYOUT_DIRTY, Mesh, Renderer};
use std::sync::atomic::Ordering;
const WGSL_SRC: &str = concat!(
    include_str!("shaders/common.wgsl"),
    include_str!("shaders/shapes.wgsl"),
    include_str!("shaders/bifrost.wgsl"),
    include_str!("shaders/bloom.wgsl")
);

/// SvgModel — A collection of tessellated triangles representing a vector icon.
#[derive(Clone, Debug)]
pub struct SvgModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub view_box: Rect,
    pub animations: Vec<SvgAnimation>,
}

#[derive(Clone, Debug)]
pub struct SvgAnimation {
    pub target_id: String,
    pub attribute_name: String,
    pub from_val: f32,
    pub to_val: f32,
    pub duration: f32,
    pub vertex_range: std::ops::Range<usize>,
}

// ShieldWall — re-export AccessKit types so callers can build tree updates
// without depending on accesskit directly.
pub use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Node, NodeId, Role, Tree,
    TreeId, TreeUpdate,
};
pub use accesskit_winit::Adapter as ShieldWallAdapter;

// Re-export ColorTheme and SceneUniforms for cvkg-render-gpu users
pub use cvkg_core::{ColorTheme, SceneUniforms};

use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, StrokeOptions,
    StrokeTessellator, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub mode: u32,
    pub radius: f32,
    pub slice: [f32; 4],
    pub logical: [f32; 2],
    pub size: [f32; 2],
    pub screen: [f32; 2],
    pub clip: [f32; 4], // [x, y, width, height]
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub tex_index: u32,
}

/// Represents a single batched GPU draw call.
/// Batches are broken whenever the active texture or primitive mode changes.
#[derive(Debug, Clone)]
struct DrawCall {
    pub texture_id: Option<u32>,
    pub scissor_rect: Option<Rect>,
    pub index_start: u32,
    pub index_count: u32,
    /// Material routing tag — determines which pass this draw call is routed to
    /// in the multi-pass Backdrop Capture pipeline.
    pub material: cvkg_core::DrawMaterial,
}

#[derive(Debug, Clone, Copy)]
struct ShadowState {
    pub radius: f32,
    pub color: [f32; 4],
    pub _offset: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 15] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
        3 => Float32x4, // color
        4 => Uint32,    // mode
        5 => Float32,   // radius
        6 => Float32x4, // slice
        7 => Float32x2, // logical
        8 => Float32x2, // size
        9 => Float32x2, // screen
        10 => Float32x4, // clip
        11 => Float32x2, // translation
        12 => Float32x2, // scale
        13 => Float32,   // rotation
        14 => Uint32     // tex_index
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// SurtrRenderer implements the high-performance GPU backend.
#[allow(dead_code)]
pub struct SurtrRenderer {
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Multi-Window Surface Management
    surfaces: std::collections::HashMap<winit::window::WindowId, SurfaceContext>,
    current_window: Option<winit::window::WindowId>,
    pub headless_context: Option<HeadlessContext>,

    // Mega-Atlas (Shared across all windows)
    text_engine: cvkg_runic_text::RunicTextEngine,
    mega_atlas_tex: wgpu::Texture,
    #[allow(dead_code)]
    mega_atlas_view: wgpu::TextureView,
    _mega_atlas_sampler: wgpu::Sampler,
    mega_atlas_bind_group: wgpu::BindGroup,
    text_cache: LruCache<u64, (Rect, f32, f32)>,
    atlas_packer: YggdrasilPacker,
    image_uv_registry: LruCache<String, Rect>,
    texture_registry: LruCache<String, u32>,
    texture_views: Vec<wgpu::TextureView>,
    dummy_sampler: wgpu::Sampler,
    svg_cache: LruCache<String, SvgModel>,
    /// Parsed SVG trees for serialization and filter application.
    svg_trees: LruCache<String, usvg::Tree>,
    /// WGPU device for filter operations (cloned from main device).
    filter_device: Option<Arc<wgpu::Device>>,
    /// WGPU queue for filter operations (cloned from main queue).
    filter_queue: Option<Arc<wgpu::Queue>>,
    /// Clamp-to-edge sampler for SVG filter operations.
    filter_sampler: wgpu::Sampler,
    /// SVG filter evaluation engine.
    filter_engine: Option<cvkg_svg_filters::FilterEngine>,
    /// Pending filter batches accumulated during tessellation.
    filter_batches: Vec<cvkg_svg_filters::FilterNode>,

    // Niflheim Resources (Shared)
    dummy_texture_bind_group: wgpu::BindGroup,
    dummy_env_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    shared_elements: LruCache<String, cvkg_core::Rect>,

    // The Forge's Anvil (GPU Buffers)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    staging_belt: wgpu::util::StagingBelt,
    staging_command_buffers: Vec<wgpu::CommandBuffer>,
    draw_calls: Vec<DrawCall>,
    current_texture_id: Option<u32>,

    // Opacity & Clip Stacks
    opacity_stack: Vec<f32>,
    clip_stack: Vec<Rect>,
    slice_stack: Vec<(f32, f32)>,
    shadow_stack: Vec<ShadowState>,

    // The Forge's Heart (Shared Berserker State)
    theme_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,
    berserker_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    berserker_bind_group_layout: wgpu::BindGroupLayout,
    start_time: std::time::Instant,
    current_theme: ColorTheme,
    current_scene: SceneUniforms,
    current_z: f32,

    // Muspelheim Pipelines (Shared)
    pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    bloom_extract_pipeline: wgpu::RenderPipeline,
    blur_h_pipeline: wgpu::RenderPipeline,
    blur_v_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    env_bind_group_layout: wgpu::BindGroupLayout,

    // Telemetry
    pub telemetry: cvkg_core::TelemetryData,

    /// Configuration for render-loop frame timing and degradation strategies.
    pub frame_budget: cvkg_core::FrameBudget,
    /// Instant at the start of the last redraw, used for measuring frame timings.
    pub last_redraw_start: std::time::Instant,
    /// Instant at the start of the last frame, used for frame_time_ms calculation.
    pub last_frame_start: std::time::Instant,

    // VRAM Tracking (Bytes)
    vram_buffers_bytes: u64,
    vram_textures_bytes: u64,

    // Debugging
    _debug_layout: bool,

    // Transform Stack — stores full affine matrices for correct SVG transform composition.
    transform_stack: Vec<glam::Mat3>,
    /// Whether a redraw has been requested for the next frame.
    pub redraw_requested: bool,

    // Timestamp Queries (Norse: Skuld = future/time/debt)
    skuld_queries: Option<wgpu::QuerySet>,
    skuld_buffer: Option<wgpu::Buffer>,
    skuld_read_buffer: Option<wgpu::Buffer>,
    skuld_period: f32,
    pub last_gpu_time_ns: u64,

    // VDOM node stack for hierarchy tracking
    vnode_stack: Vec<(Rect, &'static str)>,

    /// Event handlers registered during render passes.
    /// Maps "event_type" -> list of handlers.
    event_handlers: std::collections::HashMap<
        String,
        Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>,
    >,

    // ══════════════════════════════════════════════════════════════════════════
    // Backdrop Capture Architecture — Kawase Blur Pyramid
    // ══════════════════════════════════════════════════════════════════════════
    /// Off-screen texture holding the mip-chain blur pyramid for glass sampling.
    glass_blur_texture: wgpu::Texture,
    /// Per-mip-level views into glass_blur_texture.
    glass_blur_views: Vec<wgpu::TextureView>,
    /// Bind groups for the downsample pass (one per mip level).
    glass_blur_down_bind_groups: Vec<wgpu::BindGroup>,
    /// Bind groups for the upsample pass (one per mip level).
    glass_blur_up_bind_groups: Vec<wgpu::BindGroup>,
    /// Uniform buffer for blur parameters (src_size, mip_level, kernel_width, mode).
    glass_blur_uniform_buffer: wgpu::Buffer,
    /// Render pipeline for Kawase downsample passes.
    glass_blur_pipeline: wgpu::RenderPipeline,
    /// Render pipeline for Kawase upsample passes.
    glass_blur_upsample_pipeline: wgpu::RenderPipeline,
    /// Bind group layout for blur passes (uniform + texture + sampler).
    glass_blur_bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group layout for reading blur output in glass composite pass.
    glass_output_bind_group_layout: wgpu::BindGroupLayout,
    /// Current material state — draw calls are tagged with this material.
    current_draw_material: cvkg_core::DrawMaterial,
    /// Number of mip levels in the glass blur pyramid.
    blur_pyramid_mip_count: u32,
}

struct SurfaceContext {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    scene_texture: wgpu::TextureView,
    scene_bind_group: wgpu::BindGroup,
    scene_texture_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    blur_texture_a: wgpu::TextureView,
    blur_texture_b: wgpu::TextureView,
    blur_bind_group_a: wgpu::BindGroup,
    blur_bind_group_b: wgpu::BindGroup,
    blur_env_bind_group_a: wgpu::BindGroup,
    scale_factor: f32,
    sampler: wgpu::Sampler,
}

/// HeadlessContext — A rendering target for surface-less execution.
pub struct HeadlessContext {
    pub scene_texture: wgpu::TextureView,
    pub scene_bind_group: wgpu::BindGroup,
    pub scene_texture_bind_group: wgpu::BindGroup,
    pub depth_texture_view: wgpu::TextureView,
    pub blur_texture_a: wgpu::TextureView,
    pub blur_texture_b: wgpu::TextureView,
    pub blur_bind_group_a: wgpu::BindGroup,
    pub blur_bind_group_b: wgpu::BindGroup,
    pub blur_env_bind_group_a: wgpu::BindGroup,
    pub scale_factor: f32,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub output_texture: wgpu::Texture,
    pub output_view: wgpu::TextureView,
}

const MAX_VERTICES: usize = 100_000;
const MAX_INDICES: usize = 150_000;

impl SurtrRenderer {
    /// forge — Initializes the Surtr GPU renderer from a winit window.
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
        println!("[GPU] Requesting HighPerformance adapter...");

        let mut adapter = None;

        // Manual override for driver/adapter selection (e.g. forcing amdgpu-pro over RADV)
        if let Ok(filter) = std::env::var("WGPU_ADAPTER_NAME") {
            let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
            println!("[GPU] Available adapters:");
            for a in &adapters {
                let info = a.get_info();
                println!(
                    "  - Name: '{}' | Driver: '{}' | Backend: {:?}",
                    info.name, info.driver, info.backend
                );
            }

            adapter = adapters.into_iter().find(|a| {
                let info = a.get_info();
                let match_found = info.name.to_lowercase().contains(&filter.to_lowercase())
                    || info.driver.to_lowercase().contains(&filter.to_lowercase());
                if match_found {
                    println!(
                        "[GPU] Manual selection match: {} | Driver: {}",
                        info.name, info.driver
                    );
                }
                match_found
            });

            if adapter.is_some() {
                println!(
                    "[GPU] Forced adapter selection via WGPU_ADAPTER_NAME='{}'",
                    filter
                );
            } else {
                println!(
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
            println!(
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
            println!("[GPU] Hardware adapters failed, trying Software fallback...");
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
        println!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name, info.device_type, info.backend
        );
        println!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let mut required_features =
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY;
        if supports_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
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

    async fn forge_internal(
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Muspelheim Main Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_SRC)),
        });

        // Niflheim Bind Group Layout (for textures/samplers)
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
                        count: std::num::NonZeroU32::new(256),
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
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
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
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
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
                cache: None,
            });

        // Muspelheim Blur Pipelines (H and V)
        let blur_h_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Horizontal Blur"),
            layout: Some(&post_process_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blur_h"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let blur_v_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Muspelheim Vertical Blur"),
            layout: Some(&post_process_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_blur_v"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
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
                    // Additive blend: src + dst — glow lights up the scene
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Forge the Mega-Atlas (4096x4096 RGBA for production batching)
        let mega_atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Mega-Atlas"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mega_atlas_view_obj =
            mega_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
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
            &[0, 0, 0, 255],
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
            (0..256).map(|_| dummy_view.clone()).collect();
        texture_views_list[0] = mega_atlas_view_obj.clone();

        let views_refs: Vec<&wgpu::TextureView> = texture_views_list.iter().collect();
        let mega_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            label: Some("Mega-Atlas Bind Group"),
        });

        let dummy_views_refs: Vec<&wgpu::TextureView> = (0..256).map(|_| &dummy_view).collect();
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

        let mut texture_registry = std::collections::HashMap::new();
        let mut texture_bind_groups = Vec::new();

        texture_registry.insert("__mega_atlas".to_string(), 0);
        texture_bind_groups.push(mega_atlas_bind_group.clone());

        // Forge the Anvil (Buffers)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Vertex Anvil"),
            size: (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surtr Index Anvil"),
            size: (MAX_INDICES * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
            ));
        }

        let staging_belt = wgpu::util::StagingBelt::new((*device).clone(), 1024 * 1024);

        // Clone bind group layouts before they are moved into the Self struct
        let glass_blur_bind_group_layout = env_bind_group_layout.clone();
        let glass_output_bind_group_layout = env_bind_group_layout.clone();
        let glass_blur_pipeline = pipeline.clone();
        let glass_blur_upsample_pipeline = pipeline.clone();

        // Create glass blur pyramid resources (must be before Self struct, which moves device)
        let glass_blur_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glass Blur Pyramid"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let glass_blur_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Glass Blur Uniform"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),
            surfaces,
            current_window,
            headless_context,
            pipeline,
            bloom_extract_pipeline,
            blur_h_pipeline,
            blur_v_pipeline,
            composite_pipeline,
            env_bind_group_layout,
            text_engine: cvkg_runic_text::RunicTextEngine::default(),
            mega_atlas_tex,
            mega_atlas_view: mega_atlas_view_obj,
            _mega_atlas_sampler: text_sampler,
            mega_atlas_bind_group,
            text_cache: LruCache::new(NonZeroUsize::new(2048).unwrap()),
            atlas_packer: YggdrasilPacker::new(4096, 4096),
            image_uv_registry: LruCache::new(NonZeroUsize::new(256).unwrap()),
            texture_registry: LruCache::new(NonZeroUsize::new(255).unwrap()),
            texture_views: texture_views_list,
            dummy_sampler,
            svg_cache: LruCache::new(NonZeroUsize::new(128).unwrap()),
            svg_trees: LruCache::new(NonZeroUsize::new(128).unwrap()),
            filter_device: Some(device.clone()),
            filter_queue: Some(queue.clone()),
            filter_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("SVG Filter Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }),
            filter_engine: None,
            filter_batches: Vec::new(),
            dummy_texture_bind_group,
            dummy_env_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
            shared_elements: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            vertex_buffer,
            index_buffer,
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
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
            telemetry: cvkg_core::TelemetryData::default(),
            last_frame_start: std::time::Instant::now(),
            last_redraw_start: std::time::Instant::now(),
            frame_budget: cvkg_core::FrameBudget::default(),
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
            vnode_stack: Vec::new(),
            event_handlers: std::collections::HashMap::new(),
            staging_belt,
            staging_command_buffers: Vec::new(),
            // Backdrop Capture Architecture — Kawase Blur Pyramid
            glass_blur_texture,
            glass_blur_views: Vec::new(),
            glass_blur_down_bind_groups: Vec::new(),
            glass_blur_up_bind_groups: Vec::new(),
            glass_blur_uniform_buffer,
            glass_blur_pipeline,
            glass_blur_upsample_pipeline,
            glass_blur_bind_group_layout,
            glass_output_bind_group_layout,
            current_draw_material: cvkg_core::DrawMaterial::Opaque,
            blur_pyramid_mip_count: 1,
        }
    }

    fn rebuild_texture_array_bind_group(&mut self) {
        let views: Vec<&wgpu::TextureView> = self.texture_views.iter().collect();
        self.mega_atlas_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
    fn update_vram_telemetry(&mut self) {
        // Calculate Buffer VRAM
        let mut buffer_bytes = 0;
        buffer_bytes += (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64;
        buffer_bytes += (MAX_INDICES * std::mem::size_of::<u32>()) as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::ColorTheme>() as u64;
        buffer_bytes += std::mem::size_of::<cvkg_core::SceneUniforms>() as u64;
        self.vram_buffers_bytes = buffer_bytes;

        // Calculate Texture VRAM
        let mut texture_bytes = 0;
        texture_bytes += 4096 * 4096 * 4; // Mega Atlas (RGBA8)
        texture_bytes += 4; // Dummy (RGBA8)

        // Add Texture Array VRAM
        for _ in &self.texture_views {
            // Approximation: 1MB per texture
            texture_bytes += 1024 * 1024 * 4;
        }

        for ctx in self.surfaces.values() {
            let bpp = 4;
            let surface_bytes = (ctx.config.width * ctx.config.height * bpp) as u64;
            texture_bytes += surface_bytes * 3; // scene, blur_a, blur_b
            texture_bytes += (ctx.config.width * ctx.config.height * 4) as u64; // depth (Depth32Float)
        }

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

    /// resize — Reconfigures a specific surface and its internal textures.
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
                format: ctx.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };

            let scene_tex = self.device.create_texture(&texture_desc);
            ctx.scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_tex_a = self.device.create_texture(&texture_desc);
            ctx.blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_tex_b = self.device.create_texture(&texture_desc);
            ctx.blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

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
                (0..256).map(|_| &ctx.scene_texture).collect();
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

            let blur_views_a: Vec<&wgpu::TextureView> =
                (0..256).map(|_| &ctx.blur_texture_a).collect();
            ctx.blur_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Bind Group A Resize"),
            });

            let blur_views_b: Vec<&wgpu::TextureView> =
                (0..256).map(|_| &ctx.blur_texture_b).collect();
            ctx.blur_bind_group_b = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Bind Group B Resize"),
            });

            ctx.blur_env_bind_group_a = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.env_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ctx.blur_texture_a),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ctx.sampler),
                    },
                ],
                label: Some("Blur Env Bind Group A Resize"),
            });

            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Surtr Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            ctx.depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    /// begin_frame_headless — Strike the flaming sword to begin a new GPU frame for headless rendering.
    pub fn begin_frame_headless(&mut self) -> wgpu::CommandEncoder {
        self.current_window = None;
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.filter_batches.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.current_z = 0.0;
        self.vnode_stack.clear();
        self.event_handlers.clear();

        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

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

    /// begin_frame — Strike the flaming sword to begin a new GPU frame for a specific window.
    pub fn begin_frame(&mut self, window_id: winit::window::WindowId) -> wgpu::CommandEncoder {
        // Skuld: Read the timestamps from the previous frame
        if let Some(rb) = &self.skuld_read_buffer {
            let slice = rb.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());

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
                    // println!("[Skuld] GPU Time: {} ms", self.last_gpu_time_ns as f64 / 1_000_000.0);
                }
            }
        }

        self.staging_belt.recall();
        self.current_window = Some(window_id);
        self.vertices.clear();
        self.indices.clear();
        self.draw_calls.clear();
        self.shared_elements.clear();
        self.current_texture_id = None;
        self.opacity_stack = vec![1.0];
        self.clip_stack.clear();
        self.slice_stack.clear();
        self.transform_stack.clear();
        self.current_z = 0.0;
        self.vnode_stack.clear();
        self.event_handlers.clear();

        self.last_frame_start = std::time::Instant::now();
        self.telemetry.draw_calls = 0;
        self.telemetry.vertices = 0;

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

    /// register_window — Attaches a new OS window to the shared GPU context.
    pub fn register_window(&mut self, window: Arc<winit::window::Window>) {
        let size = window.inner_size();
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.device, &config);

        let ctx = Self::create_surface_context(
            &self.device,
            surface,
            config,
            &self.env_bind_group_layout,
            &self.texture_bind_group_layout,
            window.scale_factor() as f32,
        );

        self.surfaces.insert(window.id(), ctx);
    }

    fn create_headless_context(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
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
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_width = (width / 2).max(1);
        let blur_height = (height / 2).max(1);
        let blur_texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Blur Texture"),
            size: wgpu::Extent3d {
                width: blur_width,
                height: blur_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let blur_tex_a = device.create_texture(&blur_texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&blur_texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

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

        let scene_views: Vec<&wgpu::TextureView> = (0..256).map(|_| &scene_texture).collect();
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

        let blur_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_a).collect();
        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Bind Group A"),
        });

        let blur_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_b).collect();
        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Bind Group B"),
        });

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Headless Blur Env Bind Group A"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Headless Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
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

        HeadlessContext {
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_texture_a,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            scale_factor: 1.0,
            sampler,
            width,
            height,
            output_texture,
            output_view,
        }
    }

    fn create_surface_context(
        device: &wgpu::Device,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        env_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        scale_factor: f32,
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
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let scene_tex = device.create_texture(&texture_desc);
        let scene_texture = scene_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_width = (width / 2).max(1);
        let blur_height = (height / 2).max(1);
        let blur_texture_desc = wgpu::TextureDescriptor {
            label: Some("Surtr Blur Texture"),
            size: wgpu::Extent3d {
                width: blur_width,
                height: blur_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let blur_tex_a = device.create_texture(&blur_texture_desc);
        let blur_texture_a = blur_tex_a.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_tex_b = device.create_texture(&blur_texture_desc);
        let blur_texture_b = blur_tex_b.create_view(&wgpu::TextureViewDescriptor::default());

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

        let scene_views: Vec<&wgpu::TextureView> = (0..256).map(|_| &scene_texture).collect();
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

        let blur_views_a: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_a).collect();
        let blur_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Bind Group A"),
        });

        let blur_views_b: Vec<&wgpu::TextureView> = (0..256).map(|_| &blur_texture_b).collect();
        let blur_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&blur_views_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Bind Group B"),
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surtr Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_env_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blur_texture_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blur Env Bind Group A"),
        });

        SurfaceContext {
            surface,
            config,
            scene_texture,
            scene_bind_group,
            scene_texture_bind_group,
            depth_texture_view,
            blur_texture_a,
            blur_texture_b,
            blur_bind_group_a,
            blur_bind_group_b,
            blur_env_bind_group_a,
            scale_factor,
            sampler,
        }
    }

    pub fn reset_time(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    /// reclaim_vram — Atomic recycling of the Mega-Atlas and all associated caches.
    /// This prevents OOM and silent failures by quenching the atlas when full.
    pub fn reclaim_vram(&mut self) {
        log::warn!("[GPU] Yggdrasil Compaction: Compacting Mega-Atlas...");

        let new_mega_atlas_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Yggdrasil Mega-Atlas (Compacted)"),
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

        let mut new_packer = YggdrasilPacker::new(4096, 4096);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Atlas Compaction Encoder"),
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
                            texture: &self.mega_atlas_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: old_x_px,
                                y: old_y_px,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: &new_mega_atlas_tex,
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

        let text_entries: Vec<(u64, (Rect, f32, f32))> =
            self.text_cache.iter().map(|(k, v)| (*k, *v)).collect();
        for (hash, (old_uv, w_f, h_f)) in text_entries {
            let w_px = (old_uv.width * 4096.0).round() as u32;
            let h_px = (old_uv.height * 4096.0).round() as u32;
            let old_x_px = (old_uv.x * 4096.0).round() as u32;
            let old_y_px = (old_uv.y * 4096.0).round() as u32;

            if let Some((new_x, new_y)) = new_packer.pack(w_px, h_px) {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.mega_atlas_tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: old_x_px,
                            y: old_y_px,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &new_mega_atlas_tex,
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
                self.text_cache.put(hash, (new_uv, w_f, h_f));
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.mega_atlas_tex = new_mega_atlas_tex;
        let mega_atlas_view_obj = self
            .mega_atlas_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.texture_views[0] = mega_atlas_view_obj.clone();

        self.rebuild_texture_array_bind_group();

        if !self.texture_bind_groups.is_empty() {
            self.texture_bind_groups[0] = self.mega_atlas_bind_group.clone();
        }

        self.atlas_packer = new_packer;
        self.telemetry.vram_exhausted = false;
    }

    fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        mode: u32,
    ) {
        // High-Fidelity Variable Particle Density
        let count = (pieces as f32).sqrt().ceil() as u32;
        let dw = rect.width / count as f32;
        let dh = rect.height / count as f32;

        let c = self.apply_opacity(color);

        for y in 0..count {
            for x in 0..count {
                let shard_rect = Rect {
                    x: rect.x + x as f32 * dw,
                    y: rect.y + y as f32 * dh,
                    width: dw,
                    height: dh,
                };

                let uv = Rect {
                    x: x as f32 / count as f32,
                    y: y as f32 / count as f32,
                    width: 1.0 / count as f32,
                    height: 1.0 / count as f32,
                };

                self.fill_rect_with_full_params(shard_rect, c, mode, None, force, uv);
            }
        }
    }

    fn recursive_bolt(&mut self, from: [f32; 2], to: [f32; 2], depth: u32, color: [f32; 4]) {
        if depth == 0 {
            self.draw_lightning_segment(from, to, color);
            return;
        }

        let mid_x = (from[0] + to[0]) * 0.5;
        let mid_y = (from[1] + to[1]) * 0.5;

        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();

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

    fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
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

    fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        mode: u32,
        uv_rect: Rect,
    ) {
        let scissor = self.clip_stack.last().copied();
        let texture_id = None; // Oriented quads like lightning don't use textures yet

        if self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || self.draw_calls.last().unwrap().scissor_rect != scissor
        {
            self.current_texture_id = texture_id;
            self.draw_calls.push(DrawCall {
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                material: if mode == 7 {
                    cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
                } else if mode == 6 {
                    cvkg_core::DrawMaterial::TopUI
                } else {
                    cvkg_core::DrawMaterial::Opaque
                },
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
                mode,
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                screen,
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
                translation,
                scale: scale_transform,
                rotation,
                tex_index: 0,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }
    fn get_texture_id(&mut self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode — Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color,
            mode,
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

    fn fill_rect_with_full_params(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
    ) {
        // If a shadow is active, draw it first
        if let Some(shadow) = self.shadow_stack.last().copied()
            && shadow.color[3] > 0.001
        {
            Renderer::draw_drop_shadow(
                self,
                rect,
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
            rect, color, mode, texture_id, radius, uv_rect, slice,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn fill_rect_with_full_params_and_slice(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        mode: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
    ) {
        let scissor = self.clip_stack.last().copied();

        let material = if mode == 7 {
            cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
        } else if mode == 6 {
            cvkg_core::DrawMaterial::TopUI
        } else {
            self.current_draw_material
        };

        // Batching: check if we need to start a new DrawCall
        // With Texture Array, we no longer need to break batches when the texture changes,
        // as long as they are all part of the same array bind group (Group 0).
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = Some(0); // All textures are now in the binding array at Group 0
            self.draw_calls.push(DrawCall {
                texture_id: self.current_texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                material,
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

        let (translation, scale_transform, rotation, _, _) = self.current_transform();

        let tex_index = texture_id.unwrap_or(0);

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color,
            mode,
            radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color,
            mode,
            radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color,
            mode,
            radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color,
            mode,
            radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            screen,
            clip,
            translation,
            scale: scale_transform,
            rotation,
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

    /// end_frame — Quench the blade by submitting the full Muspelheim multi-pass effect.
    pub fn end_frame(&mut self, mut encoder: wgpu::CommandEncoder) {
        let (
            surface_texture,
            target_view,
            ctx_scene_texture,
            ctx_depth_texture_view,
            ctx_blur_env_bind_group_a,
            ctx_scene_texture_bind_group,
            ctx_blur_texture_a,
            ctx_blur_texture_b,
            _ctx_sampler,
            ctx_blur_bind_group_a,
            ctx_blur_bind_group_b,
            scale,
        ) = if let Some(window_id) = self.current_window {
            let ctx = self
                .surfaces
                .get(&window_id)
                .expect("Missing surface context");
            let frame = match ctx.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) => t,
                wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                    ctx.surface.configure(&self.device, &ctx.config);
                    t
                }
                _ => {
                    log::warn!("[GPU] Surface texture acquisition failed, reconfiguring surface");
                    ctx.surface.configure(&self.device, &ctx.config);
                    return;
                }
            };
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            (
                Some(frame),
                view,
                &ctx.scene_texture,
                &ctx.depth_texture_view,
                &ctx.blur_env_bind_group_a,
                &ctx.scene_texture_bind_group,
                &ctx.blur_texture_a,
                &ctx.blur_texture_b,
                &ctx.sampler,
                &ctx.blur_bind_group_a,
                &ctx.blur_bind_group_b,
                ctx.scale_factor,
            )
        } else {
            let ctx = self
                .headless_context
                .as_ref()
                .expect("No headless context for end_frame");
            (
                None,
                ctx.output_view.clone(),
                &ctx.scene_texture,
                &ctx.depth_texture_view,
                &ctx.blur_env_bind_group_a,
                &ctx.scene_texture_bind_group,
                &ctx.blur_texture_a,
                &ctx.blur_texture_b,
                &ctx.sampler,
                &ctx.blur_bind_group_a,
                &ctx.blur_bind_group_b,
                self.current_scale_factor(),
            )
        };

        // ── Pass 1: Opaque Background & Atmosphere ──────────────────────────
        {
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P1 Opaque Background"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_scene_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: ctx_depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0), // Reversed-Z
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: self.skuld_queries.as_ref().map(|q| {
                    wgpu::RenderPassTimestampWrites {
                        query_set: q,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: None,
                    }
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 1a. Background Atmosphere
            p.set_pipeline(&self.background_pipeline);
            p.set_bind_group(0, &self.dummy_texture_bind_group, &[]);
            p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]); // Use previous frame's blur for background depth
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);

            // 1b. Opaque Main Elements (non-glass, non-ui)
            if !self.draw_calls.is_empty() {
                p.set_pipeline(&self.pipeline);
                p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);

                for call in self
                    .draw_calls
                    .iter()
                    .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Opaque))
                {
                    let bg = if let Some(id) = call.texture_id {
                        if id == 0 {
                            &self.mega_atlas_bind_group
                        } else {
                            self.texture_bind_groups
                                .get(id as usize)
                                .unwrap_or(&self.dummy_texture_bind_group)
                        }
                    } else {
                        &self.dummy_texture_bind_group
                    };
                    p.set_bind_group(0, bg, &[]);
                    p.draw_indexed(
                        call.index_start..call.index_start + call.index_count,
                        0,
                        0..1,
                    );
                    self.telemetry.draw_calls += 1;
                    self.telemetry.vertices += call.index_count;
                }
            }
        }

        // ── Pass 2: Backdrop Blur (Bifrost) ──────────────────────────────────
        // Capture the background into blur_texture_b
        {
            // First extract into texture_a
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr Blur Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline); // Use extract as a direct copy for now
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
        }

        let blur_iters: u32 = 4;
        for _i in 0..blur_iters {
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur H"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_b, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
        }

        // 1. Finalize the PRE-parallel work (Background & Atmosphere)
        self.staging_command_buffers.push(encoder.finish());

        let rt_w = self.current_width() as i32;
        let rt_h = self.current_height() as i32;

        // 2. Parallel Encoding Phase: Glass & UI ─────────────────────────────
        // We utilize rayon to record these independent passes in parallel.
        let (glass_cb, ui_cb) = rayon::join(
            || {
                let mut glass_encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Parallel Glass Encoder"),
                        });
                {
                    let mut p = glass_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Surtr P3 Liquid Glass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: ctx_scene_texture,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: ctx_depth_texture_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });

                    p.set_pipeline(&self.pipeline);
                    p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]);
                    p.set_bind_group(2, &self.berserker_bind_group, &[]);

                    for call in self
                        .draw_calls
                        .iter()
                        .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }))
                    {
                        let bg = if let Some(id) = call.texture_id {
                            if id == 0 {
                                &self.mega_atlas_bind_group
                            } else {
                                self.texture_bind_groups
                                    .get(id as usize)
                                    .unwrap_or(&self.dummy_texture_bind_group)
                            }
                        } else {
                            &self.dummy_texture_bind_group
                        };
                        p.set_bind_group(0, bg, &[]);
                        if let Some(rect) = call.scissor_rect {
                            // Scissor rect clamping logic:
                            // wgpu validation requires that the scissor rect is entirely contained within
                            // the physical render target dimensions and has a non-zero area (width > 0, height > 0).
                            // We compute the physical boundaries using the current render target size and scale factor,
                            // intersect/clamp them to the render target viewport, and fallback to a minimal 1x1 region
                            // if the intersection results in zero/negative area.
                            if rt_w > 0 && rt_h > 0 {
                                let x1 = (rect.x * scale).round() as i32;
                                let y1 = (rect.y * scale).round() as i32;
                                let x2 = ((rect.x + rect.width) * scale).round() as i32;
                                let y2 = ((rect.y + rect.height) * scale).round() as i32;

                                let x1_clamped = x1.clamp(0, rt_w);
                                let y1_clamped = y1.clamp(0, rt_h);
                                let x2_clamped = x2.clamp(0, rt_w);
                                let y2_clamped = y2.clamp(0, rt_h);

                                let w = x2_clamped - x1_clamped;
                                let h = y2_clamped - y1_clamped;

                                if w > 0 && h > 0 {
                                    p.set_scissor_rect(
                                        x1_clamped as u32,
                                        y1_clamped as u32,
                                        w as u32,
                                        h as u32,
                                    );
                                } else {
                                    p.set_scissor_rect(0, 0, 1, 1);
                                }
                            }
                        }
                        p.draw_indexed(
                            call.index_start..call.index_start + call.index_count,
                            0,
                            0..1,
                        );
                    }
                }
                glass_encoder.finish()
            },
            || {
                let mut ui_encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Parallel UI Encoder"),
                        });
                {
                    let mut p = ui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Surtr P4 UI Layer"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: ctx_scene_texture,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: ctx_depth_texture_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });

                    p.set_pipeline(&self.pipeline);
                    p.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    p.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                    p.set_bind_group(2, &self.berserker_bind_group, &[]);

                    for call in self
                        .draw_calls
                        .iter()
                        .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::TopUI))
                    {
                        let bg = if let Some(id) = call.texture_id {
                            if id == 0 {
                                &self.mega_atlas_bind_group
                            } else {
                                self.texture_bind_groups
                                    .get(id as usize)
                                    .unwrap_or(&self.dummy_texture_bind_group)
                            }
                        } else {
                            &self.dummy_texture_bind_group
                        };
                        p.set_bind_group(0, bg, &[]);
                        if let Some(rect) = call.scissor_rect {
                            // Scissor rect clamping logic:
                            // wgpu validation requires that the scissor rect is entirely contained within
                            // the physical render target dimensions and has a non-zero area (width > 0, height > 0).
                            // We compute the physical boundaries using the current render target size and scale factor,
                            // intersect/clamp them to the render target viewport, and fallback to a minimal 1x1 region
                            // if the intersection results in zero/negative area.
                            if rt_w > 0 && rt_h > 0 {
                                let x1 = (rect.x * scale).round() as i32;
                                let y1 = (rect.y * scale).round() as i32;
                                let x2 = ((rect.x + rect.width) * scale).round() as i32;
                                let y2 = ((rect.y + rect.height) * scale).round() as i32;

                                let x1_clamped = x1.clamp(0, rt_w);
                                let y1_clamped = y1.clamp(0, rt_h);
                                let x2_clamped = x2.clamp(0, rt_w);
                                let y2_clamped = y2.clamp(0, rt_h);

                                let w = x2_clamped - x1_clamped;
                                let h = y2_clamped - y1_clamped;

                                if w > 0 && h > 0 {
                                    p.set_scissor_rect(
                                        x1_clamped as u32,
                                        y1_clamped as u32,
                                        w as u32,
                                        h as u32,
                                    );
                                } else {
                                    p.set_scissor_rect(0, 0, 1, 1);
                                }
                            }
                        }
                        p.draw_indexed(
                            call.index_start..call.index_start + call.index_count,
                            0,
                            0..1,
                        );
                    }
                }
                ui_encoder.finish()
            },
        );

        self.staging_command_buffers.push(glass_cb);
        self.staging_command_buffers.push(ui_cb);

        // Update telemetry for parallel work
        let glass_calls = self
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }))
            .count();
        let glass_verts: u32 = self
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::Glass { .. }))
            .map(|c| c.index_count)
            .sum();
        let ui_calls = self
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::TopUI))
            .count();
        let ui_verts: u32 = self
            .draw_calls
            .iter()
            .filter(|c| matches!(c.material, cvkg_core::DrawMaterial::TopUI))
            .map(|c| c.index_count)
            .sum();
        self.telemetry.draw_calls += (glass_calls + ui_calls) as u32;
        self.telemetry.vertices += glass_verts + ui_verts;

        // 3. Start POST-parallel work (Bloom & Composite)
        let mut post_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surtr Post-Process Encoder"),
                });

        // ── Pass 5: Bloom Extract (Complete Scene) ──────────────────────────
        {
            let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr Bloom Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ctx_blur_texture_a,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            p.set_pipeline(&self.bloom_extract_pipeline);
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
        }

        // ── Pass 6: Blur Bloom ──────────────────────────────────────────────
        for _ in 0..2 {
            {
                let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Bloom Blur H"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_b,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_h_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_a, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
            {
                let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Bloom Blur V"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: ctx_blur_texture_a,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                p.set_pipeline(&self.blur_v_pipeline);
                p.set_bind_group(0, ctx_blur_bind_group_b, &[]);
                p.set_bind_group(1, &self.dummy_env_bind_group, &[]);
                p.set_bind_group(2, &self.berserker_bind_group, &[]);
                p.draw(0..6, 0..1);
            }
        }

        // ── Pass 7: Composite & Tone Map ────────────────────────────────────
        {
            let mut p = post_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Surtr P7 Composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: self.skuld_queries.as_ref().map(|q| {
                    wgpu::RenderPassTimestampWrites {
                        query_set: q,
                        beginning_of_pass_write_index: None,
                        end_of_pass_write_index: Some(1),
                    }
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&self.composite_pipeline);
            p.set_bind_group(0, ctx_scene_texture_bind_group, &[]);
            p.set_bind_group(1, ctx_blur_env_bind_group_a, &[]);
            p.set_bind_group(2, &self.berserker_bind_group, &[]);
            p.draw(0..6, 0..1);
            self.telemetry.draw_calls += 1;
        }

        self.telemetry.frame_time_ms = self.last_frame_start.elapsed().as_secs_f32() * 1000.0;
        self.update_vram_telemetry();

        // Skuld: Resolve timestamps
        if let (Some(q), Some(b), Some(rb)) = (
            &self.skuld_queries,
            &self.skuld_buffer,
            &self.skuld_read_buffer,
        ) {
            post_encoder.resolve_query_set(q, 0..2, b, 0);
            post_encoder.copy_buffer_to_buffer(b, 0, rb, 0, 16);
        }

        // Finalize post-parallel work
        self.staging_command_buffers.push(post_encoder.finish());

        // Atomic submission: all blocks in correct sequence
        let cmds = std::mem::take(&mut self.staging_command_buffers);
        self.queue.submit(cmds);
        if let Some(f) = surface_texture {
            f.present();
        }
    }
}

impl cvkg_core::ElapsedTime for SurtrRenderer {
    fn delta_time(&self) -> f32 {
        self.current_scene.delta_time
    }

    fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

impl SurtrRenderer {
    /// load_image_to_atlas — Packs a raw asset into the Mega-Atlas.
    /// This is used for common icons to enable aggressive batching (1 draw call).
    pub fn load_image_to_atlas(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to load image {} to atlas: {}", name, e);
                return;
            }
        };
        let (width, height) = img.dimensions();

        // Pack into atlas
        if let Some((x, y)) = self.atlas_packer.pack(width, height) {
            let uv_rect = Rect {
                x: x as f32 / 4096.0,
                y: y as f32 / 4096.0,
                width: width as f32 / 4096.0,
                height: height as f32 / 4096.0,
            };

            // Upload to GPU
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.mega_atlas_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            self.image_uv_registry.put(name.to_string(), uv_rect);
            self.texture_registry.put(name.to_string(), 0); // Index 0 is the dummy white texture
            log::debug!(
                "[Surtr] Packed '{}' into Mega-Atlas at ({}, {})",
                name,
                x,
                y
            );
        } else {
            log::warn!(
                "ATLAS_FULL: Failed to pack '{}' into Mega-Atlas. Falling back to Texture Array.",
                name
            );
            self.load_image(name, data);
        }
    }

    /// Shapes a text string using a predefined system font stack.
    ///
    /// # Contract
    /// Evaluates text shaping with fallbacks: queries "SF Pro Text", "SF Pro", "Inter",
    /// "Helvetica Neue", "Helvetica", "Arial", and defaults back to "sans-serif".
    /// This ensures visual typographic consistency across platforms where specific
    /// branding faces may or may not be installed.
    fn shape_text_with_stack(&mut self, text: &str, size: f32) -> cvkg_runic_text::ShapedText {
        let mut style = cvkg_runic_text::TextStyle::new("SF Pro Text", size);
        style.fallback_families = vec![
            "SF Pro".to_string(),
            "Inter".to_string(),
            "Helvetica Neue".to_string(),
            "Helvetica".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];
        let spans = vec![cvkg_runic_text::TextSpan::new(text, style)];
        self.text_engine
            .shape_layout(
                &spans,
                None,
                cvkg_runic_text::TextAlign::Start,
                cvkg_runic_text::TextOverflow::WordWrap,
            )
            .unwrap_or_else(|_| cvkg_runic_text::ShapedText {
                glyphs: Vec::new(),
                lines: Vec::new(),
                width: 0.0,
                height: 0.0,
                text: text.to_string(),
                spans: Vec::new(),
                has_rtl: false,
                ascent: 0.0,
                descent: 0.0,
                line_gap: 0.0,
                grapheme_boundaries: vec![],
            })
    }
}

impl cvkg_core::Renderer for SurtrRenderer {
    fn is_over_budget(&self) -> bool {
        self.frame_budget.allow_degradation
            && self.last_frame_start.elapsed().as_secs_f32() * 1000.0 > self.frame_budget.target_ms
    }

    /// fill_rect — Standard rectangle drawing method.
    fn prewarm_vram(&mut self, assets: Vec<(String, Vec<u8>)>) {
        log::info!(
            "[Surtr] Pre-warming Mega-Atlas with {} assets...",
            assets.len()
        );
        for (name, data) in assets {
            self.load_image_to_atlas(&name, &data);
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_mode(rect, self.apply_opacity(color), 0, None);
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            3,
            None,
            radius,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            4,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    fn draw_3d_cube(&mut self, rect: Rect, color: [f32; 4], rotation: [f32; 3]) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(color),
            21,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            [rotation[0], rotation[1], rotation[2], 0.0],
        );
    }

    fn bifrost(&mut self, rect: Rect, blur: f32, _saturation: f32, opacity: f32) {
        // Calculate screen-space UVs for high-fidelity global refraction
        let screen_uv = Rect {
            x: rect.x / self.current_width() as f32,
            y: rect.y / self.current_height() as f32,
            width: rect.width / self.current_width() as f32,
            height: rect.height / self.current_height() as f32,
        };
        // Use mode 7 for high-fidelity background blur sampling
        // Use the blur parameter as corner radius for the glass panel
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, opacity], 7, None, blur, screen_uv);
    }

    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        // Create neon glow effect using additive blending
        // This renders a glowing aura around the element
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        let max_dim = rect.width.max(rect.height) * 0.5 + radius;

        // Draw expanding glow layers
        for i in 0..8 {
            let alpha = intensity / (i as f32 + 1.0) * 0.3;
            let glow_color = [color[0], color[1], color[2], alpha];
            self.fill_rect_with_mode(
                Rect {
                    x: center_x - max_dim - i as f32 * 2.0,
                    y: center_y - max_dim - i as f32 * 2.0,
                    width: max_dim * 2.0 + i as f32 * 4.0,
                    height: max_dim * 2.0 + i as f32 * 4.0,
                },
                glow_color,
                8, // Mode for additive blending
                None,
            );
        }
    }

    /// Renders a dynamic glowing hover boundary field around a hit target.
    ///
    /// # Contract
    /// Expands the bounding box of the visual target by `radius` to establish
    /// a continuous proximity glow. Uses blending mode 18 (GPU drop shadow/glow)
    /// to rasterize the glow with specialized radius-to-margin uv coordinate mappings.
    fn mani_glow(&mut self, rect: Rect, color: [f32; 4], radius: f32) {
        let margin = radius;
        let glow_rect = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + 2.0 * margin,
            height: rect.height + 2.0 * margin,
        };
        let uv_rect = Rect {
            x: margin,
            y: radius,
            width: 0.0,
            height: 0.0,
        };
        self.fill_rect_with_full_params(
            glow_rect,
            self.apply_opacity(color),
            18,
            None,
            8.0,
            uv_rect,
        );
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        let c = self.apply_opacity(color);
        let hw = stroke_width;
        // Top, bottom, left, right edge bars
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y + rect.height - hw,
                width: rect.width,
                height: hw,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
        self.fill_rect_with_mode(
            Rect {
                x: rect.x + rect.width - hw,
                y: rect.y,
                width: hw,
                height: rect.height,
            },
            c,
            1,
            None,
        );
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            17,
            None,
            radius,
            Rect {
                x: stroke_width,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        // Tessellate an ellipse stroke using Lyon's StrokeTessellator.
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let rx = rect.width / 2.0;
        let ry = rect.height / 2.0;

        // Build an ellipse path using Lyon
        let mut builder = lyon::path::Path::builder();
        if rx > 0.0 && ry > 0.0 {
            // Approximate ellipse with 64 segments
            let segments = 64;
            for i in 0..segments {
                let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
                let x = cx + rx * angle.cos();
                let y = cy + ry * angle.sin();
                if i == 0 {
                    builder.begin(lyon::math::point(x, y));
                } else {
                    builder.line_to(lyon::math::point(x, y));
                }
            }
            builder.close();
        }
        let path = builder.build();
        self.stroke_path(&path, color, stroke_width);
    }

    fn draw_linear_gradient(
        &mut self,
        rect: Rect,
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32,
    ) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(start_color),
            15,
            None,
            0.0,
            Rect {
                x: angle,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            end_color,
        );
    }

    fn draw_radial_gradient(&mut self, rect: Rect, inner_color: [f32; 4], outer_color: [f32; 4]) {
        self.fill_rect_with_full_params_and_slice(
            rect,
            self.apply_opacity(inner_color),
            16,
            None,
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            outer_color,
        );
    }

    fn draw_drop_shadow(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        blur: f32,
        spread: f32,
    ) {
        let margin = blur + spread;
        let inflated = Rect {
            x: rect.x - margin,
            y: rect.y - margin,
            width: rect.width + margin * 2.0,
            height: rect.height + margin * 2.0,
        };
        // uv.x = total margin (for SDF offset), uv.y = blur width (for falloff)
        self.fill_rect_with_full_params(
            inflated,
            self.apply_opacity(color),
            18,
            None,
            radius,
            Rect {
                x: margin,
                y: blur,
                width: 0.0,
                height: 0.0,
            },
        );
    }

    fn stroke_dashed_rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        width: f32,
        dash: f32,
        gap: f32,
    ) {
        self.fill_rect_with_full_params(
            rect,
            self.apply_opacity(color),
            19,
            None,
            radius,
            Rect {
                x: width,
                y: dash,
                width: gap,
                height: 0.0,
            },
        );
    }

    fn draw_9slice(
        &mut self,
        image_name: &str,
        rect: Rect,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) {
        let c = self.apply_opacity([1.0, 1.0, 1.0, 1.0]);
        let tid = self.get_texture_id(image_name);
        self.fill_rect_with_full_params(
            rect,
            c,
            20,
            tid,
            bottom,
            Rect {
                x: left,
                y: top,
                width: right,
                height: 0.0,
            },
        );
    }

    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        let c = self.apply_opacity(color);
        let tid = self.get_texture_id("__mega_atlas");

        self.fill_rect_with_mode(
            Rect {
                x: (x1 + x2) / 2.0 - len / 2.0,
                y: (y1 + y2) / 2.0 - stroke_width / 2.0,
                width: len,
                height: stroke_width,
            },
            c,
            1, // Gungnir Mode for glowing lines
            tid,
        );
    }

    fn draw_image(&mut self, image_name: &str, rect: Rect) {
        let tid = self
            .get_texture_id(image_name)
            .or_else(|| self.get_texture_id("__mega_atlas"));
        let uv_rect = self
            .image_uv_registry
            .get(image_name)
            .copied()
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            });
        self.fill_rect_with_full_params(rect, [1.0, 1.0, 1.0, 1.0], 2, tid, 0.0, uv_rect);
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // High-DPI: Shape and rasterize at the physical scale factor for maximum sharpness.
        let scaled_size = size * self.current_scale_factor();
        let shaped = self.shape_text_with_stack(text, scaled_size);
        let c = self.apply_opacity(color);

        for glyph in shaped.glyphs {
            let cache_key = glyph.cache_key;

            let (uv_rect, w, h) = if let Some(info) = self.text_cache.get(&cache_key) {
                *info
            } else {
                if let Some(image) = self.text_engine.rasterize(cache_key) {
                    let gw = image.width;
                    let gh = image.height;

                    let pack_res = self.atlas_packer.pack(gw, gh);
                    let (nx, ny) = if let Some(pos) = pack_res {
                        pos
                    } else {
                        // RECLAIM & RETRY: Atlas is full, quench the forge and try again.
                        self.reclaim_vram();
                        self.atlas_packer.pack(gw, gh).unwrap_or((0, 0))
                    };

                    let mut rgba_data = Vec::with_capacity((gw * gh * 4) as usize);
                    for alpha in &image.data {
                        rgba_data.push(255);
                        rgba_data.push(255);
                        rgba_data.push(255);
                        rgba_data.push(*alpha);
                    }

                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_atlas_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(gw * 4),
                            rows_per_image: Some(gh),
                        },
                        wgpu::Extent3d {
                            width: gw,
                            height: gh,
                            depth_or_array_layers: 1,
                        },
                    );

                    let info = (
                        Rect {
                            x: nx as f32 / 4096.0,
                            y: ny as f32 / 4096.0,
                            width: gw as f32 / 4096.0,
                            height: gh as f32 / 4096.0,
                        },
                        gw as f32,
                        gh as f32,
                    );
                    self.text_cache.put(cache_key, info);
                    info
                } else {
                    (Rect::zero(), 0.0, 0.0)
                }
            };

            if w > 0.0 {
                // Map physical glyph dimensions and positions back to logical units
                // so the logical orthographic projection matrix places them correctly.
                let glyph_rect = Rect {
                    x: x + glyph.x / self.current_scale_factor(),
                    y: y + glyph.y / self.current_scale_factor(),
                    width: w / self.current_scale_factor(),
                    height: h / self.current_scale_factor(),
                };
                let tid = self.get_texture_id("__mega_atlas");
                self.fill_rect_with_full_params(glyph_rect, c, 6, tid, 0.0, uv_rect);
            }
        }
    }

    /// measure_text — Calculates the dimensions of a text string without rendering.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let shaped = self.shape_text_with_stack(text, size);
        (shaped.width, shaped.height)
    }

    fn shape_rich_text(
        &mut self,
        spans: &[cvkg_runic_text::TextSpan],
        max_width: Option<f32>,
        align: cvkg_runic_text::TextAlign,
        overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        let sf = self.current_scale_factor();
        let mut scaled_spans = spans.to_vec();
        for span in &mut scaled_spans {
            span.style.font_size *= sf;
            if span.style.fallback_families.is_empty() {
                span.style.fallback_families = vec![
                    "SF Pro".to_string(),
                    "Inter".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ];
            }
        }
        let scaled_max_width = max_width.map(|w| w * sf);
        self.text_engine
            .shape_layout(&scaled_spans, scaled_max_width, align, overflow)
            .ok()
    }

    fn draw_shaped_text(&mut self, shaped: &cvkg_runic_text::ShapedText, x: f32, y: f32) {
        for glyph in &shaped.glyphs {
            let byte_idx = shaped
                .grapheme_boundaries
                .get(glyph.cluster as usize)
                .copied()
                .unwrap_or(0);
            let mut span_color = [1.0, 1.0, 1.0, 1.0];
            for span in &shaped.spans {
                if byte_idx >= span.byte_offset && byte_idx < span.byte_offset + span.text.len() {
                    span_color = [
                        span.style.color[0] as f32 / 255.0,
                        span.style.color[1] as f32 / 255.0,
                        span.style.color[2] as f32 / 255.0,
                        span.style.color[3] as f32 / 255.0,
                    ];
                    break;
                }
            }
            let c = self.apply_opacity(span_color);

            let cache_key = glyph.cache_key;
            let (uv_rect, w, h) = if let Some(info) = self.text_cache.get(&cache_key) {
                *info
            } else {
                if let Some(image) = self.text_engine.rasterize(cache_key) {
                    let gw = image.width;
                    let gh = image.height;

                    let pack_res = self.atlas_packer.pack(gw, gh);
                    let (nx, ny) = if let Some(pos) = pack_res {
                        pos
                    } else {
                        self.reclaim_vram();
                        self.atlas_packer.pack(gw, gh).unwrap_or((0, 0))
                    };

                    let mut rgba_data = Vec::with_capacity((gw * gh * 4) as usize);
                    for alpha in &image.data {
                        rgba_data.push(255);
                        rgba_data.push(255);
                        rgba_data.push(255);
                        rgba_data.push(*alpha);
                    }

                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.mega_atlas_tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x: nx, y: ny, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &rgba_data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(gw * 4),
                            rows_per_image: Some(gh),
                        },
                        wgpu::Extent3d {
                            width: gw,
                            height: gh,
                            depth_or_array_layers: 1,
                        },
                    );

                    let info = (
                        Rect {
                            x: nx as f32 / 4096.0,
                            y: ny as f32 / 4096.0,
                            width: gw as f32 / 4096.0,
                            height: gh as f32 / 4096.0,
                        },
                        gw as f32,
                        gh as f32,
                    );
                    self.text_cache.put(cache_key, info);
                    info
                } else {
                    (Rect::zero(), 0.0, 0.0)
                }
            };

            if w > 0.0 {
                let sf = self.current_scale_factor();
                let glyph_rect = Rect {
                    x: x + glyph.x / sf,
                    y: y + glyph.y / sf,
                    width: w / sf,
                    height: h / sf,
                };
                let tid = self.get_texture_id("__mega_atlas");
                self.fill_rect_with_full_params(glyph_rect, c, 6, tid, 0.0, uv_rect);
            }
        }
    }

    fn draw_texture(&mut self, texture_id: u32, rect: Rect) {
        self.fill_rect_with_full_params(
            rect,
            [1.0, 1.0, 1.0, 1.0],
            2,
            Some(texture_id),
            0.0,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
    }

    /// load_image — Proactively pushes a raw asset into the Mega-Atlas.
    /// load_image — Proactively pushes a raw asset into the Texture Array.
    fn load_image(&mut self, name: &str, data: &[u8]) {
        if self.image_uv_registry.contains(name) {
            return;
        }
        let img_result = image::load_from_memory(data);
        let img = match img_result {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to load image {}: {}", name, e);
                image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]))
            }
        };
        let (width, height) = img.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Texture Array Layer: {}", name)),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Slot allocation (Skip index 0 which is the dummy/atlas)
        let index = if self.texture_registry.len() < 255 {
            (self.texture_registry.len() + 1) as u32
        } else {
            // Evict the least recently used texture
            if let Some((old_name, old_index)) = self.texture_registry.pop_lru() {
                self.image_uv_registry.pop(&old_name);
                old_index
            } else {
                1 // Fallback
            }
        };

        self.texture_views[index as usize] = view;
        self.image_uv_registry.put(
            name.to_string(),
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
        self.texture_registry.put(name.to_string(), index);
        self.rebuild_texture_array_bind_group();
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip_rect(&self) -> Rect {
        self.clip_stack.last().copied().unwrap_or(Rect::new(
            0.0,
            0.0,
            self.current_width() as f32,
            self.current_height() as f32,
        ))
    }

    fn memoize(&mut self, _id: u64, _data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer)) {
        render_fn(self);
    }

    fn push_opacity(&mut self, opacity: f32) {
        let current = self.opacity_stack.last().copied().unwrap_or(1.0);
        self.opacity_stack.push(current * opacity);
    }

    fn pop_opacity(&mut self) {
        self.opacity_stack.pop();
    }

    fn push_shadow(&mut self, radius: f32, color: [f32; 4], offset: [f32; 2]) {
        self.shadow_stack.push(ShadowState {
            radius,
            color,
            _offset: offset,
        });
    }

    fn pop_shadow(&mut self) {
        self.shadow_stack.pop();
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        let c = rotation.cos();
        let sn = rotation.sin();
        let affine = glam::Mat3::from_cols(
            glam::Vec3::new(c * scale[0], sn * scale[0], 0.0),
            glam::Vec3::new(-sn * scale[1], c * scale[1], 0.0),
            glam::Vec3::new(translation[0], translation[1], 1.0),
        );

        let parent = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        self.transform_stack.push(parent * affine);
    }

    fn push_affine(&mut self, transform: [f32; 6]) {
        let affine = glam::Mat3::from_cols(
            glam::Vec3::new(transform[0], transform[1], 0.0),
            glam::Vec3::new(transform[2], transform[3], 0.0),
            glam::Vec3::new(transform[4], transform[5], 1.0),
        );
        let parent = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        self.transform_stack.push(parent * affine);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn set_theme(&mut self, theme: ColorTheme) {
        self.current_theme = theme;
        self.queue
            .write_buffer(&self.theme_buffer, 0, bytemuck::bytes_of(&theme));
    }

    fn set_rage(&mut self, rage: f32) {
        self.current_scene.berzerker_rage = rage;
        // scene_buffer is updated every frame in begin_frame, so no need to write here
    }

    fn trigger_shatter_event(&mut self, origin: [f32; 2], force: f32) {
        self.current_scene.shatter_origin = origin;
        self.current_scene.shatter_time = self.current_scene.time;
        self.current_scene.shatter_force = force;
    }

    fn set_scene_preset(&mut self, preset: u32) {
        self.current_scene.scene_type = preset;
    }

    /// push_mjolnir_slice — Pushes a geometric clipping plane onto the stack.
    /// All subsequent draw calls will be sliced by this plane until it is popped.
    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.slice_stack.push((angle, offset));
    }

    /// pop_mjolnir_slice — Removes the top-most geometric clipping plane from the stack.
    fn pop_mjolnir_slice(&mut self) {
        self.slice_stack.pop();
    }

    fn mjolnir_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 8);
    }

    fn mjolnir_fluid_shatter(&mut self, rect: Rect, pieces: u32, force: f32, color: [f32; 4]) {
        self.shatter_internal(rect, pieces, force, color, 11);
    }

    fn draw_mjolnir_bolt(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
        self.recursive_bolt(from, to, 4, color);
    }

    fn upload_data_texture(&mut self, id: &str, data: &[f32], width: u32, height: u32) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(id),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&vec![&view; 256]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some(id),
        });
        self.texture_bind_groups.push(bind_group);
        let tid = (self.texture_bind_groups.len() - 1) as u32;
        self.texture_registry.put(id.to_string(), tid);
    }

    fn draw_heatmap(&mut self, texture_id: &str, rect: Rect, _palette: &str) {
        let tid = self.get_texture_id(texture_id);
        self.fill_rect_with_mode(rect, [1.0, 1.0, 1.0, 1.0], 12, tid);
    }

    fn draw_mesh(&mut self, mesh: &Mesh, color: [f32; 4], transform: glam::Mat4) {
        let base_idx = self.vertices.len() as u32;
        let screen = [self.current_width() as f32, self.current_height() as f32];

        for i in 0..mesh.vertices.len() {
            let pos = transform.transform_point3(glam::Vec3::from(mesh.vertices[i]));
            let norm = transform.transform_vector3(glam::Vec3::from(mesh.normals[i]));

            let (translation, scale_transform, rotation, _, _) = self.current_transform();
            self.vertices.push(Vertex {
                position: pos.to_array(),
                normal: norm.to_array(),
                uv: [0.0, 0.0],
                color,
                mode: 13, // Mode 13: 3D Surface
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [0.0, 0.0],
                size: [0.0, 0.0],
                screen,
                clip: [-10000.0, -10000.0, 20000.0, 20000.0],
                translation,
                scale: scale_transform,
                rotation,
                tex_index: 0,
            });
        }

        for idx in &mesh.indices {
            self.indices.push(base_idx + idx);
        }

        if self.draw_calls.is_empty() || self.current_texture_id.is_some() {
            self.current_texture_id = None;
            self.draw_calls.push(DrawCall {
                texture_id: None,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() as u32) - (mesh.indices.len() as u32),
                index_count: mesh.indices.len() as u32,
                material: cvkg_core::DrawMaterial::Opaque,
            });
        } else {
            self.draw_calls.last_mut().unwrap().index_count += mesh.indices.len() as u32;
        }
    }

    fn register_shared_element(&mut self, id: &str, rect: Rect) {
        self.shared_elements.put(id.to_string(), rect);
    }

    fn set_z_index(&mut self, z: f32) {
        self.current_z = z;
    }

    fn set_material(&mut self, material: cvkg_core::DrawMaterial) {
        self.current_draw_material = material;
    }

    fn current_material(&self) -> cvkg_core::DrawMaterial {
        self.current_draw_material
    }

    fn get_z_index(&self) -> f32 {
        self.current_z
    }

    fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    fn push_vnode(&mut self, rect: Rect, name: &'static str) {
        self.vnode_stack.push((rect, name));
    }

    fn pop_vnode(&mut self) {
        self.vnode_stack.pop();
    }

    fn register_handler(
        &mut self,
        event_type: &str,
        handler: std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>,
    ) {
        self.event_handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    fn serialize_svg(&mut self, name: &str) -> Result<String, String> {
        let tree = self
            .svg_trees
            .get(name)
            .ok_or_else(|| format!("SVG '{}' not found", name))?;
        let config = cvkg_svg_serialize::SerializerConfig::default();
        let mut serializer = cvkg_svg_serialize::SvgSerializer::with_config(config);
        serializer
            .serialize(tree)
            .map_err(|e| format!("SVG serialization failed: {}", e))
    }

    fn apply_svg_filter(
        &mut self,
        name: &str,
        filter_id: &str,
        _region: Rect,
    ) -> Result<String, String> {
        let tree = self
            .svg_trees
            .get(name)
            .ok_or_else(|| format!("SVG '{}' not found", name))?;
        let _filter = Self::find_filter(tree, filter_id)
            .ok_or_else(|| format!("Filter '{}' not found in SVG '{}'", filter_id, name))?;
        let config = cvkg_svg_serialize::SerializerConfig::default();
        let mut serializer = cvkg_svg_serialize::SvgSerializer::with_config(config);
        serializer
            .serialize(tree)
            .map_err(|e| format!("SVG filter serialization failed: {}", e))
    }
}

// ── Inherent methods on SurtrRenderer (not part of the Renderer trait) ──

impl SurtrRenderer {
    /// Clear all registered event handlers. Call at the start of each frame
    /// before re-rendering the component tree.
    pub fn clear_event_handlers(&mut self) {
        self.event_handlers.clear();
    }

    /// Get all registered event handlers for a specific event type.
    pub fn get_handlers(
        &self,
        event_type: &str,
    ) -> Option<&Vec<std::sync::Arc<dyn Fn(cvkg_core::Event) + Send + Sync>>> {
        self.event_handlers.get(event_type)
    }

    /// Compute per-vertex transform values from the current matrix.
    /// Extracts translation, scale, rotation, and skew from the affine matrix
    /// so the existing vertex shader fields still work correctly.
    pub(crate) fn current_transform(&self) -> ([f32; 2], [f32; 2], f32, f32, f32) {
        // Returns (translation, scale, rotation, skew_x, skew_y)
        let m = self
            .transform_stack
            .last()
            .copied()
            .unwrap_or(glam::Mat3::IDENTITY);
        let t = [m.z_axis.x, m.z_axis.y];
        // Extract scale and rotation from the 2x2 submatrix
        let a = m.x_axis.x;
        let b = m.x_axis.y;
        let c = m.y_axis.x;
        let d = m.y_axis.y;
        let sx = (a * a + b * b).sqrt();
        let sy = (c * c + d * d).sqrt();
        let rotation = b.atan2(a);
        // Skew: the angle between the basis vectors minus 90 degrees
        let skew_x = (a * c + b * d) / (sx * sy); // sin(skew)
        (t, [sx, sy], rotation, skew_x, 0.0)
    }

    pub fn stroke_path(&mut self, path: &lyon::path::Path, color: [f32; 4], stroke_width: f32) {
        let c = self.apply_opacity(color);
        let mut tessellator = StrokeTessellator::new();
        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let base_vertex_idx = self.vertices.len() as u32;

        let (translation, scale, rotation, _, _) = self.current_transform();
        let screen = [self.current_width() as f32, self.current_height() as f32];
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        tessellator
            .tessellate_path(
                path,
                &StrokeOptions::default().with_line_width(stroke_width),
                &mut BuffersBuilder::new(
                    &mut buffers,
                    CustomStrokeVertexConstructor {
                        color: c,
                        translation,
                        scale,
                        rotation,
                        screen,
                        clip,
                    },
                ),
            )
            .unwrap();

        self.vertices.extend(buffers.vertices);
        for idx in &buffers.indices {
            self.indices.push(base_vertex_idx + *idx);
        }

        let material = self.current_material();
        let tid = self.get_texture_id("__mega_atlas");

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != tid
            || last_call.unwrap().scissor_rect != self.clip_stack.last().copied()
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = tid;
            self.draw_calls.push(DrawCall {
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: base_vertex_idx,
                index_count: buffers.indices.len() as u32,
                material,
            });
        } else if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += buffers.indices.len() as u32;
        }
    }
}

pub fn parse_svg_animations(data: &[u8]) -> Vec<SvgAnimation> {
    let mut parsed_animations = Vec::new();
    if let Ok(xml_doc) = roxmltree::Document::parse(std::str::from_utf8(data).unwrap_or("")) {
        for node in xml_doc.descendants() {
            if node.tag_name().name() == "animateTransform" || node.tag_name().name() == "animate" {
                let target_id = node
                    .attribute("href")
                    .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))
                    .or_else(|| node.attribute("xlink:href"))
                    .or_else(|| node.parent_element().and_then(|p| p.attribute("id")))
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .to_string();

                if !target_id.is_empty() {
                    let dur_str = node.attribute("dur").unwrap_or("1s");
                    let duration = if dur_str.ends_with("ms") {
                        dur_str
                            .trim_end_matches("ms")
                            .parse::<f32>()
                            .unwrap_or(1000.0)
                            / 1000.0
                    } else {
                        dur_str.trim_end_matches('s').parse::<f32>().unwrap_or(1.0)
                    };

                    let (from_val, to_val) = if let Some(values) = node.attribute("values") {
                        let parts: Vec<&str> = values.split(';').collect();
                        if parts.len() >= 2 {
                            let f = parts[0].trim().parse::<f32>().unwrap_or(0.0);
                            let t = parts[parts.len() - 1].trim().parse::<f32>().unwrap_or(0.0);
                            (f, t)
                        } else {
                            (0.0, 360.0) // Fallback defaults
                        }
                    } else {
                        let f = node
                            .attribute("from")
                            .unwrap_or("0")
                            .parse::<f32>()
                            .unwrap_or(0.0);
                        let t = node
                            .attribute("to")
                            .unwrap_or("360")
                            .parse::<f32>()
                            .unwrap_or(360.0);
                        (f, t)
                    };

                    let attr = node
                        .attribute("attributeName")
                        .unwrap_or("transform")
                        .to_string();

                    parsed_animations.push(SvgAnimation {
                        target_id,
                        attribute_name: attr,
                        from_val,
                        to_val,
                        duration,
                        vertex_range: 0..0, // Will be filled during tessellation
                    });
                }
            }
        }
    }
    parsed_animations
}

// --- SVG Helpers ---

fn usvg_to_lyon(path: &usvg::Path) -> lyon::path::Path {
    let mut builder = lyon::path::Path::builder();
    for segment in path.data().segments() {
        match segment {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                builder.begin(lyon::math::point(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                builder.line_to(lyon::math::point(p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p) => {
                builder.quadratic_bezier_to(
                    lyon::math::point(p1.x, p1.y),
                    lyon::math::point(p.x, p.y),
                );
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p) => {
                builder.cubic_bezier_to(
                    lyon::math::point(p1.x, p1.y),
                    lyon::math::point(p2.x, p2.y),
                    lyon::math::point(p.x, p.y),
                );
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                builder.end(true);
            }
        }
    }
    builder.build()
}

struct SceneVertexConstructor {
    color: [f32; 4],
    translation: [f32; 2],
    scale: [f32; 2],
    rotation: f32,
}

/// Vertex constructor for stroke tessellation -- includes screen and clip for transform.
struct CustomStrokeVertexConstructor {
    color: [f32; 4],
    translation: [f32; 2],
    scale: [f32; 2],
    rotation: f32,
    screen: [f32; 2],
    clip: [f32; 4],
}

impl StrokeVertexConstructor<Vertex> for CustomStrokeVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        let pos = vertex.position();
        Vertex {
            position: [pos.x, pos.y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: self.color,
            mode: 0,
            radius: 0.0,
            slice: [0.0, 0.0, 0.0, 1.0],
            logical: [pos.x, pos.y],
            size: [1.0, 1.0],
            screen: self.screen,
            clip: self.clip,
            translation: self.translation,
            scale: self.scale,
            rotation: self.rotation,
            tex_index: 0,
        }
    }
}

impl FillVertexConstructor<Vertex> for SceneVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: self.color,
            mode: 0,
            radius: 0.0,
            slice: [0.0, 0.0, 0.0, 1.0],
            logical: [vertex.position().x, vertex.position().y],
            size: [1.0, 1.0],
            screen: [0.0, 0.0],
            clip: [-10000.0, -10000.0, 20000.0, 20000.0],
            translation: self.translation,
            scale: self.scale,
            rotation: self.rotation,
            tex_index: 0,
        }
    }
}

impl Drop for SurtrRenderer {
    fn drop(&mut self) {
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
        // Scene pass — opaque draw calls
        for routed in &buckets.scene_commands {
            self.set_material(cvkg_core::DrawMaterial::Opaque);
            self.submit_routed(routed);
        }

        // Glass pass — glassmorphism draw calls sampling blur pyramid
        for routed in &buckets.glass_commands {
            let core_material = match routed.material {
                cvkg_compositor::Material::Opaque => cvkg_core::DrawMaterial::Opaque,
                cvkg_compositor::Material::Glass {
                    blur_radius,
                    depth_index: _,
                } => cvkg_core::DrawMaterial::Glass { blur_radius },
                cvkg_compositor::Material::Overlay => cvkg_core::DrawMaterial::TopUI,
                _ => cvkg_core::DrawMaterial::Opaque,
            };
            self.set_material(core_material);
            self.submit_routed(routed);
        }

        // Overlay pass — foreground UI (crisp text, icons, edge lighting)
        for routed in &buckets.overlay_commands {
            self.set_material(cvkg_core::DrawMaterial::TopUI);
            self.submit_routed(routed);
        }
    }

    /// Submit a single routed draw command through the internal pipeline.
    fn submit_routed(&mut self, routed: &cvkg_compositor::RoutedDrawCommand) {
        let cmd = &routed.command;
        self.fill_rect_with_full_params(
            cvkg_core::Rect::new(0.0, 0.0, 1.0, 1.0),
            [1.0, 1.0, 1.0, 1.0],
            0,
            cmd.texture_id,
            0.0,
            cvkg_core::Rect::new(0.0, 0.0, 1.0, 1.0),
        );
    }
}

impl cvkg_core::FrameRenderer<wgpu::CommandEncoder> for SurtrRenderer {
    fn begin_frame(&mut self) -> wgpu::CommandEncoder {
        cvkg_core::begin_render_phase();
        let id = self
            .current_window
            .expect("No target window set for frame. Call set_target_window first.");
        self.begin_frame(id)
    }

    fn render_frame(&mut self) {
        // Visual Lint: If layout was dirtied during the render phase (layout thrashing),
        // draw a 10px red border as a warning flash.
        if LAYOUT_DIRTY.swap(false, Ordering::AcqRel)
            && let Some(window_id) = self.current_window
            && let Some(surface_ctx) = self.surfaces.get(&window_id)
        {
            let w = surface_ctx.config.width as f32;
            let h = surface_ctx.config.height as f32;
            let border_rect = cvkg_core::Rect {
                x: 0.0,
                y: 0.0,
                width: w,
                height: h,
            };
            // Draw a thick red border to signal layout-thrashing
            self.stroke_rect(border_rect, [1.0, 0.0, 0.0, 1.0], 10.0);
        }

        // Dynamic Buffer Growth (Up to 4x capacity)
        let req_v_size = (self.vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        let mut cur_v_size = self.vertex_buffer.size();
        let max_v_size = (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64 * 4;

        if req_v_size > cur_v_size {
            while cur_v_size < req_v_size && cur_v_size < max_v_size {
                cur_v_size *= 2;
            }
            if req_v_size > max_v_size {
                log::error!("Exceeded dynamic vertex buffer max capacity! Capping geometry.");
                self.vertices
                    .truncate((max_v_size / std::mem::size_of::<Vertex>() as u64) as usize);
                cur_v_size = max_v_size;
            }
            log::info!("Growing vertex buffer to {} bytes", cur_v_size);
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer (Grown)"),
                size: cur_v_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let req_i_size = (self.indices.len() * std::mem::size_of::<u32>()) as u64;
        let mut cur_i_size = self.index_buffer.size();
        let max_i_size = (MAX_INDICES * std::mem::size_of::<u32>()) as u64 * 4;

        if req_i_size > cur_i_size {
            while cur_i_size < req_i_size && cur_i_size < max_i_size {
                cur_i_size *= 2;
            }
            if req_i_size > max_i_size {
                log::error!("Exceeded dynamic index buffer max capacity! Capping geometry.");
                self.indices
                    .truncate((max_i_size / std::mem::size_of::<u32>() as u64) as usize);
                cur_i_size = max_i_size;
            }
            log::info!("Growing index buffer to {} bytes", cur_i_size);
            self.index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer (Grown)"),
                size: cur_i_size,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // Forge Submission: Sync all geometry to GPU using StagingBelt with a dedicated encoder
        let mut staging_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surtr Staging Encoder"),
                });

        let mut has_writes = false;

        if !self.vertices.is_empty() {
            let v_bytes = bytemuck::cast_slice(&self.vertices);
            self.staging_belt
                .write_buffer(
                    &mut staging_encoder,
                    &self.vertex_buffer,
                    0,
                    wgpu::BufferSize::new(v_bytes.len() as u64).unwrap(),
                )
                .copy_from_slice(v_bytes);
            has_writes = true;
        }

        if !self.indices.is_empty() {
            let i_bytes = bytemuck::cast_slice(&self.indices);
            self.staging_belt
                .write_buffer(
                    &mut staging_encoder,
                    &self.index_buffer,
                    0,
                    wgpu::BufferSize::new(i_bytes.len() as u64).unwrap(),
                )
                .copy_from_slice(i_bytes);
            has_writes = true;
        }

        if has_writes {
            self.staging_belt.finish();
            self.staging_command_buffers.push(staging_encoder.finish());
        }

        // Update Time & Uniforms (Direct write is fine for small uniforms)
        self.current_scene.time = self.start_time.elapsed().as_secs_f32();
        self.queue.write_buffer(
            &self.scene_buffer,
            0,
            bytemuck::bytes_of(&self.current_scene),
        );
        self.queue.write_buffer(
            &self.theme_buffer,
            0,
            bytemuck::bytes_of(&self.current_theme),
        );
    }

    fn end_frame(&mut self, encoder: wgpu::CommandEncoder) {
        Self::end_frame(self, encoder);
        cvkg_core::end_render_phase();
    }
}

impl SurtrRenderer {
    /// Returns the current effective opacity (product of all stacked values).
    fn apply_opacity(&self, mut color: [f32; 4]) -> [f32; 4] {
        if let Some(&alpha) = self.opacity_stack.last() {
            color[3] *= alpha;
        }
        color
    }

    /// load_svg — Parses an SVG file and tessellates its paths into GPU triangles.
    pub fn load_svg(&mut self, name: &str, data: &[u8]) {
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(data, &opt).expect("Failed to parse SVG");

        let view_box = Rect {
            x: 0.0,
            y: 0.0,
            width: tree.size().width(),
            height: tree.size().height(),
        };

        let parsed_animations = parse_svg_animations(data);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut tessellator = FillTessellator::new();
        let mut finalized_animations = Vec::new();

        for child in tree.root().children() {
            self.tessellate_node(
                child,
                &mut tessellator,
                &mut vertices,
                &mut indices,
                &parsed_animations,
                &mut finalized_animations,
            );
        }

        self.svg_cache.put(
            name.to_string(),
            SvgModel {
                vertices,
                indices,
                view_box,
                animations: finalized_animations,
            },
        );
        self.svg_trees.put(name.to_string(), tree);
    }

    fn tessellate_node(
        &self,
        node: &usvg::Node,
        tessellator: &mut FillTessellator,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        parsed_animations: &[SvgAnimation],
        finalized_animations: &mut Vec<SvgAnimation>,
    ) {
        let start_idx = vertices.len();
        let node_id = match node {
            usvg::Node::Group(g) => g.id().to_string(),
            usvg::Node::Path(p) => p.id().to_string(),
            _ => String::new(),
        };

        if let usvg::Node::Group(ref group) = *node {
            for child in group.children() {
                self.tessellate_node(
                    child,
                    tessellator,
                    vertices,
                    indices,
                    parsed_animations,
                    finalized_animations,
                );
            }
        } else if let usvg::Node::Path(ref path) = *node
            && let Some(fill) = path.fill()
        {
            let color = match fill.paint() {
                usvg::Paint::Color(c) => [
                    c.red as f32 / 255.0,
                    c.green as f32 / 255.0,
                    c.blue as f32 / 255.0,
                    fill.opacity().get(),
                ],
                _ => [1.0, 1.0, 1.0, 1.0],
            };

            let lyon_path = usvg_to_lyon(path);
            let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
            let base_vertex_idx = vertices.len() as u32;

            tessellator
                .tessellate_path(
                    &lyon_path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(
                        &mut buffers,
                        SceneVertexConstructor {
                            color,
                            translation: [0.0, 0.0],
                            scale: [1.0, 1.0],
                            rotation: 0.0,
                        },
                    ),
                )
                .unwrap();

            vertices.extend(buffers.vertices);
            for idx in buffers.indices {
                indices.push(base_vertex_idx + idx);
            }
        }

        let end_idx = vertices.len();
        if !node_id.is_empty() && start_idx < end_idx {
            for anim in parsed_animations {
                if anim.target_id == node_id {
                    let mut final_anim = anim.clone();
                    final_anim.vertex_range = start_idx..end_idx;
                    finalized_animations.push(final_anim);
                }
            }
        }
    }

    /// draw_svg — Renders a pre-loaded SVG icon at the specified logical rect.
    pub fn draw_svg(&mut self, name: &str, rect: Rect, color: Option<[f32; 4]>, mode: u32) {
        let model = if let Some(m) = self.svg_cache.get(name) {
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

        let mut local_vertices = model.vertices.clone();
        for anim in &model.animations {
            let t = (self.current_scene.time % anim.duration) / anim.duration;
            let val = anim.from_val + (anim.to_val - anim.from_val) * t;

            if anim.attribute_name == "transform" {
                // assume rotation
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for i in anim.vertex_range.clone() {
                    let p = local_vertices[i].position;
                    if p[0] < min_x {
                        min_x = p[0];
                    }
                    if p[1] < min_y {
                        min_y = p[1];
                    }
                    if p[0] > max_x {
                        max_x = p[0];
                    }
                    if p[1] > max_y {
                        max_y = p[1];
                    }
                }
                let cx = (min_x + max_x) * 0.5;
                let cy = (min_y + max_y) * 0.5;

                let c = val.to_radians().cos();
                let s = val.to_radians().sin();

                for i in anim.vertex_range.clone() {
                    let p = local_vertices[i].position;
                    let dx = p[0] - cx;
                    let dy = p[1] - cy;
                    local_vertices[i].position[0] = cx + dx * c - dy * s;
                    local_vertices[i].position[1] = cy + dx * s + dy * c;
                }
            } else if anim.attribute_name == "opacity" {
                for i in anim.vertex_range.clone() {
                    local_vertices[i].color[3] = val;
                }
            }
        }

        for mut v in local_vertices {
            let rel_x = (v.position[0] - model.view_box.x) / model.view_box.width;
            let rel_y = (v.position[1] - model.view_box.y) / model.view_box.height;

            v.position[0] = snap(rect.x + rel_x * rect.width);
            v.position[1] = snap(rect.y + rel_y * rect.height);
            v.position[2] = self.current_z;
            v.logical = [v.position[0], v.position[1]];
            v.screen = screen;
            v.clip = clip;
            v.mode = mode;

            if let Some(override_color) = color {
                let mut c = override_color;
                c[3] *= v.color[3]; // preserve animated opacity
                v.color = self.apply_opacity(c);
            } else {
                v.color = self.apply_opacity(v.color);
            }
            self.vertices.push(v);
        }

        for idx in &model.indices {
            self.indices.push(base_idx + *idx);
        }

        let material = if mode == 7 {
            cvkg_core::DrawMaterial::Glass { blur_radius: 20.0 }
        } else {
            cvkg_core::DrawMaterial::TopUI
        };
        let tid = self.get_texture_id("__mega_atlas");

        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != tid
            || last_call.unwrap().scissor_rect != self.clip_stack.last().copied()
            || last_call.unwrap().material != material;

        if needs_new_call {
            self.current_texture_id = tid;
            self.draw_calls.push(DrawCall {
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: (self.indices.len() - model.indices.len()) as u32,
                index_count: 0,
                material,
            });
        }

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += model.indices.len() as u32;
        }
    }

    /// forge_headless — Initializes Surtr without a window for visual regression testing.
    pub async fn forge_headless(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        // Request adapter with robust multi-stage fallback for Bumblebee/Optimus compatibility
        println!("[GPU] Requesting HighPerformance adapter...");
        let mut adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok();

        if adapter.is_none() {
            println!(
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
            println!("[GPU] Hardware adapters failed, trying Software fallback...");
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
        println!(
            "[GPU] Selected adapter: {} ({:?}) on backend: {:?}",
            info.name, info.device_type, info.backend
        );
        println!("[GPU] Driver info: {} - {}", info.driver, info.driver_info);
        let required_features = adapter.features()
            & (wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Surtr Headless Forge"),
                required_features,
                required_limits: wgpu::Limits {
                    max_bindings_per_bind_group: adapter.limits().max_bindings_per_bind_group.min(256),
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

    /// capture_frame — Read back the rendered frame as a byte buffer (RGBA8).
    pub async fn capture_frame(&self) -> Result<Vec<u8>, String> {
        let ctx = self
            .headless_context
            .as_ref()
            .ok_or("Headless context required for capture")?;
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

            drop(data);
            output_buffer.unmap();
            Ok(result)
        } else {
            Err("Failed to capture frame".to_string())
        }
    }

    fn current_width(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().config.width
        } else {
            self.headless_context.as_ref().unwrap().width
        }
    }

    fn current_height(&self) -> u32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().config.height
        } else {
            self.headless_context.as_ref().unwrap().height
        }
    }

    fn current_scale_factor(&self) -> f32 {
        if let Some(id) = self.current_window {
            self.surfaces.get(&id).unwrap().scale_factor
        } else {
            self.headless_context.as_ref().unwrap().scale_factor
        }
    }

    /// Find a filter by ID in the SVG tree's filter list.
    fn find_filter<'a>(tree: &'a usvg::Tree, filter_id: &str) -> Option<&'a usvg::filter::Filter> {
        tree.filters()
            .iter()
            .find(|f| f.id() == filter_id)
            .map(|arc| arc.as_ref())
    }
}

#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::field_reassign_with_default)]
//! # CVKG SVG Filters
//!
//! WGPU-based SVG filter primitive evaluation.
//! Parses `usvg::filter::Filter` into a directed acyclic graph of filter primitives,
//! then evaluates each primitive as a WGPU render/compute pass.

use std::collections::{HashMap, VecDeque};
use thiserror::Error;

// ── Error Type ──────────────────────────────────────────────────────────────

/// Errors that can occur during filter evaluation.
#[derive(Error, Debug)]
pub enum FilterError {
    /// The filter graph has a cycle.
    #[error("filter graph contains a cycle")]
    CyclicGraph,
    /// A referenced input could not be resolved.
    #[error("unresolved filter input: {0}")]
    UnresolvedInput(String),
    /// WGPU operation failed.
    #[error("WGPU error: {0}")]
    Wgpu(String),
    /// Filter region is invalid (zero or negative size).
    #[error("invalid filter region: {0}x{1}")]
    InvalidRegion(f32, f32),
    /// Texture allocation failed.
    #[error("texture allocation failed: {0}")]
    TextureError(String),
}

impl From<wgpu::Error> for FilterError {
    fn from(e: wgpu::Error) -> Self {
        FilterError::Wgpu(e.to_string())
    }
}

// ── Core Types ───────────────────────────────────────────────────────────────

/// WGPU device/queue pair, stored as Arcs for cheap cloning.
#[derive(Clone)]
pub struct GpuContext {
    pub device: std::sync::Arc<wgpu::Device>,
    pub queue: std::sync::Arc<wgpu::Queue>,
}

/// Manages WGPU resources for SVG filter evaluation.
///
/// Owns reusable textures for ping-pong rendering, samplers, and the
/// filter shader pipeline. Created once and reused across frames.
pub struct FilterEngine {
    color_interpolation: usvg::filter::ColorInterpolation,
    gpu: GpuContext,
    /// Reusable textures for ping-pong rendering.
    temp_textures: Vec<wgpu::Texture>,
    temp_views: Vec<wgpu::TextureView>,
    /// Linear sampler for smooth filtering.
    linear_sampler: wgpu::Sampler,
    /// Nearest sampler for pixel-exact operations.
    nearest_sampler: wgpu::Sampler,
    /// Bind group layout for filter passes.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Render pipeline for filter passes.
    pipeline: wgpu::RenderPipeline,
    /// Pipeline layout for filter passes.
    _render_pipeline_layout: wgpu::PipelineLayout,
    /// Fullscreen quad vertex buffer (shared across all filter passes).
    quad_vertex_buffer: wgpu::Buffer,
    /// Uniform buffer for filter parameters.
    uniform_buffer: wgpu::Buffer,
    /// Shader module (kept alive for pipeline lifetime).
    _shader_module: wgpu::ShaderModule,
    /// LUT texture for component transfer table/discrete modes.
    lut_texture: Option<wgpu::Texture>,
    lut_view: Option<wgpu::TextureView>,
    /// Uploaded image textures for feImage primitives.
    image_textures: HashMap<String, (wgpu::Texture, wgpu::TextureView)>,
    /// Current time for animated uniforms
    current_time: f32,
}

/// Context for a single filter evaluation pass.
pub struct FilterContext<'a> {
    /// The source texture to filter.
    pub source_view: &'a wgpu::TextureView,
    /// The filter region in pixel coordinates.
    pub region: (u32, u32, u32, u32), // x, y, width, height
    /// The element's bounding box in user space (for objectBoundingBox resolution).
    pub element_bbox: usvg::NonZeroRect,
    /// Color interpolation mode.
    pub color_interpolation: usvg::filter::ColorInterpolation,
    /// Backdrop texture for glassmorphism
    pub backdrop_view: Option<&'a wgpu::TextureView>,
    /// Time parameter for animated filters
    pub time: f32,
    /// The full screen size (width, height)
    pub screen_size: (u32, u32),
}

/// Result of evaluating a single filter primitive.
pub struct FilterResult {
    /// The output texture view.
    pub output_view: std::sync::Arc<wgpu::TextureView>,
    /// The actual pixel region covered by this result.
    pub region: (u32, u32, u32, u32),
}

// ── Filter Graph ─────────────────────────────────────────────────────────────

/// A node in the filter DAG.
#[derive(Debug)]
pub struct FilterNode {
    /// Index into the primitives array.
    index: usize,
    /// The result name (for `in`/`result` references).
    result_name: String,
    /// Input slots: which other nodes (or special inputs) feed into this one.
    inputs: Vec<FilterInput>,
    /// The primitive kind.
    kind: usvg::filter::Kind,
    /// Filter subregion (may be different from the overall filter rect).
    rect: usvg::NonZeroRect,
}

/// Resolved input reference for a filter node.
#[derive(Debug, Clone)]
pub enum FilterInput {
    /// The original source graphic.
    SourceGraphic,
    /// The source graphic's alpha channel.
    SourceAlpha,
    /// The backdrop image (rendered behind the element).
    BackdropImage,
    /// The backdrop image's alpha channel.
    BackdropAlpha,
    /// Output of another filter primitive by result name.
    Reference(String),
}

/// Directed acyclic graph of filter primitives.
///
/// Built from `usvg::filter::Filter`, topologically sorted for correct evaluation order.
pub struct FilterGraph {
    /// Nodes in topological evaluation order.
    nodes: Vec<FilterNode>,
    /// Map from result name -> node index.
    name_to_index: HashMap<String, usize>,
}

impl FilterGraph {
    /// Build a `FilterGraph` from a `usvg::filter::Filter`.
    ///
    /// Performs topological sort so that all inputs to a node are evaluated
    /// before the node itself.
    pub fn from_usvg_filter(filter: &usvg::filter::Filter) -> Result<Self, FilterError> {
        let primitives = filter.primitives();
        let mut nodes = Vec::with_capacity(primitives.len());
        let mut name_to_index: HashMap<String, usize> = HashMap::new();

        // First pass: create nodes and build name map.
        for (i, prim) in primitives.iter().enumerate() {
            let result_name = prim.result().to_string();
            let inputs = Self::resolve_inputs(prim.kind());
            let kind = prim.kind().clone();
            let rect = prim.rect();

            if !result_name.is_empty() {
                name_to_index.insert(result_name.clone(), i);
            }

            nodes.push(FilterNode {
                index: i,
                result_name,
                inputs,
                kind,
                rect,
            });
        }

        // Topological sort: Kahn's algorithm.
        let sorted = Self::topological_sort(&nodes)?;

        Ok(FilterGraph {
            nodes: sorted,
            name_to_index,
        })
    }

    /// Return the nodes in evaluation order.
    pub fn nodes(&self) -> &[FilterNode] {
        &self.nodes
    }

    /// Resolve a `FilterInput` to either a special source or a node index.
    pub fn resolve_input(&self, input: &FilterInput) -> Result<ResolvedInput, FilterError> {
        match input {
            FilterInput::SourceGraphic => Ok(ResolvedInput::SourceGraphic),
            FilterInput::SourceAlpha => Ok(ResolvedInput::SourceAlpha),
            FilterInput::BackdropImage => Ok(ResolvedInput::BackdropImage),
            FilterInput::BackdropAlpha => Ok(ResolvedInput::BackdropAlpha),
            FilterInput::Reference(name) => {
                if let Some(&idx) = self.name_to_index.get(name) {
                    Ok(ResolvedInput::NodeIndex(idx))
                } else {
                    // Check if it's a special input that was stored as a reference.
                    match name.as_str() {
                        "SourceGraphic" => Ok(ResolvedInput::SourceGraphic),
                        "SourceAlpha" => Ok(ResolvedInput::SourceAlpha),
                        "BackgroundImage" => Ok(ResolvedInput::BackdropImage),
                        "BackgroundAlpha" => Ok(ResolvedInput::BackdropAlpha),
                        _ => Err(FilterError::UnresolvedInput(name.clone())),
                    }
                }
            }
        }
    }

    /// Resolve the `usvg::Input` enum to our `FilterInput`.
    fn resolve_inputs(kind: &usvg::filter::Kind) -> Vec<FilterInput> {
        match kind {
            usvg::filter::Kind::Blend(blend) => {
                vec![
                    Self::input_to_filter_input(blend.input1()),
                    Self::input_to_filter_input(blend.input2()),
                ]
            }
            usvg::filter::Kind::ColorMatrix(cm) => {
                vec![Self::input_to_filter_input(cm.input())]
            }
            usvg::filter::Kind::ComponentTransfer(ct) => {
                vec![Self::input_to_filter_input(ct.input())]
            }
            usvg::filter::Kind::Composite(comp) => {
                vec![
                    Self::input_to_filter_input(comp.input1()),
                    Self::input_to_filter_input(comp.input2()),
                ]
            }
            usvg::filter::Kind::ConvolveMatrix(cm) => {
                vec![Self::input_to_filter_input(cm.input())]
            }
            usvg::filter::Kind::DiffuseLighting(dl) => {
                vec![Self::input_to_filter_input(dl.input())]
            }
            usvg::filter::Kind::DisplacementMap(dm) => {
                vec![
                    Self::input_to_filter_input(dm.input1()),
                    Self::input_to_filter_input(dm.input2()),
                ]
            }
            usvg::filter::Kind::DropShadow(ds) => {
                vec![Self::input_to_filter_input(ds.input())]
            }
            usvg::filter::Kind::Flood(_) => vec![],
            usvg::filter::Kind::GaussianBlur(gb) => {
                vec![Self::input_to_filter_input(gb.input())]
            }
            usvg::filter::Kind::Image(_) => vec![],
            usvg::filter::Kind::Merge(merge) => merge
                .inputs()
                .iter()
                .map(Self::input_to_filter_input)
                .collect(),
            usvg::filter::Kind::Morphology(m) => {
                vec![Self::input_to_filter_input(m.input())]
            }
            usvg::filter::Kind::Offset(o) => {
                vec![Self::input_to_filter_input(o.input())]
            }
            usvg::filter::Kind::SpecularLighting(sl) => {
                vec![Self::input_to_filter_input(sl.input())]
            }
            usvg::filter::Kind::Tile(t) => {
                vec![Self::input_to_filter_input(t.input())]
            }
            usvg::filter::Kind::Turbulence(_) => vec![],
        }
    }

    fn input_to_filter_input(input: &usvg::filter::Input) -> FilterInput {
        match input {
            usvg::filter::Input::SourceGraphic => FilterInput::SourceGraphic,
            usvg::filter::Input::SourceAlpha => FilterInput::SourceAlpha,
            usvg::filter::Input::Reference(name) => match name.as_str() {
                "BackgroundImage" => FilterInput::BackdropImage,
                "BackgroundAlpha" => FilterInput::BackdropAlpha,
                _ => FilterInput::Reference(name.clone()),
            },
        }
    }

    /// Topological sort using Kahn's algorithm.
    fn topological_sort(nodes: &[FilterNode]) -> Result<Vec<FilterNode>, FilterError> {
        let n = nodes.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        // Build adjacency list and in-degree count.
        // We need to map result names to indices.
        let name_to_idx: HashMap<String, usize> = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.result_name.is_empty())
            .map(|(i, node)| (node.result_name.clone(), i))
            .collect();

        let mut in_degree = vec![0u32; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (i, node) in nodes.iter().enumerate() {
            for input in &node.inputs {
                if let FilterInput::Reference(name) = input
                    && let Some(&dep_idx) = name_to_idx.get(name)
                {
                    adj[dep_idx].push(i);
                    in_degree[i] += 1;
                }
                // SourceGraphic/SourceAlpha have no dependency.
            }
        }

        // Seed queue with zero in-degree nodes.
        let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut sorted = Vec::with_capacity(n);

        while let Some(idx) = queue.pop_front() {
            sorted.push(FilterNode {
                index: nodes[idx].index,
                result_name: nodes[idx].result_name.clone(),
                inputs: nodes[idx].inputs.clone(),
                kind: nodes[idx].kind.clone(),
                rect: nodes[idx].rect,
            });
            for &next in &adj[idx] {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }

        if sorted.len() != n {
            return Err(FilterError::CyclicGraph);
        }

        Ok(sorted)
    }
}

/// A resolved input reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedInput {
    SourceGraphic,
    SourceAlpha,
    BackdropImage,
    BackdropAlpha,
    NodeIndex(usize),
}

// ── Filter Region Resolution ─────────────────────────────────────────────────

/// Compute the pixel extent of a filter primitive's region.
///
/// Handles `objectBoundingBox` (percentages relative to the element's bbox)
/// and `userSpaceOnUse` (absolute units).
pub fn resolve_filter_region(
    primitive_rect: usvg::NonZeroRect,
    element_bbox: usvg::NonZeroRect,
    filter_units: FilterUnits,
    padding: f32,
) -> (u32, u32, u32, u32) {
    let (x, y, w, h) = match filter_units {
        FilterUnits::ObjectBoundingBox => {
            let x = element_bbox.x() + primitive_rect.x() / 100.0 * element_bbox.width();
            let y = element_bbox.y() + primitive_rect.y() / 100.0 * element_bbox.height();
            let w = primitive_rect.width() / 100.0 * element_bbox.width();
            let h = primitive_rect.height() / 100.0 * element_bbox.height();
            (x, y, w, h)
        }
        FilterUnits::UserSpaceOnUse => (
            primitive_rect.x(),
            primitive_rect.y(),
            primitive_rect.width(),
            primitive_rect.height(),
        ),
    };

    // Apply padding for filters that extend beyond their nominal region.
    let x = x - padding;
    let y = y - padding;
    let w = w + padding * 2.0;
    let h = h + padding * 2.0;

    (
        x.max(0.0) as u32,
        y.max(0.0) as u32,
        w.max(1.0) as u32,
        h.max(1.0) as u32,
    )
}

/// Filter coordinate system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterUnits {
    ObjectBoundingBox,
    UserSpaceOnUse,
}

/// Compute padding for filters that extend beyond their nominal region.
pub fn filter_padding(kind: &usvg::filter::Kind) -> f32 {
    match kind {
        usvg::filter::Kind::GaussianBlur(gb) => {
            let sx = gb.std_dev_x().get();
            let sy = gb.std_dev_y().get();
            // 3-sigma rule covers 99.7% of the Gaussian.
            (sx.max(sy) * 3.0).ceil()
        }
        usvg::filter::Kind::DropShadow(ds) => {
            let blur = (ds.std_dev_x().get() + ds.std_dev_y().get()) * 1.5;
            let offset = (ds.dx().abs() + ds.dy().abs()).ceil();
            blur + offset
        }
        usvg::filter::Kind::Morphology(m) => {
            let rx = m.radius_x().get();
            let ry = m.radius_y().get();
            rx.max(ry).ceil()
        }
        usvg::filter::Kind::ConvolveMatrix(cm) => {
            let data = cm.matrix();
            let tx = data.target_x() as f32;
            let ty = data.target_y() as f32;
            let cols = data.columns() as f32;
            let rows = data.rows() as f32;
            // Padding = half kernel size minus target offset.
            let px = (cols / 2.0 - tx).max(0.0);
            let py = (rows / 2.0 - ty).max(0.0);
            px.max(py).ceil()
        }
        _ => 0.0,
    }
}

// ── Filter Engine Implementation ─────────────────────────────────────────────

/// GPU-side uniform buffer matching the WGSL FilterParams struct.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FilterUniforms {
    region: [f32; 4],
    src_size: [f32; 4],
    src2_size: [f32; 4],
    mode: u32,
    sub_mode: u32,
    param0: f32,
    param1: f32,
    param2: f32,
    param3: f32,
    cm_row0: [f32; 4],
    cm_row1: [f32; 4],
    cm_row2: [f32; 4],
    cm_row3: [f32; 4],
    flood_color: [f32; 4],
    offset: [f32; 2],
    _offset_pad: [f32; 2],
    kernel: [f32; 4],
    kernel2: [f32; 4],
    kernel3: f32,
    kernel_divisor: f32,
    kernel_bias: f32,
    _kpad: f32,
    disp_scale: f32,
    _dpad: [f32; 3],
    turb_base_freq: [f32; 2],
    turb_seed: f32,
    turb_num_octaves: f32,
    _tpad: f32,
    // Lighting parameters
    light_position: [f32; 3],
    light_color: [f32; 3],
    light_ambient: f32,
    light_diffuse_k: f32,
    light_specular_k: f32,
    light_shininess: f32,
    light_surface_scale: f32,
    time: f32,
}

impl Default for FilterUniforms {
    fn default() -> Self {
        Self {
            region: [0.0; 4],
            src_size: [1.0, 1.0, 0.0, 0.0],
            src2_size: [1.0, 1.0, 0.0, 0.0],
            mode: 0,
            sub_mode: 0,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
            cm_row0: [1.0, 0.0, 0.0, 0.0],
            cm_row1: [0.0, 1.0, 0.0, 0.0],
            cm_row2: [0.0, 0.0, 1.0, 0.0],
            cm_row3: [0.0, 0.0, 0.0, 1.0],
            flood_color: [0.0, 0.0, 0.0, 1.0],
            offset: [0.0, 0.0],
            _offset_pad: [0.0; 2],
            kernel: [0.0; 4],
            kernel2: [0.0; 4],
            kernel3: 0.0,
            kernel_divisor: 1.0,
            kernel_bias: 0.0,
            _kpad: 0.0,
            disp_scale: 0.0,
            _dpad: [0.0; 3],
            turb_base_freq: [0.01, 0.01],
            turb_seed: 0.0,
            turb_num_octaves: 1.0,
            _tpad: 0.0,
            // Lighting defaults: white light from top-left, moderate ambient
            light_position: [0.5, 0.5, 1.0],
            light_color: [1.0, 1.0, 1.0],
            light_ambient: 0.2,
            light_diffuse_k: 0.8,
            light_specular_k: 0.5,
            light_shininess: 32.0,
            light_surface_scale: 1.0,
            time: 0.0,
        }
    }
}

const FILTER_SHADER_WGSL: &str = include_str!("svg_filters.wgsl");

impl FilterEngine {
    pub fn new(gpu: GpuContext) -> Result<Self, FilterError> {
        let linear_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("svg_filter_linear_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            anisotropy_clamp: 1,
            border_color: None,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
        });

        let nearest_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("svg_filter_nearest_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            anisotropy_clamp: 1,
            border_color: None,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
        });

        let shader_module = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("svg_filter_shader"),
                source: wgpu::ShaderSource::Wgsl(FILTER_SHADER_WGSL.into()),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("svg_filter_bind_group_layout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // LUT texture for component transfer table/discrete
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("svg_filter_pipeline_layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("svg_filter_pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_filter"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_filter"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let quad_vertices: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        let quad_vertex_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("svg_filter_quad"),
            size: std::mem::size_of_val(&quad_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        quad_vertex_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&quad_vertices));
        quad_vertex_buffer.unmap();

        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("svg_filter_uniforms"),
            size: std::mem::size_of::<FilterUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(FilterEngine {
            gpu,
            color_interpolation: usvg::filter::ColorInterpolation::SRGB,
            temp_textures: Vec::new(),
            temp_views: Vec::new(),
            linear_sampler,
            nearest_sampler,
            bind_group_layout,
            pipeline: render_pipeline,
            _render_pipeline_layout: render_pipeline_layout,
            quad_vertex_buffer,
            uniform_buffer,
            _shader_module: shader_module,
            lut_texture: None,
            lut_view: None,
            image_textures: HashMap::new(),
            current_time: 0.0,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn render_pass(
        &self,
        input_view: &wgpu::TextureView,
        input_sampler: &wgpu::Sampler,
        input_size: (f32, f32),
        output_view: &wgpu::TextureView,
        mode: u32,
        sub_mode: u32,
        params: &FilterUniforms,
        src2_view: Option<&wgpu::TextureView>,
        src2_sampler: Option<&wgpu::Sampler>,
        src2_size: (f32, f32),
    ) -> Result<(), FilterError> {
        let mut uniforms = *params;
        uniforms.mode = mode;
        uniforms.sub_mode = sub_mode;
        uniforms.src_size = [input_size.0, input_size.1, 0.0, 0.0];
        uniforms.time = self.current_time;
        if src2_size.0 > 0.0 {
            uniforms.src2_size = [src2_size.0, src2_size.1, 0.0, 0.0];
        }
        self.gpu
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let src2_v = src2_view.unwrap_or(input_view);
        let src2_s = src2_sampler.unwrap_or(input_sampler);

        // Use a dummy 1x1 texture for LUT if none is uploaded.
        let lut_view_ref = self
            .lut_view
            .as_ref()
            .map(|v| v as &wgpu::TextureView)
            .unwrap_or(input_view);

        let bind_group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("svg_filter_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(input_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(src2_v),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(src2_s),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(lut_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                    },
                ],
            });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("svg_filter_encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("svg_filter_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.draw(0..4, 0..1);
        }
        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn crop_backdrop(
        &mut self,
        backdrop_view: &wgpu::TextureView,
        screen_size: (u32, u32),
        region: (u32, u32, u32, u32),
    ) -> Result<FilterResult, FilterError> {
        let out_view = self.get_temp_view(region.2, region.3)?;
        let mut uniforms = FilterUniforms::default();
        uniforms.region = [
            region.0 as f32,
            region.1 as f32,
            region.2 as f32,
            region.3 as f32,
        ];

        // Use MODE_BACKDROP_CROP (18) to extract from full screen to filter region
        self.render_pass(
            backdrop_view,
            &self.linear_sampler,
            (screen_size.0 as f32, screen_size.1 as f32),
            &out_view,
            18,
            0,
            &uniforms,
            None,
            None,
            (0.0, 0.0),
        )?;

        Ok(FilterResult {
            output_view: std::sync::Arc::new(out_view),
            region,
        })
    }

    fn get_temp_view(&mut self, width: u32, height: u32) -> Result<wgpu::TextureView, FilterError> {
        for (i, tex) in self.temp_textures.iter().enumerate() {
            if tex.width() == width && tex.height() == height {
                return Ok(self.temp_views[i].clone());
            }
        }
        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("svg_filter_temp"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.temp_textures.push(texture);
        self.temp_views.push(view.clone());
        Ok(view)
    }

    pub fn clear_pool(&mut self) {
        self.temp_textures.clear();
        self.temp_views.clear();
    }

    /// Evaluate a complete filter graph, returning the final output view.
    pub fn evaluate<'a>(
        &mut self,
        graph: &FilterGraph,
        ctx: &FilterContext<'a>,
    ) -> Result<FilterResult, FilterError> {
        self.current_time = ctx.time;
        if graph.nodes().is_empty() {
            return Ok(FilterResult {
                output_view: std::sync::Arc::new(ctx.source_view.clone()),
                region: ctx.region,
            });
        }

        let mut results: HashMap<usize, std::sync::Arc<wgpu::TextureView>> = HashMap::new();
        let mut cached_backdrop: Option<std::sync::Arc<wgpu::TextureView>> = None;

        for node in graph.nodes() {
            let padding = filter_padding(&node.kind);
            let _region = resolve_filter_region(
                node.rect,
                ctx.element_bbox,
                FilterUnits::ObjectBoundingBox,
                padding,
            );

            let input_views: Vec<std::sync::Arc<wgpu::TextureView>> = node
                .inputs
                .iter()
                .map(|input| match graph.resolve_input(input)? {
                    ResolvedInput::SourceGraphic => {
                        Ok(std::sync::Arc::new(ctx.source_view.clone()))
                    }
                    ResolvedInput::SourceAlpha => {
                        Ok(std::sync::Arc::new(ctx.source_view.clone()))
                    }
                    ResolvedInput::BackdropImage | ResolvedInput::BackdropAlpha => {
                        if cached_backdrop.is_none() {
                            let bv = ctx.backdrop_view.ok_or_else(|| {
                                FilterError::UnresolvedInput(
                                    "Backdrop view requested but not provided".to_string(),
                                )
                            })?;
                            let cropped = self.crop_backdrop(bv, ctx.screen_size, ctx.region)?;
                            cached_backdrop = Some(cropped.output_view);
                        }
                        Ok(cached_backdrop.clone().unwrap())
                    }
                    ResolvedInput::NodeIndex(idx) => results
                        .get(&idx)
                        .cloned()
                        .ok_or_else(|| {
                            FilterError::UnresolvedInput(format!("node {idx} not yet evaluated"))
                        }),
                })
                .collect::<Result<Vec<_>, _>>()?;

            let result = self.evaluate_primitive(
                &node.kind,
                &input_views,
                node.rect,
                ctx.element_bbox,
                ctx.color_interpolation,
            )?;
            results.insert(node.index, result.output_view);
        }

        let last_idx = graph.nodes().last().unwrap().index;
        let output_view = results
            .remove(&last_idx)
            .ok_or_else(|| FilterError::UnresolvedInput("last node missing".to_string()))?;

        Ok(FilterResult {
            output_view,
            region: ctx.region,
        })
    }

    fn evaluate_primitive(
        &mut self,
        kind: &usvg::filter::Kind,
        input_views: &[std::sync::Arc<wgpu::TextureView>],
        rect: usvg::NonZeroRect,
        element_bbox: usvg::NonZeroRect,
        color_interpolation: usvg::filter::ColorInterpolation,
    ) -> Result<FilterResult, FilterError> {
        self.color_interpolation = color_interpolation;
        let w = rect.width().ceil().max(1.0) as u32;
        let h = rect.height().ceil().max(1.0) as u32;

        match kind {
            usvg::filter::Kind::GaussianBlur(gb) => {
                self.apply_gaussian_blur(&*input_views[0], w, h, gb)
            }
            usvg::filter::Kind::ColorMatrix(cm) => {
                self.apply_color_matrix(&*input_views[0], w, h, cm)
            }
            usvg::filter::Kind::Blend(blend) => {
                self.apply_blend(&*input_views[0], &*input_views[1], w, h, blend)
            }
            usvg::filter::Kind::Composite(comp) => {
                self.apply_composite(&*input_views[0], &*input_views[1], w, h, comp)
            }
            usvg::filter::Kind::Flood(flood) => self.apply_flood(w, h, flood),
            usvg::filter::Kind::Offset(offset) => {
                self.apply_offset(&*input_views[0], w, h, rect, element_bbox, offset)
            }
            usvg::filter::Kind::Merge(merge) => {
                self.apply_merge(input_views, w, h, merge)
            }
            usvg::filter::Kind::DropShadow(ds) => self.apply_drop_shadow(&*input_views[0], w, h, ds),
            usvg::filter::Kind::ComponentTransfer(ct) => {
                self.apply_component_transfer(&*input_views[0], w, h, ct)
            }
            usvg::filter::Kind::ConvolveMatrix(cm) => {
                self.apply_convolve_matrix(&*input_views[0], w, h, cm)
            }
            usvg::filter::Kind::DisplacementMap(dm) => {
                self.apply_displacement_map(&*input_views[0], &*input_views[1], w, h, dm)
            }
            usvg::filter::Kind::Morphology(m) => self.apply_morphology(&*input_views[0], w, h, m),
            usvg::filter::Kind::Tile(tile) => {
                self.apply_tile(&*input_views[0], w, h, rect, element_bbox, tile)
            }
            usvg::filter::Kind::Turbulence(t) => self.apply_turbulence(w, h, t),
            usvg::filter::Kind::DiffuseLighting(dl) => {
                self.apply_diffuse_lighting(&*input_views[0], w, h, dl)
            }
            usvg::filter::Kind::SpecularLighting(sl) => {
                self.apply_specular_lighting(&*input_views[0], w, h, sl)
            }
            usvg::filter::Kind::Image(img) => self.apply_image(&*input_views[0], w, h, img),
        }
    }

    // ── Gaussian Blur (Two-Pass Separable) ──────────────────────────────────

    fn apply_gaussian_blur(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        gb: &usvg::filter::GaussianBlur,
    ) -> Result<FilterResult, FilterError> {
        let std_x = gb.std_dev_x().get();
        let std_y = gb.std_dev_y().get();
        let radius_x = ((std_x * 3.0).ceil() as u32).min(64);
        let radius_y = ((std_y * 3.0).ceil() as u32).min(64);

        if radius_x == 0 && radius_y == 0 {
            return self.apply_passthrough(input, w, h);
        }

        let input_size = (w as f32, h as f32);
        let temp_view = self.get_temp_view(w, h)?;
        let output_view = self.get_temp_view(w, h)?;
        let sampler = &self.linear_sampler;

        // Horizontal pass.
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = radius_x as f32;
        params.param1 = std_x;
        self.render_pass(
            input,
            sampler,
            input_size,
            &temp_view,
            0, // MODE_GAUSSIAN_BLUR_H
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Vertical pass.
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = radius_y as f32;
        params.param1 = std_y;
        self.render_pass(
            &temp_view,
            sampler,
            input_size,
            &output_view,
            1, // MODE_GAUSSIAN_BLUR_V
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Return the output texture and its surface as the result.
        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Color Matrix ────────────────────────────────────────────────────────

    fn apply_color_matrix(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        cm: &usvg::filter::ColorMatrix,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let values: &[f32] = match cm.kind() {
            usvg::filter::ColorMatrixKind::Matrix(v) => v.as_slice(),
            _ => &[
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
            ],
        };
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        // ColorMatrix stores 20 values: 4 rows of 5 (4 color + 1 offset).
        // Our uniform stores 4 rows of 4 (color only), offset in param0-3.
        params.cm_row0 = [values[0], values[1], values[2], values[3]];
        params.cm_row1 = [values[5], values[6], values[7], values[8]];
        params.cm_row2 = [values[10], values[11], values[12], values[13]];
        params.cm_row3 = [values[15], values[16], values[17], values[18]];
        params.param0 = values[4]; // offset r
        params.param1 = values[9]; // offset g
        params.param2 = values[14]; // offset b
        params.param3 = values[19]; // offset a

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            2, // MODE_COLOR_MATRIX
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Blend ───────────────────────────────────────────────────────────────

    fn apply_blend(
        &mut self,
        input_a: &wgpu::TextureView,
        input_b: &wgpu::TextureView,
        w: u32,
        h: u32,
        blend: &usvg::filter::Blend,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let sub_mode = match blend.mode() {
            usvg::BlendMode::Normal => 0u32,
            usvg::BlendMode::Multiply => 1u32,
            usvg::BlendMode::Screen => 2u32,
            usvg::BlendMode::Darken => 3u32,
            usvg::BlendMode::Lighten => 4u32,
            usvg::BlendMode::Overlay => 5u32,
            usvg::BlendMode::HardLight => 6u32,
            usvg::BlendMode::SoftLight => 7u32,
            usvg::BlendMode::ColorDodge => 8u32,
            usvg::BlendMode::ColorBurn => 9u32,
            usvg::BlendMode::Exclusion => 10u32,
            usvg::BlendMode::Hue => 11u32,
            usvg::BlendMode::Saturation => 12u32,
            usvg::BlendMode::Color => 13u32,
            usvg::BlendMode::Luminosity => 14u32,
            // Difference has no WGSL equivalent; fall through to Normal.
            usvg::BlendMode::Difference => 0u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        self.render_pass(
            input_a,
            &self.linear_sampler,
            input_size,
            &output_view,
            3, // MODE_BLEND
            sub_mode,
            &params,
            Some(input_b),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Composite ───────────────────────────────────────────────────────────

    fn apply_composite(
        &mut self,
        input_a: &wgpu::TextureView,
        input_b: &wgpu::TextureView,
        w: u32,
        h: u32,
        comp: &usvg::filter::Composite,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        let sub_mode = match comp.operator() {
            usvg::filter::CompositeOperator::Over => 0u32,
            usvg::filter::CompositeOperator::In => 1u32,
            usvg::filter::CompositeOperator::Out => 2u32,
            usvg::filter::CompositeOperator::Atop => 3u32,
            usvg::filter::CompositeOperator::Xor => 4u32,
            usvg::filter::CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
                params.param0 = k1;
                params.param1 = k2;
                params.param2 = k3;
                params.param3 = k4;
                6u32
            }
        };

        self.render_pass(
            input_a,
            &self.linear_sampler,
            input_size,
            &output_view,
            4, // MODE_COMPOSITE
            sub_mode,
            &params,
            Some(input_b),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Flood ───────────────────────────────────────────────────────────────

    fn apply_flood(
        &mut self,
        w: u32,
        h: u32,
        flood: &usvg::filter::Flood,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;

        let c = flood.color();
        let o = flood.opacity().get();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.flood_color = [
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            o,
        ];

        self.render_pass(
            &output_view,
            &self.nearest_sampler,
            (w as f32, h as f32),
            &output_view,
            5, // MODE_FLOOD
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Offset ──────────────────────────────────────────────────────────────
    //
    // Applies dx/dy offset to the source image. The offset is interpreted
    // according to filterUnits: for objectBoundingBox, dx/dy are fractions
    // of the element's bounding box; for userSpaceOnUse, they are absolute.

    fn apply_offset(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        rect: usvg::NonZeroRect,
        _element_bbox: usvg::NonZeroRect,
        offset: &usvg::filter::Offset,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        // Compute the offset in normalized texture coordinates.
        // The SVG spec defines dx/dy relative to the primitive's filter region.
        // rect already accounts for filterUnits (objectBoundingBox percentages
        // are resolved to user-space by the caller), so we normalize by rect size.
        let (dx, dy) = (
            offset.dx() / rect.width().max(0.001),
            offset.dy() / rect.height().max(0.001),
        );

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.offset = [dx, dy];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            6, // MODE_OFFSET
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Merge ───────────────────────────────────────────────────────────────

    fn apply_merge(
        &mut self,
        inputs: &[std::sync::Arc<wgpu::TextureView>],
        w: u32,
        h: u32,
        _merge: &usvg::filter::Merge,
    ) -> Result<FilterResult, FilterError> {
        if inputs.is_empty() {
            return Err(FilterError::UnresolvedInput("merge: no inputs".into()));
        }

        let input_size = (w as f32, h as f32);

        // Merge: composite all inputs using the merge shader.
        // For simplicity, we do pairwise merging.
        let mut result_view = inputs[0].clone();
        for input in inputs.iter().skip(1) {
            let temp_out = self.get_temp_view(w, h)?;
            let mut params = FilterUniforms::default();
            params.region = [0.0, 0.0, w as f32, h as f32];
            self.render_pass(
                &result_view,
                &self.linear_sampler,
                input_size,
                &temp_out,
                7, // MODE_MERGE
                0,
                &params,
                Some(input),
                Some(&self.linear_sampler),
                input_size,
            )?;
            result_view = std::sync::Arc::new(temp_out);
        }

        Ok(FilterResult {
            output_view: result_view,
            region: (0, 0, w, h),
        })
    }

    // ── Drop Shadow ─────────────────────────────────────────────────────────

    fn apply_drop_shadow(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        ds: &usvg::filter::DropShadow,
    ) -> Result<FilterResult, FilterError> {
        let std_x = ds.std_dev_x().get();
        let std_y = ds.std_dev_y().get();
        let radius_x = ((std_x * 3.0).ceil() as u32).min(64);
        let radius_y = ((std_y * 3.0).ceil() as u32).min(64);
        let input_size = (w as f32, h as f32);

        // Step 1: Flood with shadow color.
        let c = ds.color();
        let o = ds.opacity().get();
        let flood_view = self.get_temp_view(w, h)?;
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.flood_color = [
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            o,
        ];
        self.render_pass(
            &flood_view,
            &self.nearest_sampler,
            input_size,
            &flood_view,
            5,
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Step 2: Blur the flood.
        if radius_x > 0 || radius_y > 0 {
            let blur_temp = self.get_temp_view(w, h)?;
            let mut blur_params = FilterUniforms::default();
            blur_params.region = [0.0, 0.0, w as f32, h as f32];
            blur_params.param0 = radius_x as f32;
            blur_params.param1 = std_x;
            self.render_pass(
                &flood_view,
                &self.linear_sampler,
                input_size,
                &blur_temp,
                0,
                0,
                &blur_params,
                None,
                None,
                (0.0, 0.0),
            )?;
            let mut blur_params2 = FilterUniforms::default();
            blur_params2.region = [0.0, 0.0, w as f32, h as f32];
            blur_params2.param0 = radius_y as f32;
            blur_params2.param1 = std_y;
            self.render_pass(
                &blur_temp,
                &self.linear_sampler,
                input_size,
                &flood_view,
                1,
                0,
                &blur_params2,
                None,
                None,
                (0.0, 0.0),
            )?;
        }

        // Step 3: Offset by dx/dy.
        let offset_view = self.get_temp_view(w, h)?;
        let mut offset_params = FilterUniforms::default();
        offset_params.region = [0.0, 0.0, w as f32, h as f32];
        offset_params.offset = [ds.dx() / w as f32, ds.dy() / h as f32];
        self.render_pass(
            &flood_view,
            &self.linear_sampler,
            input_size,
            &offset_view,
            6,
            0,
            &offset_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Step 4: Composite offset shadow over original (merge).
        let output_view = self.get_temp_view(w, h)?;
        let mut merge_params = FilterUniforms::default();
        merge_params.region = [0.0, 0.0, w as f32, h as f32];
        self.render_pass(
            &offset_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            7,
            0,
            &merge_params,
            Some(input),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Component Transfer ──────────────────────────────────────────────────
    //
    // Supports Identity, Linear, and Gamma transfer functions.
    // Table and Discrete are accepted but fall through to identity (the WGSL
    // shader does not yet implement 1D LUT sampling for these modes).
    // A full implementation would upload the LUT as a 1D texture.

    fn apply_component_transfer(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        ct: &usvg::filter::ComponentTransfer,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        // Use the R channel transfer function to determine the mode.
        // A full implementation would run separate passes per channel.
        let func = ct.func_r();
        let sub_mode = match func {
            usvg::filter::TransferFunction::Identity => 0u32,
            usvg::filter::TransferFunction::Table(_) => 1u32,
            usvg::filter::TransferFunction::Discrete(_) => 2u32,
            usvg::filter::TransferFunction::Linear { .. } => 3u32,
            usvg::filter::TransferFunction::Gamma { .. } => 4u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        // For linear: slope in param1, intercept in param2.
        // For gamma: amplitude in param0, exponent in param1, offset in param2.
        match func {
            usvg::filter::TransferFunction::Linear { slope, intercept } => {
                params.param1 = *slope;
                params.param2 = *intercept;
            }
            usvg::filter::TransferFunction::Gamma {
                amplitude,
                exponent,
                offset,
            } => {
                params.param0 = *amplitude;
                params.param1 = *exponent;
                params.param2 = *offset;
            }
            _ => {}
        }

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            8, // MODE_COMPONENT_XFER
            sub_mode,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }
    fn apply_convolve_matrix(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        cm: &usvg::filter::ConvolveMatrix,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let matrix = cm.matrix();
        let values = matrix.data();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        // Pack up to 9 kernel coefficients.
        if values.len() >= 9 {
            params.kernel = [values[0], values[1], values[2], values[3]];
            params.kernel2 = [values[4], values[5], values[6], values[7]];
            params.kernel3 = values[8];
        }
        params.kernel_divisor = cm.divisor().get();
        params.kernel_bias = cm.bias();

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            9, // MODE_CONVOLVE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Displacement Map ────────────────────────────────────────────────────

    fn apply_displacement_map(
        &mut self,
        input: &wgpu::TextureView,
        displacement: &wgpu::TextureView,
        w: u32,
        h: u32,
        dm: &usvg::filter::DisplacementMap,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.disp_scale = dm.scale();
        // Channel selectors: pack x_sel in bits 0-1, y_sel in bits 2-3.
        let x_sel = match dm.x_channel_selector() {
            usvg::filter::ColorChannel::R => 0u32,
            usvg::filter::ColorChannel::G => 1u32,
            usvg::filter::ColorChannel::B => 2u32,
            usvg::filter::ColorChannel::A => 3u32,
        };
        let y_sel = match dm.y_channel_selector() {
            usvg::filter::ColorChannel::R => 0u32,
            usvg::filter::ColorChannel::G => 1u32,
            usvg::filter::ColorChannel::B => 2u32,
            usvg::filter::ColorChannel::A => 3u32,
        };
        params.sub_mode = x_sel | (y_sel << 2);

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            10, // MODE_DISPLACEMENT
            params.sub_mode,
            &params,
            Some(displacement),
            Some(&self.linear_sampler),
            input_size,
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Morphology ──────────────────────────────────────────────────────────

    fn apply_morphology(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        m: &usvg::filter::Morphology,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let rx = m.radius_x();
        let ry = m.radius_y();
        let sub_mode = match m.operator() {
            usvg::filter::MorphologyOperator::Erode => 0u32,
            usvg::filter::MorphologyOperator::Dilate => 1u32,
        };

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.param0 = rx.get();
        params.param1 = ry.get();

        self.render_pass(
            input,
            &self.nearest_sampler,
            input_size,
            &output_view,
            11, // MODE_MORPHOLOGY
            sub_mode,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Tile ────────────────────────────────────────────────────────────────

    fn apply_tile(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        rect: usvg::NonZeroRect,
        _element_bbox: usvg::NonZeroRect,
        _tile: &usvg::filter::Tile,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [rect.x(), rect.y(), rect.width(), rect.height()];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            12, // MODE_TILE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Turbulence ──────────────────────────────────────────────────────────

    fn apply_turbulence(
        &mut self,
        w: u32,
        h: u32,
        t: &usvg::filter::Turbulence,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;

        let bfx = t.base_frequency_x();
        let bfy = t.base_frequency_y();
        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.turb_base_freq = [bfx.get(), bfy.get()];
        params.turb_seed = t.seed() as f32;
        params.turb_num_octaves = t.num_octaves() as f32;

        self.render_pass(
            &output_view,
            &self.nearest_sampler,
            (w as f32, h as f32),
            &output_view,
            13, // MODE_TURBULENCE
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Normal Map ──────────────────────────────────────────────────────────────
    /// Generate a normal map from the input alpha channel using Sobel operators.
    /// This is used internally by the lighting filters, but also exposed for
    /// direct use (e.g., for bump mapping effects).
    #[allow(dead_code)]
    fn apply_normal_map(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_surface_scale = 1.0;

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            14, // MODE_NORMAL_MAP
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Diffuse Lighting ───────────────────────────────────────────────────────

    fn apply_diffuse_lighting(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        dl: &usvg::filter::DiffuseLighting,
    ) -> Result<FilterResult, FilterError> {
        // Step 1: Generate normal map from the input alpha channel.
        let normal_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut normal_params = FilterUniforms::default();
        normal_params.region = [0.0, 0.0, w as f32, h as f32];
        normal_params.light_surface_scale = dl.surface_scale();
        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &normal_view,
            14, // MODE_NORMAL_MAP
            0,
            &normal_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Step 2: Apply diffuse lighting using the normal map.
        let output_view = self.get_temp_view(w, h)?;

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_diffuse_k = dl.diffuse_constant();
        params.light_ambient = 0.0;
        params.light_color = [1.0, 1.0, 1.0];
        params.light_position = [0.5, 0.5, 1.0];
        params.light_surface_scale = dl.surface_scale();

        // Extract light source position/direction.
        let light = dl.light_source();
        match light {
            usvg::filter::LightSource::DistantLight(dl_light) => {
                let azimuth = dl_light.azimuth.to_radians();
                let elevation = dl_light.elevation.to_radians();
                params.light_position = [
                    azimuth.cos() * elevation.cos(),
                    azimuth.sin() * elevation.cos(),
                    elevation.sin(),
                ];
            }
            usvg::filter::LightSource::PointLight(pl) => {
                params.light_position = [pl.x, pl.y, pl.z];
            }
            usvg::filter::LightSource::SpotLight(sl) => {
                params.light_position = [sl.x, sl.y, sl.z];
            }
        }

        self.render_pass(
            &normal_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            15, // MODE_DIFFUSE_LIGHT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Specular Lighting ──────────────────────────────────────────────────────

    fn apply_specular_lighting(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        sl: &usvg::filter::SpecularLighting,
    ) -> Result<FilterResult, FilterError> {
        // Step 1: Generate normal map from the input alpha channel.
        let normal_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut normal_params = FilterUniforms::default();
        normal_params.region = [0.0, 0.0, w as f32, h as f32];
        normal_params.light_surface_scale = sl.surface_scale();
        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &normal_view,
            14, // MODE_NORMAL_MAP
            0,
            &normal_params,
            None,
            None,
            (0.0, 0.0),
        )?;

        // Step 2: Apply specular lighting using the normal map.
        let output_view = self.get_temp_view(w, h)?;

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];
        params.light_specular_k = sl.specular_constant();
        params.light_shininess = sl.specular_exponent();
        params.light_ambient = 0.0;
        params.light_diffuse_k = 0.0;
        params.light_color = [1.0, 1.0, 1.0];
        params.light_position = [0.5, 0.5, 1.0];
        params.light_surface_scale = sl.surface_scale();

        let light = sl.light_source();
        match light {
            usvg::filter::LightSource::DistantLight(dl_light) => {
                let azimuth = dl_light.azimuth.to_radians();
                let elevation = dl_light.elevation.to_radians();
                params.light_position = [
                    azimuth.cos() * elevation.cos(),
                    azimuth.sin() * elevation.cos(),
                    elevation.sin(),
                ];
            }
            usvg::filter::LightSource::PointLight(pl) => {
                params.light_position = [pl.x, pl.y, pl.z];
            }
            usvg::filter::LightSource::SpotLight(sp) => {
                params.light_position = [sp.x, sp.y, sp.z];
            }
        }

        self.render_pass(
            &normal_view,
            &self.linear_sampler,
            input_size,
            &output_view,
            16, // MODE_SPECULAR_LIGHT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Component Transfer LUT ─────────────────────────────────────────────────

    /// Component transfer using a 1D LUT texture for table/discrete modes.
    /// The LUT must be uploaded via `upload_lut` before calling this.
    #[allow(dead_code)]
    fn apply_component_transfer_lut(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        _ct: &usvg::filter::ComponentTransfer,
    ) -> Result<FilterResult, FilterError> {
        let output_view = self.get_temp_view(w, h)?;
        let input_size = (w as f32, h as f32);

        let mut params = FilterUniforms::default();
        params.region = [0.0, 0.0, w as f32, h as f32];

        self.render_pass(
            input,
            &self.linear_sampler,
            input_size,
            &output_view,
            17, // MODE_COMPONENT_XFER_LUT
            0,
            &params,
            None,
            None,
            (0.0, 0.0),
        )?;

        let output_surface = output_view.clone();
        Ok(FilterResult {
            output_view: std::sync::Arc::new(output_surface),
            region: (0, 0, w, h),
        })
    }

    // ── Image ──────────────────────────────────────────────────────────────────

    fn apply_image(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
        img: &usvg::filter::Image,
    ) -> Result<FilterResult, FilterError> {
        // feImage: render the referenced image/SVG into a texture.
        // usvg::filter::Image contains a root Group with child nodes that would
        // need to be rendered to a texture. Full SVG subtree rendering requires
        // the host renderer (cvkg-render-gpu) which is not available in this
        // filter context. Instead, check for a pre-uploaded image texture keyed
        // by the root Group's ID (uploaded via `upload_image`).
        let root = img.root();
        if root.has_children() {
            // Look for a pre-uploaded image texture matching this feImage's ID.
            let id = root.id();
            if !id.is_empty() {
                if let Some((_tex, view)) = self.image_textures.get(id) {
                    return Ok(FilterResult {
                        output_view: std::sync::Arc::new(view.clone()),
                        region: (0, 0, w, h),
                    });
                }
            }
            // No pre-uploaded texture found; fall through to passthrough.
            // A full implementation would render the subtree via an SVG renderer
            // callback provided by the host (see upload_image for manual texture
            // upload as an alternative).
        }
        self.apply_passthrough(input, w, h)
    }

    // ── LUT Upload ─────────────────────────────────────────────────────────────

    /// Upload a 2D LUT texture for component transfer table/discrete modes.
    /// The LUT should be a 256-element array of [f32; 4] RGBA values, where each entry
    /// represents the output color for input value i/255.
    /// The texture is laid out as 256x4 with each channel in its own row, so that
    /// sampling at (value, row) returns the output for that channel.
    pub fn upload_lut(&mut self, data: &[[f32; 4]]) -> Result<(), FilterError> {
        // Restructure data: transpose from [256 RGBA entries] to [256 texels x 4 rows]
        // Input: data[i] = [r_out, g_out, b_out, a_out] for input value i/255
        // Output texture: row 0 contains all r_out values, row 1 all g_out values, etc.
        // Each texel in a row holds [channel_value, _, _, _] where R component holds the output.
        let mut tex_data: [[[f32; 4]; 256]; 4] = [[[0.0; 4]; 256]; 4];
        for (i, rgba) in data.iter().enumerate() {
            tex_data[0][i] = [rgba[0], 0.0, 0.0, 0.0]; // R channel row
            tex_data[1][i] = [0.0, rgba[1], 0.0, 0.0]; // G channel row
            tex_data[2][i] = [0.0, 0.0, rgba[2], 0.0]; // B channel row
            tex_data[3][i] = [0.0, 0.0, 0.0, rgba[3]]; // A channel row
        }

        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("svg_filter_lut"),
            size: wgpu::Extent3d {
                width: 256,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&tex_data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<[f32; 4]>() as u32 * 256),
                rows_per_image: Some(4),
            },
            wgpu::Extent3d {
                width: 256,
                height: 4,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.lut_texture = Some(texture);
        self.lut_view = Some(view);
        Ok(())
    }

    // ── Image Upload ───────────────────────────────────────────────────────────

    /// Upload an image texture for feImage primitives.
    /// The data should be RGBA8 pixel data of the given dimensions.
    pub fn upload_image(
        &mut self,
        key: &str,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), FilterError> {
        if data.len() != (width as usize * height as usize * 4) {
            return Err(FilterError::TextureError(format!(
                "image data size {} does not match {}x{}x4",
                data.len(),
                width,
                height
            )));
        }

        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("svg_filter_image_upload"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.image_textures.insert(key.to_string(), (texture, view));
        Ok(())
    }

    // ── Passthrough ─────────────────────────────────────────────────────────

    fn apply_passthrough(
        &mut self,
        input: &wgpu::TextureView,
        w: u32,
        h: u32,
    ) -> Result<FilterResult, FilterError> {
        Ok(FilterResult {
            output_view: std::sync::Arc::new(input.clone()),
            region: (0, 0, w, h),
        })
    }

    /// Current version of the cvkg-svg-filters crate.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    // ── Tests ────────────────────────────────────────────────────────────────────
}

#[cfg(test)]
mod tests {
    use super::*;

    // This is the kind stub for Flood used in tests to verify graph topology.
    // We can't construct the real usvg filter types from outside the crate,
    // but we can construct our own FilterNode structs to test the graph logic.
    fn flood_kind() -> usvg::filter::Kind {
        // Parse a minimal SVG with a flood filter to get a real Kind.
        // This is the only way to get a usvg::filter::Kind from outside the crate.
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
            <defs>
                <filter id="f1">
                    <feFlood flood-color="red" flood-opacity="1"/>
                </filter>
            </defs>
            <rect width="100" height="100" filter="url(#f1)"/>
        </svg>"#;
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
        let root = tree.root();
        // The rect element should have a filter. Walk the tree to find it.
        find_first_filter_kind(root).expect("should find flood filter in parsed SVG")
    }

    #[allow(dead_code)]
    fn gaussian_blur_kind() -> usvg::filter::Kind {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
            <defs>
                <filter id="f1">
                    <feGaussianBlur stdDeviation="3"/>
                </filter>
            </defs>
            <rect width="100" height="100" filter="url(#f1)"/>
        </svg>"#;
        let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
        let root = tree.root();
        find_first_filter_kind(root).expect("should find blur filter")
    }

    fn find_first_filter_kind(group: &usvg::Group) -> Option<usvg::filter::Kind> {
        for child in group.children() {
            if let usvg::Node::Group(g) = child {
                for filter in g.filters() {
                    if let Some(prim) = filter.primitives().first() {
                        return Some(prim.kind().clone());
                    }
                }
                if let Some(kind) = find_first_filter_kind(g) {
                    return Some(kind);
                }
            }
        }
        None
    }

    #[test]
    fn test_filter_graph_empty() {
        let nodes: Vec<FilterNode> = vec![];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_filter_graph_single_node() {
        let nodes = vec![FilterNode {
            index: 0,
            result_name: "out".to_string(),
            inputs: vec![FilterInput::SourceGraphic],
            kind: flood_kind(),
            rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
        }];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert_eq!(sorted.len(), 1);
    }

    #[test]
    fn test_filter_graph_chain() {
        // A -> B (B references A)
        let nodes = vec![
            FilterNode {
                index: 0,
                result_name: "a".to_string(),
                inputs: vec![FilterInput::SourceGraphic],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
            FilterNode {
                index: 1,
                result_name: "b".to_string(),
                inputs: vec![FilterInput::Reference("a".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
        ];
        let sorted = FilterGraph::topological_sort(&nodes).unwrap();
        assert_eq!(sorted.len(), 2);
        // A must come before B.
        assert_eq!(sorted[0].result_name, "a");
        assert_eq!(sorted[1].result_name, "b");
    }

    #[test]
    fn test_filter_graph_cycle_detection() {
        // A -> B -> A (cycle)
        let nodes = vec![
            FilterNode {
                index: 0,
                result_name: "a".to_string(),
                inputs: vec![FilterInput::Reference("b".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
            FilterNode {
                index: 1,
                result_name: "b".to_string(),
                inputs: vec![FilterInput::Reference("a".to_string())],
                kind: flood_kind(),
                rect: usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap(),
            },
        ];
        let result = FilterGraph::topological_sort(&nodes);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_filter_region_user_space() {
        let rect = usvg::NonZeroRect::from_xywh(10.0, 20.0, 100.0, 200.0).unwrap();
        let bbox = usvg::NonZeroRect::from_xywh(0.0, 0.0, 500.0, 500.0).unwrap();
        let (x, y, w, h) = resolve_filter_region(rect, bbox, FilterUnits::UserSpaceOnUse, 0.0);
        assert_eq!((x, y, w, h), (10, 20, 100, 200));
    }

    #[test]
    fn test_resolve_filter_region_object_bbox() {
        let rect = usvg::NonZeroRect::from_xywh(10.0, 20.0, 50.0, 50.0).unwrap();
        let bbox = usvg::NonZeroRect::from_xywh(0.0, 0.0, 200.0, 100.0).unwrap();
        let (x, y, w, h) = resolve_filter_region(rect, bbox, FilterUnits::ObjectBoundingBox, 0.0);
        // x = 0 + 10/100 * 200 = 20
        // y = 0 + 20/100 * 100 = 20
        // w = 50/100 * 200 = 100
        // h = 50/100 * 100 = 50
        assert_eq!((x, y, w, h), (20, 20, 100, 50));
    }

    #[test]
    fn test_resolve_filter_region_with_padding() {
        let rect = usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
        let bbox = usvg::NonZeroRect::from_xywh(0.0, 0.0, 100.0, 100.0).unwrap();
        let (x, y, w, h) = resolve_filter_region(rect, bbox, FilterUnits::UserSpaceOnUse, 5.0);
        // x = 0 - 5 = -5 -> clamped to 0
        // y = 0 - 5 = -5 -> clamped to 0
        // w = 100 + 10 = 110
        // h = 100 + 10 = 110
        assert_eq!((x, y, w, h), (0, 0, 110, 110));
    }
}

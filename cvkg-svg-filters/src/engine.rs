use std::collections::HashMap;
use crate::types::{
    FilterContext, FilterError, FilterResult, FilterUnits, GpuContext,
    filter_padding, resolve_filter_region, ResolvedInput,
};
use crate::graph::FilterGraph;

/// GPU-side uniform buffer matching the WGSL FilterParams struct.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FilterUniforms {
    pub(crate) region: [f32; 4],
    pub(crate) src_size: [f32; 4],
    pub(crate) src2_size: [f32; 4],
    pub(crate) mode: u32,
    pub(crate) sub_mode: u32,
    pub(crate) param0: f32,
    pub(crate) param1: f32,
    pub(crate) param2: f32,
    pub(crate) param3: f32,
    pub(crate) cm_row0: [f32; 4],
    pub(crate) cm_row1: [f32; 4],
    pub(crate) cm_row2: [f32; 4],
    pub(crate) cm_row3: [f32; 4],
    pub(crate) flood_color: [f32; 4],
    pub(crate) offset: [f32; 2],
    pub(crate) _offset_pad: [f32; 2],
    pub(crate) kernel: [f32; 4],
    pub(crate) kernel2: [f32; 4],
    pub(crate) kernel3: f32,
    pub(crate) kernel_divisor: f32,
    pub(crate) kernel_bias: f32,
    pub(crate) _kpad: f32,
    pub(crate) disp_scale: f32,
    pub(crate) _dpad: [f32; 3],
    pub(crate) turb_base_freq: [f32; 2],
    pub(crate) turb_seed: f32,
    pub(crate) turb_num_octaves: f32,
    pub(crate) _tpad: f32,
    // Lighting parameters
    pub(crate) light_position: [f32; 3],
    pub(crate) light_color: [f32; 3],
    pub(crate) light_ambient: f32,
    pub(crate) light_diffuse_k: f32,
    pub(crate) light_specular_k: f32,
    pub(crate) light_shininess: f32,
    pub(crate) light_surface_scale: f32,
    pub(crate) time: f32,
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

pub(crate) const FILTER_SHADER_WGSL: &str = include_str!("svg_filters.wgsl");

/// Manages WGPU resources for SVG filter evaluation.
pub struct FilterEngine {
    pub(crate) color_interpolation: usvg::filter::ColorInterpolation,
    pub(crate) gpu: GpuContext,
    pub(crate) temp_textures: Vec<wgpu::Texture>,
    pub(crate) temp_views: Vec<wgpu::TextureView>,
    pub(crate) linear_sampler: wgpu::Sampler,
    pub(crate) nearest_sampler: wgpu::Sampler,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) _render_pipeline_layout: wgpu::PipelineLayout,
    pub(crate) quad_vertex_buffer: wgpu::Buffer,
    pub(crate) uniform_buffer: wgpu::Buffer,
    pub(crate) _shader_module: wgpu::ShaderModule,
    pub(crate) lut_texture: Option<wgpu::Texture>,
    pub(crate) lut_view: Option<wgpu::TextureView>,
    pub(crate) image_textures: HashMap<String, (wgpu::Texture, wgpu::TextureView)>,
    pub(crate) current_time: f32,
}

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
    pub(crate) fn render_pass(
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

    pub(crate) fn crop_backdrop(
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

    pub(crate) fn get_temp_view(&mut self, width: u32, height: u32) -> Result<wgpu::TextureView, FilterError> {
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

    pub fn upload_lut(&mut self, data: &[[f32; 4]]) -> Result<(), FilterError> {
        let mut tex_data: [[[f32; 4]; 256]; 4] = [[[0.0; 4]; 256]; 4];
        for (i, rgba) in data.iter().enumerate() {
            tex_data[0][i] = [rgba[0], 0.0, 0.0, 0.0];
            tex_data[1][i] = [0.0, rgba[1], 0.0, 0.0];
            tex_data[2][i] = [0.0, 0.0, rgba[2], 0.0];
            tex_data[3][i] = [0.0, 0.0, 0.0, rgba[3]];
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

    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

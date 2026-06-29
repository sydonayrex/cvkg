use crate::types::{GpuParticle, MAX_PARTICLES, ParticleUniforms};
use crate::vertex::{InstanceData, Vertex};
use crate::{WGSL_PARTICLES, WGSL_TONEMAP};

pub(crate) struct CompiledPipelines {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) opaque_pipeline: wgpu::RenderPipeline,
    pub(crate) ui_pipeline: wgpu::RenderPipeline,
    pub(crate) glass_pipeline: wgpu::RenderPipeline,
    pub(crate) background_pipeline: wgpu::RenderPipeline,
    pub(crate) bloom_extract_pipeline: wgpu::RenderPipeline,
    pub(crate) copy_pipeline: wgpu::RenderPipeline,
    pub(crate) composite_pipeline: wgpu::RenderPipeline,
    pub(crate) color_blind_pipeline: wgpu::RenderPipeline,
    pub(crate) volumetric_pipeline: wgpu::RenderPipeline,
    pub(crate) volumetric_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) volumetric_uniform_buffer: wgpu::Buffer,
    pub(crate) volumetric_depth_sampler: wgpu::Sampler,
    pub(crate) color_blind_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) color_blind_uniform_buffer: wgpu::Buffer,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) kawase_down_pipeline: wgpu::RenderPipeline,
    pub(crate) kawase_up_pipeline: wgpu::RenderPipeline,
    pub(crate) kawase_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) kawase_uniform: wgpu::Buffer,
    pub(crate) kawase_uniform_buffers: Vec<wgpu::Buffer>,
    pub(crate) particle_compute_pipeline: wgpu::ComputePipeline,
    pub(crate) particle_compute_bgl: wgpu::BindGroupLayout,
    pub(crate) particle_buffer: wgpu::Buffer,
    pub(crate) particle_uniform_buffer: wgpu::Buffer,
    pub(crate) particle_render_pipeline: wgpu::RenderPipeline,
    pub(crate) particle_render_bgl: wgpu::BindGroupLayout,
    pub(crate) tonemap_pipeline: wgpu::RenderPipeline,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_render_pipelines(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    pipeline_cache: Option<&wgpu::PipelineCache>,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    env_bind_group_layout: &wgpu::BindGroupLayout,
    berserker_bind_group_layout: &wgpu::BindGroupLayout,
    gradient_bind_group_layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
    wgsl_opaque: &str,
    wgsl_glass: &str,
    queue: &wgpu::Queue,
) -> CompiledPipelines {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Surtr Main Pipeline Layout"),
        bind_group_layouts: &[
            Some(texture_bind_group_layout),
            Some(env_bind_group_layout),
            Some(berserker_bind_group_layout),
            Some(gradient_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let post_process_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Muspelheim Post Process Layout"),
        bind_group_layouts: &[
            Some(texture_bind_group_layout),
            Some(env_bind_group_layout),
            Some(berserker_bind_group_layout),
            Some(gradient_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Muspelheim Composite Layout"),
        bind_group_layouts: &[
            Some(texture_bind_group_layout),
            Some(env_bind_group_layout),
            Some(berserker_bind_group_layout),
            Some(gradient_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Surtr Main Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc(), InstanceData::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
            depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: pipeline_cache,
    });

    let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Surtr Background Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
        cache: pipeline_cache,
    });

    let opaque_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Muspelheim Opaque"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl_opaque)),
    });
    let glass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Muspelheim Glass"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(wgsl_glass)),
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
            depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: pipeline_cache,
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
        cache: pipeline_cache,
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
        cache: pipeline_cache,
    });

    let bloom_extract_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Muspelheim Bloom Extract"),
        layout: Some(&post_process_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
        cache: pipeline_cache,
    });

    let copy_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Muspelheim Copy"),
        layout: Some(&post_process_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
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
        cache: pipeline_cache,
    });

    let kawase_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Kawase Blur Pyramid"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            "../shaders/blur_pyramid.wgsl"
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
        cache: pipeline_cache,
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
        cache: pipeline_cache,
    });

    let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Muspelheim Composite"),
        layout: Some(&composite_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_composite"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
        cache: pipeline_cache,
    });

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
        cache: pipeline_cache,
    });

    let volumetric_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Surtr Volumetric Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            "../shaders/volumetric.wgsl"
        ))),
    });

    let volumetric_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Volumetric Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
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
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
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
        cache: pipeline_cache,
    });

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
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[f32; 4]>() as u64),
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
        cache: pipeline_cache,
    });

    let color_blind_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Color Blind Uniforms"),
        size: std::mem::size_of::<crate::color_blindness::ColorBlindUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let volumetric_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Volumetric Uniforms"),
        size: std::mem::size_of::<[f32; 24]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let volumetric_depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::Less),
        ..Default::default()
    });

    let particle_compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Particle Compute BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
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
                        std::mem::size_of::<ParticleUniforms>() as u64,
                    ),
                },
                count: None,
            },
        ],
    });

    let particle_compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
            cache: pipeline_cache,
        });

    let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Particle Storage Buffer"),
        size: (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::VERTEX,
        mapped_at_creation: false,
    });

    let particle_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Particle Uniform Buffer"),
        size: std::mem::size_of::<ParticleUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let particle_render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Particle Render BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    (MAX_PARTICLES * std::mem::size_of::<GpuParticle>()) as u64,
                ),
            },
            count: None,
        }],
    });

    let particle_render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Particle Render Layout"),
        bind_group_layouts: &[Some(&particle_render_bgl)],
        immediate_size: 0,
    });

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
    let life = p.color_life.w;
    if (life <= 0.0) {
        out.pos = vec4<f32>(0.0, 0.0, 2.0, 1.0);
        out.color = vec4<f32>(0.0);
    } else {
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

    let particle_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        cache: pipeline_cache,
    });

    let kawase_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Kawase Persistent Uniform"),
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let kawase_uniform_buffers: Vec<wgpu::Buffer> = (0..16)
        .map(|i| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Kawase Persistent Uniform {}", i)),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();

    CompiledPipelines {
        pipeline,
        opaque_pipeline,
        ui_pipeline,
        glass_pipeline,
        background_pipeline,
        bloom_extract_pipeline,
        copy_pipeline,
        composite_pipeline,
        color_blind_pipeline,
        volumetric_pipeline,
        volumetric_bind_group_layout: volumetric_bgl,
        volumetric_uniform_buffer,
        volumetric_depth_sampler,
        color_blind_bind_group_layout: color_blind_bgl,
        color_blind_uniform_buffer,
        sampler,
        kawase_down_pipeline,
        kawase_up_pipeline,
        kawase_bind_group_layout: kawase_bgl,
        kawase_uniform,
        kawase_uniform_buffers,
        particle_compute_pipeline,
        particle_compute_bgl,
        particle_buffer,
        particle_uniform_buffer,
        particle_render_pipeline,
        particle_render_bgl,
        tonemap_pipeline,
    }
}

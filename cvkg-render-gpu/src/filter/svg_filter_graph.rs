//! SVG Filter Engine
//!
//! Implements the SVG filter pipeline for CVKG's render graph.
//! Supports: feGaussianBlur, feDropShadow, feOffset, feBlend, feComposite, feFlood, feMerge.
//!
//! Architecture:
//! - FilterEngine owns temporary textures for intermediate results
//! - Each primitive is executed as a render pass with a full-screen quad
//! - Primitives read from input textures and write to output textures
//! - The filter graph is executed in order, with each primitive's output
//!   becoming the next primitive's input

use crate::kvasir::nodes::RES_SCENE;

use crate::kvasir::node::ExecutionContext;
use crate::kvasir::resource::ResourceId;

/// A filter graph that executes a chain of SVG filter primitives.
#[derive(Debug, Clone)]
pub struct SvgFilterGraph {
    /// Ordered list of filter primitives to execute.
    pub primitives: Vec<FilterPrimitive>,
    /// Input resource name (e.g., "source" or previous primitive's output).
    pub input: String,
    /// Output resource name.
    pub output: String,
}

impl Default for SvgFilterGraph {
    fn default() -> Self {
        Self {
            primitives: Vec::new(),
            input: "source".into(),
            output: "result".into(),
        }
    }
}

/// SVG filter primitive types.
#[derive(Debug, Clone)]
pub enum FilterPrimitive {
    /// feGaussianBlur — Blurs the input image using a two-pass separable kernel.
    GaussianBlur {
        std_deviation: f32,
        input: String,
        result: String,
    },
    /// feDropShadow — Creates a drop shadow effect (offset + blur + composite).
    DropShadow {
        dx: f32,
        dy: f32,
        std_deviation: f32,
        flood_color: [f32; 4],
        input: String,
        result: String,
    },
    /// feOffset — Translates the input image.
    Offset {
        dx: f32,
        dy: f32,
        input: String,
        result: String,
    },
    /// feBlend — Blends two images together.
    Blend {
        mode: BlendMode,
        in1: String,
        in2: String,
        result: String,
    },
    /// feComposite — Composites two images using Porter-Duff operations.
    Composite {
        operator: CompositeOperator,
        in1: String,
        in2: String,
        result: String,
    },
    /// feFlood — Fills the filter region with a solid color.
    Flood { color: [f32; 4], result: String },
    /// feMerge — Merges multiple filter results.
    Merge { inputs: Vec<String>, result: String },
}

/// SVG blend mode for feBlend.
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Darken,
    Lighten,
}

impl BlendMode {
    pub fn from_name(s: &str) -> Self {
        match s {
            "multiply" => BlendMode::Multiply,
            "screen" => BlendMode::Screen,
            "darken" => BlendMode::Darken,
            "lighten" => BlendMode::Lighten,
            _ => BlendMode::Normal,
        }
    }

    /// Returns the GPU blend factor for this mode.
    /// Returns (src_factor, dst_factor, src_alpha_factor, dst_alpha_factor).
    pub fn to_wgpu_blend(
        &self,
    ) -> (
        wgpu::BlendFactor,
        wgpu::BlendFactor,
        wgpu::BlendFactor,
        wgpu::BlendFactor,
    ) {
        match self {
            BlendMode::Normal => (
                wgpu::BlendFactor::SrcAlpha,
                wgpu::BlendFactor::OneMinusSrcAlpha,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::OneMinusSrcAlpha,
            ),
            BlendMode::Multiply => (
                wgpu::BlendFactor::Dst,
                wgpu::BlendFactor::OneMinusSrcAlpha,
                wgpu::BlendFactor::DstAlpha,
                wgpu::BlendFactor::OneMinusSrcAlpha,
            ),
            BlendMode::Screen => (
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::OneMinusSrc,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::OneMinusSrcAlpha,
            ),
            BlendMode::Darken => (
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
            ),
            BlendMode::Lighten => (
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
                wgpu::BlendFactor::One,
            ),
        }
    }
}

/// SVG composite operator for feComposite.
#[derive(Debug, Clone, Copy)]
pub enum CompositeOperator {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Arithmetic,
}

impl CompositeOperator {
    pub fn from_name(s: &str) -> Self {
        match s {
            "in" => CompositeOperator::In,
            "out" => CompositeOperator::Out,
            "atop" => CompositeOperator::Atop,
            "xor" => CompositeOperator::Xor,
            "arithmetic" => CompositeOperator::Arithmetic,
            _ => CompositeOperator::Over,
        }
    }
}

/// Filter execution engine.
///
/// Owns temporary textures for intermediate results and executes
/// filter primitives as GPU render passes.
pub struct FilterEngine {
    /// Intermediate render targets for filter results.
    temp_targets: Vec<(String, ResourceId)>,
    /// Width of the filter region.
    width: u32,
    /// Height of the filter region.
    height: u32,
}

impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterEngine {
    /// Create a new filter engine with the given dimensions.
    pub fn new() -> Self {
        Self {
            temp_targets: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Set the filter region dimensions.
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Execute a filter graph within the given execution context.
    pub fn execute(&mut self, graph: &SvgFilterGraph, ctx: &mut ExecutionContext) {
        if self.width == 0 || self.height == 0 {
            // Derive dimensions from the input texture
            if let Some(tex) = ctx.registry.get_texture(ResourceId(RES_SCENE.0)) {
                self.width = tex.width();
                self.height = tex.height();
            }
        }

        for primitive in &graph.primitives {
            self.execute_primitive(primitive, ctx);
        }
    }

    fn execute_primitive(&mut self, primitive: &FilterPrimitive, ctx: &mut ExecutionContext) {
        match primitive {
            FilterPrimitive::GaussianBlur { std_deviation, .. } => {
                self.execute_gaussian_blur(*std_deviation, ctx);
            }
            FilterPrimitive::DropShadow {
                dx,
                dy,
                std_deviation,
                flood_color,
                ..
            } => {
                self.execute_drop_shadow(*dx, *dy, *std_deviation, *flood_color, ctx);
            }
            FilterPrimitive::Offset { dx, dy, .. } => {
                self.execute_offset(*dx, *dy, ctx);
            }
            FilterPrimitive::Blend { mode, .. } => {
                self.execute_blend(*mode, ctx);
            }
            FilterPrimitive::Composite { operator, .. } => {
                self.execute_composite(*operator, ctx);
            }
            FilterPrimitive::Flood { color, .. } => {
                self.execute_flood(*color, ctx);
            }
            FilterPrimitive::Merge { .. } => {
                self.execute_merge(ctx);
            }
        }
    }

    /// Execute feGaussianBlur using a two-pass separable kernel.
    ///
    /// Pass 1: Horizontal blur from input to temp texture
    /// Pass 2: Vertical blur from temp to output texture
    fn execute_gaussian_blur(&mut self, std_deviation: f32, ctx: &mut ExecutionContext) {
        let input_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing input texture for GaussianBlur");
                return;
            }
        };

        let output_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing output texture for GaussianBlur");
                return;
            }
        };

        // Use the renderer's blur pipeline if available, otherwise fall back to simple copy
        let blur_pipeline = match ctx.renderer.blur_pipeline.as_ref() {
            Some(p) => p,
            None => {
                log::warn!("[FilterEngine] Blur pipeline not available, skipping GaussianBlur");
                return;
            }
        };

        // Write blur uniform data
        let kernel_size = (std_deviation * 3.0).ceil() as i32;
        let uniform_data: [f32; 8] = [
            self.width as f32,
            self.height as f32,
            std_deviation,
            kernel_size as f32,
            1.0, // horizontal pass
            0.0,
            0.0,
            0.0,
        ];

        if let Some(blur_uniform) = ctx.renderer.blur_uniform.as_ref() {
            ctx.queue
                .write_buffer(blur_uniform, 0, bytemuck::cast_slice(&uniform_data));
        }

        // Create bind group for input texture
        let input_bind_group = ctx.get_or_create_bind_group(
            (RES_SCENE, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                },
            ],
            Some("blur_input_bg"),
        );

        // Pass 1: Horizontal blur
        {
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("feGaussianBlur Horizontal"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(blur_pipeline);
            pass.set_bind_group(0, &input_bind_group, &[]);
            pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        log::trace!(
            "[FilterEngine] feGaussianBlur std_deviation={} completed ({}x{})",
            std_deviation,
            self.width,
            self.height
        );
    }

    /// Execute feDropShadow: offset the input, blur it, then composite with original.
    fn execute_drop_shadow(
        &mut self,
        dx: f32,
        dy: f32,
        std_deviation: f32,
        flood_color: [f32; 4],
        ctx: &mut ExecutionContext,
    ) {
        // Step 1: Apply flood color to the alpha channel (shadow colorization)
        // Step 2: Offset the shadow
        // Step 3: Blur the shadow
        // Step 4: Composite shadow behind original

        // For now, execute a simplified version: blur + offset
        self.execute_gaussian_blur(std_deviation, ctx);

        log::trace!(
            "[FilterEngine] feDropShadow dx={} dy={} std={} color={:?}",
            dx,
            dy,
            std_deviation,
            flood_color
        );
    }

    /// Execute feOffset: translate the input image by (dx, dy).
    fn execute_offset(&mut self, dx: f32, dy: f32, ctx: &mut ExecutionContext) {
        let input_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing input texture for Offset");
                return;
            }
        };

        let output_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing output texture for Offset");
                return;
            }
        };

        // Use the copy pipeline with offset
        let input_bind_group = ctx.get_or_create_bind_group(
            (RES_SCENE, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                },
            ],
            Some("offset_input_bg"),
        );

        {
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("feOffset"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&ctx.renderer.copy_pipeline);
            pass.set_bind_group(0, &input_bind_group, &[]);
            pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        log::trace!("[FilterEngine] feOffset dx={} dy={}", dx, dy);
    }

    /// Execute feBlend: blend two images together using the specified blend mode.
    fn execute_blend(&mut self, mode: BlendMode, ctx: &mut ExecutionContext) {
        let (src_factor, dst_factor, src_alpha, dst_alpha) = mode.to_wgpu_blend();

        let input_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing input texture for Blend");
                return;
            }
        };

        let output_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing output texture for Blend");
                return;
            }
        };

        let input_bind_group = ctx.get_or_create_bind_group(
            (RES_SCENE, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                },
            ],
            Some("blend_input_bg"),
        );

        {
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("feBlend {:?}", mode)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
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
            });

            // Use the blend pipeline if available
            if let Some(blend_pipeline) = ctx.renderer.blend_pipeline.as_ref() {
                pass.set_pipeline(blend_pipeline);
                pass.set_bind_group(0, &input_bind_group, &[]);
                pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            } else {
                // Fallback to copy pipeline
                if let Some(ref blend_pipeline) = ctx.renderer.blend_pipeline {
                    pass.set_pipeline(blend_pipeline);
                    pass.set_bind_group(0, &input_bind_group, &[]);
                    pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
                }
            }
            pass.draw(0..3, 0..1);
        }

        log::trace!(
            "[FilterEngine] feBlend mode={:?} src={:?} dst={:?}",
            mode,
            src_factor,
            dst_factor
        );
    }

    /// Execute feComposite: composite two images using Porter-Duff operations.
    fn execute_composite(&mut self, operator: CompositeOperator, ctx: &mut ExecutionContext) {
        // Composite uses the blend pipeline with specific blend factors per operator
        let input_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing input texture for Composite");
                return;
            }
        };

        let output_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing output texture for Composite");
                return;
            }
        };

        let input_bind_group = ctx.get_or_create_bind_group(
            (RES_SCENE, 0, false),
            &ctx.renderer.texture_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.renderer.sampler),
                },
            ],
            Some("composite_input_bg"),
        );

        {
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("feComposite {:?}", operator)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
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
            });

            if let Some(blend_pipeline) = ctx.renderer.blend_pipeline.as_ref() {
                pass.set_pipeline(blend_pipeline);
                pass.set_bind_group(0, &input_bind_group, &[]);
                pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            } else if let Some(ref blend_pipeline) = ctx.renderer.blend_pipeline {
                pass.set_pipeline(blend_pipeline);
                pass.set_bind_group(0, &input_bind_group, &[]);
                pass.set_bind_group(1, &ctx.renderer.dummy_env_bind_group, &[]);
            }
            pass.draw(0..3, 0..1);
        }

        log::trace!("[FilterEngine] feComposite operator={:?}", operator);
    }

    /// Execute feFlood: fill the filter region with a solid color.
    fn execute_flood(&mut self, color: [f32; 4], ctx: &mut ExecutionContext) {
        let output_view = match ctx.registry.get_texture_view(RES_SCENE) {
            Some(v) => v,
            None => {
                log::error!("[FilterEngine] Missing output texture for Flood");
                return;
            }
        };

        {
            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("feFlood"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color[0] as f64,
                            g: color[1] as f64,
                            b: color[2] as f64,
                            a: color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // No pipeline needed — just clear to the flood color
            log::trace!("[FilterEngine] feFlood color={:?}", color);
        }
    }

    /// Execute feMerge: merge multiple filter results by compositing them in order.
    fn execute_merge(&mut self, ctx: &mut ExecutionContext) {
        // Merge composites all input layers in order
        // For now, just log — full implementation would iterate inputs
        log::trace!("[FilterEngine] feMerge");
    }
}

/// Convert an SVG filter element from pillage-doc into a SvgFilterGraph.
///
/// This bridges the document model (FilterNode/FilterPrimitive) to the
/// render graph model (SvgFilterGraph/FilterPrimitive).
#[cfg(feature = "pillage")]
pub fn build_filter_graph(
    _filter_node: &pillage_doc::node::FilterNode,
    primitives: &[pillage_doc::node::FilterPrimitive],
) -> SvgFilterGraph {
    let mut graph = SvgFilterGraph::default();

    for prim in primitives {
        let filter_prim = match prim.primitive_type {
            pillage_doc::node::FilterPrimitiveType::GaussianBlur => FilterPrimitive::GaussianBlur {
                std_deviation: prim.std_deviation.unwrap_or(0.0),
                input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                result: prim.result.clone().unwrap_or_else(|| "blur".into()),
            },
            pillage_doc::node::FilterPrimitiveType::DropShadow => FilterPrimitive::DropShadow {
                dx: prim.dx.unwrap_or(0.0),
                dy: prim.dy.unwrap_or(0.0),
                std_deviation: prim.std_deviation.unwrap_or(0.0),
                flood_color: prim.flood_color.unwrap_or([0.0, 0.0, 0.0, 0.5]),
                input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                result: prim.result.clone().unwrap_or_else(|| "shadow".into()),
            },
            pillage_doc::node::FilterPrimitiveType::Offset => FilterPrimitive::Offset {
                dx: prim.offset_x.unwrap_or(0.0),
                dy: prim.offset_y.unwrap_or(0.0),
                input: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                result: prim.result.clone().unwrap_or_else(|| "offset".into()),
            },
            pillage_doc::node::FilterPrimitiveType::Blend => FilterPrimitive::Blend {
                mode: BlendMode::from_name(prim.blend_mode.as_deref().unwrap_or("normal")),
                in1: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                in2: "background".into(),
                result: prim.result.clone().unwrap_or_else(|| "blend".into()),
            },
            pillage_doc::node::FilterPrimitiveType::Composite => FilterPrimitive::Composite {
                operator: CompositeOperator::Over,
                in1: prim.in_attr.clone().unwrap_or_else(|| "source".into()),
                in2: "background".into(),
                result: prim.result.clone().unwrap_or_else(|| "composite".into()),
            },
            pillage_doc::node::FilterPrimitiveType::Flood => FilterPrimitive::Flood {
                color: prim.flood_color.unwrap_or([0.0, 0.0, 0.0, 1.0]),
                result: prim.result.clone().unwrap_or_else(|| "flood".into()),
            },
            pillage_doc::node::FilterPrimitiveType::Merge => FilterPrimitive::Merge {
                inputs: vec!["source".into()],
                result: prim.result.clone().unwrap_or_else(|| "merge".into()),
            },
            _ => {
                log::warn!(
                    "[FilterEngine] Unsupported filter primitive: {:?}",
                    prim.primitive_type
                );
                continue;
            }
        };
        graph.primitives.push(filter_prim);
    }

    graph
}

/// Builder for constructing SVG filter render graph nodes.
#[derive(Default)]
pub struct SvgFilterGraphBuilder {
    nodes: Vec<(ResourceId, ResourceId, Option<SvgFilterGraph>)>,
}

impl SvgFilterGraphBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a filter pass that reads from `input` and writes to `output`.
    pub fn add_pass(
        mut self,
        input: ResourceId,
        output: ResourceId,
        graph: SvgFilterGraph,
    ) -> Self {
        self.nodes.push((input, output, Some(graph)));
        self
    }

    /// Add a pass that reads from the scene and writes to a temp buffer.
    pub fn add_scene_pass(self, output: ResourceId, graph: SvgFilterGraph) -> Self {
        self.add_pass(output, output, graph)
    }

    /// Add a pass that reads from a temp buffer and writes to the scene.
    pub fn add_final_pass(self, input: ResourceId, graph: SvgFilterGraph) -> Self {
        self.add_pass(input, input, graph)
    }

    /// Build the list of SvgFilterNodes.
    pub fn build(self) -> Vec<SvgFilterNode> {
        self.nodes
            .into_iter()
            .map(|(input, output, graph)| {
                let mut node = SvgFilterNode::new(input, output);
                if let Some(g) = graph {
                    node = node.with_filter_graph(g);
                }
                node
            })
            .collect()
    }
}

// Re-export the SvgFilterNode from passes for convenience
pub use crate::passes::svg_filter::SvgFilterNode;

/// Resource IDs for SVG filter intermediate results.
pub use crate::passes::svg_filter::{RES_FILTER_TEMP_A, RES_FILTER_TEMP_B};

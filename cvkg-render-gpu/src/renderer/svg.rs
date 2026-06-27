use crate::draw::{parse_svg_animations, usvg_to_lyon};
use crate::renderer::GpuRenderer;
use crate::types::{SvgAnimation, SvgModel, SvgPath};
use crate::vertex::{CustomStrokeVertexConstructor, SceneVertexConstructor, Vertex};
use cvkg_core::Rect;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator, VertexBuffers,
};

/// SVG tessellation parameters.
pub(crate) struct TessellateParams<'a> {
    pub(crate) fill_tessellator: &'a mut FillTessellator,
    pub(crate) stroke_tessellator: &'a mut StrokeTessellator,
    pub(crate) vertices: &'a mut Vec<Vertex>,
    pub(crate) indices: &'a mut Vec<u32>,
    pub(crate) parsed_animations: &'a [SvgAnimation],
    pub(crate) finalized_animations: &'a mut Vec<SvgAnimation>,
    pub(crate) paths: &'a mut Vec<crate::types::SvgPath>,
}

impl GpuRenderer {
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
                        Self::tessellate_fill_solid(&lyon_path, color, &node_id, params, fill_rule);
                    }
                    usvg::Paint::LinearGradient(g) => {
                        Self::tessellate_fill_gradient(
                            &lyon_path,
                            g,
                            fill_opacity,
                            &node_id,
                            params,
                            fill_rule,
                        );
                    }
                    usvg::Paint::RadialGradient(g) => {
                        Self::tessellate_fill_radial_gradient(
                            &lyon_path,
                            g,
                            fill_opacity,
                            &node_id,
                            params,
                            fill_rule,
                        );
                    }
                    usvg::Paint::Pattern(_) => {
                        log::warn!(
                            "SVG path '{}' uses pattern fill which is not supported, using white fallback",
                            node_id
                        );
                        let color = [1.0, 1.0, 1.0, fill_opacity];
                        Self::tessellate_fill_solid(&lyon_path, color, &node_id, params, fill_rule);
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
                let mut stroke_opts = StrokeOptions::default().with_line_width(stroke_width);

                // Line cap
                stroke_opts = match stroke.linecap() {
                    usvg::LineCap::Butt => {
                        stroke_opts.with_line_cap(lyon::tessellation::LineCap::Butt)
                    }
                    usvg::LineCap::Round => {
                        stroke_opts.with_line_cap(lyon::tessellation::LineCap::Round)
                    }
                    usvg::LineCap::Square => {
                        stroke_opts.with_line_cap(lyon::tessellation::LineCap::Square)
                    }
                };

                // Line join
                stroke_opts = match stroke.linejoin() {
                    usvg::LineJoin::Miter => {
                        stroke_opts.with_line_join(lyon::tessellation::LineJoin::Miter)
                    }
                    usvg::LineJoin::Round => {
                        stroke_opts.with_line_join(lyon::tessellation::LineJoin::Round)
                    }
                    usvg::LineJoin::Bevel => {
                        stroke_opts.with_line_join(lyon::tessellation::LineJoin::Bevel)
                    }
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
                        CustomStrokeVertexConstructor {
                            color,
                            clip,
                            path_length,
                        },
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
    fn gradient_color_at(stops: &[usvg::Stop], pos: f32, fill_opacity: f32) -> [f32; 4] {
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
            return [
                c.red as f32 / 255.0,
                c.green as f32 / 255.0,
                c.blue as f32 / 255.0,
                start.opacity().get() * fill_opacity,
            ];
        }
        if pos >= eo {
            let c = end.color();
            return [
                c.red as f32 / 255.0,
                c.green as f32 / 255.0,
                c.blue as f32 / 255.0,
                end.opacity().get() * fill_opacity,
            ];
        }
        let range = eo - so;
        if range < 0.0001 {
            let c = start.color();
            return [
                c.red as f32 / 255.0,
                c.green as f32 / 255.0,
                c.blue as f32 / 255.0,
                start.opacity().get() * fill_opacity,
            ];
        }
        let t = (pos - so) / range;
        let sc = start.color();
        let ec = end.color();
        [
            (sc.red as f32 + (ec.red as f32 - sc.red as f32) * t) / 255.0,
            (sc.green as f32 + (ec.green as f32 - sc.green as f32) * t) / 255.0,
            (sc.blue as f32 + (ec.blue as f32 - sc.blue as f32) * t) / 255.0,
            (start.opacity().get() + (end.opacity().get() - start.opacity().get()) * t)
                * fill_opacity,
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
            &mut BuffersBuilder::new(
                &mut buffers,
                SceneVertexConstructor {
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ),
        ) {
            log::warn!(
                "SVG gradient fill tessellation failed for path '{}': {:?}, skipping",
                node_id,
                e
            );
            return;
        }

        let stops = gradient.stops();
        for mut vertex in buffers.vertices {
            let px = vertex.position[0];
            let py = vertex.position[1];
            let t = if grad_len_sq < 0.0001 {
                0.5
            } else {
                ((px - x1) * dx + (py - y1) * dy) / grad_len_sq
            };
            vertex.color = Self::gradient_color_at(stops, t, fill_opacity);
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
            &mut BuffersBuilder::new(
                &mut buffers,
                SceneVertexConstructor {
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ),
        ) {
            log::warn!(
                "SVG radial gradient fill tessellation failed for path '{}': {:?}, skipping",
                node_id,
                e
            );
            return;
        }

        for mut vertex in buffers.vertices {
            let px = vertex.position[0];
            let py = vertex.position[1];
            let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)).sqrt();
            let r_val = r.get();
            let t = if r_val < 0.001 {
                0.5
            } else {
                (dist / r_val).clamp(0.0, 1.0)
            };
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

    pub fn draw_svg_with_offset(
        &mut self,
        name: &str,
        rect: Rect,
        color: Option<[f32; 4]>,
        material_id: u32,
        animation_time_offset: f32,
    ) {
        self.draw_svg_with_order(name, rect, color, material_id, animation_time_offset, 0);
    }

    pub fn draw_svg_with_order(
        &mut self,
        name: &str,
        rect: Rect,
        color: Option<[f32; 4]>,
        material_id: u32,
        animation_time_offset: f32,
        draw_order: i32,
    ) {
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

        log::info!(
            "DRAW_SVG '{}' called with rect: {:?}, model_view_box: {:?}",
            name,
            rect,
            self.svg.model_cache.get(name).map(|m| m.view_box)
        );

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

        let base_idx = self.vertices.len() as u32;
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
            Self::position_vertices(
                &mut local_vertices,
                model.view_box,
                rect,
                material_id,
                clip,
                snap,
            );
            let base_vertex = self.vertices.len() as u32;
            self.vertices.extend(local_vertices);
            let index_count = model.indices.len();
            for idx in &model.indices {
                self.indices.push(base_vertex + *idx);
            }
            let material = Self::resolve_material(material_id);
            let tid = self.get_texture_id("__mega_heim");
            Self::emit_draw_call(
                self,
                material,
                tid,
                clip_rect,
                index_count as u32,
                base_vertex,
            );
        } else {
            // Per-path rendering: each path gets its own transform and draw call.
            for path in &model.paths {
                let mut path_verts: Vec<Vertex> =
                    model.vertices[path.vertex_range.clone()].to_vec();
                // Apply local transform (translate, rotate, scale) in SVG space.
                if path.local_transform.scale != 1.0
                    || path.local_transform.rotation != 0.0
                    || path.local_transform.translate != [0.0, 0.0]
                {
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
                            let mut min_x = f32::MAX;
                            let mut min_y = f32::MAX;
                            let mut max_x = f32::MIN;
                            let mut max_y = f32::MIN;
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
                            for v in &mut path_verts {
                                v.color[3] = val;
                            }
                        } else if anim.attribute_name == "stroke-dashoffset" {
                            for v in &mut path_verts {
                                v.slice[3] = 1.0 - val;
                            }
                        }
                    }
                }
                // Position into output rect.
                Self::position_vertices(
                    &mut path_verts,
                    model.view_box,
                    rect,
                    material_id,
                    clip,
                    snap,
                );
                let base_vertex = self.vertices.len() as u32;
                let index_start = self.indices.len();
                self.vertices.extend(path_verts);
                // Remap indices for this path's vertex offset.
                let path_index_start = path.index_range.start;
                for idx in &model.indices[path.index_range.clone()] {
                    self.indices
                        .push(base_vertex + *idx - path_index_start as u32);
                }
                let index_count = path.index_range.len() as u32;
                let material = Self::resolve_material(material_id);
                let tid = self.get_texture_id("__mega_heim");
                Self::emit_draw_call(self, material, tid, clip_rect, index_count, base_vertex);
            }
        }
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

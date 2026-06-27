use crate::renderer::GpuRenderer;
use crate::types::DrawCall;
use crate::vertex::{CustomStrokeVertexConstructor, InstanceData, Vertex};
use cvkg_core::Renderer;
use lyon::tessellation::{BuffersBuilder, StrokeOptions, StrokeTessellator, VertexBuffers};
use std::hash::Hasher;

impl GpuRenderer {
    /// Inherent method: stroke a lyon path using wgpu.
    pub fn stroke_path_impl(
        &mut self,
        path: &lyon::path::Path,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        let c = self.apply_opacity(color);
        let base_vertex_idx = self.vertices.len() as u32;
        let base_index_idx = self.indices.len() as u32;

        let path_hash = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            let num_elements = path.iter().count();
            std::hash::Hash::hash(&num_elements, &mut h);
            std::hash::Hash::hash(&stroke_width.to_bits(), &mut h);
            h.finish()
        };

        let (vert_count, idx_count) = match self.path_geometry_cache.get(&path_hash) {
            Some((cached_verts, cached_indices)) => {
                self.vertices.extend_from_slice(cached_verts);
                for idx in cached_indices {
                    self.indices.push(base_vertex_idx + *idx);
                }
                (cached_verts.len(), cached_indices.len())
            }
            None => {
                let mut tessellator = StrokeTessellator::new();
                let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
                let result = tessellator.tessellate_path(
                    path,
                    &StrokeOptions::default().with_line_width(stroke_width),
                    &mut BuffersBuilder::new(
                        &mut buffers,
                        CustomStrokeVertexConstructor {
                            color: c,
                            clip: [0.0, 0.0, 0.0, 0.0],
                            path_length: 1.0,
                        },
                    ),
                );
                if let Err(e) = result {
                    log::warn!("Failed to tessellate stroke path: {:?}", e);
                    return;
                }
                let vert_count = buffers.vertices.len();
                let idx_count = buffers.indices.len();
                let cached_verts = buffers.vertices.clone();
                let cached_indices = buffers.indices.clone();
                self.path_geometry_cache
                    .put(path_hash, (cached_verts, cached_indices));
                self.vertices.extend(buffers.vertices);
                for idx in &buffers.indices {
                    self.indices.push(base_vertex_idx + *idx);
                }
                (vert_count, idx_count)
            }
        };

        let material = self.current_material();
        let tid = self.get_texture_id("__mega_heim");

        if self.draw_calls.last().is_none()
            || self.current_texture_id != tid
            || self.draw_calls.last().unwrap().scissor_rect != self.clip_stack.last().copied()
            || self.draw_calls.last().unwrap().material != material
        {
            self.current_texture_id = tid;
            let (translation, scale, rotation, _, _) = self.current_transform();
            self.instance_data.push(InstanceData {
                translation,
                scale,
                rotation,
                blur_radius: 0.0,
                ior_override: 0.0,
                glass_intensity: 1.0,
            });
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: tid,
                scissor_rect: self.clip_stack.last().copied(),
                index_start: base_index_idx,
                index_count: idx_count as u32,
                instance_count: 1,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            if let Some(last) = self.draw_calls.last_mut() {
                last.index_count += idx_count as u32;
            }
        }
    }
}

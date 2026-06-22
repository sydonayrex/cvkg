//! Geometry drawing methods for GpuRenderer.
//!
//! Extracted from draw.rs for modularization.
//! Contains fill_rect, shatter effects, lightning bolt effects, and oriented quad rendering.

use crate::types::*;
use crate::vertex::Vertex;
use crate::renderer::GpuRenderer;
use cvkg_core::{Rect, Renderer};

impl GpuRenderer {
    pub(crate) fn shatter_internal(
        &mut self,
        rect: Rect,
        pieces: u32,
        force: f32,
        color: [f32; 4],
        material_id: u32,
    ) {
        // High-Fidelity Variable Particle Density
        let count = (pieces as f32).sqrt().ceil() as u32;
        let dw = rect.width / count as f32;
        let dh = rect.height / count as f32;

        let c = self.apply_opacity(color);

        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;

        for y in 0..count {
            for x in 0..count {
                let init_x = rect.x + x as f32 * dw;
                let init_y = rect.y + y as f32 * dh;

                // Center of the shard relative to the card center
                let dx = (init_x + dw * 0.5) - cx;
                let dy = (init_y + dh * 0.5) - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);

                // Normal direction outwards
                let nx = dx / dist;
                let ny = dy / dist;

                // Hash-based pseudo-random variations for dispersion
                let hash =
                    ((x as f32 * 12.9898 + y as f32 * 78.233).sin().fract() * 43_758.547).fract();
                let hash2 =
                    ((x as f32 * 37.11 + y as f32 * 149.87).sin().fract() * 23_412.19).fract();

                let speed_var = 0.5 + hash * 1.5;
                let angle = ny.atan2(nx) + (hash2 - 0.5) * 0.6;
                let disp_x = angle.cos() * force * 50.0 * speed_var;
                let disp_y = angle.sin() * force * 50.0 * speed_var;

                // Downward gravity-like drift over time/force
                let gravity = force * force * 20.0;

                // Shrink shard size as it scatters away
                // Assuming max force in demo is ~6.0
                let scale_factor = (1.0 - (force / 6.0).min(1.0)).max(0.0);
                let shard_w = dw * scale_factor;
                let shard_h = dh * scale_factor;

                let displaced_x = init_x + disp_x + (dw - shard_w) * 0.5;
                let displaced_y = init_y + disp_y + gravity + (dh - shard_h) * 0.5;

                let shard_rect = Rect {
                    x: displaced_x,
                    y: displaced_y,
                    width: shard_w,
                    height: shard_h,
                };

                let uv = Rect {
                    x: x as f32 / count as f32,
                    y: y as f32 / count as f32,
                    width: 1.0 / count as f32,
                    height: 1.0 / count as f32,
                };

                self.fill_rect_with_full_params(shard_rect, c, material_id, None, force, uv);
            }
        }
    }

    pub(crate) fn recursive_bolt(
        &mut self,
        from: [f32; 2],
        to: [f32; 2],
        depth: u32,
        color: [f32; 4],
    ) {
        if depth == 0 {
            self.draw_lightning_segment(from, to, color);
            return;
        }

        let mid_x = (from[0] + to[0]) * 0.5;
        let mid_y = (from[1] + to[1]) * 0.5;

        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();

        if len < 1e-4 {
            return;
        }

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

    pub(crate) fn draw_lightning_segment(&mut self, from: [f32; 2], to: [f32; 2], color: [f32; 4]) {
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
        let cp4 = [from[0] - cnx, from[1] - gny];
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

    pub(crate) fn push_oriented_quad(
        &mut self,
        points: [[f32; 2]; 4],
        color: [f32; 4],
        material_id: u32,
        uv_rect: Rect,
    ) {
        let scissor = self.clip_stack.last().copied();
        let texture_id = None; // Oriented quads like lightning don't use textures yet

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius: 0.0,
            ior_override: 0.0,
            glass_intensity: 1.0,
        };

        // CRITICAL FIX: Only break batch on material/scissor/texture state changes.
        // Transform (translation/scale/rotation) is per-instance data.
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || self.current_texture_id != texture_id
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != Self::resolve_material_with_context(material_id, &self.current_draw_material)
            || {
                let last_material = last_call.unwrap().material;
                let current_material = Self::resolve_material_with_context(material_id, &self.current_draw_material);
                matches!((current_material, last_material),
                    (cvkg_core::DrawMaterial::Glass { blur_radius: a, ior_override: b, glass_intensity: c },
                     cvkg_core::DrawMaterial::Glass { blur_radius: d, ior_override: e, glass_intensity: f })
                    if a != d || b != e || c != f)
            };

        if needs_new_call {
            self.current_texture_id = texture_id;
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                instance_count: 1,
                material: Self::resolve_material_with_context(material_id, &self.current_draw_material),
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            // Same batch - add instance data and increment instance count
            self.instance_data.push(current_instance_data);
            if let Some(call) = self.draw_calls.last_mut() {
                call.instance_count += 1;
            }
        }

        let uvs = [
            [uv_rect.x, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y],
            [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            [uv_rect.x, uv_rect.y + uv_rect.height],
        ];

        let rect = Rect {
            x: points[0][0],
            y: points[0][1],
            width: 1.0,
            height: 1.0,
        };

        for i in 0..4 {
            let px = points[i][0];
            let py = points[i][1];

            self.vertices.push(Vertex {
                position: [px, py, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: uvs[i],
                color,
                material_id,
                radius: 0.0,
                slice: [0.0, 0.0, 0.0, 1.0],
                logical: [px - rect.x, py - rect.y],
                size: [rect.width, rect.height],
                clip: [-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY],
                tex_index: 0,
            });
        }

        // Push indices for the quad (two triangles: 0-1-2 and 0-2-3)
        let base = self.vertices.len() as u32 - 4;
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        if let Some(call) = self.draw_calls.last_mut() {
            call.index_count += 6;
        }
    }

    pub(crate) fn get_texture_id(&mut self, name: &str) -> Option<u32> {
        self.texture_registry.get(name).copied()
    }

    /// fill_rect_with_mode -- Specialized rectangle drawing with mode-specific shader logic.
    pub fn fill_rect_with_mode(
        &mut self,
        rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
    ) {
        self.fill_rect_with_full_params(
            rect,
            color,
            material_id,
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fill_rect_with_full_params_and_slice(
        &mut self,
        mut rect: Rect,
        color: [f32; 4],
        material_id: u32,
        texture_id: Option<u32>,
        radius: f32,
        uv_rect: Rect,
        slice: [f32; 4],
        _glyph_time: [f32; 2],
    ) {
        // Pixel-snap rect coordinates to prevent sub-pixel blurring on high-DPI displays.
        // Only snap for non-glass materials where visual crispness matters.
        if material_id != crate::renderer::material_id::GLASS {
            let scale = self.current_scale_factor();
            let snap = |v: f32| (v * scale).round() / scale;
            rect.x = snap(rect.x);
            rect.y = snap(rect.y);
            rect.width = snap(rect.width);
            rect.height = snap(rect.height);
        }

        let scissor = self.clip_stack.last().copied();

        let material = Self::resolve_material_with_context(material_id, &self.current_draw_material);

        let (translation, scale_transform, rotation, _, _) = self.current_transform();
        let (blur_radius, ior_override, glass_intensity) = if let cvkg_core::DrawMaterial::Glass {
            blur_radius,
            ior_override,
            glass_intensity,
        } = material
        {
            (blur_radius, ior_override, glass_intensity)
        } else {
            (0.0, 0.0, 1.0)
        };

        let current_instance_data = InstanceData {
            translation,
            scale: scale_transform,
            rotation,
            blur_radius,
            ior_override,
            glass_intensity,
        };

        // Batching: check if we need to start a new DrawCall
        // With Texture Array, we no longer need to break batches when the texture changes,
        // as long as they are all part of the same array bind group (Group 0).
        // CRITICAL FIX: Only break batch on material/scissor/blur/glass state changes.
        // Transform (translation/scale/rotation) is per-instance data and should NOT
        // break the batch - multiple instances with different transforms can share a draw call.
        let last_call = self.draw_calls.last();
        let needs_new_call = self.draw_calls.is_empty()
            || last_call.unwrap().scissor_rect != scissor
            || last_call.unwrap().material != material
            || last_call.unwrap().texture_id != self.current_texture_id
            || {
                // Check if glass/blur state changed (these require pipeline changes)
                let last_material = last_call.unwrap().material;
                matches!((material, last_material),
                    (cvkg_core::DrawMaterial::Glass { blur_radius: a, ior_override: b, glass_intensity: c },
                     cvkg_core::DrawMaterial::Glass { blur_radius: d, ior_override: e, glass_intensity: f })
                    if a != d || b != e || c != f)
            };

        if needs_new_call {
            self.current_texture_id = Some(0); // All textures are now in the binding array at Group 0
            self.instance_data.push(current_instance_data);
            self.draw_calls.push(DrawCall {
                target_id: None,
                texture_id: self.current_texture_id,
                scissor_rect: scissor,
                index_start: self.indices.len() as u32,
                index_count: 0,
                instance_count: 1,
                material,
                instance_start: (self.instance_data.len() - 1) as u32,
                draw_order: 0,
            });
        } else {
            // Same batch - add instance data and increment instance count
            self.instance_data.push(current_instance_data);
            if let Some(call) = self.draw_calls.last_mut() {
                call.instance_count += 1;
            }
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
        let clip_rect = self.clip_stack.last().copied().unwrap_or(cvkg_core::Rect {
            x: -10000.0,
            y: -10000.0,
            width: 20000.0,
            height: 20000.0,
        });
        let clip = [clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height];

        let tex_index = texture_id.unwrap_or(0);

        self.vertices.push(Vertex {
            position: [x1, y1, z],
            normal,
            uv: [uv_rect.x, uv_rect.y],
            color,
            material_id,
            radius,
            slice,
            logical: [0.0, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y1, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y],
            color,
            material_id,
            radius,
            slice,
            logical: [rect.width, 0.0],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x2, y2, z],
            normal,
            uv: [uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height],
            color,
            material_id,
            radius,
            slice,
            logical: [rect.width, rect.height],
            size: [rect.width, rect.height],
            clip,
            tex_index,
        });
        self.vertices.push(Vertex {
            position: [x1, y2, z],
            normal,
            uv: [uv_rect.x, uv_rect.y + uv_rect.height],
            color,
            material_id,
            radius,
            slice,
            logical: [0.0, rect.height],
            size: [rect.width, rect.height],
            clip,
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
}
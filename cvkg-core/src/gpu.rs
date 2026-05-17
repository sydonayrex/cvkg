//! GPU batching interface for efficient draw-call submission.
//!
//! The `GpuBatch` trait and `BatchRenderer` abstraction allow the renderer to
//! collect multiple draw operations into a single GPU submission, reducing
//! API overhead and enabling instanced rendering.

use crate::Rect;
use bytemuck::{Pod, Zeroable};

/// A 2D instance transform for GPU instanced rendering.
///
/// Encodes translation, scale, and rotation as a compact 8-float array
/// that maps directly to a 3x3 affine transform matrix in column-major order:
/// ```text
/// | mat[0] mat[2] mat[4] |
/// | mat[1] mat[3] mat[5] |
/// |   0      0      1    |
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceTransform {
    /// Column-major 2x2 scale+rotation matrix (mat[0..3]) plus translation (mat[4..5]).
    pub mat: [f32; 6],
}

impl InstanceTransform {
    /// Creates an identity transform (no translation, no scale, no rotation).
    pub fn identity() -> Self {
        Self {
            mat: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    /// Creates a translation-only transform.
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            mat: [1.0, 0.0, 0.0, 1.0, x, y],
        }
    }

    /// Creates a uniform scale transform centered at the origin.
    pub fn scale(s: f32) -> Self {
        Self {
            mat: [s, 0.0, 0.0, s, 0.0, 0.0],
        }
    }

    /// Creates a rotation transform (angle in radians) about the origin.
    pub fn rotation(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            mat: [c, s, -s, c, 0.0, 0.0],
        }
    }

    /// Creates a combined translate + uniform scale transform.
    pub fn translate_scale(x: f32, y: f32, s: f32) -> Self {
        Self {
            mat: [s, 0.0, 0.0, s, x, y],
        }
    }

    /// Creates a combined translate + rotation + uniform scale transform.
    pub fn trs(x: f32, y: f32, angle: f32, s: f32) -> Self {
        let c = angle.cos() * s;
        let sn = angle.sin() * s;
        Self {
            mat: [c, sn, -sn, c, x, y],
        }
    }
}

impl Default for InstanceTransform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Per-instance color data for GPU instanced rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceColor {
    pub rgba: [f32; 4],
}

impl InstanceColor {
    /// Creates an instance color from RGBA values in [0.0, 1.0].
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { rgba: [r, g, b, a] }
    }
}

impl From<[f32; 4]> for InstanceColor {
    fn from(rgba: [f32; 4]) -> Self {
        Self { rgba }
    }
}

/// A batch of instanced draw data for a single GPU draw call.
///
/// Collects transforms, colors, and an optional shared vertex buffer
/// into one structure that the renderer can submit as a single instanced draw.
#[derive(Debug)]
pub struct DrawBatch {
    /// Per-instance transforms.
    pub transforms: Vec<InstanceTransform>,
    /// Per-instance colors (must be same length as transforms, or empty for uniform color).
    pub colors: Vec<InstanceColor>,
    /// Optional per-vertex data for the base geometry (e.g. a unit quad).
    pub vertices: Vec<[f32; 2]>,
    /// Triangle index buffer into the vertex array.
    pub indices: Vec<u32>,
    /// Scissor rect to apply during this batch, or None for no scissor.
    pub scissor: Option<Rect>,
    /// Z-index for depth ordering.
    pub z_index: f32,
}

impl DrawBatch {
    /// Creates an empty draw batch.
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
            colors: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            scissor: None,
            z_index: 0.0,
        }
    }

    /// Creates a batch pre-allocated for `capacity` instances.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            transforms: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            vertices: Vec::new(),
            indices: Vec::new(),
            scissor: None,
            z_index: 0.0,
        }
    }

    /// Sets the scissor rectangle for this batch.
    pub fn with_scissor(mut self, rect: Rect) -> Self {
        self.scissor = Some(rect);
        self
    }

    /// Sets the Z-index for this batch.
    pub fn with_z_index(mut self, z: f32) -> Self {
        self.z_index = z;
        self
    }

    /// Adds an instance with a transform and optional color.
    pub fn push_instance(&mut self, transform: InstanceTransform, color: Option<InstanceColor>) {
        self.transforms.push(transform);
        if let Some(c) = color {
            self.colors.push(c);
        }
    }

    /// Sets the shared vertex geometry for this batch (e.g. a unit quad).
    pub fn set_vertices(&mut self, vertices: &[[f32; 2]], indices: &[u32]) {
        self.vertices = vertices.to_vec();
        self.indices = indices.to_vec();
    }

    /// Returns the number of instances in this batch.
    pub fn instance_count(&self) -> usize {
        self.transforms.len()
    }

    /// Returns true if this batch has no instances.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Clears all instances while retaining allocated capacity.
    pub fn clear(&mut self) {
        self.transforms.clear();
        self.colors.clear();
    }

    /// Validates that colors length matches transforms length if colors are present.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.colors.is_empty() && self.colors.len() != self.transforms.len() {
            return Err("colors length must match transforms length");
        }
        if self.vertices.is_empty() {
            return Err("vertices must not be empty");
        }
        if self.indices.is_empty() {
            return Err("indices must not be empty");
        }
        Ok(())
    }
}

impl Default for DrawBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard unit-quad vertex data for instanced rendering.
///
/// Returns `(vertices, indices)` for a unit quad centered at the origin
/// with corners at (-0.5, -0.5) to (0.5, 0.5).
pub fn unit_quad() -> (Vec<[f32; 2]>, Vec<u32>) {
    (
        vec![[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
        vec![0, 1, 2, 0, 2, 3],
    )
}

/// Standard unit-circle vertex data approximated with a regular polygon.
///
/// `segments` controls the tessellation quality (minimum 8).
pub fn unit_circle(segments: usize) -> (Vec<[f32; 2]>, Vec<u32>) {
    let n = segments.max(8);
    let mut vertices = Vec::with_capacity(n);
    let mut indices = Vec::with_capacity(n * 3);

    for i in 0..n {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
        vertices.push([angle.cos(), angle.sin()]);
    }

    // Triangle fan from center (vertex 0)
    for i in 1..n - 1 {
        indices.push(0);
        indices.push(i as u32);
        indices.push((i + 1) as u32);
    }
    indices.push(0);
    indices.push((n - 1) as u32);
    indices.push(1);

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_transform_identity() {
        let t = InstanceTransform::identity();
        assert_eq!(t.mat[0], 1.0);
        assert_eq!(t.mat[3], 1.0);
    }

    #[test]
    fn instance_transform_translation() {
        let t = InstanceTransform::translation(10.0, 20.0);
        assert_eq!(t.mat[4], 10.0);
        assert_eq!(t.mat[5], 20.0);
    }

    #[test]
    fn instance_transform_trs() {
        let t = InstanceTransform::trs(5.0, 10.0, std::f32::consts::PI / 2.0, 2.0);
        // 90 degree rotation + scale 2 = [0, 2, -2, 0, 5, 10]
        assert!((t.mat[0]).abs() < 0.001);
        assert!((t.mat[1] - 2.0).abs() < 0.001);
        assert!((t.mat[2] + 2.0).abs() < 0.001);
        assert!((t.mat[3]).abs() < 0.001);
        assert_eq!(t.mat[4], 5.0);
        assert_eq!(t.mat[5], 10.0);
    }

    #[test]
    fn draw_batch_push_and_clear() {
        let mut batch = DrawBatch::new();
        assert!(batch.is_empty());

        let (verts, inds) = unit_quad();
        batch.set_vertices(&verts, &inds);
        batch.push_instance(InstanceTransform::translation(100.0, 200.0), None);
        batch.push_instance(
            InstanceTransform::translation(300.0, 400.0),
            Some(InstanceColor::new(1.0, 0.0, 0.0, 1.0)),
        );

        assert_eq!(batch.instance_count(), 2);
        assert!(batch.validate().is_ok());

        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn draw_batch_validate_colors_mismatch() {
        let mut batch = DrawBatch::new();
        let (verts, inds) = unit_quad();
        batch.set_vertices(&verts, &inds);
        batch.push_instance(
            InstanceTransform::identity(),
            Some(InstanceColor::new(1.0, 0.0, 0.0, 1.0)),
        );
        batch.push_instance(InstanceTransform::identity(), None);
        assert!(batch.validate().is_err());
    }

    #[test]
    fn unit_quad_geometry() {
        let (verts, inds) = unit_quad();
        assert_eq!(verts.len(), 4);
        assert_eq!(inds.len(), 6);
    }

    #[test]
    fn unit_circle_geometry() {
        let (verts, inds) = unit_circle(16);
        assert_eq!(verts.len(), 16);
        assert_eq!(inds.len(), 16 * 3);
    }
}

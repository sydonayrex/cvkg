//! Heightmap / terrain collision shape.
//!
//! A heightmap shape samples a 2D grid of height values to produce a 3D
//! collision surface. Useful for landscape UI, scrollable terrain maps,
//! and 3D ground planes with elevation data.

use glam::{Vec2, Vec3};

use crate::Shape;

/// A heightmap collision shape.
///
/// Stores a regular grid of height values. The shape covers a rectangular
/// region in the XZ plane (Y is up). Collision queries sample the height
/// at a given (x, z) position and compare against the query point's y.
#[derive(Debug, Clone, PartialEq)]
pub struct HeightmapShape {
    /// Width of the heightmap grid (number of columns).
    pub width: usize,
    /// Depth of the heightmap grid (number of rows).
    pub depth: usize,
    /// Physical size in world units (x = width direction, z = depth direction).
    pub world_size: Vec2,
    /// Height values stored row-major: data[z * width + x].
    pub heights: Vec<f32>,
    /// Minimum height value (for AABB).
    pub min_height: f32,
    /// Maximum height value (for AABB).
    pub max_height: f32,
}

impl HeightmapShape {
    /// Create a heightmap from a flat grid of height values.
    ///
    /// # Arguments
    /// * `heights` - Row-major height data, must have exactly `width * depth` elements.
    /// * `world_size` - Physical size in world units (x, z).
    pub fn new(heights: Vec<f32>, width: usize, depth: usize, world_size: Vec2) -> Self {
        assert_eq!(
            heights.len(),
            width * depth,
            "Height data length must equal width * depth"
        );
        let min_height = heights.iter().copied().fold(f32::INFINITY, f32::min);
        let max_height = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        Self {
            width,
            depth,
            world_size,
            heights,
            min_height,
            max_height,
        }
    }

    /// Create a flat heightmap at y=0.
    pub fn flat(width: usize, depth: usize, world_size: Vec2) -> Self {
        Self::new(vec![0.0; width * depth], width, depth, world_size)
    }

    /// Create a heightmap from a procedural function.
    ///
    /// `height_fn(x, z)` returns the height at grid position (x, z).
    ///
    /// # Example: sine wave terrain
    /// ```
    /// use cvkg_physics::heightmap::HeightmapShape;
    /// use glam::Vec2;
    /// let terrain = HeightmapShape::from_fn(64, 64, Vec2::new(100.0, 100.0), |x, z| {
    ///     (x as f32 * 0.1).sin() * (z as f32 * 0.1).cos() * 5.0
    /// });
    /// ```
    pub fn from_fn<F>(width: usize, depth: usize, world_size: Vec2, height_fn: F) -> Self
    where
        F: Fn(usize, usize) -> f32,
    {
        let mut heights = Vec::with_capacity(width * depth);
        for z in 0..depth {
            for x in 0..width {
                heights.push(height_fn(x, z));
            }
        }
        Self::new(heights, width, depth, world_size)
    }

    /// Sample the height at a world-space (x, z) position using bilinear interpolation.
    /// Returns None if the position is outside the heightmap bounds.
    pub fn sample_height(&self, world_x: f32, world_z: f32) -> Option<f32> {
        let half_w = self.world_size.x * 0.5;
        let half_d = self.world_size.y * 0.5;

        // Map world coordinates to grid coordinates
        let gx = (world_x + half_w) / self.world_size.x * (self.width as f32 - 1.0);
        let gz = (world_z + half_d) / self.world_size.y * (self.depth as f32 - 1.0);

        if gx < 0.0 || gz < 0.0 || gx > (self.width - 1) as f32 || gz > (self.depth - 1) as f32 {
            return None;
        }

        let x0 = gx.floor() as usize;
        let z0 = gz.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.depth - 1);

        let fx = gx - x0 as f32;
        let fz = gz - z0 as f32;

        let h00 = self.heights[z0 * self.width + x0];
        let h10 = self.heights[z0 * self.width + x1];
        let h01 = self.heights[z1 * self.width + x0];
        let h11 = self.heights[z1 * self.width + x1];

        // Bilinear interpolation
        let h = h00 * (1.0 - fx) * (1.0 - fz)
            + h10 * fx * (1.0 - fz)
            + h01 * (1.0 - fx) * fz
            + h11 * fx * fz;

        Some(h)
    }

    /// Get the bounding radius for this heightmap.
    pub fn bounding_radius(&self) -> f32 {
        let half_diag =
            (self.world_size.x * self.world_size.x + self.world_size.y * self.world_size.y).sqrt()
                * 0.5;
        let height_range = (self.max_height - self.min_height).abs();
        (half_diag * half_diag + height_range * height_range).sqrt()
    }

    /// Get the axis-aligned bounding box.
    pub fn aabb(&self) -> (Vec3, Vec3) {
        (
            Vec3::new(
                -self.world_size.x * 0.5,
                self.min_height,
                -self.world_size.y * 0.5,
            ),
            Vec3::new(
                self.world_size.x * 0.5,
                self.max_height,
                self.world_size.y * 0.5,
            ),
        )
    }

    /// Test if a point is below the heightmap surface (inside terrain).
    pub fn contains_point(&self, point: Vec3) -> bool {
        if let Some(height) = self.sample_height(point.x, point.z) {
            point.y < height
        } else {
            false
        }
    }

    /// Get the surface normal at a world position using finite differences.
    pub fn surface_normal(&self, world_x: f32, world_z: f32) -> Vec3 {
        let eps = self.world_size.x / self.width as f32;
        let hx = self.sample_height(world_x + eps, world_z).unwrap_or(0.0)
            - self.sample_height(world_x - eps, world_z).unwrap_or(0.0);
        let hz = self.sample_height(world_x, world_z + eps).unwrap_or(0.0)
            - self.sample_height(world_x, world_z - eps).unwrap_or(0.0);

        // Normal from gradient
        let normal = Vec3::new(-hx, 2.0 * eps, -hz);
        let len = normal.length();
        if len > 1e-8 { normal / len } else { Vec3::Y }
    }

    /// Depenetrate a point from the heightmap surface.
    /// Returns the correction vector needed to push the point above the surface.
    pub fn depenetrate(&self, point: Vec3) -> Option<Vec3> {
        if let Some(height) = self.sample_height(point.x, point.z) {
            if point.y < height {
                let normal = self.surface_normal(point.x, point.z);
                let penetration = height - point.y;
                Some(normal * penetration)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl HeightmapShape {
    /// Create a `Shape::Heightmap` variant from this heightmap.
    pub fn to_shape(self) -> Shape {
        Shape::heightmap(self.heights, self.width, self.depth, self.world_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_heightmap() {
        let hm = HeightmapShape::flat(8, 8, Vec2::new(100.0, 100.0));
        assert_eq!(hm.sample_height(0.0, 0.0), Some(0.0));
        assert_eq!(hm.sample_height(50.0, 50.0), Some(0.0));
        assert_eq!(hm.sample_height(200.0, 200.0), None); // Out of bounds
    }

    #[test]
    fn test_procedural_heightmap() {
        let hm = HeightmapShape::from_fn(16, 16, Vec2::new(100.0, 100.0), |x, z| {
            (x as f32) * 0.5 + (z as f32) * 0.3
        });

        // Height should increase with x and z
        let h_center = hm.sample_height(0.0, 0.0).unwrap();
        let h_far = hm.sample_height(40.0, 40.0).unwrap();
        assert!(h_far > h_center);
    }

    #[test]
    fn test_aabb() {
        let hm = HeightmapShape::from_fn(8, 8, Vec2::new(100.0, 100.0), |x, z| (x as f32) * 2.0);
        let (min, max) = hm.aabb();
        assert_eq!(min.x, -50.0);
        assert_eq!(max.x, 50.0);
        assert!(min.y <= 0.0);
        assert!(max.y >= 14.0);
    }

    #[test]
    fn test_surface_normal() {
        let hm = HeightmapShape::flat(16, 16, Vec2::new(100.0, 100.0));
        let normal = hm.surface_normal(0.0, 0.0);
        // Flat surface should have normal pointing up
        assert!((normal.y - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_depenetrate() {
        let hm = HeightmapShape::flat(8, 8, Vec2::new(100.0, 100.0));
        // Point below surface
        let correction = hm.depenetrate(Vec3::new(0.0, -5.0, 0.0));
        assert!(correction.is_some());
        let c = correction.unwrap();
        assert!(c.y > 0.0); // Should push up

        // Point above surface
        let correction2 = hm.depenetrate(Vec3::new(0.0, 5.0, 0.0));
        assert!(correction2.is_none());
    }

    #[test]
    fn test_bounding_radius() {
        let hm = HeightmapShape::flat(8, 8, Vec2::new(100.0, 100.0));
        let r = hm.bounding_radius();
        // Should be at least half the diagonal
        assert!(r > 70.0);
    }

    #[test]
    fn test_bilinear_interpolation() {
        // Create a 2x2 heightmap with known values
        let heights = vec![0.0, 10.0, 10.0, 20.0];
        let hm = HeightmapShape::new(heights, 2, 2, Vec2::new(100.0, 100.0));

        // Center should be average
        let h = hm.sample_height(0.0, 0.0).unwrap();
        assert!((h - 10.0).abs() < 0.01);
    }
}

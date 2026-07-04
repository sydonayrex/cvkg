//! Frustum culling for 3D meshes.
//!
//! Wraps `cvkg_spatial::frustum::Frustum` with a convenient API for
//! testing meshes against the camera frustum. The actual GPU-side draw
//! loop uses `is_visible()` to skip meshes that are entirely outside
//! the view frustum.

use cvkg_spatial::frustum::Frustum;
use glam::{Mat4, Vec3};

/// Frustum culler — tests AABBs against the camera frustum.
#[derive(Debug, Clone)]
pub struct FrustumCuller {
    /// The underlying frustum planes.
    pub frustum: Frustum,
}

impl FrustumCuller {
    /// Create a new culler from a view-projection matrix.
    pub fn from_view_projection(vp: &Mat4) -> Self {
        Self {
            frustum: Frustum::from_view_projection(vp),
        }
    }

    /// Create a culler from an existing Frustum.
    pub fn new(frustum: Frustum) -> Self {
        Self { frustum }
    }

    /// Test if an AABB (center + half-extents) is potentially visible.
    /// Returns true if the box intersects or is inside the frustum.
    pub fn is_visible(&self, center: Vec3, half_extents: Vec3) -> bool {
        self.frustum.intersects_aabb(center, half_extents)
    }

    /// Test if a sphere (center + radius) is potentially visible.
    pub fn is_sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        self.frustum.intersects_sphere(center, radius)
    }
}

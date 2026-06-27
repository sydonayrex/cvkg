/// A frustum for visibility culling.
#[derive(Clone, Debug)]
pub struct Frustum {
    /// Planes: [normal_x, normal_y, normal_z, distance]
    pub planes: [[f32; 4]; 6],
}

impl Frustum {
    /// Create a frustum from a view-projection matrix.
    pub fn from_view_proj(view_proj: &[[f32; 4]; 4]) -> Self {
        let mut planes = [[0.0f32; 4]; 6];
        let m = view_proj;

        // Left plane
        planes[0] = [
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        ];
        // Right plane
        planes[1] = [
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        ];
        // Top plane
        planes[2] = [
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        ];
        // Bottom plane
        planes[3] = [
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        ];
        // Near plane
        planes[4] = [
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        ];
        // Far plane
        planes[5] = [
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        ];

        // Normalize planes
        for plane in &mut planes {
            let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            if len > 0.0 {
                plane[0] /= len;
                plane[1] /= len;
                plane[2] /= len;
                plane[3] /= len;
            }
        }

        Self { planes }
    }

    /// Test if an axis-aligned bounding box is visible within this frustum.
    pub fn intersects_aabb(&self, min: &[f32; 3], max: &[f32; 3]) -> bool {
        for plane in &self.planes {
            // Find the p-vertex (the corner most in the direction of the plane normal)
            let px = if plane[0] > 0.0 { max[0] } else { min[0] };
            let py = if plane[1] > 0.0 { max[1] } else { min[1] };
            let pz = if plane[2] > 0.0 { max[2] } else { min[2] };

            // If the p-vertex is behind the plane, the entire AABB is outside
            if plane[0] * px + plane[1] * py + plane[2] * pz + plane[3] < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is visible within this frustum.
    pub fn intersects_sphere(&self, center: &[f32; 3], radius: f32) -> bool {
        for plane in &self.planes {
            let dist =
                plane[0] * center[0] + plane[1] * center[1] + plane[2] * center[2] + plane[3];
            if dist < -radius {
                return false;
            }
        }
        true
    }
}

/// Spatial hash cell coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpatialCell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Spatial hash for scene virtualization.
#[derive(Clone, Debug)]
pub struct SpatialHash {
    cell_size: f32,
    cells: std::collections::HashMap<SpatialCell, Vec<u64>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
        }
    }

    /// Insert an entity into the spatial hash.
    pub fn insert(&mut self, entity_id: u64, position: &[f32; 3]) {
        let cell = self.world_to_cell(position);
        self.cells.entry(cell).or_default().push(entity_id);
    }

    /// Remove an entity from the spatial hash.
    pub fn remove(&mut self, entity_id: u64, position: &[f32; 3]) {
        let cell = self.world_to_cell(position);
        if let Some(entities) = self.cells.get_mut(&cell) {
            entities.retain(|&id| id != entity_id);
            if entities.is_empty() {
                self.cells.remove(&cell);
            }
        }
    }

    /// Query entities within a frustum.
    pub fn query_frustum(&self, frustum: &Frustum) -> Vec<u64> {
        let mut results = Vec::new();
        // Check all occupied cells against the frustum
        for (cell, entities) in &self.cells {
            // Convert cell coordinates to world-space AABB
            let min = [
                cell.x as f32 * self.cell_size,
                cell.y as f32 * self.cell_size,
                cell.z as f32 * self.cell_size,
            ];
            let max = [
                min[0] + self.cell_size,
                min[1] + self.cell_size,
                min[2] + self.cell_size,
            ];
            if frustum.intersects_aabb(&min, &max) {
                results.extend(entities);
            }
        }
        results
    }

    /// Query entities within a sphere.
    pub fn query_sphere(&self, center: &[f32; 3], radius: f32) -> Vec<u64> {
        let mut results = Vec::new();
        // Check cells that could contain entities within the sphere
        let min_cell =
            self.world_to_cell(&[center[0] - radius, center[1] - radius, center[2] - radius]);
        let max_cell =
            self.world_to_cell(&[center[0] + radius, center[1] + radius, center[2] + radius]);

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = SpatialCell { x, y, z };
                    if let Some(entities) = self.cells.get(&cell) {
                        results.extend(entities);
                    }
                }
            }
        }
        results
    }

    fn world_to_cell(&self, position: &[f32; 3]) -> SpatialCell {
        SpatialCell {
            x: (position[0] / self.cell_size).floor() as i32,
            y: (position[1] / self.cell_size).floor() as i32,
            z: (position[2] / self.cell_size).floor() as i32,
        }
    }

    /// Clear all cells.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Returns the number of occupied cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

#[cfg(test)]
mod p2_28_virtualization_tests {
    use super::*;

    #[test]
    fn frustum_intersects_aabb_visible() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        assert!(frustum.intersects_aabb(&[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0]));
    }

    #[test]
    fn frustum_intersects_aabb_outside() {
        let frustum = Frustum {
            planes: [
                [0.0, 0.0, -1.0, -10.0], // Near plane at z=-10
                [0.0, 0.0, 1.0, -10.0],  // Far plane
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
        };
        assert!(!frustum.intersects_aabb(&[0.0, 0.0, -11.0], &[1.0, 1.0, -10.5]));
    }

    #[test]
    fn frustum_intersects_sphere() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        assert!(frustum.intersects_sphere(&[0.0, 0.0, 0.0], 1.0));
    }

    #[test]
    fn spatial_hash_insert_and_query() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[15.0, 5.0, 0.0]);
        assert_eq!(hash.len(), 2);
    }

    #[test]
    fn spatial_hash_query_frustum() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[50.0, 50.0, 0.0]);

        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let frustum = Frustum::from_view_proj(&identity);
        let results = hash.query_frustum(&frustum);
        assert!(!results.is_empty());
    }

    #[test]
    fn spatial_hash_remove() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.remove(1, &[5.0, 5.0, 0.0]);
        assert_eq!(hash.len(), 0);
    }

    #[test]
    fn spatial_hash_clear() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(1, &[5.0, 5.0, 0.0]);
        hash.insert(2, &[15.0, 5.0, 0.0]);
        hash.clear();
        assert!(hash.is_empty());
    }
}

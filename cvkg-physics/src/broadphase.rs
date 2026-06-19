//! Spatial hashing broad-phase collision detection.
//!
//! Provides both 2D and 3D spatial hash grids. The 2D variant uses `(i32, i32)`
//! cell keys and `Vec2` AABBs; the 3D variant uses `(i32, i32, i32)` keys and
//! `Vec3` AABBs.

use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::BodyId;

/// Cell size for the spatial hash grid.
const DEFAULT_CELL_SIZE: f32 = 64.0;

/// Spatial hash grid for fast broad-phase collision culling (2D).
///
/// Maps bodies to grid cells based on their AABB. Querying returns
/// candidate pairs that need narrow-phase testing.
#[derive(Debug, Default)]
pub struct SpatialHash {
    cells: HashMap<(i32, i32), Vec<BodyId>>,
    /// Cell size for the spatial hash grid. Tuning this to match your
    /// typical collider size improves broadphase performance.
    pub cell_size: f32,
}

impl SpatialHash {
    /// Create a new empty spatial hash.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            cell_size: DEFAULT_CELL_SIZE,
        }
    }

    /// Create a spatial hash with a custom cell size.
    pub fn with_cell_size(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size: cell_size.max(1.0),
        }
    }

    /// Clear all cells.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Insert a body into the spatial hash given its 2D AABB.
    pub fn insert(&mut self, body_id: BodyId, min: Vec2, max: Vec2) {
        let cs = self.cell_size;
        let min_cell = ((min.x / cs).floor() as i32, (min.y / cs).floor() as i32);
        let max_cell = ((max.x / cs).floor() as i32, (max.y / cs).floor() as i32);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                self.cells.entry((x, y)).or_default().push(body_id);
            }
        }
    }

    /// Query all body IDs that might overlap the given 2D AABB.
    pub fn query(&self, min: Vec2, max: Vec2) -> Vec<BodyId> {
        let mut result = Vec::new();
        let min_cell = Self::world_to_cell_2d(min, self.cell_size);
        let max_cell = Self::world_to_cell_2d(max, self.cell_size);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(ids) = self.cells.get(&(x, y)) {
                    result.extend(ids);
                }
            }
        }
        result
    }

    /// Generate all candidate collision pairs from the spatial hash.
    pub fn candidate_pairs(&self) -> Vec<(BodyId, BodyId)> {
        let mut pairs = Vec::new();
        let mut seen = HashMap::new();

        for ids in self.cells.values() {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = ids[i].0;
                    let b = ids[j].0;
                    let key = if a < b { (a, b) } else { (b, a) };
                    if seen.insert(key, true).is_none() {
                        pairs.push((ids[i], ids[j]));
                    }
                }
            }
        }
        pairs
    }

    fn world_to_cell_2d(pos: Vec2, cell_size: f32) -> (i32, i32) {
        (
            (pos.x / cell_size).floor() as i32,
            (pos.y / cell_size).floor() as i32,
        )
    }

    /// Get the current cell size.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Set the cell size. Clears existing entries.
    pub fn set_cell_size(&mut self, size: f32) {
        self.cell_size = size.max(1.0);
        self.clear();
    }
}

/// 3-D spatial hash grid for fast broad-phase collision culling.
///
/// Extends the 2D concept with a third axis. Cell keys are `(i32, i32, i32)`.
#[derive(Debug, Default)]
pub struct SpatialHash3D {
    cells: HashMap<(i32, i32, i32), Vec<BodyId>>,
    /// Cell size for each axis.
    pub cell_size: f32,
}

impl SpatialHash3D {
    /// Create a new empty 3D spatial hash.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            cell_size: DEFAULT_CELL_SIZE,
        }
    }

    /// Create a 3D spatial hash with a custom cell size.
    pub fn with_cell_size(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size: cell_size.max(1.0),
        }
    }

    /// Clear all cells.
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Insert a body into the spatial hash given its 3D AABB.
    pub fn insert(&mut self, body_id: BodyId, min: Vec3, max: Vec3) {
        let cs = self.cell_size;
        let min_cell = (
            (min.x / cs).floor() as i32,
            (min.y / cs).floor() as i32,
            (min.z / cs).floor() as i32,
        );
        let max_cell = (
            (max.x / cs).floor() as i32,
            (max.y / cs).floor() as i32,
            (max.z / cs).floor() as i32,
        );

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    self.cells.entry((x, y, z)).or_default().push(body_id);
                }
            }
        }
    }

    /// Query all body IDs that might overlap the given 3D AABB.
    pub fn query(&self, min: Vec3, max: Vec3) -> Vec<BodyId> {
        let mut result = Vec::new();
        let cs = self.cell_size;
        let min_cell = (
            (min.x / cs).floor() as i32,
            (min.y / cs).floor() as i32,
            (min.z / cs).floor() as i32,
        );
        let max_cell = (
            (max.x / cs).floor() as i32,
            (max.y / cs).floor() as i32,
            (max.z / cs).floor() as i32,
        );

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    if let Some(ids) = self.cells.get(&(x, y, z)) {
                        result.extend(ids);
                    }
                }
            }
        }
        result
    }

    /// Generate all candidate collision pairs from the spatial hash.
    pub fn candidate_pairs(&self) -> Vec<(BodyId, BodyId)> {
        let mut pairs = Vec::new();
        let mut seen = HashMap::new();

        for ids in self.cells.values() {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = ids[i].0;
                    let b = ids[j].0;
                    let key = if a < b { (a, b) } else { (b, a) };
                    if seen.insert(key, true).is_none() {
                        pairs.push((ids[i], ids[j]));
                    }
                }
            }
        }
        pairs
    }

    /// Get the current cell size.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Set the cell size. Clears existing entries.
    pub fn set_cell_size(&mut self, size: f32) {
        self.cell_size = size.max(1.0);
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut grid = SpatialHash::new();
        grid.insert(BodyId(1), Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0));
        grid.insert(BodyId(2), Vec2::new(10.0, 10.0), Vec2::new(42.0, 42.0));
        grid.insert(BodyId(3), Vec2::new(200.0, 200.0), Vec2::new(232.0, 232.0));

        let result = grid.query(Vec2::new(0.0, 0.0), Vec2::new(64.0, 64.0));
        assert!(result.contains(&BodyId(1)));
        assert!(result.contains(&BodyId(2)));
        assert!(!result.contains(&BodyId(3)));
    }

    #[test]
    fn test_candidate_pairs() {
        let mut grid = SpatialHash::new();
        grid.insert(BodyId(1), Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0));
        grid.insert(BodyId(2), Vec2::new(10.0, 10.0), Vec2::new(42.0, 42.0));
        grid.insert(BodyId(3), Vec2::new(200.0, 200.0), Vec2::new(232.0, 232.0));

        let pairs = grid.candidate_pairs();
        // Bodies 1 and 2 share a cell; body 3 is in its own cell
        assert!(pairs.iter().any(|(a, b)| {
            (*a == BodyId(1) && *b == BodyId(2)) || (*a == BodyId(2) && *b == BodyId(1))
        }));
    }

    // ── 3D broadphase tests ──────────────────────────────────────────────

    #[test]
    fn test_3d_insert_and_query() {
        let mut grid = SpatialHash3D::new();
        grid.insert(
            BodyId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(32.0, 32.0, 32.0),
        );
        grid.insert(
            BodyId(2),
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(42.0, 42.0, 42.0),
        );
        grid.insert(
            BodyId(3),
            Vec3::new(200.0, 200.0, 200.0),
            Vec3::new(232.0, 232.0, 232.0),
        );

        let result = grid.query(Vec3::new(0.0, 0.0, 0.0), Vec3::new(64.0, 64.0, 64.0));
        assert!(result.contains(&BodyId(1)));
        assert!(result.contains(&BodyId(2)));
        assert!(!result.contains(&BodyId(3)));
    }

    #[test]
    fn test_3d_candidate_pairs() {
        let mut grid = SpatialHash3D::new();
        grid.insert(
            BodyId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(32.0, 32.0, 32.0),
        );
        grid.insert(
            BodyId(2),
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(42.0, 42.0, 42.0),
        );
        grid.insert(
            BodyId(3),
            Vec3::new(200.0, 200.0, 200.0),
            Vec3::new(232.0, 232.0, 232.0),
        );

        let pairs = grid.candidate_pairs();
        assert!(pairs.iter().any(|(a, b)| {
            (*a == BodyId(1) && *b == BodyId(2)) || (*a == BodyId(2) && *b == BodyId(1))
        }));
    }

    #[test]
    fn test_3d_z_separation() {
        let mut grid = SpatialHash3D::new();
        // Two bodies at same XY but far apart in Z
        grid.insert(
            BodyId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(32.0, 32.0, 32.0),
        );
        grid.insert(
            BodyId(2),
            Vec3::new(0.0, 0.0, 200.0),
            Vec3::new(32.0, 32.0, 232.0),
        );

        let pairs = grid.candidate_pairs();
        assert!(pairs.is_empty());
    }
}

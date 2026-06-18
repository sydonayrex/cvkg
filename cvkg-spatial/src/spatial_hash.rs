//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//  Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//  CVKG Extended: Section 2 of the CVKG Design Specification

//! Generic spatial hash grid for O(1) insert and O(k) region queries.
//!
//! # Why this exists
//! Finding #5 from the crosscrate audit: Scene, Physics, and Flow each maintained
//! hand-rolled spatial grids with no deduplication or shared contract. This
//! generic implementation replaces all of them.
//!
//! # Algorithm
//! Space is divided into a uniform grid of square cells of size `cell_size`.
//! Each inserted item is registered in every cell its bounding rect overlaps.
//! Queries collect items from all cells the query rect overlaps — O(1) amortized
//! insert, O(k) query where k = items per cell (typically small for uniform data).
//!
//! # Limitation
//! Items are stored by value (cloned on insert). There is no removal API — to
//! update positions, call `clear()` and re-insert. This is appropriate for
//! per-frame rebuild patterns used by CVKG's retained scene graph.

use cvkg_core::Rect;
use std::collections::HashMap;

/// A uniform-grid spatial hash map keyed by `(cell_x, cell_y)`.
///
/// `T` must be `Clone` because a single large item may be registered in
/// multiple cells simultaneously.
pub struct SpatialHash<T> {
    /// Backing storage: each cell holds a vec of items that touch it.
    cells: HashMap<(i32, i32), Vec<T>>,
    /// Edge length of each square grid cell in world-space units.
    cell_size: f32,
    /// Total number of individual cell-item registrations (not unique items).
    registration_count: usize,
}

impl<T: Clone> SpatialHash<T> {
    /// Create an empty spatial hash with the given cell size.
    ///
    /// `cell_size` should be chosen to match the typical object size: too small
    /// wastes memory, too large reduces query selectivity. 64.0 px is a good
    /// default for UI graphs; use smaller values for dense physics simulations.
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "cell_size must be positive");
        Self {
            cells: HashMap::new(),
            cell_size,
            registration_count: 0,
        }
    }

    /// Insert `item` into every cell that `rect` overlaps.
    ///
    /// The item is cloned once per overlapping cell. For items that span many
    /// cells this can be expensive — prefer large `cell_size` for large objects.
    pub fn insert(&mut self, rect: Rect, item: T) {
        let (min_cx, min_cy, max_cx, max_cy) = self.cells_for_rect(rect);
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                self.cells.entry((cx, cy)).or_default().push(item.clone());
                self.registration_count += 1;
            }
        }
    }

    /// Return all items registered in cells that overlap `rect`.
    ///
    /// The returned list may contain duplicates if an item spans multiple cells
    /// that all overlap the query rect. Callers must deduplicate if needed.
    /// This is intentionally a broad-phase query — exact AABB testing is the
    /// caller's responsibility.
    pub fn query(&self, rect: Rect) -> Vec<T> {
        let (min_cx, min_cy, max_cx, max_cy) = self.cells_for_rect(rect);
        let mut results = Vec::new();
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                if let Some(cell) = self.cells.get(&(cx, cy)) {
                    results.extend_from_slice(cell);
                }
            }
        }
        results
    }

    /// Remove all items from every cell, resetting to an empty grid.
    ///
    /// This does NOT free the backing HashMap's allocated memory; call this at
    /// the start of each frame before re-inserting the current object set.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.registration_count = 0;
    }

    /// Return the number of cell-item registrations (not unique item count).
    ///
    /// This is useful for profiling and capacity planning. An item spanning
    /// N cells contributes N to this count.
    pub fn len(&self) -> usize {
        self.registration_count
    }

    /// Returns `true` if no items have been inserted since the last `clear()`.
    pub fn is_empty(&self) -> bool {
        self.registration_count == 0
    }

    /// Compute the inclusive cell range `(min_cx, min_cy, max_cx, max_cy)` for a rect.
    fn cells_for_rect(&self, rect: Rect) -> (i32, i32, i32, i32) {
        let min_cx = (rect.x / self.cell_size).floor() as i32;
        let min_cy = (rect.y / self.cell_size).floor() as i32;
        let max_cx = ((rect.x + rect.width) / self.cell_size).floor() as i32;
        let max_cy = ((rect.y + rect.height) / self.cell_size).floor() as i32;
        (min_cx, min_cy, max_cx, max_cy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::Rect;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn test_insert_and_query() {
        let mut sh: SpatialHash<u32> = SpatialHash::new(100.0);
        sh.insert(rect(0.0, 0.0, 50.0, 50.0), 1);
        sh.insert(rect(200.0, 200.0, 50.0, 50.0), 2);

        let results = sh.query(rect(0.0, 0.0, 60.0, 60.0));
        assert!(results.contains(&1), "Should find item 1");
        assert!(!results.contains(&2), "Should NOT find item 2");
    }

    #[test]
    fn test_clear_resets_state() {
        let mut sh: SpatialHash<u32> = SpatialHash::new(64.0);
        sh.insert(rect(0.0, 0.0, 10.0, 10.0), 42);
        assert!(!sh.is_empty());

        sh.clear();
        assert!(sh.is_empty());
        assert_eq!(sh.len(), 0);
        let results = sh.query(rect(0.0, 0.0, 10.0, 10.0));
        assert!(results.is_empty());
    }

    #[test]
    fn test_multi_cell_item() {
        // An item spanning 2 cells horizontally should appear in queries to either cell.
        let mut sh: SpatialHash<&str> = SpatialHash::new(100.0);
        // rect spans cells (0,0) and (1,0)
        sh.insert(rect(50.0, 0.0, 100.0, 50.0), "wide");

        let left = sh.query(rect(0.0, 0.0, 60.0, 50.0));
        let right = sh.query(rect(100.0, 0.0, 60.0, 50.0));
        assert!(left.contains(&"wide"), "wide should appear in left cell query");
        assert!(
            right.contains(&"wide"),
            "wide should appear in right cell query"
        );
    }

    #[test]
    fn test_negative_coordinates() {
        let mut sh: SpatialHash<i32> = SpatialHash::new(50.0);
        sh.insert(rect(-100.0, -100.0, 20.0, 20.0), 99);

        let results = sh.query(rect(-110.0, -110.0, 40.0, 40.0));
        assert!(results.contains(&99), "Should handle negative coordinates");
    }
}

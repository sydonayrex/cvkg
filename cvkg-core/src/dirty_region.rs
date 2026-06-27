// =========================================================================
// P1-39: DirtyRegionManager -- tracks changed rectangles
// =========================================================================

use crate::*;

// This is a passive container -- callers add dirty regions
// when they change something, and the renderer can clear
// them after a frame.

/// P1-39: a list of regions that have changed and need to be
/// re-rendered. Coalesces overlapping rectangles on add to
/// avoid unbounded growth.
#[derive(Debug, Clone, Default)]
pub struct DirtyRegionManager {
    /// The dirty rectangles, in screen-space coordinates.
    regions: Vec<Rect>,
    /// Counter incremented on each clear, useful for detecting
    /// "stale" dirty regions after multiple frames.
    generation: u64,
}

impl DirtyRegionManager {
    /// Create a new empty dirty region manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a region as dirty. The region is in screen-space
    /// coordinates (typically the same coordinate system as the
    /// rest of the rendering).
    ///
    /// If `region` overlaps with an existing dirty region, the
    /// two are coalesced into a single larger rectangle. This
    /// prevents the dirty list from growing unbounded for
    /// large UIs with many small changes.
    pub fn mark_dirty(&mut self, region: Rect) {
        // Try to merge with an existing overlapping region.
        for existing in self.regions.iter_mut() {
            if Self::rects_overlap(*existing, region) {
                *existing = Self::union_rect(*existing, region);
                return;
            }
        }
        // No overlap -- add as new region.
        self.regions.push(region);
    }

    /// Get the current dirty regions. The renderer can use
    /// this list to clip drawing to only the changed areas.
    pub fn regions(&self) -> &[Rect] {
        &self.regions
    }

    /// Check if any region is dirty. Useful for skipping a
    /// frame when nothing has changed.
    pub fn is_dirty(&self) -> bool {
        !self.regions.is_empty()
    }

    /// Clear all dirty regions. Called by the renderer after
    /// processing a frame.
    ///
    /// Increments the generation counter so callers can detect
    /// when a clear has happened.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.generation = self.generation.wrapping_add(1);
    }

    /// Get the current generation counter. Increases on every
    /// clear(). Callers can cache this to detect when the
    /// dirty state has been reset.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Number of dirty regions currently tracked. After
    /// coalescing, this should be much smaller than the number
    /// of mark_dirty() calls.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Check whether the dirty region list is empty.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Check if two rectangles overlap.
    fn rects_overlap(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }

    /// Compute the union of two rectangles (the smallest
    /// rectangle that contains both).
    fn union_rect(a: Rect, b: Rect) -> Rect {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = (a.x + a.width).max(b.x + b.width);
        let max_y = (a.y + a.height).max(b.y + b.height);
        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[cfg(test)]
mod p1_39_dirty_region_tests {
    use super::{DirtyRegionManager, Rect};

    #[test]
    fn new_manager_is_empty() {
        let m = DirtyRegionManager::new();
        assert!(!m.is_dirty());
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn mark_dirty_adds_region() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        });
        assert!(m.is_dirty());
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn overlapping_regions_coalesce() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        });
        m.mark_dirty(Rect {
            x: 5.0,
            y: 5.0,
            width: 10.0,
            height: 10.0,
        });
        // Should be coalesced into a single region.
        assert_eq!(m.len(), 1);
        let r = &m.regions()[0];
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.width, 15.0);
        assert_eq!(r.height, 15.0);
    }

    #[test]
    fn non_overlapping_regions_dont_coalesce() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        });
        m.mark_dirty(Rect {
            x: 100.0,
            y: 100.0,
            width: 10.0,
            height: 10.0,
        });
        // Should remain as 2 separate regions.
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn clear_resets_regions_and_increments_generation() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        });
        let g1 = m.generation();
        m.clear();
        assert!(!m.is_dirty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.generation(), g1 + 1);
    }

    #[test]
    fn many_overlapping_marks_coalesce_to_one() {
        let mut m = DirtyRegionManager::new();
        // Mark 100 overlapping small regions.
        for i in 0..100 {
            m.mark_dirty(Rect {
                x: i as f32,
                y: i as f32,
                width: 10.0,
                height: 10.0,
            });
        }
        // All should coalesce to a single region.
        assert_eq!(m.len(), 1);
    }
}

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

//! QuadTree spatial partitioning structure.
//!
//! # Why this exists
//! A QuadTree recursively subdivides a 2D region into four quadrants when a node
//! exceeds a capacity threshold. This enables O(log n) broad-phase collision checks
//! and dirty-rect merge queries instead of O(n²) brute-force comparisons.
//!
//! This was previously duplicated in `cvkg-scene`. Moving it here makes it available
//! to Physics, Flow, and Layout without import cycles.

use cvkg_core::Rect;

/// An axis-aligned 2D spatial partitioning tree.
///
/// # Contract
/// - Insertions that fall entirely outside `bounds` are silently dropped.
/// - `retrieve` returns ALL rects in leaf nodes that overlap the query rect;
///   callers must perform their own exact AABB test on the returned set.
/// - Maximum recursion depth is capped at `max_depth` (default 5) to bound
///   memory usage even with degenerate inputs (all rects at one point).
pub struct Quadtree {
    bounds: Rect,
    rects: Vec<Rect>,
    children: Option<Box<[Quadtree; 4]>>,
    max_rects: usize,
    max_depth: usize,
    depth: usize,
}

impl Quadtree {
    /// Create a new root QuadTree node covering the given bounds.
    ///
    /// Default capacity per leaf is 10 rects, maximum depth is 5 levels.
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            rects: Vec::new(),
            children: None,
            max_rects: 10,
            max_depth: 5,
            depth: 0,
        }
    }

    /// Internal constructor for child nodes with an inherited depth counter.
    fn new_with_depth(bounds: Rect, depth: usize) -> Self {
        Self {
            bounds,
            rects: Vec::new(),
            children: None,
            max_rects: 10,
            max_depth: 5,
            depth,
        }
    }

    /// Insert a rect into the tree.
    ///
    /// The rect is dropped if it does not intersect this node's bounds.
    /// If this is an interior node (already subdivided), the rect is
    /// propagated to all overlapping children. Otherwise it is stored
    /// in this leaf; if the leaf is now over capacity and below max depth,
    /// the node is subdivided and existing rects redistributed.
    pub fn insert(&mut self, rect: Rect) {
        if !self.intersects(self.bounds, rect) {
            return;
        }

        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                child.insert(rect);
            }
            return;
        }

        self.rects.push(rect);

        if self.rects.len() > self.max_rects && self.depth < self.max_depth {
            self.subdivide();
        }
    }

    /// Split this leaf into four equal quadrants and redistribute stored rects.
    ///
    /// After subdivision, this node becomes interior — its `rects` vec is drained
    /// into the children. The subdivision is skipped if `depth >= max_depth`.
    fn subdivide(&mut self) {
        let hw = self.bounds.width / 2.0;
        let hh = self.bounds.height / 2.0;
        let x = self.bounds.x;
        let y = self.bounds.y;
        let d = self.depth + 1;

        let mut children = Box::new([
            Quadtree::new_with_depth(
                Rect {
                    x,
                    y,
                    width: hw,
                    height: hh,
                },
                d,
            ),
            Quadtree::new_with_depth(
                Rect {
                    x: x + hw,
                    y,
                    width: hw,
                    height: hh,
                },
                d,
            ),
            Quadtree::new_with_depth(
                Rect {
                    x,
                    y: y + hh,
                    width: hw,
                    height: hh,
                },
                d,
            ),
            Quadtree::new_with_depth(
                Rect {
                    x: x + hw,
                    y: y + hh,
                    width: hw,
                    height: hh,
                },
                d,
            ),
        ]);

        for rect in self.rects.drain(..) {
            for child in children.iter_mut() {
                child.insert(rect);
            }
        }

        self.children = Some(children);
    }

    /// Returns true if rect `a` and rect `b` overlap (strict interior intersection).
    fn intersects(&self, a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
    }

    /// Collect all candidate rects that may overlap `rect` into `out`.
    ///
    /// This is a broad-phase query: returned rects are candidates from overlapping
    /// leaf nodes. Callers MUST perform their own exact intersection test on results.
    /// The `out` vec is appended to — it is never cleared.
    pub fn retrieve(&self, rect: Rect, out: &mut Vec<Rect>) {
        if !self.intersects(self.bounds, rect) {
            return;
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.retrieve(rect, out);
            }
        } else {
            for r in &self.rects {
                out.push(*r);
            }
        }
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
    fn test_insert_and_retrieve_basic() {
        let bounds = rect(0.0, 0.0, 100.0, 100.0);
        let mut qt = Quadtree::new(bounds);

        qt.insert(rect(10.0, 10.0, 20.0, 20.0));
        qt.insert(rect(60.0, 60.0, 20.0, 20.0));

        // Query that overlaps first rect
        let mut out = Vec::new();
        qt.retrieve(rect(5.0, 5.0, 30.0, 30.0), &mut out);
        assert!(!out.is_empty(), "Should find at least one candidate");
    }

    #[test]
    fn test_out_of_bounds_insert_is_dropped() {
        let bounds = rect(0.0, 0.0, 100.0, 100.0);
        let mut qt = Quadtree::new(bounds);

        // Entirely outside bounds
        qt.insert(rect(200.0, 200.0, 10.0, 10.0));

        let mut out = Vec::new();
        qt.retrieve(rect(0.0, 0.0, 100.0, 100.0), &mut out);
        assert!(out.is_empty(), "Out-of-bounds rect should be dropped");
    }

    #[test]
    fn test_subdivide_triggers_under_load() {
        let bounds = rect(0.0, 0.0, 1000.0, 1000.0);
        let mut qt = Quadtree::new(bounds);

        // Insert 15 rects spread across bounds to trigger subdivision (threshold = 10)
        for i in 0..15_u32 {
            let offset = (i * 60) as f32;
            qt.insert(rect(offset % 900.0, offset % 900.0, 30.0, 30.0));
        }

        // After subdivision, retrieval must still work
        let mut out = Vec::new();
        qt.retrieve(rect(0.0, 0.0, 1000.0, 1000.0), &mut out);
        assert!(!out.is_empty(), "Should retrieve rects after subdivision");
    }
}

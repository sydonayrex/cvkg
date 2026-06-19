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

//! Bounding Volume Hierarchy (BVH) over 2D AABBs.
//!
//! # Why this exists
//! Finding #5 from the crosscrate audit: O(n) brute-force overlap queries are
//! too slow for physics collision detection and scene-picking with 100k+ nodes
//! (knowledge graphs, data-lake visualisations). A BVH gives O(log n) average
//! query cost by recursively bisecting the item set along the longest axis.
//!
//! # Build / Query lifecycle
//! 1. Call `insert()` for every item with its bounding rect.
//! 2. Call `build()` once — this constructs the binary tree in place.
//! 3. Call `query()` to retrieve items whose AABB overlaps a query rect.
//! 4. Call `clear()` to reset; repeat from step 1 for a new frame.
//!
//! `build()` uses a simple top-down median split along the longest AABB axis.
//! This is O(n log n) and produces a balanced tree for uniform distributions,
//! which covers the common cases in CVKG's graph views. A SAH (Surface Area
//! Heuristic) can be substituted later without changing the public API.

use cvkg_core::Rect;

/// A single node in the BVH tree — either an interior node (two children)
/// or a leaf node (references one item from the item list).
pub struct BvhNode {
    /// The axis-aligned bounding box that encompasses all items in this subtree.
    pub aabb: Rect,
    /// Left child node (present for interior nodes).
    pub left: Option<Box<BvhNode>>,
    /// Right child node (present for interior nodes).
    pub right: Option<Box<BvhNode>>,
    /// Index into `Bvh::items` for leaf nodes; `None` for interior nodes.
    pub item_index: Option<usize>,
}

impl BvhNode {
    /// Create a leaf node referencing the given item index.
    fn leaf(aabb: Rect, item_index: usize) -> Self {
        Self {
            aabb,
            left: None,
            right: None,
            item_index: Some(item_index),
        }
    }

    /// Create an interior node that bounds two children.
    fn interior(aabb: Rect, left: BvhNode, right: BvhNode) -> Self {
        Self {
            aabb,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            item_index: None,
        }
    }
}

/// Axis-aligned Bounding Volume Hierarchy over generic items.
///
/// # Type parameter
/// `T` is the payload stored alongside each AABB. It must be `Clone` because
/// `build()` rebuilds the tree from the `items` vec without consuming it.
pub struct Bvh<T> {
    /// Flat list of (bounding rect, payload) pairs as inserted.
    items: Vec<(Rect, T)>,
    /// Root of the BVH tree; `None` until `build()` is called.
    root: Option<BvhNode>,
}

impl<T: Clone> Bvh<T> {
    /// Create an empty BVH.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            root: None,
        }
    }

    /// Add an item with its bounding rect.
    ///
    /// This does NOT rebuild the tree. After adding all items for a frame,
    /// call `build()` once before issuing any `query()` calls. Calling
    /// `query()` before `build()` always returns an empty result.
    pub fn insert(&mut self, rect: Rect, item: T) {
        self.items.push((rect, item));
        // Invalidate the tree since it is now stale.
        self.root = None;
    }

    /// Rebuild the BVH tree from all currently inserted items.
    ///
    /// Uses a top-down recursive median split along the longest axis of the
    /// current item set's combined AABB. Complexity: O(n log n).
    ///
    /// This must be called after all `insert()` calls for a frame and before
    /// any `query()` calls. Subsequent `insert()` calls invalidate the tree
    /// and require another `build()`.
    pub fn build(&mut self) {
        if self.items.is_empty() {
            self.root = None;
            return;
        }

        let indices: Vec<usize> = (0..self.items.len()).collect();
        self.root = Some(Self::build_recursive(&self.items, &indices));
    }

    /// Recursively build a subtree over the given index slice.
    ///
    /// Base case: one item → leaf node.
    /// Recursive case: compute combined AABB, split by centroid median along
    /// the longest axis, build left and right subtrees.
    fn build_recursive(items: &[(Rect, T)], indices: &[usize]) -> BvhNode {
        assert!(!indices.is_empty());

        if indices.len() == 1 {
            let idx = indices[0];
            return BvhNode::leaf(items[idx].0, idx);
        }

        // Compute the combined AABB for all items in this slice.
        let combined = combined_aabb(items, indices);

        if indices.len() == 2 {
            // Two items: simple binary split without further recursion.
            let left = BvhNode::leaf(items[indices[0]].0, indices[0]);
            let right = BvhNode::leaf(items[indices[1]].0, indices[1]);
            return BvhNode::interior(combined, left, right);
        }

        // Determine split axis: longest dimension of the combined AABB.
        let split_on_x = combined.width >= combined.height;

        // Sort by centroid along split axis to find the median.
        let mut sorted = indices.to_vec();
        sorted.sort_by(|&a, &b| {
            let ca = centroid(&items[a].0, split_on_x);
            let cb = centroid(&items[b].0, split_on_x);
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = sorted.len() / 2;
        let left_indices = &sorted[..mid];
        let right_indices = &sorted[mid..];

        let left = Self::build_recursive(items, left_indices);
        let right = Self::build_recursive(items, right_indices);

        BvhNode::interior(combined, left, right)
    }

    /// Return references to all items whose AABB overlaps `rect`.
    ///
    /// Traversal is O(log n) on average for uniform distributions. The query
    /// prunes entire subtrees when the node's AABB does not overlap `rect`.
    ///
    /// Returns an empty slice if `build()` has not been called since the last
    /// `insert()` or `clear()`.
    pub fn query(&self, rect: Rect) -> Vec<&T> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            Self::query_recursive(root, rect, &self.items, &mut results);
        }
        results
    }

    /// Recursive tree traversal for `query()`.
    fn query_recursive<'a>(
        node: &BvhNode,
        rect: Rect,
        items: &'a [(Rect, T)],
        results: &mut Vec<&'a T>,
    ) {
        if !aabbs_overlap(node.aabb, rect) {
            return;
        }

        if let Some(idx) = node.item_index {
            // Leaf: the node's AABB IS the item's AABB (already confirmed to overlap).
            results.push(&items[idx].1);
            return;
        }

        if let Some(ref left) = node.left {
            Self::query_recursive(left, rect, items, results);
        }
        if let Some(ref right) = node.right {
            Self::query_recursive(right, rect, items, results);
        }
    }

    /// Return the number of items inserted.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if no items have been inserted since the last `clear()`.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Remove all items and discard the tree.
    pub fn clear(&mut self) {
        self.items.clear();
        self.root = None;
    }
}

impl<T: Clone> Default for Bvh<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compute the combined AABB of a subset of items specified by indices.
fn combined_aabb<T>(items: &[(Rect, T)], indices: &[usize]) -> Rect {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for &i in indices {
        let r = &items[i].0;
        min_x = min_x.min(r.x);
        min_y = min_y.min(r.y);
        max_x = max_x.max(r.x + r.width);
        max_y = max_y.max(r.y + r.height);
    }

    Rect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    }
}

/// Compute the centroid of a rect along x (if `use_x`) or y axis.
fn centroid(r: &Rect, use_x: bool) -> f32 {
    if use_x {
        r.x + r.width * 0.5
    } else {
        r.y + r.height * 0.5
    }
}

/// Returns `true` if two AABBs overlap (strict interior; touching edges = false).
fn aabbs_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
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
    fn test_empty_bvh_query_returns_nothing() {
        let bvh: Bvh<u32> = Bvh::new();
        let results = bvh.query(rect(0.0, 0.0, 100.0, 100.0));
        assert!(results.is_empty());
    }

    #[test]
    fn test_single_item_query() {
        let mut bvh: Bvh<&str> = Bvh::new();
        bvh.insert(rect(10.0, 10.0, 20.0, 20.0), "alpha");
        bvh.build();

        let hits = bvh.query(rect(0.0, 0.0, 50.0, 50.0));
        assert_eq!(hits, vec![&"alpha"]);

        let misses = bvh.query(rect(200.0, 200.0, 10.0, 10.0));
        assert!(misses.is_empty());
    }

    #[test]
    fn test_multiple_items_selective_query() {
        let mut bvh: Bvh<u32> = Bvh::new();
        // Left cluster
        bvh.insert(rect(0.0, 0.0, 10.0, 10.0), 1);
        bvh.insert(rect(5.0, 5.0, 10.0, 10.0), 2);
        // Right cluster
        bvh.insert(rect(500.0, 500.0, 10.0, 10.0), 3);
        bvh.insert(rect(510.0, 510.0, 10.0, 10.0), 4);
        bvh.build();

        let left = bvh.query(rect(0.0, 0.0, 30.0, 30.0));
        assert!(left.contains(&&1), "Should find 1");
        assert!(left.contains(&&2), "Should find 2");
        assert!(!left.contains(&&3), "Should not find 3");
        assert!(!left.contains(&&4), "Should not find 4");

        let right = bvh.query(rect(495.0, 495.0, 40.0, 40.0));
        assert!(!right.contains(&&1), "Should not find 1");
        assert!(right.contains(&&3), "Should find 3");
        assert!(right.contains(&&4), "Should find 4");
    }

    #[test]
    fn test_clear_invalidates_tree() {
        let mut bvh: Bvh<u32> = Bvh::new();
        bvh.insert(rect(0.0, 0.0, 50.0, 50.0), 7);
        bvh.build();
        bvh.clear();

        assert!(bvh.is_empty());
        assert_eq!(bvh.len(), 0);
        let results = bvh.query(rect(0.0, 0.0, 50.0, 50.0));
        assert!(results.is_empty());
    }

    #[test]
    fn test_build_then_insert_requires_rebuild() {
        let mut bvh: Bvh<u32> = Bvh::new();
        bvh.insert(rect(0.0, 0.0, 10.0, 10.0), 1);
        bvh.build();

        // Insert a second item WITHOUT rebuilding
        bvh.insert(rect(100.0, 100.0, 10.0, 10.0), 2);

        // Query should NOT find item 2 because tree is stale
        let results = bvh.query(rect(95.0, 95.0, 20.0, 20.0));
        assert!(
            !results.contains(&&2),
            "Stale tree must not return post-build inserts"
        );

        // After rebuild, item 2 should be found
        bvh.build();
        let results = bvh.query(rect(95.0, 95.0, 20.0, 20.0));
        assert!(results.contains(&&2), "Rebuilt tree must return item 2");
    }
}

use crate::*;

/// Key used to identify a cached layout entry.
/// Combines a view hash with a generation counter for cache invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutKey {
    pub view_hash: u64,
    pub generation: u64,
}

// Layout pass scratch space
pub struct LayoutCache {
    pub safe_area: SafeArea,
    pub delta_time: f32,
    /// Device scale factor for HiDPI / retina snapping. Defaults to 1.0.
    pub scale_factor: f32,
    /// The visible viewport bounds in logical pixels.
    /// If Some, layout execution can cull offscreen subtrees.
    pub viewport: Option<Rect>,
    /// Time budget for the layout pass. Defaults to 4.0ms.
    pub layout_time_budget: std::time::Duration,
    /// Start of the layout pass, captured at the beginning of the frame/layout run.
    pub layout_start_time: Option<std::time::Instant>,
    size_cache: HashMap<(u64, u32, u32), Size>, // (ViewHash, ProposalW, ProposalH)
    /// Map tracking child-to-parent view hash relationships for bottom-up invalidation.
    pub parent_map: HashMap<u64, u64>,
    /// Monotonically increasing generation counter for cache invalidation.
    /// When a view tree changes, bumping the generation causes stale entries
    /// to be treated as invalid without eagerly clearing the entire cache.
    generation: u64,
    /// Opaque pointer to the active layout engine (e.g. Taffy)
    /// Opaque pointer to the active layout engine (e.g. Taffy)
    pub engine: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// Opaque pointer to the active animation orchestrator
    pub animators: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// Cached previous rects for view transitions
    pub previous_rects: HashMap<u64, Rect>,
    /// Generation counter for cache eviction.
    /// Incremented each frame; entries not touched for N frames are evicted.
    pub eviction_generation: u64,
    /// Tracks which generation each previous_rects entry was last touched in.
    pub previous_rects_generation: HashMap<u64, u64>,
    /// Number of generations an entry can go untouched before eviction.
    eviction_threshold: u64,
}

thread_local! {
    static LAYOUT_BUDGET_DEADLINE: std::cell::RefCell<Option<std::time::Instant>> =
        const { std::cell::RefCell::new(None) };
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            safe_area: SafeArea::default(),
            delta_time: 0.016,
            scale_factor: 1.0,
            viewport: None,
            layout_time_budget: std::time::Duration::from_millis(4),
            layout_start_time: None,
            size_cache: HashMap::new(),
            parent_map: HashMap::new(),
            generation: 0,
            engine: None,
            animators: None,
            previous_rects: HashMap::new(),
            eviction_generation: 0,
            previous_rects_generation: HashMap::new(),
            eviction_threshold: 300, // ~5 seconds at 60fps
        }
    }

    /// Returns the current generation counter.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Evict entries from previous_rects that haven't been touched for N generations.
    pub fn evict_stale_entries(&mut self) {
        self.eviction_generation += 1;
        let threshold = self.eviction_threshold;
        let current_gen = self.eviction_generation;
        self.previous_rects.retain(|hash, _| {
            self.previous_rects_generation
                .get(hash)
                .is_some_and(|g| current_gen - *g < threshold)
        });
        self.previous_rects_generation
            .retain(|hash, _| self.previous_rects.contains_key(hash));
    }

    /// Checks if the layout pass is currently running over its allocated time budget.
    pub fn is_over_budget(&self) -> bool {
        let deadline_red = LAYOUT_BUDGET_DEADLINE.with(|deadline| {
            deadline
                .borrow()
                .as_ref()
                .is_some_and(|deadline| std::time::Instant::now() >= *deadline)
        });
        if deadline_red {
            return true;
        }
        if let Some(start) = self.layout_start_time {
            start.elapsed() > self.layout_time_budget
        } else {
            false
        }
    }

    /// Set a process-local deadline for layout cache consumers.
    /// When this deadline is exceeded, caches should reuse previous
    /// rects instead of recomputing expensive layout work.
    pub fn set_layout_budget_deadline(deadline: Option<std::time::Instant>) {
        LAYOUT_BUDGET_DEADLINE.with(|slot| {
            *slot.borrow_mut() = deadline;
        });
    }

    /// Clear any process-local layout budget deadline.
    pub fn clear_layout_budget_deadline() {
        Self::set_layout_budget_deadline(None);
    }

    /// Bump the generation counter, logically invalidating all cached entries
    /// without eagerly clearing them. Subsequent lookups with the old generation
    /// will miss until re-populated.
    pub fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Check whether a cached entry for the given key is still valid
    /// against the current generation.
    pub fn is_valid(&self, key: LayoutKey, current_gen: u64) -> bool {
        key.generation == current_gen && key.generation == self.generation
    }

    pub fn clear(&mut self) {
        self.safe_area = SafeArea::default();
        self.viewport = None;
        self.layout_start_time = None;
        self.size_cache.clear();
        self.parent_map.clear();
    }

    pub fn get_size(&self, view_hash: u64, proposal: SizeProposal) -> Option<Size> {
        debug_assert!(
            proposal.width.map_or(true, |v| v.is_finite()),
            "layout proposal width is not finite: {:?}",
            proposal.width
        );
        debug_assert!(
            proposal.height.map_or(true, |v| v.is_finite()),
            "layout proposal height is not finite: {:?}",
            proposal.height
        );
        let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
        let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
        self.size_cache.get(&(view_hash, pw, ph)).copied()
    }

    pub fn set_size(&mut self, view_hash: u64, proposal: SizeProposal, size: Size) {
        debug_assert!(
            proposal.width.map_or(true, |v| v.is_finite()),
            "layout proposal width is not finite: {:?}",
            proposal.width
        );
        debug_assert!(
            proposal.height.map_or(true, |v| v.is_finite()),
            "layout proposal height is not finite: {:?}",
            proposal.height
        );
        debug_assert!(
            size.width.is_finite() && size.height.is_finite(),
            "layout size is not finite: {:?}",
            size
        );
        let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
        let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
        self.size_cache.insert((view_hash, pw, ph), size);
    }

    /// Register a child-to-parent layout relationship for bottom-up invalidation propagation.
    pub fn register_parent(&mut self, child_hash: u64, parent_hash: u64) {
        if child_hash != 0 && parent_hash != 0 {
            self.parent_map.insert(child_hash, parent_hash);
        }
    }

    /// Remove all cached size entries for a specific view hash and propagate the invalidation
    /// bottom-up to all its layout ancestors to ensure consistent layout updates.
    pub fn invalidate_view(&mut self, view_hash: u64) {
        let mut to_invalidate = vec![view_hash];
        let mut visited = std::collections::HashSet::new();
        while let Some(hash) = to_invalidate.pop() {
            if !visited.insert(hash) {
                continue;
            }
            self.size_cache.retain(|&(h, _, _), _| h != hash);
            if let Some(&parent) = self.parent_map.get(&hash) {
                to_invalidate.push(parent);
            }
        }
    }
}

/// Proposed size from parent view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeProposal {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl SizeProposal {
    pub fn unspecified() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    pub fn width(width: f32) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }

    pub fn height(height: f32) -> Self {
        Self {
            width: None,
            height: Some(height),
        }
    }

    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
        }
    }

    pub fn new(width: Option<f32>, height: Option<f32>) -> Self {
        Self { width, height }
    }
}

/// A view that can participate in layout
pub trait LayoutView: Send {
    /// Propose a size for this view given the available space
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size;

    /// Place subviews within the given bounds
    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    );

    /// Returns the flex weight of this view (default is 0.0, which means fixed/intrinsic)
    fn flex_weight(&self) -> f32 {
        0.0
    }

    /// Returns a persistent unique identifier for this view to enable Layout View Transitions.
    /// Return 0 (default) to disable layout animations for this node.
    fn view_hash(&self) -> u64 {
        0
    }

    /// Return true when this view's layout may have changed since the last pass.
    ///
    /// The layout engine uses this to skip cache lookups for views that are
    /// guaranteed static (e.g., chrome elements that never change between frames).
    /// Default false to avoid redundant passes for static subtrees.
    /// Override true for views whose layout may change between frames.
    ///
    /// When false, the engine may skip `size_that_fits` entirely and reuse the
    /// cached rect from `LayoutCache::previous_rects`.
    fn changed(&self) -> bool {
        false
    }

    /// Return true when this view needs per-frame updates (animations, timers, etc.).
    /// Default false. Views that drive continuous updates should override this.
    fn needs_update(&self) -> bool {
        false
    }

    /// Return a debug representation of this layout subtree.
    /// The `indent` parameter controls the indentation level for nested display.
    fn debug_layout(&self, indent: usize) -> String {
        let prefix = " ".repeat(indent);
        format!("{}LayoutView", prefix)
    }
}
/// Edge insets for padding, margins, and safe areas
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct EdgeInsets {
    pub top: f32,
    pub leading: f32,
    pub bottom: f32,
    pub trailing: f32,
}

impl EdgeInsets {
    pub fn new(top: f32, leading: f32, bottom: f32, trailing: f32) -> Self {
        Self {
            top,
            leading,
            bottom,
            trailing,
        }
    }

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            leading: value,
            bottom: value,
            trailing: value,
        }
    }

    /// Vertical insets (top and bottom, leading and trailing are zero).
    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            leading: 0.0,
            bottom: value,
            trailing: 0.0,
        }
    }

    /// Horizontal insets (leading and trailing, top and bottom are zero).
    pub fn horizontal(value: f32) -> Self {
        Self {
            top: 0.0,
            leading: value,
            bottom: 0.0,
            trailing: value,
        }
    }
}

/// SafeArea constraints provided by the platform
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SafeArea {
    pub insets: EdgeInsets,
}

/// SDF Shape definitions for Vili Interaction Paradigm hit-testing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SdfShape {
    Rect(Rect),
    RoundedRect { rect: Rect, radius: f32 },
    Circle { center: [f32; 2], radius: f32 },
}

/// Rectangle in logical pixels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn inset(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - amount * 2.0).max(0.0),
            height: (self.height - amount * 2.0).max(0.0),
        }
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..*self
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Determines whether this rectangle overlaps with another rectangle.
    ///
    /// # Contract
    /// Two rectangles overlap if their projection intervals on both the X
    /// and Y axes overlap. This is used for viewport intersection checks
    /// to determine visibility constraints during layout culling.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// Split the rect horizontally into N equal pieces
    pub fn split_horizontal(&self, n: usize) -> Vec<Rect> {
        if n == 0 {
            return vec![];
        }
        let item_width = self.width / n as f32;
        (0..n)
            .map(|i| Rect {
                x: self.x + i as f32 * item_width,
                y: self.y,
                width: item_width,
                height: self.height,
            })
            .collect()
    }

    /// Split the rect vertically into N equal pieces
    pub fn split_vertical(&self, n: usize) -> Vec<Rect> {
        if n == 0 {
            return vec![];
        }
        let item_height = self.height / n as f32;
        (0..n)
            .map(|i| Rect {
                x: self.x,
                y: self.y + i as f32 * item_height,
                width: self.width,
                height: item_height,
            })
            .collect()
    }
}

#[cfg(test)]
mod changed_default_tests {
    use super::*;
    use crate::{LayoutCache, LayoutView, SizeProposal};

    struct StaticLabel {
        width: f32,
        height: f32,
    }

    impl LayoutView for StaticLabel {
        fn size_that_fits(
            &self,
            _proposal: SizeProposal,
            _subviews: &[&dyn LayoutView],
            _cache: &mut LayoutCache,
        ) -> crate::Size {
            crate::Size {
                width: self.width,
                height: self.height,
            }
        }

        fn place_subviews(
            &self,
            _rect: Rect,
            _subviews: &mut [&mut dyn LayoutView],
            _cache: &mut LayoutCache,
        ) {
        }
    }

    #[test]
    fn static_view_changed_returns_false() {
        let label = StaticLabel {
            width: 100.0,
            height: 20.0,
        };
        assert!(
            !label.changed(),
            "Static view should return false from changed()"
        );
    }

    #[test]
    fn static_view_needs_update_returns_false() {
        let label = StaticLabel {
            width: 100.0,
            height: 20.0,
        };
        assert!(
            !label.needs_update(),
            "Static view should return false from needs_update()"
        );
    }

    #[test]
    fn static_view_changed_consistent_across_renders() {
        let label = StaticLabel {
            width: 50.0,
            height: 10.0,
        };
        let first = label.changed();
        let second = label.changed();
        assert!(!first, "First render changed() should be false");
        assert!(!second, "Second render changed() should be false");
    }
}

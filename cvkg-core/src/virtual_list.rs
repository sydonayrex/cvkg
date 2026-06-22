//! Virtual list computation extracted from lib.rs.
//!
//! P1-41: Tests for virtual scrolling window calculations.
//! P1-42: DependencyGraph tests
//! P1-43: FrameBudget tests
//! P2-36: InputLatencyTracker tests

use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// P1-41: Virtual List Window
// =============================================================================

/// Outcome of a `compute_virtual_list_window` call — describes which rows
/// should be rendered and what scroll offset to apply to position them.
///
/// P1-41: IDE and visualization workloads with tens-of-thousands of rows must
/// only render the rows visible in the current viewport.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualWindow {
    /// Index of the first row that should be rendered (inclusive).
    pub first_visible: usize,
    /// Index one past the last row that should be rendered (exclusive).
    pub last_visible: usize,
    /// Total estimated height of all rows above `first_visible`, in logical
    /// pixels.  Use this as the scroll-offset padding above the rendered rows.
    pub offset_before: f32,
    /// Total estimated height of all rows below `last_visible`, in logical
    /// pixels.  Use this as the placeholder height below the rendered rows.
    pub offset_after: f32,
}

/// Computes the visible slice of a uniform-height virtual list.
///
/// Contract:
/// - `total_rows`: total number of items in the list (can be enormous).
/// - `row_height`: uniform logical height of every row in pixels.
/// - `viewport_y`: scroll offset of the viewport top edge.
/// - `viewport_height`: height of the visible window in pixels.
/// - `overscan`: number of extra rows to render above/below the viewport for smooth scrolling.
///
/// Returns a `VirtualWindow` describing the rendered slice and offset padding.
pub fn compute_virtual_list_window(
    total_rows: usize,
    row_height: f32,
    viewport_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> VirtualWindow {
    if total_rows == 0 || row_height <= 0.0 {
        return VirtualWindow {
            first_visible: 0,
            last_visible: 0,
            offset_before: 0.0,
            offset_after: 0.0,
        };
    }

    // How many rows fit in the viewport (rounded up for partial rows).
    let visible_rows = (viewport_height / row_height).ceil() as usize;

    // First row whose bottom edge is below the viewport top.
    let first = (viewport_y / row_height).floor() as isize - overscan as isize;
    let first = first.max(0) as usize;

    // Last row whose top edge is above the viewport bottom.
    let last = first + visible_rows + 2 * overscan;
    let last = last.min(total_rows);

    VirtualWindow {
        first_visible: first,
        last_visible: last,
        offset_before: first as f32 * row_height,
        offset_after: (total_rows - last) as f32 * row_height,
    }
}

/// Computes the visible slice of a variable-height virtual list using
/// a precomputed prefix-sum of row heights.
///
/// Contract:
/// - `prefix_heights[i]` is the cumulative height of all rows 0..i (not
///   including row i).  `prefix_heights.len()` must equal `total_rows + 1`
///   where `prefix_heights[0] == 0` and `prefix_heights[total_rows]` is the
///   total list height.
/// - `viewport_y` and `viewport_height` are in the same logical pixel units.
/// - `overscan` works the same as in `compute_virtual_list_window`.
///
/// This is O(log N) via binary search on the prefix-sum array.
pub fn compute_virtual_list_window_variable(
    prefix_heights: &[f32],
    viewport_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> VirtualWindow {
    let total_rows = prefix_heights.len().saturating_sub(1);
    if total_rows == 0 {
        return VirtualWindow {
            first_visible: 0,
            last_visible: 0,
            offset_before: 0.0,
            offset_after: 0.0,
        };
    }

    // Binary search for the first row whose cumulative top is >= viewport_y.
    let first_idx = prefix_heights
        .partition_point(|&h| h < viewport_y)
        .saturating_sub(1);
    let first = first_idx.saturating_sub(overscan);

    // Binary search for the last row whose top < viewport_y + viewport_height.
    let viewport_bottom = viewport_y + viewport_height;
    let last_idx = prefix_heights.partition_point(|&h| h < viewport_bottom);
    let last = (last_idx + overscan).min(total_rows);

    VirtualWindow {
        first_visible: first,
        last_visible: last,
        offset_before: prefix_heights[first],
        offset_after: prefix_heights[total_rows] - prefix_heights[last],
    }
}

// =============================================================================
// P1-42: DependencyGraph
// =============================================================================

/// Tracks fine-grained dependencies between components (subscribers) and the
/// state keys they depend on.
///
/// P1-42: A single `State<T>` mutation currently fans out to every subscriber,
/// even those that do not depend on the changed value.  `DependencyGraph`
/// enables callers to register which state keys a component depends on, then
/// query only the components that are actually affected by a given change.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    deps: HashMap<u64, std::collections::HashSet<u64>>,
    reverse: HashMap<u64, Vec<u64>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    pub fn register(&mut self, component_id: u64, state_key: u64) {
        let is_new = self.deps
            .entry(state_key)
            .or_default()
            .insert(component_id);
        if is_new {
            self.reverse
                .entry(component_id)
                .or_default()
                .push(state_key);
        }
    }

    pub fn unregister(&mut self, component_id: u64) {
        if let Some(keys) = self.reverse.remove(&component_id) {
            for key in keys {
                if let Some(set) = self.deps.get_mut(&key) {
                    set.remove(&component_id);
                }
            }
        }
    }

    pub fn affected_components(&self, state_key: u64) -> impl Iterator<Item = u64> + '_ {
        self.deps
            .get(&state_key)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    pub fn has_dependents(&self, state_key: u64) -> bool {
        self.deps
            .get(&state_key)
            .map_or(false, |set| !set.is_empty())
    }

    pub fn edge_count(&self) -> usize {
        self.deps.values().map(|s| s.len()).sum()
    }
}

// =============================================================================
// P1-43: Subsystem Budget
// =============================================================================

/// P1-43: per-subsystem budget allocation. A frame's total
/// time is split across animation, layout, and render subsystems.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubsystemBudget {
    pub time_slice: Duration,
    pub skippable: bool,
    pub name: &'static str,
}

/// P1-43: global frame budget tracker.
#[derive(Debug)]
pub struct FrameBudgetTracker {
    total: Duration,
    allocations: Vec<SubsystemBudget>,
    start: Option<Instant>,
    elapsed: Vec<Duration>,
}

impl FrameBudgetTracker {
    pub fn default_60fps() -> Self {
        Self {
            total: Duration::from_micros(16_666),
            allocations: vec![
                SubsystemBudget {
                    time_slice: Duration::from_micros(4_000),
                    skippable: true,
                    name: "animation",
                },
                SubsystemBudget {
                    time_slice: Duration::from_micros(4_000),
                    skippable: true,
                    name: "layout",
                },
                SubsystemBudget {
                    time_slice: Duration::from_micros(8_000),
                    skippable: false,
                    name: "render",
                },
            ],
            start: None,
            elapsed: vec![Duration::ZERO, Duration::ZERO, Duration::ZERO],
        }
    }

    pub fn default_120fps() -> Self {
        Self {
            total: Duration::from_micros(8_333),
            allocations: vec![
                SubsystemBudget {
                    time_slice: Duration::from_micros(2_000),
                    skippable: true,
                    name: "animation",
                },
                SubsystemBudget {
                    time_slice: Duration::from_micros(2_000),
                    skippable: true,
                    name: "layout",
                },
                SubsystemBudget {
                    time_slice: Duration::from_micros(4_000),
                    skippable: false,
                    name: "render",
                },
            ],
            start: None,
            elapsed: vec![Duration::ZERO, Duration::ZERO, Duration::ZERO],
        }
    }

    pub fn total(&self) -> Duration { self.total }
    pub fn allocations(&self) -> &[SubsystemBudget] { &self.allocations }

    pub fn new_frame(&mut self) {
        self.start = Some(Instant::now());
        for e in self.elapsed.iter_mut() { *e = Duration::ZERO; }
    }

    pub fn subsystem_finish(&mut self, index: usize) {
        if let Some(start) = self.start {
            if index < self.elapsed.len() {
                self.elapsed[index] = start.elapsed();
            }
        }
    }

    pub fn is_within_budget(&self, index: usize) -> bool {
        if index >= self.allocations.len() || index >= self.elapsed.len() { return false; }
        self.elapsed[index] <= self.allocations[index].time_slice
    }

    pub fn frame_within_budget(&self) -> bool {
        for (i, alloc) in self.allocations.iter().enumerate() {
            if i < self.elapsed.len() && self.elapsed[i] > alloc.time_slice && !alloc.skippable {
                return false;
            }
        }
        true
    }

    pub fn elapsed(&self, index: usize) -> Duration {
        if index < self.elapsed.len() { self.elapsed[index] } else { Duration::ZERO }
    }

    pub fn total_elapsed(&self) -> Duration {
        match self.start {
            Some(start) => start.elapsed(),
            None => Duration::ZERO,
        }
    }
}

// =============================================================================
// P2-36: Input Latency Tracker
// =============================================================================

/// P2-36: input latency telemetry. Tracks end-to-end latency
/// between input event receipt and frame rendering completion.
#[derive(Debug, Clone)]
pub struct InputLatencyTracker {
    window_size: usize,
    samples: std::collections::VecDeque<(Instant, Instant)>,
}

impl InputLatencyTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            samples: std::collections::VecDeque::with_capacity(window_size),
        }
    }

    pub fn record_frame(&mut self, event_time: Instant, render_time: Instant) {
        if self.window_size == 0 { return; }
        if self.samples.len() >= self.window_size { self.samples.pop_front(); }
        self.samples.push_back((event_time, render_time));
    }

    pub fn percentile(&self, p: f64) -> Duration {
        if self.samples.is_empty() || p < 0.0 || p > 100.0 { return Duration::ZERO; }
        let mut latencies: Vec<Duration> = self.samples
            .iter()
            .map(|&(e, r)| if r > e { r.duration_since(e) } else { Duration::ZERO })
            .collect();
        latencies.sort();
        let len = latencies.len();
        let rank = p / 100.0;
        let index = ((len as f64 * rank).ceil() as usize).saturating_sub(1);
        let index = index.min(len - 1);
        latencies[index]
    }

    pub fn clear(&mut self) { self.samples.clear(); }
    pub fn window_size(&self) -> usize { self.window_size }

    pub fn set_window_size(&mut self, size: usize) {
        self.window_size = size;
        while self.samples.len() > self.window_size { self.samples.pop_front(); }
    }

    pub fn len(&self) -> usize { self.samples.len() }
    pub fn is_empty(&self) -> bool { self.samples.is_empty() }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // P1-41: VirtualListWindow tests
    mod p1_41_virtual_list_tests {
        use super::{compute_virtual_list_window, compute_virtual_list_window_variable};

        #[test]
        fn uniform_list_at_top_shows_first_rows() {
            let w = compute_virtual_list_window(1000, 20.0, 0.0, 200.0, 2);
            assert_eq!(w.first_visible, 0);
            assert!(w.last_visible <= 14);
            assert_eq!(w.offset_before, 0.0);
            assert!(w.offset_after > 0.0);
        }

        #[test]
        fn uniform_list_mid_scroll_correct_window() {
            let w = compute_virtual_list_window(1000, 20.0, 1000.0, 200.0, 2);
            assert!(w.first_visible <= 48);
            assert!(w.last_visible > 60);
        }

        #[test]
        fn uniform_list_near_end_does_not_exceed_total() {
            let w = compute_virtual_list_window(10, 20.0, 150.0, 100.0, 5);
            assert!(w.last_visible <= 10);
        }

        #[test]
        fn variable_list_basic() {
            let prefix: Vec<f32> = vec![0.0, 10.0, 30.0, 60.0, 100.0, 150.0];
            let w = compute_virtual_list_window_variable(&prefix, 0.0, 40.0, 0);
            assert!(w.first_visible == 0);
            assert!(w.last_visible >= 3 && w.last_visible <= 5);
        }

        #[test]
        fn empty_list_returns_zero_window() {
            let w = compute_virtual_list_window(0, 20.0, 0.0, 200.0, 2);
            assert_eq!(w.first_visible, 0);
            assert_eq!(w.last_visible, 0);
            assert_eq!(w.offset_before, 0.0);
            assert_eq!(w.offset_after, 0.0);
        }
    }

    // P1-42: DependencyGraph tests
    mod p1_42_dependency_graph_tests {
        use super::DependencyGraph;

        #[test]
        fn register_and_query_single_dep() {
            let mut g = DependencyGraph::new();
            g.register(42, 100);
            let affected: Vec<u64> = g.affected_components(100).collect();
            assert_eq!(affected, vec![42]);
        }

        #[test]
        fn unregister_removes_component() {
            let mut g = DependencyGraph::new();
            g.register(1, 10);
            g.register(2, 10);
            g.unregister(1);
            let affected: Vec<u64> = g.affected_components(10).collect();
            assert!(!affected.contains(&1));
            assert!(affected.contains(&2));
        }

        #[test]
        fn no_deps_returns_empty() {
            let g = DependencyGraph::new();
            let affected: Vec<u64> = g.affected_components(999).collect();
            assert!(affected.is_empty());
        }

        #[test]
        fn edge_count_reflects_registrations() {
            let mut g = DependencyGraph::new();
            assert_eq!(g.edge_count(), 0);
            g.register(1, 10);
            g.register(2, 10);
            g.register(1, 20);
            assert_eq!(g.edge_count(), 3);
        }
    }

    // P1-43: FrameBudget tests
    mod p1_43_frame_budget_tests {
        use super::FrameBudgetTracker;

        #[test]
        fn default_60fps_has_16ms_total() {
            let fb = FrameBudgetTracker::default_60fps();
            assert!(fb.total().as_micros() >= 16_000);
            assert!(fb.total().as_micros() <= 17_000);
        }

        #[test]
        fn render_is_essential_layout_is_skippable() {
            let fb = FrameBudgetTracker::default_60fps();
            let render = fb.allocations().iter().find(|a| a.name == "render").unwrap();
            let layout = fb.allocations().iter().find(|a| a.name == "layout").unwrap();
            assert!(!render.skippable);
            assert!(layout.skippable);
        }
    }

    // P2-36: InputLatencyTracker tests
    mod p2_36_input_latency_tests {
        use super::InputLatencyTracker;

        #[test]
        fn test_empty_tracker() {
            let tracker = InputLatencyTracker::new(10);
            assert!(tracker.is_empty());
        }

        #[test]
        fn test_record_and_sliding_window() {
            let mut tracker = InputLatencyTracker::new(3);
            let now = Instant::now();
            tracker.record_frame(now, now + Duration::from_millis(10));
            tracker.record_frame(now, now + Duration::from_millis(20));
            tracker.record_frame(now, now + Duration::from_millis(30));
            tracker.record_frame(now, now + Duration::from_millis(40));
            assert_eq!(tracker.percentile(50.0), Duration::from_millis(30));
        }
    }
}
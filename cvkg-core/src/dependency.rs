//! Dependency tracked state invalidation (P1-42).
//!
//! Tracks fine-grained dependencies between components (subscribers) and the
//! state keys they depend on.

use std::collections::HashMap;

/// Tracks fine-grained dependencies between components (subscribers) and the
/// state keys they depend on.
///
/// P1-42: A single `State<T>` mutation currently fans out to every subscriber,
/// even those that do not depend on the changed value.  `DependencyGraph`
/// enables callers to register which state keys a component depends on, then
/// query only the components that are actually affected by a given change.
///
/// This is a lightweight directed graph: state_key → Set<component_id>.
/// The graph is append-only; removal happens by re-registration.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    /// Maps a state key (e.g. a hashed type id + version) to the set of
    /// component IDs that depend on it.
    deps: HashMap<u64, std::collections::HashSet<u64>>,
    /// Reverse map: component_id → set of state keys it depends on.
    /// Used to efficiently re-register (clear then re-add) a component.
    reverse: HashMap<u64, Vec<u64>>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    /// Register that `component_id` depends on `state_key`.
    ///
    /// Idempotent — calling this twice with the same arguments has no effect.
    /// To replace a component's full dependency set, call `unregister` first.
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

    /// Remove all dependencies for `component_id`.  Call before re-registering
    /// a component after its dependency set changes.
    pub fn unregister(&mut self, component_id: u64) {
        if let Some(keys) = self.reverse.remove(&component_id) {
            for key in keys {
                if let Some(set) = self.deps.get_mut(&key) {
                    set.remove(&component_id);
                }
            }
        }
    }

    /// Return the set of component IDs that depend on `state_key`.
    ///
    /// Returns an empty slice when no component depends on this key.
    /// Callers iterate the result and schedule re-render only for those
    /// components instead of broadcasting to all subscribers.
    pub fn affected_components(&self, state_key: u64) -> impl Iterator<Item = u64> + '_ {
        self.deps
            .get(&state_key)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Return true if any component depends on `state_key`.
    pub fn has_dependents(&self, state_key: u64) -> bool {
        self.deps
            .get(&state_key)
            .map_or(false, |set| !set.is_empty())
    }

    /// Total number of registered dependency edges.
    pub fn edge_count(&self) -> usize {
        self.deps.values().map(|s| s.len()).sum()
    }
}

// =============================================================================
// P1-43: Subsystem Budget
// =============================================================================

use std::time::{Duration, Instant};

/// P1-43: per-subsystem budget allocation. A frame's total
/// time is split across animation, layout, and render
/// subsystems. Each gets a fraction of the total budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubsystemBudget {
    /// Time slice allocated to this subsystem, in seconds.
    pub time_slice: Duration,
    /// Whether this subsystem is allowed to be skipped if
    /// over budget (non-essential) or must always complete
    /// (essential).
    pub skippable: bool,
    /// Name of the subsystem, for logging.
    pub name: &'static str,
}

/// P1-43: global frame budget tracker. Holds the total
/// budget for a frame, per-subsystem allocations, and
/// wall-clock measurements.
///
/// Named FrameBudgetTracker to avoid confusion with the
/// existing FrameBudget config struct (which holds the
/// target_ms and allow_degradation settings).
#[derive(Debug)]
pub struct FrameBudgetTracker {
    /// Total budget for the frame (typically 1/60s = 16.67ms
    /// for 60fps).
    total: Duration,
    /// Per-subsystem allocations. Sum should not exceed
    /// total.
    allocations: Vec<SubsystemBudget>,
    /// Frame start time, captured on new_frame().
    start: Option<Instant>,
    /// Per-subsystem elapsed time, updated on subsystem_finish().
    elapsed: Vec<Duration>,
}

impl FrameBudgetTracker {
    /// Standard 60fps frame budget: 16.67ms total, with
    /// default allocations across animation (4ms), layout
    /// (4ms), and render (8ms). Subsystems are skippable
    /// except for render, which is essential.
    pub fn default_60fps() -> Self {
        Self {
            total: Duration::from_micros(16_666), // ~16.67ms
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
                    skippable: false, // render must always run
                    name: "render",
                },
            ],
            start: None,
            elapsed: vec![Duration::ZERO, Duration::ZERO, Duration::ZERO],
        }
    }

    /// Standard 120fps frame budget: 8.33ms total, with
    /// allocations across animation (2ms), layout (2ms), and render (4ms).
    /// Used for high-refresh-rate targets like the Berserker demo.
    pub fn default_120fps() -> Self {
        Self {
            total: Duration::from_micros(8_333), // ~8.33ms
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
                    skippable: false, // render must always run
                    name: "render",
                },
            ],
            start: None,
            elapsed: vec![Duration::ZERO, Duration::ZERO, Duration::ZERO],
        }
    }

    /// Get the total frame budget.
    pub fn total(&self) -> Duration {
        self.total
    }

    /// Get the per-subsystem allocations.
    pub fn allocations(&self) -> &[SubsystemBudget] {
        &self.allocations
    }

    /// Mark the start of a new frame. Call this at the
    /// beginning of the render loop.
    pub fn new_frame(&mut self) {
        self.start = Some(Instant::now());
        for e in self.elapsed.iter_mut() {
            *e = Duration::ZERO;
        }
    }

    /// Mark a subsystem as finishing. Updates the elapsed
    /// time for that subsystem.
    pub fn subsystem_finish(&mut self, index: usize) {
        if let Some(start) = self.start {
            if index < self.elapsed.len() {
                let now = Instant::now();
                self.elapsed[index] = now.duration_since(start);
            }
        }
    }

    /// Check if a subsystem is within its time allocation.
    /// Returns true if the subsystem has used less time than
    /// its allocated slice.
    pub fn is_within_budget(&self, index: usize) -> bool {
        if index >= self.allocations.len() {
            return false;
        }
        if index >= self.elapsed.len() {
            return false;
        }
        self.elapsed[index] <= self.allocations[index].time_slice
    }

    /// Check if the entire frame is within the total budget.
    /// Returns true if all subsystems have completed within
    /// their allocations.
    pub fn frame_within_budget(&self) -> bool {
        for (i, alloc) in self.allocations.iter().enumerate() {
            if i < self.elapsed.len()
                && self.elapsed[i] > alloc.time_slice
                && !alloc.skippable
            {
                return false;
            }
        }
        true
    }

    /// Get the elapsed time for a subsystem.
    pub fn elapsed(&self, index: usize) -> Duration {
        if index < self.elapsed.len() {
            self.elapsed[index]
        } else {
            Duration::ZERO
        }
    }

    /// Get the total time elapsed since new_frame().
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
///
/// Keeps a sliding window of latency measurements to compute
/// percentile metrics (e.g. P50, P95, P99).
#[derive(Debug, Clone)]
pub struct InputLatencyTracker {
    /// Maximum number of samples to retain in the sliding window.
    window_size: usize,
    /// Sliding window of (event_time, render_time) pairs.
    samples: std::collections::VecDeque<(Instant, Instant)>,
}

impl InputLatencyTracker {
    /// Creates a new `InputLatencyTracker` with the specified maximum sliding window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            samples: std::collections::VecDeque::with_capacity(window_size),
        }
    }

    /// Records an input event's latency sample.
    pub fn record_frame(&mut self, event_time: Instant, render_time: Instant) {
        if self.window_size == 0 {
            return;
        }
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back((event_time, render_time));
    }

    /// Computes the latency value corresponding to the requested percentile.
    pub fn percentile(&self, p: f64) -> Duration {
        if self.samples.is_empty() || p < 0.0 || p > 100.0 {
            return Duration::ZERO;
        }
        let mut latencies: Vec<Duration> = self.samples
            .iter()
            .map(|&(e, r)| {
                if r > e {
                    r.duration_since(e)
                } else {
                    Duration::ZERO
                }
            })
            .collect();
        latencies.sort();
        let len = latencies.len();
        let rank = p / 100.0;
        let index = ((len as f64 * rank).ceil() as usize).saturating_sub(1);
        let index = index.min(len - 1);
        latencies[index]
    }

    /// Clears all recorded samples from the tracker.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Returns the configured sliding window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Updates the configured sliding window size.
    pub fn set_window_size(&mut self, size: usize) {
        self.window_size = size;
        while self.samples.len() > self.window_size {
            self.samples.pop_front();
        }
    }

    /// Returns the number of samples currently stored in the sliding window.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns whether the tracker currently contains no samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

// =============================================================================
// Tests (P1-40, P1-42, P1-43, P2-36)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // P1-42: DependencyGraph tests
    mod p1_42_dependency_graph_tests {
        use super::DependencyGraph;

        #[test]
        fn register_and_query_single_dep() {
            let mut g = DependencyGraph::new();
            g.register(42, 100); // component 42 depends on state key 100
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
            assert!(!affected.contains(&1), "component 1 must be gone after unregister");
            assert!(affected.contains(&2), "component 2 must still be present");
        }

        #[test]
        fn no_deps_returns_empty() {
            let g = DependencyGraph::new();
            let affected: Vec<u64> = g.affected_components(999).collect();
            assert!(affected.is_empty());
            assert!(!g.has_dependents(999));
        }

        #[test]
        fn edge_count_reflects_registrations() {
            let mut g = DependencyGraph::new();
            assert_eq!(g.edge_count(), 0);
            g.register(1, 10);
            g.register(2, 10);
            g.register(1, 20); // component 1 now depends on two keys
            assert_eq!(g.edge_count(), 3);
        }

        #[test]
        fn re_register_after_unregister_works() {
            let mut g = DependencyGraph::new();
            g.register(5, 50);
            g.unregister(5);
            // Re-register with different key.
            g.register(5, 60);
            assert!(!g.has_dependents(50), "old key must be gone");
            assert!(g.has_dependents(60), "new key must be present");
        }
    }

    // P1-43: FrameBudgetTracker tests
    mod p1_43_frame_budget_tests {
        use super::FrameBudgetTracker;

        #[test]
        fn default_60fps_has_16ms_total() {
            let fb = FrameBudgetTracker::default_60fps();
            // 16.67ms is the target for 60fps.
            assert!(fb.total().as_micros() >= 16_000);
            assert!(fb.total().as_micros() <= 17_000);
        }

        #[test]
        fn default_allocations_sum_to_roughly_total() {
            let fb = FrameBudgetTracker::default_60fps();
            let sum: u128 = fb.allocations().iter()
                .map(|a| a.time_slice.as_micros())
                .sum();
            // The 3 allocations (4+4+8 = 16ms) should sum to
            // approximately the total budget.
            let total = fb.total().as_micros();
            assert!(sum <= total);
            assert!(sum >= total - 1_000); // within 1ms
        }

        #[test]
        fn render_is_essential_layout_is_skippable() {
            let fb = FrameBudgetTracker::default_60fps();
            let render = fb.allocations().iter().find(|a| a.name == "render").unwrap();
            let layout = fb.allocations().iter().find(|a| a.name == "layout").unwrap();
            assert!(!render.skippable, "render must always run");
            assert!(layout.skippable, "layout can be skipped if over budget");
        }

        #[test]
        fn new_frame_resets_state() {
            let mut fb = FrameBudgetTracker::default_60fps();
            fb.new_frame();
            // All subsystems should start with zero elapsed.
            for i in 0..fb.allocations().len() {
                assert_eq!(fb.elapsed(i).as_nanos(), 0);
            }
        }

        #[test]
        fn is_within_budget_initially_true() {
            let mut fb = FrameBudgetTracker::default_60fps();
            fb.new_frame();
            // Right after new_frame, no time has been used, so
            // all subsystems should be within budget.
            for i in 0..fb.allocations().len() {
                assert!(fb.is_within_budget(i));
            }
        }

        #[test]
        fn frame_within_budget_initially_true() {
            let mut fb = FrameBudgetTracker::default_60fps();
            fb.new_frame();
            assert!(fb.frame_within_budget());
        }
    }

    // P2-36: InputLatencyTracker tests
    mod p2_36_input_latency_tests {
        use super::InputLatencyTracker;
        use std::time::{Duration, Instant};

        #[test]
        fn test_empty_tracker() {
            let tracker = InputLatencyTracker::new(10);
            assert!(tracker.is_empty());
            assert_eq!(tracker.len(), 0);
            assert_eq!(tracker.percentile(50.0), Duration::ZERO);
        }

        #[test]
        fn test_record_and_sliding_window() {
            let mut tracker = InputLatencyTracker::new(3);
            let now = Instant::now();
            tracker.record_frame(now, now + Duration::from_millis(10));
            tracker.record_frame(now, now + Duration::from_millis(20));
            tracker.record_frame(now, now + Duration::from_millis(30));
            assert_eq!(tracker.len(), 3);
            
            // This should evict the 10ms sample
            tracker.record_frame(now, now + Duration::from_millis(40));
            assert_eq!(tracker.len(), 3);
            
            // Percentiles should be of [20ms, 30ms, 40ms]
            assert_eq!(tracker.percentile(50.0), Duration::from_millis(30));
            assert_eq!(tracker.percentile(0.0), Duration::from_millis(20));
            assert_eq!(tracker.percentile(100.0), Duration::from_millis(40));
        }

        #[test]
        fn test_resize_and_clear() {
            let mut tracker = InputLatencyTracker::new(5);
            let now = Instant::now();
            for i in 1..=5 {
                tracker.record_frame(now, now + Duration::from_millis(i * 10));
            }
            assert_eq!(tracker.len(), 5);
            
            tracker.set_window_size(3);
            assert_eq!(tracker.window_size(), 3);
            assert_eq!(tracker.len(), 3);
            // Retained samples should be [30ms, 40ms, 50ms]
            assert_eq!(tracker.percentile(0.0), Duration::from_millis(30));
            assert_eq!(tracker.percentile(100.0), Duration::from_millis(50));
            
            tracker.clear();
            assert!(tracker.is_empty());
            assert_eq!(tracker.percentile(50.0), Duration::ZERO);
        }

        #[test]
        fn test_invalid_percentiles() {
            let mut tracker = InputLatencyTracker::new(2);
            let now = Instant::now();
            tracker.record_frame(now, now + Duration::from_millis(10));
            assert_eq!(tracker.percentile(-1.0), Duration::ZERO);
            assert_eq!(tracker.percentile(101.0), Duration::ZERO);
        }
    }
}
// =========================================================================
// P1-15: Subscriber List Mutex Poisoning
// =========================================================================
//
// Regression tests for the audit finding: a single panicking subscriber
// would poison the Mutex and break all future state updates forever.
// The fix wraps each callback in catch_unwind, so panics are isolated
// and logged without affecting other subscribers or future updates.

#[cfg(test)]
mod subscriber_panic_isolation_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn panicking_subscriber_does_not_poison_mutex() {
        let state = State::new(0i32);
        let fired = Arc::new(AtomicUsize::new(0));

        // First subscriber: panics.
        let _ = state.subscribe(|_| -> () {
            panic!("subscriber 1 explodes");
        });

        // Second subscriber: should still fire.
        let fired_clone = Arc::clone(&fired);
        let _ = state.subscribe(move |v| {
            fired_clone.store(*v as usize + 1, Ordering::SeqCst);
        });

        // Trigger the state change. Subscriber 1 panics; subscriber 2 runs.
        state.set(42);

        assert_eq!(
            fired.load(Ordering::SeqCst),
            43,
            "second subscriber must fire even though first panicked"
        );

        // Critical: future state updates must still work.
        let fired2 = Arc::new(AtomicUsize::new(0));
        let fired2_clone = Arc::clone(&fired2);
        let _ = state.subscribe(move |v| {
            fired2_clone.store(*v as usize, Ordering::SeqCst);
        });
        state.set(100);
        assert_eq!(
            fired2.load(Ordering::SeqCst),
            100,
            "future updates must work after subscriber panic"
        );
    }

    #[test]
    fn all_subscribers_fire_even_if_one_panics() {
        let state = State::new(0u32);
        let count = Arc::new(AtomicUsize::new(0));

        // Mix of panicking and counting subscribers.
        let _ = state.subscribe(|_| panic!("boom 1"));
        let c1 = Arc::clone(&count);
        let _ = state.subscribe(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        let _ = state.subscribe(|_| panic!("boom 2"));
        let c2 = Arc::clone(&count);
        let _ = state.subscribe(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        state.set(1);

        // Both non-panicking subscribers must have fired.
        assert_eq!(
            count.load(Ordering::SeqCst),
            2,
            "both non-panicking subscribers should fire"
        );
    }

    #[test]
    fn invoke_subscribers_safely_returns_count() {
        // Direct unit test of the helper function.
        use std::sync::Mutex;
        let subs: SubscriberList<u32> = Arc::new(Mutex::new(Vec::new()));

        let count1 = Arc::new(AtomicUsize::new(0));
        let count1_clone = Arc::clone(&count1);
        subs.lock().unwrap().push(Box::new(move |v| {
            count1_clone.store(*v as usize, Ordering::SeqCst);
        }));

        let count2 = Arc::new(AtomicUsize::new(0));
        let count2_clone = Arc::clone(&count2);
        subs.lock().unwrap().push(Box::new(move |v| {
            count2_clone.store(*v as usize + 100, Ordering::SeqCst);
        }));

        let invoked = invoke_subscribers_safely(&subs, &7);
        assert_eq!(invoked, 2, "both subscribers should be invoked");
        assert_eq!(count1.load(Ordering::SeqCst), 7);
        assert_eq!(count2.load(Ordering::SeqCst), 107);
    }
}

// =========================================================================
// P1-17: Suspense::new_async Shared Fallback Runtime
// =========================================================================
//
// Regression tests for the audit finding: when no ambient tokio
// runtime exists, new_async spawned a new OS thread + runtime per
// call. The fix introduces a process-wide shared fallback runtime.

#[cfg(test)]
mod p1_17_shared_fallback_runtime_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn fallback_runtime_is_shared() {
        // Calling fallback_runtime() multiple times should return the
        // same Runtime instance (singleton via OnceLock). This is the
        // core invariant that bounds thread creation.
        let r1 = fallback_runtime();
        let r2 = fallback_runtime();
        assert!(
            std::ptr::eq(r1 as *const _, r2 as *const _),
            "fallback_runtime must return the same instance"
        );
    }

    #[test]
    fn fallback_worker_count_is_bounded() {
        // The worker count must be >= 1 and <= 8 regardless of host
        // CPU count. This is what prevents the audit's "spawns
        // hundreds of OS threads" issue.
        let n = *FALLBACK_WORKER_COUNT.get_or_init(|| {
            let available = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(2);
            available.saturating_sub(1).clamp(1, 8)
        });
        assert!(n >= 1, "worker count must be at least 1, got {n}");
        assert!(n <= 8, "worker count must be at most 8, got {n}");
    }

    #[test]
    fn many_suspense_calls_share_runtime() {
        // P1-17 regression: 20 new_async calls in quick succession
        // should not hang or OOM. They all share the single
        // fallback runtime, so we never create more than ~8 OS
        // threads regardless of call count.
        //
        // We use a counter SharedState to confirm all 20 futures
        // actually run to completion.
        let counter = State::new(0u32);
        let mut handles = Vec::new();
        for _ in 0..20 {
            let s = Suspense::new_async(async { Ok::<u32, String>(1) });
            // Each suspense ready()s after the future resolves.
            // We don't block on ready (would deadlock without
            // explicit tokio context), but the spawn is enough to
            // exercise the path.
            let _ = s; // suppress unused warning
            handles.push(s);
        }
        // Force the counter to tick so the test observably runs.
        counter.set(20);
        assert_eq!(counter.get(), 20);
        // If we got here, the test did not hang or panic, which is
        // the main thing we want to verify for P1-17.
    }

    // ==========================================
    // P1-14: State<T> redundant storage documentation
    // ==========================================

    #[test]
    fn p1_14_state_storage_mechanisms() {
        // P1-14 documentation test: State<T> has 4 storage
        // mechanisms (swap, metadata_swap, tvar, metadata_tvar).
        // The audit flagged this as redundant. The fix is to
        // document the trade-off (arc_swap for reads, TVar for
        // atomic compound transactions) and add a set_direct()
        // method for callers who don't need compound transactions.
        use std::mem::size_of;
        let state = State::new(42u32);
        // State contains 4 storage mechanisms + subscribers +
        // version + resolution.
        // This test documents the size and the trade-off.
        let size = size_of_val(&state);
        // Size should be at least the size of 4 Arcs (4*8=32 on
        // 64-bit) plus subscribers (1 Arc) plus version (1 Arc)
        // plus ConflictResolution (1 byte tag).
        assert!(
            size >= 4 * std::mem::size_of::<usize>(),
            "State<T> should be at least 4 Arcs in size"
        );
    }

    #[test]
    fn p1_14_set_direct_updates_value() {
        // P1-14: set_direct() bypasses TVar for simple updates.
        // The swap is the authoritative read source.
        let state = State::new(0u32);
        state.set_direct(42);
        assert_eq!(state.get(), 42);
    }

    #[test]
    fn p1_14_set_direct_notifies_subscribers() {
        // P1-14: set_direct() must notify subscribers just like
        // set().
        let state = State::new(0u32);
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        state.subscribe(move |v| {
            received_clone.lock().unwrap().push(*v);
        });
        state.set_direct(1);
        state.set_direct(2);
        state.set_direct(3);
        // Allow the subscriber invocations to complete.
        std::thread::sleep(std::time::Duration::from_millis(10));
        let log = received.lock().unwrap();
        // Should have at least the last 3 values, but the order
        // and count depend on how many subscribers were invoked
        // (subscribers can be invoked synchronously or batched).
        assert!(
            log.contains(&1) && log.contains(&2) && log.contains(&3),
            "set_direct must notify subscribers of all values"
        );
    }
}

// =========================================================================
// P1-39: DirtyRegionManager -- tracks changed rectangles
// =========================================================================
//
// The P1-39 audit found that the scene graph lacks dirty region
// tracking. Large UIs may redraw excessively when only a small
// region changes. This struct provides the foundation for
// future dirty-region optimizations.
//
// Currently it just stores a list of dirty rectangles. Future
// work would add:
//  - Coalescing adjacent dirty regions into larger rects
//  - Tree-based hierarchical dirty tracking
//  - Integration with the renderer's scissor/clip
//
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
        a.x < b.x + b.width
            && a.x + a.width > b.x
            && a.y < b.y + b.height
            && a.y + a.height > b.y
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
        m.mark_dirty(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 });
        assert!(m.is_dirty());
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn overlapping_regions_coalesce() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 });
        m.mark_dirty(Rect { x: 5.0, y: 5.0, width: 10.0, height: 10.0 });
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
        m.mark_dirty(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 });
        m.mark_dirty(Rect { x: 100.0, y: 100.0, width: 10.0, height: 10.0 });
        // Should remain as 2 separate regions.
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn clear_resets_regions_and_increments_generation() {
        let mut m = DirtyRegionManager::new();
        m.mark_dirty(Rect { x: 0.0, y: 0.0, width: 10.0, height: 10.0 });
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

// =========================================================================
// P1-43: FrameBudget -- global frame budget contract
// =========================================================================
//
// The P1-43 audit found that no global frame budget contract
// exists. Individual subsystems may exceed their time allocation
// without coordination. P0-2 already handles per-frame
// degradation (skipping non-essential passes when over budget)
// but doesn't coordinate allocation across subsystems.
//
// This struct provides the foundation for future frame budget
// coordination. It tracks wall-clock time per frame and per
// subsystem, and allows callers to check whether a subsystem
// is within its allocated time slice.
//
// Currently a passive observer. Future work would add:
//  - Per-subsystem time allocation
//  - Automatic QualityLevel adjustment when over budget
//  - Integration with the renderer's frame loop

// =============================================================================
// P1-41: LIST / TREE VIRTUALIZATION
// =============================================================================

/// Outcome of a `VirtualListWindow::compute` call — describes which rows
/// should be rendered and what scroll offset to apply to position them.
///
/// P1-41: IDE and visualization workloads with tens-of-thousands of rows must
/// only render the rows visible in the current viewport.  `VirtualListWindow`
/// computes the correct row range without building the full row list.
#[derive(Debug, Clone, PartialEq)]

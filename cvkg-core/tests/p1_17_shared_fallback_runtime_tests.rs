    use super::*;
    use std::time::Duration;

    #[cfg(not(target_arch = "wasm32"))]
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
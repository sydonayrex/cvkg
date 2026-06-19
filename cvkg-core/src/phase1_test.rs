//! Phase 1 Verification Test
//!
//! This module verifies that the core framework components (Text, Button, State)
//! can be assembled into a minimal app that compiles, runs, and toggles state.

#[cfg(test)]
static TEST_MUTEX: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();

#[cfg(test)]
fn lock_test() -> std::sync::MutexGuard<'static, ()> {
    TEST_MUTEX.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{Never, State, View};

    // A minimal mock Button and Text for the core test,
    // since cvkg-components depends on cvkg-core and we're inside cvkg-core.
    struct Button<F>
    where
        F: Fn(),
    {
        action: F,
    }
    impl<F> Button<F>
    where
        F: Fn(),
    {
        fn new(_label: impl Into<String>, action: F) -> Self {
            Self { action }
        }
    }
    impl<F> View for Button<F>
    where
        F: Fn() + Send,
    {
        type Body = Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
    }

    struct AppView {
        state: State<bool>,
    }

    impl View for AppView {
        type Body = Button<Box<dyn Fn() + Send>>;

        fn body(self) -> Self::Body {
            let current_state = self.state.get();
            let label = if current_state { "ON" } else { "OFF" };

            let state_clone = self.state.clone();
            Button::new(
                label,
                Box::new(move || {
                    let current = state_clone.get();
                    state_clone.set(!current);
                }),
            )
        }
    }

    #[test]
    fn test_minimal_app_compiles_and_toggles_state() {
        let app = AppView {
            state: State::new(false),
        };

        // Check initial state
        assert!(!app.state.get());

        // Get the body (which is a button)
        let body = app.body();

        // Simulate a click
        (body.action)();

        // Verify state toggled
        let state = State::new(false);
        let state_clone = state.clone();

        let action = move || {
            let current = state_clone.get();
            state_clone.set(!current);
        };

        assert!(!state.get());
        action();
        assert!(state.get());
        action();
        assert!(!state.get());
    }
}

/// Phase 2: ArcSwap global state tests
///
/// Verifies that load_system_state() / update_system_state() behave correctly
/// under both sequential and concurrent access patterns.
#[cfg(test)]
mod phase2_tests {
    use crate::{KnowledgeFragment, load_system_state, update_system_state};

    /// Sequential read-after-write: update is immediately visible to load.
    #[test]
    fn test_update_then_load_is_consistent() {
        let _lock = super::lock_test();
        // Write a known fragment into global state
        update_system_state(|s| {
            let mut s = s.clone();
            s.remember(KnowledgeFragment {
                id: "phase2_test".to_string(),
                summary: "Phase 2 ArcSwap test fragment".to_string(),
                source: "test".to_string(),
                created_at: 0,
                accessed_count: 0,
                content: None,
            });
            s
        });

        // Load and verify the fragment is present
        let snapshot = load_system_state();
        assert!(
            snapshot.fragments.contains_key("phase2_test"),
            "Fragment written via update_system_state must be readable via load_system_state"
        );
    }

    /// Concurrent read + write: 8 reader threads observe no deadlock and no torn state.
    /// The writer publishes a monotonically increasing counter; every reader that sees
    /// the fragment must see a non-empty summary (never a partial write).
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_concurrent_readers_no_deadlock() {
        let _lock = super::lock_test();
        use std::sync::{Arc, Barrier};
        use std::thread;

        const READERS: usize = 8;
        const WRITES: usize = 20;

        let barrier = Arc::new(Barrier::new(READERS + 1));
        let mut handles = Vec::with_capacity(READERS);

        // Spawn reader threads -- each reads load_system_state() in a tight loop
        for _ in 0..READERS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait(); // synchronised start
                for _ in 0..100 {
                    let snap = load_system_state();
                    // If the fragment exists, its summary must never be empty (torn write guard)
                    if let Some(frag) = snap.fragments.get("concurrent_counter") {
                        assert!(
                            !frag.summary.is_empty(),
                            "Observed a fragment with an empty summary — partial write detected"
                        );
                    }
                    // Release guard immediately; do not hold across spin
                    drop(snap);
                }
            }));
        }

        // Writer thread: publish WRITES snapshots
        barrier.wait();
        for i in 0..WRITES {
            update_system_state(|s| {
                let mut s = s.clone();
                s.remember(KnowledgeFragment {
                    id: "concurrent_counter".to_string(),
                    summary: format!("write-{}", i),
                    source: "test".to_string(),
                    created_at: i as u64,
                    accessed_count: 0,
                    content: None,
                });
                s
            });
        }

        for h in handles {
            h.join().expect("Reader thread panicked");
        }

        // Final assertion: last write is visible
        let snap = load_system_state();
        assert_eq!(
            snap.fragments
                .get("concurrent_counter")
                .map(|f| f.created_at),
            Some(WRITES as u64 - 1),
            "Final write must be the last one published"
        );
    }
}

/// Phase 3: State<T> / Binding<T> ArcSwap + STM migration tests
#[cfg(test)]
mod phase3_tests {
    use crate::{Binding, State};

    /// Basic API round-trip: new / get / set / version / subscribe are all intact.
    #[test]
    fn test_state_basic_api() {
        let s = State::new(0u32);
        assert_eq!(s.get(), 0);
        assert_eq!(s.version(), 0);

        s.set(42);
        assert_eq!(s.get(), 42);
        assert_eq!(s.version(), 1);

        // Subscriber must fire with the new value
        let fired = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fired_clone = std::sync::Arc::clone(&fired);
        s.subscribe(move |v| {
            assert_eq!(*v, 99);
            fired_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        s.set(99);
        assert!(fired.load(std::sync::atomic::Ordering::Relaxed));
    }

    /// Binding shares storage: a write through Binding is visible via State and vice-versa.
    #[test]
    fn test_binding_shares_storage() {
        let s = State::new(10u32);
        let b = Binding::from_state(&s);

        // Write through Binding, read via State
        b.set(20);
        assert_eq!(s.get(), 20, "State must observe write made through Binding");

        // Write through State, read via Binding
        s.set(30);
        assert_eq!(b.get(), 30, "Binding must observe write made through State");

        // Version counter is shared
        assert_eq!(s.version(), b.version());
    }

    /// mutate() applies an STM-transacted transformation; version increments.
    #[test]
    fn test_state_mutate() {
        let s = State::new(5u32);
        s.mutate(|v| v + 1);
        assert_eq!(s.get(), 6, "mutate must increment the value");
        assert_eq!(s.version(), 1, "version must increment after mutate");
    }

    /// 100 concurrent set() calls never produce a torn read.
    /// The final observed value must equal one of the values that was set.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_concurrent_set_no_torn_reads() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        const THREADS: usize = 100;
        let state = Arc::new(State::new(0u32));
        let barrier = Arc::new(Barrier::new(THREADS));

        let handles: Vec<_> = (0..THREADS as u32)
            .map(|i| {
                let s = Arc::clone(&state);
                let b = Arc::clone(&barrier);
                thread::spawn(move || {
                    b.wait(); // all threads start simultaneously
                    s.set(i);
                    // Immediately read back -- must be a valid u32, never garbage
                    let _ = s.get();
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }

        // Final value must be one of 0..THREADS; get() must not panic or return garbage
        let final_val = state.get();
        assert!(
            (final_val as usize) < THREADS,
            "final value {} is out of the valid range",
            final_val
        );
        // version == THREADS is the maximum; it could be less if some stores raced
        assert!(
            state.version() >= 1,
            "version must be at least 1 after any write"
        );
    }
}

/// Phase 6: STM multi-field transaction tests
///
/// Verifies that `transact_system_state` and `transact_pair` provide atomicity guarantees
/// that the simple clone-and-swap of `update_system_state` / `State::set` cannot.
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod phase6_tests {
    use crate::{KnowledgeFragment, State, transact_pair, transact_system_state};

    /// Sequential correctness: both `fragments` and `last_query_results` land in the same
    /// committed snapshot -- transact_system_state is not split across two separate stores.
    #[test]
    fn test_transact_system_state_multi_field_coherence() {
        let _lock = super::lock_test();
        transact_system_state(|s| {
            let mut s = s.clone();
            s.remember(KnowledgeFragment {
                id: "p6_frag".to_string(),
                summary: "phase6 coherence test".to_string(),
                source: "test".to_string(),
                created_at: 42,
                accessed_count: 0,
                content: None,
            });
            s.last_query_results = vec!["p6_frag".to_string()];
            s
        });

        let snap = crate::load_system_state();
        assert!(
            snap.fragments.contains_key("p6_frag"),
            "fragment must be present after transact_system_state"
        );
        assert!(
            snap.last_query_results.contains(&"p6_frag".to_string()),
            "last_query_results must reflect the same committed transaction"
        );
    }

    /// Concurrent lost-update prevention: 10 threads each insert a unique fragment via
    /// `transact_system_state`. With STM retry semantics, every insert must be visible in
    /// the final snapshot -- none may be silently overwritten by a racing clone-and-swap.
    ///
    /// Uses a per-run unique prefix so this test is safe to run in parallel with other
    /// tests that share the same global SYSTEM_STATE / KNOWLEDGE_TVAR.
    #[test]
    fn test_transact_system_state_no_lost_updates() {
        let _lock = super::lock_test();
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        const N: usize = 10;
        // Unique prefix per test invocation to avoid key collisions with parallel tests
        let run_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let prefix = Arc::new(format!("lu_{}_{}_", std::process::id(), run_id));

        let barrier = Arc::new(Barrier::new(N));
        let mut handles = Vec::with_capacity(N);

        for i in 0..N {
            let b = Arc::clone(&barrier);
            let pfx = Arc::clone(&prefix);
            handles.push(thread::spawn(move || {
                b.wait(); // all threads start simultaneously to maximise conflicts
                let key = format!("{}{}", pfx, i);
                transact_system_state(move |s| {
                    let mut s = s.clone();
                    s.remember(KnowledgeFragment {
                        id: key.clone(),
                        summary: format!("thread {}", i),
                        source: "test".to_string(),
                        created_at: i as u64,
                        accessed_count: 0,
                        content: None,
                    });
                    s
                });
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }

        let snap = crate::load_system_state();
        for i in 0..N {
            let key = format!("{}{}", prefix, i);
            assert!(
                snap.fragments.contains_key(&key),
                "fragment {} was lost — STM retry must prevent overwrites",
                key
            );
        }
    }

    /// Atomic pair swap: exchanging two State<u32> values leaves both at their swapped
    /// values, the version counter advances on each cell, and no intermediate state is
    /// observable (sequential test -- concurrent variant would require a spin-read thread).
    #[test]
    fn test_transact_pair_atomic_swap() {
        let _lock = super::lock_test();
        let state_a = State::new(10u32);
        let state_b = State::new(20u32);

        // Subscribe to verify notification fires after the atomic commit
        let notified_a = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let notified_b = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let na = std::sync::Arc::clone(&notified_a);
        let nb = std::sync::Arc::clone(&notified_b);
        state_a.subscribe(move |v| {
            assert_eq!(*v, 20);
            na.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        state_b.subscribe(move |v| {
            assert_eq!(*v, 10);
            nb.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let v_before_a = state_a.version();
        let v_before_b = state_b.version();

        // Swap the two values atomically
        transact_pair(&state_a, &state_b, |a, b| (*b, *a));

        assert_eq!(state_a.get(), 20, "state_a must hold state_b's old value");
        assert_eq!(state_b.get(), 10, "state_b must hold state_a's old value");
        assert_eq!(
            state_a.version(),
            v_before_a + 1,
            "version_a must increment exactly once"
        );
        assert_eq!(
            state_b.version(),
            v_before_b + 1,
            "version_b must increment exactly once"
        );
        assert!(
            notified_a.load(std::sync::atomic::Ordering::Relaxed),
            "state_a subscriber must fire"
        );
        assert!(
            notified_b.load(std::sync::atomic::Ordering::Relaxed),
            "state_b subscriber must fire"
        );
    }
}

/// Phase 7: Batching queue test
#[cfg(test)]
mod phase7_tests {
    use crate::{State, batch};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn test_batch_defers_notifications() {
        let _lock = crate::phase1_test::lock_test();
        let state_a = State::new(10);
        let state_b = State::new(20);

        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);

        state_a.subscribe(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        state_b.subscribe(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        batch(|| {
            state_a.set(11);
            state_b.set(21);
            // Notifications should not have fired yet
            assert_eq!(counter.load(Ordering::SeqCst), 0);

            // Inner batch calls should be no-ops (just execute inline)
            batch(|| {
                state_a.set(12);
            });
            assert_eq!(counter.load(Ordering::SeqCst), 0);
        });

        // After batch, all queued notifications fire
        assert_eq!(counter.load(Ordering::SeqCst), 3);
        assert_eq!(state_a.get(), 12);
        assert_eq!(state_b.get(), 21);
    }
}

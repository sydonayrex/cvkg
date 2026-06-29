use crate::agents;
use crate::*;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// Thread-safe reactive state with multiple storage mechanism.
///
/// State<T> supports both lightweight single-reader access (via arc_swap)
/// and compound atomic transactions (via stm::TVar for non-WASM targets).
///
/// # Storage Trade-offs
/// - `swap` (ArcSwap): Lock-free reads, atomic pointer swap. Best for read-heavy workloads.
/// - `tvar` (TVar): Blocking compound transactions. Best for coordinated multi-state operations.
/// - `subscribers`: Async notification list for reactive updates.
///
/// # Example
/// ```no_run
/// use cvkg_core::State;
/// let state = State::new(42u32);
/// state.set(100);
/// assert_eq!(state.get(), 100);
/// ```
#[derive(Clone)]
pub struct State<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    metadata_swap: Arc<arc_swap::ArcSwap<Option<agents::MutationMetadata>>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    metadata_tvar: Arc<stm::TVar<Option<agents::MutationMetadata>>>,
    subscribers: SubscriberList<T>,
    version: Arc<AtomicU64>,
    resolution: agents::ConflictResolution,
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    /// Create a new reactive state with the given initial value.
    pub fn new(value: T) -> Self {
        // Initialize metadata (None for fresh state, no mutation history)
        let metadata: Option<agents::MutationMetadata> = None;
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            tvar: Arc::new(stm::TVar::new(value.clone())),
            #[cfg(not(target_arch = "wasm32"))]
            metadata_tvar: Arc::new(stm::TVar::new(metadata)),
            swap: Arc::new(arc_swap::ArcSwap::new(Arc::new(value))),
            metadata_swap: Arc::new(arc_swap::ArcSwap::new(Arc::new(metadata))),
            subscribers: Arc::new(std::sync::Mutex::new(Vec::new())),
            version: Arc::new(AtomicU64::new(0)),
            resolution: agents::ConflictResolution::LastWriterWins,
        }
    }

    /// Get the current value.
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }

    /// Set the conflict resolution strategy.
    pub fn with_resolution(mut self, resolution: agents::ConflictResolution) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set the value via arc_swap (fast path). Notifies subscribers.
    /// Respects conflict resolution strategy (PriorityWins skips lower-priority writes).
    pub fn set(&self, value: T) {
        // Debug-only invariant: version must never overflow in practice.
        debug_assert!(
            self.version.load(Ordering::Acquire) < u64::MAX,
            "State version overflow"
        );
        #[cfg(not(target_arch = "wasm32"))]
        {
            let result = stm::atomically(|tx| {
                let new_meta = agents::get_current_mutation_metadata();
                let existing_meta = self.metadata_tvar.read(tx)?;
                let mut skip = false;
                if self.resolution == agents::ConflictResolution::PriorityWins
                    && let (Some(new_m), Some(old_m)) = (new_meta, existing_meta)
                    && new_m.priority < old_m.priority
                {
                    skip = true;
                }
                if !skip {
                    self.tvar.write(tx, value.clone())?;
                    self.metadata_tvar.write(tx, new_meta)?;
                    Ok::<_, stm::StmError>(false)
                } else {
                    Ok::<_, stm::StmError>(true)
                }
            });
            if result {
                return;
            }
        }
        self.swap.store(Arc::new(value.clone()));
        let _version = self.version.fetch_add(1, Ordering::Release) + 1;
        self.notify(&value);
    }

    /// Subscribe to state changes.
    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        self.subscribers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Box::new(callback));
    }

    /// Notify all subscribers of a new value.
    fn notify(&self, value: &T) {
        invoke_subscribers_safely(&self.subscribers, value);
    }

    /// Get the current version counter.
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Set the value directly via arc_swap without TVar transaction (fastest path).
    /// Suitable for high-frequency updates where transactional consistency is not required.
    /// Notifies subscribers just like `set()`.
    pub fn set_direct(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        let _version = self.version.fetch_add(1, Ordering::Release) + 1;
        // Bypass TVar: direct mode does not update the transactional storage.
        // This means compound transactions won't see this value.
        self.notify(&value);
    }

    /// Get metadata for the last mutation.
    pub fn get_metadata(&self) -> Option<agents::MutationMetadata> {
        **self.metadata_swap.load()
    }

    /// Set value with mutation metadata (actor + reason).
    pub fn set_with_metadata(&self, value: T, meta: agents::MutationMetadata) {
        self.swap.store(Arc::new(value.clone()));
        self.metadata_swap.store(Arc::new(Some(meta)));
        let _version = self.version.fetch_add(1, Ordering::Release) + 1;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = &*self.tvar;
            stm::atomically(|tx| {
                tvar.write(tx, value.clone())?;
                Ok::<_, stm::StmError>(())
            });
        }
        self.notify(&value);
    }
}

/// A read/write projection into a `State<T>` owned elsewhere.
#[derive(Clone)]
pub struct Binding<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    version: Arc<AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> Binding<T> {
    #[allow(dead_code)]
    pub(crate) fn new(swap: Arc<arc_swap::ArcSwap<T>>, version: Arc<AtomicU64>) -> Self {
        Self { swap, version }
    }

    /// Read the current value.
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }

    /// Set the value (updates the underlying State).
    pub fn set(&self, value: T) {
        self.swap.store(Arc::new(value));
        self.version.fetch_add(1, Ordering::Release);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::atomic::AtomicUsize;

    proptest! {
        #[test]
        fn test_state_version_monotonic(values in prop::collection::vec(any::<u32>(), 1..100)) {
            let state = State::new(0u32);
            let mut last_version = 0u64;
            for v in values {
                state.set(v);
                let current = state.version.load(Ordering::Relaxed);
                prop_assert!(current > last_version, "version must increase: {} <= {}", current, last_version);
                last_version = current;
            }
        }

        #[test]
        fn test_state_subscriber_called_on_set(
            vals in prop::collection::vec(any::<u32>(), 1..50)
        ) {
            let state = State::new(0u32);
            let call_count = Arc::new(AtomicUsize::new(0));
            let cc = call_count.clone();
            state.subscribe(move |_| { cc.fetch_add(1, Ordering::Relaxed); });
            let total = vals.len();
            for v in vals {
                state.set(v);
            }
            prop_assert_eq!(call_count.load(Ordering::Relaxed), total,
                "subscriber must be called once per set()");
        }

        #[test]
        fn test_state_value_roundtrip(vals in prop::collection::vec(any::<u32>(), 1..50)) {
            let state = State::new(0u32);
            for v in vals {
                state.set(v);
                prop_assert_eq!(state.get(), v);
            }
        }

        /// Multiple subscribers must ALL be called on every set.
        #[test]
        fn test_state_multiple_subscribers_all_called(
            vals in prop::collection::vec(any::<u32>(), 1..20),
            subscriber_count in 1..5usize,
        ) {
            let state = State::new(0u32);
            let counts: Vec<Arc<AtomicUsize>> = (0..subscriber_count)
                .map(|_| Arc::new(AtomicUsize::new(0)))
                .collect();
            for c in &counts {
                let cc = c.clone();
                state.subscribe(move |_| { cc.fetch_add(1, Ordering::Relaxed); });
            }
            for v in &vals {
                state.set(*v);
            }
            for (i, c) in counts.iter().enumerate() {
                prop_assert_eq!(
                    c.load(Ordering::Relaxed),
                    vals.len(),
                    "subscriber {} must be called {} times",
                    i,
                    vals.len()
                );
            }
        }

        /// Get always returns the most recently set value.
        #[test]
        fn test_state_get_returns_last_set(
            a in any::<u32>(),
            b in any::<u32>(),
        ) {
            let state = State::new(a);
            prop_assert_eq!(state.get(), a);
            state.set(b);
            prop_assert_eq!(state.get(), b);
        }

        /// Empty subscription list must not panic on set.
        #[test]
        fn test_state_no_subscribers_ok(vals in prop::collection::vec(any::<u32>(), 1..20)) {
            let state = State::new(0u32);
            for v in vals {
                state.set(v);
            }
            // Just verifying no panic occurred
            prop_assert!(true);
        }
    }
}

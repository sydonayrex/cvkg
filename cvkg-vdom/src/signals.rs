//! Fine-grained reactivity primitives (Signals) for modern UI state management.
//!
//! This module provides a foundational Signal architecture similar to SolidJS,
//! designed to replace expensive VDOM tree-diffing with targeted, instantaneous
//! side-effects when reactive state changes.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use cvkg_core::DependencyGraph;
use cvkg_core::DirtyFlags;

thread_local! {
    /// Tracks the currently executing effect to auto-subscribe it to signals.
    /// Thread-local because dependency tracking only matters for the thread executing the effect.
    static CURRENT_EFFECT: RwLock<Option<Arc<dyn EffectRunner>>> = const { RwLock::new(None) };
    /// Tracks signal reads during a component render pass.
    static COMPONENT_TRACKING: RwLock<Option<(u64, Vec<u64>)>> = const { RwLock::new(None) };
    /// OR-ed across every set_with_flags call in this frame.
    /// Read and reset by FrameScheduler::begin_frame().
    pub static FRAME_DIRTY_FLAGS: AtomicU8 = const { AtomicU8::new(0) };
}

static DEPENDENCY_GRAPH: OnceLock<RwLock<DependencyGraph>> = OnceLock::new();

pub fn dependency_graph() -> &'static RwLock<DependencyGraph> {
    DEPENDENCY_GRAPH.get_or_init(|| RwLock::new(DependencyGraph::new()))
}

pub trait EffectRunner: Send + Sync {
    fn id(&self) -> u64;
    fn run(self: Arc<Self>);
}

static NEXT_SIGNAL_ID: AtomicU64 = AtomicU64::new(1);

/// Subscriber entry with accumulated dirty flags.
/// Replaces bare `Arc<dyn EffectRunner>` so we can track per-subscriber flags.
#[derive(Clone)]
struct SubscriberEntry {
    runner: Arc<dyn EffectRunner>,
    /// Bitmask of DirtyFlags accumulated across set_with_flags calls
    /// since the last run. Reset when the runner is dispatched.
    accumulated: Arc<AtomicU8>,
}

/// A reactive primitive that holds a value and notifies subscribers when it changes.
pub struct Signal<T> {
    pub id: u64,
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<SubscriberEntry>>>,
    /// Monotonically increasing version counter. Incremented on every `set()`.
    /// Used by the VDOM layer to detect when a signal's value has changed since
    /// the last frame, enabling incremental VDOM rebuilds.
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl<T: Clone> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            id: NEXT_SIGNAL_ID.fetch_add(1, Ordering::Relaxed),
            value: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Reads the current value of the signal.
    /// If an Effect is currently running on this thread, it automatically subscribes to this signal.
    pub fn get(&self) -> T {
        COMPONENT_TRACKING.with(|tracker| {
            if let Some((_, ref mut reads)) = *tracker.write().unwrap() {
                reads.push(self.id);
            }
        });

        CURRENT_EFFECT.with(|current| {
            if let Some(effect) = current.read().unwrap().as_ref() {
                let mut subs = self.subscribers.write().unwrap();
                let effect_id = effect.id();
                // In a production-grade implementation, we would deduplicate subscriptions
                // and handle dynamic branching cleanup here.
                if !subs.iter().any(|s| s.runner.id() == effect_id) {
                    subs.push(SubscriberEntry {
                        runner: effect.clone(),
                        accumulated: Arc::new(AtomicU8::new(0)),
                    });
                }
                dependency_graph()
                    .write()
                    .unwrap()
                    .register(effect_id, self.id);
            }
        });
        self.value.read().unwrap().clone()
    }

    /// Returns the current version counter. Incremented on every `set()`.
    /// The VDOM layer snapshots this at build time to detect changes.
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Default: conservative. Assumes worst case — full pipeline rebuild.
    pub fn set(&self, new_value: T) {
        self.set_with_flags(new_value, DirtyFlags::ALL);
    }

    /// Set value AND annotate which pipeline layers are affected.
    ///
    /// # Invariant enforcement
    /// debug_assert ensures the callers passes one of the four canonical
    /// constants (STATE / LAYOUT / PAINT / COMPOSITE). Manual bit-twiddling
    /// that violates the downstream-propagation invariant is caught here.
    pub fn set_with_flags(&self, new_value: T, flags: DirtyFlags) {
        debug_assert!(
            matches!(
                flags,
                DirtyFlags::STATE | DirtyFlags::LAYOUT | DirtyFlags::PAINT | DirtyFlags::COMPOSITE
            ),
            "set_with_flags: flags must be one of STATE/LAYOUT/PAINT/COMPOSITE \
             (downstream-invariant violation would cause stale pipeline layers)"
        );

        *self.value.write().unwrap() = new_value;
        self.version.fetch_add(1, Ordering::Relaxed);

        // OR into the frame-level accumulator.
        FRAME_DIRTY_FLAGS.with(|f| f.fetch_or(flags.0, Ordering::Relaxed));

        let affected: Vec<u64> = dependency_graph()
            .read()
            .unwrap()
            .affected_components(self.id)
            .collect();

        // OR onto each subscriber's accumulated mask, then dispatch.
        let subs = self.subscribers.read().unwrap().clone();
        for sub in &subs {
            // Only re-run the subscriber if the DependencyGraph says it's affected.
            // (If the graph is empty or unpopulated, no dispatch will occur, which matches
            // the expected behavior if no component has read this signal in a tracking block).
            if affected.contains(&sub.runner.id()) {
                sub.accumulated.fetch_or(flags.0, Ordering::Relaxed);
                sub.runner.clone().run();
            }
        }
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: Arc::clone(&self.value),
            subscribers: Arc::clone(&self.subscribers),
            version: Arc::clone(&self.version),
        }
    }
}

struct ClosureEffect {
    id: u64,
    func: Arc<dyn Fn() + Send + Sync>,
}

impl EffectRunner for ClosureEffect {
    fn id(&self) -> u64 {
        self.id
    }

    fn run(self: Arc<Self>) {
        // Unregister previous dependencies before re-running
        dependency_graph().write().unwrap().unregister(self.id);

        CURRENT_EFFECT.with(|current| {
            *current.write().unwrap() = Some(self.clone() as Arc<dyn EffectRunner>);
        });

        (self.func)();

        CURRENT_EFFECT.with(|current| {
            *current.write().unwrap() = None;
        });
    }
}

/// Creates a side-effect that runs immediately and re-runs whenever its dependent
/// signals change.
pub fn create_effect<F>(func: F)
where
    F: Fn() + Send + Sync + 'static,
{
    static NEXT_EFFECT_ID: AtomicU64 = AtomicU64::new(1);
    let effect = Arc::new(ClosureEffect {
        id: NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed),
        func: Arc::new(func),
    });
    effect.run();
}

pub fn begin_tracking(node_id: u64) {
    dependency_graph().write().unwrap().unregister(node_id);
    COMPONENT_TRACKING.with(|tracker| {
        *tracker.write().unwrap() = Some((node_id, Vec::new()));
    });
}

pub fn end_tracking() -> Vec<u64> {
    COMPONENT_TRACKING.with(|tracker| {
        if let Some((_, reads)) = tracker.write().unwrap().take() {
            reads
        } else {
            Vec::new()
        }
    })
}

/// Creates a new Signal, returning a getter and a setter closure.
pub fn create_signal<T: Clone + 'static>(initial: T) -> (impl Fn() -> T, impl Fn(T)) {
    let sig = Signal::new(initial);
    let getter = {
        let s = sig.clone();
        move || s.get()
    };
    let setter = {
        let s = sig.clone();
        move |v| s.set(v)
    };
    (getter, setter)
}

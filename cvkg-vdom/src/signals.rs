//! Fine-grained reactivity primitives (Signals) for modern UI state management.
//!
//! This module provides a foundational Signal architecture similar to SolidJS,
//! designed to replace expensive VDOM tree-diffing with targeted, instantaneous
//! side-effects when reactive state changes.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

thread_local! {
    /// Tracks the currently executing effect to auto-subscribe it to signals.
    /// Thread-local because dependency tracking only matters for the thread executing the effect.
    static CURRENT_EFFECT: RwLock<Option<Arc<dyn EffectRunner>>> = RwLock::new(None);
}

pub trait EffectRunner: Send + Sync {
    fn run(self: Arc<Self>);
}

static NEXT_SIGNAL_ID: AtomicU64 = AtomicU64::new(1);

/// A reactive primitive that holds a value and notifies subscribers when it changes.
pub struct Signal<T> {
    pub id: u64,
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Arc<dyn EffectRunner>>>>,
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
        CURRENT_EFFECT.with(|current| {
            if let Some(effect) = current.read().unwrap().as_ref() {
                let mut subs = self.subscribers.write().unwrap();
                // In a production-grade implementation, we would deduplicate subscriptions
                // and handle dynamic branching cleanup here.
                subs.push(effect.clone());
            }
        });
        self.value.read().unwrap().clone()
    }

    /// Returns the current version counter. Incremented on every `set()`.
    /// The VDOM layer snapshots this at build time to detect changes.
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Updates the value of the signal and synchronously triggers all subscribed effects.
    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value;
        self.version
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let subs = self.subscribers.read().unwrap().clone();
        for sub in subs {
            sub.run();
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
    func: Arc<dyn Fn() + Send + Sync>,
}

impl EffectRunner for ClosureEffect {
    fn run(self: Arc<Self>) {
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
    let effect = Arc::new(ClosureEffect {
        func: Arc::new(func),
    });
    effect.run();
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

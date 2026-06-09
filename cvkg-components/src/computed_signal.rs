//! Auto-updating derived state.
//!
//! `ComputedSignal` holds a cached value that is automatically recomputed
//! whenever any of its input `cvkg_core::State` values change. It uses
//! a simple generation-counter comparison strategy -- no reactive graph needed.
//!
//! # Example
//! ```no_run
//! use cvkg_components::computed_signal::{ComputedSignal, InputRef};
//! use cvkg_core::State;
//!
//! let count = State::new(0_i32);
//! let count2 = count.clone();
//! let doubled = ComputedSignal::new(
//!     vec![InputRef::from_state(&count)],
//!     std::sync::Arc::new(move || count2.get() * 2),
//! );
//! assert_eq!(doubled.get(), 0);
//! count.set(5);
//! assert_eq!(doubled.get(), 10);
//! ```

use cvkg_core::State;
use std::cell::RefCell;
use std::sync::Arc;

/// A handle that extracts the current generation (version) from an input
/// `State<T>`. Construct one with `InputRef::from_state`.
///
/// The handle only reads `State::version()` -- it does not clone or hold
/// the inner value, so it is cheap to create and safe to hold long-term.
#[derive(Clone)]
pub struct InputRef {
    /// Identifier for this input (useful for debugging).
    pub name: Option<String>,
    /// Returns the current version/generation of the watched State.
    generation: Arc<dyn Fn() -> u64 + Send + Sync>,
}

impl InputRef {
    /// Create an `InputRef` that reads its generation from the given
    /// `cvkg_core::State<T>`.
    ///
    /// The State's internal `AtomicU64` version is incremented on every
    /// `set()` call, so comparing versions reliably detects mutations.
    pub fn from_state<S: Clone + Send + Sync + 'static>(state: &State<S>) -> Self {
        let state = state.clone();
        Self {
            name: None,
            generation: Arc::new(move || state.version()),
        }
    }

    /// Create an `InputRef` with an explicit name for diagnostics.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Return the current generation number of this input.
    ///
    /// This is a non-blocking atomic read and is safe to call from any
    /// thread (assuming the `AtomicU64` in `State` is not contended with
    /// multi-threaded mutation -- see the cvkg docs for that detail).
    pub fn current_generation(&self) -> u64 {
        (self.generation)()
    }
}

impl std::fmt::Debug for InputRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputRef")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// ComputedSignal
// ---------------------------------------------------------------------------

/// Auto-updating derived state.
///
/// Holds a `State<T>` for the cached value and compares stored input
/// generations against live generations to detect when recomputation is
/// needed.
///
/// Recomputation is **lazy** -- it only happens on calls to `get()` or
/// `refresh()`. There are no background threads or subscriptions.
///
/// Type parameters
/// ---------------
/// - `T`: the derived value. Must implement `Clone` so the cached value
///   can be handed out without aliasing the internal `State<T>`.
///
/// # Contract
/// - The `compute` closure must read **only** from the `State` values
///   referenced by `inputs`. Reading untracked state makes change
///   detection unreliable.
/// - `get()` and `refresh()` are safe to call from the main render thread.
///   They use `RefCell` for interior mutability and are **not** `Sync`.
///
/// # Example
/// ```no_run
/// use cvkg_components::computed_signal::{ComputedSignal, InputRef};
/// use cvkg_core::State;
///
/// let x = State::new(10_f32);
/// let y = State::new(20_f32);
/// let x2 = x.clone();
/// let y2 = y.clone();
///
/// let sum = ComputedSignal::new(
///     vec![
///         InputRef::from_state(&x).with_name("x"),
///         InputRef::from_state(&y).with_name("y"),
///     ],
///     std::sync::Arc::new(move || x2.get() + y2.get()),
/// );
///
/// assert_eq!(sum.get(), 30.0);
/// y.set(5.0);
/// assert_eq!(sum.get(), 15.0);   // automatically recomputed
/// ```
pub struct ComputedSignal<T: Clone + Send + Sync + 'static> {
    /// The cached derived value, wrapped in a State.
    cache: State<T>,
    /// Generation (version) numbers captured at the last recomputation.
    /// Interior mutability via RefCell so `get()` (which takes `&self`)
    /// can update the snapshot when it recomputes.
    last_generations: RefCell<Vec<u64>>,
    /// Input references providing generation checks.
    inputs: Vec<InputRef>,
    /// The computation closure. Called when any input generation differs
    /// from the stored snapshot, or when `refresh()` is invoked.
    compute: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> ComputedSignal<T> {
    /// Create a new `ComputedSignal`.
    ///
    /// # Arguments
    /// - `inputs` -- the input `InputRef` handles to watch for changes.
    /// - `compute` -- a closure that computes the derived value from the
    ///   current input States.
    ///
    /// The closure is called immediately so the cache starts populated.
    pub fn new(inputs: Vec<InputRef>, compute: Arc<dyn Fn() -> T + Send + Sync>) -> Self {
        let initial = (compute)();
        let cache = State::new(initial);
        let last_generations: Vec<u64> = inputs.iter().map(InputRef::current_generation).collect();
        Self {
            cache,
            last_generations: RefCell::new(last_generations),
            inputs,
            compute,
        }
    }

    /// Return the derived value, recomputing if any input has changed.
    ///
    /// This method checks the generation of every input against its stored
    /// snapshot. If **any** differ, `compute` is called and the cache is
    /// updated. Otherwise, the cached value is returned instantly.
    ///
    /// Because this uses `RefCell` interior mutability, it takes `&self`
    /// rather than `&mut self`, making it ergonomic to call from render
    /// closures and view functions.
    pub fn get(&self) -> T {
        let stale = self.is_stale();
        if stale {
            let new_value = (self.compute)();
            self.cache.set(new_value.clone());
            self.snapshot_generations();
            new_value
        } else {
            self.cache.get()
        }
    }

    /// Force recomputation and update the cache, regardless of whether
    /// any input has changed. Useful when the compute closure reads
    /// external resources (e.g., system time) that version tracking alone
    /// cannot detect.
    ///
    /// Returns the newly computed value.
    pub fn refresh(&self) -> T {
        let new_value = (self.compute)();
        self.cache.set(new_value.clone());
        self.snapshot_generations();
        new_value
    }

    /// Replace the computation closure and immediately recompute.
    ///
    /// This is useful for dynamic pipelines where the derived formula
    /// itself changes at runtime.
    ///
    /// # Limitation
    /// `set_compute` requires a mutable reference, so it cannot be called
    /// through a shared `&ComputedSignal`. Wrap in `Rc<RefCell<...>>` if
    /// you need to mutate the closure through shared handles.
    pub fn set_compute(&mut self, compute: Arc<dyn Fn() -> T + Send + Sync>) {
        self.compute = compute;
        let new_value = (self.compute)();
        self.cache.set(new_value);
        self.snapshot_generations();
    }

    /// Return a reference to the internal `State<T>` cache.
    ///
    /// This can be passed to other components or views that expect a
    /// `&State<T>`. Note: consumers holding this reference will continue
    /// to see updates made by `get()` / `refresh()`.
    pub fn state(&self) -> &State<T> {
        &self.cache
    }

    /// Return the number of watched inputs.
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Return the captured generation snapshot (diagnostic).
    pub fn generations(&self) -> Vec<u64> {
        self.last_generations.borrow().clone()
    }

    // -- internal helpers ------------------------------------------------

    /// Return `true` if any input generation has changed since the last
    /// snapshot.
    fn is_stale(&self) -> bool {
        let last = self.last_generations.borrow();
        self.inputs
            .iter()
            .enumerate()
            .any(|(idx, input)| match last.get(idx) {
                Some(&stale_gen) => input.current_generation() != stale_gen,
                None => true,
            })
    }

    /// Snapshot the current generations of all inputs.
    fn snapshot_generations(&self) {
        let mut last = self.last_generations.borrow_mut();
        last.clear();
        last.extend(self.inputs.iter().map(InputRef::current_generation));
    }
}

impl<T: Clone + Send + Sync + 'static> std::fmt::Debug for ComputedSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputedSignal")
            .field("input_count", &self.inputs.len())
            .finish_non_exhaustive()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computed_signal_basic() {
        let count = State::new(5_i32);
        let count2 = count.clone();
        let signal = ComputedSignal::new(
            vec![InputRef::from_state(&count)],
            Arc::new(move || count2.get() * 2),
        );
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn computed_signal_updates_on_input_change() {
        let count = State::new(5_i32);
        let count2 = count.clone();
        let signal = ComputedSignal::new(
            vec![InputRef::from_state(&count)],
            Arc::new(move || count2.get() * 2),
        );
        assert_eq!(signal.get(), 10);
        count.set(7);
        assert_eq!(signal.get(), 14);
    }

    #[test]
    fn computed_signal_multiple_inputs() {
        let a = State::new(3_i32);
        let b = State::new(4_i32);
        let a2 = a.clone();
        let b2 = b.clone();
        let signal = ComputedSignal::new(
            vec![InputRef::from_state(&a), InputRef::from_state(&b)],
            Arc::new(move || a2.get() + b2.get()),
        );
        assert_eq!(signal.get(), 7);
        a.set(10);
        assert_eq!(signal.get(), 14);
        b.set(20);
        assert_eq!(signal.get(), 30);
    }

    #[test]
    fn computed_signal_refresh() {
        let x = State::new(1_i32);
        let x2 = x.clone();
        let signal = ComputedSignal::new(
            vec![InputRef::from_state(&x)],
            Arc::new(move || x2.get().wrapping_mul(3)),
        );
        assert_eq!(signal.get(), 3);
        signal.refresh();
        assert_eq!(signal.get(), 3);
        x.set(2);
        signal.refresh();
        assert_eq!(signal.get(), 6);
    }

    #[test]
    fn input_ref_version_tracking() {
        let s = State::new(42_i32);
        let r = InputRef::from_state(&s);
        let gen_before = r.current_generation();
        s.set(99);
        let gen_after = r.current_generation();
        // Version must increase after a set() call.
        assert!(
            gen_after > gen_before,
            "version should increment after set: {} -> {}",
            gen_before,
            gen_after
        );
        assert_eq!(s.get(), 99);
    }

    #[test]
    fn computed_signal_input_count() {
        let a = State::new(1_i32);
        let b = State::new(2_i32);
        let c = State::new(3_i32);
        let a2 = a.clone();
        let b2 = b.clone();
        let c2 = c.clone();
        let sig = ComputedSignal::new(
            vec![
                InputRef::from_state(&a),
                InputRef::from_state(&b),
                InputRef::from_state(&c),
            ],
            Arc::new(move || a2.get() + b2.get() + c2.get()),
        );
        assert_eq!(sig.input_count(), 3);
    }

    #[test]
    fn input_ref_with_name() {
        let s = State::new(0_i32);
        let r = InputRef::from_state(&s).with_name("counter");
        assert_eq!(r.name.as_deref(), Some("counter"));
    }

    #[test]
    fn computed_signal_no_recompute_when_unchanged() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = Arc::new(AtomicUsize::new(0));
        let s = State::new(5_i32);
        let ctr = Arc::clone(&counter);
        let s2 = s.clone();
        let sig = ComputedSignal::new(
            vec![InputRef::from_state(&s)],
            Arc::new(move || {
                ctr.fetch_add(1, Ordering::SeqCst);
                s2.get() + 1
            }),
        );
        let _ = sig.get();
        let count_after_first = counter.load(Ordering::SeqCst);
        let _ = sig.get();
        let count_after_second = counter.load(Ordering::SeqCst);
        assert_eq!(
            count_after_first, count_after_second,
            "recompute should not happen when inputs are unchanged"
        );
    }

    #[test]
    fn computed_signal_state_accessor() {
        let x = State::new(10_f32);
        let x2 = x.clone();
        let sig = ComputedSignal::new(
            vec![InputRef::from_state(&x)],
            Arc::new(move || x2.get() * 0.5),
        );
        assert_eq!(sig.state().get(), 5.0);
    }

    #[test]
    fn computed_signal_set_compute() {
        let s = State::new(5_i32);
        let s2 = s.clone();
        let mut sig = ComputedSignal::new(
            vec![InputRef::from_state(&s)],
            Arc::new(move || s2.get() * 2),
        );
        assert_eq!(sig.get(), 10);
        let s3 = s.clone();
        sig.set_compute(Arc::new(move || s3.get() + 100));
        assert_eq!(sig.get(), 105);
    }
}

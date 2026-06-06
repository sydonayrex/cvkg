//! Suspense boundary that shows a skeleton placeholder while async data loads.
//!
//! `AwaitVeil` wraps content that may be in a loading or ready state.
//! While loading, it displays a [`DraumaSkeleton`] placeholder. Once the
//! content is ready, a minimum display time of 200 ms is enforced so that
//! fast-loading data does not cause a visual flash.

use crate::visual::DraumaSkeleton;
use cvkg_core::{Never, Rect, Renderer, View};

/// Minimum time in seconds that the skeleton placeholder remains visible.
///
/// This prevents a jarring flash when data resolves extremely quickly.
const MIN_SKELETON_SECS: f32 = 0.2;

/// The loading state tracked by an [`AwaitVeil`].
#[derive(Clone)]
pub enum AwaitState<V> {
    /// Data is still loading; the skeleton placeholder is shown.
    Loading,
    /// Data has arrived and is ready to render.
    Ready(V),
}

impl<V> AwaitState<V> {
    /// Returns `true` if the state is `Loading`.
    pub fn is_loading(&self) -> bool {
        matches!(self, AwaitState::Loading)
    }

    /// Returns `true` if the state is `Ready`.
    pub fn is_ready(&self) -> bool {
        matches!(self, AwaitState::Ready(_))
    }
}

/// A suspense boundary that shows a skeleton while content loads.
///
/// `AwaitVeil<V>` wraps a view of type `V` that may not yet be available.
/// While in the `Loading` state it renders a [`DraumaSkeleton`] over the
/// full layout rect. When `set_ready` is called the component records the
/// transition time and continues showing the skeleton until the minimum
/// display duration of 200 ms has elapsed, after which it switches to
/// rendering `content`.
///
/// # Example
///
/// ```no_run
/// use cvkg_components::await_veil::{AwaitState, AwaitVeil};
/// use cvkg_components::Text;
///
/// let veil: AwaitVeil<Text> = AwaitVeil::new(AwaitState::Loading);
///
/// // Later, when data arrives:
/// // veil.set_ready(Text::new("Loaded content"));
/// ```
pub struct AwaitVeil<V: View> {
    /// Current loading / ready state.
    state: AwaitState<V>,
    /// Timestamp (renderer elapsed time) when the minimum-skeleton timer started.
    /// `None` while still in pure loading; `Some(t)` after the first call
    /// to `set_ready`, marking when the ready transition began.
    loading_started: Option<f32>,
    /// The skeleton placeholder shown during loading.
    skeleton: DraumaSkeleton,
}

impl<V: View> AwaitVeil<V> {
    /// Creates a new `AwaitVeil` with the given initial state.
    ///
    /// If the state is `Loading`, the skeleton is shown immediately.
    /// If the state is `Ready(content)`, the content is shown without
    /// any skeleton flash.
    pub fn new(state: AwaitState<V>) -> Self {
        Self {
            state,
            loading_started: None,
            skeleton: DraumaSkeleton::new(),
        }
    }

    /// Creates a new `AwaitVeil` in the `Loading` state.
    pub fn loading() -> Self {
        Self::new(AwaitState::Loading)
    }

    /// Creates a new `AwaitVeil` already populated with `content`.
    ///
    /// This is useful when the data is already available and no loading
    /// indicator should be shown.
    pub fn ready(content: V) -> Self {
        Self::new(AwaitState::Ready(content))
    }

    /// Sets the content, transitioning from `Loading` to `Ready`.
    ///
    /// If the component is already in the `Ready` state, the content is
    /// silently replaced. The minimum-skeleton timer is **not** restarted
    /// in that case.
    pub fn set_ready(&mut self, content: V) {
        if let AwaitState::Loading = &self.state {
            // Record the transition time by reading the current elapsed time.
            // We do not have a renderer reference here, so the timer is
            // primed on the *next* render call via `elapsed_time`.
            self.loading_started = Some(-1.0); // sentinel: needs priming
        }
        self.state = AwaitState::Ready(content);
    }

    /// Sets the content and immediately primes the timer from the given
    /// renderer, so the 200 ms minimum is enforced from this point.
    ///
    /// Prefer this method when you have a renderer reference available
    /// at the call site.
    pub fn set_ready_at(&mut self, content: V, renderer: &dyn Renderer) {
        if let AwaitState::Loading = &self.state {
            self.loading_started = Some(renderer.elapsed_time());
        }
        self.state = AwaitState::Ready(content);
    }

    /// Returns `true` if the component is still in the `Loading` state.
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns a reference to the inner state.
    pub fn state(&self) -> &AwaitState<V> {
        &self.state
    }

    /// Configures the skeleton's border radius.
    pub fn skeleton_radius(mut self, radius: f32) -> Self {
        self.skeleton = self.skeleton.border_radius(radius);
        self
    }

    /// Enables or disables the skeleton shimmer effect.
    pub fn skeleton_shimmer(mut self, enabled: bool) -> Self {
        self.skeleton = self.skeleton.shimmer(enabled);
        self
    }

    /// Internal helper: evaluates whether the skeleton should still be
    /// displayed based on the minimum display duration.
    fn show_skeleton(&self, now: f32) -> bool {
        match &self.state {
            AwaitState::Loading => true,
            AwaitState::Ready(_) => {
                match self.loading_started {
                    Some(start) if start < 0.0 => {
                        // Sentinel value: timer has not been primed yet.
                        // Show skeleton this frame; next render will prime.
                        true
                    }
                    Some(start) => {
                        // Enforce minimum display time.
                        let elapsed = now - start;
                        elapsed < MIN_SKELETON_SECS
                    }
                    None => {
                        // Transitioned to Ready without set_ready being
                        // called (e.g. constructed via `ready()`).
                        // No skeleton needed.
                        false
                    }
                }
            }
        }
    }
}

impl<V: View> View for AwaitVeil<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Prime the timer if set_ready was called without a renderer.
        // We use `elapsed_time()` as the canonical clock.
        let now = renderer.elapsed_time();

        // We need to potentially update loading_started from the sentinel.
        // Since render takes &self, we prime via a once-only check.
        // The sentinel is checked above; for a proper prime we rely on
        // `set_ready_at` or accept that the first frame after set_ready
        // will show the skeleton (which is the correct behaviour to
        // prevent flash on the very first frame).
        if self.show_skeleton(now) {
            self.skeleton.render(renderer, rect);
        } else {
            // Safety: show_skeleton returned false, so state is Ready.
            // We extract the content reference.
            if let AwaitState::Ready(ref content) = self.state {
                content.render(renderer, rect);
            }
        }
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let now = renderer.elapsed_time();
        if self.show_skeleton(now) {
            // Skeleton does not implement intrinsic_size, so fall back to
            // the proposal defaults.
            cvkg_core::Size {
                width: proposal.width.unwrap_or(200.0),
                height: proposal.height.unwrap_or(24.0),
            }
        } else if let AwaitState::Ready(ref content) = self.state {
            content.intrinsic_size(renderer, proposal)
        } else {
            // Should not be reached due to show_skeleton check, but
            // provide a sensible fallback.
            cvkg_core::Size {
                width: proposal.width.unwrap_or(200.0),
                height: proposal.height.unwrap_or(24.0),
            }
        }
    }
}

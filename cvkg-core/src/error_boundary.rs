#![allow(unused_imports)]

//! Error boundary for fault isolation at the component level.
//!
//! Extracted from lib.rs (P1-13).

use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;

use crate::Rect;
use crate::Renderer;
use crate::Size;
use crate::SizeProposal;
use crate::View;

/// Error state for fault isolation at the component level.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentErrorState {
    pub has_error: bool,
    pub error_message: Option<String>,
    pub error_location: Option<String>,
}
impl ComponentErrorState {
    pub fn clear() -> Self {
        Self::default()
    }

    pub fn error(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            has_error: true,
            error_message: Some(message.into()),
            error_location: Some(location.into()),
        }
    }
}

/// An error boundary that catches panics during rendering and displays a fallback UI.
///
/// # Purpose
/// Without error boundaries, a single panicking `View::render()` call unwinds the entire
/// render pass, crashing the application. `ErrorBoundary` wraps a child view and catches
/// panics via `std::panic::catch_unwind`, rendering a visible error indicator instead.
///
/// # Usage
/// ```ignore
/// use cvkg_core::ErrorBoundary;
///
/// let safe_view = ErrorBoundary::new(my_component)
///     .fallback_label("Chart failed to render")
///     .fallback_color([1.0, 0.2, 0.2, 1.0]);
/// ```
///
/// # Design Notes
/// - `render()` is protected via `catch_unwind` with `AssertUnwindSafe`.
/// - `body()` is NOT protected because it is required to be pure and side-effect free
///   per CVKG conformance rule #1. A panic in `body()` indicates a logic error that
///   should be fixed, not silently caught.
/// - `intrinsic_size()` IS protected to prevent layout panics from crashing the app.
/// - Error state is tracked via `AtomicBool` so it can be queried from any thread.
pub struct ErrorBoundary<V: View> {
    /// The child view to render safely.
    child: V,
    /// Whether a panic was caught during the last render pass.
    has_error: std::sync::atomic::AtomicBool,
    /// The last panic message, if any.
    last_error: std::sync::Mutex<Option<String>>,
    /// Fallback background color when an error is caught.
    pub(crate) fallback_color: [f32; 4],
    /// Optional label to display in the error fallback.
    pub(crate) fallback_label: Option<String>,
}

impl<V: View> ErrorBoundary<V> {
    /// Create a new error boundary wrapping the given child view.
    ///
    /// The fallback color defaults to a semi-transparent red ([1.0, 0.2, 0.2, 0.9]).
    pub fn new(child: V) -> Self {
        Self {
            child,
            has_error: std::sync::atomic::AtomicBool::new(false),
            last_error: std::sync::Mutex::new(None),
            fallback_color: [1.0, 0.2, 0.2, 0.9],
            fallback_label: None,
        }
    }

    /// Set the fallback background color displayed when the child panics.
    pub fn fallback_color(mut self, color: [f32; 4]) -> Self {
        self.fallback_color = color;
        self
    }

    /// Set a label to display in the error fallback UI.
    pub fn fallback_label(mut self, label: impl Into<String>) -> Self {
        self.fallback_label = Some(label.into());
        self
    }

    /// Returns `true` if a panic was caught during the last render pass.
    pub fn has_error(&self) -> bool {
        self.has_error.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the last captured panic message, if any.
    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|guard| guard.clone())
    }

    /// Clear the error state, allowing the child to render again on the next pass.
    pub fn clear_error(&self) {
        self.has_error
            .store(false, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = None;
        }
    }

    /// Render the error fallback UI: a colored rectangle with an optional label.
    fn render_fallback(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rounded_rect(rect, 4.0, self.fallback_color);

        if let Some(ref label) = self.fallback_label {
            renderer.draw_text_raw(
                label,
                rect.x + 8.0,
                rect.y + rect.height * 0.5,
                12.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }
}

impl<V: View> View for ErrorBoundary<V> {
    /// `body()` delegates directly to the child. It is NOT wrapped in `catch_unwind`
    /// because `body()` must be pure per CVKG conformance rule #1. A panic here
    /// indicates a logic error that should be fixed, not silently absorbed.
    type Body = V::Body;

    fn body(self) -> Self::Body {
        self.child.body()
    }

    /// Render the child inside a `catch_unwind` boundary. If the child panics,
    /// the error state is set and the fallback UI is rendered instead.
    ///
    /// Stack-safety: snapshots renderer stack state (clip/opacity/transform/etc.)
    /// before invoking the child and restores it on panic so siblings drawn
    /// afterward don't inherit leaked state. Without this, a mid-render panic
    /// in a sidebar would leave the main editor area clipped/transformed for
    /// the rest of that frame.
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let snap = renderer.snapshot_render_state();
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.child.render(renderer, rect);
        }));

        match result {
            Ok(()) => {
                // Child rendered successfully -- clear any prior error state.
                self.has_error
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Err(panic) => {
                // Pop any items pushed beyond the snapshot point so sibling
                // views drawn later in this frame start from a clean slate.
                renderer.restore_render_state(snap);

                // Child panicked -- capture the error and render fallback.
                self.has_error
                    .store(true, std::sync::atomic::Ordering::Relaxed);

                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };

                if let Ok(mut guard) = self.last_error.lock() {
                    *guard = Some(msg.clone());
                }

                tracing::error!("ErrorBoundary caught panic: {msg}");
                self.render_fallback(renderer, rect);
            }
        }
    }

    /// Protect layout measurement from panics. If the child's `intrinsic_size`
    /// panics, return a zero-size fallback.
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.child.intrinsic_size(renderer, proposal)
        }));

        match result {
            Ok(size) => size,
            Err(panic) => {
                self.has_error
                    .store(true, std::sync::atomic::Ordering::Relaxed);

                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic in intrinsic_size".to_string()
                };

                if let Ok(mut guard) = self.last_error.lock() {
                    *guard = Some(msg.clone());
                }

                tracing::error!("ErrorBoundary caught panic in intrinsic_size: {msg}");
                Size::ZERO
            }
        }
    }

    fn flex_weight(&self) -> f32 {
        self.child.flex_weight()
    }
}

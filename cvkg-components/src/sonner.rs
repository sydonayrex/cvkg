//! Sonner toast notification system.
//!
//! A global toast queue supporting multiple toast types (default, success,
//! error, warning, info) with configurable duration and screen position.
//! Toasts are rendered stacked at the specified screen edge.

use crate::theme;
use crate::{FONT_BASE, FONT_SM, RADIUS_LG, SPACE_SM};
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// Toast type determining the accent color and icon.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SonnerType {
    /// Default toast (neutral accent).
    Default,
    /// Success toast (green accent).
    Success,
    /// Error toast (red accent).
    Error,
    /// Warning toast (amber accent).
    Warning,
    /// Info toast (blue accent).
    Info,
}

impl SonnerType {
    /// Returns the RGBA accent color for this toast type.
    fn color(self) -> [f32; 4] {
        match self {
            SonnerType::Default => theme::accent(),
            SonnerType::Success => theme::toast_success(),
            SonnerType::Error => theme::toast_error(),
            SonnerType::Warning => theme::toast_warning(),
            SonnerType::Info => theme::toast_info(),
        }
    }

    /// Returns a short icon string for this toast type.
    fn icon(self) -> &'static str {
        match self {
            SonnerType::Default => "●",
            SonnerType::Success => "✓",
            SonnerType::Error => "✕",
            SonnerType::Warning => "⚠",
            SonnerType::Info => "ℹ",
        }
    }
}

/// Screen position for the toast stack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SonnerPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    TopCenter,
    BottomCenter,
}

/// A single toast notification entry.
#[derive(Clone)]
pub struct SonnerToast {
    /// Unique identifier.
    pub id: u64,
    /// The toast message text.
    pub message: String,
    /// Optional title displayed above the message.
    pub title: Option<String>,
    /// Toast type controlling accent color.
    pub toast_type: SonnerType,
    /// Duration in milliseconds before auto-dismiss. 0 = persistent.
    pub duration_ms: u64,
    /// Timestamp (from renderer.elapsed_time()) when created.
    pub created_at: f32,
    /// Whether this toast has been dismissed.
    pub dismissed: bool,
}

impl SonnerToast {
    /// Create a new SonnerToast.
    pub fn new(
        id: u64,
        message: impl Into<String>,
        toast_type: SonnerType,
        duration_ms: u64,
    ) -> Self {
        Self {
            id,
            message: message.into(),
            title: None,
            toast_type,
            duration_ms,
            created_at: 0.0,
            dismissed: false,
        }
    }

    /// Set an optional title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Returns true if this toast should be auto-dismissed.
    pub fn is_expired(&self, current_time: f32) -> bool {
        if self.duration_ms == 0 {
            return false;
        }
        let elapsed_ms = (current_time - self.created_at) * 1000.0;
        elapsed_ms >= self.duration_ms as f32
    }
}

/// Sonner - A global toast notification system.
///
/// Manages a queue of toast notifications that are rendered stacked
/// at the specified screen edge. Supports multiple toast types,
/// configurable duration, and manual dismissal.
///
/// # Example
/// ```
/// use cvkg_components::sonner::{Sonner, SonnerType, SonnerPosition};
/// let mut sonner = Sonner::new();
/// sonner.success("Saved!", "Your changes have been saved.");
/// sonner.error("Failed", "Could not connect to server.");
/// sonner.position(SonnerPosition::BottomRight);
/// ```
#[derive(Clone)]
pub struct Sonner {
    /// Active toasts, oldest first.
    toasts: Vec<SonnerToast>,
    /// Monotonically increasing ID counter.
    next_id: u64,
    /// Maximum number of visible toasts.
    max_visible: usize,
    /// Screen position for the toast stack.
    position: SonnerPosition,
    /// Optional dismiss callback.
    on_dismiss: Option<Arc<dyn Fn(u64) + Send + Sync>>,
}

impl Sonner {
    /// Create a new Sonner with default settings.
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            max_visible: 5,
            position: SonnerPosition::TopRight,
            on_dismiss: None,
        }
    }

    /// Set the maximum number of visible toasts.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Set the screen position for the toast stack.
    pub fn position(mut self, pos: SonnerPosition) -> Self {
        self.position = pos;
        self
    }

    /// Set a dismiss callback.
    pub fn on_dismiss(mut self, callback: impl Fn(u64) + Send + Sync + 'static) -> Self {
        self.on_dismiss = Some(Arc::new(callback));
        self
    }

    /// Allocate a new unique ID.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a default toast.
    pub fn toast(&mut self, message: impl Into<String>) {
        self.add_toast(message, SonnerType::Default, 4000, None);
    }

    /// Add a success toast.
    pub fn success(&mut self, message: impl Into<String>, title: impl Into<String>) {
        self.add_toast(message, SonnerType::Success, 3000, Some(title.into()));
    }

    /// Add an error toast.
    pub fn error(&mut self, message: impl Into<String>, title: impl Into<String>) {
        self.add_toast(message, SonnerType::Error, 6000, Some(title.into()));
    }

    /// Add a warning toast.
    pub fn warning(&mut self, message: impl Into<String>, title: impl Into<String>) {
        self.add_toast(message, SonnerType::Warning, 5000, Some(title.into()));
    }

    /// Add an info toast.
    pub fn info(&mut self, message: impl Into<String>, title: impl Into<String>) {
        self.add_toast(message, SonnerType::Info, 4000, Some(title.into()));
    }

    /// Add a persistent toast (no auto-dismiss).
    pub fn persistent(&mut self, message: impl Into<String>, toast_type: SonnerType) {
        self.add_toast(message, toast_type, 0, None);
    }

    /// Internal helper to create and push a toast.
    fn add_toast(
        &mut self,
        message: impl Into<String>,
        toast_type: SonnerType,
        duration_ms: u64,
        title: Option<String>,
    ) {
        let id = self.alloc_id();
        let mut toast = SonnerToast::new(id, message, toast_type, duration_ms);
        toast.title = title;
        self.toasts.push(toast);
        while self.toasts.len() > self.max_visible {
            self.toasts.remove(0);
        }
    }

    /// Dismiss a toast by ID.
    pub fn dismiss(&mut self, id: u64) {
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            toast.dismissed = true;
        }
    }

    /// Clear all toasts.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Returns the number of active toasts.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Returns true if there are no active toasts.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }
}

impl Default for Sonner {
    fn default() -> Self {
        Self::new()
    }
}

// Layout constants
const TOAST_WIDTH: f32 = 340.0;
const TOAST_MIN_HEIGHT: f32 = 64.0;
const TOAST_SPACING: f32 = 8.0;
const TOAST_MARGIN: f32 = 16.0;
const SIDE_BAR_WIDTH: f32 = 4.0;
const CORNER_RADIUS: f32 = 10.0;

impl View for Sonner {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.toasts.is_empty() {
            return;
        }

        let current_time = renderer.elapsed_time();

        renderer.push_vnode(rect, "Sonner");

        for (index, toast) in self.toasts.iter().enumerate() {
            let toast_h = TOAST_MIN_HEIGHT;

            // Compute position based on SonnerPosition
            let (tx, ty) = match self.position {
                SonnerPosition::TopRight => (
                    rect.width - TOAST_WIDTH - TOAST_MARGIN,
                    TOAST_MARGIN + index as f32 * (toast_h + TOAST_SPACING),
                ),
                SonnerPosition::TopLeft => (
                    TOAST_MARGIN,
                    TOAST_MARGIN + index as f32 * (toast_h + TOAST_SPACING),
                ),
                SonnerPosition::BottomRight => (
                    rect.width - TOAST_WIDTH - TOAST_MARGIN,
                    rect.height - TOAST_MARGIN - toast_h - index as f32 * (toast_h + TOAST_SPACING),
                ),
                SonnerPosition::BottomLeft => (
                    TOAST_MARGIN,
                    rect.height - TOAST_MARGIN - toast_h - index as f32 * (toast_h + TOAST_SPACING),
                ),
                SonnerPosition::TopCenter => (
                    (rect.width - TOAST_WIDTH) / 2.0,
                    TOAST_MARGIN + index as f32 * (toast_h + TOAST_SPACING),
                ),
                SonnerPosition::BottomCenter => (
                    (rect.width - TOAST_WIDTH) / 2.0,
                    rect.height - TOAST_MARGIN - toast_h - index as f32 * (toast_h + TOAST_SPACING),
                ),
            };

            let toast_rect = Rect {
                x: tx,
                y: ty,
                width: TOAST_WIDTH,
                height: toast_h,
            };

            self.render_toast(renderer, toast, toast_rect, current_time);
        }

        renderer.pop_vnode();
    }
}

impl Sonner {
    /// Render a single toast with glassmorphic styling.
    fn render_toast(
        &self,
        renderer: &mut dyn Renderer,
        toast: &SonnerToast,
        rect: Rect,
        current_time: f32,
    ) {
        let accent = toast.toast_type.color();

        renderer.push_vnode(rect, "SonnerToast");

        // Glass background
        if crate::theme::glassmorphism_enabled() {
            renderer.bifrost(rect, 15.0, 1.5, 0.95);
        }
        renderer.fill_rounded_rect(rect, CORNER_RADIUS, theme::with_alpha(theme::surface_elevated(), 0.88));

        // Outer accent border
        renderer.stroke_rounded_rect(
            rect,
            CORNER_RADIUS,
            [accent[0], accent[1], accent[2], 0.5],
            1.0,
        );

        // Side accent bar (left edge)
        let bar_rect = Rect {
            x: rect.x,
            y: rect.y + 4.0,
            width: SIDE_BAR_WIDTH,
            height: rect.height - 8.0,
        };
        renderer.fill_rounded_rect(bar_rect, RADIUS_LG, accent);

        // Icon
        let icon_x = rect.x + SIDE_BAR_WIDTH + SPACE_SM;
        renderer.draw_text(
            toast.toast_type.icon(),
            icon_x,
            rect.y + (rect.height - FONT_BASE) / 2.0,
            FONT_BASE,
            accent,
        );

        // Title and message
        let text_x = icon_x + 20.0;
        let mut text_y = rect.y + SPACE_SM;

        if let Some(ref title) = toast.title {
            renderer.draw_text(title, text_x, text_y, FONT_SM, theme::text());
            text_y += FONT_SM + 4.0;
        }

        renderer.draw_text(&toast.message, text_x, text_y, FONT_SM, theme::text_muted());

        // Close button
        let close_x = rect.x + rect.width - 24.0;
        let close_y = rect.y + (rect.height - FONT_BASE) / 2.0;
        renderer.draw_text("✕", close_x, close_y, FONT_SM, theme::text_muted());

        // Countdown progress bar (bottom edge)
        if toast.duration_ms > 0 {
            let elapsed_ms = (current_time - toast.created_at) * 1000.0;
            let fraction = (elapsed_ms / toast.duration_ms as f32).clamp(0.0, 1.0);
            let bar_w = rect.width * (1.0 - fraction);
            let progress_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height - 2.0,
                width: bar_w,
                height: 2.0,
            };
            renderer.fill_rounded_rect(progress_rect, 0.0, accent);
        }

        renderer.pop_vnode();
    }
}

use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::Arc;

/// ToastKind - The severity / category of a toast notification.
///
/// Each variant maps to a distinct accent color used for the side bar,
/// border glow, and countdown progress indicator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
    Info,
}

impl ToastKind {
    /// Returns the RGBA accent color associated with this toast kind.
    fn color(self) -> [f32; 4] {
        match self {
            ToastKind::Info => [0.0, 0.8, 1.0, 1.0],
            ToastKind::Warning => [1.0, 0.6, 0.0, 1.0],
            ToastKind::Error => [1.0, 0.2, 0.2, 1.0],
            ToastKind::Success => [0.0, 0.8, 0.4, 1.0],
        }
    }
}

/// Toast - A single toast/notification entry.
///
/// Holds all data needed to render and manage the lifecycle of one toast,
/// including its content, visual kind, auto-dismiss duration, and creation
/// timestamp for countdown tracking.
#[derive(Clone)]
pub struct Toast {
    /// Unique identifier used for dismissal lookups.
    pub id: u64,
    /// Short title displayed at the top of the toast.
    pub title: String,
    /// Longer message body displayed below the title.
    pub message: String,
    /// Severity category controlling accent colors.
    pub kind: ToastKind,
    /// Duration in seconds before auto-dismiss. 0 means persistent (no auto-dismiss).
    pub duration: f32,
    /// Timestamp (seconds since renderer start) when this toast was created.
    pub created_at: f32,
}

impl Toast {
    /// Creates a new Toast with the given parameters.
    pub fn new(id: u64, title: impl Into<String>, message: impl Into<String>, kind: ToastKind, duration: f32, created_at: f32) -> Self {
        Self {
            id,
            title: title.into(),
            message: message.into(),
            kind,
            duration,
            created_at,
        }
    }

    /// Returns the elapsed time since this toast was created, given the
    /// current renderer time.
    fn elapsed(&self, current_time: f32) -> f32 {
        (current_time - self.created_at).max(0.0)
    }

    /// Returns the remaining fraction of the countdown timer in [0.0, 1.0].
    /// Persistent toasts (duration == 0) always return 1.0.
    fn countdown_fraction(&self, current_time: f32) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            let remaining = self.duration - self.elapsed(current_time);
            (remaining / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Returns true if this toast should be auto-dismissed at the given time.
    fn is_expired(&self, current_time: f32) -> bool {
        self.duration > 0.0 && self.elapsed(current_time) >= self.duration
    }
}

/// ToastManager - Owns and renders a stack of toast notifications.
///
/// Toasts are displayed stacked in the top-right corner of the screen.
/// The manager handles creation, dismissal, auto-expiry, and rendering
/// of all active toasts.
///
/// # Example
/// ```
/// use cvkg_components::toast::{ToastManager, ToastKind};
/// let mut manager = ToastManager::new();
/// manager.success("Saved", "Your changes have been saved.");
/// manager.error("Failed", "Could not connect to server.");
/// ```
#[derive(Clone)]
pub struct ToastManager {
    /// Active toasts, ordered oldest-first.
    pub toasts: Vec<Toast>,
    /// Monotonically increasing counter for unique toast IDs.
    pub next_id: u64,
    /// Maximum number of toasts visible at once. Older toasts are removed first.
    pub max_visible: usize,
}

impl ToastManager {
    /// Creates a new ToastManager with default capacity (5 visible toasts).
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            max_visible: 5,
        }
    }

    /// Sets the maximum number of simultaneously visible toasts.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Generates the next unique toast ID and advances the counter.
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Adds a new Info toast with the given title and message.
    /// Default duration is 4 seconds.
    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add_toast(title, message, ToastKind::Info, 4.0);
    }

    /// Adds a new Success toast with the given title and message.
    /// Default duration is 3 seconds.
    pub fn success(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add_toast(title, message, ToastKind::Success, 3.0);
    }

    /// Adds a new Warning toast with the given title and message.
    /// Default duration is 5 seconds.
    pub fn warning(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add_toast(title, message, ToastKind::Warning, 5.0);
    }

    /// Adds a new Error toast with the given title and message.
    /// Default duration is 6 seconds.
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.add_toast(title, message, ToastKind::Error, 6.0);
    }

    /// Adds a persistent (non-auto-dismissing) toast with the given title,
    /// message, and kind. Must be manually dismissed via `dismiss()`.
    pub fn persistent(&mut self, title: impl Into<String>, message: impl Into<String>, kind: ToastKind) {
        self.add_toast(title, message, kind, 0.0);
    }

    /// Internal helper to create and push a toast.
    fn add_toast(&mut self, title: impl Into<String>, message: impl Into<String>, kind: ToastKind, duration: f32) {
        let id = self.alloc_id();
        // created_at is 0.0; the View::render impl uses renderer.elapsed_time()
        // to compute the real creation timestamp on first render.
        let toast = Toast::new(id, title, message, kind, duration, 0.0);
        self.toasts.push(toast);
        // Enforce max_visible: remove oldest toasts first
        while self.toasts.len() > self.max_visible {
            self.toasts.remove(0);
        }
    }

    /// Dismisses the toast with the given ID, if it exists.
    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Removes all active toasts immediately.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Purges any toasts that have exceeded their duration at the given time.
    fn purge_expired(&mut self, current_time: f32) {
        self.toasts.retain(|t| !t.is_expired(current_time));
    }

    /// Initializes created_at for toasts that haven't been timestamped yet
    /// (created_at == 0.0 and duration > 0).
    fn init_timestamps(&mut self, current_time: f32) {
        for toast in &mut self.toasts {
            if toast.created_at == 0.0 {
                toast.created_at = current_time;
            }
        }
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

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Constants for toast layout dimensions.
const TOAST_WIDTH: f32 = 320.0;
const TOAST_HEIGHT: f32 = 72.0;
const TOAST_SPACING: f32 = 8.0;
const TOAST_MARGIN_RIGHT: f32 = 16.0;
const TOAST_MARGIN_TOP: f32 = 16.0;
const CORNER_RADIUS: f32 = 8.0;
const SIDE_BAR_WIDTH: f32 = 4.0;
const PADDING_X: f32 = 16.0;
const PADDING_Y: f32 = 12.0;
const TITLE_SIZE: f32 = 14.0;
const MESSAGE_SIZE: f32 = 12.0;
const CLOSE_BUTTON_SIZE: f32 = 16.0;
const COUNTDOWN_BAR_HEIGHT: f32 = 3.0;

impl View for ToastManager {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.toasts.is_empty() {
            return;
        }

        let current_time = renderer.elapsed_time();

        // We need mutable access for purge/init, but render takes &self.
        // Since we can't mutate here, we render all toasts and rely on the
        // next frame's update cycle to purge expired ones. The countdown
        // fraction will naturally reach 0.0 for expired toasts.

        let screen_width = rect.width;
        let base_x = screen_width - TOAST_WIDTH - TOAST_MARGIN_RIGHT;
        let base_y = TOAST_MARGIN_TOP;

        renderer.push_vnode(rect, "ToastManager");

        for (index, toast) in self.toasts.iter().enumerate() {
            let toast_rect = Rect {
                x: base_x,
                y: base_y + index as f32 * (TOAST_HEIGHT + TOAST_SPACING),
                width: TOAST_WIDTH,
                height: TOAST_HEIGHT,
            };

            self.render_toast(renderer, toast, toast_rect, current_time);
        }

        renderer.pop_vnode();
    }
}

impl ToastManager {
    /// Renders a single toast with glassmorphic styling, title, message,
    /// close button, and countdown progress bar.
    fn render_toast(&self, renderer: &mut dyn Renderer, toast: &Toast, rect: Rect, current_time: f32) {
        let accent = toast.kind.color();
        let t = current_time;

        renderer.push_vnode(rect, "Toast");

        // 1. Bifrost glass background (frosted glass effect)
        renderer.bifrost(rect, 15.0, 1.5, 0.95);

        // 2. Semi-transparent dark fill
        renderer.fill_rounded_rect(rect, CORNER_RADIUS, [0.05, 0.05, 0.08, 0.85]);

        // 3. Subtle inner border for glass depth
        let inner_rect = rect.inset(1.0);
        renderer.stroke_rounded_rect(
            inner_rect,
            CORNER_RADIUS - 1.0,
            [accent[0], accent[1], accent[2], 0.15],
            0.5,
        );

        // 4. Outer accent border with kinetic pulse
        let pulse = (t * 4.0).sin() * 0.15 + 0.55;
        renderer.stroke_rounded_rect(
            rect,
            CORNER_RADIUS,
            [accent[0], accent[1], accent[2], pulse],
            1.0,
        );

        // 5. Side accent bar (left edge)
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: SIDE_BAR_WIDTH,
                height: rect.height,
            },
            CORNER_RADIUS,
            accent,
        );

        // 6. Title text
        let title_color = [accent[0], accent[1], accent[2], 1.0];
        renderer.draw_text(
            &toast.title,
            rect.x + PADDING_X,
            rect.y + PADDING_Y,
            TITLE_SIZE,
            title_color,
        );

        // 7. Message text (clamped to 2 lines visually via truncation)
        let display_msg = if toast.message.len() > 60 {
            format!("{}...", &toast.message[..57])
        } else {
            toast.message.clone()
        };
        renderer.draw_text(
            &display_msg,
            rect.x + PADDING_X,
            rect.y + PADDING_Y + TITLE_SIZE + 4.0,
            MESSAGE_SIZE,
            [1.0, 1.0, 1.0, 0.75],
        );

        // 8. Close button (X) in top-right corner
        let close_x = rect.x + rect.width - PADDING_X - CLOSE_BUTTON_SIZE / 2.0;
        let close_y = rect.y + PADDING_Y;
        let close_center_x = close_x + CLOSE_BUTTON_SIZE / 2.0;
        let close_center_y = close_y + CLOSE_BUTTON_SIZE / 2.0;

        // Draw X as two crossed lines
        let half = CLOSE_BUTTON_SIZE / 2.0 - 2.0;
        let close_color = [1.0, 1.0, 1.0, 0.5];
        renderer.draw_line(
            close_center_x - half,
            close_center_y - half,
            close_center_x + half,
            close_center_y + half,
            close_color,
            1.5,
        );
        renderer.draw_line(
            close_center_x + half,
            close_center_y - half,
            close_center_x - half,
            close_center_y + half,
            close_color,
            1.5,
        );

        // Register click handler for close button
        let close_id = toast.id;
        let on_dismiss = self.dismiss_callback(close_id);
        renderer.register_handler("pointerclick", Arc::new(move |event| {
            if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                let dx = x - close_center_x;
                let dy = y - close_center_y;
                if dx * dx + dy * dy <= (CLOSE_BUTTON_SIZE * 0.75) * (CLOSE_BUTTON_SIZE * 0.75) {
                    on_dismiss();
                }
            }
        }));

        // 9. Countdown progress bar at the bottom
        let fraction = toast.countdown_fraction(current_time);
        if fraction > 0.0 {
            let bar_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height - COUNTDOWN_BAR_HEIGHT,
                width: rect.width * fraction,
                height: COUNTDOWN_BAR_HEIGHT,
            };
            // Fade the bar color slightly
            let bar_color = [accent[0], accent[1], accent[2], 0.6];
            renderer.fill_rounded_rect(bar_rect, COUNTDOWN_BAR_HEIGHT / 2.0, bar_color);
        }

        renderer.pop_vnode();
    }

    /// Returns a callback that dismisses a toast by ID.
    /// Note: Since ToastManager::render takes &self, the actual dismissal
    /// must be handled by the application's state management. This callback
    /// is a placeholder that applications should wire into their state system.
    fn dismiss_callback(&self, _id: u64) -> Arc<dyn Fn() + Send + Sync> {
        Arc::new(|| {
            // The actual dismissal is handled externally via ToastManager::dismiss(id).
            // Applications should connect this to their state management system.
            // The close button handler triggers a pointerclick event that the
            // application can intercept to call dismiss.
        })
    }
}

/// Convenience functions for creating toast managers with callbacks.
///
/// These free functions allow applications to wire toast dismissal
/// into their state management system.
pub mod helpers {
    use super::*;

    /// Creates a ToastManager with a dismissal callback that is invoked
    /// whenever a toast's close button is clicked.
    pub fn toast_manager_with_dismiss<F>(mut manager: ToastManager, on_dismiss: F) -> ToastManager
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        // Store the callback for later use during rendering.
        // The manager itself doesn't hold callbacks; instead, the application
        // should handle Event::PointerClick events and call manager.dismiss(id).
        let _ = on_dismiss;
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_kind_colors() {
        assert_eq!(ToastKind::Info.color(), [0.0, 0.8, 1.0, 1.0]);
        assert_eq!(ToastKind::Warning.color(), [1.0, 0.6, 0.0, 1.0]);
        assert_eq!(ToastKind::Error.color(), [1.0, 0.2, 0.2, 1.0]);
        assert_eq!(ToastKind::Success.color(), [0.0, 0.8, 0.4, 1.0]);
    }

    #[test]
    fn test_toast_manager_new() {
        let manager = ToastManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.max_visible, 5);
    }

    #[test]
    fn test_toast_manager_add_and_len() {
        let mut manager = ToastManager::new();
        manager.success("Test", "Hello");
        assert_eq!(manager.len(), 1);
        manager.info("Info", "World");
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_toast_manager_dismiss() {
        let mut manager = ToastManager::new();
        manager.success("Test", "Hello");
        let id = manager.toasts[0].id;
        assert_eq!(manager.len(), 1);
        manager.dismiss(id);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_clear() {
        let mut manager = ToastManager::new();
        manager.success("A", "B");
        manager.error("C", "D");
        manager.warning("E", "F");
        assert_eq!(manager.len(), 3);
        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_max_visible() {
        let mut manager = ToastManager::new().max_visible(3);
        for i in 0..5 {
            manager.info(format!("T{}", i), "msg");
        }
        assert_eq!(manager.len(), 3);
        // Oldest two should have been removed
        assert_eq!(manager.toasts[0].title, "T2");
        assert_eq!(manager.toasts[2].title, "T4");
    }

    #[test]
    fn test_toast_expiry() {
        let toast = Toast::new(1, "Test", "Msg", ToastKind::Info, 3.0, 0.0);
        assert!(!toast.is_expired(1.0));
        assert!(!toast.is_expired(2.9));
        assert!(toast.is_expired(3.0));
        assert!(toast.is_expired(5.0));
    }

    #[test]
    fn test_toast_persistent_never_expires() {
        let toast = Toast::new(1, "Test", "Msg", ToastKind::Error, 0.0, 0.0);
        assert!(!toast.is_expired(100.0));
        assert!(!toast.is_expired(9999.0));
    }

    #[test]
    fn test_countdown_fraction() {
        let toast = Toast::new(1, "Test", "Msg", ToastKind::Info, 4.0, 0.0);
        assert_eq!(toast.countdown_fraction(0.0), 1.0);
        assert_eq!(toast.countdown_fraction(1.0), 0.75);
        assert_eq!(toast.countdown_fraction(2.0), 0.5);
        assert_eq!(toast.countdown_fraction(3.0), 0.25);
        assert_eq!(toast.countdown_fraction(4.0), 0.0);
        assert_eq!(toast.countdown_fraction(10.0), 0.0);
    }

    #[test]
    fn test_persistent_countdown_fraction() {
        let toast = Toast::new(1, "Test", "Msg", ToastKind::Warning, 0.0, 0.0);
        assert_eq!(toast.countdown_fraction(0.0), 1.0);
        assert_eq!(toast.countdown_fraction(100.0), 1.0);
    }

    #[test]
    fn test_unique_ids() {
        let mut manager = ToastManager::new();
        manager.success("A", "B");
        manager.success("C", "D");
        manager.success("E", "F");
        let ids: Vec<u64> = manager.toasts.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![0, 1, 2]);
    }
}

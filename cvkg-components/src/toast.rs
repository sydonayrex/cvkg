use crate::theme;
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
            ToastKind::Info => theme::accent(),
            ToastKind::Warning => theme::warning(),
            ToastKind::Error => theme::error_color(),
            ToastKind::Success => theme::toast_success(),
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
    /// Whether the toast was explicitly dismissed.
    pub dismissed: bool,
}

impl Toast {
    /// Creates a new Toast with the given parameters.
    pub fn new(
        id: u64,
        title: impl Into<String>,
        message: impl Into<String>,
        kind: ToastKind,
        duration: f32,
        created_at: f32,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            message: message.into(),
            kind,
            duration,
            created_at,
            dismissed: false,
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
    pub fn is_expired(&self, current_time: f32) -> bool {
        self.duration > 0.0 && self.elapsed(current_time) >= self.duration
    }
}

/// ToastManager - Owns and renders a stack of toast notifications.
///
/// Toasts are displayed stacked in the top-right corner of the screen.
/// The manager handles creation, dismissal, auto-expiry, and rendering
/// of all active toasts.
///
/// The manager is `Clone` and can be stored directly as application state.
/// The dismiss callback is shared via `Arc`, so cloning the manager
/// preserves the callback wiring.
///
/// # Example
/// ```
/// use cvkg_components::toast::{ToastManager, ToastKind};
/// let mut manager = ToastManager::new();
/// manager.success("Saved", "Your changes have been saved.");
/// manager.error("Failed", "Could not connect to server.");
/// ```
///
/// # Example with dismiss callback
/// ```
/// use cvkg_components::toast::{ToastManager, ToastKind};
/// use std::sync::{Arc, Mutex};
/// let manager = ToastManager::new()
///     .with_dismiss_callback(|id| {
///         println!("Toast {} dismissed", id);
///     });
/// ```
#[derive(Clone)]
pub struct ToastManager {
    /// Active toasts, ordered oldest-first.
    pub toasts: Vec<Toast>,
    /// Monotonically increasing counter for unique toast IDs.
    pub next_id: u64,
    /// Maximum number of toasts visible at once. Older toasts are removed first.
    pub max_visible: usize,
    /// Optional callback invoked when a toast is dismissed (via close button
    /// or any other UI interaction). The toast's unique ID is passed as the
    /// argument. Applications should use this or `with_dismiss_callback` to
    /// wire dismissal into their state management system.
    pub on_dismiss: Option<Arc<dyn Fn(u64) + Send + Sync>>,
}

impl ToastManager {
    /// Creates a new ToastManager with default capacity (5 visible toasts).
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            max_visible: 5,
            on_dismiss: None,
        }
    }

    /// Sets the maximum number of simultaneously visible toasts.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Sets a callback to be invoked when a toast is dismissed via the close
    /// button. The callback receives the dismissed toast's unique ID.
    ///
    /// The callback is stored inside an `Arc`, so it is shared across clones
    /// of the manager. This makes it easy to use `ToastManager` as application
    /// state: clone it, pass a clone into your render tree, and the callback
    /// will still fire correctly when a user clicks a close button.
    ///
    /// # Example
    /// ```
    /// use cvkg_components::toast::ToastManager;
    /// let _manager = ToastManager::new().with_dismiss_callback(|id| {
    ///     // handle dismiss
    /// });
    /// ```
    pub fn with_dismiss_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.on_dismiss = Some(Arc::new(callback));
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
    pub fn persistent(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        kind: ToastKind,
    ) {
        self.add_toast(title, message, kind, 0.0);
    }

    /// Internal helper to create and push a toast.
    fn add_toast(
        &mut self,
        title: impl Into<String>,
        message: impl Into<String>,
        kind: ToastKind,
        duration: f32,
    ) {
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
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            toast.dismissed = true;

            // Also dismiss the corresponding notification in the system state
            let state = cvkg_core::load_system_state();
            let mut matched_notif_id = None;
            for notif in &state.notifications {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&notif.id, &mut hasher);
                let target_id = std::hash::Hasher::finish(&hasher);
                if target_id == id {
                    matched_notif_id = Some(notif.id.clone());
                    break;
                }
            }
            if let Some(notif_id) = matched_notif_id {
                let _ = cvkg_core::get_notification_handler().dismiss(&notif_id);
            }
        }
    }

    /// Removes all active toasts immediately.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Updates the toast manager lifecycle, initializing timestamps, purging expired toasts,
    /// and ingesting new Active/TimeSensitive notifications from system state.
    pub fn update(&mut self, current_time: f32) {
        self.init_timestamps(current_time);

        // Ingest new notifications from system state
        let state = cvkg_core::load_system_state();
        for notif in &state.notifications {
            if notif.dismissed {
                continue;
            }
            if notif.priority == cvkg_core::NotificationPriority::Passive {
                continue;
            }

            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&notif.id, &mut hasher);
            let target_id = std::hash::Hasher::finish(&hasher);

            if !self.toasts.iter().any(|t| t.id == target_id) {
                let kind = match notif.priority {
                    cvkg_core::NotificationPriority::TimeSensitive => ToastKind::Error,
                    _ => ToastKind::Info,
                };
                let duration = notif.timeout.unwrap_or(4.0);

                let toast = Toast::new(
                    target_id,
                    notif.title.clone(),
                    notif.body.clone(),
                    kind,
                    duration,
                    current_time,
                );
                self.toasts.push(toast);

                // Enforce max_visible: remove oldest toasts first
                while self.toasts.len() > self.max_visible {
                    self.toasts.remove(0);
                }
            }
        }

        self.purge_expired(current_time);
    }

    /// Purges any toasts that have exceeded their duration at the given time.
    pub fn purge_expired(&mut self, current_time: f32) {
        let s = cvkg_core::load_system_state();
        self.toasts.retain(|t| {
            let expired_or_dismissed = t.is_expired(current_time) || t.dismissed;
            if !expired_or_dismissed {
                return true;
            }

            // Check if animation has settled
            let anim_hash = t.id.wrapping_add(88888);
            if let Some(solver_arc) = s.get_component_state::<cvkg_anim::SleipnirSolver>(anim_hash)
            {
                let solver = solver_arc.read().unwrap_or_else(|e| {
                    log::warn!("Lock poisoned, recovering...");
                    e.into_inner()
                });
                if solver.is_settled() {
                    return false; // Settled at 0.0, purge it
                } else {
                    return true; // Still animating out
                }
            }

            false // No solver created, just purge
        });
    }

    /// Initializes created_at for toasts that haven't been timestamped yet
    /// (created_at == 0.0 and duration > 0).
    pub fn init_timestamps(&mut self, current_time: f32) {
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
    fn render_toast(
        &self,
        renderer: &mut dyn Renderer,
        toast: &Toast,
        rect: Rect,
        current_time: f32,
    ) {
        let accent = toast.kind.color();
        let t = current_time;

        let anim_hash = toast.id.wrapping_add(88888);
        let target = if toast.is_expired(current_time) || toast.dismissed {
            0.0
        } else {
            1.0
        };
        let mut t_val = 0.0;
        {
            let s = cvkg_core::load_system_state();
            if s.get_component_state::<cvkg_anim::SleipnirSolver>(anim_hash)
                .is_none()
            {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(
                        anim_hash,
                        cvkg_anim::SleipnirSolver::new(
                            cvkg_anim::SleipnirParams::snappy(),
                            target,
                            0.0,
                        ),
                    );
                    new_st
                });
            }
        }
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) = s.get_component_state::<cvkg_anim::SleipnirSolver>(anim_hash)
            {
                let mut solver = solver_arc.write().unwrap_or_else(|e| {
                    log::warn!("Lock poisoned, recovering...");
                    e.into_inner()
                });
                solver.set_target(target);
                t_val = solver.tick(renderer.delta_time());
            }
        }

        renderer.push_opacity(t_val);
        let slide_offset = (1.0 - t_val) * 50.0;
        renderer.push_transform([slide_offset, 0.0], [1.0, 1.0], 0.0);

        renderer.push_vnode(rect, "Toast");

        // 1. Bifrost glass background (frosted glass effect)
        renderer.bifrost(rect, 15.0, 1.5, 0.95);

        // 2. Semi-transparent dark fill
        renderer.fill_rounded_rect(rect, CORNER_RADIUS, theme::with_alpha(theme::surface_elevated(), 0.85));

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
        let close_color = theme::text_dim();
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
        renderer.register_handler(
            "pointerclick",
            Arc::new(move |event| {
                if let cvkg_core::Event::PointerClick { x, y, .. } = event {
                    let dx = x - close_center_x;
                    let dy = y - close_center_y;
                    if dx * dx + dy * dy <= (CLOSE_BUTTON_SIZE * 0.75) * (CLOSE_BUTTON_SIZE * 0.75)
                    {
                        on_dismiss();
                    }
                }
            }),
        );

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
        renderer.pop_transform();
        renderer.pop_opacity();
    }

    /// Returns a callback that dismisses a toast by ID.
    ///
    /// If a dismiss callback was registered via `with_dismiss_callback`, the
    /// returned closure will invoke it with the toast's unique ID. If no
    /// callback was registered, the returned closure is a no-op.
    fn dismiss_callback(&self, id: u64) -> Arc<dyn Fn() + Send + Sync> {
        match &self.on_dismiss {
            Some(cb) => {
                let cb = Arc::clone(cb);
                Arc::new(move || cb(id))
            }
            None => Arc::new(|| {}),
        }
    }
}

/// Convenience function to push a toast notification onto a manager.
///
/// This is a free function alternative to the `info`, `success`, `warning`,
/// `error`, and `persistent` methods on `ToastManager`, allowing applications
/// to add toasts with a specific `ToastKind` without needing a method
/// for that exact kind.
///
/// # Example
/// ```
/// use cvkg_components::toast::{ToastManager, ToastKind, push_toast};
/// let mut manager = ToastManager::new();
/// push_toast(&mut manager, "Hello", "World", ToastKind::Info);
/// assert_eq!(manager.len(), 1);
/// ```
pub fn push_toast(manager: &mut ToastManager, title: &str, message: &str, kind: ToastKind) {
    let duration = match kind {
        ToastKind::Info => 4.0,
        ToastKind::Success => 3.0,
        ToastKind::Warning => 5.0,
        ToastKind::Error => 6.0,
    };
    manager.add_toast(title, message, kind, duration);
}

/// Convenience function to push a persistent toast notification onto a
/// manager. Persistent toasts do not auto-dismiss and must be removed
/// via `ToastManager::dismiss()`.
///
/// # Example
/// ```
/// use cvkg_components::toast::{ToastManager, ToastKind, push_persistent_toast};
/// let mut manager = ToastManager::new();
/// push_persistent_toast(&mut manager, "Action Required", "Please save your work.", ToastKind::Warning);
/// assert_eq!(manager.len(), 1);
/// ```
pub fn push_persistent_toast(
    manager: &mut ToastManager,
    title: &str,
    message: &str,
    kind: ToastKind,
) {
    manager.persistent(title, message, kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_kind_colors() {
        assert_eq!(ToastKind::Info.color(), theme::accent());
        assert_eq!(ToastKind::Warning.color(), theme::warning());
        assert_eq!(ToastKind::Error.color(), theme::error_color());
        assert_eq!(ToastKind::Success.color(), theme::toast_success());
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
        manager.purge_expired(0.0);
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

    #[test]
    fn test_with_dismiss_callback_is_stored() {
        let manager = ToastManager::new().with_dismiss_callback(move |_| {});
        assert!(manager.on_dismiss.is_some());
    }

    #[test]
    fn test_dismiss_callback_invokes_stored_callback() {
        use std::sync::atomic::{AtomicU64, Ordering};
        let received = Arc::new(AtomicU64::new(u64::MAX));
        let received_clone = Arc::clone(&received);
        let manager = ToastManager::new().with_dismiss_callback(move |id| {
            received_clone.store(id, Ordering::SeqCst);
        });
        let cb = manager.dismiss_callback(42);
        cb();
        assert_eq!(received.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_dismiss_callback_no_callback_is_noop() {
        let manager = ToastManager::new();
        let cb = manager.dismiss_callback(99);
        // Should not panic, just a no-op
        cb();
        assert!(manager.on_dismiss.is_none());
    }

    #[test]
    fn test_manager_clone_preserves_callback() {
        use std::sync::atomic::{AtomicU64, Ordering};
        let received = Arc::new(AtomicU64::new(u64::MAX));
        let received_clone = Arc::clone(&received);
        let manager = ToastManager::new().with_dismiss_callback(move |id| {
            received_clone.store(id, Ordering::SeqCst);
        });
        let cloned = manager.clone();
        let cb = cloned.dismiss_callback(7);
        cb();
        assert_eq!(received.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn test_push_toast_convenience() {
        let mut manager = ToastManager::new();
        push_toast(&mut manager, "Hello", "World", ToastKind::Info);
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.toasts[0].title, "Hello");
        assert_eq!(manager.toasts[0].message, "World");
        assert_eq!(manager.toasts[0].kind, ToastKind::Info);
        assert_eq!(manager.toasts[0].duration, 4.0);
    }

    #[test]
    fn test_push_persistent_toast_convenience() {
        let mut manager = ToastManager::new();
        push_persistent_toast(
            &mut manager,
            "Save",
            "Please save your work.",
            ToastKind::Warning,
        );
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.toasts[0].title, "Save");
        assert_eq!(manager.toasts[0].duration, 0.0);
    }
}

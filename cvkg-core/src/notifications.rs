use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationAction {
    /// Unique identifier of the action.
    pub id: String,
    /// The text label to display on the action button.
    pub title: String,
    /// Indicates whether the action performs a destructive task (e.g. Delete).
    pub is_destructive: bool,
}

/// Priority tier of a notification.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationPriority {
    /// Placed silently into the notification center without visual alerts.
    Passive,
    /// Triggers a visual alert (toast) but does not interrupt focus.
    #[default]
    Active,
    /// Important alert that bypasses standard DND/Focus bounds.
    TimeSensitive,
}

/// A structured notification representation.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Unique identifier for this notification.
    pub id: String,
    /// App or source identifier spawning this notification.
    pub app_name: Option<String>,
    /// The bold heading/title text.
    pub title: String,
    /// The detailed descriptive body text.
    pub body: String,
    /// Optional URI or path to an icon asset.
    pub icon: Option<String>,
    /// Optional sound identifier to play when posting.
    pub sound: Option<String>,
    /// Interactive actions available on this notification.
    pub actions: Vec<NotificationAction>,
    /// Timer duration in seconds after which the toast auto-dismisses.
    pub timeout: Option<f32>,
    /// Priority level for delivery logic.
    pub priority: NotificationPriority,
    /// Time (in seconds since renderer startup) when this notification was posted.
    pub timestamp: f32,
    /// Whether the notification has been dismissed/read.
    pub dismissed: bool,
}

/// Error type indicating a failure in generating or posting a notification.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, thiserror::Error)]
pub enum NotificationError {
    /// Permissions denied.
    #[error("Notification permission denied")]
    PermissionDenied,
    /// Failed to post the notification.
    #[error("Failed to post notification")]
    PostFailed,
}

/// State of notification permissions.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationPermission {
    /// Explicitly allowed.
    Granted,
    /// Explicitly blocked.
    Denied,
    /// Prompt has not been shown or decided yet.
    #[default]
    NotDetermined,
}

/// Core interface for routing and dispatching notification events.
pub trait NotificationHandler: Send + Sync {
    /// Posts a new notification.
    fn show(&self, notification: Notification) -> Result<(), NotificationError>;
    /// Dismisses a notification by ID.
    fn dismiss(&self, id: &str) -> Result<(), NotificationError>;
    /// Requests delivery permission.
    fn request_permission(&self) -> NotificationPermission;
}

static NEXT_NOTIFICATION_ID: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(1);

/// Default in-app notification handler that writes state to AppState.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultNotificationHandler;

impl NotificationHandler for DefaultNotificationHandler {
    /// Save the notification to the global system state (history) and auto-assign an ID if empty.
    fn show(&self, notification: Notification) -> Result<(), NotificationError> {
        let mut notif = notification;
        if notif.id.is_empty() {
            let id = NEXT_NOTIFICATION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            notif.id = format!("notif_{}", id);
        }
        update_system_state(|state| {
            let mut new_state = state.clone();
            new_state.notifications.push(notif.clone());
            new_state
        });
        Ok(())
    }

    /// Mark a notification as dismissed/read in the global system state.
    fn dismiss(&self, id: &str) -> Result<(), NotificationError> {
        update_system_state(|state| {
            let mut new_state = state.clone();
            for notif in &mut new_state.notifications {
                if notif.id == id {
                    notif.dismissed = true;
                }
            }
            new_state
        });
        Ok(())
    }

    /// Returns the permission state (always Granted for internal in-app notifications).
    fn request_permission(&self) -> NotificationPermission {
        NotificationPermission::Granted
    }
}

static NOTIFICATION_HANDLER: once_cell::sync::OnceCell<std::sync::Arc<dyn NotificationHandler>> =
    once_cell::sync::OnceCell::new();

/// Sets the global notification handler.
pub fn set_notification_handler(handler: std::sync::Arc<dyn NotificationHandler>) {
    let _ = NOTIFICATION_HANDLER.set(handler);
}

/// Gets the global notification handler, fallback to DefaultNotificationHandler.
pub fn get_notification_handler() -> std::sync::Arc<dyn NotificationHandler> {
    NOTIFICATION_HANDLER
        .get_or_init(|| std::sync::Arc::new(DefaultNotificationHandler))
        .clone()
}

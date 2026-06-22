// =============================================================================
// TEXT INPUT -- Direction enum for cursor movement
// =============================================================================

/// Direction for cursor movement in text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Forward,
    Backward,
    Up,
    Down,
    LineStart,
    LineEnd,
    WordForward,
    WordBackward,
}

/// Text input state managed by the renderer.
///
/// Components don't store this directly -- the renderer maintains it
/// and components query/modify it through the Renderer trait methods.
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    /// The full text content.
    pub text: String,
    /// Cursor position as byte offset into the text.
    pub cursor_pos: usize,
    /// Selection anchor. If Some, the selection is from anchor to cursor.
    /// If None, there is no selection.
    pub selection_anchor: Option<usize>,
    /// Whether the input is focused (shows cursor, accepts keyboard).
    pub focused: bool,
    /// Whether the caret is currently visible (for blinking).
    pub caret_visible: bool,
    /// Last edit timestamp for undo coalescing.
    pub last_edit_time: f32,
}

impl TextInputState {
    /// Create a new TextInputState with the given initial text.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor_pos = text.len();
        Self {
            text,
            cursor_pos,
            selection_anchor: None,
            focused: false,
            caret_visible: true,
            last_edit_time: 0.0,
        }
    }

    /// Get the selection range as (start, end) byte offsets.
    /// Returns None if there is no selection.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            if anchor <= self.cursor_pos {
                (anchor, self.cursor_pos)
            } else {
                (self.cursor_pos, anchor)
            }
        })
    }

    /// Get the selected text, or empty string if no selection.
    pub fn selected_text(&self) -> String {
        self.selection_range()
            .map(|(start, end)| self.text[start..end].to_string())
            .unwrap_or_default()
    }

    /// Insert text at the current cursor position, replacing any selection.
    pub fn insert(&mut self, new_text: &str) {
        if let Some((start, end)) = self.selection_range() {
            self.text.replace_range(start..end, new_text);
            self.cursor_pos = start + new_text.len();
        } else {
            self.text.insert_str(self.cursor_pos, new_text);
            self.cursor_pos += new_text.len();
        }
        self.selection_anchor = None;
    }

    /// Delete characters. If there's a selection, delete it.
    /// Otherwise delete `count` characters backward (backspace) or forward (delete).
    pub fn delete(&mut self, backward: bool, count: usize) -> String {
        if let Some((start, end)) = self.selection_range() {
            let deleted = self.text[start..end].to_string();
            self.text.replace_range(start..end, "");
            self.cursor_pos = start;
            self.selection_anchor = None;
            return deleted;
        }

        if backward && self.cursor_pos > 0 {
            let start = self.cursor_pos.saturating_sub(count);
            let deleted = self.text[start..self.cursor_pos].to_string();
            self.text.replace_range(start..self.cursor_pos, "");
            self.cursor_pos = start;
            deleted
        } else if !backward && self.cursor_pos < self.text.len() {
            let end = (self.cursor_pos + count).min(self.text.len());
            let deleted = self.text[self.cursor_pos..end].to_string();
            self.text.replace_range(self.cursor_pos..end, "");
            deleted
        } else {
            String::new()
        }
    }

    /// Move the cursor in the given direction.
    pub fn move_cursor(&mut self, direction: TextDirection, extend_selection: bool) {
        if !extend_selection {
            self.selection_anchor = None;
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_pos);
        }

        match direction {
            TextDirection::Forward if self.cursor_pos < self.text.len() => {
                // Move to next character boundary (UTF-8 safe)
                let next = self.text[self.cursor_pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor_pos + i)
                    .unwrap_or(self.text.len());
                self.cursor_pos = next;
            }
            TextDirection::Backward if self.cursor_pos > 0 => {
                let prev = self.text[..self.cursor_pos]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.cursor_pos = prev;
            }
            TextDirection::LineStart => {
                self.cursor_pos = 0;
            }
            TextDirection::LineEnd => {
                self.cursor_pos = self.text.len();
            }
            TextDirection::WordForward => {
                // Find next word boundary
                let rest = &self.text[self.cursor_pos..];
                // Skip current word chars
                let after_word = rest
                    .char_indices()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(i, _)| i)
                    .unwrap_or(rest.len());
                // Skip whitespace
                let after_space = rest[after_word..]
                    .char_indices()
                    .find(|(_, c)| !c.is_whitespace())
                    .map(|(i, _)| after_word + i)
                    .unwrap_or(rest.len());
                self.cursor_pos = (self.cursor_pos + after_space).min(self.text.len());
            }
            TextDirection::WordBackward => {
                let before = &self.text[..self.cursor_pos];
                // Skip whitespace going backward
                let before_word = before
                    .char_indices()
                    .rev()
                    .find(|(_, c)| !c.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                // Skip word chars going backward
                let word_start = before[..before_word]
                    .char_indices()
                    .rev()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.cursor_pos = word_start;
            }
            _ => {} // Up/Down handled by multi-line components
        }

        if !extend_selection {
            self.selection_anchor = None;
        }
    }

    /// Select all text.
    pub fn select_all(&mut self) {
        self.cursor_pos = self.text.len();
        self.selection_anchor = Some(0);
    }

    /// Get the byte offset of the cursor.
    pub fn cursor_byte_pos(&self) -> usize {
        self.cursor_pos
    }
}

/// Action details for interactive buttons inside a notification.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
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

/// Filter mapping name to extension list for a file dialog.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileFilter {
    /// Friendly name of the filter (e.g. "Images").
    pub name: String,
    /// List of file extensions (e.g. ["png", "jpg"]).
    pub extensions: Vec<String>,
}

/// The mode/purpose of the file dialog.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FileDialogMode {
    /// Pick a single or multiple files to open.
    #[default]
    OpenFile,
    /// Pick a directory path.
    OpenDirectory,
    /// Prompt for a location/name to save a file.
    SaveFile,
}

/// Dialog options for picking files or directories.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDialog {
    /// Title displayed in the dialog window.
    pub title: String,
    /// Optional starting directory path.
    pub default_path: Option<String>,
    /// Extensions used to filter selection.
    pub filters: Vec<FileFilter>,
    /// Open/save mode.
    pub mode: FileDialogMode,
    /// Allows selecting multiple files if in OpenFile mode.
    pub allow_multiple: bool,
}

/// Errors returned by the file dialog.
#[derive(Debug, thiserror::Error)]
pub enum FileDialogError {
    /// The user closed the dialog without selecting anything.
    #[error("File dialog cancelled")]
    Cancelled,
    /// An input/output error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Platform-specific error.
    #[error("Platform error: {0}")]
    Platform(String),
}

impl FileDialog {
    /// Creates a new FileDialog with the given mode.
    pub fn new(mode: FileDialogMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Sets the dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Adds a file filter.
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Sets the default starting directory path.
    pub fn default_path(mut self, path: impl Into<String>) -> Self {
        self.default_path = Some(path.into());
        self
    }

    /// Sets whether selecting multiple files is allowed.
    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FileDialog {
    /// Pick file(s) or folder based on current mode configuration.
    pub fn pick(self) -> Result<Vec<std::path::PathBuf>, FileDialogError> {
        let mut dialog = rfd::FileDialog::new();
        dialog = dialog.set_title(&self.title);
        if let Some(path) = &self.default_path {
            dialog = dialog.set_directory(path);
        }
        for filter in &self.filters {
            let refs: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &refs);
        }

        match self.mode {
            FileDialogMode::OpenFile => {
                if self.allow_multiple {
                    dialog.pick_files().ok_or(FileDialogError::Cancelled)
                } else {
                    Ok(dialog.pick_file().into_iter().collect())
                }
            }
            FileDialogMode::OpenDirectory => Ok(dialog.pick_folder().into_iter().collect()),
            FileDialogMode::SaveFile => Ok(dialog.save_file().into_iter().collect()),
        }
    }

    /// Helper to pick a single file/directory, returning None if cancelled.
    pub fn pick_single(self) -> Result<Option<std::path::PathBuf>, FileDialogError> {
        let results = self.pick()?;
        Ok(results.into_iter().next())
    }
}

#[cfg(target_arch = "wasm32")]
impl FileDialog {
    /// Pick is unsupported/mocked on WASM.
    pub fn pick(self) -> Result<Vec<std::path::PathBuf>, FileDialogError> {
        Err(FileDialogError::Platform(
            "FileDialog is not supported synchronously on WebAssembly".to_string(),
        ))
    }

    /// Helper to pick a single file/directory, returning None if cancelled.
    pub fn pick_single(self) -> Result<Option<std::path::PathBuf>, FileDialogError> {
        Err(FileDialogError::Platform(
            "FileDialog is not supported synchronously on WebAssembly".to_string(),
        ))
    }
}

/// Error type representing a failure in Document load/save/parse operations.
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    /// An input/output error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Failure during deserialization or parsing.
    #[error("Parse error: {0}")]
    Parse(String),
    /// Failure during serialization.
    #[error("Serialization error: {0}")]
    Serialize(String),
}

/// A document interface mapping to local filesystem persistence.
pub trait Document: Send + Sync {
    /// Loads the document from the specified path.
    fn read_from(path: &std::path::Path) -> Result<Self, DocumentError>
    where
        Self: Sized;

    /// Saves the document to the specified path.
    fn write_to(&self, path: &std::path::Path) -> Result<(), DocumentError>;

    /// Returns true if the document has unsaved modifications.
    fn is_dirty(&self) -> bool;

    /// Marks the document as clean/saved.
    fn mark_clean(&mut self);
}

/// Periodic auto-save coordinator for open Documents.
pub struct AutoSaveManager {
    /// Time interval in seconds between auto-saves.
    pub interval: f32,
    /// Elapsed timer tracker.
    pub timer: f32,
    /// Registered open documents under management.
    pub documents: Vec<(std::path::PathBuf, Box<dyn Document>)>,
}

impl AutoSaveManager {
    /// Creates a new AutoSaveManager with the specified check interval.
    pub fn new(interval: f32) -> Self {
        Self {
            interval,
            timer: 0.0,
            documents: Vec::new(),
        }
    }

    /// Register a document with its current file path.
    pub fn register(&mut self, path: std::path::PathBuf, doc: Box<dyn Document>) {
        self.documents.push((path, doc));
    }

    /// Advance the timer and auto-save any dirty documents when the interval is reached.
    pub fn tick(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.interval {
            self.timer = 0.0;
            for (path, doc) in &mut self.documents {
                if doc.is_dirty() {
                    match doc.write_to(path) {
                        Ok(()) => {
                            doc.mark_clean();
                            log::info!("[AutoSaveManager] Auto-saved document to {:?}", path);
                        }
                        Err(e) => {
                            log::error!(
                                "[AutoSaveManager] Failed to auto-save document to {:?}: {:?}",
                                path,
                                e
                            );
                        }
                    }
                }
            }
        }
    }
}

// ── Menu Bar API ──────────────────────────────────────────────────────────────

/// Keyboard modifier flags used by [`KeyboardShortcut`].
///
/// On macOS, `cmd` maps to the Command (⌘) key.
/// On all other platforms, `cmd` maps to the Control key.
/// This is enforced at the renderer level, not here; the data model is OS-agnostic.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Command on macOS, Control on Windows/Linux.
    pub cmd: bool,
    /// Shift key.
    pub shift: bool,
    /// Alt/Option key.
    pub alt: bool,
    /// Control key (distinct from cmd on all platforms).
    pub ctrl: bool,
}

/// A keyboard shortcut binding to a menu action.
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    /// The key character or name, e.g. `"s"`, `"z"`, `"Return"`.
    pub key: String,
    /// The required modifier combination.
    pub modifiers: Modifiers,
}

impl KeyboardShortcut {
    /// Convenience constructor: cmd (or ctrl on non-macOS) + `key`.
    pub fn cmd(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers {
                cmd: true,
                ..Default::default()
            },
        }
    }

    /// Convenience constructor: cmd+Shift + `key`.
    pub fn cmd_shift(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers {
                cmd: true,
                shift: true,
                ..Default::default()
            },
        }
    }
}

/// A single entry in a [`MenuBar`].
///
/// Actions hold a callback that is invoked when the user activates the item
/// (either via the menu UI or via the associated keyboard shortcut).
/// Separators provide visual grouping. Submenus allow hierarchical menus.
pub enum MenuItem {
    /// An activatable menu entry with an optional shortcut and enabled/disabled state.
    Action {
        label: String,
        shortcut: Option<KeyboardShortcut>,
        action: std::sync::Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
    },
    /// A nested submenu.
    Submenu { label: String, items: Vec<MenuItem> },
    /// A visual separator line between groups of items.
    Separator,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action { label, enabled, .. } => f
                .debug_struct("Action")
                .field("label", label)
                .field("enabled", enabled)
                .finish(),
            Self::Submenu { label, items } => f
                .debug_struct("Submenu")
                .field("label", label)
                .field("items", items)
                .finish(),
            Self::Separator => write!(f, "Separator"),
        }
    }
}

/// A top-level menu bar containing [`MenuItem`]s.
///
/// The menu bar is a data model only; rendering it into an OS-native menu is
/// handled by the platform renderer (`cvkg-render-native`).
pub struct MenuBar {
    /// Ordered list of top-level menu items.
    pub items: Vec<MenuItem>,
}

impl MenuBar {
    /// Create an empty menu bar.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append a menu item to the bar.
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Build the standard CVKG menu structure with all conventional shortcuts.
    ///
    /// The `cmd` modifier maps to ⌘ on macOS and Ctrl on Windows/Linux -- this
    /// translation is enforced by the renderer, not here.
    ///
    /// Menus included:
    /// - **File**: New, Open, Save, Close
    /// - **Edit**: Undo, Redo, Cut, Copy, Paste, Select All, Find
    /// - **View**: Zoom In, Zoom Out, Fullscreen
    /// - **Window**: Minimize, Zoom, Bring All to Front
    /// - **Help**: Search Help
    #[allow(clippy::too_many_arguments)]
    pub fn standard(
        new_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        open_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        save_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        close_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        quit_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        undo_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        redo_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        cut_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        copy_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        paste_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        select_all_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
        find_fn: std::sync::Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        let mut bar = Self::new();

        // ── File ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "File".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "New".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("n")),
                    action: new_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Open…".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("o")),
                    action: open_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Save".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("s")),
                    action: save_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Close".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("w")),
                    action: close_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Quit".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("q")),
                    action: quit_fn,
                    enabled: true,
                },
            ],
        });

        // ── Edit ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Edit".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Undo".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("z")),
                    action: undo_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Redo".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd_shift("z")),
                    action: redo_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Cut".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("x")),
                    action: cut_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Copy".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("c")),
                    action: copy_fn,
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Paste".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("v")),
                    action: paste_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Select All".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("a")),
                    action: select_all_fn,
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Find…".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("f")),
                    action: find_fn,
                    enabled: true,
                },
            ],
        });

        // ── View ──────────────────────────────────────────────────────────────
        // View items carry no application-level callbacks at the model layer;
        // zoom and fullscreen are handled by the renderer directly.
        let noop: std::sync::Arc<dyn Fn() + Send + Sync> = std::sync::Arc::new(|| {});
        bar.add_item(MenuItem::Submenu {
            label: "View".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Zoom In".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("=")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Zoom Out".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("-")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Toggle Fullscreen".to_string(),
                    shortcut: Some(KeyboardShortcut {
                        key: "f".to_string(),
                        modifiers: Modifiers {
                            ctrl: true,
                            ..Default::default()
                        },
                    }),
                    action: noop.clone(),
                    enabled: true,
                },
            ],
        });

        // ── Window ────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Window".to_string(),
            items: vec![
                MenuItem::Action {
                    label: "Minimize".to_string(),
                    shortcut: Some(KeyboardShortcut::cmd("m")),
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Action {
                    label: "Zoom".to_string(),
                    shortcut: None,
                    action: noop.clone(),
                    enabled: true,
                },
                MenuItem::Separator,
                MenuItem::Action {
                    label: "Bring All to Front".to_string(),
                    shortcut: None,
                    action: noop.clone(),
                    enabled: true,
                },
            ],
        });

        // ── Help ──────────────────────────────────────────────────────────────
        bar.add_item(MenuItem::Submenu {
            label: "Help".to_string(),
            items: vec![MenuItem::Action {
                label: "Search Help".to_string(),
                shortcut: None,
                action: noop,
                enabled: true,
            }],
        });

        bar
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}


//! Window management types and traits.
//!
//! Extracted from lib.rs (P1-13).

use std::sync::Arc;

/// Unique identifier for a window instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WindowId(pub u64);

/// Specifies the layering behavior of the window relative to other windows.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum WindowLevel {
    /// Standard window.
    #[default]
    Normal,
    /// Window stays above all standard windows.
    AlwaysOnTop,
    /// Menu or pop-up level window.
    PopUpMenu,
}

/// Configuration settings for creating a new window.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowConfig {
    /// The window title bar text.
    pub title: String,
    /// Default width and height of the window.
    pub size: (f32, f32),
    /// Minimum allowed dimensions.
    pub min_size: Option<(f32, f32)>,
    /// Maximum allowed dimensions.
    pub max_size: Option<(f32, f32)>,
    /// Whether the window can be resized by the user.
    pub resizable: bool,
    /// Whether the window background is transparent.
    pub transparent: bool,
    /// Whether the window title bar and border decorations are drawn.
    pub decorations: bool,
    /// The window level layer.
    pub level: WindowLevel,
}

impl Default for WindowConfig {
    /// Create a standard default window configuration.
    fn default() -> Self {
        Self {
            title: "CVKG Window".to_string(),
            size: (800.0, 600.0),
            min_size: None,
            max_size: None,
            resizable: true,
            transparent: false,
            decorations: true,
            level: WindowLevel::Normal,
        }
    }
}

/// Abstract trait representing a platform-native window.
/// Implementations delegate calls back to the platform renderers and events.
pub trait Window: Send + Sync {
    /// Request closing of the window.
    fn close(&self);
    /// Change the title bar text of the window.
    fn set_title(&self, title: &str);
    /// Update the window's physical dimensions.
    fn set_size(&self, width: f32, height: f32);
    /// Check if the window currently has keyboard focus.
    fn is_key(&self) -> bool;
    /// Check if this is the primary main application window.
    fn is_main(&self) -> bool;
    /// Check if the window is currently visible/mapped.
    fn is_visible(&self) -> bool;
    /// Hide or show the window.
    fn set_visible(&self, visible: bool);
    /// Bring the window to the front and focus it.
    fn bring_to_front(&self);
}

/// A handle to a native window that can be used by application code.
#[derive(Clone)]
pub struct WindowHandle {
    /// The unique identifier of this window.
    pub id: WindowId,
    /// Reference to the underlying platform window.
    pub inner: Arc<dyn Window>,
}

impl std::fmt::Debug for WindowHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowHandle")
            .field("id", &self.id)
            .finish()
    }
}

impl WindowHandle {
    /// Create a new WindowHandle.
    pub fn new(id: WindowId, inner: Arc<dyn Window>) -> Self {
        Self { id, inner }
    }
    /// Request the window to close.
    pub fn close(self) {
        self.inner.close();
    }
    /// Set the title text of the window.
    pub fn set_title(&self, title: &str) {
        self.inner.set_title(title);
    }
    /// Resize the window.
    pub fn set_size(&self, width: f32, height: f32) {
        self.inner.set_size(width, height);
    }
    /// Returns true if this window has key focus.
    pub fn is_key(&self) -> bool {
        self.inner.is_key()
    }
    /// Returns true if this is the main application window.
    pub fn is_main(&self) -> bool {
        self.inner.is_main()
    }
    /// Returns true if the window is visible.
    pub fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }
    /// Set visibility of the window.
    pub fn set_visible(&self, visible: bool) {
        self.inner.set_visible(visible);
    }
    /// Bring this window to the foreground.
    pub fn bring_to_front(&self) {
        self.inner.bring_to_front();
    }
}

/// Action to take when a window close request event is received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WindowCloseAction {
    /// Close the window immediately.
    Allow,
    /// Request confirmation from the user (e.g. show dialog).
    Confirm,
    /// Ignore the close request.
    Deny,
}

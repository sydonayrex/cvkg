//! Native Shell Module
//!
//! Provides a unified interface for creating and managing native application
//! windows through multiple backend implementations: Tauri, Wry, or Headless
//! (for testing and CI environments).

use std::error::Error;
use std::fmt;

/// The rendering backend used for the native shell window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellBackend {
    /// Headless mode — no actual window, useful for testing and CI.
    Headless,
}

/// Configuration and handle for a native shell instance.
///
/// Use [`NativeShell::new`] to create a default shell, then chain
/// builder methods to customize it before calling [`create_window`].
#[derive(Debug, Clone)]
pub struct NativeShell {
    /// The rendering backend to use.
    pub backend: ShellBackend,
    /// The initial window title.
    pub window_title: String,
    /// The initial window width in pixels.
    pub width: u32,
    /// The initial window height in pixels.
    pub height: u32,
}

impl NativeShell {
    /// Create a new [`NativeShell`] with default dimensions (1280x720) and
    /// the [`ShellBackend::Headless`] backend.
    ///
    /// # Arguments
    ///
    /// * `title` — The initial window title.
    ///
    /// # Examples
    ///
    /// ```
    /// use cvkg_cli::native_shell::{NativeShell, ShellBackend};
    /// let shell = NativeShell::new("My App");
    /// assert_eq!(shell.window_title, "My App");
    /// assert_eq!(shell.width, 1280);
    /// assert_eq!(shell.height, 720);
    /// assert_eq!(shell.backend, ShellBackend::Headless);
    /// ```
    pub fn new(title: &str) -> Self {
        Self {
            backend: ShellBackend::Headless,
            window_title: title.to_string(),
            width: 1280,
            height: 720,
        }
    }

    /// Set the window dimensions.
    ///
    /// # Arguments
    ///
    /// * `w` — Width in pixels.
    /// * `h` — Height in pixels.
    pub fn with_size(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set the rendering backend.
    ///
    /// # Arguments
    ///
    /// * `backend` — The [`ShellBackend`] to use.
    pub fn backend(mut self, backend: ShellBackend) -> Self {
        self.backend = backend;
        self
    }
}

/// A handle to a created native window.
///
/// Obtain a [`ShellWindow`] by calling [`create_window`].
#[derive(Debug, Clone)]
pub struct ShellWindow {
    /// Unique identifier for the window.
    pub id: u32,
    /// The current window title.
    pub title: String,
    /// The current window width in pixels.
    pub width: u32,
    /// The current window height in pixels.
    pub height: u32,
}

impl ShellWindow {
    /// Update the window title.
    ///
    /// # Arguments
    ///
    /// * `title` — The new title string.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Resize the window.
    ///
    /// # Arguments
    ///
    /// * `w` — New width in pixels.
    /// * `h` — New height in pixels.
    pub fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
    }

    /// Close the window and release associated resources.
    pub fn close(self) {
        // In a real implementation this would call into the backend
        // to destroy the native window. For now the handle is simply
        // dropped.
    }
}

/// Errors that can occur when creating or managing native shell windows.
#[derive(Debug, Clone)]
pub struct ShellError {
    /// A human-readable error message.
    pub message: String,
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShellError: {}", self.message)
    }
}

impl Error for ShellError {}

/// Events that can be emitted by a native window.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// The window was resized to the given dimensions.
    Resized(u32, u32),
    /// The window gained focus.
    Focused,
    /// The window lost focus.
    Unfocused,
    /// The user requested the window be closed.
    CloseRequested,
}

/// Create a native window from the given [`NativeShell`] configuration.
///
/// Currently only [`ShellBackend::Headless`] is supported, which creates
/// an in-memory window handle suitable for testing and CI.
///
/// # Errors
///
/// Returns a [`ShellError`] if the window could not be created.
/// requested backend is not available on the current platform).
///
/// # Examples
///
/// ```
/// use cvkg_cli::native_shell::{NativeShell, ShellBackend, create_window};
/// let shell = NativeShell::new("Test").backend(ShellBackend::Headless);
/// let window = create_window(&shell).expect("Failed to create window");
/// assert_eq!(window.title, "Test");
/// ```
pub fn create_window(shell: &NativeShell) -> Result<ShellWindow, ShellError> {
    match shell.backend {
        ShellBackend::Headless => Ok(ShellWindow {
            id: 0,
            title: shell.window_title.clone(),
            width: shell.width,
            height: shell.height,
        }),
    }
}

/// Poll for pending window events in a non-blocking fashion.
///
/// Returns an empty Vec if no events are currently pending or if the
/// window is operating in headless mode.
///
/// # Arguments
///
/// * `window` — The [`ShellWindow`] to poll events for.
///
/// # Examples
///
/// ```
/// use cvkg_cli::native_shell::{NativeShell, ShellBackend, create_window, poll_events};
/// let shell = NativeShell::new("Test").backend(ShellBackend::Headless);
/// let window = create_window(&shell).unwrap();
/// let events = poll_events(&window);
/// // Headless mode always returns an empty event list
/// assert!(events.is_empty());
/// ```
pub fn poll_events(_window: &ShellWindow) -> Vec<WindowEvent> {
    // In a real implementation this would query the backend event loop.
    // Headless mode returns no events.
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_new_defaults() {
        let shell = NativeShell::new("Test App");
        assert_eq!(shell.window_title, "Test App");
        assert_eq!(shell.width, 1280);
        assert_eq!(shell.height, 720);
        assert_eq!(shell.backend, ShellBackend::Headless);
    }

    #[test]
    fn test_shell_with_size() {
        let shell = NativeShell::new("Sized").with_size(1920, 1080);
        assert_eq!(shell.width, 1920);
        assert_eq!(shell.height, 1080);
    }

    #[test]
    fn test_shell_backend() {
        let shell = NativeShell::new("Backend").backend(ShellBackend::Headless);
        assert_eq!(shell.backend, ShellBackend::Headless);
    }

    #[test]
    fn test_shell_builder_chain() {
        let shell = NativeShell::new("Chained")
            .with_size(800, 600)
            .backend(ShellBackend::Headless);
        assert_eq!(shell.window_title, "Chained");
        assert_eq!(shell.width, 800);
        assert_eq!(shell.height, 600);
        assert_eq!(shell.backend, ShellBackend::Headless);
    }

    #[test]
    fn test_create_window_headless() {
        let shell = NativeShell::new("Headless Win").backend(ShellBackend::Headless);
        let window = create_window(&shell).expect("Headless window creation should succeed");
        assert_eq!(window.id, 0);
        assert_eq!(window.title, "Headless Win");
        assert_eq!(window.width, 1280);
        assert_eq!(window.height, 720);
    }

    #[test]
    fn test_window_set_title() {
        let mut win = ShellWindow {
            id: 1,
            title: "Old".to_string(),
            width: 800,
            height: 600,
        };
        win.set_title("New Title");
        assert_eq!(win.title, "New Title");
    }

    #[test]
    fn test_window_resize() {
        let mut win = ShellWindow {
            id: 1,
            title: "Resizable".to_string(),
            width: 800,
            height: 600,
        };
        win.resize(1920, 1080);
        assert_eq!(win.width, 1920);
        assert_eq!(win.height, 1080);
    }

    #[test]
    fn test_window_close() {
        let win = ShellWindow {
            id: 1,
            title: "Closable".to_string(),
            width: 800,
            height: 600,
        };
        win.close();
        // After close, the handle is dropped. No assertion needed.
    }

    #[test]
    fn test_poll_events_headless() {
        let shell = NativeShell::new("Poll").backend(ShellBackend::Headless);
        let window = create_window(&shell).unwrap();
        let events = poll_events(&window);
        assert!(events.is_empty());
    }

    #[test]
    fn test_shell_error_display() {
        let err = ShellError {
            message: "something went wrong".to_string(),
        };
        assert_eq!(format!("{}", err), "ShellError: something went wrong");
    }

    #[test]
    fn test_shell_error_implements_std_error() {
        let err = ShellError {
            message: "test".to_string(),
        };
        let _: &dyn Error = &err;
    }

    #[test]
    fn test_window_event_equality() {
        assert_eq!(WindowEvent::Focused, WindowEvent::Focused);
        assert_eq!(
            WindowEvent::Resized(800, 600),
            WindowEvent::Resized(800, 600)
        );
        assert_ne!(WindowEvent::Focused, WindowEvent::Unfocused);
        assert_ne!(
            WindowEvent::Resized(800, 600),
            WindowEvent::Resized(1024, 768)
        );
    }
}

// =============================================================================
// CLIPBOARD -- System clipboard access
// =============================================================================

/// Trait for clipboard operations.
///
/// The native renderer implements this via `arboard` on desktop platforms.
/// On WASM, it uses the browser Clipboard API.
pub trait ClipboardProvider: Send + Sync {
    /// Read text from the system clipboard.
    fn read_text(&self) -> Option<String>;
    /// Write text to the system clipboard.
    fn write_text(&self, text: &str);
}

/// Default clipboard implementation using `arboard`.
/// Note: This is only available when the `arboard` feature is enabled.
/// The renderer provides the concrete implementation.
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub struct SystemClipboard;

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
impl ClipboardProvider for SystemClipboard {
    fn read_text(&self) -> Option<String> {
        use std::process::Command;
        // Fallback: try pbpaste on macOS
        Command::new("pbpaste")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
    }

    fn write_text(&self, text: &str) {
        use std::process::Command;
        // Fallback: try pbcopy on macOS
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}


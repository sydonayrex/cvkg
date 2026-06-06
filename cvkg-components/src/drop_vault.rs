//! DropVault — File Upload Component with drag-and-drop support.
//!
//! Provides a file upload zone that accepts files via drag-and-drop from
//! the OS file manager or via a click-to-browse file picker. Shows file
//! names, sizes, and upload progress.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::{Arc, Mutex};

/// A file upload zone with drag-and-drop support.
///
/// # Features:
/// - Drag files from the OS file manager onto the drop zone.
/// - Click to open the native file picker.
/// - Shows file name, size, and upload progress.
/// - Supports multiple files.
///
/// # Architecture:
/// The component does NOT implement file reading or network upload — it exposes
/// events (`on_files_selected`, `on_upload_progress`) that the APP handles.
/// The component is purely visual + interaction.
#[derive(Clone)]
pub struct DropVault {
    /// Accepted MIME types. Empty = accept all.
    pub accepted_types: Vec<String>,
    /// Max file count. Default = 1.
    pub max_files: usize,
    /// Max file size in bytes. Default = 10MB.
    pub max_file_size: u64,
    /// Callback when files are selected (via picker or drop).
    pub on_files_selected: Option<Arc<dyn Fn(Vec<VaultFile>) + Send + Sync>>,
    /// Current upload state.
    pub uploads: Arc<Mutex<Vec<VaultEntry>>>,
    /// Whether the drag is currently over this zone.
    pub is_drag_over: Arc<Mutex<bool>>,
}

/// Information about a selected file.
#[derive(Debug, Clone)]
pub struct VaultFile {
    /// File name only (NO path — security).
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// MIME type (inferred from extension if OS doesn't provide it).
    pub mime_type: String,
}

/// Upload state for a single file.
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub file_info: VaultFile,
    pub progress: f32,
    pub status: VaultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultStatus {
    Pending,
    Uploading,
    Complete,
    Failed(String),
}

impl Default for DropVault {
    fn default() -> Self {
        Self::new()
    }
}

impl DropVault {
    pub fn new() -> Self {
        Self {
            accepted_types: Vec::new(),
            max_files: 1,
            max_file_size: 10 * 1024 * 1024,
            on_files_selected: None,
            uploads: Arc::new(Mutex::new(Vec::new())),
            is_drag_over: Arc::new(Mutex::new(false)),
        }
    }

    pub fn accepted_types(mut self, types: Vec<String>) -> Self {
        self.accepted_types = types;
        self
    }

    pub fn max_files(mut self, max: usize) -> Self {
        self.max_files = max.max(1);
        self
    }

    pub fn max_size(mut self, bytes: u64) -> Self {
        self.max_file_size = bytes;
        self
    }

    pub fn on_files_selected<F: Fn(Vec<VaultFile>) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.on_files_selected = Some(Arc::new(f));
        self
    }

    /// Add a file to the upload queue.
    pub fn add_file(&self, file: VaultFile) {
        if let Ok(mut uploads) = self.uploads.lock() {
            uploads.push(VaultEntry {
                file_info: file,
                progress: 0.0,
                status: VaultStatus::Pending,
            });
        }
    }

    /// Update upload progress for a file by index.
    pub fn set_progress(&self, idx: usize, progress: f32) {
        if let Ok(mut uploads) = self.uploads.lock() {
            if let Some(entry) = uploads.get_mut(idx) {
                entry.progress = progress.clamp(0.0, 1.0);
                if entry.status == VaultStatus::Pending {
                    entry.status = VaultStatus::Uploading;
                }
            }
        }
    }

    /// Mark a file upload as complete.
    pub fn set_complete(&self, idx: usize) {
        if let Ok(mut uploads) = self.uploads.lock() {
            if let Some(entry) = uploads.get_mut(idx) {
                entry.progress = 1.0;
                entry.status = VaultStatus::Complete;
            }
        }
    }

    /// Mark a file upload as failed.
    pub fn set_failed(&self, idx: usize, error: impl Into<String>) {
        if let Ok(mut uploads) = self.uploads.lock() {
            if let Some(entry) = uploads.get_mut(idx) {
                entry.status = VaultStatus::Failed(error.into());
            }
        }
    }
}

impl View for DropVault {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let is_drag = self
            .is_drag_over
            .lock()
            .map(|g| *g)
            .unwrap_or(false);

        // Drop zone background
        let bg = if is_drag {
            theme::active_color()
        } else {
            theme::surface()
        };
        let border_color = if is_drag {
            theme::accent()
        } else {
            theme::border()
        };

        renderer.fill_rounded_rect(rect, 8.0, bg);
        renderer.stroke_rounded_rect(rect, 8.0, border_color, 2.0);

        // Prompt text
        let prompt = if is_drag {
            "Drop files here"
        } else {
            "Drag files here or click to browse"
        };
        let (tw, _th) = renderer.measure_text(prompt, 14.0);
        renderer.draw_text(
            prompt,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + 20.0,
            14.0,
            theme::text(),
        );

        // Render upload entries
        let uploads = match self.uploads.lock() {
            Ok(g) => g.clone(),
            Err(_) => return,
        };

        let mut y = rect.y + 50.0;
        for entry in &uploads {
            let entry_rect = Rect::new(rect.x + 8.0, y, rect.width - 16.0, 50.0);
            render_upload_entry(renderer, entry, entry_rect);
            y += 58.0;
        }
    }
}

fn render_upload_entry(renderer: &mut dyn Renderer, entry: &VaultEntry, rect: Rect) {
    // File name + size
    let size_str = format_file_size(entry.file_info.size);
    let label = format!("{} ({})", entry.file_info.name, size_str);
    renderer.draw_text(&label, rect.x + 4.0, rect.y + 4.0, 12.0, theme::text());

    // Progress bar
    let bar_y = rect.y + 22.0;
    let bar_h = 8.0;
    let bar_rect = Rect::new(rect.x + 4.0, bar_y, rect.width - 8.0, bar_h);

    // Background
    renderer.fill_rounded_rect(bar_rect, 4.0, theme::surface_elevated());

    // Fill
    let fill_width = (bar_rect.width * entry.progress).max(0.0);
    if fill_width > 0.0 {
        let fill_rect = Rect::new(bar_rect.x, bar_y, fill_width, bar_h);
        let color = match &entry.status {
            VaultStatus::Complete => theme::success(),
            VaultStatus::Failed(_) => theme::error_color(),
            _ => theme::accent(),
        };
        renderer.fill_rounded_rect(fill_rect, 4.0, color);
    }

    // Status text
    let status_text = match &entry.status {
        VaultStatus::Pending => "Waiting...".to_string(),
        VaultStatus::Uploading => format!("{}%", (entry.progress * 100.0) as u32),
        VaultStatus::Complete => "Done".to_string(),
        VaultStatus::Failed(msg) => {
            let mut msg = msg.clone();
            if msg.len() > 30 {
                msg.truncate(30);
                msg.push_str("...");
            }
            format!("Error: {}", msg)
        }
    };
    renderer.draw_text(&status_text, rect.x + 4.0, rect.y + 38.0, 10.0, theme::text_muted());
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

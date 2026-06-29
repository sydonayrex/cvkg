use thiserror::Error;

#[derive(Error, Debug)]
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
                            tracing::info!("[AutoSaveManager] Auto-saved document to {:?}", path);
                        }
                        Err(e) => {
                            tracing::error!(
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

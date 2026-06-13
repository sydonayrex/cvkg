//! Dev Runtime Controller
//! Responsible for launching runtime, maintaining connection, and coordinating updates

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::patch_engine::{CompiledArtifact, PatchEngine, RuntimePatch};

/// Abstract runtime handle trait
pub trait RuntimeHandle: Send + Sync {
    /// Send a patch to the runtime
    fn send_patch(&self, patch: RuntimePatch);

    /// Request current state from the runtime
    fn request_state(&self) -> RuntimeStateSnapshot;

    /// Send an event to the runtime
    fn send_event(&self, event: RuntimeEvent);
}

/// DevRuntimeController manages the connection to the runtime
pub struct DevRuntimeController {
    runtime: Arc<dyn RuntimeHandle>,
    patch_engine: PatchEngine,
}

impl DevRuntimeController {
    /// Create a new DevRuntimeController
    pub fn new(runtime: Arc<dyn RuntimeHandle>) -> Self {
        Self {
            runtime,
            patch_engine: PatchEngine::new(),
        }
    }

    /// Apply a code update by generating and sending a patch
    pub fn apply_code_update(&mut self, compiled_artifact: CompiledArtifact) {
        let patch = self.patch_engine.generate_patch(compiled_artifact);
        self.runtime.send_patch(patch);
    }

    /// Inject an agent stream into the runtime
    pub fn inject_agent_stream(&self, stream: Vec<RuntimeEvent>) {
        for event in stream {
            self.runtime.send_event(event);
        }
    }
}

/// Runtime event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    Agent(AgentEvent),
    // Add other event types as needed
}

/// Agent event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Token(String),
    ToolCall(String),
    StateChange(String),
    Error(String),
}

/// Runtime state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateSnapshot {
    // In a full implementation, this would contain the serialized state graph
    pub data: String,
}

impl RuntimeStateSnapshot {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

// =============================================================================
// FILE WATCHER — Item 17: Hot Reload / Dev Server
// =============================================================================
// Uses `notify` crate (already a workspace dependency) for cross-platform
// file system event monitoring.

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// File watcher that monitors paths for changes and emits debounced events.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<Event>,
    debounce: Duration,
    last_event: Option<Instant>,
    pending_paths: Vec<PathBuf>,
}

impl FileWatcher {
    pub fn new(paths: Vec<PathBuf>) -> notify::Result<Self> {
        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            })?;

        for path in &paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
            }
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            debounce: Duration::from_millis(300),
            last_event: None,
            pending_paths: Vec::new(),
        })
    }

    /// Poll for file changes. Returns paths of changed files after debounce.
    pub fn poll_changes(&mut self) -> Vec<PathBuf> {
        // Drain all pending events
        while let Ok(event) = self.rx.try_recv() {
            self.last_event = Some(Instant::now());
            for path in event.paths {
                if !self.pending_paths.contains(&path) {
                    self.pending_paths.push(path);
                }
            }
        }

        // Return paths if debounce period has elapsed
        if let Some(last) = self.last_event
            && last.elapsed() >= self.debounce
            && !self.pending_paths.is_empty()
        {
            let paths = std::mem::take(&mut self.pending_paths);
            self.last_event = None;
            return paths;
        }

        Vec::new()
    }

    /// Check if any changes are pending (even if not yet debounced).
    pub fn has_pending_changes(&self) -> bool {
        !self.pending_paths.is_empty()
    }

    /// Get the debounce duration.
    pub fn debounce_duration(&self) -> Duration {
        self.debounce
    }
}

/// State snapshot for preserving app state across hot reloads.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotReloadState {
    /// Current theme mode ("dark" or "light").
    pub theme_mode: String,
    /// Window size (width, height).
    pub window_size: (f32, f32),
    /// Scroll positions per scroll view (key = view ID).
    pub scroll_positions: std::collections::HashMap<String, Vec<f32>>,
    /// Input text per text field (key = input ID).
    pub input_text: std::collections::HashMap<String, String>,
    /// Expanded nodes per outline view (key = view ID).
    pub expanded_nodes: std::collections::HashMap<String, Vec<usize>>,
    /// Timestamp of last save.
    pub saved_at: f64,
}

impl Default for HotReloadState {
    fn default() -> Self {
        Self {
            theme_mode: "dark".to_string(),
            window_size: (1200.0, 800.0),
            scroll_positions: std::collections::HashMap::new(),
            input_text: std::collections::HashMap::new(),
            expanded_nodes: std::collections::HashMap::new(),
            saved_at: 0.0,
        }
    }
}

impl HotReloadState {
    /// Save state to a JSON file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load state from a JSON file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}

/// Error overlay for showing compilation errors in the app.
#[derive(Clone, Debug)]
pub struct ErrorOverlay {
    /// Error message to display.
    pub message: String,
    /// Source file where the error occurred.
    pub file: Option<String>,
    /// Line number (1-indexed).
    pub line: Option<u32>,
    /// Column number (1-indexed).
    pub column: Option<u32>,
}

impl ErrorOverlay {
    /// Create a new error overlay from a cargo JSON error message.
    ///
    /// Parses cargo's `--message-format=json` output to extract structured
    /// error information (file, line, column, message).
    ///
    /// Falls back to naive line scanning if JSON parsing fails.
    pub fn from_cargo_output(output: &str) -> Option<Self> {
        // Try structured JSON parsing first
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                // Cargo JSON messages have a "reason" field
                if json.get("reason").and_then(|r| r.as_str()) == Some("compiler-message")
                    && let Some(message) = json.get("message")
                {
                    let msg_text = message
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Only actual errors, not warnings
                    let is_error = message
                        .get("level")
                        .and_then(|l| l.as_str())
                        .map(|l| l == "error")
                        .unwrap_or(false);
                    if !is_error {
                        continue;
                    }

                    // Extract file/line/column from spans
                    let (file, line, column) = message
                        .get("spans")
                        .and_then(|s| s.as_array())
                        .and_then(|spans| spans.first())
                        .map(|span| {
                            (
                                span.get("file_name")
                                    .and_then(|f| f.as_str())
                                    .map(String::from),
                                span.get("line_start")
                                    .and_then(|l| l.as_u64())
                                    .map(|l| l as u32),
                                span.get("column_start")
                                    .and_then(|c| c.as_u64())
                                    .map(|c| c as u32),
                            )
                        })
                        .unwrap_or((None, None, None));

                    return Some(Self {
                        message: msg_text,
                        file,
                        line,
                        column,
                    });
                }
            }
        }

        // Fallback: naive scan for lines containing "error[" or "error:"
        for line in output.lines() {
            let lower = line.to_lowercase();
            if (lower.contains("error[") || lower.contains("error:"))
                && !lower.contains("error-handling")
            {
                return Some(Self {
                    message: line.to_string(),
                    file: None,
                    line: None,
                    column: None,
                });
            }
        }

        None
    }
}

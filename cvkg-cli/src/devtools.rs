//! DevTools Dashboard Module
//!
//! Provides an in-process developer tools dashboard for inspecting and
//! debugging CVKG applications at runtime. Includes panels for graph
//! visualization, node inspection, performance metrics, log viewing, and
//! theme editing.

use std::time::{SystemTime, UNIX_EPOCH};

/// The content type rendered inside a dashboard panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelContent {
    /// Interactive graph visualization of the scene tree.
    GraphView,
    /// Inspector panel showing properties of the selected node.
    NodeInspector,
    /// Real-time performance metrics overlay.
    PerformanceMetrics,
    /// Scrollable log viewer.
    LogView,
    /// Live theme editor with color pickers.
    ThemeEditor,
}

/// A single panel in the DevTools dashboard.
#[derive(Debug, Clone)]
pub struct Panel {
    /// The display title of the panel tab.
    pub title: String,
    /// The type of content this panel renders.
    pub content: PanelContent,
    /// The panel width as a fraction of the dashboard (0.0–1.0).
    pub width: f32,
    /// The panel height as a fraction of the dashboard (0.0–1.0).
    pub height: f32,
}

impl Panel {
    /// Create a new [`Panel`] with the given title and content type.
    ///
    /// # Arguments
    ///
    /// * `title` — The tab title displayed in the dashboard.
    /// * `content` — The [`PanelContent`] variant to render.
    pub fn new(title: &str, content: PanelContent) -> Self {
        Self {
            title: title.to_string(),
            content,
            width: 1.0,
            height: 1.0,
        }
    }

    /// Set the panel dimensions as fractions of the dashboard size.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width.clamp(0.0, 1.0);
        self.height = height.clamp(0.0, 1.0);
        self
    }
}

/// Performance metrics captured from the running application.
#[derive(Debug, Clone)]
pub struct PerfMetrics {
    /// Time in milliseconds to render the last frame.
    pub frame_time_ms: f32,
    /// Frames per second.
    pub fps: f32,
    /// Number of nodes in the current scene graph.
    pub node_count: usize,
    /// Number of edges (relationships) in the current scene graph.
    pub edge_count: usize,
    /// Estimated GPU memory usage in megabytes.
    pub gpu_memory_mb: f32,
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            fps: 0.0,
            node_count: 0,
            edge_count: 0,
            gpu_memory_mb: 0.0,
        }
    }
}

/// Severity level for a log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug-level message.
    Debug,
    /// Informational message.
    Info,
    /// Warning message.
    Warn,
    /// Error message.
    Error,
}

/// A single log entry in the DevTools log view.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// ISO-8601 formatted timestamp string.
    pub timestamp: String,
    /// The severity level of this entry.
    pub level: LogLevel,
    /// The log message body.
    pub message: String,
}

/// Widget primitives rendered by the DevTools dashboard.
///
/// Each variant represents a distinct UI element that can be drawn
/// by the rendering backend.
#[derive(Debug, Clone)]
pub enum DevToolWidget {
    /// A text label with RGBA color.
    Text {
        /// The text content to display.
        content: String,
        /// RGBA color values each in the range 0.0–1.0.
        color: [f32; 4],
    },
    /// A line graph plotting the given data points.
    Graph {
        /// Y-axis data values.
        data: Vec<f32>,
        /// Label displayed below the graph.
        label: String,
    },
    /// A key-value property inspector.
    Inspector {
        /// Pairs of (property_name, value).
        properties: Vec<(String, String)>,
    },
    /// A clickable button.
    Button {
        /// The button label.
        label: String,
        /// Whether the button was clicked this frame.
        clicked: bool,
    },
}

/// The DevTools dashboard managing panels and rendering widgets.
#[derive(Debug, Clone)]
pub struct DevToolsDashboard {
    /// The panels currently open in the dashboard.
    pub panels: Vec<Panel>,
    /// Index of the currently active (selected) panel.
    pub active_panel: usize,
    /// Whether the dashboard overlay is visible.
    pub visible: bool,
}

impl Default for DevToolsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl DevToolsDashboard {
    /// Create a new [`DevToolsDashboard`] with default panels.
    ///
    /// The dashboard starts hidden with a Performance Metrics panel
    /// and a Log View panel pre-configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use cvkg_cli::devtools::DevToolsDashboard;
    /// let dashboard = DevToolsDashboard::new();
    /// assert!(!dashboard.visible);
    /// assert_eq!(dashboard.panels.len(), 2);
    /// ```
    pub fn new() -> Self {
        Self {
            panels: vec![
                Panel::new("Performance", PanelContent::PerformanceMetrics),
                Panel::new("Logs", PanelContent::LogView),
            ],
            active_panel: 0,
            visible: false,
        }
    }

    /// Toggle the dashboard visibility on or off.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Add a new panel to the dashboard.
    ///
    /// # Arguments
    ///
    /// * `panel` — The [`Panel`] to add.
    pub fn add_panel(&mut self, panel: Panel) {
        self.panels.push(panel);
    }

    /// Remove the panel at the given index.
    ///
    /// If the index is out of bounds, this is a no-op. After removal,
    /// the active panel index is clamped to the new length.
    ///
    /// # Arguments
    ///
    /// * `index` — The zero-based index of the panel to remove.
    pub fn remove_panel(&mut self, index: usize) {
        if index < self.panels.len() {
            self.panels.remove(index);
            if self.active_panel >= self.panels.len() && !self.panels.is_empty() {
                self.active_panel = self.panels.len() - 1;
            }
        }
    }

    /// Set the active (selected) panel by index.
    ///
    /// If the index is out of bounds, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `index` — The zero-based index of the panel to activate.
    pub fn set_active(&mut self, index: usize) {
        if index < self.panels.len() {
            self.active_panel = index;
        }
    }

    /// Render the dashboard into a list of [`DevToolWidget`] primitives.
    ///
    /// Only the active panel is rendered. If the dashboard is not visible
    /// or has no panels, an empty Vec is returned.
    pub fn render(&self) -> Vec<DevToolWidget> {
        if !self.visible || self.panels.is_empty() {
            return Vec::new();
        }

        let mut widgets = Vec::new();

        // Render tab bar
        for (i, panel) in self.panels.iter().enumerate() {
            let color = if i == self.active_panel {
                [0.0, 1.0, 1.0, 1.0]
            } else {
                [0.5, 0.5, 0.5, 1.0]
            };
            widgets.push(DevToolWidget::Button {
                label: panel.title.clone(),
                clicked: false,
            });
            widgets.push(DevToolWidget::Text {
                content: panel.title.clone(),
                color,
            });
        }

        // Render active panel content
        if let Some(panel) = self.panels.get(self.active_panel) {
            match &panel.content {
                PanelContent::PerformanceMetrics => {
                    let metrics = capture_metrics();
                    widgets.push(DevToolWidget::Text {
                        content: format!("FPS: {:.1}", metrics.fps),
                        color: [0.0, 1.0, 0.0, 1.0],
                    });
                    widgets.push(DevToolWidget::Text {
                        content: format!("Frame Time: {:.2} ms", metrics.frame_time_ms),
                        color: [1.0, 1.0, 0.0, 1.0],
                    });
                    widgets.push(DevToolWidget::Text {
                        content: format!("Nodes: {}", metrics.node_count),
                        color: [1.0, 1.0, 1.0, 1.0],
                    });
                    widgets.push(DevToolWidget::Text {
                        content: format!("Edges: {}", metrics.edge_count),
                        color: [1.0, 1.0, 1.0, 1.0],
                    });
                    widgets.push(DevToolWidget::Text {
                        content: format!("GPU Memory: {:.1} MB", metrics.gpu_memory_mb),
                        color: [1.0, 0.5, 0.0, 1.0],
                    });
                    widgets.push(DevToolWidget::Graph {
                        data: vec![metrics.frame_time_ms],
                        label: "Frame Time (ms)".to_string(),
                    });
                }
                PanelContent::NodeInspector => {
                    widgets.push(DevToolWidget::Inspector {
                        properties: vec![
                            ("type".to_string(), "Node".to_string()),
                            ("id".to_string(), "0".to_string()),
                        ],
                    });
                }
                PanelContent::GraphView => {
                    widgets.push(DevToolWidget::Text {
                        content: "Graph View".to_string(),
                        color: [0.0, 1.0, 1.0, 1.0],
                    });
                }
                PanelContent::LogView => {
                    widgets.push(DevToolWidget::Text {
                        content: "Log View — No entries".to_string(),
                        color: [0.7, 0.7, 0.7, 1.0],
                    });
                }
                PanelContent::ThemeEditor => {
                    widgets.push(DevToolWidget::Text {
                        content: "Theme Editor".to_string(),
                        color: [1.0, 0.0, 1.0, 1.0],
                    });
                }
            }
        }

        widgets
    }
}

use std::sync::RwLock;

/// Global metrics store updated by the dev server.
static METRICS: RwLock<PerfMetrics> = RwLock::new(PerfMetrics {
    frame_time_ms: 0.0,
    fps: 0.0,
    node_count: 0,
    edge_count: 0,
    gpu_memory_mb: 0.0,
});

/// Capture current performance metrics.
///
/// Returns live metrics populated by the dev server, or default zeros
/// if the server is not running or metrics have not been set.
///
/// # Examples
///
/// ```
/// use cvkg_cli::devtools::capture_metrics;
/// let metrics = capture_metrics();
/// // Default metrics should have zero values
/// assert_eq!(metrics.fps, 0.0);
/// assert_eq!(metrics.node_count, 0);
/// ```
pub fn capture_metrics() -> PerfMetrics {
    METRICS.read().map(|m| m.clone()).unwrap_or_default()
}

/// Update the global metrics store with new values.
///
/// Called by the dev server or build pipeline to publish live metrics.
pub fn update_metrics(metrics: PerfMetrics) {
    if let Ok(mut m) = METRICS.write() {
        *m = metrics;
    }
}

/// Format a [`LogEntry`] into a human-readable string.
///
/// The output format is: `[TIMESTAMP] LEVEL: message`
///
/// # Arguments
///
/// * `entry` — The [`LogEntry`] to format.
///
/// # Examples
///
/// ```
/// use cvkg_cli::devtools::{LogEntry, LogLevel, format_log_entry};
/// let entry = LogEntry {
///     timestamp: "2025-01-01T00:00:00Z".to_string(),
///     level: LogLevel::Info,
///     message: "Application started".to_string(),
/// };
/// let formatted = format_log_entry(&entry);
/// assert_eq!(formatted, "[2025-01-01T00:00:00Z] INFO: Application started");
/// ```
pub fn format_log_entry(entry: &LogEntry) -> String {
    let level_str = match entry.level {
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };
    format!("[{}] {}: {}", entry.timestamp, level_str, entry.message)
}

/// Create a timestamp string suitable for [`LogEntry::timestamp`].
///
/// Returns the current system time formatted as a Unix timestamp string.
pub fn current_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_new() {
        let dashboard = DevToolsDashboard::new();
        assert!(!dashboard.visible);
        assert_eq!(dashboard.panels.len(), 2);
        assert_eq!(dashboard.active_panel, 0);
        assert_eq!(dashboard.panels[0].title, "Performance");
        assert_eq!(dashboard.panels[1].title, "Logs");
    }

    #[test]
    fn test_dashboard_toggle() {
        let mut dashboard = DevToolsDashboard::new();
        assert!(!dashboard.visible);
        dashboard.toggle();
        assert!(dashboard.visible);
        dashboard.toggle();
        assert!(!dashboard.visible);
    }

    #[test]
    fn test_dashboard_add_panel() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.add_panel(Panel::new("Inspector", PanelContent::NodeInspector));
        assert_eq!(dashboard.panels.len(), 3);
        assert_eq!(dashboard.panels[2].title, "Inspector");
        assert_eq!(dashboard.panels[2].content, PanelContent::NodeInspector);
    }

    #[test]
    fn test_dashboard_remove_panel() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.add_panel(Panel::new("Extra", PanelContent::GraphView));
        assert_eq!(dashboard.panels.len(), 3);
        dashboard.remove_panel(2);
        assert_eq!(dashboard.panels.len(), 2);
    }

    #[test]
    fn test_dashboard_remove_panel_out_of_bounds() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.remove_panel(99);
        assert_eq!(dashboard.panels.len(), 2);
    }

    #[test]
    fn test_dashboard_remove_active_panel_clamps() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.set_active(1);
        assert_eq!(dashboard.active_panel, 1);
        dashboard.remove_panel(1);
        // Active should clamp to last valid index
        assert_eq!(dashboard.active_panel, 0);
    }

    #[test]
    fn test_dashboard_set_active() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.add_panel(Panel::new("Third", PanelContent::ThemeEditor));
        dashboard.set_active(2);
        assert_eq!(dashboard.active_panel, 2);
    }

    #[test]
    fn test_dashboard_set_active_out_of_bounds() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.set_active(99);
        assert_eq!(dashboard.active_panel, 0);
    }

    #[test]
    fn test_dashboard_render_hidden() {
        let dashboard = DevToolsDashboard::new();
        let widgets = dashboard.render();
        assert!(widgets.is_empty());
    }

    #[test]
    fn test_dashboard_render_visible() {
        let mut dashboard = DevToolsDashboard::new();
        dashboard.toggle();
        let widgets = dashboard.render();
        // Should have tab buttons/text + panel content widgets
        assert!(!widgets.is_empty());
    }

    #[test]
    fn test_panel_new() {
        let panel = Panel::new("Test", PanelContent::GraphView);
        assert_eq!(panel.title, "Test");
        assert_eq!(panel.content, PanelContent::GraphView);
        assert_eq!(panel.width, 1.0);
        assert_eq!(panel.height, 1.0);
    }

    #[test]
    fn test_panel_with_size() {
        let panel = Panel::new("Sized", PanelContent::LogView).with_size(0.5, 0.75);
        assert_eq!(panel.width, 0.5);
        assert_eq!(panel.height, 0.75);
    }

    #[test]
    fn test_panel_with_size_clamped() {
        let panel = Panel::new("Clamped", PanelContent::LogView).with_size(1.5, -0.5);
        assert_eq!(panel.width, 1.0);
        assert_eq!(panel.height, 0.0);
    }

    #[test]
    fn test_capture_metrics() {
        let metrics = capture_metrics();
        assert_eq!(metrics.frame_time_ms, 0.0);
        assert_eq!(metrics.fps, 0.0);
        assert_eq!(metrics.node_count, 0);
        assert_eq!(metrics.edge_count, 0);
        assert_eq!(metrics.gpu_memory_mb, 0.0);
    }

    #[test]
    fn test_format_log_entry_info() {
        let entry = LogEntry {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            level: LogLevel::Info,
            message: "Application started".to_string(),
        };
        assert_eq!(
            format_log_entry(&entry),
            "[2025-01-01T00:00:00Z] INFO: Application started"
        );
    }

    #[test]
    fn test_format_log_entry_debug() {
        let entry = LogEntry {
            timestamp: "T1".to_string(),
            level: LogLevel::Debug,
            message: "debug msg".to_string(),
        };
        assert_eq!(format_log_entry(&entry), "[T1] DEBUG: debug msg");
    }

    #[test]
    fn test_format_log_entry_warn() {
        let entry = LogEntry {
            timestamp: "T2".to_string(),
            level: LogLevel::Warn,
            message: "watch out".to_string(),
        };
        assert_eq!(format_log_entry(&entry), "[T2] WARN: watch out");
    }

    #[test]
    fn test_format_log_entry_error() {
        let entry = LogEntry {
            timestamp: "T3".to_string(),
            level: LogLevel::Error,
            message: "something broke".to_string(),
        };
        assert_eq!(format_log_entry(&entry), "[T3] ERROR: something broke");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_perf_metrics_default() {
        let m = PerfMetrics::default();
        assert_eq!(m.frame_time_ms, 0.0);
        assert_eq!(m.fps, 0.0);
        assert_eq!(m.node_count, 0);
        assert_eq!(m.edge_count, 0);
        assert_eq!(m.gpu_memory_mb, 0.0);
    }

    #[test]
    fn test_dev_tool_widget_variants() {
        let text = DevToolWidget::Text {
            content: "hello".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let graph = DevToolWidget::Graph {
            data: vec![1.0, 2.0],
            label: "test".to_string(),
        };
        let inspector = DevToolWidget::Inspector {
            properties: vec![("k".to_string(), "v".to_string())],
        };
        let button = DevToolWidget::Button {
            label: "click".to_string(),
            clicked: false,
        };
        // Just verify they construct without panic
        drop(text);
        drop(graph);
        drop(inspector);
        drop(button);
    }

    #[test]
    fn test_current_timestamp_nonzero() {
        let ts = current_timestamp();
        assert!(!ts.is_empty());
        assert!(ts.parse::<u64>().is_ok());
    }
}

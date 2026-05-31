pub mod agent_replay;
pub mod asset_pipeline;
pub mod build_pipeline;
pub mod config;
pub mod dev_runtime;
pub mod devtools;
pub mod devtools_dashboard;
pub mod error;
pub mod native_shell;
pub mod patch_engine;
pub mod plugin;
pub mod runtime_connection;
pub mod scaffold;
pub mod token_export;
pub mod webkit_server;
pub mod ws_server;

// Public re-exports for downstream consumers
pub use config::CliConfig;
pub use devtools::{
    DevToolWidget, DevToolsDashboard, LogEntry, LogLevel, Panel, PanelContent, PerfMetrics,
    capture_metrics, current_timestamp, format_log_entry, update_metrics,
};
pub use error::{CliError, exit_with_error};
pub use native_shell::{
    NativeShell, ShellBackend, ShellError, ShellWindow, WindowEvent, create_window, poll_events,
};
pub use scaffold::{Scaffolder, Template};
pub use token_export::TokenExport;
pub use ws_server::{
    AppState, DevtoolsCommand, DevtoolsMessage, WsMessage, create_router, start_file_watcher,
    start_server,
};

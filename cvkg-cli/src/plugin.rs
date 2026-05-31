//! Plugin system for extending the CVKG CLI.
//!
//! Plugins can register custom commands, build steps, and asset processors.
//!
//! # Example
//!
//! ```ignore
//! use cvkg_cli::plugin::{Plugin, PluginContext, CommandResult};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn register(&self, ctx: &mut PluginContext) {
//!         ctx.register_command("my-cmd", |args| {
//!             println!("Hello from plugin!");
//!             CommandResult::Ok
//!         });
//!     }
//! }
//! ```

/// Result of a plugin command execution.
pub enum CommandResult {
    /// Command executed successfully.
    Ok,
    /// Command failed with an error message.
    Error(String),
}

/// Context passed to plugins during registration.
pub trait PluginContext {
    /// Register a custom command.
    fn register_command(&mut self, name: &str, handler: fn(args: &[String]) -> CommandResult);
    /// Register a build step that runs after the default build.
    fn register_build_step(&mut self, name: &str, handler: fn() -> anyhow::Result<()>);
    /// Register an asset processor for a given file extension.
    fn register_asset_processor(
        &mut self,
        extension: &str,
        handler: fn(path: &std::path::Path) -> anyhow::Result<()>,
    );
}

/// Plugin trait for extending the CVKG CLI.
///
/// Implement this trait to add custom commands, build steps, or asset processors.
pub trait Plugin: Send + Sync {
    /// Unique name for this plugin.
    fn name(&self) -> &str;
    /// Called during CLI initialization to register commands and hooks.
    fn register(&self, ctx: &mut dyn PluginContext);
    /// Called before the CLI shuts down.
    fn shutdown(&self) {}
}

/// Registry for loaded plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin.
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        log::info!("Registering plugin: {}", plugin.name());
        self.plugins.push(Box::new(plugin));
    }

    /// Get the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if no plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//! Configuration file support for the CVKG CLI.
//!
//! Reads settings from `.cvkg.toml` (or paths specified by `CVKG_CONFIG` env var)
//! and merges them with CLI flags (CLI > env > file).

use serde::Deserialize;
use std::path::PathBuf;
use tracing::{info, warn};

/// CLI configuration that can be set via config file, env vars, or CLI flags.
#[derive(Debug, Default, Deserialize)]
pub struct CliConfig {
    /// Default target platform.
    pub target: Option<String>,
    /// Default dev server port.
    pub port: Option<u16>,
    /// Default asset directory.
    pub assets_dir: Option<String>,
    /// Default output directory for builds.
    pub dist_dir: Option<String>,
    /// Enable inspector by default.
    pub inspector: Option<bool>,
    /// Respect reduced-motion preference.
    pub reduced_motion: Option<bool>,
}

impl CliConfig {
    /// Load configuration from `.cvkg.toml` in the current directory,
    /// or from the path specified by `CVKG_CONFIG` env var.
    pub fn load() -> Self {
        let config_path = std::env::var("CVKG_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".cvkg.toml"));

        if !config_path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str::<CliConfig>(&content) {
                Ok(config) => {
                    info!("Loaded config from {:?}", config_path);
                    config
                }
                Err(e) => {
                    warn!("Failed to parse {:?}: {}", config_path, e);
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {:?}: {}", config_path, e);
                Self::default()
            }
        }
    }

    /// Merge CLI flags into config. CLI flags take precedence over config file values.
    pub fn merge_cli(
        &mut self,
        target: Option<String>,
        port: Option<u16>,
        inspector: bool,
        reduced_motion: bool,
    ) {
        if target.is_some() {
            self.target = target;
        }
        if port.is_some() {
            self.port = port;
        }
        // Inspector and reduced_motion are boolean flags — CLI always overrides
        self.inspector = Some(inspector);
        self.reduced_motion = Some(reduced_motion);
    }
}

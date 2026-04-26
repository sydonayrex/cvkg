//! Build Pipeline Hook
//! Hooks into the build process for incremental rebuilds

use std::path::Path;
use std::time::{Duration, Instant};

use super::patch_engine::CompiledArtifact;

/// Build pipeline hook
pub struct BuildPipeline;

impl BuildPipeline {
    /// Create a new BuildPipeline
    pub fn new() -> Self {
        Self
    }

    /// Compile the project using cargo build with support for targets and features.
    pub fn compile_project<P: AsRef<Path>>(
        project_path: P,
        target: Option<&str>,
        release: bool,
        features: &[String],
    ) -> CompiledArtifact {
        use indicatif::{ProgressBar, ProgressStyle};
        use console::style;

        // Clear terminal for a fresh build output
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{} CVKG Forge: Rebuilding project...", style("🚀").cyan());
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap());
        pb.set_message(format!("Compiling target..."));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let start_time = Instant::now();
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("build");

        if let Some(t) = target {
            cmd.arg("--target").arg(t);
        }

        if release {
            cmd.arg("--release");
        }

        if !features.is_empty() {
            cmd.arg("--features").arg(features.join(","));
        }

        // We want to see colored output from cargo
        cmd.env("CARGO_TERM_COLOR", "always");

        let output = cmd.current_dir(project_path.as_ref()).output();

        match output {
            Ok(out) if out.status.success() => {
                let duration = start_time.elapsed();
                pb.finish_with_message(format!("{} Build Success in {:.2?}", style("✅").green(), duration));
                CompiledArtifact {
                    root_id: 1,
                    view: super::patch_engine::SerializedView {
                        view_type: "App".to_string(),
                        props: serde_json::json!({ "status": "built", "target": target, "release": release }),
                        children: Vec::new(),
                    },
                }
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                pb.finish_with_message(format!("{} Build Failed", style("❌").red()));
                println!("\n{}", style(stderr).red());
                CompiledArtifact {
                    root_id: 0,
                    view: super::patch_engine::SerializedView {
                        view_type: "Error".to_string(),
                        props: serde_json::json!({ "message": String::from_utf8_lossy(&out.stderr).into_owned() }),
                        children: Vec::new(),
                    },
                }
            }
            Err(e) => {
                pb.finish_with_message(format!("{} Failed to execute cargo: {}", style("💥").red(), e));
                CompiledArtifact {
                    root_id: 0,
                    view: super::patch_engine::SerializedView {
                        view_type: "FatalError".to_string(),
                        props: serde_json::json!({ "message": e.to_string() }),
                        children: Vec::new(),
                    },
                }
            }
        }
    }

    /// Watch for file changes and trigger incremental rebuilds with debouncing
    pub fn watch_changes<P: AsRef<Path>, F>(project_path: P, callback: F)
    where
        F: FnMut(CompiledArtifact) + Send + 'static,
    {
        use notify::{Config, RecursiveMode, Watcher};
        use std::sync::{Arc, Mutex};
        use std::sync::mpsc::RecvTimeoutError;

        let path = project_path.as_ref().to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();
        let callback = Arc::new(Mutex::new(callback));

        let mut watcher = match notify::RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Failed to create watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&path, RecursiveMode::Recursive) {
            log::error!("Failed to watch path {:?}: {}", path, e);
            return;
        }

        std::thread::spawn(move || {
            let _watcher = watcher;
            println!("{} CVKG Hot-Reload Engine watching for changes...", console::style("👀").cyan());

            let debounce_duration = Duration::from_millis(300);
            let mut pending_build = false;

            loop {
                let event_result = if pending_build {
                    rx.recv_timeout(debounce_duration)
                } else {
                    rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
                };

                match event_result {
                    Ok(Ok(event)) => {
                        // Filter events
                        let is_relevant = event.paths.iter().any(|p| {
                            // Ignore target directory and git
                            if p.components().any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git") {
                                return false;
                            }
                            
                            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                            matches!(ext, "rs" | "wgsl" | "toml" | "json")
                        });

                        if event.kind.is_modify() && is_relevant {
                            pending_build = true;
                        }
                    }
                    Ok(Err(e)) => log::error!("Watch error: {:?}", e),
                    Err(RecvTimeoutError::Timeout) => {
                        if pending_build {
                            pending_build = false;
                            let artifact = Self::compile_project(&path, None, false, &[]);
                            let mut cb = callback.lock().unwrap();
                            (cb)(artifact);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        log::info!("Watcher disconnected, stopping hot-reload engine.");
                        break;
                    }
                }
            }
        });
    }
}

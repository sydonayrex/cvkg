//! Build Pipeline Hook
//! Hooks into the build process for incremental rebuilds

use std::path::Path;

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
        log::info!(
            "Starting CVKG Forge Build: {}",
            project_path.as_ref().display()
        );

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

        let output = cmd.current_dir(project_path.as_ref()).output();

        match output {
            Ok(out) if out.status.success() => {
                log::info!("CVKG Build Success");
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
                log::error!("CVKG Build Failed: {}", stderr);
                CompiledArtifact {
                    root_id: 0,
                    view: super::patch_engine::SerializedView {
                        view_type: "Error".to_string(),
                        props: serde_json::json!({ "message": stderr }),
                        children: Vec::new(),
                    },
                }
            }
            Err(e) => {
                log::error!("Failed to execute cargo: {}", e);
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

    /// Watch for file changes and trigger incremental rebuilds
    pub fn watch_changes<P: AsRef<Path>, F>(project_path: P, callback: F)
    where
        F: FnMut(CompiledArtifact) + Send + 'static,
    {
        use notify::{Config, RecursiveMode, Watcher};
        use std::sync::{Arc, Mutex};

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

        // Keep the watcher alive in the thread
        std::thread::spawn(move || {
            let _watcher = watcher;
            log::info!("Watching for changes in {:?}", path);

            for res in rx {
                match res {
                    Ok(event) => {
                        // Filter for relevant file modifications (.rs, .wgsl)
                        let is_relevant = event.paths.iter().any(|p| {
                            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                            ext == "rs" || ext == "wgsl"
                        });

                        if event.kind.is_modify() && is_relevant {
                            log::info!("File change detected, rebuilding...");
                            let artifact = Self::compile_project(&path, None, false, &[]);
                            let mut cb = callback.lock().unwrap();
                            (cb)(artifact);
                        }
                    }
                    Err(e) => log::error!("Watch error: {:?}", e),
                }
            }
        });
    }
}

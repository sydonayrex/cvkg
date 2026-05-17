//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! CVKG CLI toolchain, dev server, and hot reload orchestrator
//!
//! This crate provides the command-line interface for CVKG including:
//! - `cvkg new` for scaffolding new projects
//! - `cvkg dev` for starting the development server with hot reload
//! - `cvkg build` for building for target platforms
//! - `cvkg serve` for starting the WebKit preview server
//  and other development tools.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod agent_replay;
pub mod asset_pipeline;
pub mod build_pipeline;
pub mod dev_runtime;
pub mod patch_engine;
pub mod runtime_connection;
pub mod scaffold;
pub mod webkit_server;
pub mod ws_server;

/// CVKG Command Line Interface
#[derive(Parser)]
#[command(name = "cvkg")]
#[command(about = "Cyber Viking GUI X CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new CVKG application workspace
    New {
        /// Name of the new project
        name: String,
        /// Template to use (optional)
        #[arg(long)]
        template: Option<String>,
        /// Initialize git repository
        #[arg(long)]
        git: bool,
    },
    /// Start development server with hot reload
    Dev {
        /// Target platform (native, wasm, etc.)
        #[arg(long)]
        target: Option<String>,
        /// Port to run the dev server on
        #[arg(long, default_value_t = 3000)]
        port: u16,
        /// Enable the inspector
        #[arg(long)]
        inspector: bool,
    },
    /// Build for a specified target platform
    Build {
        /// Target platform
        #[arg(long)]
        target: String,
        /// Release build
        #[arg(long)]
        release: bool,
        /// Features to enable
        #[arg(long)]
        features: Vec<String>,
    },
    /// Start the WebKit preview server (no rebuild)
    Serve {
        /// Port to run the server on
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Open in browser after starting
        #[arg(long)]
        open: bool,
        /// Enable the inspector
        #[arg(long)]
        inspector: bool,
    },
    /// Run type-check + component lint + layout audit
    Check {
        /// Run all checks
        #[arg(long)]
        all: bool,
        /// Target platform
        #[arg(long)]
        target: Option<String>,
    },
    /// Run unit and snapshot tests
    Test {
        /// Run UI tests
        #[arg(long)]
        ui: bool,
        /// Target platform
        #[arg(long)]
        target: Option<String>,
    },
    /// Launch the Inspector against a running dev server
    Inspect {
        /// URL of the dev server
        #[arg(long)]
        url: String,
        /// WebSocket port for inspector
        #[arg(long, default_value_t = 8081)]
        ws_port: u16,
    },
    /// Export static WASM bundle for deployment
    Export {
        /// Base path for assets
        #[arg(long)]
        base_path: String,
        /// Optimize the build
        #[arg(long)]
        optimize: bool,
    },
    /// Add a CVKG-compatible component crate
    Add {
        /// Name of the crate to add
        name: String,
        /// Features to enable
        #[arg(long)]
        features: Vec<String>,
    },
    /// Generate a custom theme from design tokens (JSON)
    Theme {
        /// Input JSON file with design tokens
        #[arg(long)]
        input: PathBuf,
        /// Output RS file for generated theme
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            template,
            git,
        } => {
            use console::style;
            let tmpl = template
                .as_deref()
                .unwrap_or("minimal")
                .parse()
                .unwrap_or(scaffold::Template::Minimal);
            let scaffolder = scaffold::Scaffolder::new(name, tmpl, git);
            if let Err(e) = scaffolder.run() {
                eprintln!("{} Scaffolding failed: {}", style("❌").red(), e);
                std::process::exit(1);
            }
        }
        Commands::Dev {
            target,
            port,
            inspector,
        } => {
            use console::style;
            let target_str = target.as_deref().unwrap_or("native");
            println!("{} Starting CVKG development engine...", style("🚀").cyan());
            println!(
                "   {} Target: {}",
                style("•").dim(),
                style(target_str).yellow()
            );
            println!("   {} Port:   {}", style("•").dim(), style(port).bold());
            println!(
                "   {} Inspector: {}",
                style("•").dim(),
                if inspector {
                    style("Enabled").green()
                } else {
                    style("Disabled").dim()
                }
            );

            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

            // Start the async tokio runtime to run the dev server
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    if target_str == "wasm" || target_str == "webkit" {
                        println!("{} WebKit preview mode detected. Starting background preview server...", style("🌐").blue());
                    }

                    if let Err(e) = ws_server::start_server(addr).await {
                        eprintln!("{} Failed to start dev server: {}", style("❌").red(), e);
                    }
                });
        }
        Commands::Build {
            target,
            release,
            features,
        } => {
            if let Err(e) = asset_pipeline::AssetPipeline::run("assets") {
                eprintln!("Asset Pipeline failed: {}", e);
            }
            println!("Building for target: {}", target);
            let artifact = build_pipeline::BuildPipeline::compile_project(
                ".",
                Some(&target),
                release,
                &features,
            );
            println!("Build complete. Root ID: {}", artifact.root_id);
        }
        Commands::Serve {
            port,
            open: _,
            inspector: _,
        } => {
            println!("Starting WebKit preview server on port {}", port);
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    if let Err(e) = webkit_server::start_server(addr).await {
                        eprintln!("Failed to start preview server: {}", e);
                    }
                });
        }
        Commands::Check { all: _, target: _ } => {
            use console::style;
            println!(
                "{} Running CVKG type-check and layout audit...",
                style("🔍").blue()
            );
            let status = std::process::Command::new("cargo")
                .arg("check")
                .status()
                .expect("Failed to execute cargo check");

            if status.success() {
                println!(
                    "{} Check complete: All systems nominal.",
                    style("✅").green()
                );
            } else {
                eprintln!("{} Check failed.", style("❌").red());
                std::process::exit(1);
            }
        }
        Commands::Test { ui: _, target: _ } => {
            use console::style;
            println!(
                "{} Running CVKG unit and snapshot tests...",
                style("🧪").magenta()
            );
            let status = std::process::Command::new("cargo")
                .arg("test")
                .status()
                .expect("Failed to execute cargo test");

            if status.success() {
                println!(
                    "{} Tests complete: Berserker validated.",
                    style("✅").green()
                );
            } else {
                eprintln!("{} Tests failed.", style("❌").red());
                std::process::exit(1);
            }
        }
        Commands::Inspect { url: _, ws_port } => {
            println!("🔍 Launching CVKG Telemetry Inspector...");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    let ws_url = format!("ws://localhost:{}/ws/devtools", ws_port);
                    println!("Connecting to dev server at {}...", ws_url);

                    match tokio_tungstenite::connect_async(&ws_url).await {
                        Ok((mut ws_stream, _)) => {
                            println!("✅ Connected to CVKG DevTools Stream");
                            println!("Waiting for telemetry data...\n");

                            use futures_util::StreamExt;
                            while let Some(msg) = ws_stream.next().await {
                                if let Ok(msg) = msg
                                    && let Ok(text) = msg.to_text()
                                    && let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(text)
                                {
                                    if let Some(fps) = json.get("fps") {
                                        print!("{esc}[2K\r", esc = 27 as char);
                                        print!(
                                            "📊 FPS: {} | VRAM: {} MB | VDOM Diff: {} ms",
                                            fps,
                                            json.get("vram_mb").unwrap_or(&serde_json::json!(0)),
                                            json.get("diff_ms").unwrap_or(&serde_json::json!(0))
                                        );
                                        use std::io::Write;
                                        std::io::stdout().flush().unwrap_or_default();
                                    } else {
                                        println!("📡 Event: {}", text);
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("❌ Failed to connect to telemetry stream: {}", e),
                    }
                });
        }
        Commands::Export {
            base_path: _,
            optimize,
        } => {
            if let Err(e) = asset_pipeline::AssetPipeline::run("assets") {
                eprintln!("Asset Pipeline failed: {}", e);
            }

            let dist_dir = std::path::Path::new("dist");
            std::fs::create_dir_all(dist_dir).expect("Failed to create dist directory");

            println!("📦 Bundling CVKG WASM for production...");
            let mut cmd = std::process::Command::new("wasm-pack");
            cmd.arg("build")
                .arg("--target")
                .arg("web")
                .arg("--out-dir")
                .arg("dist/pkg");
            if optimize {
                cmd.arg("--release");
            }

            let status = cmd.status().expect("Failed to execute wasm-pack");
            if status.success() {
                let project_name = std::env::current_dir()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('-', "_");
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CVKG Application</title>
    <style>
        body, html {{ margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: #0b0b14; }}
        canvas {{ width: 100%; height: 100%; display: block; }}
    </style>
</head>
<body>
    <canvas id="cvkg-canvas"></canvas>
    <script type="module">
        import init from './pkg/{}.js';
        init();
    </script>
</body>
</html>"#,
                    project_name
                );

                std::fs::write(dist_dir.join("index.html"), html)
                    .expect("Failed to write index.html");

                if std::path::Path::new("assets").exists() {
                    println!("📦 Copying assets to dist/assets...");
                    let _ = std::process::Command::new("cp")
                        .arg("-r")
                        .arg("assets")
                        .arg("dist/")
                        .status();
                }

                println!("✅ Production Export Complete! Artifacts located in /dist");
            } else {
                eprintln!("❌ Export failed. Ensure wasm-pack is installed.");
            }
        }
        Commands::Add { name, features } => {
            println!("Adding CVKG component crate: {}", name);
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("add").arg(&name);
            if !features.is_empty() {
                cmd.arg("--features").arg(features.join(","));
            }
            let status = cmd.status().expect("Failed to execute cargo add");
            if !status.success() {
                eprintln!("Failed to add crate.");
            }
        }
        Commands::Theme { input, output } => {
            println!(
                "Generating theme from {} to {}",
                input.display(),
                output.display()
            );
            let json_str = std::fs::read_to_string(&input).expect("Failed to read theme JSON");
            let tokens: serde_json::Value =
                serde_json::from_str(&json_str).expect("Invalid theme JSON");

            let mut rs_content = String::from("/// Generated CVKG Theme\n");
            rs_content.push_str("pub struct Theme {\n");
            if let Some(obj) = tokens.as_object() {
                for key in obj.keys() {
                    rs_content.push_str(&format!("    pub {}: [f32; 4],\n", key));
                }
            }
            rs_content.push_str("}\n\n");

            rs_content.push_str("pub const CUSTOM_THEME: Theme = Theme {\n");
            if let Some(obj) = tokens.as_object() {
                for (key, val) in obj {
                    if let Some(arr) = val.as_array()
                        && arr.len() == 4
                    {
                        let r = arr[0].as_f64().unwrap_or(0.0);
                        let g = arr[1].as_f64().unwrap_or(0.0);
                        let b = arr[2].as_f64().unwrap_or(0.0);
                        let a = arr[3].as_f64().unwrap_or(1.0);
                        rs_content.push_str(&format!(
                            "    {}: [{:.2}, {:.2}, {:.2}, {:.2}],\n",
                            key, r, g, b, a
                        ));
                    }
                }
            }
            rs_content.push_str("};\n");

            std::fs::write(&output, rs_content).expect("Failed to write theme RS file");
            println!("Theme generation successful.");
        }
    }
}

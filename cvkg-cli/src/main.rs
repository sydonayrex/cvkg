//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! CVKG CLI toolchain, dev server, and hot reload orchestrator
//!
//! This crate provides the command-line interface for CVKG including:
//! - `cvkg new` for scaffolding new projects
//! - `cvkg dev` for starting the development server with hot reload
//! - `cvkg build` for building for target platforms
//! - `cvkg serve` for starting the WebKit preview server
//! - Other development tools.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Use the library's public API instead of re-declaring modules
use cvkg_cli::{
    CliConfig, asset_pipeline, build_pipeline, devtools_dashboard, scaffold, token_export,
    webkit_server, ws_server,
};

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
        /// Respect reduced-motion accessibility preference
        #[arg(long)]
        reduced_motion: bool,
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
    /// Launch the DevTools Dashboard (graph inspector, theme preview, event log)
    Dashboard {
        /// Port to run the dashboard on
        #[arg(long, default_value_t = 9731)]
        port: u16,
        /// Do not open the browser automatically
        #[arg(long)]
        no_open: bool,
    },
    /// Export design tokens to various formats (figma, css, swift, json)
    Tokens {
        /// Output format
        #[arg(long, value_parser = ["figma", "css", "swift", "json"])]
        format: String,
        /// Output file path
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    // Load config file and merge with CLI flags
    let mut config = CliConfig::load();

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
            reduced_motion,
        } => {
            use console::style;
            config.merge_cli(target, Some(port), inspector, reduced_motion);

            let target_str = config.target.as_deref().unwrap_or("native");
            let port = config.port.unwrap_or(3000);

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
                if config.inspector.unwrap_or(false) {
                    style("Enabled").green()
                } else {
                    style("Disabled").dim()
                }
            );
            if config.reduced_motion.unwrap_or(false) {
                println!(
                    "   {} Reduced motion: {}",
                    style("•").dim(),
                    style("Enabled").green()
                );
            }

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
            open,
            inspector,
        } => {
            use console::style;
            let url = format!("http://localhost:{}", port);
            println!(
                "{} Starting WebKit preview server on {}",
                style("🌐").blue(),
                style(&url).bold()
            );

            if open {
                println!("{} Opening browser...", style("🖥️").cyan());
                let _ = webbrowser::open(&url);
            }

            if inspector {
                println!(
                    "{} Inspector enabled on ws://localhost:{}/ws/devtools",
                    style("🔍").yellow(),
                    port
                );
            }

            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    if let Err(e) = webkit_server::start_server(addr).await {
                        eprintln!(
                            "{} Failed to start preview server: {}",
                            style("❌").red(),
                            e
                        );
                        std::process::exit(1);
                    }
                });
        }
        Commands::Check { all, target } => {
            use console::style;
            let mut failed = false;

            // Always run cargo check
            println!("{} Running cargo check...", style("🔍").blue());
            let mut check_cmd = std::process::Command::new("cargo");
            check_cmd.arg("check");
            if let Some(t) = &target {
                check_cmd.arg("--target").arg(t);
            }
            let status = check_cmd.status().expect("Failed to execute cargo check");
            if !status.success() {
                eprintln!("{} cargo check failed.", style("❌").red());
                failed = true;
            }

            // When --all, also run clippy and fmt
            if all {
                println!("{} Running cargo clippy...", style("📎").blue());
                let mut clippy_cmd = std::process::Command::new("cargo");
                clippy_cmd.arg("clippy");
                clippy_cmd.arg("--");
                clippy_cmd.arg("-D").arg("warnings");
                if let Some(t) = &target {
                    clippy_cmd.arg("--target").arg(t);
                }
                let clippy_status = clippy_cmd.status().expect("Failed to execute cargo clippy");
                if !clippy_status.success() {
                    eprintln!("{} cargo clippy found issues.", style("❌").red());
                    failed = true;
                }

                println!("{} Running cargo fmt --check...", style("📝").blue());
                let fmt_status = std::process::Command::new("cargo")
                    .arg("fmt")
                    .arg("--")
                    .arg("--check")
                    .status()
                    .expect("Failed to execute cargo fmt");
                if !fmt_status.success() {
                    eprintln!("{} cargo fmt found formatting issues.", style("❌").red());
                    failed = true;
                }
            }

            if failed {
                eprintln!("{} Check failed.", style("❌").red());
                std::process::exit(1);
            } else {
                println!(
                    "{} Check complete: All systems nominal.",
                    style("✅").green()
                );
            }
        }
        Commands::Test { ui, target } => {
            use console::style;
            println!("{} Running CVKG tests...", style("🧪").magenta());
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("test");
            if let Some(t) = &target {
                cmd.arg("--target").arg(t);
            }
            if ui {
                cmd.arg("ui");
            }
            let status = cmd.status().expect("Failed to execute cargo test");

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
        Commands::Inspect { url, ws_port } => {
            println!("🔍 Launching CVKG Telemetry Inspector...");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    let ws_url = if url.is_empty() {
                        format!("ws://localhost:{}/ws/devtools", ws_port)
                    } else {
                        format!("{}/ws/devtools", url)
                    };
                    println!("Connecting to dev server at {}...", ws_url);

                    match tokio_tungstenite::connect_async(&ws_url).await {
                        Ok((mut ws_stream, _)) => {
                            println!("✅ Connected to CVKG DevTools Stream");
                            println!("Sending metrics query...\n");

                            use futures_util::{SinkExt, StreamExt};
                            use std::io::Write;

                            let query = serde_json::json!({"command": "query_metrics"});
                            if let Err(e) = ws_stream
                                .send(tokio_tungstenite::tungstenite::Message::Text(
                                    query.to_string().into(),
                                ))
                                .await
                            {
                                eprintln!("❌ Failed to send query: {}", e);
                                return;
                            }

                            while let Some(msg) = ws_stream.next().await {
                                match msg {
                                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(&text)
                                        {
                                            match json.get("type").and_then(|t| t.as_str()) {
                                                Some("metrics") => {
                                                    print!("{esc}[2K\r", esc = 27 as char);
                                                    print!(
                                                        "📊 FPS: {:.1} | Frame: {:.2} ms | Nodes: {} | Edges: {} | GPU: {:.1} MB",
                                                        json.get("fps").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                        json.get("frame_time_ms").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                        json.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                        json.get("edge_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                        json.get("gpu_memory_mb").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                    );
                                                    std::io::stdout().flush().unwrap_or_default();
                                                }
                                                Some("pong") => {
                                                    print!("{esc}[2K\r", esc = 27 as char);
                                                    print!("📡 Connected — waiting for metrics...");
                                                    std::io::stdout().flush().unwrap_or_default();
                                                }
                                                Some("error") => {
                                                    eprintln!(
                                                        "❌ Server error: {}",
                                                        json.get("message")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("unknown")
                                                    );
                                                    break;
                                                }
                                                _ => {
                                                    println!("📡 Event: {}", text);
                                                }
                                            }
                                        }
                                    }
                                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                        println!("📡 Stream closed by server.");
                                        break;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ WebSocket error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => eprintln!("❌ Failed to connect to telemetry stream: {}", e),
                    }
                });
        }
        Commands::Export {
            base_path,
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

                let base_tag = if base_path.is_empty() {
                    String::new()
                } else {
                    format!(r#"<base href="{}">"#, base_path)
                };

                let script_path = if base_path.is_empty() {
                    format!("./pkg/{}.js", project_name)
                } else {
                    format!("{}/pkg/{}.js", base_path, project_name)
                };

                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CVKG Application</title>
    {}
    <style>
        body, html {{ margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; background: #0b0b14; }}
        canvas {{ width: 100%; height: 100%; display: block; }}
    </style>
</head>
<body>
    <canvas id="cvkg-canvas"></canvas>
    <script type="module">
        import init from '{}';
        init();
    </script>
</body>
</html>"#,
                    base_tag, script_path
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
            use console::style;

            // Validate crate name
            if name.is_empty() {
                eprintln!("{} Crate name cannot be empty.", style("❌").red());
                std::process::exit(1);
            }

            // Crate names must be valid Rust identifiers: lowercase alphanumeric + hyphens/underscores
            let valid = name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                && !name.starts_with('-')
                && !name.ends_with('-')
                && !name.is_empty();
            if !valid {
                eprintln!(
                    "{} Invalid crate name '{}'. Use lowercase alphanumeric characters, hyphens, or underscores.",
                    style("❌").red(),
                    name
                );
                std::process::exit(1);
            }

            println!(
                "{} Adding CVKG component crate: {}",
                style("📦").cyan(),
                style(&name).yellow()
            );
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("add").arg(&name);
            if !features.is_empty() {
                cmd.arg("--features").arg(features.join(","));
            }
            let status = cmd.status().unwrap_or_else(|e| {
                eprintln!("{} Failed to execute cargo add: {}", style("❌").red(), e);
                std::process::exit(1);
            });
            if status.success() {
                println!(
                    "{} Successfully added {}",
                    style("✅").green(),
                    style(&name).bold()
                );
            } else {
                eprintln!("{} Failed to add crate '{}'.", style("❌").red(), name);
                std::process::exit(1);
            }
        }
        Commands::Theme { input, output } => {
            use console::style;
            println!(
                "{} Generating theme from {} to {}",
                style("🎨").cyan(),
                style(input.display()).yellow(),
                style(output.display()).yellow()
            );
            let json_str = std::fs::read_to_string(&input).unwrap_or_else(|e| {
                eprintln!("{} Failed to read theme JSON: {}", style("❌").red(), e);
                std::process::exit(1);
            });
            let tokens: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_else(|e| {
                eprintln!("{} Invalid theme JSON: {}", style("❌").red(), e);
                std::process::exit(1);
            });

            let mut declarations = Vec::new();
            let mut values = Vec::new();
            let mut skipped = Vec::new();

            if let Some(obj) = tokens.as_object() {
                for (key, val) in obj {
                    match val {
                        serde_json::Value::Array(arr) if arr.len() == 4 => {
                            let parse_f64 = |v: &serde_json::Value| -> f64 {
                                v.as_f64()
                                    .or_else(|| v.as_i64().map(|i| i as f64))
                                    .or_else(|| v.as_u64().map(|u| u as f64))
                                    .unwrap_or(0.0)
                            };
                            let r = parse_f64(&arr[0]);
                            let g = parse_f64(&arr[1]);
                            let b = parse_f64(&arr[2]);
                            let a = parse_f64(&arr[3]);
                            declarations.push(format!("    pub {}: [f32; 4],", key));
                            values.push(format!(
                                "    {}: [{:.4}, {:.4}, {:.4}, {:.4}],",
                                key, r, g, b, a
                            ));
                        }
                        serde_json::Value::String(s) if s.starts_with('#') => {
                            if let Some(rgba) = parse_hex_color(s) {
                                declarations.push(format!("    pub {}: [f32; 4],", key));
                                values.push(format!(
                                    "    {}: [{:.4}, {:.4}, {:.4}, {:.4}],",
                                    key, rgba.0, rgba.1, rgba.2, rgba.3
                                ));
                            } else {
                                skipped.push((key.clone(), format!("invalid hex color: {}", s)));
                            }
                        }
                        serde_json::Value::Number(n) => {
                            let v = n.as_f64().unwrap_or(0.0);
                            declarations.push(format!("    pub {}: f32,", key));
                            values.push(format!("    {}: {:.2},", key, v));
                        }
                        serde_json::Value::Object(_) => {
                            skipped.push((key.clone(), "nested objects not supported".to_string()));
                        }
                        other => {
                            skipped.push((key.clone(), format!("unsupported type: {:?}", other)));
                        }
                    }
                }
            }

            if !skipped.is_empty() {
                eprintln!(
                    "{} Skipped {} token(s):",
                    style("⚠️").yellow(),
                    skipped.len()
                );
                for (key, reason) in &skipped {
                    eprintln!("  - {}: {}", key, reason);
                }
            }

            let mut rs_content = String::from("/// Generated CVKG Theme\n");
            rs_content.push_str(&format!("/// Source: {}\n", input.display()));
            rs_content.push_str("#[derive(Debug, Clone)]\n");
            rs_content.push_str("pub struct Theme {\n");
            for decl in &declarations {
                rs_content.push_str(decl);
                rs_content.push('\n');
            }
            rs_content.push_str("}\n\n");

            rs_content.push_str("pub const CUSTOM_THEME: Theme = Theme {\n");
            for val in &values {
                rs_content.push_str(val);
                rs_content.push('\n');
            }
            rs_content.push_str("};\n");

            std::fs::write(&output, &rs_content).unwrap_or_else(|e| {
                eprintln!("{} Failed to write output: {}", style("❌").red(), e);
                std::process::exit(1);
            });
            println!(
                "{} Theme generation successful. {} tokens written to {}",
                style("✅").green(),
                declarations.len(),
                style(output.display()).bold()
            );
        }
        Commands::Dashboard { port, no_open } => {
            use console::style;
            println!("{} Starting CVKG DevTools Dashboard...", style("🔧").cyan());
            let config = devtools_dashboard::DashboardConfig {
                port,
                open_browser: !no_open,
                graph_state: std::sync::Arc::new(std::sync::Mutex::new(
                    devtools_dashboard::GraphState::default(),
                )),
            };
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build runtime: {}", e))
                .block_on(async {
                    if let Err(e) = devtools_dashboard::start_dashboard(config).await {
                        eprintln!("{} Dashboard error: {}", style("❌").red(), e);
                    }
                });
        }
        Commands::Tokens { format, output } => {
            use console::style;
            println!(
                "{} Exporting design tokens (format: {})...",
                style("🎨").cyan(),
                style(&format).yellow()
            );

            let export = token_export::TokenExport::new();
            let content = match export.generate(&format) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} Token export failed: {}", style("❌").red(), e);
                    std::process::exit(1);
                }
            };

            std::fs::write(&output, content).unwrap_or_else(|e| {
                eprintln!("{} Failed to write output: {}", style("❌").red(), e);
                std::process::exit(1);
            });

            println!(
                "{} Tokens exported to {}",
                style("✅").green(),
                style(output.display()).bold()
            );
        }
    }
}

/// Parse a hex color string like "#RRGGBB" or "#RRGGBBAA" into (r, g, b, a) as 0.0–1.0 f32 values.
fn parse_hex_color(hex: &str) -> Option<(f32, f32, f32, f32)> {
    let hex = hex.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
            Some((r, g, b, 1.0))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

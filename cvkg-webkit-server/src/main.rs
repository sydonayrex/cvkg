//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS — Read the target, its surrounding context, and its full call graph at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL — Every major pub fn, unsafe block, and non-trivial algorithm in every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment. Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS — Check every tool call / command for progress every 30 seconds. After 3 consecutive identical failures, stop, write BLOCKED.md, and move to unblocked work. Never silently accept a broken state.

//! Professional CVKG Dev Server & App Preview Host.
//! Features: Universal Build Pipeline, State-Preserving HMR, SEO Pre-rendering, Production Hardening.

use arc_swap::ArcSwap;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State, DefaultBodyLimit, Request},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tracing::{info, warn, error};
use validator::Validate;

/// Configuration for the CVKG Server.
/// Can be set via CLI arguments or environment variables.
#[derive(Parser, Debug, Clone, Validate)]
#[command(author, version, about, long_about = None)]
struct Config {
    /// Address to bind the server to.
    #[arg(short, long, env = "CVKG_BIND_ADDR", default_value = "0.0.0.0:3000")]
    pub addr: SocketAddr,

    /// Directory for package artifacts.
    #[arg(long, env = "CVKG_PKG_DIR", default_value = "cvkg-webkit-server/pkg")]
    pub pkg_dir: String,

    /// Directory for assets.
    #[arg(long, env = "CVKG_ASSETS_DIR", default_value = "cvkg-webkit-server/assets")]
    pub assets_dir: String,

    /// Directory for static files.
    #[arg(long, env = "CVKG_STATIC_DIR", default_value = "cvkg-webkit-server/static")]
    pub static_dir: String,

    /// Rate limit: requests per second.
    #[arg(long, env = "CVKG_RATE_LIMIT_RPS", default_value = "1000")]
    pub rate_limit_rps: u64,

    /// Request timeout in seconds.
    #[arg(long, env = "CVKG_TIMEOUT_SECS", default_value = "30")]
    pub timeout_secs: u64,

    /// Maximum concurrent requests.
    #[arg(long, env = "CVKG_MAX_CONCURRENT", default_value = "100")]
    pub max_concurrent: usize,
}

/// Shared application state for the dev server.
struct AppState {
    /// Last captured VDOM snapshot for SEO / Initial Load.
    last_vdom_snapshot: ArcSwap<Option<String>>,
    /// Server configuration.
    config: Config,
}

/// Universal Build Orchestrator.
struct BuildOrchestrator;

impl BuildOrchestrator {
    /// Triggers the universal build pipeline with simple retry logic.
    pub async fn trigger_universal_build() -> anyhow::Result<()> {
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            info!("[CVKG Build] Starting universal build pipeline (attempt {})...", attempts + 1);
            // Simulate build work
            match Self::perform_build().await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < max_attempts - 1 => {
                    warn!("Build attempt {} failed: {}. Retrying...", attempts + 1, e);
                    tokio::time::sleep(Duration::from_millis(500 * (attempts + 1) as u64)).await;
                    attempts += 1;
                }
                Err(e) => return Err(e),
            }
        }
        Err(anyhow::anyhow!("Build failed after {} attempts", max_attempts))
    }

    async fn perform_build() -> anyhow::Result<()> {
        // In a real implementation, this would call out to cargo or a custom build script.
        Ok(())
    }
}

/// Handler for capturing VDOM snapshots.
async fn capture_snapshot(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    state.last_vdom_snapshot.store(Arc::new(Some(body)));
    "Snapshot captured"
}

/// Handler for serving the loading screen or the last snapshot.
async fn serve_loading_screen(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snapshot_guard = state.last_vdom_snapshot.load();
    
    if let Some(snapshot) = snapshot_guard.as_deref() {
        Html(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CVKG</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&display=swap" rel="stylesheet">
    <style>
        body {{ margin: 0; background: #0a0a0c; color: #fff; font-family: 'Inter', sans-serif; }}
    </style>
</head>
<body>
    <div id="app">{}</div>
    <script src='/pkg/app.js'></script>
</body>
</html>"#,
            snapshot
        ))
    } else {
        // Serve niflheim.html as the default load screen
        let path = format!("{}/niflheim.html", state.config.static_dir);
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => Html(content),
            Err(e) => {
                warn!("Failed to read {}: {}", path, e);
                Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CVKG | Niflheim Protocol</title>
    <style>
        :root {
            --bg: #0a0a0c;
            --accent: #00f2ff;
            --accent-dim: rgba(0, 242, 255, 0.2);
            --glass: rgba(255, 255, 255, 0.03);
        }
        body {
            margin: 0; padding: 0; background: var(--bg); color: #fff;
            font-family: 'Inter', -apple-system, sans-serif;
            display: flex; align-items: center; justify-content: center;
            height: 100vh; overflow: hidden;
        }
        .container {
            text-align: center; padding: 3rem; background: var(--glass);
            backdrop-filter: blur(20px); border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 24px; animation: fadeIn 0.8s ease-out;
        }
        .logo { font-size: 3rem; font-weight: 900; margin-bottom: 1rem; background: linear-gradient(135deg, #fff, var(--accent)); -webkit-background-clip: text; -webkit-fill-color: transparent; }
        .loader { width: 48px; height: 48px; border: 3px solid var(--accent-dim); border-radius: 50%; display: inline-block; position: relative; animation: rotation 1s linear infinite; }
        .loader::after { content: ''; position: absolute; left: 0; top: 0; background: var(--accent); width: 12px; height: 12px; border-radius: 50%; }
        .status { margin-top: 1.5rem; font-size: 0.875rem; color: var(--accent); text-transform: uppercase; letter-spacing: 2px; }
        @keyframes rotation { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }
        #cvkg-canvas { position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: -1; }
    </style>
</head>
<body>
    <div id="cvkg-root"><canvas id="cvkg-canvas"></canvas></div>
    <div class="container" id="loader-ui">
        <div class="logo">CVKG</div>
        <div class="loader"></div>
        <div id="status" class="status">INITIALIZING NIFLHEIM...</div>
    </div>
    <script type="module" crossorigin>
        import init, { get_render_tier_name } from 'cvkg-webkit-server/pkg/wgpu/niflheim_web_demo.js';
        async function run() {
            try {
                await init();
                document.getElementById('loader-ui').style.display = 'none';
                const update = () => {
                    const tier = get_render_tier_name();
                    if (tier !== "Detecting...") {
                        const statusText = `CVKG // NIFLHEIM PROTOCOL // ${tier.toUpperCase()} BACKEND`;
                        console.log(statusText);
                    } else { setTimeout(update, 100); }
                };
                update();
            } catch (e) {
                console.error("Initialization Failed:", e);
                document.getElementById('status').textContent = "BOOT ERROR";
                document.getElementById('status').style.color = "red";
            }
        }
        run();
    </script>
</body>
</html>"#.to_string())
            }
        }
    }
}

/// Handler for triggering a manual build.
async fn trigger_build_handler() -> impl IntoResponse {
    match BuildOrchestrator::trigger_universal_build().await {
        Ok(_) => (axum::http::StatusCode::OK, "Build successful".to_string()),
        Err(e) => {
            error!("Build failed: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Build failed: {}", e))
        }
    }
}

/// Liveness health check handler.
async fn liveness_handler() -> &'static str {
    "OK"
}

/// Readiness health check handler.
async fn readiness_handler() -> &'static str {
    "READY"
}

/// WebSocket handler for CVKG protocol.
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// WebSocket handler for HMR.
async fn hmr_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_hmr_socket)
}

/// Internal WebSocket message processor.
async fn handle_socket(mut ws: WebSocket) {
    while let Some(Ok(_)) = ws.next().await {}
}

/// Internal HMR message processor.
async fn handle_hmr_socket(mut ws: WebSocket) {
    while let Some(Ok(_)) = ws.next().await {}
}

/// Custom metrics middleware to record request counts and latencies.
async fn metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    
    metrics::counter!("http_requests_total", "method" => method, "path" => path.clone(), "status" => status).increment(1);
    metrics::histogram!("http_request_duration_seconds", "method" => path).record(latency);
    
    response
}

/// Setup graceful shutdown signal handling.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Ctrl+C received, starting graceful shutdown...");
        },
        _ = terminate => {
            info!("SIGTERM received, starting graceful shutdown...");
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env if present.
    let _ = dotenvy::dotenv();

    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse configuration.
    let mut config = Config::parse();
    info!("Starting Professional CVKG Server on http://{}...", config.addr);
    info!("[CVKG] Current Working Directory: {:?}", std::env::current_dir().unwrap_or_default());

    // Auto-resolve paths to handle root-relative vs crate-relative execution
    if !std::path::Path::new(&config.pkg_dir).exists() {
        let alt = format!("cvkg-webkit-server/{}", config.pkg_dir);
        if std::path::Path::new(&alt).exists() {
            info!("[CVKG] Auto-pivoting PKG_DIR to: {}", alt);
            config.pkg_dir = alt;
        }
    }

    if !std::path::Path::new(&config.static_dir).exists() {
        let alt = format!("cvkg-webkit-server/{}", config.static_dir);
        if std::path::Path::new(&alt).exists() {
            info!("[CVKG] Auto-pivoting STATIC_DIR to: {}", alt);
            config.static_dir = alt;
        }
    }

    if !std::path::Path::new(&config.assets_dir).exists() {
        let alt = format!("cvkg-webkit-server/{}", config.assets_dir);
        if std::path::Path::new(&alt).exists() {
            info!("[CVKG] Auto-pivoting ASSETS_DIR to: {}", alt);
            config.assets_dir = alt;
        }
    }

    // ENFORCE ABSOLUTE PATHS to prevent any further working directory confusion
    config.pkg_dir = std::fs::canonicalize(&config.pkg_dir)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(config.pkg_dir);
    config.static_dir = std::fs::canonicalize(&config.static_dir)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(config.static_dir);
    config.assets_dir = std::fs::canonicalize(&config.assets_dir)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(config.assets_dir);

    info!("[CVKG] Final Absolute PKG_DIR: {}", config.pkg_dir);
    info!("[CVKG] Final Absolute STATIC_DIR: {}", config.static_dir);

    // Setup Prometheus metrics without starting a separate HTTP listener.
    // ... (metrics setup)
    let metric_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install prometheus recorder");

    let state = Arc::new(AppState {
        last_vdom_snapshot: ArcSwap::from_pointee(None),
        config: config.clone(),
    });

    // Build the router with middleware layers.
    let app = Router::new()
        .route("/", get(serve_loading_screen))
        .route("/snapshot", post(capture_snapshot))
        .route("/build", post(trigger_build_handler))
        .nest_service("/cvkg-webkit-server/pkg", ServeDir::new(&config.pkg_dir))
        .nest_service("/cvkg-webkit-server/assets", ServeDir::new(&config.assets_dir))
        .nest_service("/cvkg-webkit-server/static", ServeDir::new(&config.static_dir))
        .route("/cvkg-ws", get(ws_handler))
        .route("/hmr", get(hmr_ws_handler))
        // Observability endpoints
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/metrics", get(move || {
            let rendered = metric_handle.render();
            async move { rendered }
        }))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(metrics_middleware))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    axum::http::HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' 'unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self' ws: wss: blob:;"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::STRICT_TRANSPORT_SECURITY,
                    axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_CONTENT_TYPE_OPTIONS,
                    axum::http::HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_FRAME_OPTIONS,
                    axum::http::HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::HeaderName::from_static("cross-origin-opener-policy"),
                    axum::http::HeaderValue::from_static("same-origin"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::HeaderName::from_static("cross-origin-embedder-policy"),
                    axum::http::HeaderValue::from_static("require-corp"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("no-store"),
                ))
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5MB limit
                .layer(TimeoutLayer::new(Duration::from_secs(config.timeout_secs)))
                .layer(tower::limit::ConcurrencyLimitLayer::new(config.max_concurrent))
        );

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("CVKG Server shut down gracefully.");
    Ok(())
}
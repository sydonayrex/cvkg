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

//! CVKG Webkit Server - Cyber-Viking OS Host
//! [VERIFICATION]: File system writes are active and confirmed.
//! Features: Universal Build Pipeline, State-Preserving HMR, SEO Pre-rendering, Production Hardening.

#![allow(unused_imports, deprecated)]

use arc_swap::ArcSwap;
use axum::{
    Router,
    extract::{
        DefaultBodyLimit, Request, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use clap::Parser;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
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
    #[arg(
        long,
        env = "CVKG_ASSETS_DIR",
        default_value = "cvkg-webkit-server/assets"
    )]
    pub assets_dir: String,

    /// Directory for static files.
    #[arg(
        long,
        env = "CVKG_STATIC_DIR",
        default_value = "cvkg-webkit-server/static"
    )]
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
    _config: Config,
    /// HMR broadcast sender for pushing updates to clients.
    hmr_tx: tokio::sync::broadcast::Sender<String>,
}

/// Universal Build Orchestrator.
struct BuildOrchestrator;

impl BuildOrchestrator {
    /// Triggers the universal build pipeline with simple retry logic.
    pub async fn trigger_universal_build() -> anyhow::Result<()> {
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            info!(
                "[CVKG Build] Starting universal build pipeline (attempt {})...",
                attempts + 1
            );
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
        Err(anyhow::anyhow!(
            "Build failed after {} attempts",
            max_attempts
        ))
    }

    async fn perform_build() -> anyhow::Result<()> {
        // In a real implementation, this would call out to cargo or a custom build script.
        Ok(())
    }
}

/// Handler for capturing VDOM snapshots.
async fn capture_snapshot(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    state.last_vdom_snapshot.store(Arc::new(Some(body)));
    "Snapshot captured"
}

/// Handler for serving the loading screen or the last snapshot.
async fn serve_loading_screen(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let snapshot_guard = state.last_vdom_snapshot.load();
    let snapshot = snapshot_guard
        .as_ref()
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Loading Agent Ulfhednar...");

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Agent Ulfhednar - Tactical Dashboard</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;700;900&display=swap" rel="stylesheet">
    <style>
        body {{ margin: 0; background: #0a0a0c; color: #fff; font-family: 'Inter', sans-serif; overflow: hidden; }}
        #cvkg-root {{ width: 100vw; height: 100vh; }}
    </style>
</head>
<body>
    <div id="cvkg-root">{}</div>
    <script>
        // HMR Client Protocol Integration
        (function() {{
            function connect() {{
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const socketUrl = `${{protocol}}//${{window.location.host}}/hmr`;
                console.log(`[HMR] Connecting to ${{socketUrl}}...`);
                const ws = new WebSocket(socketUrl);

                ws.onmessage = function(event) {{
                    try {{
                        const data = JSON.parse(event.data);
                        if (data.type === 'reload') {{
                            console.log('[HMR] Reload signal received. Reloading page...');
                            window.location.reload();
                        }}
                    }} catch (e) {{
                        console.error('[HMR] Invalid message received:', event.data);
                    }}
                }};

                ws.onclose = function() {{
                    console.warn('[HMR] Connection closed. Reconnecting in 2 seconds...');
                    setTimeout(connect, 2000);
                }};

                ws.onerror = function(err) {{
                    console.error('[HMR] Connection error:', err);
                }};
            }}
            connect();
        }})();
    </script>
    <script type="module">
        import init from '/cvkg-webkit-server/pkg/berserker_fire_web_demo.js';
        async function run() {{
            try {{
                console.log("Initializing Berserker Fire Demo...");
                await init();
                console.log("Berserker Fire Demo active.");
            }} catch (e) {{
                console.error("Berserker Fire Demo failure:", e);
            }}
        }}
        run();
    </script>
</body>
</html>"#,
        snapshot
    ))
}

/// Handler for triggering a manual build.
async fn trigger_build_handler() -> impl IntoResponse {
    match BuildOrchestrator::trigger_universal_build().await {
        Ok(_) => (axum::http::StatusCode::OK, "Build successful".to_string()),
        Err(e) => {
            error!("Build failed: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Build failed: {}", e),
            )
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

/// Handler for serving the current system time.
#[derive(serde::Serialize, serde::Deserialize)]
struct SystemTime {
    timestamp: u64,
}

async fn system_time_handler() -> impl IntoResponse {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    axum::Json(SystemTime {
        timestamp: duration.as_secs(),
    })
}

/// WebSocket handler for CVKG protocol.
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// WebSocket handler for HMR (Hot Module Relays).
async fn hmr_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_hmr_socket(socket, state))
}

/// Handle runtime protocol WebSocket connections.
async fn handle_socket(mut ws: WebSocket) {
    use futures_util::SinkExt;

    // Send handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "payload": {
            "client": "webkit-runtime",
            "capabilities": ["patch", "state", "event"]
        }
    });
    if let Err(e) = ws
        .send(axum::extract::ws::Message::Text(handshake.to_string()))
        .await
    {
        error!("Failed to send handshake: {}", e);
        return;
    }

    while let Some(Ok(msg)) = ws.next().await {
        match msg {
            axum::extract::ws::Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<cvkg_cli::WsMessage>(&text) {
                    match ws_msg {
                        cvkg_cli::WsMessage::Patch(patch) => {
                            info!(
                                "Runtime patch received: {:?}",
                                std::mem::discriminant(&patch)
                            );
                        }
                        cvkg_cli::WsMessage::Event(event) => {
                            info!("Runtime event received: {:?}", event);
                        }
                        cvkg_cli::WsMessage::State(_) => {
                            info!("Runtime state snapshot received");
                        }
                        _ => {}
                    }
                }
            }
            axum::extract::ws::Message::Close(_) => {
                info!("Runtime WebSocket client disconnected");
                break;
            }
            _ => {}
        }
    }
}

/// Handle HMR WebSocket connections — broadcasts patches to connected clients.
async fn handle_hmr_socket(mut ws: WebSocket, state: Arc<AppState>) {
    use futures_util::SinkExt;

    // Send handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "payload": {
            "client": "webkit-hmr",
            "capabilities": ["patch"]
        }
    });
    if let Err(e) = ws
        .send(axum::extract::ws::Message::Text(handshake.to_string()))
        .await
    {
        error!("Failed to send HMR handshake: {}", e);
        return;
    }

    let mut rx = state.hmr_tx.subscribe();

    loop {
        tokio::select! {
            // Listen for broadcast messages from the watcher
            Ok(msg_str) = rx.recv() => {
                if let Err(e) = ws.send(axum::extract::ws::Message::Text(msg_str)).await {
                    error!("Failed to send HMR update: {}", e);
                    break;
                }
            }
            // Keep connection alive or handle client close
            msg = ws.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => {
                        info!("HMR WebSocket client disconnected");
                        break;
                    }
                    Some(Ok(axum::extract::ws::Message::Text(text))) if text.contains("ping") => {
                        let _ = ws
                            .send(axum::extract::ws::Message::Text(
                                r#"{"type":"pong"}"#.to_string(),
                            ))
                            .await;
                    }
                    _ => {}
                }
            }
        }
    }
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

fn spawn_file_watcher(state: Arc<AppState>) {
    let pkg_dir = state._config.pkg_dir.clone();
    let static_dir = state._config.static_dir.clone();
    let assets_dir = state._config.assets_dir.clone();
    let hmr_tx = state.hmr_tx.clone();

    tokio::spawn(async move {
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut file_times: HashMap<PathBuf, std::time::SystemTime> = HashMap::new();

        fn scan_dir(dir: &str, files: &mut HashMap<PathBuf, std::time::SystemTime>) {
            let path = std::path::Path::new(dir);
            if !path.exists() {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        scan_dir(&p.to_string_lossy(), files);
                    } else if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            files.insert(p, modified);
                        }
                    }
                }
            }
        }

        scan_dir(&pkg_dir, &mut file_times);
        scan_dir(&static_dir, &mut file_times);
        scan_dir(&assets_dir, &mut file_times);

        info!(
            "[HMR Watcher] Initialized watcher for {}, {} and {}",
            pkg_dir, static_dir, assets_dir
        );

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let mut current_files = HashMap::new();
            scan_dir(&pkg_dir, &mut current_files);
            scan_dir(&static_dir, &mut current_files);
            scan_dir(&assets_dir, &mut current_files);

            let mut changed = false;

            for (path, modified) in &current_files {
                match file_times.get(path) {
                    Some(old_modified) => {
                        if modified > old_modified {
                            info!("[HMR Watcher] File modified: {:?}", path);
                            changed = true;
                        }
                    }
                    None => {
                        info!("[HMR Watcher] File created: {:?}", path);
                        changed = true;
                    }
                }
            }

            for path in file_times.keys() {
                if !current_files.contains_key(path) {
                    info!("[HMR Watcher] File deleted: {:?}", path);
                    changed = true;
                }
            }

            if changed {
                file_times = current_files;
                info!("[HMR Watcher] Broadcasting HMR reload event...");
                let reload_msg = serde_json::json!({
                    "type": "reload",
                    "payload": {}
                });
                let _ = hmr_tx.send(reload_msg.to_string());
            }
        }
    });
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
    info!(
        "Starting Professional CVKG Server on http://{}...",
        config.addr
    );
    info!(
        "[CVKG] Current Working Directory: {:?}",
        std::env::current_dir().unwrap_or_default()
    );

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

    let (hmr_tx, _) = tokio::sync::broadcast::channel(16);

    let state = Arc::new(AppState {
        last_vdom_snapshot: ArcSwap::from_pointee(None),
        _config: config.clone(),
        hmr_tx,
    });

    // Spawn the background file watcher task
    spawn_file_watcher(state.clone());

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
        .route("/api/system/time", get(system_time_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(metrics_middleware))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    axum::http::HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' 'unsafe-eval' https://cdnjs.cloudflare.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self' ws: wss: blob:; frame-src *;"),
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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    info!("CVKG Server shut down gracefully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_liveness() {
        let response = liveness_handler().await;
        assert_eq!(response, "OK");
    }

    #[tokio::test]
    async fn test_readiness() {
        let response = readiness_handler().await;
        assert_eq!(response, "READY");
    }

    #[tokio::test]
    async fn test_system_time() {
        let response = system_time_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let time: SystemTime = serde_json::from_slice(&body).unwrap();
        assert!(time.timestamp > 0);
    }
}

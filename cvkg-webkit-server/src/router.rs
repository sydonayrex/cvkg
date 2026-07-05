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
    http::{StatusCode, HeaderMap, Method},
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
pub struct Config {
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
pub struct AppState {
    /// Last captured VDOM snapshot for SEO / Initial Load.
    pub last_vdom_snapshot: ArcSwap<Option<String>>,
    /// Server configuration.
    pub config: Config,
    /// HMR broadcast sender for pushing updates to clients.
    pub hmr_tx: tokio::sync::broadcast::Sender<String>,
    /// Authentication token for gating POST requests.
    pub auth_token: String,
    /// Dynamically resolved JS entrypoint file name in pkg_dir.
    pub js_entrypoint: String,
}

impl AppState {
    /// Create a new AppState instance.
    pub fn new(config: Config, hmr_tx: tokio::sync::broadcast::Sender<String>) -> Self {
        let auth_token = std::env::var("CVKG_AUTH_TOKEN").unwrap_or_else(|_| "dev-token-default-12345".to_string());
        
        let js_entrypoint = if let Ok(entries) = std::fs::read_dir(&config.pkg_dir) {
            let mut found = None;
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".js") && !name.contains("bg") {
                        found = Some(name.to_string());
                        break;
                    }
                }
            }
            found.unwrap_or_else(|| "berserker_fire_web_demo.js".to_string())
        } else {
            "berserker_fire_web_demo.js".to_string()
        };

        Self {
            last_vdom_snapshot: ArcSwap::from_pointee(None),
            config,
            hmr_tx,
            auth_token,
            js_entrypoint,
        }
    }
}

/// Universal Build Orchestrator.
pub struct BuildOrchestrator;

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
        Ok(())
    }
}

/// HTML Sanitizer to prevent XSS (H1)
fn sanitize_html(html: &str) -> String {
    let re_script = regex::Regex::new(r"(?i)<script[^>]*>[\s\S]*?</script>").unwrap();
    let cleaned = re_script.replace_all(html, "");

    let re_events = regex::Regex::new(r#"(?i)\s+on[a-z]+\s*=\s*"[^"]*""#).unwrap();
    let cleaned = re_events.replace_all(&cleaned, "");
    let re_events_single = regex::Regex::new(r#"(?i)\s+on[a-z]+\s*=\s*'[^']*'"#).unwrap();
    let cleaned = re_events_single.replace_all(&cleaned, "");

    let re_javascript = regex::Regex::new(r#"(?i)(href|src)\s*=\s*"\s*javascript:[^"]*""#).unwrap();
    let cleaned = re_javascript.replace_all(&cleaned, "");
    let re_javascript_single = regex::Regex::new(r#"(?i)(href|src)\s*=\s*'\s*javascript:[^']*'"#).unwrap();
    let cleaned = re_javascript_single.replace_all(&cleaned, "");

    cleaned.into_owned()
}

/// Middleware to validate path traversal sequences (M2)
async fn path_validation_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    
    if path.contains("..") || path.contains("//") || path.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut decoded = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut clone_chars = chars.clone();
            let h1 = clone_chars.next();
            let h2 = clone_chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(hex) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                    decoded.push(hex as char);
                    chars.next();
                    chars.next();
                    continue;
                }
            }
        }
        decoded.push(c);
    }

    if decoded.contains("..") || decoded.contains("//") || decoded.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(req).await)
}

/// Middleware to enforce authentication on POST endpoints (H1, M1)
async fn check_auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.method() == Method::POST {
        if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if token == state.auth_token {
                        return Ok(next.run(req).await);
                    }
                }
            }
        }
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

/// Create the Axum router with all middleware and routes.
pub fn create_router(
    state: Arc<AppState>,
    metric_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
) -> Router {
    let pkg_dir = state.config.pkg_dir.clone();
    let assets_dir = state.config.assets_dir.clone();
    let static_dir = state.config.static_dir.clone();
    let timeout_secs = state.config.timeout_secs;
    let max_concurrent = state.config.max_concurrent;
    Router::new()
        .route("/", get(serve_loading_screen))
        .route("/snapshot", post(capture_snapshot))
        .route("/build", post(trigger_build_handler))
        .nest_service("/cvkg-webkit-server/pkg", ServeDir::new(&pkg_dir))
        .nest_service("/cvkg-webkit-server/assets", ServeDir::new(&assets_dir))
        .nest_service("/cvkg-webkit-server/static", ServeDir::new(&static_dir))
        .route("/cvkg-ws", get(ws_handler))
        .route("/hmr", get(hmr_ws_handler))
        // Observability endpoints
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/metrics", get(move || {
            let rendered = metric_handle.as_ref().map(|h| h.render()).unwrap_or_default();
            async move { rendered }
        }))
        .route("/api/system/time", get(system_time_handler))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), check_auth_middleware))
        .layer(middleware::from_fn(path_validation_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(metrics_middleware))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    axum::http::HeaderValue::from_static("default-src 'self'; script-src 'self' 'wasm-unsafe-eval' https://cdnjs.cloudflare.com; style-src 'self' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self' ws://localhost:* ws://127.0.0.1:* wss:; frame-src 'self';"),
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
                .layer(TimeoutLayer::new(Duration::from_secs(timeout_secs)))
                .layer(tower::limit::ConcurrencyLimitLayer::new(max_concurrent))
        )
}

/// Handler for capturing VDOM snapshots.
pub async fn capture_snapshot(State(state): State<Arc<AppState>>, body: String) -> impl IntoResponse {
    if body.len() > 256 * 1024 {
        return (StatusCode::PAYLOAD_TOO_LARGE, "Snapshot payload exceeds 256KB limit").into_response();
    }
    let sanitized = sanitize_html(&body);
    state.last_vdom_snapshot.store(Arc::new(Some(sanitized)));
    "Snapshot captured".into_response()
}

/// Handler for serving the loading screen or the last snapshot.
pub async fn serve_loading_screen(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
        window.CVKG_AUTH_TOKEN = "{}";

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
        import init from '/cvkg-webkit-server/pkg/{}';
        async function run() {{
            try {{
                console.log("Initializing Web App Demo...");
                await init();
                console.log("Web App Demo active.");
            }} catch (e) {{
                console.error("Web App Demo failure:", e);
            }}
        }}
        run();
    </script>
</body>
</html>"#,
        snapshot,
        state.auth_token,
        state.js_entrypoint
    ))
}

/// Handler for triggering a manual build.
pub async fn trigger_build_handler() -> impl IntoResponse {
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
pub async fn liveness_handler() -> &'static str {
    "OK"
}

/// Readiness health check handler.
pub async fn readiness_handler() -> &'static str {
    "READY"
}

/// Handler for serving the current system time.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SystemTime {
    pub timestamp: u64,
}

pub async fn system_time_handler() -> impl IntoResponse {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    axum::Json(SystemTime {
        timestamp: duration.as_secs(),
    })
}

/// WebSocket handler for CVKG protocol.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(origin) = headers.get(axum::http::header::ORIGIN) {
        if let Ok(origin_str) = origin.to_str() {
            let is_allowed = origin_str.starts_with("http://localhost:")
                || origin_str.starts_with("http://127.0.0.1:")
                || origin_str == "null"
                || origin_str.starts_with(&format!("http://{}", state.config.addr))
                || origin_str.starts_with(&format!("https://{}", state.config.addr));
            
            if !is_allowed {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    Ok(ws.on_upgrade(handle_socket))
}

/// WebSocket handler for HMR (Hot Module Relays).
pub async fn hmr_ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(origin) = headers.get(axum::http::header::ORIGIN) {
        if let Ok(origin_str) = origin.to_str() {
            let is_allowed = origin_str.starts_with("http://localhost:")
                || origin_str.starts_with("http://127.0.0.1:")
                || origin_str == "null"
                || origin_str.starts_with(&format!("http://{}", state.config.addr))
                || origin_str.starts_with(&format!("https://{}", state.config.addr));
            
            if !is_allowed {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    Ok(ws.on_upgrade(move |socket| handle_hmr_socket(socket, state)))
}

/// Handle runtime protocol WebSocket connections.
pub async fn handle_socket(mut ws: WebSocket) {
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
        .send(axum::extract::ws::Message::Text(
            handshake.to_string().into(),
        ))
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

/// Handle HMR WebSocket connections -- broadcasts patches to connected clients.
pub async fn handle_hmr_socket(mut ws: WebSocket, state: Arc<AppState>) {
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
        .send(axum::extract::ws::Message::Text(
            handshake.to_string().into(),
        ))
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
                if let Err(e) = ws.send(axum::extract::ws::Message::Text(msg_str.into())).await {
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
                                r#"{"type":"pong"}"#.into(),
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
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
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
pub async fn shutdown_signal() {
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

/// Spawn the background file watcher task
pub fn spawn_file_watcher(state: Arc<AppState>) {
    let pkg_dir = state.config.pkg_dir.clone();
    let static_dir = state.config.static_dir.clone();
    let assets_dir = state.config.assets_dir.clone();
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
                    } else if let Ok(metadata) = entry.metadata()
                        && let Ok(modified) = metadata.modified()
                    {
                        files.insert(p, modified);
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

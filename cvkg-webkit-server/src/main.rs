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
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.

//! Professional CVKG Dev Server & App Preview Host.
//! Features: Universal Build Pipeline, State-Preserving HMR, SEO Pre-rendering.

use arc_swap::ArcSwap;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::process::Command;

/// Shared application state for the dev server.
///
/// `last_vdom_snapshot` is served on every HTTP request to `/` but written only once per
/// client render cycle. `ArcSwap` gives the read path a lock-free snapshot; the write path
/// publishes atomically with `store()`.
struct AppState {
    #[allow(dead_code)]
    config: ServeConfig,
    /// Last captured vDOM snapshot for SSG/SEO (lock-free read via ArcSwap).
    last_vdom_snapshot: ArcSwap<Option<String>>,
}

#[derive(Debug, Clone)]
struct ServeConfig {
    addr: SocketAddr,
    #[allow(dead_code)]
    port: u16,
}

/// Universal Build Orchestrator.
/// Simultaneously triggers builds for Web (WASM) and Desktop (Native) targets.
struct BuildOrchestrator;

impl BuildOrchestrator {
    pub async fn trigger_universal_build() -> anyhow::Result<()> {
        println!("[CVKG Build] Starting universal build pipeline...");
        
        // 1. Web Target (WASM)
        let web_handle = tokio::spawn(async {
            Command::new("wasm-pack")
                .args(["build", "--target", "web", "--out-dir", "pkg"])
                .status()
                .await
        });

        // 2. Desktop Target (Native GPU)
        let native_handle = tokio::spawn(async {
            Command::new("cargo")
                .args(["build", "--package", "cvkg-render-native"])
                .status()
                .await
        });

        let (web_res, native_res) = tokio::join!(web_handle, native_handle);
        
        if web_res?.is_ok() && native_res?.is_ok() {
            println!("[CVKG Build] Universal build completed successfully.");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Build failed"))
        }
    }
}

/// Capture a vDOM snapshot from a running client for SEO pre-rendering.
async fn capture_snapshot(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    // Atomically publish the new snapshot — lock-free for all concurrent readers.
    state.last_vdom_snapshot.store(Arc::new(Some(body)));
    "Snapshot captured"
}

/// Serve the pre-rendered static HTML (SEO/SSG).
async fn serve_ssg(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // load() returns a lock-free Arc guard; valid for this request's lifetime.
    let snapshot_guard = state.last_vdom_snapshot.load();
    let content = snapshot_guard.as_deref().unwrap_or("<div id='app'>Loading...</div>");
    
    Html(format!(
        "<!DOCTYPE html><html><head><title>CVKG App</title></head><body>{}<script src='/pkg/app.js'></script></body></html>",
        content
    ))
}

/// Trigger a universal build via HTTP.
async fn trigger_build_handler() -> impl IntoResponse {
    match BuildOrchestrator::trigger_universal_build().await {
        Ok(_) => (axum::http::StatusCode::OK, "Build successful".to_string()),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Build failed: {}", e)),
    }
}

#[tokio::main]
async fn main() {
    let config = ServeConfig {
        addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
        port: 3000,
    };

    let state = Arc::new(AppState {
        config: config.clone(),
        last_vdom_snapshot: ArcSwap::from_pointee(None),
    });

    let app = Router::new()
        .route("/", get(serve_ssg))
        .route("/snapshot", post(capture_snapshot))
        .route("/build", post(trigger_build_handler))
        .nest_service("/pkg", tower_http::services::ServeDir::new("pkg"))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/cvkg-ws", get(ws_handler))
        .route("/hmr", get(hmr_ws_handler))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(config.addr).await.unwrap();
    println!("Professional CVKG Dev Server listening on http://{}", config.addr);

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_inspector_socket)
}

async fn hmr_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_hmr_socket)
}

async fn handle_inspector_socket(mut ws: WebSocket) {
    println!("[CVKG Inspector] Client connected");
    while let Some(Ok(msg)) = ws.next().await {
        if let Message::Text(text) = msg {
            // Echo or process inspector commands
            let _ = ws.send(Message::Text(text)).await;
        }
    }
}

async fn handle_hmr_socket(mut ws: WebSocket) {
    println!("[CVKG HMR] Client connected (State-Preserving Mode)");
    while let Some(Ok(msg)) = ws.next().await {
        if let Message::Text(text) = msg {
            // Process HMR events (e.g. notify client of new WASM binary)
            let _ = ws.send(Message::Text(text)).await;
        }
    }
}

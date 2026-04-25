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

//! Embedded WebKit dev server and app preview host
//!
//! This crate provides the development server and WebKit-based preview host for CVKG
//  applications, including hot reload functionality and inspector integration.

use futures_util::StreamExt;
use std::net::SocketAddr;

/// Configuration for the development server
#[derive(Debug, Clone)]
struct ServeConfig {
    addr: SocketAddr,
    #[allow(dead_code)]
    port: u16,
    #[allow(dead_code)]
    inspector: bool,
}

impl ServeConfig {
    #[doc(hidden)]
    fn new(port: u16, inspector: bool) -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
            port,
            inspector,
        }
    }
}

/// Serve the HTML host shell
async fn serve_shell() -> &'static str {
    "<!DOCTYPE html><html><body><div id='app'></div></body></html>"
}

/// WebSocket handler for inspector communication
async fn ws_handler(ws: axum::extract::ws::WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(handle_inspector_socket)
}

/// WebSocket handler for hot module reload
async fn hmr_ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(handle_hmr_socket)
}

#[tokio::main]
async fn main() {
    let config = ServeConfig::new(3000, true);

    let app = axum::Router::new()
        .route("/", axum::routing::get(serve_shell))
        .nest_service(
            "/app.wasm",
            tower_http::services::ServeFile::new("pkg/app.wasm"),
        )
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/cvkg-ws", axum::routing::get(ws_handler))
        .route("/hmr", axum::routing::get(hmr_ws_handler))
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(config.addr).await.unwrap();
    println!("Server listening on http://{}", config.addr);

    axum::serve(listener, app).await.unwrap();
}

/// Handle inspector WebSocket connection
async fn handle_inspector_socket(mut ws: axum::extract::ws::WebSocket) {
    println!("Inspector WebSocket client connected");

    while let Some(result) = ws.next().await {
        match result {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Handle inspector messages
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("Received inspector message: {}", message);

                    // Echo back for now - in a real implementation, this would
                    // process inspector commands and send back vDOM snapshots
                    let response = serde_json::json!({
                        "type": "inspector_response",
                        "payload": {
                            "message": "Inspector connected"
                        }
                    });

                    if let Ok(json_str) = serde_json::to_string(&response) {
                        let _ = ws.send(axum::extract::ws::Message::Text(json_str)).await;
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                println!("Inspector WebSocket client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("Inspector WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Handle HMR WebSocket connection
async fn handle_hmr_socket(mut ws: axum::extract::ws::WebSocket) {
    println!("HMR WebSocket client connected");

    while let Some(result) = ws.next().await {
        match result {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Handle HMR messages
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("Received HMR message: {}", message);

                    // Echo back for now - in a real implementation, this would
                    // handle hot module replacement
                    let response = serde_json::json!({
                        "type": "hmr_response",
                        "payload": {
                            "message": "HMR connected"
                        }
                    });

                    if let Ok(json_str) = serde_json::to_string(&response) {
                        let _ = ws.send(axum::extract::ws::Message::Text(json_str)).await;
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                println!("HMR WebSocket client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("HMR WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

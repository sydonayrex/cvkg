//! WebSocket Server
//! Multiplexed WebSocket server for runtime communication, DevTools, hot reload, and agent streams

use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;
use std::net::SocketAddr;
use tracing::{error, info};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared application state for the WebSocket server
#[derive(Clone)]
pub struct AppState {
    pub patch_tx: broadcast::Sender<WsMessage>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    Patch(super::patch_engine::RuntimePatch),
    State(super::dev_runtime::RuntimeStateSnapshot),
    Event(super::dev_runtime::RuntimeEvent),
    Devtools(serde_json::Value),
}

/// WebSocket handler for runtime communication
async fn runtime_ws(
    axum::extract::State(_state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_runtime_socket)
}

/// WebSocket handler for DevTools
async fn devtools_ws(
    axum::extract::State(_state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_devtools_socket)
}

/// WebSocket handler for hot reload
async fn hotreload_ws(
    axum::extract::State(state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_hotreload_socket(socket, state))
}

/// WebSocket handler for agent streams
async fn agent_ws(
    axum::extract::State(_state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_agent_socket)
}

/// Handle runtime WebSocket connection
async fn handle_runtime_socket(mut ws: WebSocket) {
    info!("Runtime WebSocket client connected");

    // Send initial handshake
    let _ = ws
        .send(Message::Text(
            serde_json::to_string(&serde_json::json!({
                "type": "handshake",
                "payload": {
                    "client": "runtime",
                    "capabilities": ["patch", "state", "event"]
                }
            }))
            .unwrap()
            .into(),
        ))
        .await;

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // Handle incoming runtime messages
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Process runtime messages
                    info!("Received runtime message: {}", message);
                }
            }
            Ok(Message::Binary(bin)) => {
                // Handle binary messages if needed
                info!("Received binary message of {} bytes", bin.len());
            }
            Ok(Message::Close(_)) => {
                info!("Runtime WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Handle DevTools WebSocket connection
async fn handle_devtools_socket(mut ws: WebSocket) {
    info!("DevTools WebSocket client connected");

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // Handle DevTools messages
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                    info!("Received DevTools message: {}", message);
                }
            }
            Ok(Message::Close(_)) => {
                info!("DevTools WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("DevTools WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Handle hot reload WebSocket connection
async fn handle_hotreload_socket(mut ws: WebSocket, state: AppState) {
    info!("Hot reload WebSocket client connected");

    let mut patch_rx = state.patch_tx.subscribe();

    loop {
        tokio::select! {
            // Listen for broadcasted patches from the build pipeline
            Ok(msg) = patch_rx.recv() => {
                if let Ok(serialized) = serde_json::to_string(&msg) {
                    if let Err(e) = ws.send(Message::Text(serialized.into())).await {
                        error!("Failed to send patch to client: {}", e);
                        break;
                    }
                }
            }
            // Listen for client messages or disconnects
            Some(result) = ws.next() => {
                match result {
                    Ok(Message::Close(_)) => {
                        info!("Hot reload WebSocket client disconnected");
                        break;
                    }
                    Err(e) => {
                        error!("Hot reload WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handle agent stream WebSocket connection
async fn handle_agent_socket(mut ws: WebSocket) {
    info!("Agent stream WebSocket client connected");

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                // Handle agent stream messages
                if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                    info!("Received agent message: {}", message);
                }
            }
            Ok(Message::Close(_)) => {
                info!("Agent stream WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("Agent stream WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Create the WebSocket router with all endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ws/runtime", get(runtime_ws))
        .route("/ws/devtools", get(devtools_ws))
        .route("/ws/hotreload", get(hotreload_ws))
        .route("/ws/agent", get(agent_ws))
        .with_state(state)
}

/// Start the WebSocket server
pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _) = broadcast::channel(100);
    let state = AppState {
        patch_tx: tx.clone(),
    };

    // Start the build pipeline watcher
    let tx_clone = tx.clone();
    let patch_engine = Arc::new(tokio::sync::Mutex::new(
        super::patch_engine::PatchEngine::new(),
    ));

    super::build_pipeline::BuildPipeline::watch_changes(".", move |artifact| {
        let mut engine = patch_engine.blocking_lock();
        let patch = engine.generate_patch(artifact);
        let _ = tx_clone.send(WsMessage::Patch(patch));
    });

    let app = create_router(state);
    info!("Starting WebSocket server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

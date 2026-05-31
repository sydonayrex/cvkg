//! WebSocket Server
//! Multiplexed WebSocket server for runtime communication, DevTools, hot reload, and agent streams

use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use serde::{Deserialize, Serialize};

use crate::patch_engine::{PatchEngine, RuntimePatch};

/// Shared application state for the WebSocket server
#[derive(Clone)]
pub struct AppState {
    pub patch_tx: broadcast::Sender<WsMessage>,
    pub patch_engine: Arc<std::sync::Mutex<PatchEngine>>,
}

/// WebSocket message protocol between CLI dev server and connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Apply a hot-reload patch to the running application.
    Patch(RuntimePatch),
    /// Full state snapshot request/response.
    State(crate::dev_runtime::RuntimeStateSnapshot),
    /// Agent or runtime event.
    Event(crate::dev_runtime::RuntimeEvent),
    /// DevTools message (bidirectional).
    Devtools(DevtoolsMessage),
    /// Handshake response sent to new clients.
    Handshake {
        client: String,
        capabilities: Vec<String>,
    },
}

/// DevTool command types (bidirectional: client → server and server → client).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DevtoolsMessage {
    /// Client-side command.
    Command(DevtoolsCommand),
    /// Server-side response/event.
    Response(serde_json::Value),
}

/// DevTools command types (client → server).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DevtoolsCommand {
    /// Request current performance metrics.
    QueryMetrics,
    /// Toggle the error overlay.
    ToggleOverlay { show: bool },
    /// Request the current scene graph.
    QueryGraph,
    /// Query accessibility properties for a given component path.
    QueryAccessibility {
        /// Dot-separated component path (e.g., "root.main.content.button-1").
        path: String,
    },
    /// Echo for health checking.
    Ping,
}

/// WebSocket handler for runtime communication
async fn runtime_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_runtime_socket(socket, state))
}

/// WebSocket handler for DevTools
async fn devtools_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_devtools_socket(socket, state))
}

/// WebSocket handler for hot reload
async fn hotreload_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_hotreload_socket(socket, state))
}

/// WebSocket handler for agent streams
async fn agent_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent_socket(socket, state))
}

/// Send a JSON message over the WebSocket, logging errors.
async fn send_ws(ws: &mut WebSocket, msg: &WsMessage) {
    match serde_json::to_string(msg) {
        Ok(json) => {
            if let Err(e) = ws.send(Message::Text(json)).await {
                error!("Failed to send WS message: {}", e);
            }
        }
        Err(e) => error!("Failed to serialize WS message: {}", e),
    }
}

/// Handle runtime WebSocket connection.
///
/// Processes incoming RuntimePatch, State, and Event messages from the runtime client.
/// Forwards patches through the broadcast channel so hot-reload clients receive them.
async fn handle_runtime_socket(mut ws: WebSocket, state: AppState) {
    info!("Runtime WebSocket client connected");

    // Send initial handshake
    send_ws(
        &mut ws,
        &WsMessage::Handshake {
            client: "runtime".to_string(),
            capabilities: vec!["patch".into(), "state".into(), "event".into()],
        },
    )
    .await;

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Patch(patch)) => {
                        info!(
                            "Runtime patch received: {:?}",
                            std::mem::discriminant(&patch)
                        );
                        // Forward patch to all hot-reload subscribers
                        let _ = state.patch_tx.send(WsMessage::Patch(patch));
                    }
                    Ok(WsMessage::Event(event)) => {
                        info!("Runtime event received: {:?}", event);
                        let _ = state.patch_tx.send(WsMessage::Event(event));
                    }
                    Ok(WsMessage::State(_snapshot)) => {
                        info!("Runtime state snapshot received");
                    }
                    Ok(other) => {
                        warn!("Unexpected message type on runtime WS: {:?}", other);
                    }
                    Err(e) => {
                        warn!("Failed to parse runtime message: {}", e);
                    }
                }
            }
            Ok(Message::Binary(bin)) => {
                info!(
                    "Received binary message of {} bytes on runtime WS",
                    bin.len()
                );
            }
            Ok(Message::Close(_)) => {
                info!("Runtime WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("Runtime WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Handle DevTools WebSocket connection.
///
/// Processes DevTools commands (QueryMetrics, ToggleOverlay, QueryGraph, Ping)
/// and sends back appropriate responses.
async fn handle_devtools_socket(mut ws: WebSocket, _state: AppState) {
    info!("DevTools WebSocket client connected");

    // Send initial handshake
    send_ws(
        &mut ws,
        &WsMessage::Handshake {
            client: "devtools".to_string(),
            capabilities: vec!["metrics".into(), "overlay".into(), "graph".into()],
        },
    )
    .await;

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<DevtoolsCommand>(&text) {
                    Ok(DevtoolsCommand::QueryMetrics) => {
                        let metrics = crate::devtools::capture_metrics();
                        let response = serde_json::json!({
                            "type": "metrics",
                            "fps": metrics.fps,
                            "frame_time_ms": metrics.frame_time_ms,
                            "node_count": metrics.node_count,
                            "edge_count": metrics.edge_count,
                            "gpu_memory_mb": metrics.gpu_memory_mb,
                        });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(response)),
                        )
                        .await;
                    }
                    Ok(DevtoolsCommand::ToggleOverlay { show }) => {
                        info!("DevTools overlay toggled: {}", show);
                        let response = serde_json::json!({
                            "type": "overlay_toggled",
                            "show": show,
                        });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(response)),
                        )
                        .await;
                    }
                    Ok(DevtoolsCommand::QueryGraph) => {
                        // Return empty graph for now — populated by build pipeline
                        let response = serde_json::json!({
                            "type": "graph",
                            "nodes": [],
                            "edges": [],
                        });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(response)),
                        )
                        .await;
                    }
                    Ok(DevtoolsCommand::QueryAccessibility { path }) => {
                        // Query accessibility properties for the given component path.
                        // In a real implementation, this would traverse the component tree
                        // and return the AriaProperties for the matched component.
                        let response = serde_json::json!({
                            "type": "accessibility",
                            "path": path,
                            "properties": {
                                "role": "button",
                                "label": "Sample Button",
                                "description": None::<String>,
                                "disabled": false,
                                "checked": None::<bool>,
                                "expanded": None::<bool>,
                                "hidden": false,
                                "shortcut": None::<String>,
                            },
                        });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(response)),
                        )
                        .await;
                    }
                    Ok(DevtoolsCommand::Ping) => {
                        let response = serde_json::json!({ "type": "pong" });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(response)),
                        )
                        .await;
                    }
                    Err(e) => {
                        warn!("Failed to parse DevTools message: {}", e);
                        let error = serde_json::json!({
                            "type": "error",
                            "message": format!("Invalid command: {}", e),
                        });
                        send_ws(
                            &mut ws,
                            &WsMessage::Devtools(DevtoolsMessage::Response(error)),
                        )
                        .await;
                    }
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

/// Handle hot reload WebSocket connection.
///
/// Broadcasts patches from the build pipeline to connected clients.
async fn handle_hotreload_socket(mut ws: WebSocket, state: AppState) {
    info!("Hot reload WebSocket client connected");

    let mut patch_rx = state.patch_tx.subscribe();

    // Send initial handshake
    send_ws(
        &mut ws,
        &WsMessage::Handshake {
            client: "hotreload".to_string(),
            capabilities: vec!["patch".into()],
        },
    )
    .await;

    loop {
        tokio::select! {
            Ok(msg) = patch_rx.recv() => {
                send_ws(&mut ws, &msg).await;
            }
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

/// Handle agent stream WebSocket connection.
///
/// Receives AgentEvent messages and forwards them through the broadcast channel.
async fn handle_agent_socket(mut ws: WebSocket, state: AppState) {
    info!("Agent stream WebSocket client connected");

    // Send initial handshake
    send_ws(
        &mut ws,
        &WsMessage::Handshake {
            client: "agent".to_string(),
            capabilities: vec!["event".into()],
        },
    )
    .await;

    while let Some(result) = ws.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<crate::dev_runtime::AgentEvent>(&text) {
                    Ok(event) => {
                        let runtime_event = crate::dev_runtime::RuntimeEvent::Agent(event);
                        let _ = state.patch_tx.send(WsMessage::Event(runtime_event));
                    }
                    Err(e) => {
                        // Try parsing as a raw RuntimeEvent
                        match serde_json::from_str::<crate::dev_runtime::RuntimeEvent>(&text) {
                            Ok(event) => {
                                let _ = state.patch_tx.send(WsMessage::Event(event));
                            }
                            Err(e2) => {
                                warn!(
                                    "Failed to parse agent message as AgentEvent ({}) or RuntimeEvent ({})",
                                    e, e2
                                );
                            }
                        }
                    }
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
        .route("/health", get(|| async { "OK" }))
        .route("/", get(serve_shell))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve a minimal HTML shell that connects back via WebSocket.
async fn serve_shell() -> impl IntoResponse {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CVKG Dev Server</title>
    <style>
        body { margin: 0; background: #0b0b14; color: #c0c0c8; font-family: 'JetBrains Mono', monospace; display: flex; align-items: center; justify-content: center; height: 100vh; }
        .status { text-align: center; }
        .status h1 { font-size: 24px; color: #00cccc; margin-bottom: 8px; }
        .status p { font-size: 14px; color: #6a6a8a; }
        .status .indicator { display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: #4a8a4a; margin-right: 6px; }
    </style>
</head>
<body>
    <div class="status">
        <h1>⚡ CVKG Dev Server</h1>
        <p><span class="indicator"></span>Connected — WebSocket hot reload active</p>
        <p style="margin-top: 16px; font-size: 12px;">Waiting for changes...</p>
    </div>
</body>
</html>"#,
    )
}

/// Path for the hot-reload state file.
const HOT_RELOAD_STATE_PATH: &str = ".cvkg/hot_reload_state.json";

/// Shared dashboard state, populated by the dev server and file watcher.
pub type DashboardState = Arc<std::sync::Mutex<crate::devtools_dashboard::GraphState>>;

/// Starts the file watcher and returns a broadcast sender for patches.
pub fn start_file_watcher(
    path: &str,
    patch_engine: Arc<std::sync::Mutex<crate::patch_engine::PatchEngine>>,
) -> broadcast::Sender<WsMessage> {
    use crate::build_pipeline::BuildPipeline;

    let (tx, _) = broadcast::channel(100);
    let tx_clone = tx.clone();
    let patch_engine = Arc::clone(&patch_engine);
    // Ensure the .cvkg directory exists for state persistence
    let _ = std::fs::create_dir_all(".cvkg");

    BuildPipeline::watch_changes(path, move |artifact| {
        // Update live metrics for the dashboard from the shared state
        if let Some(ds) = crate::devtools_dashboard::dashboard_state() {
            let guard = ds.lock().unwrap_or_else(|e| e.into_inner());
            crate::devtools::update_metrics(crate::devtools::PerfMetrics {
                frame_time_ms: guard.frame_time_ms,
                fps: if guard.frame_time_ms > 0.0 {
                    1000.0 / guard.frame_time_ms
                } else {
                    0.0
                },
                node_count: guard.nodes.len(),
                edge_count: guard.edges.len(),
                gpu_memory_mb: guard.gpu_memory_mb,
            });
        }

        // Save hot-reload state before applying the patch
        let state = crate::dev_runtime::HotReloadState {
            theme_mode: "dark".to_string(),
            window_size: (1200.0, 800.0),
            scroll_positions: std::collections::HashMap::new(),
            input_text: std::collections::HashMap::new(),
            expanded_nodes: std::collections::HashMap::new(),
            saved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        };
        if let Err(e) = state.save(std::path::Path::new(HOT_RELOAD_STATE_PATH)) {
            warn!("Failed to save hot-reload state: {}", e);
        }

        let mut engine = match patch_engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let patch = engine.generate_patch(artifact);
        let _ = tx_clone.send(WsMessage::Patch(patch));
    });

    // Attempt to load any previously saved state
    if std::path::Path::new(HOT_RELOAD_STATE_PATH).exists() {
        match crate::dev_runtime::HotReloadState::load(std::path::Path::new(HOT_RELOAD_STATE_PATH))
        {
            Ok(state) => {
                info!(
                    "Loaded hot-reload state from {} (theme: {}, saved_at: {})",
                    HOT_RELOAD_STATE_PATH, state.theme_mode, state.saved_at
                );
            }
            Err(e) => {
                debug!("No previous hot-reload state found: {}", e);
            }
        }
    }

    tx
}

/// Start the WebSocket server with graceful shutdown.
pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let patch_engine = Arc::new(std::sync::Mutex::new(PatchEngine::new()));
    let patch_tx = start_file_watcher(".", Arc::clone(&patch_engine));

    let state = AppState {
        patch_tx: patch_tx.clone(),
        patch_engine: Arc::clone(&patch_engine),
    };

    let app = create_router(state);
    info!("Starting WebSocket server on {} (Ctrl+C to stop)", addr);

    // Spawn animation tick task
    let animation_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(16)); // ~60fps
        let mut solver =
            cvkg_anim::SleipnirSolver::new(cvkg_anim::SleipnirParams::default(), 0.0, 0.0);
        let mut physics_world =
            cvkg_physics::PhysicsWorld::new(cvkg_physics::WorldConfig::default());
        loop {
            interval.tick().await;
            let dt = 0.016;
            // Tick the animation solver
            let _value = solver.tick(dt);
            // Tick the physics world
            physics_world.step(dt);
        }
    });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    animation_handle.abort();
    info!("CVKG dev server shut down gracefully.");
    Ok(())
}

/// Wait for Ctrl+C or SIGTERM.
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
            info!("Ctrl+C received, shutting down gracefully...");
        },
        _ = terminate => {
            info!("SIGTERM received, shutting down gracefully...");
        },
    }
}

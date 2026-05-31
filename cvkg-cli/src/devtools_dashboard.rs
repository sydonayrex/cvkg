// ── DevTools Dashboard ──────────────────────────────────────────────────────
//
// HTTP API server for inspecting the CVKG graph, themes, and real-time events.
// Uses axum for robust HTTP handling (replaces previous raw TCP implementation).

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};

/// Global dashboard state shared between the HTTP server and file watcher.
static DASHBOARD_STATE: OnceLock<Arc<Mutex<GraphState>>> = OnceLock::new();

/// Initialize the global dashboard state. Called once when the dashboard starts.
pub fn init_dashboard_state() -> Arc<Mutex<GraphState>> {
    let state = Arc::new(Mutex::new(GraphState::default()));
    let _ = DASHBOARD_STATE.set(state.clone());
    state
}

/// Get the global dashboard state for updating from the file watcher.
pub fn dashboard_state() -> Option<Arc<Mutex<GraphState>>> {
    DASHBOARD_STATE.get().cloned()
}

/// Configuration for the DevTools dashboard server.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub port: u16,
    pub open_browser: bool,
    #[allow(dead_code)]
    pub graph_state: Arc<Mutex<GraphState>>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 9731,
            open_browser: true,
            graph_state: Arc::new(Mutex::new(GraphState::default())),
        }
    }
}

/// Serializable graph state for the dashboard.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GraphState {
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<EdgeInfo>,
    pub themes: HashMap<String, Vec<f32>>,
    pub events: Vec<EventInfo>,
}

/// Node information for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct NodeInfo {
    pub id: u64,
    pub label: String,
    pub node_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Edge information for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct EdgeInfo {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub label: String,
}

/// Event information for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct EventInfo {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
}

/// Axum state type alias.
pub type AppState = Arc<Mutex<GraphState>>;

/// Starts the DevTools dashboard HTTP server using axum.
pub async fn start_dashboard(config: DashboardConfig) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    println!("🔧 CVKG DevTools Dashboard starting at http://{}", addr);

    if config.open_browser {
        let url = format!("http://{}", addr);
        println!("🌐 Opening browser at {}", url);
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }

    let state = init_dashboard_state();

    let app = Router::new()
        .route("/", get(serve_dashboard_html))
        .route("/api/graph", get(api_graph))
        .route("/api/nodes", get(api_nodes))
        .route("/api/edges", get(api_edges))
        .route("/api/themes", get(api_themes))
        .route("/api/events", get(api_events))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("✅ DevTools server listening on {}", addr);
    axum::serve(listener, app).await
}

/// Serve the main dashboard HTML page.
async fn serve_dashboard_html() -> impl IntoResponse {
    let html = include_str!("dashboard.html");
    (
        StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        html,
    )
}

/// API: full graph state.
async fn api_graph(State(state): State<AppState>) -> Json<GraphState> {
    let guard = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(guard.clone())
}

/// API: nodes list.
async fn api_nodes(State(state): State<AppState>) -> Json<Vec<NodeInfo>> {
    let guard = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(guard.nodes.clone())
}

/// API: edges list.
async fn api_edges(State(state): State<AppState>) -> Json<Vec<EdgeInfo>> {
    let guard = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(guard.edges.clone())
}

/// API: theme tokens.
async fn api_themes(State(state): State<AppState>) -> Json<HashMap<String, Vec<f32>>> {
    let guard = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(guard.themes.clone())
}

/// API: event log.
async fn api_events(State(state): State<AppState>) -> Json<Vec<EventInfo>> {
    let guard = state.lock().unwrap_or_else(|e| e.into_inner());
    Json(guard.events.clone())
}

// ── Public helper functions ──────────────────────────────────────────────────

/// Adds a node to the shared graph state.
pub fn add_node(state: &Arc<Mutex<GraphState>>, node: NodeInfo) {
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    s.nodes.push(node);
}

/// Adds an edge to the shared graph state.
pub fn add_edge(state: &Arc<Mutex<GraphState>>, edge: EdgeInfo) {
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    s.edges.push(edge);
}

/// Adds an event to the shared graph state.
pub fn add_event(state: &Arc<Mutex<GraphState>>, event_type: &str, message: &str) {
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    let timestamp = format!(
        "{:.3}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    );
    s.events.push(EventInfo {
        timestamp,
        event_type: event_type.to_string(),
        message: message.to_string(),
    });
    if s.events.len() > 100 {
        let trim = s.events.len() - 100;
        s.events.drain(0..trim);
    }
}

/// Updates a theme token in the shared graph state.
pub fn set_theme_token(state: &Arc<Mutex<GraphState>>, name: &str, rgba: [f32; 4]) {
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    s.themes.insert(name.to_string(), rgba.to_vec());
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.port, 9731);
        assert!(config.open_browser);
    }

    #[test]
    fn test_graph_state_default() {
        let state = GraphState::default();
        assert!(state.nodes.is_empty());
        assert!(state.edges.is_empty());
        assert!(state.themes.is_empty());
        assert!(state.events.is_empty());
    }

    #[test]
    fn test_add_node() {
        let state = Arc::new(Mutex::new(GraphState::default()));
        add_node(
            &state,
            NodeInfo {
                id: 1,
                label: "Test".into(),
                node_type: "process".into(),
                x: 10.0,
                y: 20.0,
                width: 120.0,
                height: 60.0,
            },
        );
        let s = state.lock().unwrap();
        assert_eq!(s.nodes.len(), 1);
        assert_eq!(s.nodes[0].id, 1);
    }

    #[test]
    fn test_add_edge() {
        let state = Arc::new(Mutex::new(GraphState::default()));
        add_edge(
            &state,
            EdgeInfo {
                id: 1,
                source: 1,
                target: 2,
                label: "flows".into(),
            },
        );
        let s = state.lock().unwrap();
        assert_eq!(s.edges.len(), 1);
    }

    #[test]
    fn test_add_event_trims() {
        let state = Arc::new(Mutex::new(GraphState::default()));
        for i in 0..150 {
            add_event(&state, "test", &format!("event {}", i));
        }
        let s = state.lock().unwrap();
        assert_eq!(s.events.len(), 100);
    }

    #[test]
    fn test_set_theme_token() {
        let state = Arc::new(Mutex::new(GraphState::default()));
        set_theme_token(&state, "primary", [0.5, 0.5, 1.0, 1.0]);
        let s = state.lock().unwrap();
        assert_eq!(s.themes.get("primary"), Some(&vec![0.5, 0.5, 1.0, 1.0]));
    }
}

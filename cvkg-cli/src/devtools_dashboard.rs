// ── DevTools Dashboard ──────────────────────────────────────────────────────
//
// Local HTTP server that serves an HTML dashboard for inspecting the CVKG
// graph, themes, and real-time events. Uses the existing tokio runtime and
// serves a self-contained HTML page with embedded CSS/JS.

use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Configuration for the DevTools dashboard server.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Port to listen on (default: 9731).
    pub port: u16,
    /// Whether to open the browser automatically.
    pub open_browser: bool,
    /// Graph state shared between the dashboard and the CLI.
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
    /// Nodes in the graph.
    pub nodes: Vec<NodeInfo>,
    /// Edges in the graph.
    pub edges: Vec<EdgeInfo>,
    /// Current theme tokens.
    pub themes: HashMap<String, Vec<f32>>,
    /// Recent events.
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

/// Starts the DevTools dashboard HTTP server.
///
/// Serves a self-contained HTML dashboard at the configured port.
/// The dashboard includes:
/// - SVG-based node graph visualization
/// - Theme preview panel
/// - Real-time event log
/// - Auto-refresh via polling
///
/// # Arguments
/// * `config` - Dashboard configuration.
///
/// # Returns
/// `Ok(())` when the server shuts down, or an `std::io::Error` on failure.
pub async fn start_dashboard(config: DashboardConfig) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    println!("🔧 CVKG DevTools Dashboard starting at http://{}", addr);

    if config.open_browser {
        let url = format!("http://{}", addr);
        println!("🌐 Opening browser at {}", url);
        // Attempt to open the browser; ignore errors if no browser is available
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("✅ DevTools server listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let graph_state = config.graph_state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, graph_state).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

/// Handles a single HTTP connection.
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    graph_state: Arc<Mutex<GraphState>>,
) -> Result<(), std::io::Error> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;

    if n == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buf[..n]);
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }

    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    let response = match (method, path) {
        ("GET", "/") => serve_dashboard_html(),
        ("GET", "/api/graph") => serve_graph_json(&graph_state),
        ("GET", "/api/nodes") => serve_nodes_json(&graph_state),
        ("GET", "/api/edges") => serve_edges_json(&graph_state),
        ("GET", "/api/themes") => serve_themes_json(&graph_state),
        ("GET", "/api/events") => serve_events_json(&graph_state),
        _ => not_found(),
    };

    let mut stream = stream;
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Serves the main dashboard HTML page.
fn serve_dashboard_html() -> String {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>CVKG DevTools Dashboard</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: 'JetBrains Mono', 'Fira Code', monospace; background: #0b0b14; color: #c0c0c8; height: 100vh; display: flex; flex-direction: column; }
.header { background: #141428; padding: 12px 20px; border-bottom: 1px solid #2a2a4a; display: flex; align-items: center; gap: 12px; }
.header h1 { font-size: 14px; color: #7a7aff; font-weight: 600; }
.header .status { font-size: 11px; color: #4a8a4a; margin-left: auto; }
.main { display: flex; flex: 1; overflow: hidden; }
.sidebar { width: 280px; background: #10101f; border-right: 1px solid #2a2a4a; overflow-y: auto; padding: 12px; }
.sidebar h2 { font-size: 11px; text-transform: uppercase; letter-spacing: 1px; color: #6a6a8a; margin-bottom: 8px; }
.node-list, .edge-list { list-style: none; }
.node-list li, .edge-list li { padding: 6px 8px; margin-bottom: 2px; border-radius: 4px; font-size: 11px; cursor: pointer; }
.node-list li:hover, .edge-list li:hover { background: #1a1a3a; }
.node-list li .id { color: #7a7aff; margin-right: 6px; }
.node-list li .type { color: #4a8a4a; font-size: 10px; }
.content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.graph-view { flex: 1; position: relative; overflow: hidden; }
.graph-view svg { width: 100%; height: 100%; }
.graph-view .node-rect { fill: #1a1a3a; stroke: #3a3a6a; stroke-width: 1; rx: 4; }
.graph-view .node-label { fill: #c0c0c8; font-size: 10px; text-anchor: middle; }
.graph-view .edge-line { stroke: #3a3a6a; stroke-width: 1.5; fill: none; }
.graph-view .edge-label { fill: #6a6a8a; font-size: 9px; text-anchor: middle; }
.bottom-panel { height: 180px; background: #10101f; border-top: 1px solid #2a2a4a; display: flex; }
.theme-panel { flex: 1; padding: 12px; border-right: 1px solid #2a2a4a; overflow-y: auto; }
.theme-panel h2 { font-size: 11px; text-transform: uppercase; letter-spacing: 1px; color: #6a6a8a; margin-bottom: 8px; }
.color-swatch { display: inline-block; width: 24px; height: 24px; border-radius: 4px; margin: 2px; border: 1px solid #2a2a4a; }
.event-log { flex: 1; padding: 12px; overflow-y: auto; }
.event-log h2 { font-size: 11px; text-transform: uppercase; letter-spacing: 1px; color: #6a6a8a; margin-bottom: 8px; }
.event-log .entry { font-size: 10px; padding: 3px 0; border-bottom: 1px solid #1a1a2a; }
.event-log .entry .time { color: #5a5a7a; margin-right: 8px; }
.event-log .entry .type { color: #7a7aff; margin-right: 6px; }
</style>
</head>
<body>
<div class="header">
    <h1>⚡ CVKG DevTools</h1>
    <span class="status" id="status">● Connected</span>
</div>
<div class="main">
    <div class="sidebar">
        <h2>Nodes</h2>
        <ul class="node-list" id="node-list"></ul>
        <h2 style="margin-top:16px">Edges</h2>
        <ul class="edge-list" id="edge-list"></ul>
    </div>
    <div class="content">
        <div class="graph-view" id="graph-view">
            <svg id="graph-svg"></svg>
        </div>
        <div class="bottom-panel">
            <div class="theme-panel" id="theme-panel">
                <h2>Theme Tokens</h2>
                <div id="color-swatches"></div>
            </div>
            <div class="event-log" id="event-log">
                <h2>Event Log</h2>
                <div id="event-entries"></div>
            </div>
        </div>
    </div>
</div>
<script>
const API = '';
let state = { nodes: [], edges: [], themes: {}, events: [] };

async function fetchJSON(path) {
    try {
        const r = await fetch(API + path);
        return await r.json();
    } catch (e) {
        console.error('Fetch error:', e);
        return null;
    }
}

function renderGraph() {
    const svg = document.getElementById('graph-svg');
    const view = document.getElementById('graph-view');
    const w = view.clientWidth || 800;
    const h = view.clientHeight || 400;

    let svgContent = `<svg width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">`;

    // Draw edges
    state.edges.forEach(edge => {
        const src = state.nodes.find(n => n.id === edge.source);
        const tgt = state.nodes.find(n => n.id === edge.target);
        if (!src || !tgt) return;
        const sx = src.x + src.width / 2;
        const sy = src.y + src.height / 2;
        const tx = tgt.x + tgt.width / 2;
        const ty = tgt.y + tgt.height / 2;
        const mx = (sx + tx) / 2;
        const my = (sy + ty) / 2 - 30;
        svgContent += `<path class="edge-line" d="M${sx},${sy} Q${mx},${my} ${tx},${ty}" />`;
        svgContent += `<text class="edge-label" x="${mx}" y="${my - 5}">${edge.label || ''}</text>`;
    });

    // Draw nodes
    state.nodes.forEach(node => {
        svgContent += `<rect class="node-rect" x="${node.x}" y="${node.y}" width="${node.width}" height="${node.height}" />`;
        svgContent += `<text class="node-label" x="${node.x + node.width / 2}" y="${node.y + node.height / 2 + 4}">${node.label}</text>`;
    });

    svgContent += '</svg>';
    svg.outerHTML = svgContent;
}

function renderSidebar() {
    const nodeList = document.getElementById('node-list');
    nodeList.innerHTML = state.nodes.map(n =>
        `<li><span class="id">#${n.id}</span>${n.label}<span class="type">${n.node_type}</span></li>`
    ).join('');

    const edgeList = document.getElementById('edge-list');
    edgeList.innerHTML = state.edges.map(e =>
        `<li><span class="id">#${e.id}</span>${e.source} → ${e.target}</li>`
    ).join('');
}

function renderThemes() {
    const container = document.getElementById('color-swatches');
    container.innerHTML = Object.entries(state.themes).map(([name, rgba]) => {
        const [r, g, b, a] = rgba;
        const color = `rgba(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)},${a})`;
        return `<div class="color-swatch" style="background:${color}" title="${name}"></div>`;
    }).join('');
}

function renderEvents() {
    const container = document.getElementById('event-entries');
    container.innerHTML = state.events.slice(-50).map(e =>
        `<div class="entry"><span class="time">${e.timestamp}</span><span class="type">${e.event_type}</span>${e.message}</div>`
    ).join('');
}

async function refresh() {
    const graph = await fetchJSON('/api/graph');
    if (graph) {
        state = graph;
        renderGraph();
        renderSidebar();
        renderThemes();
        renderEvents();
        document.getElementById('status').textContent = '● Connected';
        document.getElementById('status').style.color = '#4a8a4a';
    } else {
        document.getElementById('status').textContent = '● Disconnected';
        document.getElementById('status').style.color = '#8a4a4a';
    }
}

refresh();
setInterval(refresh, 2000);
window.addEventListener('resize', renderGraph);
</script>
</body>
</html>"#;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    )
}

/// Serves the full graph state as JSON.
fn serve_graph_json(state: &Arc<Mutex<GraphState>>) -> String {
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&*state).unwrap_or_else(|_| "{}".to_string());
    json_response(&json)
}

/// Serves the nodes list as JSON.
fn serve_nodes_json(state: &Arc<Mutex<GraphState>>) -> String {
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&state.nodes).unwrap_or_else(|_| "[]".to_string());
    json_response(&json)
}

/// Serves the edges list as JSON.
fn serve_edges_json(state: &Arc<Mutex<GraphState>>) -> String {
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&state.edges).unwrap_or_else(|_| "[]".to_string());
    json_response(&json)
}

/// Serves the theme tokens as JSON.
fn serve_themes_json(state: &Arc<Mutex<GraphState>>) -> String {
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&state.themes).unwrap_or_else(|_| "{}".to_string());
    json_response(&json)
}

/// Serves the event log as JSON.
fn serve_events_json(state: &Arc<Mutex<GraphState>>) -> String {
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let json = serde_json::to_string(&state.events).unwrap_or_else(|_| "[]".to_string());
    json_response(&json)
}

/// Builds a JSON HTTP response with CORS headers.
fn json_response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

/// Builds a 404 Not Found response.
fn not_found() -> String {
    let body = r#"{"error": "Not Found"}"#;
    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

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
    // Keep only the last 100 events
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

    #[test]
    fn test_json_response() {
        let resp = json_response(r#"{"ok":true}"#);
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("application/json"));
        assert!(resp.contains(r#"{"ok":true}"#));
    }

    #[test]
    fn test_not_found() {
        let resp = not_found();
        assert!(resp.contains("404 Not Found"));
    }

    #[test]
    fn test_dashboard_html_contains_key_elements() {
        let html = serve_dashboard_html();
        assert!(html.contains("CVKG DevTools"));
        assert!(html.contains("/api/graph"));
        assert!(html.contains("renderGraph"));
        assert!(html.contains("setInterval"));
    }
}

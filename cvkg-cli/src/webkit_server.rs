//!
//! WebKit Preview Server
//! Serves the host shell and WASM bundle for app preview
//!

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

/// Configuration for the WebKit preview server.
pub struct WebKitConfig {
    /// Path to the WASM file to serve.
    pub wasm_path: String,
    /// Path to the JS glue file to serve.
    pub js_path: String,
    /// Directory for assets.
    pub assets_dir: String,
    /// Directory for static files.
    pub static_dir: String,
}

impl Default for WebKitConfig {
    fn default() -> Self {
        Self {
            wasm_path: "dist/pkg/app_bg.wasm".to_string(),
            js_path: "dist/pkg/app.js".to_string(),
            assets_dir: "dist/assets".to_string(),
            static_dir: "dist/static".to_string(),
        }
    }
}

/// Start the WebKit preview server.
///
/// # Arguments
/// * `addr` — Socket address to bind to.
/// * `config` — Server configuration with paths to WASM, JS, assets, and static files.
pub async fn start_server_with_config(
    addr: SocketAddr,
    config: WebKitConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(serve_shell))
        .nest_service("/app.wasm", ServeFile::new(config.wasm_path))
        .nest_service("/app.js", ServeFile::new(config.js_path))
        .nest_service("/assets", ServeDir::new(config.assets_dir))
        .nest_service("/static", ServeDir::new(config.static_dir))
        .layer(CorsLayer::permissive());

    log::info!("Starting WebKit preview server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Start the WebKit preview server with default paths.
///
/// Uses default paths: `dist/pkg/app_bg.wasm`, `dist/pkg/app.js`, etc.
pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    start_server_with_config(addr, WebKitConfig::default()).await
}

async fn serve_shell() -> &'static str {
    "<!DOCTYPE html><html><head><meta charset='utf-8'><title>CVKG Preview</title></head><body><div id='app'></div><script type='module'>import init from './app.js'; init();</script></body></html>"
}

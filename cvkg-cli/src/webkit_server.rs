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

/// Start the WebKit preview server
pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(serve_shell))
        .nest_service("/app.wasm", ServeFile::new("pkg/wgpu/niflheim_web_demo_bg.wasm"))
        .nest_service("/app.js", ServeFile::new("pkg/wgpu/niflheim_web_demo.js"))
        .nest_service("/assets", ServeDir::new("cvkg-webkit-server/assets"))
        .nest_service("/static", ServeDir::new("cvkg-webkit-server/static"))
        .layer(CorsLayer::permissive());

    log::info!("Starting WebKit preview server on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_shell() -> &'static str {
    "<!DOCTYPE html><html><head><meta charset='utf-8'><title>CVKG Preview</title></head><body><div id='app'></div><script type='module'>import init from './app.js'; init();</script></body></html>"
}
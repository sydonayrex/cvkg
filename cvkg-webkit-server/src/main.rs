//! CVKG Webkit Server - Cyber-Viking OS Host Binary Runner

#![allow(unused_imports, deprecated)]

use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use cvkg_webkit_server::router::{
    AppState, Config, create_router, shutdown_signal, spawn_file_watcher,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env if present.
    let _ = dotenvy::dotenv();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

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
    let metric_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .ok();

    let (hmr_tx, _) = tokio::sync::broadcast::channel(16);

    let state = Arc::new(AppState::new(config.clone(), hmr_tx));
    info!("[CVKG] Authentication token active: {}", state.auth_token);

    // Spawn the background file watcher task
    spawn_file_watcher(state.clone());

    // Build the router with middleware layers.
    let app = create_router(state, metric_handle);

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
    use axum::response::IntoResponse;
    use cvkg_webkit_server::router::{liveness_handler, readiness_handler, system_time_handler, SystemTime};

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

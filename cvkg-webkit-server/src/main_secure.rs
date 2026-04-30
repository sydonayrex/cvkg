// Security-hardened version of main.rs for cvkg-webkit-server
// This module contains security improvements while maintaining backward compatibility

use arc_swap::ArcSwap;
use axum::{
    body::Body,
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::process::Command;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use wasm_server::NativeWasmServer;

// ============================================================================
// Security: Custom Error Types for Proper Error Handling
// ============================================================================

/// Application errors with proper context
#[derive(Debug)]
pub enum AppError {
    /// Path validation failed (directory traversal attempt)
    InvalidPath { path: String },
    /// Authentication failed
    Unauthorized,
    /// Rate limit exceeded
    RateLimited,
    /// Internal server error
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidPath { path } => write!(f, "Invalid path: access denied"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::RateLimited => write!(f, "Rate limit exceeded"),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::InvalidPath { .. } => StatusCode::FORBIDDEN,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl std::error::Error for AppError {}

// ============================================================================
// Security: Input Validation Functions
// ============================================================================

/// Validates that a path doesn't escape the allowed base directory
/// Prevents directory traversal attacks
pub fn validate_path(base: &Path, target: &Path) -> Result<PathBuf, AppError> {
    let canonical_base = base.canonicalize()
        .map_err(|e| AppError::Internal(format!(
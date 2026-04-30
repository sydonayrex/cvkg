// CVKG WebKit Production Server - Security Hardened Version
// This file contains the security-hardened version of main.rs

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

//! # CVKG WebKit Production Server
//!
//! Serves CVKG web apps with automatic rendering backend negotiation.
//!
//! ## Security Notes
//! - All file paths are validated to prevent directory traversal
//! - Admin endpoints require API key authentication
//! - CORS is restricted to explicit whitelist
//! - All errors are handled gracefully without panics

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

mod wasm_server;
use wasm_server::NativeWasmServer;

// ============================================================================
// Security: Custom Error Types for Proper Error Handling
// ============================================================================

/// Application errors with proper context for production-safe error handling
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
            AppError::InvalidPath { .. } => write!(f, 
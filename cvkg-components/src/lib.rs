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
//!
//! Sources:
//!   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//!   CVKG Extended: Section 2 of the CVKG Design Specification

//! Built-in component library for CVKG
//!
//! This crate implements standard CVKG components using public CVKG APIs.

// --- Shared Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Bold,
    Italic,
}

pub use cvkg_core::Color;

// Re-export submodules
pub mod container;
pub mod devtools;
pub mod error;
pub mod grid;
pub mod image;
pub mod interactive;
pub mod memory;
pub mod niflheim_demo;
pub mod primitive;
pub mod richtext;
pub mod visual;

pub use container::*;
pub use devtools::*;
pub use error::*;
pub use grid::*;
pub use image::*;
pub use interactive::*;
pub use memory::*;
pub use niflheim_demo::*;
pub use primitive::*;
pub use richtext::*;
pub use visual::*;

// Re-export layout components
pub use cvkg_layout as layout;

// Internal Never type for primitive views
#[doc(hidden)]
pub use cvkg_core::Never;
pub use cvkg_core::Orientation;

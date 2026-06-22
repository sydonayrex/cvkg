//! # CVKG Agentic Development Guidelines (v1.3)
//!
//! All AI agents contributing to this crate MUST follow ALL eight rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–8) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.
//! 8. HARDWARE VERIFIED -- NEVER declare success based on mock data/rendering for native crates.
//!                      Any change to input, rendering, or lifecycle MUST be verified via physical
//!                      loopback (e.g., cargo run -p berserker) and signal path tracing.
//!
//! Sources:
//! Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//! CVKG Extended: Section 14 of the CVKG Design Specification (v1.3)

#![allow(
    unused_imports,
    clippy::single_component_path_imports,
    dead_code,
    clippy::items_after_test_module,
    clippy::field_reassign_with_default,
    clippy::collapsible_if,
    clippy::unnecessary_map_or,
    clippy::needless_return
)]

//! Platform-native widget delegation using winit and AccessKit
//!
//! This crate provides platform-specific rendering backends for native desktop targets
//! using winit for window/event handling and AccessKit for accessibility tree integration.

pub mod audio;
pub mod asset_manager;
pub mod window;
pub mod main_loop;
pub mod events;
pub mod contracts;
pub mod regression;
pub mod renderer;

#[cfg(test)]
mod tests;

// Re-export public interface for backward compatibility
pub use audio::{RodioAudioEngine, VisualHapticEngine};
pub use asset_manager::NativeAssetManager;
pub use window::{
    WindowState, WindowStateDetector, ResizeHitTest, SafeAreaInsets,
    NativeWindowWrapper, WindowManager, WindowData, WindowType,
    WindowCapabilityMatrix, MonitorConfig, MultiMonitorManager,
};
pub use main_loop::{AppEvent, ShieldWall};
pub use events::{convert_keyboard_event, convert_ime_event, convert_mouse_event, load_icon};
pub use contracts::{
    RenderingMode, TranslationContract, TranslationContractRegistry,
    SyncDirection, StateSyncContract, StateSyncRegistry,
    WidgetVirtualizationConfig, SemanticRoleMapping, SemanticRoleRegistry,
};
pub use regression::VisualRegressionTracker;
pub use renderer::{NativeRenderer, GPU_FRAME_PTR};

//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! The View trait is the fundamental building block of CVKG. Every UI element -- from a plain text label
//! to a complex navigation controller -- is a View. The trait is intentionally minimal; complexity emerges
//! through modifier composition.
//!
//! # Conformance rules:
//! 1. `body()` must be pure and side-effect free
//! 2. Primitive views use `Never` as `Body` and register a `PaintCommand` directly with the scene graph
//! 3. `View` types must implement `Send` but not necessarily `Sync`, enabling safe multi-threaded layout passes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod error_types;
pub mod future_views;
pub mod security;

pub use future_views::{HologramView, ParticleEmitter, StreamingText};

// P1-13: extracted modules
pub mod asset;
pub mod dependency;
pub mod error_boundary;
pub mod knowledge;
pub mod renderer;
pub mod undo;
pub mod virtual_list;
pub mod window;

// P1-13: re-exports for backward compatibility
pub use asset::{AssetKey, AssetState, DesignTokens, TokenValue};
pub use dependency::{DependencyGraph, FrameBudgetTracker, InputLatencyTracker, SubsystemBudget};
pub use error_boundary::{ComponentErrorState, ErrorBoundary};
pub use knowledge::{
    AnnouncementPriority, AppState, KnowledgeFragment, KnowledgeId, LAYOUT_DIRTY, MemoryLayer,
    SYSTEM_STATE, TemporalEdge, TemporalNode, UiFidelityLevel, begin_render_phase,
    end_render_phase, fallback_runtime, get_system_state, is_rendering, load_system_state,
    set_rendering, snapshot_system_state, update_system_state,
};
pub use undo::{UndoGroup, UndoManager};
pub use window::{Window, WindowCloseAction, WindowConfig, WindowHandle, WindowId, WindowLevel};

pub mod view;
pub use view::*;
pub mod aria;
pub use aria::*;

pub mod keyboard;
pub use keyboard::*;

pub mod focus;
pub use focus::*;

// =============================================================================
// REDUCED MOTION
// =============================================================================

/// Detects OS-level reduced motion preference via [`AccessibilityPreferences`].
///
/// This delegates to `AccessibilityPreferences::detect_from_system()` which
/// queries the correct OS API on macOS, Linux, and Windows.
pub mod reduced_motion;
pub use reduced_motion::*;

/// An object-safe version of the View trait for type erasure.
pub mod erased_view;
pub use erased_view::*;

/// SharedElementModifier enables shared-element transitions.
/// When two views share the same Bifrost Bridge ID, the Sleipnir solver will
/// interpolate their geometry and effects (blur, glow) during the transition.
pub mod modifiers;
pub use modifiers::*;
pub mod render_base;
pub use render_base::*;

/// The Renderer trait defines the atomic drawing operations for all CVKG backends.
/// This trait is object-safe and used by the View::render system.
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
///    Trait providing timing information for the render loop.
pub mod renderer_trait;
pub use renderer::*;
pub use renderer_trait::*;
pub mod accessibility;
pub use accessibility::*;
/// Defines the hardware acceleration tier and feature set available to the renderer.
pub mod render_tier;
pub use render_tier::*;
pub mod mesh;
pub use mesh::*;

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 0.0, 10.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            fov_y: 45.0f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            perspective: true,
            aspect: 16.0 / 9.0,
        }
    }
}

impl Camera3D {
    /// Compute the view matrix (world → camera space).
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_lh(self.position, self.target, self.up)
    }

    /// Compute the projection matrix.
    pub fn projection_matrix(&self) -> glam::Mat4 {
        if self.perspective {
            glam::Mat4::perspective_lh(self.fov_y, self.aspect, self.near, self.far)
        } else {
            // Orthographic with fov_y as half-height
            let top = self.fov_y;
            let right = top * self.aspect;
            glam::Mat4::orthographic_lh(-right, right, -top, top, self.near, self.far)
        }
    }

    /// Compute the combined view-projection matrix.
    pub fn view_projection(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// FrameRenderer extends Renderer with frame lifecycle management.
/// It is typically implemented by the host windowing/rendering environment.
pub mod spring;
pub use spring::*;
pub mod frame_renderer;
pub use frame_renderer::*;
pub mod state;
pub use state::*;
pub mod env_core;
pub use env_core::*;
pub mod env;
pub use env::*;
/// Geometry modifiers
/// Size of the view in logical pixels
pub mod geometry_modifiers;
#[allow(ambiguous_glob_reexports)]
pub use geometry_modifiers::*;
pub mod layout;
pub use layout::*;

// Size and FrameRenderer are pub items in this module; no re-export alias needed.

pub mod agents;
pub mod animation;
pub mod gpu;
pub mod material;
pub mod runtime;
pub mod scene_graph;
pub mod sdf_shadow;
pub mod shadow;

// Re-export commonly used types
pub use layout::{LayoutCache, LayoutKey, LayoutView, Rect, SizeProposal};
pub use material::DrawMaterial;
pub use scene_graph::{NodeId, bifrost_registry};
pub mod color;
pub mod data_table;
pub mod elevation;
pub mod form_validation;
pub mod responsive;
pub use color::SemanticColors;

// Duplicate AssetState removed - original definition at line 67

/// AssetManager defines the interface for loading and caching external resources.
pub mod event;
pub use event::*;
pub mod suspense;
pub use suspense::*;
/// Berserker mode states for the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderIntensityMode {
    Normal,
    Rage,    // Red tint, slight shake
    Frenzy,  // Heavy red tint, motion blur, aggressive shake
    GodMode, // Golden aura, lightning arcs
}

/// Seer trait for AI-assisted UI components.
/// Allows components to receive "prophecies" (predictions) from an AI backend.
pub trait Seer: Send + Sync {
    /// Provide a prediction for the next user action or content.
    fn predict(&self, context: &str) -> String;
    /// Stream real-time "whispers" (transcriptions/intent).
    fn whispers(&self) -> Vec<String>;
}

pub mod theme;
pub use theme::{
    ThemeContext, glassmorphism_enabled, set_current_theme, use_theme, use_theme_context,
};

pub mod hooks;
pub use hooks::*;

pub mod a11y_prefs;
pub use a11y_prefs::*;
pub mod clipboard;
pub use clipboard::*;

// =============================================================================
// TEXT INPUT -- Direction enum for cursor movement
// =============================================================================

pub mod text_input;
pub use text_input::*;

/// Action details for interactive buttons inside a notification.
pub mod notifications;
pub use notifications::*;
pub mod file_dialog;
pub use file_dialog::*;
pub mod document;
pub use document::*;
pub mod menu;
pub use menu::*;
pub mod localization;
pub use localization::*;

pub mod system_theme;
pub use system_theme::*;
// AUDIO / HAPTIC -- Item 14: Spatial Audio / Haptic Feedback
// =============================================================================
// OS-agnostic: pure trait abstractions. Platform backends via cfg in renderer.

pub mod audio_haptic;
pub use audio_haptic::{
    AudioEngine, HapticEngine, HapticIntensity, NullAudioEngine, NullHapticEngine, haptic_error,
    haptic_impact, haptic_selection, haptic_success, play_sound, set_audio_engine,
    set_haptic_engine, sounds,
};

// =============================================================================
// PARALLAX -- Depth-based scroll offset system
// =============================================================================

pub mod parallax;
pub use parallax::{DisplayEnvironment, ParallaxModifier, PerformanceContract, Tier3Fallback};

pub mod identity;
pub use identity::*;

pub mod simple_geom;
pub use simple_geom::*;
pub mod dirty_flags;
pub use dirty_flags::*;

// =========================================================================
// P1-15: Subscriber List Mutex Poisoning
// =========================================================================
//
// Regression tests for the audit finding: a single panicking subscriber
// would poison the Mutex and break all future state updates forever.
// The fix wraps each callback in catch_unwind, so panics are isolated
// and logged without affecting other subscribers or future updates.

// =========================================================================
// P1-17: Suspense::new_async Shared Fallback Runtime
// =========================================================================
//
// Regression tests for the audit finding: when no ambient tokio
// runtime exists, new_async spawned a new OS thread + runtime per
// call. The fix introduces a process-wide shared fallback runtime.

pub mod dirty_region;
pub use dirty_region::*;

// =========================================================================
// P1-43: FrameBudget -- global frame budget contract
// =========================================================================
//
// The P1-43 audit found that no global frame budget contract
// exists. Individual subsystems may exceed their time allocation
// without coordination. P0-2 already handles per-frame
// degradation (skipping non-essential passes when over budget)
// but doesn't coordinate allocation across subsystems.
//
// This struct provides the foundation for future frame budget
// coordination. It tracks wall-clock time per frame and per
// subsystem, and allows callers to check whether a subsystem
// is within its allocated time slice.
//
// Currently a passive observer. Future work would add:
//  - Per-subsystem time allocation
//  - Automatic QualityLevel adjustment when over budget
//  - Integration with the renderer's frame loop
pub mod virtual_window;
pub use virtual_window::*;

// Test infrastructure -- MockRenderer and test-call recording
pub mod testing;

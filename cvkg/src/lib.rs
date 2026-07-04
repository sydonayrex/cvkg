//! # CVKG Rendering Pipelines
//!
//! When building an application with CVKG, you MUST explicitly select ONE and ONLY ONE rendering pipeline
//! via your `Cargo.toml` features. Do not mix rendering pipelines in a single application.

// Compile-time enforcement: mutually exclusive features
#[cfg(all(feature = "gpu", feature = "native", feature = "web"))]
compile_error!(
    "cvkg: select exactly one rendering pipeline feature (`gpu`, `native`, or `web`), not multiple. See the module-level docs in cvkg/src/lib.rs."
);
#[cfg(all(feature = "gpu", feature = "native"))]
compile_error!("cvkg: `gpu` and `native` features are mutually exclusive. Pick one.");
#[cfg(all(feature = "gpu", feature = "web"))]
compile_error!("cvkg: `gpu` and `web` features are mutually exclusive. Pick one.");
#[cfg(all(feature = "native", feature = "web"))]
compile_error!("cvkg: `native` and `web` features are mutually exclusive. Pick one.");

// ## 1. GPU Rendering (Feature: `gpu`)
// High-performance, direct GPU rendering using `wgpu`. This provides the full "Cyberpunk Viking" aesthetic
// with shaders (Surtr/Muspelheim), frosted glass (`bifrost`), and complex geometry.
// Use this for high-fidelity native games or data-heavy tactical dashboards.
//
// ## 2. Native Primitive Rendering (Feature: `native`)
// Uses `winit` and `AccessKit` to wrap the `gpu` renderer for cross-platform desktop applications.
// This is the default choice for standard desktop GUIs that need windowing and accessibility.
//
// ## 3. Web/WASM VDOM Rendering (Feature: `web`)
// Compiles your UI to WebAssembly and renders using a Virtual DOM translated to HTML/CSS.
// Use this to deploy your CVKG application to the browser.
//
// # Example `Cargo.toml` Selection:
// ```toml
// # Select only one feature for your target platform:
// cvkg = { version = "0.1.10", features = ["native"] }
// ```

pub use cvkg_anim as anim;
pub use cvkg_components as components;
pub use cvkg_core as core;
pub use cvkg_layout as layout;
pub use cvkg_scene as scene;
pub use cvkg_themes as themes;

// --- Rendering Pipelines (Mutually Exclusive by Design) ---

#[cfg(feature = "gpu")]
pub use cvkg_render_gpu as render;

#[cfg(feature = "native")]
pub use cvkg_render_native as native;

#[cfg(feature = "web")]
pub use cvkg_render_gpu as web;

#[cfg(feature = "framemanifest")]
cvkg_core::merge_manifests! {
    cvkg_physics::MANIFEST,
    cvkg_flow::MANIFEST,
    cvkg_render_gpu::MANIFEST,
}

/// Configure a frame scheduler using the merged FrameManifest budget requests.
///
/// Sets per-phase time budgets from all crate manifests. Currently a no-op
/// until `FrameScheduler` exposes a set_phase_budget() API.
#[cfg(feature = "framemanifest")]
pub fn configure_scheduler(_scheduler: &mut cvkg_scheduler::FrameScheduler) {
    #[allow(unused_imports)]
    use cvkg_core::{FramePhase, TimeBudgetRequest};

    // Placeholder for future budget API integration.
    // Once FrameScheduler::set_phase_budget() is available:
    // for budget in MERGED_BUDGET_REQUESTS {
    //     scheduler.set_phase_budget(budget.phase, budget.time_slice_us);
    // }
}

pub mod headless;
pub use headless::{CvkgHeadless, HeadlessFrame, HeadlessOptions};

pub mod prelude {
    // === Macros (always needed) ===
    pub use cvkg_macros::{View, view_component};

    // === Core types (always needed) ===
    pub use cvkg_core::{
        AnyView, AppState, AssetKey, AssetState, Binding, ComponentErrorState, Never, Rect, State,
        View,
    };

    // === Color type ===
    pub use cvkg_components::Color;

    // === Re-export cvkg-components prelude (standard English names) ===
    pub use cvkg_components::prelude::*;

    // === Animation triggers ===
    pub use cvkg_components::animation_triggers::{AnimationTriggers, TriggerSpring};

    // === Gradients ===
    pub use cvkg_components::gradient::{GradientStop, LinearGradient, RadialGradient};
}

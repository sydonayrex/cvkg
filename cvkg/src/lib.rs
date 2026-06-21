//! # CVKG Rendering Pipelines
//!
//! When building an application with CVKG, you MUST explicitly select ONE and ONLY ONE rendering pipeline
//! via your `Cargo.toml` features. Do not mix rendering pipelines in a single application.
//!
//! ## 1. GPU Rendering (Feature: `gpu`)
//! High-performance, direct GPU rendering using `wgpu`. This provides the full "Cyberpunk Viking" aesthetic
//! with shaders (Surtr/Muspelheim), frosted glass (`bifrost`), and complex geometry.
//! Use this for high-fidelity native games or data-heavy tactical dashboards.
//!
//! ## 2. Native Primitive Rendering (Feature: `native`)
//! Uses `winit` and `AccessKit` to wrap the `gpu` renderer for cross-platform desktop applications.
//! This is the default choice for standard desktop GUIs that need windowing and accessibility.
//!
//! ## 3. Web/WASM VDOM Rendering (Feature: `web`)
//! Compiles your UI to WebAssembly and renders using a Virtual DOM translated to HTML/CSS.
//! Use this to deploy your CVKG application to the browser.
//!
//! # Example `Cargo.toml` Selection:
//! ```toml
//! # Select only one feature for your target platform:
//! cvkg = { version = "0.1.10", features = ["native"] }
//! ```

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

pub mod prelude {
    // === Macros (always needed) ===
    pub use cvkg_macros::{View, view_component};

    // === Core types (always needed) ===
    pub use cvkg_core::{
        AppState, AssetKey, AssetState, Binding, ComponentErrorState, Never, Rect, State, View,
    };

    // === Color type ===
    pub use cvkg_components::Color;

    // === Layout primitives (most common imports) ===
    pub use cvkg_components::{FlexBox, Grid, HStack, ScrollView, VStack};

    // === Common interactive components ===
    pub use cvkg_components::{Button, Checkbox, Input, Select, Slider, Text};

    // === English API aliases for all Norse-named components ===
    // These type aliases point to the canonical (Norse) implementation.
    // Both names are valid; the English name is preferred for new code.
    pub use cvkg_components::{
        Accordion, Alert, Analytics, Avatar, ColorPicker, CommandPalette, CreativeTools,
        Decoder, Dialog, HolographicDisplay, HUD, Indicator, Messenger, Orb, Pagination,
        Progress, PromptBuilder, Rating, Sheet, Spinner, Splitter, StepIndicator, Tabs,
        Timeline, Tooltip, TreeView, Well, Window,
    };
}

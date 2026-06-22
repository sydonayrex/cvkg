//! Core module — fundamental types and traits for CVKG.
//!
//! This module contains the View trait, Renderer trait, focus management,
//! theme system, input handling, animation, and accessibility primitives.

// Submodule declarations
pub mod animation;
pub mod aria;
pub mod a11y_prefs;
pub mod audio_haptic;
pub mod clipboard;
pub mod color;
pub mod events;
pub mod focus;
pub mod identity;
pub mod invalidation;
pub mod keyboard;
pub mod localization;
pub mod state;
pub mod system_theme;
pub mod tests;
pub mod text_input;
pub mod theme;
pub mod uniforms;
pub mod virtual_list;

// Re-export key types for backward compatibility
pub use animation::*;
pub use aria::*;
pub use a11y_prefs::*;
pub use audio_haptic::*;
pub use clipboard::*;
pub use color::*;
pub use events::*;
pub use focus::*;
pub use identity::*;
pub use invalidation::*;
pub use keyboard::*;
pub use localization::*;
pub use state::*;
pub use system_theme::*;
pub use text_input::*;
pub use theme::*;
pub use uniforms::*;
pub use virtual_list::*;

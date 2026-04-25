//! # Niflheim Mist Demo
//!
//! A high-fidelity demonstration of the CVKG Phase 6 aesthetics:
//! - **Bifrost**: Frosted glass backdrop blur (Mist of Niflheim)
//! - **Gungnir**: Neon cyan glow
//! - **Ginnungagap**: Deep void background

use crate::primitive::{Shape, Text};
use cvkg_core::{StyleResolver, View};

/// Returns a view demonstrating the Niflheim/Bifrost aesthetic.
pub fn niflheim_demo() -> impl View {
    // Resolve tokens from the Yggdrasil environment
    let nifl_cyan = StyleResolver::color("primary");
    let _muspel_magenta = StyleResolver::color("secondary");

    // A Berserker-themed card
    Shape::rounded_rect(24.0)
        // Apply the Mist of Niflheim (25px blur, 65% opacity)
        .bifrost(25.0, 1.2, 0.65)
        // Apply Gungnir Neon Glow (NiflCyan, 15px radius)
        .gungnir(nifl_cyan, 15.0, 1.2)
        .mjolnir_slice(12.0, 0.0) // Geometric Mjolnir cut
}

/// A more complex composite demo
pub fn berserker_card() -> impl View {
    let nifl_cyan = StyleResolver::color("primary");

    Text::new("BERSERKER PROTOCOL")
        .gungnir(nifl_cyan, 10.0, 2.0)
        .bifrost(10.0, 1.0, 0.9)
}

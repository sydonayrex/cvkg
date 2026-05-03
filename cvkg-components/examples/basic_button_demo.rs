// Basic Button Demo - Copy-Paste Runnable
// Run with: cargo run --example basic_button_demo -p cvkg-components --features native

use cvkg_core::{Rect, Renderer};
use cvkg_components::Button;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
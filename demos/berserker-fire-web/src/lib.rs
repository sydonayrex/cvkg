//! Berserker Fire Demo -- Web (WASM) target.
//!
//! This crate compiles as a cdylib for wasm32-unknown-unknown.
//! Full wasm integration is not yet implemented.
//! The desktop demo at ../berserker/ is the primary demonstration target.
//!
//! This crate exists as a placeholder for future wasm support.

#![allow(dead_code, unused_variables, unused_imports)]

// Placeholder types for future wasm integration
pub struct Particle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub color: [f32; 4],
    pub life: f32,
    pub size: f32,
    pub is_ember: bool,
}

pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }
    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state & 0x7FFFFFFF) as f32 / 2147483647.0
    }
}

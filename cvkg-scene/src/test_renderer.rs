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
//! 12.                     at least THREE TIMES before making any edit or revision.
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

use cvkg_core::{ElapsedTime, FrameRenderer, Rect, Renderer};
use serde::{Deserialize, Serialize};

/// Command represents a single drawing operation recorded by the TestRenderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    BeginFrame,
    EndFrame,
    FillRect {
        rect: Rect,
        color: [f32; 4],
    },
    FillRoundedRect {
        rect: Rect,
        radius: f32,
        color: [f32; 4],
    },
    FillEllipse {
        rect: Rect,
        color: [f32; 4],
    },
    StrokeRect {
        rect: Rect,
        color: [f32; 4],
        stroke_width: f32,
    },
    StrokeRoundedRect {
        rect: Rect,
        radius: f32,
        color: [f32; 4],
        stroke_width: f32,
    },
    StrokeEllipse {
        rect: Rect,
        color: [f32; 4],
        stroke_width: f32,
    },
    DrawLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: [f32; 4],
    },
    DrawTexture {
        texture_id: u32,
        rect: Rect,
    },
    DrawImage {
        path: String,
        rect: Rect,
    },
    LoadImage {
        name: String,
    },
    PushClipRect {
        rect: Rect,
    },
    PopClipRect,
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,
    Bifrost {
        rect: Rect,
        blur: f32,
        saturation: f32,
        opacity: f32,
    },
    Gungnir {
        rect: Rect,
        color: [f32; 4],
        radius: f32,
        intensity: f32,
    },
    PushMjolnirSlice {
        angle: f32,
        offset: f32,
    },
    PopMjolnirSlice,
    PushTransform {
        translation: [f32; 2],
        scale: [f32; 2],
        rotation: f32,
    },
    PopTransform,
}

/// TestRenderer implements the Renderer trait but only records commands.
/// Used for cross-backend consistency verification and snapshot testing.
pub struct TestRenderer {
    pub commands: Vec<Command>,
}

impl Default for TestRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRenderer {
    /// Create a new TestRenderer
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl ElapsedTime for TestRenderer {
    fn delta_time(&self) -> f32 {
        0.016 // Fixed 60fps for testing
    }

    fn elapsed_time(&self) -> f32 {
        0.0 // Constant 0 for test snapshots
    }
}

impl Renderer for TestRenderer {

    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]) {
        self.commands.push(Command::FillRect { rect, color });
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]) {
        self.commands.push(Command::FillRoundedRect {
            rect,
            radius,
            color,
        });
    }

    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]) {
        self.commands.push(Command::FillEllipse { rect, color });
    }

    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.commands.push(Command::StrokeRect {
            rect,
            color,
            stroke_width,
        });
    }

    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32) {
        self.commands.push(Command::StrokeRoundedRect {
            rect,
            radius,
            color,
            stroke_width,
        });
    }

    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32) {
        self.commands.push(Command::StrokeEllipse {
            rect,
            color,
            stroke_width,
        });
    }

    fn draw_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        stroke_width: f32,
    ) {
        self.commands.push(Command::DrawLine {
            x1,
            y1,
            x2,
            y2,
            color,
            stroke_width,
        });
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.commands.push(Command::DrawText {
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
    }

    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        (text.len() as f32 * size * 0.6, size)
    }

    fn draw_texture(&mut self, texture_id: u32, rect: Rect) {
        self.commands
            .push(Command::DrawTexture { texture_id, rect });
    }

    fn draw_image(&mut self, path: &str, rect: Rect) {
        self.commands.push(Command::DrawImage {
            path: path.to_string(),
            rect,
        });
    }

    fn load_image(&mut self, name: &str, _data: &[u8]) {
        self.commands.push(Command::LoadImage {
            name: name.to_string(),
        });
    }

    fn push_clip_rect(&mut self, rect: Rect) {
        self.commands.push(Command::PushClipRect { rect });
    }

    fn pop_clip_rect(&mut self) {
        self.commands.push(Command::PopClipRect);
    }

    fn push_opacity(&mut self, opacity: f32) {
        self.commands.push(Command::PushOpacity { opacity });
    }

    fn pop_opacity(&mut self) {
        self.commands.push(Command::PopOpacity);
    }

    fn bifrost(&mut self, rect: Rect, blur: f32, saturation: f32, opacity: f32) {
        self.commands.push(Command::Bifrost {
            rect,
            blur,
            saturation,
            opacity,
        });
    }

    fn gungnir(&mut self, rect: Rect, color: [f32; 4], radius: f32, intensity: f32) {
        self.commands.push(Command::Gungnir {
            rect,
            color,
            radius,
            intensity,
        });
    }

    fn push_mjolnir_slice(&mut self, angle: f32, offset: f32) {
        self.commands
            .push(Command::PushMjolnirSlice { angle, offset });
    }

    fn pop_mjolnir_slice(&mut self) {
        self.commands.push(Command::PopMjolnirSlice);
    }

    fn push_transform(&mut self, translation: [f32; 2], scale: [f32; 2], rotation: f32) {
        self.commands.push(Command::PushTransform {
            translation,
            scale,
            rotation,
        });
    }

    fn pop_transform(&mut self) {
        self.commands.push(Command::PopTransform);
    }
}

impl FrameRenderer<()> for TestRenderer {
    fn begin_frame(&mut self) {
        self.commands.push(Command::BeginFrame);
    }

    fn end_frame(&mut self, _encoder: ()) {
        self.commands.push(Command::EndFrame);
    }
}

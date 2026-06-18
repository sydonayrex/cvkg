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

//! CVKG Materials — data models for Glass, Mica, Acrylic, and Elevation effects.
//!
//! # Why this exists
//! Finding #7 from the crosscrate audit: material types were scattered across
//! cvkg-core and cvkg-flow with no canonical shared definition. Backends
//! (cvkg-render-gpu, cvkg-compositor) need a single authoritative source for
//! material parameters that does not create circular dependencies.
//!
//! # Design
//! All types in this crate are pure data structs with no GPU code.
//! Backends (cvkg-render-gpu) consume these types and produce GPU resources.
//! `cvkg-core::DrawMaterial` (routing enum) and `cvkg-flow::GlassNodeMaterial`
//! (flow-specific, OKLCH-aware) remain in their respective crates; this crate
//! provides the canonical parameter bundles that flow into the render pipeline.

pub mod acrylic;
pub mod elevation;
pub mod glass;
pub mod mica;

pub use acrylic::AcrylicMaterial;
pub use elevation::{ElevationLevel, ElevationShadow};
pub use glass::GlassMaterial;
pub use mica::MicaMaterial;

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

//! CVKG Scheduler — Frame and task scheduling for platform-wide update ordering.
//!
//! # Architecture
//! The scheduler solves Finding #4 from the crosscrate audit: multiple crates
//! (VDOM, Layout, Animation, Physics, Render, Telemetry) all need ordered,
//! prioritized update execution. Without a scheduler, systems execute immediately
//! and in arbitrary order, risking stale data and wasted GPU work.
//!
//! # Frame Pipeline Order
//! Each frame executes tasks in this fixed order:
//! 1. Critical — must complete before the frame can proceed (input, state)
//! 2. High — layout and animation (required for correct render)
//! 3. Normal — general work (component updates)
//! 4. Idle — deferred work (telemetry flush, prefetch)

pub mod frame;
pub mod task;

pub use frame::FrameScheduler;
pub use task::{Priority, Task, TaskHandle, TaskScheduler};

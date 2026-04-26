# CYBER VIKING GUI X (CVKG)
## Design Specification & Agentic AI Development Document
### v1.2 — Karpathy Guidelines + CVKG Extended Agent Protocols

---

| Field | Value |
|---|---|
| Document Type | Agentic AI System Design Specification |
| Project Acronym | CVKG — Cyber Viking GUI X |
| Revision | 1.2 — Karpathy Guidelines + CVKG Extended Agent Protocols |
| Date | April 2026 |
| Status | DRAFT — For AI Implementation |
| Language | Rust (primary), WASM, WebGPU |
| Inspiration Projects | water-rs/waterui · OpenSwiftUIProject/OpenSwiftUI |
| Coding Guidelines | multica-ai/andrej-karpathy-skills (Karpathy Guidelines) + CVKG Extended Protocols (Guidelines 5–7) |

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Agentic AI Coding Guidelines](#2-agentic-ai-coding-guidelines)
   - [2.1 Guideline 1: Think Before Coding](#21-guideline-1-think-before-coding)
   - [2.2 Guideline 2: Simplicity First](#22-guideline-2-simplicity-first)
   - [2.3 Guideline 3: Surgical Changes](#23-guideline-3-surgical-changes)
   - [2.4 Guideline 4: Goal-Driven Execution](#24-guideline-4-goal-driven-execution)
   - [2.5 Guideline 5: Triple-Pass Context Review Before Edits](#25-guideline-5-triple-pass-context-review-before-edits)
   - [2.6 Guideline 6: Mandatory Code Comments for All Major Functions](#26-guideline-6-mandatory-code-comments-for-all-major-functions)
   - [2.7 Guideline 7: Progress Monitoring & Circular Loop Prevention](#27-guideline-7-progress-monitoring--circular-loop-prevention)
   - [2.8 Full Guidelines Summary Card](#28-full-guidelines-summary-card)
3. [Architecture Overview](#3-architecture-overview)
4. [Core Framework Design](#4-core-framework-design-critical)
5. [Component Library](#5-component-library-cvkg-components-critical)
6. [Rendering Backends](#6-rendering-backends)
7. [CLI Toolchain](#7-cli-toolchain-cvkg-cli-critical)
8. [Theme Engine & Design System](#8-theme-engine--design-system-cvkg-themes)
9. [Animation System](#9-animation-system-cvkg-anim)
10. [Accessibility](#10-accessibility-critical)
11. [Testing Strategy](#11-testing-strategy)
12. [Agentic AI Implementation Phases](#12-agentic-ai-implementation-phases)
13. [Key Dependencies & Rationale](#13-key-dependencies--rationale)
14. [Glossary](#14-glossary)

---

## 1. Executive Summary

Cyber Viking GUI X (CVKG) is a declarative, reactive, cross-platform GUI framework written entirely in Rust. It draws architectural and visual inspiration from Apple's SwiftUI paradigm — as explored in open-source projects [waterui](https://github.com/water-rs/waterui) and [OpenSwiftUI](https://github.com/OpenSwiftUIProject/OpenSwiftUI) — while extending their scope to encompass every major rendering surface: native OS widgets, desktop GPU rendering, WebGPU for browser-based rendering, and WebAssembly (WASM) for full in-browser deployment.

The framework is governed by a single unified component model. An application written once in CVKG compiles and runs with contextually appropriate rendering on each target. CVKG is not a thin wrapper over existing toolkits; it owns its rendering stack from the scene graph down to GPU draw calls on desktop, and from the virtual DOM to browser paint calls on the web.

### 1.1 Core Design Philosophy

- **Write once, render everywhere** — one Rust codebase, all surfaces
- **SwiftUI-inspired declarative syntax** adapted for Rust's ownership model
- **Reactive state management** with fine-grained differential re-rendering
- **Zero unsafe code** in public API surface; unsafe confined to renderer backends
- **First-class developer ergonomics**: hot reload, WASM virtual DOM inspection, CLI scaffolding
- **Progressive rendering model**: fall back gracefully from WebGPU → WebGL → Canvas → DOM
- **Disciplined agentic development**: all AI agents contributing to CVKG MUST follow all seven Agentic AI Coding Guidelines defined in Section 2, covering pre-coding discipline, simplicity, surgical edits, goal verification, triple-pass context review, mandatory documentation, and loop prevention

---

## 2. Agentic AI Coding Guidelines

> **Mandatory for all AI agents contributing to CVKG.** Guidelines 1–4 are derived from [Andrej Karpathy's observations](https://x.com/karpathy/status/2015883857489522876) on common LLM coding pitfalls, as codified in the [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills/blob/main/skills/karpathy-guidelines/SKILL.md) repository. Guidelines 5–7 are CVKG-specific extended agent protocols that address code review discipline, documentation standards, and runtime progress monitoring. Every agentic system implementing CVKG SHALL adhere to all seven guidelines without exception. Violations are treated as implementation defects.

> **Tradeoff note:** These guidelines bias toward caution over speed. For genuinely trivial tasks (e.g., renaming a local variable), use judgment. For any task touching public API surfaces, cross-crate interfaces, or renderer backends, these guidelines are non-negotiable.

---

### 2.1 Guideline 1: Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before writing any implementation code for a CVKG subsystem, the agentic system MUST:

- **State assumptions explicitly.** If the specification is ambiguous, surface the ambiguity in a comment block at the top of the relevant module before proceeding. Do not silently pick an interpretation.
- **Present multiple interpretations** when they exist. For example, if the layout engine spec could mean either a strict two-pass or a lazy single-pass, name both options, state the tradeoffs (memory, frame latency, implementation complexity), and select one with a documented rationale.
- **Push back when simpler approaches exist.** If the specification calls for a capability that can be solved with 30 lines instead of a 300-line abstraction, implement the simpler version and document why. Do not gold-plate to seem thorough.
- **Stop and name confusion.** If a specification section is genuinely unclear (e.g., the exact semantics of a Binding through a NavigationStack), halt that subsystem's implementation, emit a `// CVKG-CLARIFY:` comment block describing the confusion precisely, and move on to unblocked work.

**Applied to CVKG specifically:**

The renderer backend trait (`CvkgRenderer`) has intentional abstraction overhead. Before implementing a new backend, the agent SHALL write a plain-text design note (as a `//! # Design Note` doc comment at the top of the backend crate) explaining: which `CvkgRenderer` methods are trivially implemented, which require non-trivial work, and what platform-specific assumptions are being made. This note is part of the implementation deliverable, not optional commentary.

---

### 2.2 Guideline 2: Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- **No features beyond what was asked.** If Section 5 specifies a WebGL2 fallback renderer, do not also implement a Canvas 2D fallback during the same task — even if it seems like a natural extension. Implement exactly what is specified; propose additions separately.
- **No abstractions for single-use code.** If a helper function is called exactly once in the codebase, it should almost certainly be inlined. Trait abstractions in CVKG are justified only when two or more concrete implementors exist or are specified to exist.
- **No speculative configurability.** Do not add configuration flags, feature gates, or builder parameters that are not required by this specification. Every `pub` API surface is a maintenance commitment.
- **No error handling for impossible scenarios.** Rust's type system eliminates many failure modes; use it. Do not add `Result` return types to functions that cannot fail given valid inputs — use `-> T` and enforce validity at the boundary.
- **The 200-to-50 rule.** If an implementation exceeds 4× the line count a senior Rust engineer would write for the same behavior, rewrite it before committing. Verbosity is not thoroughness.

**Applied to CVKG specifically:**

The View trait body is intentionally minimal (see Section 4.1). Agents SHALL resist the urge to add convenience methods, default implementations, or associated types beyond those specified. The modifier system exists precisely so that complexity lives in composable, independently testable units — not on the core trait. Every proposed addition to the `View` trait requires a written justification in the PR description explaining why the modifier system cannot address the need.

**Self-check prompt for agents:** *"Would a senior Rust engineer say this is overcomplicated?"* If yes, simplify before proceeding.

---

### 2.3 Guideline 3: Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When implementing or modifying CVKG subsystems:

- **Do not "improve" adjacent code.** If implementing the WebGPU backend and noticing that the scene graph diff algorithm could be slightly more efficient, do not touch it. File it as a separate issue or `// CVKG-TODO:` comment.
- **Do not refactor things that aren't broken.** The crate structure defined in Section 3.2 is fixed for v1.0. Agents must not reorganize module hierarchies, rename public types, or restructure `Cargo.toml` files beyond what their assigned task requires.
- **Match existing style, even if you'd do it differently.** CVKG uses `rustfmt` defaults. Do not introduce custom formatting, reorder `use` statements, or change comment style in files outside the scope of the current task.
- **Mention, don't delete, unrelated dead code.** If an agent discovers dead code in an unrelated module, it SHALL emit a `// CVKG-DEAD:` comment and continue. Removal requires a dedicated cleanup task.
- **Own your orphans.** If an agent's changes make an `import`, `const`, or helper function unused, it MUST remove those — but only those created or made orphaned by its own changes. Pre-existing dead code is not the agent's responsibility.

**The line test:** Every changed line in a commit MUST trace directly to a requirement in this specification or a bug in a prior implementation. If a changed line cannot be traced, it should not be in the commit.

**Applied to CVKG specifically:**

Cross-crate changes are high-risk. If an agent must modify a `pub` API in `cvkg-core` while implementing `cvkg-components`, it MUST first verify that no other crate in the workspace depends on the old signature (via `cargo check --workspace`), document the API change in `CHANGELOG.md`, and implement the change in a single atomic commit separate from the feature work.

---

### 2.4 Guideline 4: Goal-Driven Execution

**Define success criteria. Loop until verified.**

Every implementation task MUST be expressed as a verifiable goal before work begins. Vague goals ("implement the layout engine") are not acceptable starting points. The agent SHALL restate each task in terms of concrete, checkable outcomes before writing any code.

**Transformation examples:**

| Vague Goal | Verifiable Goal |
|---|---|
| "Implement the View trait" | "The `View` trait compiles, `Never` implements it, `ModifiedView<V,M>` implements it, and `cargo test -p cvkg-core` passes with zero failures" |
| "Add WebGPU rendering" | "A hello-world CVKG app renders a colored rectangle in a browser via WASM+WebGPU, confirmed by visual screenshot in CI" |
| "Fix the layout bug" | "Write a regression test that reproduces the incorrect HStack spacing, then make it pass without touching unrelated code" |
| "Implement hot reload" | "Source change to a component triggers WASM rebuild in <8s, browser reflects change without page reload, state is preserved across reload" |

**Multi-step task planning format:**

For any task spanning more than one file or more than ~50 lines of new code, the agent SHALL produce a brief plan before coding:

```
Task: [name]
Goal: [verifiable outcome]

1. [Step] → verify: [cargo test / cargo check / visual check]
2. [Step] → verify: [cargo test / cargo check / visual check]
3. [Step] → verify: [cargo test / cargo check / visual check]
```

This plan SHALL be committed as a `PLAN.md` in the relevant crate's directory and deleted upon task completion. The plan is a working document, not permanent documentation.

**Looping behavior:** If a verification step fails, the agent MUST fix the failure before proceeding to the next step. It MUST NOT proceed past a failing `cargo check`, `cargo test`, or `cargo clippy --deny warnings`. Broken intermediate states must never be committed to `main`.

**Applied to CVKG phase gates:** Each implementation phase in Section 12 has an explicit completion criterion. The agentic system MUST treat these as hard gates. Phase N+1 work SHALL NOT begin until Phase N's completion criterion is met and verified by running the specified commands.

---

### 2.5 Guideline 5: Triple-Pass Context Review Before Edits

**Read before you write. Understand what surrounds the code you are about to change.**

Before making any edit or revision to an existing CVKG source file, the agentic system MUST perform a minimum of three distinct context review passes over the relevant code. This applies to all file types with substantial logic: `.rs`, `.ts`, `.h`, `.wgsl`, `.toml` (when modifying feature flags or dependencies), and any generated source files.

#### The Three Required Passes

**Pass 1 — Immediate Context (the target site)**
Read the function, block, or declaration being modified. Identify: what it does, what types it operates on, what it returns, and what invariants it assumes. Do not begin editing until this is fully understood.

**Pass 2 — Surrounding Context (before and after)**
Read the code immediately preceding and immediately following the target site — typically the full containing `impl` block, `mod` block, or file section. Identify: what calls into the target, what the target calls out to, what shared state or references are in scope, and what would break if the target's signature or behavior changed.

**Pass 3 — Cross-File Call Graph (linked dependencies)**
Trace all inbound and outbound call links to the target site. For Rust: search `cargo doc --open` or `rust-analyzer` references for all callers of the modified function. For TypeScript: check `tsserver` references. Identify every site in the codebase that would be affected by the proposed change. If the call graph is non-trivial (more than 3 callers or crosses a crate boundary), document it explicitly in a `// CVKG-CONTEXT:` comment at the top of the edit before applying any changes.

#### Enforcement in CVKG

- The three passes MUST be completed **before** any edit is applied, not during or after.
- If the agent is working in a large file (>300 lines), Pass 1 must include reading the full function, Pass 2 must include the full enclosing `impl` or `mod`, and Pass 3 must include a `cargo check --workspace` dry-run to surface cross-crate impact.
- For changes to public API functions in `cvkg-core`, `cvkg-scene`, or `cvkg-render-*`, all three passes are mandatory regardless of how small the change appears. A one-line type alias change in `cvkg-core` can cascade to all 13 crates.
- The agent SHALL NOT rely on memory of a prior read from earlier in its context window as a substitute for an active re-read. Code may have changed. Re-read it.

**Applied to CVKG renderer backends specifically:**

The `CvkgRenderer` trait methods have precise ordering semantics (e.g., `push_layer` must be balanced by `pop_layer`; `begin_frame` must precede all draw calls). Before modifying any renderer backend implementation, Pass 2 MUST include reading all call sites of `begin_frame` and `end_frame` to verify the invariant is preserved. Pass 3 MUST include verifying that the backend's `CvkgRenderer` impl still satisfies the trait's documented contracts after the change.

**Self-check for agents:** *"Have I read what calls this? Have I read what this calls? Have I read this at least three separate times?"* If any answer is no, stop and re-read before editing.

---

### 2.6 Guideline 6: Mandatory Code Comments for All Major Functions

**Document intent, not mechanics. Every major function in every code file must have a comment.**

All CVKG source files with executable logic SHALL have code comments for every major function, method, trait implementation, and non-trivial type. "Major" means any of the following: a `pub` function or method, a `pub(crate)` function used in more than one module, any `unsafe` block, any function exceeding 20 lines, any function with a non-obvious algorithm, or any function that participates in a public API.

This requirement applies to all file types that contain executable or declarative logic: `.rs`, `.ts`, `.h`, `.wgsl` (shader files), `.toml` (when containing non-obvious feature flag combinations), and Inspector protocol handler files.

#### Comment Standards by File Type

**Rust (`.rs`) — use `///` doc comments for all `pub` items, `//` for internal logic:**

```rust
/// Proposes a size to this view given the available space from the parent.
///
/// # Arguments
/// - `proposal`: The size the parent is offering. May be infinite on one
///   or both axes for greedy layouts (e.g., inside a ScrollView).
/// - `subviews`: Read-only references to child layout proxies.
/// - `cache`: Per-layout-pass scratch space; cleared between frames.
///
/// # Returns
/// The size this view wants to occupy. Must be <= proposal on both axes
/// unless the view is explicitly unconstrained.
///
/// # Panics
/// Panics in debug builds if `proposal` contains NaN values.
pub fn size_that_fits(
    &self,
    proposal: SizeProposal,
    subviews: &[LayoutSubview],
    cache: &mut LayoutCache,
) -> Size { ... }
```

**WGSL Shader Files (`.wgsl`) — comment every entry point and non-trivial function:**

```wgsl
// Vertex shader entry point for the rounded-rectangle fill pipeline.
// Inputs:  clip-space position, UV coordinates, instance color.
// Outputs: interpolated UV and color for the fragment stage.
// Note: corner rounding is performed in the fragment stage using
//       signed-distance field evaluation, not geometry subdivision.
@vertex
fn vs_rounded_rect(in: VertexInput) -> VertexOutput { ... }
```

**TypeScript (`.ts`) — Inspector UI and HMR client code:**

```typescript
/**
 * Connects to the CVKG Inspector WebSocket endpoint and begins
 * streaming vDOM snapshots. Reconnects automatically on disconnect
 * with exponential backoff (max 30s).
 *
 * @param url - WebSocket URL, typically ws://localhost:PORT/cvkg-ws
 * @param onSnapshot - Called on each incoming vDOM snapshot frame
 * @returns A disposable handle; call dispose() to close the connection
 */
export function connectInspector(
  url: string,
  onSnapshot: (snap: VDomSnapshot) => void
): InspectorHandle { ... }
```

#### Comment Quality Rules

- Comments MUST describe **why** and **what contract is upheld**, not mechanically restate the code. `// increments count` is not acceptable. `// count tracks the number of live VNode references; must not overflow u32 on 64-bit targets` is acceptable.
- Comments MUST be updated whenever the function's behavior, signature, or invariants change. A stale comment is treated as a documentation defect equivalent to a failing test.
- Comments MUST NOT be omitted on the grounds that the function "is obvious." Obviousness is context-dependent; the next agent reading this code has no context.
- `unsafe` blocks MUST include a `// SAFETY:` comment immediately preceding the block explaining precisely why the unsafe operation is sound — this is a Rust convention and is MANDATORY in CVKG with no exceptions.

#### Review Step: Comment Audit Before Commit

As part of the pre-commit checklist (see Guideline 5's Pass 3), the agent SHALL scan every modified file and verify that all new or modified major functions have compliant comments. If a pre-existing function in the same file lacks a comment and the agent's task touches that function, adding a comment is **the agent's responsibility** — this is one of the few cases where improving adjacent code is required, not prohibited (Guideline 3's "own your orphans" extended to documentation debt introduced by the current task's changes).

**Applied to CVKG phase gates:** Phase completion criteria in Section 12 explicitly include documentation coverage. A phase is not complete if `cargo doc --document-private-items` produces missing-doc warnings on any `pub` or `pub(crate)` item in a crate modified during that phase.

---

### 2.7 Guideline 7: Progress Monitoring & Circular Loop Prevention

**Know when you're spinning. Stop. Adjust. Move forward.**

Agentic systems are susceptible to circular execution patterns — issuing the same command repeatedly, making reversible edits, or oscillating between two broken states without converging on a solution. CVKG mandates explicit progress monitoring to detect and break these patterns before they consume significant time or produce irreversible file system changes.

#### 7.1 The 30-Second Monitoring Rule

**Every active tool call, shell command, or file operation MUST be evaluated for progress every 30 seconds of wall-clock execution time.**

Concretely, this means:

- If a `cargo build`, `wasm-pack build`, or any other compilation command has been running for more than 30 seconds without producing new output lines, the agent MUST check whether the process is genuinely working (expected for large builds) or stalled (no output, no CPU activity, awaiting a resource that will not arrive).
- If a command has been running for more than **5 minutes** without completing and the prior invocation of the same command completed in under 2 minutes, the agent MUST kill the process, diagnose the stall (check disk space, check for deadlocked file locks, check for a missing feature flag), and retry with a corrective adjustment.
- The agent MUST log each tool invocation and its outcome in a local `AGENT_LOG.md` at the workspace root. Each entry includes: timestamp, command/tool, expected outcome, actual outcome, and any corrective action taken.

#### 7.2 Circular Loop Detection

A **circular loop** is defined as any sequence of three or more consecutive actions that produce identical or semantically equivalent states without net forward progress. Common CVKG-specific loop patterns to watch for:

| Loop Pattern | Detection Signal | Required Action |
|---|---|---|
| Edit → compile error → revert → same edit | Same compiler error appears 3 times | Stop. Re-read the error. Apply Guideline 5 (triple-pass review). Try a fundamentally different fix. |
| `cargo check` passes → `cargo test` fails → patch → same test fails | Identical test failure 3 times | Stop. Add a `println!` or `dbg!` diagnostic. Understand root cause before patching. |
| WASM build succeeds → browser does not reflect change → rebuild → same result | HMR not delivering after 3 cycles | Stop. Check WebSocket connection. Verify wasm-bindgen output path. Restart the dev server fresh. |
| Shader compile passes → runtime GPU validation error → shader edit → same validation error | Same GPU error 3 consecutive frames | Stop. Read the full WGSL spec for the failing construct. Reduce to a minimal reproducer before editing the full shader. |
| `wasm-pack build` stalls at the same step | No stdout for >30s at the same build stage | Kill process, clear `pkg/` output directory, retry with `--dev` flag to skip optimization. |

#### 7.3 Progress Journal Protocol

For any task that spans more than 15 minutes of active execution, the agent SHALL maintain a `PROGRESS.md` file at the workspace root, updated at each meaningful checkpoint. Format:

```markdown
# CVKG Agent Progress Journal

Task: [task name]
Started: [timestamp]

--- [timestamp] — Step N ---
- Action: [what was done]
- Result: [what happened]
- Next: [what will be done next]
- Loops detected: [none | describe if applicable]
- Adjustment made: [none | describe if applicable]
```

This file is deleted upon task completion. It is a live working document — not permanent documentation — and serves as the agent's own audit trail for detecting when it has entered a loop.

#### 7.4 Escalation Protocol

If after **three distinct corrective adjustments** a loop persists, the agent MUST:

1. Stop all tool calls immediately.
2. Write a `BLOCKED.md` at the workspace root describing: the task, the loop pattern, all three corrective adjustments attempted, and a precise description of what human clarification or intervention is needed.
3. Emit a `// CVKG-BLOCKED:` comment at the relevant location in source code.
4. Halt work on that task and move to the next unblocked task in the PLAN.md sequence.

The agent MUST NOT silently give up, silently accept a broken state, or attempt a fourth speculative fix without human input. Silent failure is the worst outcome — a clearly documented `BLOCKED.md` allows a human or a different agent to resume with full context.

**Applied to CVKG CI:** The CI pipeline SHALL enforce a maximum job runtime of 20 minutes per phase. Any CI job exceeding this limit is automatically cancelled and flagged as a potential loop. The agent reviewing CI results MUST consult the build logs, apply the loop detection table above, and produce a root cause analysis before rerunning.

---

### 2.8 Full Guidelines Summary Card

The following card summarizes all seven guidelines and MUST be included as a doc comment block at the top of every CVKG crate's `lib.rs` or `main.rs`. It replaces the four-guideline card used in v1.1.

```rust
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
//!                      at least THREE TIMES before making any edit or revision.
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
```

---

## 3. Architecture Overview

CVKG is structured as a layered architecture with clean separation between the application layer (user code), the framework core, the scene graph, and the renderer backends. Layers communicate via defined trait boundaries — no layer reaches across another to access internal state.

### 3.1 Layer Diagram

```
┌─────────────────────────────────────────────────────┐
│           Application Layer (User Rust Code)        │
│   Views · State · Bindings · Modifiers · Scenes     │
├─────────────────────────────────────────────────────┤
│              CVKG Framework Core                    │
│   Component Tree · Diff Engine · Layout Engine      │
│   State Graph · Event Router · Animation System     │
├─────────────────────────────────────────────────────┤
│                 Scene Graph Layer                   │
│   Retained Mode Tree · Transform Hierarchy          │
│   Paint Commands · Clip Regions · Opacity Groups    │
├─────────────┬──────────────┬────────────────────────┤
│  Native     │  Desktop GPU │  Web Renderer          │
│  Backend    │  Backend     │  Backend               │
│  (winit +   │  (wgpu /     │  (WASM + WebGPU /      │
│  platform   │  WebGPU)     │  vDOM Inspector)       │
│  widgets)   │              │                        │
└─────────────┴──────────────┴────────────────────────┘
```

### 3.2 Crate Structure

The repository SHALL be organized as a Cargo workspace. Each crate has a single responsibility and depends only on crates strictly below it in the dependency hierarchy.

| Crate Name | Responsibility | Public API |
|---|---|---|
| `cvkg-core` | View protocol, state types, layout traits, modifier chain | Yes — stable API |
| `cvkg-scene` | Scene graph, retained tree, diff/patch engine | Internal + Inspector |
| `cvkg-layout` | Flexbox + constraint solver (Taffy integration) | Internal |
| `cvkg-anim` | Animation primitives, spring physics, transitions | Yes |
| `cvkg-render-native` | Platform-native widget delegation (winit, AccessKit) | Backend trait |
| `cvkg-render-gpu` | wgpu-based GPU renderer for desktop | Backend trait |
| `cvkg-render-web` | WASM WebGPU/WebGL renderer + vDOM bridge | Backend trait |
| `cvkg-vdom` | Virtual DOM implementation, diff, inspector protocol | Yes — Debug API |
| `cvkg-cli` | cvkg CLI toolchain, dev server, hot reload orchestrator | Binary |
| `cvkg-webkit-server` | Embedded WebKit dev server and app preview host | Binary/Lib |
| `cvkg-components` | Built-in component library (Button, Text, Stack, etc.) | Yes — stable API |
| `cvkg-themes` | Theme engine, design token system, dark/light mode | Yes |
| `cvkg-macros` | Procedural macros: `#[view]`, `#[state]`, `#[binding]` | Yes |

### 3.3 Rendering Target Matrix

| Target | Renderer Backend | Key Dependencies |
|---|---|---|
| macOS Native | cvkg-render-native (AppKit delegation) | winit, objc2 |
| Windows Native | cvkg-render-native (Win32 / DXGI) | winit, windows-rs |
| Linux Native | cvkg-render-native (Wayland/X11) | winit, smithay-client-toolkit |
| Desktop GPU | cvkg-render-gpu (wgpu, Vulkan/Metal/DX12) | wgpu, naga |
| Browser WebGPU | cvkg-render-web (WebGPU primary) | wasm-bindgen, web-sys |
| Browser WebGL | cvkg-render-web (WebGL2 fallback) | web-sys, glow |
| Browser Canvas | cvkg-render-web (2D Canvas fallback) | web-sys |
| WASM + vDOM | cvkg-vdom (virtual DOM + inspector) | cvkg-render-web, cvkg-vdom |
| Dev Preview | cvkg-webkit-server (WebKit shell) | cvkg-cli, axum, wry |

---

## 4. Core Framework Design [CRITICAL]

### 4.1 The View Protocol

The `View` trait is the fundamental building block of CVKG. Every UI element — from a plain text label to a complex navigation controller — is a `View`. The trait is intentionally minimal; complexity emerges through modifier composition.

> **Karpathy Guideline 2 enforcement:** The `View` trait must remain exactly as specified below. Agents MUST NOT add methods, associated types, or default implementations beyond this definition without a documented specification change.

```rust
// cvkg-core/src/view.rs

pub trait View: Sized + Send {
    /// The concrete type produced after applying modifiers.
    /// For primitive views this is Self.
    type Body: View;

    fn body(self) -> Self::Body;

    // Provided modifier entry point
    fn modifier<M: ViewModifier>(self, m: M) -> ModifiedView<Self, M> {
        ModifiedView::new(self, m)
    }
}

// Primitive (leaf) views implement Never as body
pub enum Never {}
impl View for Never {
    type Body = Never;
    fn body(self) -> Never { unreachable!() }
}
```

Conformance rules: (1) `body()` must be pure and side-effect free; (2) primitive views use `Never` as `Body` and register a `PaintCommand` directly with the scene graph; (3) `View` types must implement `Send` but not necessarily `Sync`, enabling safe multi-threaded layout passes.

### 4.2 State Management

CVKG uses a reactive state graph modeled on SwiftUI's property wrappers, expressed as Rust attributes via procedural macros. State ownership is explicit — every piece of mutable state has a single owner view, and child views receive read-only bindings or projections.

```rust
// State — owned, local to a view
#[state]
struct CounterState {
    count: i32,
    label: String,
}

// Binding — read/write reference to parent state
#[view]
fn Counter(mut state: State<CounterState>) -> impl View {
    VStack {
        Text(state.label.clone())
        Button("Increment") {
            state.count += 1;
            state.label = format!("Count: {}", state.count);
        }
    }
}

// Environment — ambient values propagated through tree
#[env_key(default = Theme::light())]
struct ThemeKey;
```

#### 4.2.1 State Graph Requirements

- State changes SHALL trigger a diff pass only on the subtree owned by the modified state node
- Bindings SHALL propagate changes upward; Environment values propagate downward
- The framework MUST prevent cycles in the state dependency graph at compile time where possible, and panic with a clear diagnostic at runtime where not
- State SHALL be serializable to JSON for dev tools inspection and hot-reload persistence

### 4.3 The Modifier System

Modifiers transform a view by wrapping it. Each modifier implements `ViewModifier` and produces a `ModifiedView<Inner, Self>`. Modifier chains are fully type-erased at the scene graph boundary.

```rust
pub trait ViewModifier: Sized + Send {
    fn modify<V: View>(self, content: V) -> impl View;
}

// Usage — chained modifiers
Text("Hello, CVKG")
    .font(Font::system(18.0).weight(Weight::Bold))
    .foreground_color(Color::from_hex("#1E5F9E"))
    .padding(EdgeInsets::all(12.0))
    .background(RoundedRectangle::corner_radius(8.0)
        .fill(Color::surface()))
    .shadow(Shadow::medium())
```

### 4.4 Layout Engine

Layout is a two-pass process: a propose-size/return-size protocol inspired by SwiftUI's size proposal system, implemented on top of the Taffy library for flexbox-compatible constraint solving.

```rust
pub trait Layout: Send {
    fn size_that_fits(&self,
        proposal: SizeProposal,
        subviews: &[LayoutSubview],
        cache: &mut LayoutCache,
    ) -> Size;

    fn place_subviews(&self,
        bounds: Rect,
        subviews: &mut [LayoutSubview],
        cache: &mut LayoutCache,
    );
}
```

Built-in layout containers required in `cvkg-components`: `HStack`, `VStack`, `ZStack`, `Grid`, `LazyVStack`, `LazyHStack`, and `ScrollView`.

---

## 5. Component Library (cvkg-components) [CRITICAL]

All components are implemented using public CVKG APIs — no internal shortcuts. Third-party component libraries operate on equal footing with built-ins.

> **Karpathy Guideline 3 enforcement:** When implementing a component, agents MUST NOT also refactor adjacent components, update unrelated modifiers, or "improve" the layout engine. Each component is a discrete, independently verifiable unit of work.

### 5.1 Primitive Views

| Component | Description | Key Properties |
|---|---|---|
| `Text` | Rendered text with rich inline formatting | font, weight, color, tracking, multiline |
| `Image` | Raster and vector images with aspect fill/fit | source, resizable, aspect-ratio, clip-shape |
| `Divider` | Horizontal or vertical rule | orientation, color, thickness |
| `Spacer` | Flexible space within stack layouts | min-length |
| `Color` | Solid color fill view | hex, rgba, named semantic color |
| `Canvas` | Immediate-mode 2D drawing context | draw closure, symbols |
| `Shape` | Vector shapes (rect, circle, path, rounded-rect) | fill, stroke, trim |

### 5.2 Interactive Controls

| Component | Description | Key Properties |
|---|---|---|
| `Button` | Tappable action trigger with role semantics | action, role (destructive/cancel), label |
| `Toggle` | Boolean on/off switch | binding, style (switch/checkbox/button) |
| `Slider` | Continuous value input | binding, range, step, label |
| `Stepper` | Discrete increment/decrement | binding, range, step |
| `TextField` | Single-line text input | binding, placeholder, keyboard-type, validator |
| `SecureField` | Password input | binding, placeholder |
| `TextEditor` | Multi-line text area | binding, axis, max-height |
| `Picker` | Selection from a list of options | binding, values, style (wheel/inline/menu) |
| `DatePicker` | Calendar date/time selection | binding, range, display-components |
| `ColorPicker` | RGBA color selection UI | binding, supports-opacity |

### 5.3 Container & Navigation Views

| Component | Description | Key Properties |
|---|---|---|
| `NavigationStack` | Push/pop navigation with path binding | path, root, destination-builder |
| `NavigationSplitView` | Sidebar + detail layout (macOS/iPad style) | sidebar, detail, column-visibility |
| `TabView` | Tab bar navigation | selection, tab-items |
| `Sheet` | Modal bottom-sheet or centered dialog | is-presented, detents, drag-indicator |
| `Alert` | System alert dialog | is-presented, title, message, actions |
| `ConfirmationDialog` | Action sheet style dialog | is-presented, title-visibility, actions |
| `Menu` | Contextual dropdown menu | label, content (MenuItems) |
| `List` | Scrollable list with sections and row actions | content, style, selection |
| `Table` | Multi-column tabular data view | rows, columns, selection, sort-order |
| `Form` | Settings-style grouped form layout | content, sections |

### 5.4 Visual & Decorative Views

| Component | Description | Key Properties |
|---|---|---|
| `ProgressView` | Linear or circular progress indicator | value, total, style (linear/circular) |
| `Gauge` | Radial or linear gauge display | value, range, label, current-value-label |
| `GroupBox` | Labeled content group | label, content |
| `DisclosureGroup` | Expandable/collapsible section | is-expanded, label, content |
| `Badge` | Notification count overlay | count, label |
| `Label` | Icon + text combination | title, system-image, image |
| `Link` | Tappable URL-navigating text | destination URL, label |
| `Tag` | Pill-shaped categorical label | text, color, removable |

### 5.5 Visual Inspiration from Reference Projects

#### From water-rs/waterui

- **Universal backend abstraction:** components must not assume a specific renderer at definition time. The waterui pattern of deferring rendering to an abstract backend is MANDATORY.
- **Platform-agnostic layout values:** spacing, font sizes, and corner radii must be expressed as semantic tokens, not raw pixel values.
- **Platform context injection:** platform-specific behavior (cursor, input method, clipboard) injected into components without changing component source code.

#### From OpenSwiftUIProject/OpenSwiftUI

- **Alignment guide system:** `HStack` and `VStack` accept explicit alignment guides customizable per-view via `alignmentGuide()` modifier.
- **Preference key protocol:** child-to-parent data flow for collecting geometry from leaf views.
- **`SizeReader`** (CVKG equivalent of `GeometryReader`): provides a view its own bounds without breaking the layout pass.
- **Environment overlay system:** child views can override environment values in their subtree.

---

## 6. Rendering Backends

### 6.1 Backend Trait Contract

All renderer backends implement the `CvkgRenderer` trait. The framework core communicates with backends exclusively through this interface.

```rust
pub trait CvkgRenderer: Send + Sync {
    fn begin_frame(&mut self, size: PhysicalSize, scale: f32);
    fn end_frame(&mut self) -> FrameResult;

    fn fill_rect(&mut self, rect: Rect, paint: &Paint);
    fn stroke_rect(&mut self, rect: Rect, stroke: &Stroke);
    fn fill_rounded_rect(&mut self, rect: Rect, radii: CornerRadii, paint: &Paint);
    fn fill_path(&mut self, path: &BezPath, paint: &Paint);
    fn draw_text(&mut self, layout: &TextLayout, origin: Point, paint: &Paint);
    fn draw_image(&mut self, image: &ImageHandle, dest: Rect, options: &ImageOptions);
    fn push_layer(&mut self, opacity: f32, clip: Option<&ClipShape>);
    fn pop_layer(&mut self);
    fn push_transform(&mut self, transform: Affine);
    fn pop_transform(&mut self);
}
```

> **Karpathy Guideline 1 enforcement:** Before implementing any backend, the agent MUST write a `//! # Design Note` doc comment at the top of the backend crate stating: which `CvkgRenderer` methods are trivially implemented, which require non-trivial platform work, and what assumptions are being made about the host environment. This is a required deliverable, not optional.

### 6.2 Native Backend (cvkg-render-native)

- **[CRITICAL]** winit for window creation and event loop on all desktop targets
- AccessKit integration for accessibility tree — MANDATORY for all native targets
- On macOS: CoreText for text shaping, CoreAnimation layers for compositing
- On Windows: DirectWrite for text, DXGI swap chain management
- On Linux: Pango for text shaping, Wayland compositor protocol integration

### 6.3 GPU Backend (cvkg-render-gpu) [CRITICAL]

The GPU backend uses `wgpu` — supporting Vulkan, Metal, DirectX 12, and WebGPU through a single API.

#### GPU Renderer Requirements

- Maintain a per-frame command buffer built from scene graph paint commands
- GPU atlas for glyph rasterization (runic-text + swash)
- Batch draw calls: group solid fills → gradient fills → textured quads → text
- Texture atlas for images with LRU eviction policy
- Shadow/blur pipeline using a two-pass Gaussian blur compute shader
- All shaders written in WGSL (WebGPU Shading Language) for cross-backend compatibility via naga transpilation

#### WebGPU vs WebGL Policy

CVKG targets WebGPU as the primary GPU API for web deployment. WebGL2 is maintained as an automatic fallback, detected at runtime.

| Capability | WebGPU | WebGL2 Fallback |
|---|---|---|
| Compute shaders | Yes (blur, effects) | No — CPU fallback |
| Multi-sampling (MSAA) | 4x native | EXT_multisample |
| Storage buffers | Yes | Simulated via textures |
| Pipeline cache | Yes | No |
| Async compilation | Yes | Limited |
| Max texture size | 8192+ px | 4096 px guaranteed |

### 6.4 WASM + Virtual DOM Backend (cvkg-render-web) [CRITICAL]

When compiled to WebAssembly, CVKG uses a hybrid approach: WebGPU/WebGL for pixel-perfect rendering, combined with a parallel virtual DOM tree maintained for developer tooling, accessibility, and troubleshooting.

#### 6.4.1 Virtual DOM Architecture

The vDOM in CVKG is NOT the rendering mechanism — it is a shadow representation of the component tree maintained in parallel to the GPU rendering path. It serves three purposes:

1. **Developer Inspector:** source of truth for the CVKG Inspector tool — inspect, highlight, and modify component properties at runtime without recompiling
2. **Accessibility Tree:** maps to ARIA roles and properties, injected into the browser's accessibility tree via hidden DOM elements with correct semantic markup
3. **Visual Debugging:** when the Inspector is active, overlays the vDOM tree atop the WebGPU canvas, highlighting component boundaries, layout boxes, and state annotations

```rust
// cvkg-vdom/src/lib.rs

pub struct VNode {
    pub id: NodeId,
    pub component_type: &'static str,
    pub props: PropMap,           // serialized view properties
    pub state: Option<StateMap>,  // debug: current state snapshot
    pub layout: LayoutRect,       // resolved layout bounds
    pub children: Vec<VNode>,
    pub aria_role: AriaRole,
    pub aria_props: AriaProps,
}

pub struct VDom {
    root: VNode,
    // Diff the previous frame's vdom against the new one
    pub fn diff(&self, prev: &VDom) -> Vec<VDomPatch> { ... }
    // Apply patches to the browser's accessibility DOM
    pub fn apply_to_dom(&self, patches: &[VDomPatch]) { ... }
    // Serialize vdom for Inspector WebSocket protocol
    pub fn serialize_for_inspector(&self) -> serde_json::Value { ... }
}
```

#### 6.4.2 vDOM Diff Algorithm

- Keyed diffing algorithm (`key` attribute on list items, analogous to React's `key` prop)
- Unkeyed diffing: positional matching with longest-common-subsequence for minimal patch sets
- Patch categories: `Create`, `Remove`, `Update` (props only), `Move` (reorder), `Replace`
- Patches applied to accessibility DOM are batched and applied in a single `requestAnimationFrame` callback

#### 6.4.3 Inspector Protocol

The CVKG Inspector communicates with the running WASM application over a WebSocket channel (JSON-based, bidirectional):

- **App → Inspector:** vDOM snapshot, state updates, layout metrics, frame timing
- **Inspector → App:** component highlight requests, property override injection, state mutation commands
- Inspector UI is a separate web application served by the cvkg CLI dev server

---

## 7. CLI Toolchain (cvkg-cli) [CRITICAL]

### 7.1 CLI Command Reference

| Command | Description | Key Flags |
|---|---|---|
| `cvkg new <name>` | Scaffold a new CVKG application workspace | `--template`, `--no-git` |
| `cvkg dev` | Start development server with hot reload | `--target`, `--port`, `--inspector` |
| `cvkg build` | Build for a specified target platform | `--target`, `--release`, `--features` |
| `cvkg serve` | Start the WebKit preview server (no rebuild) | `--port`, `--open`, `--inspector` |
| `cvkg check` | Run type-check + component lint + layout audit | `--all`, `--target` |
| `cvkg test` | Run unit and snapshot tests | `--ui`, `--target` |
| `cvkg inspect` | Launch the Inspector against a running dev server | `--url`, `--ws-port` |
| `cvkg export` | Export static WASM bundle for deployment | `--base-path`, `--optimize` |
| `cvkg add <crate>` | Add a CVKG-compatible component crate | `--features` |
| `cvkg theme generate` | Generate a custom theme from design tokens (JSON) | `--input`, `--output` |

### 7.2 Development Server

The development server is built on Axum and serves the compiled WASM bundle alongside a minimal host HTML shell.

```rust
// cvkg-webkit-server/src/main.rs (simplified)

async fn serve(config: ServeConfig) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(serve_shell))       // HTML host shell
        .route("/app.wasm", get(serve_wasm)) // WASM bundle
        .route("/assets/*path", get(serve_static))
        .route("/cvkg-ws", get(ws_handler))  // Inspector WebSocket
        .route("/hmr", get(hmr_ws_handler))  // Hot Module Reload
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(config.addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

#### WebKit Browser Integration

- **[CRITICAL]** wry integration for native WebKit shell — must support WebGPU via WKWebView
- The WebKit shell exposes a JavaScript bridge for Inspector communication: `window.cvkg_inspector`
- The shell injects the CVKG Inspector sidebar as a separate iframe when `--inspector` flag is active
- On Windows, Microsoft Edge WebView2 is used as the fallback

#### Hot Reload System

**Logic Hot Reload [CRITICAL]:** component Rust source changes trigger an incremental wasm-pack rebuild of only the changed crates (via cargo-chef build plan caching). The new WASM module is pushed to the browser via HMR WebSocket and swapped into the running application without full page reload. State is preserved via the serialized state snapshot stored in the vDOM.

**Asset Hot Reload:** changes to theme JSON, image assets, or token files are pushed directly without WASM rebuild.

### 7.3 Project Scaffold Structure

`cvkg new myapp` produces:

```
myapp/
├── Cargo.toml              # workspace manifest
├── cvkg.toml               # project config (targets, theme, entry)
├── src/
│   ├── main.rs             # native entry point
│   ├── lib.rs              # WASM entry point
│   └── app.rs              # root App view
├── components/
│   └── mod.rs              # application components
├── theme/
│   ├── tokens.json         # design token definitions
│   └── theme.rs            # generated theme (do not edit)
├── assets/
│   └── images/             # image assets
└── tests/
    ├── unit/               # unit tests
    └── snapshots/          # visual regression snapshots
```

---

## 8. Theme Engine & Design System (cvkg-themes)

### 8.1 Design Token Categories

| Category | Examples | Token Prefix |
|---|---|---|
| Colors (Semantic) | primary, secondary, accent, destructive, surface, background | `color.*` |
| Colors (Elevated) | surface.elevated, surface.grouped, surface.inset | `color.surface.*` |
| Typography | largeTitle, title1-3, headline, body, callout, subheadline, footnote, caption1-2 | `font.*` |
| Spacing | xs(4), sm(8), md(16), lg(24), xl(32), xxl(48) — logical pixels | `spacing.*` |
| Corner Radius | none, sm(4), md(8), lg(12), xl(16), full(999) | `radius.*` |
| Shadow | none, xs, sm, md, lg — parameterized blur/offset/color | `shadow.*` |
| Border | hairline(0.5), thin(1), medium(2), thick(4) | `border.*` |
| Animation | fast(150ms), default(250ms), slow(400ms), spring configs | `anim.*` |

### 8.2 Dark Mode & Adaptive Colors

All semantic color tokens automatically adapt to the system appearance. Components never reference raw hex colors — always semantic tokens.

```json
{
  "color": {
    "primary": {
      "light": "#1E5F9E",
      "dark":  "#5BA3DE"
    },
    "surface": {
      "light": "#FFFFFF",
      "dark":  "#1C1C1E"
    },
    "background": {
      "light": "#F2F2F7",
      "dark":  "#000000"
    }
  }
}
```

### 8.3 SwiftUI Visual Inspiration

- **Vibrancy and blur effects:** translucent surfaces with backdrop blur (compute shader on desktop; CSS `backdrop-filter` on web)
- **Viking Icons:** built-in SVG icon set using dot-separated naming convention (`arrow.right`, `plus.circle.fill`)
- **Continuous corner curves (superellipse):** default corner radius shape is a squircle (continuous corner), not a circular arc, matching macOS geometry
- **Physics-based animations:** spring animations are the default transition type, parameterized by stiffness, damping, and mass
- **Scroll bounce and rubber-banding** on all scrollable containers
- **Haptic feedback hooks** (maps to native APIs on supporting platforms, no-op on web)

---

## 9. Animation System (cvkg-anim)

### 9.1 Animation Primitives

```rust
// Explicit animation block
withAnimation(.spring(stiffness: 300.0, damping: 20.0)) {
    state.is_expanded = true;
}

// Implicit animation on a modifier
Rectangle()
    .frame(width: state.is_expanded ? 300.0 : 100.0)
    .animation(.easeInOut(duration: 0.3), value: state.is_expanded)

// Transition (appear/disappear)
if state.show_panel {
    SidePanel()
        .transition(.slide.combined(with: .opacity))
}
```

### 9.2 Animation Curve Types

- **Linear** — constant velocity
- **EaseIn / EaseOut / EaseInOut** — cubic bezier curves matching SwiftUI defaults
- **Spring** — damped harmonic oscillator with stiffness, damping, and mass parameters
- **Interpolating Spring** — velocity-preserving spring for gesture-driven animations
- **Custom Bezier** — arbitrary cubic bezier defined by two control points
- **Stepped** — discrete keyframe stepping

### 9.3 Transition System

Built-in transitions: `opacity`, `scale`, `slide` (from edge), `move` (translate to offset), `blur`, `push` (navigate-style), and `asymmetric` (different in/out). Transitions compose with `.combined(with:)` and can be delayed with `.delay()`.

---

## 10. Accessibility [CRITICAL]

Accessibility is a first-class requirement. Every built-in component SHALL expose correct accessibility semantics on every rendering target. Failure to pass accessibility validation is treated as a build-breaking defect.

### 10.1 Accessibility Architecture

- **[CRITICAL]** All components implement the `Accessible` trait providing `role`, `label`, `value`, `hint`, and `traits`
- Native targets use AccessKit to bridge to AT-SPI (Linux), UIA (Windows), and NSAccessibility (macOS)
- Web/WASM targets map the vDOM accessibility layer to ARIA attributes on hidden DOM elements
- Components accept `.accessibility_label()`, `.accessibility_hint()`, `.accessibility_value()`, and `.accessibility_hidden()` modifiers
- Keyboard navigation: Tab order follows view tree order; arrow keys navigate within compound controls
- Focus management: `.focused()` binding, programmatic `focus()`, and `FocusState` environment

### 10.2 Accessibility Modifiers

```rust
Button("Delete") { delete() }
    .accessibility_label("Delete document")
    .accessibility_hint("Permanently removes the document")
    .accessibility_role(.button)
    .accessibility_traits([.destructive])
```

---

## 11. Testing Strategy

> **Karpathy Guideline 4 enforcement:** Tests are not optional or deferred. Every implementation task produces its tests as part of the same commit. A subsystem without tests does not meet its phase completion criterion.

### 11.1 Test Layers

| Layer | Test Type | Tooling |
|---|---|---|
| State graph | Unit tests — state propagation, cycle detection | `cargo test` |
| Layout engine | Unit tests — constraint solving, size proposals | `cargo test` |
| Component API | Integration tests — modifier chains, composition | `cargo test` |
| Renderer output | Snapshot tests — pixel-level regression | `cvkg-test-renderer` + `insta` |
| Accessibility tree | Accessibility tests — ARIA role and label validation | axe-core (WASM), AccessKit validator |
| vDOM diff | Unit tests — diff algorithm correctness, patch application | `cargo test` |
| CLI | Integration tests — scaffold, build, serve commands | `cargo test` + tempdir |
| Visual regression | Screenshot comparison across renderer backends | `cvkg test --ui` |

### 11.2 Snapshot Testing

Every component in `cvkg-components` SHALL have a snapshot test rendering it to a headless framebuffer and comparing against a committed reference image. The `insta` crate is used for snapshot management.

### 11.3 Cross-Backend Consistency

A dedicated test suite — `cvkg-consistency-tests` — renders the same component tree on every backend (native GPU, WebGPU, WebGL) and validates that layout, color, and typography output are within acceptable pixel tolerance (default: 2px, 1% color delta).

### 11.4 Karpathy Goal-Driven Test Formulation

In accordance with Guideline 4, all test tasks MUST be expressed in verifiable form before implementation:

| Task | Verifiable Form |
|---|---|
| "Test the diff algorithm" | "Write tests covering: identical trees (no patches), added node, removed node, moved keyed node, updated props. All pass via `cargo test -p cvkg-vdom`" |
| "Test HStack layout" | "Write tests: zero children, one child, N children with fixed spacing, N children with Spacer. Assert child frame origins match expected values. `cargo test -p cvkg-layout`" |
| "Test hot reload" | "Integration test: modify a Text component source, run wasm-pack build, assert HMR WebSocket delivers the new module within 10s, assert DOM reflects the new text" |

---

## 12. Agentic AI Implementation Phases

The agentic AI system SHALL implement CVKG in the following phases, in strict order. Phase N+1 work SHALL NOT begin until Phase N's completion criterion is fully met and verified. This is a hard rule per Karpathy Guideline 4.

> For each phase, the agent SHALL produce a `PLAN.md` in the root of the primary crate being worked on, following the multi-step task planning format defined in Section 2.4, before writing any implementation code.

---

### Phase 0: Workspace & Scaffolding

**Verifiable completion criterion:** `cargo build --workspace` succeeds with zero errors and zero warnings.

- Initialize Cargo workspace with all 13 crates listed in Section 3.2
- Set up CI (GitHub Actions): `cargo check`, `cargo test`, `cargo clippy --deny warnings`; max job runtime 20 minutes (Guideline 7.4)
- Implement `cvkg-macros` crate with `#[view]`, `#[state]`, `#[binding]` procedural macros
- Add the full seven-guideline summary card (Section 2.8) to the `lib.rs` of every crate
- Create `AGENT_LOG.md` at workspace root for tool call tracking (Guideline 7.1)

---

### Phase 1: Core Framework [CRITICAL]

**Verifiable completion criterion:** A minimal app with `Text` + `Button` + state toggling compiles, runs, and all `cargo test -p cvkg-core` tests pass. `cargo doc --document-private-items -p cvkg-core` produces zero missing-doc warnings (Guideline 6).

- Before writing any code, perform triple-pass review of the View trait spec in Section 4.1 and all references to it across the specification (Guideline 5)
- Implement `View` trait, `Never` type, `ModifiedView` in `cvkg-core`
- Implement state graph: `State<T>`, `Binding<T>`, `Environment<K>` types
- Implement modifier chain protocol and built-in geometry modifiers (`frame`, `padding`, `offset`)
- Implement layout engine with `HStack`, `VStack`, `ZStack` in `cvkg-layout`
- All `pub` and `pub(crate)` functions MUST have `///` doc comments before the phase is marked complete (Guideline 6)

---

### Phase 2: GPU Renderer [CRITICAL]

**Verifiable completion criterion:** Hello-world app renders a colored rectangle in a native window using the wgpu backend. `cargo test -p cvkg-render-gpu` passes. All WGSL entry points and Rust backend methods have comments (Guideline 6).

- Write the required `//! # Design Note` doc comment before any implementation code (Guideline 1)
- Before modifying any file, perform triple-pass review: read the `CvkgRenderer` trait, all its call sites in the scene graph, and the full wgpu backend skeleton (Guideline 5)
- Implement `CvkgRenderer` trait in `cvkg-render-gpu` using wgpu
- Implement WGSL shaders for: solid fill, gradient fill, rounded rect, text glyph atlas, image blit — each entry point MUST have a leading block comment (Guideline 6)
- Integrate runic-text for text shaping and layout
- Monitor `wgpu` device initialization; if stalled >30s, check adapter enumeration and apply Guideline 7 corrective protocol

---

### Phase 3: Component Library

**Verifiable completion criterion:** All components from Sections 5.1–5.4 render correctly on the GPU backend. Each component has: a snapshot test, an accessibility test, and full API documentation with no missing-doc warnings (`cargo doc -p cvkg-components`). `cargo test -p cvkg-components` passes.

- Implement all Primitive Views (Section 5.1)
- Implement all Interactive Controls (Section 5.2)
- Implement all Container & Navigation Views (Section 5.3)
- Implement all Visual & Decorative Views (Section 5.4)

> **Guideline 3:** Each component is implemented as a discrete task. Implementing `Button` does not authorize changes to `Text`, the layout engine, or any other component.

> **Guideline 5:** Before modifying a component that calls into the layout engine or state graph (e.g., `List`, `NavigationStack`), perform all three context passes including tracing the call graph into `cvkg-layout` and `cvkg-core`.

> **Guideline 6:** Every component's `pub` methods and `ViewModifier` implementations MUST have `///` doc comments. Component-level doc comments MUST include a brief usage example in a `# Examples` section.

> **Guideline 7:** If the same component snapshot test fails three consecutive times with an identical pixel diff, stop patching. Reduce to a minimal render case and apply root-cause analysis before resuming.

---

### Phase 4: WASM + vDOM Backend [CRITICAL]

**Verifiable completion criterion:** Hello-world runs in a browser via WASM with WebGPU rendering. CVKG Inspector connects over WebSocket and displays the vDOM tree. `cargo test -p cvkg-vdom` passes. All public Inspector protocol functions have TypeScript JSDoc comments (Guideline 6).

- Before implementing the diff algorithm, perform triple-pass review: read the `VDom` struct spec in Section 6.4, the Inspector protocol spec in Section 6.4.3, and trace how `VDomPatch` types map to accessibility DOM mutations (Guideline 5)
- Set up wasm-bindgen and wasm-pack build pipeline in `cvkg-render-web`
- Implement WebGPU canvas rendering path
- Implement WebGL2 fallback rendering path with automatic runtime detection
- Implement `VDom`, `VNode`, `VDomPatch` types in `cvkg-vdom`; all structs MUST have `///` doc comments (Guideline 6)
- Implement vDOM diff algorithm and accessibility DOM injection
- Implement Inspector WebSocket protocol; all TypeScript handler functions MUST have JSDoc comments (Guideline 6)
- Monitor `wasm-pack build` — if the same build step stalls for >30 seconds across two consecutive runs, apply Guideline 7 stall protocol (clear `pkg/`, retry with `--dev`)

---

### Phase 5: CLI & Dev Server [CRITICAL]

**Verifiable completion criterion:** `cvkg dev` launches, opens a WebKit window showing the running app, and hot-reloads on source change in under 10 seconds without page reload. All CLI command handler functions and Axum route handlers have `///` doc comments (Guideline 6).

- Before implementing the HMR system, perform triple-pass review: read the HMR spec in Section 7.2, the vDOM state serialization spec in Section 6.4.1, and trace the full path from source file change → `cargo-chef` rebuild → WebSocket push → WASM swap (Guideline 5)
- Implement `cvkg` CLI binary using clap in `cvkg-cli`; each subcommand handler MUST have a doc comment describing its purpose, flags, and expected side effects (Guideline 6)
- Implement Axum-based dev server in `cvkg-webkit-server`
- Implement wry WebKit shell for native preview
- Implement HMR WebSocket channel and incremental WASM rebuild
- Implement project scaffold (`cvkg new`)
- Monitor the HMR rebuild loop: if source change → rebuild → browser update takes more than 15 seconds on the second consecutive cycle, investigate cargo-chef cache validity and apply Guideline 7 corrective action

---

### Phase 6: Theme Engine & Polish

**Verifiable completion criterion:** Theme switching and spring animations work correctly across all render backends. `cargo test -p cvkg-themes` and `cargo test -p cvkg-anim` pass.

- Implement design token system and `cvkg-themes` crate
- Implement dark/light mode adaptive color system
- Implement spring physics animation system in `cvkg-anim`
- Implement transition system and `withAnimation()`

---

### Phase 7: Native Backend & Accessibility

**Verifiable completion criterion:** VoiceOver (macOS), Narrator (Windows), and Orca (Linux) correctly read all standard components. `cargo test -p cvkg-render-native` passes.

- Implement `cvkg-render-native` with winit + AccessKit
- Platform-specific text shaping integration (CoreText, DirectWrite, Pango)
- Full accessibility tree validation on all three desktop platforms

---

### Phase 8: Testing, Docs & Release

**Verifiable completion criterion:** All tests pass, zero clippy warnings, mdBook documentation builds, crates publish to crates.io at semver `0.1.0`.

- Complete cross-backend consistency test suite (`cvkg-consistency-tests`)
- Visual regression baseline snapshots for all components
- mdBook documentation site generated from source
- Publish all crates to crates.io

---

## 13. Key Dependencies & Rationale

| Crate | Version Policy | Purpose & Rationale |
|---|---|---|
| `wgpu` | latest stable | WebGPU implementation for desktop GPU + WASM WebGPU rendering |
| `winit` | latest stable | Cross-platform window + event loop |
| `wasm-bindgen` | latest stable | Rust ↔ JavaScript interop for WASM targets |
| `web-sys` | latest stable | Browser Web APIs bindings (WebGPU, WebGL, DOM, WebSocket) |
| `wasm-pack` | latest stable | WASM build toolchain, bundler integration |
| `axum` | latest stable | Async HTTP server for dev server and static file serving |
| `wry` | latest stable | WebKit/WebView2 native shell for `cvkg serve` preview |
| `taffy` | latest stable | Flexbox layout engine, constraint solving backend |
| `runic-text` | latest stable | Cross-platform text shaping, layout, and glyph rendering |
| `swash` | latest stable | Font loading, glyph rasterization into wgpu atlas |
| `accesskit` | latest stable | Cross-platform accessibility tree bridge |
| `serde` / `serde_json` | latest stable | Serialization for Inspector protocol, state persistence, tokens |
| `tokio` | latest stable | Async runtime for dev server and HMR WebSocket |
| `clap` | latest stable | CLI argument parsing for `cvkg` binary |
| `naga` | latest stable | WGSL shader transpilation (bundled with wgpu) |
| `insta` | latest stable | Snapshot testing framework |
| `syn` / `quote` / `proc-macro2` | latest stable | Procedural macro infrastructure |
| `cargo-chef` | latest stable | Incremental WASM rebuild caching for HMR |

---

## 14. Glossary

| Term | Definition |
|---|---|
| CVKG | Cyber Viking GUI X — the framework defined in this document |
| View | A declarative description of a piece of UI; the fundamental CVKG unit |
| Modifier | A transformation applied to a View to change its appearance or behavior |
| Scene Graph | A retained hierarchy of rendered nodes used for efficient differential updates |
| vDOM | Virtual DOM — a shadow tree of CVKG component nodes used for debugging and accessibility |
| wgpu | A Rust implementation of the WebGPU API, supporting Vulkan/Metal/DX12/WebGPU |
| WASM | WebAssembly — binary instruction format for a stack-based VM; target for web deployment |
| HMR | Hot Module Reload — update WASM modules in a running browser without full page reload |
| Inspector | The CVKG developer tool for visualizing and interacting with a running app's vDOM and state |
| Design Token | A named semantic value (color, spacing, radius) abstracting platform-specific raw values |
| AccessKit | A cross-platform Rust library for exposing accessibility trees to assistive technologies |
| wry | A Rust cross-platform WebView library wrapping WebKit (macOS/Linux) and WebView2 (Windows) |
| Karpathy Guidelines | The four agentic coding behavioral guidelines defined in Section 2, mandatory for all AI contributors |
| `CVKG-CLARIFY:` | Comment tag used by agents to flag ambiguous specification points requiring human review |
| `CVKG-TODO:` | Comment tag used by agents to flag observed improvements outside current task scope |
| `CVKG-DEAD:` | Comment tag used by agents to flag observed dead code outside current task scope |
| `CVKG-CONTEXT:` | Comment tag placed at the top of an edit site documenting the call graph reviewed during Guideline 5 triple-pass |
| `CVKG-BLOCKED:` | Comment tag and accompanying `BLOCKED.md` file written when an agent cannot resolve a loop after three corrective attempts |
| Triple-Pass Review | Guideline 5 mandatory pre-edit protocol: read target site, surrounding context, and full call graph — three separate passes — before making any change |
| `AGENT_LOG.md` | Per-workspace log file tracking all tool calls, outcomes, and corrective actions; maintained by the agent per Guideline 7 |
| `PROGRESS.md` | Per-task working journal maintained by the agent for tasks >15 minutes; deleted upon completion |
| `BLOCKED.md` | Escalation document written when an agent cannot break a circular loop; signals need for human intervention |
| Circular Loop | Three or more consecutive actions producing identical or equivalent states without net forward progress; detected and escalated per Guideline 7 |

---

*CVKG Design Specification v1.2 — April 2026*
*Karpathy Guidelines (1–4): https://github.com/multica-ai/andrej-karpathy-skills*
*CVKG Extended Protocols (5–7): Section 2.5–2.7 of this document*
*Reference Projects: https://github.com/water-rs/waterui · https://github.com/OpenSwiftUIProject/OpenSwiftUI*

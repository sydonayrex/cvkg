# Contributing to CVKG

**Cyber Viking Kvasir Graph** — High-fidelity agentic UI framework

Thank you for contributing to CVKG! This guide covers everything you need to get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Accessibility Requirements](#accessibility-requirements)
- [Development Guidelines](#development-guidelines)

---

## Getting Started

### Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **Git** for version control
- For GPU rendering work: a Vulkan-compatible GPU and drivers

### Setup

```bash
# Clone the repository
git clone https://github.com/sydonayrex/cvkg.git
cd cvkg

# Build the entire workspace
cargo build

# Run all tests
cargo test --workspace

# Run a specific crate's tests
cargo test -p cvkg-components
```

### Running Demos

```bash
# Run the Berserker demo (native)
cargo run -p berserker

# Run a WASM demo (requires wasm-pack or wasm-bindgen-cli)
cargo run -p niflheim-wasi
```

---

## Project Structure

CVKG is a Cargo workspace with 26+ crates. Key crates:

| Crate | Purpose |
|---|---|
| `cvkg-core` | Core types: `View` trait, `AriaRole`, knowledge state, window management |
| `cvkg-components` | UI component library (buttons, dialogs, tabs, datepickers, etc.) |
| `cvkg-vdom` | Virtual DOM / AccessKit bridge |
| `cvkg-scene` | Scene graph and paint commands |
| `cvkg-layout` | Layout engine (constraint-based) |
| `cvkg-anim` | Animation system and transitions |
| `cvkg-render-gpu` | GPU renderer (wgpu-based, WGSL shaders) |
| `cvkg-render-native` | Native platform renderer |
| `cvkg-render-software` | Software rasterizer fallback |
| `cvkg-themes` | Theming and adaptive color tokens |
| `cvkg-compositor` | Compositing layer |
| `cvkg-flow` | Agentic workflow orchestration |
| `cvkg-cli` | Command-line interface |
| `cvkg-physics` | Physics simulation for UI elements |
| `cvkg-test` | Shared test utilities |

### Demos

| Demo | Description |
|---|---|
| `demos/berserker` | Native desktop demo |
| `demos/berserker-fire-web` | Web-based fire effects demo |
| `demos/adele-web` | Web demo |
| `demos/niflheim-wasi` | WASI-based demo |

---

## Code Style

### General Rules

1. **Rust edition 2024** — all code must compile under Rust 2024 edition.
2. Follow standard `rustfmt` formatting. Run `cargo fmt` before committing.
3. Run `cargo clippy --workspace -- -D warnings` and fix all warnings.
4. Use `rustfmt` defaults unless a crate-local `rustfmt.toml` overrides them.

### Naming Conventions

- **Types**: `PascalCase` — e.g. `HlinAccessibility`, `KnowledgeState`
- **Functions/methods**: `snake_case` — e.g. `focus_next()`, `detect_reduced_motion()`
- **Constants**: `SCREAMING_SNAKE_CASE` — e.g. `FOCUS_RING_COLOR` (prefer `theme::focus_ring()` instead)
- **Modules**: `snake_case` — e.g. `hlin_accessibility.rs`, `consent_gate.rs`
- **Crate names**: `kebab-case` (e.g. `cvkg-components`) with `snake_case` lib names (e.g. `cvkg_components`)

### Documentation

- **Every `pub fn`**, `unsafe` block, and non-trivial algorithm must have a doc comment (`///`).
- Doc comments describe **WHY** and the **CONTRACT**, not mechanical **HOW**.
- Use `//!` for module-level documentation.

```rust
/// Advance focus to the next element in focus order.
/// Wraps around to the first element when at the end.
/// Caller must hold a mutable reference to the accessibility state.
pub fn focus_next(&mut self) { ... }
```

### Imports

- Group imports in this order: `std`/`core`/`alloc`, then external crates, then `crate`/`self`.
- Prefer `use crate::` over relative paths.

### Error Handling

- Use `anyhow` for application-level errors.
- Use `thiserror` for library error types that callers need to match on.
- Never use `process::exit()` in library code.

---

## Testing Requirements

### Test Organization

Tests live in `<crate>/tests/` for integration tests and inline `#[cfg(test)]` modules for unit tests. The `cvkg-components` crate includes:

- `tests/accessibility_tests.rs` — accessibility compliance
- `tests/component_tests.rs` — component behavior
- `tests/snapshot_tests.rs` — visual snapshot tests (via `insta`)
- `tests/memory_leak_prevention.rs` — memory safety
- `tests/theming_test.rs` — theme integration

### What to Test

1. **New components**: must include unit tests and at least one integration test.
2. **Bug fixes**: must include a regression test that fails before the fix and passes after.
3. **Accessibility changes**: must include tests in `accessibility_tests.rs`.
4. **Theme changes**: must include tests in `theming_test.rs`.

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p cvkg-components

# Snapshot tests (review changes)
cargo insta review

# Run only accessibility tests
cargo test -p cvkg-components --test accessibility_tests

# Clippy (must pass with zero warnings)
cargo clippy --workspace -- -D warnings
```

### Snapshot Testing

We use [insta](https://insta.rs/) for snapshot tests. When a snapshot intentionally changes:

```bash
cargo insta review     # interactively accept/reject
cargo insta accept     # accept all pending
```

Commit the updated `.snap` files alongside your code changes.

### CI Requirements

All tests must pass in CI before a PR can be merged. The CI pipeline runs:

1. `cargo fmt --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
4. Snapshot test review

---

## Pull Request Process

### Before You Start

1. Check existing [issues](https://github.com/sydonayrex/cvkg/issues) and open one if none exists for your change.
2. For large changes, discuss the approach in the issue first.
3. Fork the repo and create a feature branch from `main`:

```bash
git checkout -b feature/my-feature
```

### Making Changes

1. **Think first** — state assumptions, surface ambiguity, push back on complexity.
2. **Stay simple** — minimum code, no speculative features, no unasked-for abstractions.
3. **Be surgical** — touch only what's required. Don't improve neighbors.
4. **Verify goals** — turn tasks into checkable criteria. Never commit broken code.

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add loading spinner to Button component

- Add animated spinner state to Button
- Add `loading` parameter to Button::new()
- Includes accessibility role mapping for busy state

Closes #42
```

Prefix format: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`

### PR Checklist

Before submitting your PR, verify:

- [ ] `cargo fmt` has been run
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] New/changed code has doc comments
- [ ] New components have unit tests
- [ ] Accessibility requirements are met (see below)
- [ ] No `unwrap()` in library code (use `?` or proper error handling)
- [ ] No `process::exit()` in library code
- [ ] Snapshot files updated if visual output changed

### Review Process

1. At least one maintainer review required.
2. All CI checks must pass.
3. Address review feedback — push additional commits (don't force-push during review).
4. Squash merge into `main` after approval.

---

## Accessibility Requirements

CVKG treats accessibility as a first-class feature, not an afterthought. **All UI components must meet WCAG 2.2 AA compliance.**

### Mandatory Requirements

#### 1. Minimum Touch Targets — 44×44px (WCAG 2.5.8)

All interactive elements (buttons, inputs, toggles, checkboxes, etc.) must have a minimum touch target of **44×44 pixels**. This is enforced project-wide.

```rust
// Good — meets minimum touch target
let button = Button::new("Submit")
    .min_size(Size { width: 44.0, height: 44.0 });

// Bad — too small for touch
let button = Button::new("X").size(24.0, 24.0);
```

#### 2. ARIA Roles

All interactive and semantic elements must use `cvkg_core::AriaRole` for their accessibility role. The unified `AriaRole` enum (53 variants) is the single source of truth — do **not** create parallel role enums.

```rust
use cvkg_core::AriaRole;

// Every interactive element gets a role
let node = HlinNode {
    role: AriaRole::Button,
    label: "Submit Form".to_string(),
    ..
};
```

#### 3. Keyboard Navigation

- All interactive components must be keyboard-navigable.
- Support **Tab** / **Shift+Tab** for focus traversal.
- Support **Enter** and **Space** for activation.
- Modal components must implement **focus trapping** (`FocusTrap`).
- Provide visible focus indicators (theme-aware via `theme::focus_ring()`).

#### 4. Screen Reader Support

- All visible text must have an accessible name (`label`).
- Use `AnnouncementPriority::Polite` for non-urgent updates.
- Use `AnnouncementPriority::Assertive` for urgent/alert messages.
- Decorative images use `AriaRole::None` (or `aria-hidden`).

#### 5. Reduced Motion

Respect the user's system preference for reduced motion. Use the platform-aware detection:

```rust
let accessibility = HlinAccessibility::new();
if accessibility.is_reduced_motion() {
    // Disable or simplify animations
}
```

The `is_reduced_motion()` check respects:
- GNOME/GTK `enable-animations` setting
- macOS `MACOS_REDUCED_MOTION` environment variable
- Windows `ACCESSIBILITY_REDUCED_MOTION` environment variable
- Generic `NO_ANIMATIONS` flag

#### 6. Color and Contrast

- Use adaptive color tokens from `cvkg_themes` (e.g. `theme::text()`, `theme::bg()`).
- Do **not** hardcode color constants — use theme-aware functions.
- Ensure 4.5:1 contrast ratio for normal text, 3:1 for large text (WCAG 1.4.3).
- Support high-contrast mode via `HlinAccessibility::high_contrast()`.

#### 7. Focus Ring

Use `theme::focus_ring()` for focus indicators — never hardcode `FOCUS_RING_COLOR`.

### Accessibility Testing

All accessibility-related changes must be tested in `tests/accessibility_tests.rs`. Test cases should verify:

- Correct ARIA role is assigned
- Focus order is logical and complete
- Touch targets meet 44px minimum
- Reduced motion is respected
- Screen reader announcements are sent at correct priority

---

## Development Guidelines

All contributors — human and AI agents — must follow the [CVKG Agentic Development Guidelines](cvkg-core/src/lib.rs):

### Karpathy Guidelines (1–4)

1. **Think First** — State assumptions. Surface ambiguity. Push back on complexity.
2. **Stay Simple** — Minimum code. No speculative features. No unasked-for abstractions.
3. **Be Surgical** — Touch only what's required. Own your orphans. Don't improve neighbors.
4. **Verify Goals** — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.

### CVKG Extended Protocols (5–7)

5. **Triple-Pass** — Read the target, its surrounding context, and its full call graph at least three times before making any edit.
6. **Comment All** — Every major `pub fn`, `unsafe` block, and non-trivial algorithm must have a descriptive doc comment explaining WHY and the CONTRACT.
7. **Monitor Loops** — Check progress every 30 seconds. After 3 consecutive identical failures, stop and move to unblocked work.

---

## Questions?

Open a discussion at [GitHub Discussions](https://github.com/sydonayrex/cvkg/discussions) or file an issue.

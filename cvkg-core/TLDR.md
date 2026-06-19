# cvkg-core TLDR.md

## Purpose
Own the core View system, Renderer trait, layout engine, focus management, and all foundational types for the CVKG UI framework.

## Ownership
- `src/lib.rs` — View trait, Renderer trait, layout, focus, accessibility, all core types
- `src/error_types.rs` — Error handling types
- Do NOT modify component implementations — those live in cvkg-components

## Local Contracts
- The View trait is the central contract. Any change to View::render, View::body, or View::intrinsic_size affects ALL components.
- Renderer trait methods must remain object-safe (no generics on methods).
- ErrorBoundary must use `catch_unwind` + `AssertUnwindSafe` — never suppress panics silently.
- All public types must have doc comments.
- KvasirId is the platform-wide unified identity type. All crates (cvkg-scene, cvkg-vdom, cvkg-flow, etc.) must use KvasirId or a type alias of it — never define a competing NodeId struct.

## Work Guidance
- When adding new Renderer methods, provide a default no-op implementation so existing backends don't break.
- Keep the View trait minimal — prefer ViewModifiers over trait bloat.
- Test changes with `cargo test -p cvkg-core` before committing.

## Verification
- Run `cargo check -p cvkg-core` and `cargo test -p cvkg-core`
- Run `cargo check --workspace` to verify no downstream breakage

# cvkg-flow AGENTS.md

## Purpose
Own the flow engine: data flow management, reactive state propagation, and the connection system between CVKG components and external data sources.

## Ownership
- `src/lib.rs` — Flow engine core, reactive primitives
- Data binding between CVKG state and external sources

## Local Contracts
- Flow propagation must be deterministic and cycle-safe.
- State updates must follow the STM (Software Transactional Memory) model where applicable.
- Must integrate with cvkg-core's state management (use_state, etc.).
- NodeId (in types.rs) is a type alias for cvkg_core::KvasirId — never define a separate identity struct.

## Verification
- Run `cargo test -p cvkg-flow`
- Run `cargo check --workspace`

# demos/niflheim-wasi

Headless server-side WASI target for checking view validation.

## Purpose

Demonstrates CVKG running in a headless WASI environment. Used to validate that view composition, layout, and state management work without a GPU or display.

## Boundaries

- No rendering output -- validates correctness, not visuals.
- Requires WASI target: `rustup target add wasm32-wasi` (if available).

## Usage

```bash
cd demos/niflheim-wasi
cargo check --target wasm32-unknown-unknown
```

## Dependencies

- `cvkg-core` -- View trait
- `cvkg-components` -- Widget library (validates that components compile without GPU)

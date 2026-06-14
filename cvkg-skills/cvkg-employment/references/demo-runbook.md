# CVKG Demo Runbook

Native Berserker app:

```bash
cargo run -p berserker
```

Berserker fire GPU example:

```bash
cargo run --example berserker_fire_demo -p cvkg --features gpu
```

Other GPU examples from the `cvkg` facade:

```bash
cargo run --example shatter_demo -p cvkg --features gpu
cargo run --example hit_test_demo -p cvkg --features gpu
```

Linux recovery when the renderer reports `no adapter found`:

```bash
export WGPU_ADAPTER=mesa
cargo run -p berserker
```

Web demo checks:

```bash
cargo check -p adele-web-demo
cargo check -p berserker-fire-web-demo
cargo build --target wasm32-unknown-unknown --features web --release
```

Definition of done for a demo request:

- The command matches the requested demo.
- The target platform is correct: native crate for `berserker`, GPU feature for `cvkg` examples, WASM target for browser demos.
- The demo runs or the failure is diagnosed from actual output.
- The user gets the exact command, not just a conceptual explanation.

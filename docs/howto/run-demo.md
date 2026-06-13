# How to Run a Demo

Goal: Execute a CVKG demo application to see the framework in action.

## Process Overview

```mermaid
graph TD
    A["Install Rust & GPU Deps"] --> B["Clone CVKG Repository"]
    B --> C["Build Workspace (cargo build)"]
    C --> D{"Choose Target Environment"}
    D -->|"Native GPU Desktop"| E["Run cargo run --example shatter_demo"]
    D -->|"Web Canvas"| F["Run wasm-pack / demo server"]
    D -->|"Headless Testing"| G["Run cargo test -p cvkg-test"]
```

## Prerequisites


- GPU with Vulkan, Metal, or DirectX 12 support
- Rust toolchain installed

## Steps

### 1. Build the workspace

```bash
cd /path/to/cvkg
cargo build --workspace
```

### 2. Run a GPU demo

```bash
# Shatter demo with visual effects
cargo run --example shatter_demo -p cvkg --features gpu

# Hit test demo showing pointer interactions
cargo run --example hit_test_demo -p cvkg --features gpu

# Berserker fire demo with particle effects
cargo run --example berserker_fire_demo -p cvkg --features gpu
```

### 3. Run a web demo

```bash
cargo run --example niflheim_demo -p cvkg-components --features web
```

### 4. Running Headless Verification & Screenshots

You can execute headless render tests to verify graphics pipelines or update the primary showcase screenshot (`docs/images/cvkg_hero.png`):

```bash
cargo test -p cvkg-test --test visual_regression -- --nocapture
```

This runs the Surtr renderer headlessly, maps the frame buffers, and writes the output directly.

## Expected Output

A native window opens with the demo content. Running the headless regression test will regenerate `docs/images/cvkg_hero.png` from the latest `niflheim_demo()` layouts.

## Recovery

If the application crashes with "no adapter found":

```bash
# Try software rendering (Linux only)
export WGPU_ADAPTER=mesa
cargo run --example shatter_demo -p cvkg --features gpu
```
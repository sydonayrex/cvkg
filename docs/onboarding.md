# Onboarding Guide

This guide walks through setting up a development environment for CVKG.

## Prerequisites

- **Rust**: Install Rust 1.81.0 or later
- **GPU**: Vulkan, Metal, or DirectX 12 support for native rendering
- **Git**: For cloning the repository

## Step-by-Step Setup

### 1. Clone the Repository

```bash
git clone https://github.com/sydonayrex/cvkg.git
cd cvkg
```

### 2. Install Rust Toolchain

```bash
rustup update stable
rustup target add wasm32-unknown-unknown
```

### 3. Install System Dependencies

On Ubuntu/Debian:
```bash
sudo apt-get install -y libvulkan-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

On macOS:
```bash
brew install vulkan-sdk
```

### 4. Build the Workspace

```bash
cargo build --workspace
```

### 5. Run the Full Test Suite

```bash
cargo test --workspace
```

### 6. Run a Single Crate's Tests

```bash
cargo test -p cvkg-core
```

### 7. Run a Single Test by Name

```bash
cargo test -p cvkg-core test_view_trait
```

## Where to Find Things

| Purpose | Location |
|---------|----------|
| Source code | `cvkg-core/src/`, `cvkg-components/src/`, etc. |
| Tests | `cvkg-*/tests/` or `cvkg-*/src/` with `#[cfg(test)]` |
| Config | `cvkg/Cargo.toml`, `cvkg-*/Cargo.toml` |
| Examples | `cvkg-*/examples/` |

## Making Changes

1. Create a branch
2. Make your changes
3. Run `cargo test --workspace`
4. Submit a pull request

## Getting Help

Maintainer: sydonayrex (https://github.com/sydonayrex)
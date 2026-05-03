# CVKG - Cyber Viking Kvasir Graph

**High-fidelity agentic UI framework written in Rust**

## Overview

CVKG is a cutting-edge UI framework that combines the power of high-performance rendering with agentic system design principles. It provides a declarative, reactive UI system with support for multiple rendering backends (native, web, GPU-accelerated).

## Features

- **Multi-backend rendering**: Native (winit), Web (wasm-bindgen), GPU-accelerated (wgpu)
- **Reactive state management**: STM-based transactions with ArcSwap for lock-free reads
- **High-fidelity visuals**: Cyberpunk aesthetics with neon effects, particle systems, and custom shaders
- **Component-based architecture**: Reusable, composable UI components
- **Accessibility-first**: WCAG AA compliance built-in
- **Animation system**: High-performance animations with Sleipnir solver
- **Scene graph**: Efficient rendering with dirty tracking and culling
- **Security**: Path validation and CORS handling

## Quick Start

```rust
use cvkg::prelude::*;

fn app() -> impl View {
    VStack::new()
        .child(Button::new(
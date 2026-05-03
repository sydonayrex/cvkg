# Getting Started Guide for Developers

## Architecture Overview

CVKG follows a layered architecture with clear separation of concerns:

### Core Layers
1. **View Layer**: Declarative UI definition using the View trait
2. **VDOM Layer**: Virtual DOM for efficient updates
3. **Renderer Layer**: GPU/Native/Web rendering backends
4. **Component Layer**: Reusable UI components

## Project Structure

```
cvkg/
├── cvkg-core/          # Core traits and types
├── cvkg-vdom/          # Virtual DOM implementation
├── cvkg-components/    # Reusable component library
├── cvkg-render-gpu/    # GPU renderer (wgpu)
├── cvkg-render-native/ # Native window integration
├── cvkg-render-web/    # Web/WASM renderer
└── cvkg-themes/        # Cyber Viking themes
```

## Setting Up a New Project

```toml
# Cargo.toml
[dependencies]
cvkg = { version = 
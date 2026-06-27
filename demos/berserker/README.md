# Berserker

Native tactical HUD demo application built on CVKG.

## Purpose

Demonstrates the CVKG framework's capabilities in a native desktop window. Showcases layout, rendering, animation, physics, and component library usage.

## Boundaries

This is a demo binary. It is NOT a framework crate -- it consumes CVKG APIs but does not provide reusable library code.

## Dependency Graph

```mermaid
graph TD
    berserker["berserker<br/>(Native HUD demo)"]
    berserker --> cvkg
    berserker --> cvkg-core
    berserker --> cvkg-vdom
    berserker --> cvkg-physics
    berserker --> cvkg-anim
    berserker --> cvkg-components
    berserker --> cvkg-themes

    classDef demo fill:#4a1d96,stroke:#a855f7,color:#c084fc,stroke-width:1.5px
    classDef entry fill:#064e3b,stroke:#10b981,color:#a7f3d0,stroke-width:2px
    classDef core fill:#1a1a2e,stroke:#1e293b,color:#e2e8f0,stroke-width:1px
    classDef ui fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    classDef services fill:#14532d,stroke:#22c55e,color:#86efac,stroke-width:1px
    class berserker demo
    class cvkg entry
    class cvkg-core cvkg-vdom core
    class cvkg-physics services
    class cvkg-anim cvkg-components cvkg-themes ui
```

## Usage

```bash
cargo run -p berserker
```

## Prerequisites

- Native GPU drivers (Vulkan/Metal/DX12)
- System dependencies: `libfontconfig1-dev`, `pkg-config`, `libx11-dev`, `libwayland-dev`

## What It Demonstrates

- Declarative view composition
- Spring-physics animations
- Rigid body physics simulation
- GPU rendering pipeline
- Component library usage (buttons, sliders, HUD elements)
- Theme application

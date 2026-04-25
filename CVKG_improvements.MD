# CVKG Framework: Optimization & Feature Roadmap

This document outlines the systematic improvements required for the CVKG (Cyberpunk Viking Knowledge Graph) framework to achieve full feature parity and production-grade stability for the Ulfhednar native application.

## 1. Core Rendering & Typography
*   [ ] **Full Font Rasterization**: Finalize `cosmic-text` integration in the `SurtrRenderer` to support high-fidelity font rendering with proper kerning and anti-aliasing.
*   [ ] **Rich Text Support**: Implement multi-style text (bold, italic, colored spans) within a single `Text` component.
*   [ ] **Iconography Pipeline**: Integrate a GPU-accelerated SVG or icon-font loader to support the terminal's tactical iconography.
*   [ ] **Texture Caching**: Implement an LRU cache for glyphs and images to optimize GPU memory and draw call batching.

## 2. Advanced Layout System
*   [ ] **Flexbox/Grid Parity**: Move beyond simple division in `HStack`/`VStack` to a robust constraint-based or flex-based layout engine (integrating `cvkg-layout`).
*   [ ] **Padding & Margin Consistency**: Standardize the `PaddingModifier` across all composite views.
*   [ ] **Z-Index Management**: Implement a proper depth-sorting pass in the `SurtrRenderer` to handle overlapping modals and tooltips correctly.
*   [ ] **Intrinsic Sizing**: Enable views to calculate their own natural dimensions based on content (e.g., a Button sizing itself to its Label).

## 3. Interactive Component Library
*   [ ] **Functional Text Fields**: Implement the `TextField` and `SecureField` components with full keyboard focus, cursor management, and selection support.
*   [ ] **Pickers & Dropdowns**: Finalize the `Picker` component for model selection and provider switching.
*   [ ] **Scrollable Containers**: Implement the `List` and `Scrollable` components with GPU-accelerated clipping and inertial scrolling.
*   [ ] **Progress & Status Indicators**: Add `ProgressRing` and `StatusBar` components for mission tracking and resource monitoring.

## 4. Muspelheim Aesthetic Suite (Shaders)
*   [ ] **Gungnir (Neon Glow)**: Finalize the additive multi-pass bloom shader to create the signature "Berserker" glow effect on buttons and borders.
*   [ ] **Bifrost (Frosted Glass)**: Implement the backdrop blur shader for high-fidelity modal backgrounds.
*   [ ] **Mjolnir (Geometric Slicing)**: Enable the SDF-based clipping modifiers for sharp, angled terminal aesthetics.
*   [ ] **Dynamic Animations**: Fully integrate the `Sleipnir` RK4 physics solver for smooth state transitions and reactive UI feedback.

## 5. Input & Event Handling
*   [ ] **Event Propagation**: Build a robust hit-testing engine to propagate mouse clicks and hover states through the view hierarchy.
*   [ ] **Focus Management**: Implement a global focus controller for keyboard-driven navigation.
*   [ ] **Gesture Support**: Basic support for scroll-wheel and drag-and-drop operations for asset management.

## 6. State & Lifecycle Management
*   [ ] **Reactive Bindings**: Optimize the `Binding` system to minimize redraws by tracking specific property changes rather than whole-state refreshes.
*   [ ] **Async Lifecycle Hooks**: Standardize `on_appear` and `on_disappear` hooks for views to trigger background data fetching.

## 7. Distribution & Infrastructure
*   [ ] **Native Asset Embedding**: Implement `include_bytes!` integration for shaders, fonts, and icons to ensure single-binary portability.
*   [ ] **Platform Stabilization**: Verify windowing and input parity across Linux, macOS, and Windows targets.
*   [ ] **Performance Profiling**: Integrate the `cvkg-telemetry` hooks into the renderer to monitor frame times and GPU utilization in real-time.

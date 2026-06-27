# CVKG Core Module Split Map

**Audit recommendation #2:** The original 9,014-line `cvkg-core/src/lib.rs` monolith has been
split into **57 modules** totalling **312 lines** (97% reduction). This document records the
final structure.

## Current module layout

Each module is listed with its role and the public items it re-exports.

| # | Module file | Re-exports (key items) |
|---|-------------|------------------------|
| 1 | `view.rs` | `View` trait (all methods, `changed()`, `needs_update()`, `body()`, modifiers, transitions) |
| 2 | `aria.rs` | `AriaRole` enum (all variants), ARIA support |
| 3 | `keyboard.rs` | Keyboard event types, shortcut handling |
| 4 | `focus.rs` | Focus management types |
| 5 | `reduced_motion.rs` | Reduced motion detection (OS-level accessibility) |
| 6 | `erased_view.rs` | `AnyView`, `ErasedView` — type erasure for `View` |
| 7 | `modifiers.rs` | `SharedElementModifier`, view modifiers |
| 8 | `render_base.rs` | Base render types |
| 9 | `renderer_trait.rs` | `Renderer` trait (drawing operations), `draw_text`, `draw_text_centered` |
| 10 | `renderer/mod.rs` | `Renderer` module — backend interface |
| 11 | `accessibility.rs` | A11y preferences, `A11yPrefs` |
| 12 | `render_tier.rs` | `RenderTier` — hardware acceleration tier |
| 13 | `mesh.rs` | `Camera3D` (struct + methods + default impl) |
| 14 | `spring.rs` | `SpringSolver`, animation spring types |
| 15 | `frame_renderer.rs` | `FrameRenderer` trait (frame lifecycle) |
| 16 | `state.rs` | `State<T>`, `Binding<T>` — reactive state with ArcSwap |
| 17 | `env_core.rs` | `YggdrasilKey`, `ENVIRONMENT` static (type-erased global store) |
| 18 | `env.rs` | Environment abstraction |
| 19 | `geometry_modifiers.rs` | Geometry modifier types |
| 20 | `layout.rs` | `LayoutView`, `Rect`, `SizeProposal`, `LayoutCache`, `LayoutKey` |
| 21 | `agents.rs` | AI agent support |
| 22 | `animation.rs` | Animation types |
| 23 | `gpu.rs` | GPU resource types |
| 24 | `material.rs` | `DrawMaterial` enum |
| 25 | `runtime.rs` | Runtime abstraction |
| 26 | `scene_graph.rs` | `NodeId`, `BifrostRegistry`, `bifrost_registry()` |
| 27 | `sdf_shadow.rs` | SDF shadow rendering |
| 28 | `color.rs` | `SemanticColors`, OKLCH color types |
| 29 | `event.rs` | Event system types |
| 30 | `suspense.rs` | `Suspense` component, async loading |
| 31 | `theme.rs` | `ThemeContext`, `use_theme()`, theme utilities |
| 32 | `hooks.rs` | Hook system (like React hooks for CVKG) |
| 33 | `a11y_prefs.rs` | A11y preferences types |
| 34 | `clipboard.rs` | Clipboard abstraction |
| 35 | `text_input.rs` | `Direction` enum, text input types |
| 36 | `notifications.rs` | Notification types |
| 37 | `file_dialog.rs` | File dialog types |
| 38 | `document.rs` | Document support |
| 39 | `menu.rs` | Menu types |
| 40 | `localization.rs` | Localization/i18n support |
| 41 | `system_theme.rs` | System theme detection |
| 42 | `audio_haptic.rs` | `AudioEngine`, `HapticEngine` traits |
| 43 | `parallax.rs` | `ParallaxModifier`, scroll depth effects |
| 44 | `identity.rs` | `KvasirId` — stable identity wrapper |
| 45 | `simple_geom.rs` | Simple geometry types |
| 46 | `dirty_flags.rs` | Dirty flag tracking |
| 47 | `dirty_region.rs` | Dirty region tracking |
| 48 | `virtual_window.rs` | Virtual window abstraction |
| 49 | `asset.rs` | `AssetKey`, `AssetState`, `DesignTokens` |
| 50 | `dependency.rs` | `DependencyGraph`, `FrameBudgetTracker` |
| 51 | `error_boundary.rs` | `ComponentErrorState`, `ErrorBoundary` |
| 52 | `knowledge.rs` | `AppState`, `KnowledgeFragment`, `MemoryLayer`, system state |
| 53 | `undo.rs` | `UndoGroup`, `UndoManager` |
| 54 | `window.rs` | `Window`, `WindowConfig`, `WindowId`, `WindowHandle` |
| 55 | `error_types.rs` | Error type definitions |
| 56 | `future_views.rs` | `HologramView`, `ParticleEmitter`, `StreamingText` |
| 57 | `security.rs` | Security utilities |

## Items remaining in lib.rs (not extracted)

These are the items that were **left in lib.rs** because they are small enough not to justify
their own module, or because they are aggregator/glue code:

| Item | Kind | Notes |
|------|------|-------|
| `RenderIntensityMode` | enum | Lines 197-203. Berserker render modes (Normal/Rage/Frenzy/GodMode) |
| `Seer` | trait | Lines 207-212. AI prediction trait for AI-assisted components |
| `impl Default for Camera3D` | impl | Lines 114-127. Could be moved into `mesh.rs` |
| `impl Camera3D` (view/projection) | impl | Lines 129-151. Camera matrix methods. Could be moved into `mesh.rs` |
| Module declarations + re-exports | — | Lines 44-313. 57 `pub mod` + `pub use` statements |

## Proposed follow-up extraction (if desired)

These small items in lib.rs could be moved into the modules they logically belong to:

| Item | Target file | Reason |
|------|-------------|--------|
| `impl Default for Camera3D` | `mesh.rs` | Camera3D is already defined there |
| `impl Camera3D` methods | `mesh.rs` | Already defined there |
| `RenderIntensityMode` | `render_base.rs` or new `render_intensity.rs` | Render pipeline concept |
| `Seer` | new `seer.rs` or `agents.rs` | AI ecosystem, only 2 methods |

## Design notes

- Every module uses `pub use module::*;` pattern — this is intentional for backward compatibility
  during the migration. Eventually specific re-exports could replace the globs.
- 3 modules (`error_types`, `future_views`, `security`) were kept outside the main module
  re-export cascade because they have minimal public surface.
- `env_core.rs` and `env.rs` could potentially be merged — they were split during extraction
  but share the same conceptual domain.

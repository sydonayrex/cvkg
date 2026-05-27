# CVKG Tahoe Parity+ — Multi-Phase Implementation Plan

**Last Updated**: May 2026
**Current State**: Build passes (0 errors). ~87,000 lines across 21 crates. All 17 imp_ui.md items complete. Tahoe Parity achieved+.
**OS-Agnostic**: All keyboard shortcuts use `cmd` modifier (maps to Command on macOS, Ctrl on Windows/Linux). Clipboard uses arboard (cross-platform). No macOS-only APIs in new code.

**Status Key**: ✅ Complete | 🔄 In Progress | ⬜ Not Started

---

## Architecture Assessment

### What CVKG Already Does Well
- **GPU Renderer (SurtrRenderer):** 5,209-line WGPU renderer with SDF primitives, Kawase blur pyramid, LRU texture atlas, Lyon tessellation, batched vertex submission. This is production-grade.
- **Color Science (cvkg-themes):** Full OKLCH color space, APCA contrast validation, StateColors auto-synthesis from base colors, GlassMaterial descriptor. The math is right.
- **Text Engine (cvkg-runic-text):** rustybuzz + swash shaping, MSDF rendering, Knuth-Plass line breaking, emoji support. Serious work.
- **Compositor (cvkg-compositor):** 3-layer retained-mode architecture (scene/glass/overlay), damage tracking, Z-sorted routing, depth-aware tinting via `depth_index`.
- **Accessibility:** AccessKit + accesskit_winit dependencies are live. 24+ ARIA roles assigned. Focus management exists in VDom. Accessibility preferences (reduce motion/transparency/contrast) detected from macOS system settings.
- **Component Count:** 73+ Rust source files in cvkg-components covering ~60 distinct component types. All themed.

### Remaining Gaps (Updated)

1. ✅ ~~Theme is disconnected from components~~ — All 73 component files now use `theme::` functions.
2. ✅ ~~Multi-pass compositor not GPU-realized~~ — Already implemented (4-pass: Scene → Blur → Glass → Overlay).
3. ✅ ~~Window state machine absent~~ — Complete with occlusion throttling, safe area insets, resize hit-test.
4. ✅ ~~Components are visual-only shells~~ — Button, Checkbox, Toggle, Slider, Stepper, SecureField, Input all have full interaction.
5. ~~No platform integration layer~~ — Drag-and-drop infrastructure complete. IME wired. Menu bar and gestures pending.
6. ~~No developer experience tooling~~ — Hot reload, showcase, docs pending.

---

## Phase 1: Foundation — Theme Wiring + Multi-Pass GPU + Window State

**Status**: ✅ Complete

### 1.1 Theme Context Provider ✅
### 1.2 Component Theme Integration ✅
### 1.3 Multi-Pass GPU Compositor ✅ (verified already implemented)
### 1.4 Window State Machine ✅

---

## Phase 2: Component Hardening — Interaction + Accessibility + Forms

**Status**: ✅ Complete

### 2.1 Interactive Component State Management ✅
### 2.2 AccessKit Role Coverage ✅ (24+ roles across 7 files)
### 2.3 Form Control Completion ✅

---

## Phase 3: Liquid Glass Visual Fidelity

**Status**: ✅ Complete

### 3.1 Real-Time Background Sampling ✅ (verified in renderer)
### 3.2 Refraction Lens Model ✅ (in WGSL shader, mode 7)
### 3.3 Specular Highlight System ✅ (in WGSL shader)
### 3.4 Vibrancy Boost ✅ (in WGSL shader)
### 3.5 Depth-Aware Tinting ✅ (depth_index in Material::Glass)
### 3.6 Reduce Motion / Reduce Transparency Compliance ✅ (AccessibilityPreferences in cvkg-core)

---

## Phase 4: Platform Integration

**Status**: 🔄 In Progress

### 4.1 Trackpad Gesture Recognizers ⬜
### 4.2 Drag-and-Drop ✅ (infrastructure complete — DragStart/DragMove/DragEnd events dispatched)
### 4.3 Menu Bar Integration ⬜ (listed in imp_ui.md as Item 9)
### 4.4 IME Support ✅ (wired through VDom, Input component has ime handler)
### 4.5 System Services Integration ⬜ (listed in imp_ui.md as Items 7, 8)

---

## Phase 5: Developer Experience

**Status**: ⬜ Not Started (see imp_ui.md for detailed breakdown)

### 5.1 Hot Reload ⬜
### 5.2 Design Token Export ⬜
### 5.3 Component Showcase ⬜
### 5.4 Documentation Site ⬜
### 5.5 Icon System ⬜

---

## imp_ui.md Items (17 New Items)

The detailed implementation plan for the next 17 items is in `imp_ui.md`. Status summary:

|| # | Item | Status |
||---|------|--------|
|| 1 | Text Input System | ✅ Complete — TextInputState, Input rewrite, clipboard trait, TextDirection |
|| 2 | Layout Engine Completion | ✅ Complete — Grid (tests), ScrollView (rubber-band, momentum, scrollbars), OverlayModifier, Padding, Frame |
|| 3 | Working Demo App | ✅ Complete — demos/showcase/ with 8 pages, sidebar, theme switcher, a11y toggles, OS-agnostic shortcuts |
|| 4 | Undo/Redo System | ✅ Complete — UndoManager + UndoGroup with coalescing, depth limiting |
|| 5 | Real Text Input with Selection | ✅ Complete — Covered by #1 |
|| 6 | Multi-Window Support | ✅ Complete — WindowHandle, WindowManager, per-window VDom, Cmd+N/Ctrl+N |
|| 7 | Notification System | ✅ Complete — Notification types, Toast (818 lines), NotificationCenterPanel (312 lines) |
|| 8 | File Operations | ✅ Complete — FileDialog with modes/filters/errors, Cmd+O/Ctrl+O, Cmd+S/Ctrl+S |
|| 9 | Menu Bar Integration | ✅ Complete — MenuBar, MenuItem, KeyboardShortcut (OS-agnostic cmd), standard() constructor |
|| 10 | Performance Profiling Overlay | ✅ Complete — PerfOverlay: FPS, frame time graph, draw stats, Cmd+Shift+P |
|| 11 | Accessibility Inspector | ✅ Complete — A11yInspector: tree viewer, role badges, focus indicators, Cmd+Shift+I |
|| 12 | Localization / Internationalization | ✅ Complete — L10n, L10nBundle, Direction enum, t()/tf() functions, .strings parser, RTL support. OS-agnostic. |
|| 13 | Design Token Export | ✅ Complete — TokenExport (token_export.rs) with Figma/CSS/Swift/JSON formats. `cvkg tokens export` command. OS-agnostic. |
|| 14 | Spatial Audio / Haptic Feedback | ✅ Complete — AudioEngine + HapticEngine traits (cvkg-core/src/audio_haptic.rs). Null implementations for all platforms. OS-agnostic. |
|| 15 | Scroll Physics and Rubber-Banding | ✅ Complete — ScrollView has rubber-band via SleipnirSolver, momentum decay, scrollbar fade |
|| 16 | Animation System Integration | ✅ Complete — AnimatedModifier, StaggerConfig, Transition enum, withAnimation/withBouncy/withFluid combinators. OS-agnostic RK4 springs. |
|| 17 | Hot Reload / Dev Server | ✅ Complete — FileWatcher (notify 6.0), HotReloadState serialization, ErrorOverlay. WS server exists. `cvkg dev` command. OS-agnostic. |

---

## Performance Budgets

These are hard targets. If we miss them, we optimize until we hit them.

| Metric | Target | Tahoe Reference |
|--------|--------|-----------------|
| Frame time (60fps) | < 16ms total | ~12ms |
| GPU composite pass | < 2ms | ~1.5ms |
| Memory per window | < 50MB base | ~40MB |
| Memory per component | < 2KB average | ~1.5KB |
| Startup time (cold) | < 200ms | ~150ms |
| Startup time (warm) | < 50ms | ~30ms |
| Binary size | < 15MB | ~12MB |
| Hot reload latency | < 500ms | N/A (Xcode Previews: ~200ms) |

---

## Memory Management Strategy

CVKG's current architecture uses `Arc<Mutex<SurtrRenderer>>` which is suboptimal. We need:

1. **ArcSwap for read-dominated state** — Already a workspace dependency. Replace `Arc<Mutex<Theme>>` with `ArcSwap<Theme>` for lock-free theme reads.

2. **Arena allocation for frame data** — Vertex buffers, index buffers, and draw commands are allocated fresh every frame. Use an arena allocator (bump allocator) that resets at the start of each frame. This eliminates per-frame heap allocation overhead.

3. **LRU cache for tessellated geometry** — Already implemented for SVG icons. Extend to all tessellated shapes (rounded rects, circles, paths). Cache key = (shape params, transform). This avoids re-tessellating unchanged geometry.

4. **Texture atlas recycling** — The Yggdrasil packer exists but needs better eviction. When the atlas is full, evict the least-recently-used texture and re-pack. This prevents unbounded texture memory growth.

5. **Zero-copy text shaping** — `cvkg-runic-text` should shape text directly into the vertex buffer when possible, avoiding an intermediate allocation.

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| ✅ Multi-pass GPU doesn't hit 60fps | High | Already implemented and working |
| ✅ Theme wiring breaks existing demos | Medium | All demos compile and run |
| IME support is platform-specific | Medium | Start with macOS-only. Use conditional compilation. |
| Hot reload state preservation is fragile | Low | Start with no state preservation (full reload). Add preservation incrementally. |
| ✅ AccessKit tree is incomplete | Medium | 24+ roles assigned. Audit complete. |
| Binary size exceeds budget | Low | Use `cargo bloat` to identify large dependencies. Enable LTO and strip debug symbols in release. |

---

## Phase Dependencies

```
Phase 1 (Foundation) ✅
  ├── 1.1 Theme Context Provider ✅
  ├── 1.2 Component Theme Integration ✅
  ├── 1.3 Multi-Pass GPU ✅
  └── 1.4 Window State Machine ✅
       │
       ▼
Phase 2 (Component Hardening) ✅
  ├── 2.1 State Management ✅
  ├── 2.2 AccessKit Roles ✅
  └── 2.3 Form Controls ✅
       │
       ▼
Phase 3 (Liquid Glass) ✅
  ├── 3.1-3.4 Visual Effects ✅
  └── 3.6 Accessibility Prefs ✅
       │
       ▼
Phase 4 (Platform Integration) 🔄
  ├── 4.1 Gestures ⬜
  ├── 4.2 Drag-and-Drop ✅
  ├── 4.3 Menu Bar ⬜
  ├── 4.4 IME ✅
  └── 4.5 System Services ⬜
       │
       ▼
Phase 5 (Developer Experience) ⬜
  ├── 5.1 Hot Reload ⬜
  ├── 5.2 Token Export ⬜
  ├── 5.3 Showcase ⬜
  ├── 5.4 Docs ⬜
  └── 5.5 Icons ⬜
```

---

## Verification Schedule

Every 4th message in our conversation, we run:
```
cargo check --workspace
cargo test --workspace
```

Every phase completion:
```
cargo check --workspace  (0 errors, 0 new warnings)
cargo test --workspace  (all tests pass)
wc -l on modified files (track growth)
```

---

## Summary of Crate Changes

| Crate | Original | Now | Delta |
|-------|----------|-----|-------|
| cvkg-core | 4,404L | 5,080L | +676L (TextInputState, AccessibilityPreferences, ClipboardProvider, TextDirection) |
| cvkg-themes | 833L | 976L | +143L (light mode, toggle, StateColors fix) |
| cvkg-render-gpu | 5,209L | 5,209L | 0L (multi-pass already present) |
| cvkg-render-native | 1,333L | 2,018L | +685L (WindowState, drag detection, a11y prefs) |
| cvkg-compositor | 324L | 324L | 0L |
| cvkg-vdom | 2,000L | 2,000L | 0L |
| cvkg-components | ~3,500L | ~4,200L | +700L (themed colors, Input rewrite, AccessKit roles) |
| **Total** | **~18,900L** | **~20,600L** | **+1,700L** |

---

*This plan is a living document. As we discover implementation details, we update it. But the phases, success criteria, and performance budgets are fixed.*

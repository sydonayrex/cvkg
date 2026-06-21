# Persona 1: Design Engineer (iOS Game Developer)

## Executive Summary

CVKG is a visually ambitious Rust UI framework with a SwiftUI-like View/Modifier pattern that offers genuinely impressive GPU-powered effects (frosted glass, neon glow, geometric shatter) and a solid spring animation engine. However, for an iOS game developer targeting 165Hz displays, it has critical gaps: no gamepad input, no multi-touch gesture recognizers, no custom shader injection API, and a Norse mythology naming convention that creates a steep cognitive tax. The framework is better suited for cyberpunk-themed desktop dashboards than production game UI — adopt for visual prototyping, not for shipping game HUDs.

## Onboarding Experience

**Time to first pixel: ~4-6 hours** for a Rust-proficient iOS dev; **8-12 hours** if learning Rust alongside.

The `cvkg/src/lib.rs` entry point is clean — three rendering pipelines (gpu/native/web) selected via Cargo features. The prelude exports `View`, `State`, `Binding`, `Rect`, and all components. The `cvkg-macros` crate provides `#[derive(View)]` and `view_component!` which feel similar to SwiftUI's `@ViewBuilder`.

However, the onboarding friction is significant:

1. **Rust barrier**: No Swift/Objective-C interop. You're committing to a full Rust stack with `wgpu` for GPU rendering. For an iOS game dev used to Xcode, Instruments, and Metal, this is a paradigm shift.
2. **Naming tax**: Every API is named after Norse mythology. `bifrost()` = frosted glass, `gungnir()` = neon glow, `mjolnir_slice()` = geometric clip, `mjolnir_shatter()` = fragment transition, `sleipnir` = spring animation, `ginnungagap` = no-op animation, `vegvísir` = radial menu, `tacticalgauge` = progress bar. You'll need a cheat sheet open at all times. This is the single biggest onboarding blocker.
3. **Feature flag confusion**: The `native` feature wraps `gpu` with `winit`+AccessKit, but the docs at `cvkg/src/lib.rs:11` say "Uses `winit` and `AccessKit` to wrap the `gpu` renderer" — this is misleading since `winit` doesn't support iOS. There's no Metal backend.
4. **The Berserker demo** (`demos/berserker/src/main.rs`) is impressive — it shows a full game-like scene with particles, fluid simulation (Navier-Stokes at line 137), RK4 spring physics, shatter effects, and a PerfOverlay. But it's 1,494 lines of dense Rust with no tutorial explaining how to replicate it step by step.

## SwiftUI Comparison

| SwiftUI | CVKG | Notes |
|---------|------|-------|
| `struct MyView: View { var body: some View }` | `struct MyView; impl View for MyView { type Body = Never; fn body(self) -> Self::Body { unreachable!() } }` | CVKG uses `Never` for primitive views (no children). Composite views return `ModifiedView`. More boilerplate. |
| `.foregroundColor()`, `.padding()`, `.background()` | `.foreground_color()`, `.padding()`, `.background()` | Direct mapping. CVKG uses `[f32; 4]` for colors instead of SwiftUI's `Color`. |
| `.onTapGesture {}` | `.on_click(\|\| { ... })` | CVKG's `on_click` takes `Fn() + Send + Sync + 'static` — no event parameter, no tap location. |
| `.onDrag {}` | ❌ Not available | No drag gesture recognizer in the View trait. Only `on_drag_start` exists in `FileTree` component. |
| `.gesture(DragGesture())` | ❌ Not available | No composable gesture system. |
| `.animation(.spring())` | `.modifier()` with `Animation::Sleipnir(SpringParams::fluid())` | CVKG's spring system is more powerful (RK4 solver, 4 presets) but not as ergonomic. |
| `@State var count = 0` | `Signal::new(0)` / `Binding` | CVKG uses explicit signals. Similar to `@StateObject` + `ObservableObject`. |
| `NavigationStack { }` | `NavigationStack` (from `cvkg_components::navigation`) | Direct equivalent exists. |
| `TabView { }` | `BifrostTabs` | Exists but Norse-named. |
| `.sheet(isPresented:) { }` | `.sheet(is_presented, content)` | Via `ViewExt` trait. Direct mapping. |
| `withAnimation { }` | `Animation` enum + `Motion` struct | CVKG requires explicit animation enum variants rather than implicit animation context. |
| `GeometryReader` | `intrinsic_size()` + `SizeProposal` | CVKG's layout is more explicit — you implement `intrinsic_size()` to report natural size. |
| `LazyVStack` | `LazyVStack` | Direct equivalent. |
| `ForEach` | ❌ No direct equivalent | CVKG uses `Vec<View>` or `HStack`/`VStack` with manually composed children. |
| `ViewModifier` protocol | `ViewModifier` trait | Nearly identical concept. CVKG's has `render_view`, `transform_rect`, `transform_proposal`, `transform_size`. |
| `AnyView` | `AnyView` | Direct equivalent with type erasure via `ErasedView` trait. |

**Key gap**: SwiftUI's `@ViewBuilder` result builder pattern has no equivalent in CVKG. Composing views requires manual `HStack::new().child(...)` style or direct struct construction, which is more verbose than SwiftUI's trailing closure syntax.

## Game UI Component Coverage

### What's There (Good)

**HUD Components** (`cvkg-components/src/hud.rs`):
- `TacticalGauge` — Horizontal bar gauge with warning/critical thresholds, kinetic flicker animation. Good for health/energy bars. API: `TacticalGauge::new("HP", 0.75).warning_level(0.5).critical_level(0.9)`.
- `GjallarAlert` — Toast notification with `AlertKind::{Information, Warning, Critical}`. Glassmorphic rendering with pulsing border. Good for damage indicators.
- `Vegvísir` — Radial menu (Norse compass). Supports `Arc<dyn Fn(usize)>` callbacks. Decal-style game menus.

**Progress/Status** (`cvkg-components/src/visual.rs`):
- `SkollProgress` — Linear + circular progress indicator. `ProgressVariant::{Linear, Circular}`.
- `StatusBar` — Text + color bar for system indicators.
- `HatiSpinner` — Loading spinner with `SpinnerVariant`.
- `DraumaSkeleton` — Skeleton loading placeholder.

**Text Animations** (`cvkg-components/src/text_anim.rs`):
- `TypewriterEffect` — Character-by-character reveal with blinking cursor. Perfect for dialogue systems.
- `NumberTicker` — Animated number counter with prefix/suffix. Good for score displays.
- `CardStack` — Stacked cards with depth/parallax. Could work for card game UIs.
- `CardHoverEffect` — 3D tilt on hover (progress 0.0-1.0).
- `DraggableCard` — Draggable card component.
- `RippleButton`, `ShimmerButton` — Animated button variants.

**Data Visualization** (`cvkg-components/src/visual.rs`):
- `ValkyrieAnalytics` — Line/Scatter/Bar/Radar charts. Good for post-game stats.
- `TelemetryView` — Real-time FPS/frame-time display with bifrost glassmorphism.

**Particles** (`cvkg-core/src/future_views.rs`):
- `ParticleEmitter` — GPU particle system. `ParticleEmitter::new(64, "fire", [1.0, 0.5, 0.0, 1.0])`. Calls `renderer.dispatch_particles()`.

### What's Missing (Critical for Games)

- **No minimap component** — Essential for strategy/RPG games.
- **No inventory grid** — No `GridView` with drag-and-drop slots.
- **No cooldown timer** — No circular countdown overlay for ability cooldowns.
- **No damage number flyout** — No floating text animation for combat numbers.
- **No waypoint/compass bar** — No directional indicator component.
- **No dialogue box** — `TypewriterEffect` exists but no full dialogue box with portrait + nameplate.
- **No scrollable inventory list** — `ScrollView` exists but no `LazyVGrid` equivalent for item grids.
- **No tabbed equipment screen** — `BifrostTabs` exists but no equipment slot pattern.

## Animation & Visual Effects

### Spring System: Excellent ★★★★☆

The `cvkg-anim/src/lib.rs` animation engine is genuinely impressive:

- **RK4 Spring Solver** (`SpringSolver`): 4th-order Runge-Kutta integration for stable, high-fidelity physics. This is the same class of solver used in professional game engines.
- **4 Spring Presets**: `snappy()` (230/22), `fluid()` (170/26), `heavy()` (90/20), `bouncy()` (190/14) — stiffness/damping values. These cover most game UI motion needs.
- **Hybrid Animations**: Keyframe path + spring settle. Perfect for card-flip animations.
- **Sequence/Parallel/Stagger**: Full animation composition. `Stagger` with interval timing is great for list animations.
- **Momentum/Decay**: Inertial scrolling with friction. `RubberBand` utility with iOS-like logarithmic resistance.
- **ProgressDriver::{Time, Scalar}`**: Scroll-linked animations for parallax effects.

**Cytus/Amonite-quality motion?** Yes, the spring solver is capable. The `MjolnirShatter` animation (line 563-615) even simulates per-fragment spring physics for shatter effects. However, there's no `UIViewPropertyAnimator`-equivalent for interactive scrubbing — you can't easily drive an animation with a pan gesture's progress.

### Text Animations: Good ★★★★☆

`TypewriterEffect` with cursor blinking, `NumberTicker` with rolling digits, `TextAnimate` with Fade/Slide/Scale/Blur effects. These are production-ready for game dialogue and score displays.

### Visual Effects: Impressive but Opaque ★★★★☆

- **Bifrost** (Frosted Glass): `renderer.bifrost(rect, blur, saturation, opacity)` — real backdrop blur with fresnel. Has automatic quality degradation when `is_over_budget()` (line 974).
- **Gungnir** (Neon Glow): `renderer.gungnir(rect, color, radius, intensity)` — additive glow.
- **Mjolnir Slice**: `renderer.push_mjolnir_slice(angle, offset)` — SDF-based geometric clip.
- **Mjolnir Shatter**: Radial fragment decomposition with per-piece spring physics.
- **NiflheimFrost**: Frosted glass wrapper with Liquid Glass morphing corners.
- **HeimdallSweep**: Radar sweep effect.
- **Seiðr**: Holographic scanline projection.
- **LokiGlitch**: Digital distortion text effect.

**Quality assessment**: The effects are visually ambitious and the frame budget system (`is_over_budget()` at `cvkg-core/src/lib.rs:2157`) provides automatic degradation. However, the effects are hardcoded in the renderer — you can't customize the shader logic without modifying the framework source.

## Performance & Frame Budget

### Frame Budget System: Good ★★★★☆

`PerformanceContract` (`cvkg-core/src/parallax.rs:69`) provides:
- `frame_budget_ms: f32` — Target frame time (e.g., 6.06ms for 165Hz, 16.67ms for 60Hz)
- `frame_budget_remaining_ms: f32` — Remaining budget
- `is_over_budget()` — Query in modifiers to degrade quality

The `FrostedGlassModifier` (line 974) demonstrates the pattern: when over budget, it halves the blur radius. This is a good start but **not sufficient for 165Hz**:

1. **No per-component budget allocation** — You can't say "this HUD element gets 2ms, particles get 3ms."
2. **No GPU timer queries** — `PerfOverlay` uses CPU-side `Instant::now()` timestamps. The `gpu_time_ms` field exists but is never populated.
3. **165Hz = 6.06ms budget** — The `PerfOverlay` graph's target line is hardcoded at 16.67ms (60Hz) at `perf_overlay.rs:265`. No way to change it.
4. **No frame pacing** — No `CADisplayLink`-equivalent for vsync-locked frame submission.

### PerfOverlay: Adequate ★★★☆☆

`PerfOverlay` (`cvkg-components/src/perf_overlay.rs`) shows:
- FPS, frame time, draw calls, triangles, vertices
- Average and P99 frame times
- Rolling frame-time bar graph with color-coded thresholds
- 120-frame sample window

**Gaps for game dev**:
- No GPU frame time (only CPU-side estimation)
- No memory usage tracking
- No thermal throttling detection
- Toggle is `Cmd/Ctrl+Shift+P` — not accessible from the API (`show()` method exists but sets `visible = false` at line 49, which looks like a bug)
- No frame time histogram (only bar graph)
- No export/snapshot capability for performance regression testing

## Shader & GPU Pipeline Access

### Current State: Limited ★★☆☆☆

**No public shader injection API.** The GPU renderer (`cvkg-render-gpu`) uses Surtr/Muspelheim shaders internally, but there's no `CustomShader` view modifier or `ShaderLibrary` equivalent. The `renderer` trait has methods like `bifrost()`, `gungnir()`, `push_mjolnir_slice()` — these are fixed-function, not programmable.

**What you'd need for game UI:**
- Custom WGSL shader injection (e.g., dissolve transitions, holographic shaders, distortion effects)
- Shader uniform binding from the view layer
- Access to the render pass for custom draw calls
- Texture sampling from game assets

**Workaround**: The `renderer` trait is extensible — you could add custom methods to a renderer implementation. But this requires forking the framework, not using it as a library.

**Comparison**: Unity's URP has `ShaderGraph`, Unreal has `MaterialEditor`, SpriteKit has `SKShader`. CVKG has none of this.

## Input Handling

### Current State: Insufficient ★★☆☆☆

**Pointer Events** (available via `View` trait):
- `on_click<F: Fn()>()` — Click/tap. No event args (no location, no button).
- `on_pointer_enter<F: Fn()>()` — Hover enter.
- `on_pointer_leave<F: Fn()>()` — Hover leave.
- `on_pointer_move<F: Fn(f32, f32)>()` — Mouse move with x, y.
- `on_pointer_down<F: Fn()>()` — Pointer press.
- `on_pointer_up<F: Fn()>()` — Pointer release.

**Keyboard Events**:
- `on_key_event(&self, key: &str, modifiers: KeyModifiers) -> bool` — Raw key handling.
- `key_shortcuts()` — `KeyShortcut` with `KeyModifiers { shift, ctrl, alt, meta }`.
- `FocusManager` — Tab/Shift+Tab navigation with focus traps.

**Critical Gaps for Game Dev**:

| Input Type | SwiftUI | CVKG | Impact |
|-----------|---------|------|--------|
| Gamepad/Controller | `GCController` | ❌ Not supported | **Blocker** for console-style games |
| Multi-touch | `GestureState` | ❌ Single pointer only | **Blocker** for mobile games |
| Drag gesture | `DragGesture` | ❌ Not in View trait | Can't implement drag-to-reorder |
| Long press | `LongPressGesture` | ❌ Not available | Can't implement context menus |
| Pinch/Zoom | `MagnificationGesture` | ❌ Not available | Can't implement map zoom |
| Rotation | `RotationGesture` | ❌ Not available | Can't implement rotation controls |
| Swipe | `DragGesture` + velocity | ❌ Not available | Can't implement swipe actions |
| Accelerometer | `CoreMotion` | ❌ Not available | Can't implement tilt controls |
| Haptic feedback | `UIImpactFeedbackGenerator` | `HapticEngine` trait exists | ✅ Abstracted but platform support unclear |

The `MagneticPullModifier` (line 1086) reads `renderer.get_pointer_position()` — proving the renderer tracks pointer position — but there's no multi-touch support, no gesture velocity, and no gesture state machine.

## Physics Integration

### cvkg-physics: Powerful but Disconnected ★★★☆☆

`cvkg-physics` (`cvkg-physics/src/lib.rs`) is a full 2D rigid body physics engine:
- Impulse-based constraint solving (Gauss-Seidel)
- GJK/EPA narrow-phase collision
- Spatial hash broad-phase
- Semi-implicit Euler integration
- Shapes: Circle, AABB, ConvexHull, Capsule
- Constraints: distance, pin, hinge, angular limit
- `scene_bridge.rs` — reads/writes `cvkg-scene` NodeId transforms
- `mjolnir_bridge` — `shatter_at_constraint_break()`, `shatter_at_position()`

**The problem**: `cvkg-physics` is a standalone crate with no integration into the component layer. There's no `PhysicsBody` view modifier, no `on_collision` event on views, and no way to attach a physics body to a UI element from the component API. The `scene_bridge` connects to `cvkg-scene` (a scene graph), not to `cvkg-components`.

**For game UI**: You'd want to attach physics to HUD elements (e.g., a health bar that bounces when hit). Currently, you'd need to manually sync `cvkg-physics` bodies with view positions in your render loop — the Berserker demo does this at `demos/berserker/src/main.rs:559-600` with manual `fluid.step()` and particle updates.

## Theming for Game UIs

### Theme System: Excellent ★★★★☆

`cvkg-themes/src/lib.rs` provides:

**OKLCH Color Space**: Perceptually uniform color manipulation. `OklchColor::new(lightness, chroma, hue, alpha)` with `lighten()`, `darken()`, `saturate()`, `rotate_hue()`. This is superior to SwiftUI's HSL-based `Color` for creating harmonious palettes.

**Theme Generation**:
```rust
let theme = Theme::dark();  // Cyberpunk defaults
let theme = Theme::from_seed(OklchColor::new(0.55, 0.12, 260.0, 1.0));
let theme = ThemeBuilder::dark()
    .with_primary(Color::new(1.0, 0.2, 0.4, 1.0))  // Neon pink
    .with_glass_blur(30.0)
    .with_density(Density::Compact)
    .build();
```

**Dark Mode**: First-class support. `Theme::dark()` produces a deep void background (0.02, 0.02, 0.05) with Viking Gold primary and Cyan accent. Perfect for game UIs out of the box.

**Neon Aesthetic**: The default theme is cyberpunk by design — `primary_neon`, `shatter_neon`, `glass_base`, `glass_edge`, `rune_glow`, `ember_core` are all first-class theme tokens. For a Cytus-style rhythm game, this is ideal.

**Glass Materials**: `GlassMaterial` with backdrop blur, refraction index, frost intensity, tint, and border glow. `GlassMaterial::default_glass()` provides sensible defaults.

**Accessibility**: APCA contrast validation built into the theme system. `validate_accessibility()` checks all text/background pairs. `AccessibilityOverrides` supports `reduce_transparency`, `reduce_motion`, `increase_contrast`.

**Density**: `Density::{Compact, Default, Spacious}` with 0.75x/1.0x/1.25x multipliers. Good for adapting UI between phone and tablet.

**Verdict**: The theming system is production-quality and well-suited for game UIs. The OKLCH color science is a genuine advantage over SwiftUI's color system.

## Gaps & Recommendations

### P0: Critical Blockers

1. **No Gamepad Support**
   ```rust
   // What's needed:
   fn on_gamepad_button<F: Fn(GamepadButton, f32) + Send + Sync + 'static>(
       self, action: F
   ) -> ModifiedView<Self, OnGamepadButtonModifier>
   ```
   The `winit` backend could expose gamepad events via `winit::event::DeviceEvent`. The `gilrs` crate is the standard Rust gamepad library. Without this, CVKG can't be used for console-style games or controller-navigated menus.

2. **No Multi-Touch / Gesture Recognizers**
   ```rust
   // What's needed:
   fn on_drag<F: Fn(DragGesture)>(self, action: F) -> ModifiedView<Self, DragGestureModifier>
   fn on_pinch<F: Fn(f32)>(self, action: F) -> ModifiedView<Self, PinchGestureModifier>
   fn on_long_press<F: Fn()>(self, duration: Duration, action: F) -> ModifiedView<Self, LongPressModifier>
   ```
   The current `on_pointer_*` events are single-pointer only. Games need multi-touch for pinch-to-zoom maps, drag-to-reorder inventories, and long-press context menus.

3. **No Custom Shader Injection**
   ```rust
   // What's needed:
   struct CustomShader {
       wgsl_source: String,
       uniforms: Vec<UniformBinding>,
   }
   fn shader(self, shader: CustomShader) -> ModifiedView<Self, ShaderModifier>
   ```
   Game UIs frequently need custom effects (dissolve, hologram, distortion). The renderer trait should expose a `draw_custom_shader()` method.

### P1: High Priority

4. **Norse Naming Tax** — Provide English aliases:
   ```rust
   // Current:
   view.bifrost(20.0, 1.2, 0.85)
   view.gungnir("cyan", 15.0, 0.8)
   view.mjolnir_slice(45.0, 0.0)
   
   // Recommended:
   view.frosted_glass(20.0, 1.2, 0.85)  // or view.blur(...)
   view.neon_glow("cyan", 15.0, 0.8)    // or view.glow(...)
   view.geometric_clip(45.0, 0.0)       // or view.clip_angle(...)
   ```
   Keep the Norse names as aliases but add English-primary APIs. The `#[doc(alias = "bifrost")]` attribute can maintain backward compatibility.

5. **No `ForEach` / Dynamic View Generation**
   ```rust
   // SwiftUI:
   ForEach(inventory.items) { item in ItemSlot(item) }
   
   // CVKG — no equivalent. You must manually build Vec<Box<dyn View>>.
   ```
   Add a `ForEach` component that takes a `Vec<T>` and a closure `Fn(&T) -> impl View`.

6. **No Cooldown/Ability Timer Component**
   ```rust
   // What's needed:
   struct CooldownOverlay {
       remaining: f32,  // seconds
       total: f32,
       icon: String,
   }
   ```
   Circular countdown with sweep animation. Essential for MOBA/RPG ability bars.

7. **No Inventory Grid Component**
   ```rust
   // What's needed:
   struct InventoryGrid {
       slots: Vec<Slot>,
       columns: usize,
       slot_size: f32,
       on_drag_start: Fn(usize),
       on_drag_end: Fn(usize, usize),  // from, to
   }
   ```

### P2: Medium Priority

8. **PerfOverlay 165Hz Support** — Make the target frame time configurable:
   ```rust
   PerfOverlay::new().with_target_fps(165.0)  // Currently hardcoded at 60Hz
   ```

9. **No Damage Number / Floating Text Component** — Common in games:
   ```rust
   struct FloatingText {
       text: String,
       origin: [f32; 2],
       velocity: [f32; 2],
       color: [f32; 4],
       lifetime: f32,
   }
   ```

10. **No `UIViewRepresentable` Equivalent** — No way to embed platform-native views (e.g., `MTKView` for Metal rendering, `SKView` for SpriteKit). This means you can't render 3D game content behind/within CVKG UI.

11. **Frame Budget Per-Component** — The `is_over_budget()` check is global. Games need per-component budget allocation:
    ```rust
    view.performance_tier(PerformanceTier::Critical)  // Always full quality
    view.performance_tier(PerformanceTier::Background) // Degrade first
    ```

12. **No Audio Engine Integration** — `AudioEngine` trait exists (`cvkg-core/src/lib.rs`) but `NullAudioEngine` is the default. No spatial audio, no sound effect triggering from UI events.

### P3: Nice to Have

13. **No `matchedGeometryEffect` Equivalent** — `bifrost_bridge()` (shared element transitions) exists but requires manual ID management. SwiftUI's `matchedGeometryEffect` is more ergonomic.

14. **No Preview Provider** — SwiftUI's `#Preview` lets you see views in Xcode. CVKG has no equivalent. The Berserker demo is the only way to see components.

15. **No Asset Catalog** — No equivalent of `UIImage(named:)` or `Color(named:)`. Assets are loaded via `AssetKey` but there's no hot-reload or catalog system.

## Verdict

**Score: 5.5/10** — Promising foundation, not production-ready for game UI.

**Adoption likelihood: Low** for shipping games, **Medium** for prototypes and tools.

### Strengths
- Best-in-class spring animation engine (RK4 solver, 4 presets, hybrid keyframe+spring)
- Genuinely impressive GPU effects (bifrost glass, gungnir glow, mjolnir shatter)
- OKLCH color science for theming is superior to SwiftUI's HSL
- Frame budget system with automatic quality degradation
- 215+ components with good coverage of standard UI patterns
- Full 2D physics engine (cvkg-physics) available alongside UI

### Weaknesses
- **No gamepad support** — Blocker for console/controller games
- **No multi-touch/gestures** — Blocker for mobile games
- **No custom shader injection** — Can't achieve unique visual styles
- **Norse naming convention** — Significant cognitive overhead, no English aliases
- **No iOS/Metal backend** — `winit` doesn't support iOS; no `UIViewController` integration
- **Physics disconnected from components** — Can't attach physics to views declaratively
- **No `ForEach` / dynamic view generation** — Can't render lists from data
- **PerfOverlay hardcoded to 60Hz** — Not suitable for 165Hz targets
- **No preview/live-reload tooling** — Slow iteration cycle

### Recommendation

If you're an iOS game developer looking for a cross-platform UI framework for game overlays, **CVKG is not ready**. The animation engine and visual effects are excellent, but the input handling gaps (no gamepad, no multi-touch, no gestures) and lack of shader access make it unsuitable for production game UI.

**Better alternatives for iOS game dev:**
- **SpriteKit** (`SKScene` + `SKLabelNode` + `SKShader`) — Native, Metal-backed, gamepad support
- **Metal + Dear ImGui** — Maximum control, custom shaders, but more code
- **Unity UI Toolkit** — If you're already in Unity

**When to consider CVKG:**
- Building a cyberpunk-themed desktop game tool/level editor
- Prototyping UI animations before implementing in-engine
- Cross-platform game dashboard (desktop + web) where gamepad isn't needed
- You're already committed to Rust for your game engine

The framework needs gamepad support, gesture recognizers, custom shader injection, and English API aliases before it can be recommended for game UI. These are all technically feasible additions — the architecture is sound — but they're not trivial.

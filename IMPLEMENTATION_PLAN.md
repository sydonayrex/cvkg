# CVKG Implementation Plan: 16 Gaps + 8 SVG App Maintainer Issues

**Date:** 2026-06-14
**Targets:** LIQUID_GLASS_REVIEW.md gaps (P0-P3) + SVG app maintainer workarounds
**Version:** 0.2.13 target

---

## P0: Critical (This Week)

### P0-1: Fix Glass Pipeline Tests

**Problem:** `test_glass_pipeline_renders`, `test_glass_pipeline_debug`, `test_glass_pipeline_is_valid` are `#[ignore]`d.

**Root cause:** `fill_glass_rect` produces black output — the glass shader (mode 7) has a pipeline bug.

**Steps:**
1. Read `cvkg-render-gpu/tests/hello_world.rs` lines 134-320 (glass test code)
2. Trace the draw call: `fill_glass_rect` → `fill_rect_with_full_params` → mode 7
3. Check `material_glass.wgsl` `@fragment fs_main` for:
   - Missing `t_env`/`s_env` bind group setup in test harness
   - Glass output bind group layout mismatch (texture + sampler not bound)
   - `textureSampleLevel` on unbound texture = black
4. Fix the renderer's glass pass to bind a dummy white texture when no backdrop is available
5. Un-ignore all three tests in `hello_world.rs`
6. Run `cargo test -p cvkg-render-gpu` and verify pixel output

**File:** `cvkg-render-gpu/tests/hello_world.rs`
**Owner:** render-gpu crate (renderer.rs + api.rs)

---

### P0-2: Add CI Accessibility Gate

**Problem:** `Theme::validate_accessibility()` exists but is never called in CI.

**Steps:**
1. In `cvkg-themes/tests/themes_tests.rs`, change `test_accessibility_validation` from info-only to assert:
   ```rust
   #[test]
   fn test_accessibility_validation() {
       let theme = Theme::dark();
       let results = theme.validate_accessibility();
       for result in &results {
           assert!(result.passes, "APCA fail: {} Lc={:.1}", result.level, result.contrast);
       }
   }
   ```
2. Add same test for `Theme::light()` and `Theme::from_seed(OklchColor::new(0.55, 0.12, 260.0, 1.0))`
3. Add to `cvkg-render-native` integration test: after creating default theme, call `validate_accessibility()` and assert all pass
4. Verify with `cargo test -p cvkg-themes`

**File:** `cvkg-themes/tests/themes_tests.rs`

---

### P0-3: Fix SVG Rendering Order (SVG Before Canvas)

**Problem:** SVG drawing appears to render before the canvas drawing. The biggest issue from the SVG app maintainer.

**Root cause:** `draw_svg` uses material_id=0 (Opaque) but the draw call ordering in `render_frame()` processes all draw calls (including SVG) before the compose/blit pass that draws the canvas.

**Steps:**
1. In `cvkg-render-gpu/src/api.rs`, check `draw_svg` — it fills vertices/indices into the same buffer as canvas draws but with material_id=0
2. The Kvasir graph in `end_frame()` processes: Opaque pass → Glass pass → Compose pass
3. SVG content drawn via `draw_svg` goes into the Opaque pass, but the canvas background (if drawn as a rect) also goes into Opaque pass
4. **Fix:** Add a `layer_index` field to draw calls. SVG calls should get a higher layer index than UI chrome but lower than overlays. Sort draw calls within the Opaque pass by layer_index.
5. Alternatively: Add `begin_svg_layer()` / `end_svg_layer()` API that sets a `pending_svg_layer` flag in the renderer. When the Opaque pass encounters SVG draws, it batches them separately and renders them after opaque UI.

**Preferred fix:** Add `draw_order` field to `RenderCommand` in compositor:
```rust
pub struct RenderCommand {
    pub material: DrawMaterial,
    pub draw_order: i32,  // 0 = background, 100 = UI, 200 = SVG content, 300 = overlay
    // ... existing fields
}
```

Sort `scene_commands` by `(material_id, draw_order)` before encoding render passes.

**Files:** `cvkg-compositor/src/engine.rs`, `cvkg-render-gpu/src/api.rs`

---

### P0-4: Add Default Canvas Background

**Problem:** "No default canvas bg (v0.2.11+)" — apps must explicitly draw a background rect.

**Root cause:** `render_frame()` in api.rs no longer clears to a default color; it only clears the depth buffer.

**Steps:**
1. Add `default_background_color: [f32; 4]` field to `SurtrRenderer` (or to `SceneConfig`)
2. Initialize to `[0.02, 0.02, 0.05, 1.0]` (matching Deep Void theme default)
3. In `render_frame()`, if `has_writes` is false (no app draws), still clear the surface to `default_background_color`
4. If the app draws its own background, this is a no-op (overwritten by app's draw)
5. Expose `set_default_background_color(color: [f32; 4])` in the `Renderer` trait

**Files:** `cvkg-render-gpu/src/renderer.rs`, `cvkg-render-gpu/src/api.rs`

---

## P1: High Priority (Next 2 Weeks)

### P1-1: Per-Component Glass Intensity

**Problem:** No way to control glass intensity per-component; only global `reduce_transparency` toggle.

**Steps:**
1. Add `glass_intensity: f32` field to `UiParams` (or a new `GlassParams` struct):
   ```rust
   pub struct GlassParams {
       pub intensity: f32,  // 0.0 = solid, 1.0 = full glass
       pub blur_radius: f32,
       pub frost_intensity: f32,
   }
   ```
2. Default to `intensity: 1.0`
3. In `fill_glass_rect()`, multiply tint alpha and blur strength by intensity:
   ```rust
   let effective_blur = blur_radius * params.glass_intensity;
   let effective_alpha = 0.4 * params.glass_intensity;
   ```
4. Add `with_glass_intensity(self, intensity: f32)` to component builders that use glass
5. Wire `AccessibilityOverrides.reduce_transparency` to set global `glass_intensity = 0.0`

**Files:** `cvkg-render-gpu/src/api.rs`, `cvkg-components/src/` (component builders)

---

### P1-2: Improved Backdrop Sampling

**Problem:** Only 4 samples at mip-6 for backdrop dominant color.

**Steps:**
1. In `material_glass.wgsl`, replace `sample_backdrop_dominant`:
   ```wgsl
   fn sample_backdrop_dominant(uv: vec2<f32>) -> vec3<f32> {
       // 9-tap sample at mip-4 (faster than mip-6 on most GPUs)
       let offsets = array<vec2<f32>, 9>(
           vec2(-0.05, -0.05), vec2(0.0, -0.05), vec2(0.05, -0.05),
           vec2(-0.05,  0.0),  vec2(0.0,  0.0),  vec2(0.05,  0.0),
           vec2(-0.05,  0.05), vec2(0.0,  0.05), vec2(0.05,  0.05)
       );
       var sum = vec3<f32>(0.0);
       for (var i = 0u; i < 9u; i++) {
           sum += textureSampleLevel(t_env, s_env, uv + offsets[i], 4.0).rgb;
       }
       return sum / 9.0;
   }
   ```
2. Add backdrop variance detection:
   ```wgsl
   fn backdrop_variance(uv: vec2<f32>) -> f32 {
       let mean = sample_backdrop_dominant(uv);
       var var_sum = 0.0;
       // compute variance across 9 taps; return 0.0-1.0
       // high variance = complex backdrop = reduce tint adapt
   }
   ```
3. Use variance to dynamically adjust `glass_tint_adapt`:
   ```wgsl
   let adapt = mix(0.1, theme.glass_tint_adapt, 1.0 - backdrop_variance(screen_uv));
   ```

**File:** `cvkg-render-gpu/src/shaders/material_glass.wgsl`

---

### P1-3: Reduce Motion Component Integration

**Problem:** `AccessibilityOverrides.reduce_motion` exists but components don't honor it.

**Steps:**
1. Add `should_reduce_motion()` check to the animation system:
   ```rust
   // In cvkg-anim or cvkg-render-native
   if AccessibilityPreferences::current().should_reduce_motion() {
       // Skip spring animation, snap to final position
       return final_value;
   }
   ```
2. Audit all components in `cvkg-components/src/` for animation usage:
   - Button: hover/active transitions
   - Modal: enter/exit transitions
   - TabBar: indicator slide
   - Slider: thumb bounce
3. Add `#[test] fn test_reduce_motion_disables_animations()` to each animated component
4. Wire `SleipnirParams::snappy()` with `reduce_motion` override:
   ```rust
   fn animation_params(&self) -> SleipnirParams {
       if self.a11y.reduce_motion {
           return SleipnirParams::instant(); // zero duration
       }
       self.theme.motion.snappy
   }
   ```

**Files:** `cvkg-anim/src/lib.rs`, `cvkg-components/src/` (per-component audit)

---

### P1-4: Clamp Resize Events

**Problem:** Resize events can be 0 or oversize, causing panics.

**Steps:**
1. In `cvkg-render-native/src/lib.rs`, find the resize handler (around line 724-734)
2. Add clamping:
   ```rust
   fn on_resize(&mut self, width: u32, height: u32) {
       let w = width.clamp(2, 16384);  // 16384 = max texture size on most GPUs
       let h = height.clamp(2, 16384);
       if w == 0 || h == 0 { return; } // Skip zero-size frames
       // ... existing resize logic
   }
   ```
3. Add test: create window, resize to 0x0, resize to 20000x20000, verify no panic

**File:** `cvkg-render-native/src/lib.rs`

---

## P2: Medium Priority (Next Month)

### P2-1: Animation Vocabulary Documentation

**Problem:** No central "animation vocabulary" document. Inconsistent use of `snappy`/`fluid`/`heavy`/`bouncy`.

**Steps:**
1. Create `docs/ANIMATION_VOCABULARY.md`:
   ```
   ## CVKG Animation Vocabulary
   
   | Interaction | Spring Params | Duration | Rationale |
   |------------|---------------|----------|-----------|
   | Button press | snappy | ~150ms | Immediate feedback |
   | Hover enter | fluid | ~200ms | Subtle awareness |
   | Modal open | heavy | ~300ms | Authoritative entrance |
   | Modal close | snappy | ~150ms | Fast dismissal |
   | Tab switch | fluid | ~250ms | Spatial continuity |
   | Error shake | bouncy | ~400ms | Attention-grabbing |
   | Drag start | heavy | ~200ms | Weight/authority |
   | Tooltip show | snappy | ~100ms | Instant information |
   ```
2. Update component doc comments to reference vocabulary
3. Add lint in `cvkg-anim`: warn if spring params deviate >20% from vocabulary defaults

**File:** `docs/ANIMATION_VOCABULARY.md`

---

### P2-2: Density Variants

**Problem:** Single spacing scale; no compact/cozy/spacious variants.

**Steps:**
1. Add `Density` enum to `cvkg-themes`:
   ```rust
   pub enum Density {
       Compact,  // 0.75x spacing
       Default,  // 1.0x spacing
       Spacious, // 1.25x spacing
   }
   ```
2. Store in `Theme` as `pub density: Density`
3. Effective spacing = base scale * density multiplier
4. `Theme::compact()`, `Theme::spacious()` constructors
5. Apply to container padding only (not internal component spacing)

**File:** `cvkg-themes/src/lib.rs`

---

### P2-3: Icon System (Basic)

**Problem:** No icon registry. `fill_squircle()` provides shape but no icon semantics.

**Steps:**
1. Create `cvkg-icons` crate (or module in `cvkg-components`):
   ```rust
   pub struct IconRegistry {
       icons: HashMap<String, IconData>,
   }
   
   pub enum IconData {
       Svg(String),     // SVG path data
       RgbaIcon(u32),   // Icon font glyph index
   }
   ```
2. Add `draw_icon(&mut self, name: &str, rect: Rect, color: [f32; 4])` to `Renderer` trait
3. Provide default icon set (20-30 common icons: close, settings, add, remove, etc.)
4. Icons render as `material_id=0` (opaque) with `draw_order=200` (above UI, below overlays)
5. Accessibility: icons must have `aria-label` — enforce at type level:
   ```rust
   fn draw_labeled_icon(&mut self, name: &str, label: &str, rect: Rect)
   ```

**Files:** New `cvkg-icons/src/lib.rs`, update `src/api.rs`

---

### P2-4: Visual Theme Picker (Basic CLI)

**Problem:** `Theme::from_seed()` has no UI for trying themes.

**Steps:**
1. Create `examples/theme_explorer.rs` demo:
   - Live preview of all components with current theme
   - Sliders for OKLCH seed (L, C, H)
   - Toggle: dark/light mode
   - Toggle: compact/default/spacious density
   - Display APCA contrast values in real-time
2. Use `cvkg-cli` for keyboard navigation
3. Screenshot output for documentation

**File:** `examples/theme_explorer.rs`

---

### P2-5: Design Steward Role Definition

**Problem:** No named design authority. Risk of token system fragmentation.

**Steps:**
1. Create `doc/DESIGN_STEWARD.md`:
   ```
   # CVKG Design Steward
   
   The Design Steward reviews all changes to:
   - cvkg-themes/src/lib.rs (Theme, GlassMaterial, tokens)
   - docs/ANIMATION_VOCABULARY.md
   - SpacingScale, RadiusScale, TypographyScale
   
   Approval required for:
   - New semantic color roles
   - Spacing/radius scale additions
   - Animation vocabulary changes
   - Density variants
   ```
2. Add `## Design-Steward-Review` label to PR template
3. Define escalation path: component author → steward → RFC

**File:** `doc/DESIGN_STEWARD.md`, `.github/PULL_REQUEST_TEMPLATE.md`

---

### P2-6: User Research Pipeline (Foundation)

**Problem:** No beta program, no user testing framework.

**Steps:**
1. Create `doc/USER_RESEARCH.md` defining:
   - Heuristic evaluation checklist (based on this review's gap list)
   - Contrast testing procedure (APCA validation + real-wallpaper testing)
   - Motion sensitivity testing (prefers-reduced-motion compliance)
   - Touch target audit (44×44 verification)
2. Add `#[test] fn heuristic_glass_legibility()` test:
   - Create glass rect over synthetic "busy" background (checkerboard, gradient, photo)
   - Read back pixel contrast
   - Assert APCA Lc >= 60 for text overlaid on glass
3. Recruit 3-5 beta testers from CVKG Discord/GitHub
4. Quarterly survey template (adapt Tech Edvocate's 65%/72%/80% metrics)

**File:** `doc/USER_RESEARCH.md`, new test in `cvkg-tests/`

---

### P2-7: Competitive Benchmarking (Foundation)

**Problem:** No documented comparison with SwiftUI, Jetpack Compose.

**Steps:**
1. Create `doc/BENCHMARKS.md` with:
   - Feature matrix: CVKG vs SwiftUI vs Jetpack Compose vs Flutter
   - Glass/blur implementation comparison
   - Accessibility feature comparison
   - Animation system comparison
2. Focus on glass-specific benchmarks:
   - Glass blur CVKG (Snell + GGX) vs SwiftUI (.glassEffect)
   - Backdrop sampling quality
   - Accessibility override granularity

**File:** `doc/BENCHMARKS.md`

---

## P3: Long-Term (Next Quarter)

### P3-1: Progressive Enhancement / CPU Fallback

**Problem:** Requires wgpu/WebGPU; no CPU rendering fallback.

**Steps:**
1. Define `SoftwareRenderer` trait matching `Renderer` subset
2. Implement CPU rasterizer for:
   - Opaque rectangles (trivial)
   - Rounded rectangles (analytical AA)
   - Blur (Kawase on CPU, limited radius)
   - NO glass refraction/software ray-tracing (degrade to solid)
3. Feature gate: `cvkg-software` crate
4. Priority: support for CI environments without GPU

**File:** New `cvkg-render-software/src/lib.rs`

---

### P3-2: Touch/Pressure API

**Problem:** "Tactile" limited to visual; no touch pressure input.

**Steps:**
1. Define `PointerEvent` with `pressure: f32` field in `cvkg-core`
2. Update `Renderer` trait to receive pointer events with pressure
3. In glass shader: scale refraction distortion by pressure
4. Stub on desktop (pressure = 1.0 for mouse, 0 for no input)
5. Implement on mobile targets (iOS/Android touch)

**Files:** `cvkg-core/src/lib.rs` (Event types), `cvkg-render-native/src/lib.rs` (platform input)

---

### P3-3: Spatial Computing Bridge

**Problem:** No Vision Pro / AR spatial computing support.

**Steps:**
1. Design `cvkg-spatial` crate architecture
2. Define `SpatialRenderer` trait extending `Renderer`
3. Pass-through rendering for Vision Pro stereoscopic
4. Portal rendering for floating UI panels
5. Defer to post-1.0

**File:** Future `cvkg-spatial/src/lib.rs`

---

### P3-4: Usability Telemetry

**Problem:** No metrics on contrast failures, motion reductions.

**Steps:**
1. Define `cvkg-telemetry` crate (opt-in, compile-time feature gate):
   ```rust
   #[cfg(feature = "telemetry")]
   pub struct Telemetry {
       pub accessibility: AccessibilityOverrides,
       pub frame_time_ms: f64,
       pub glass_element_count: u32,
   }
   ```
2. Log (not ship) when:
   - Text contrast fails APCA (Lc < 60)
   - User enables `reduce_transparency` (glass was too heavy)
   - User enables `reduce_motion` (animation was too much)
   - Frame time > 16ms (performance budget exceeded)
3. Local-only by default; opt-in anonymous upload

**File:** New `cvkg-telemetry/src/lib.rs`

---

## SVG App Maintainer Issues (8 Additional Fixes)

### SA-1: No Default Canvas Background
**→ Covered by P0-4 above**

### SA-2: render_frame() Required Before end_frame()
**Problem:** Apps must call `render_frame()` before `end_frame()` or nothing renders.

**Steps:**
1. In `end_frame()` (renderer.rs:3316), add early-return check:
   ```rust
   if !self.frame_rendered && !self.staging_belt.is_empty() {
       // Auto-flush staging if app forgot render_frame()
       self.submit_staging(&mut encoder);
   }
   ```
2. OR: Make `render_frame()` a no-op if already called (idempotent)
3. Update docs: `end_frame()` now auto-calls `render_frame()` if needed
4. Add test: skip `render_frame()`, call `end_frame()`, verify output

**File:** `cvkg-render-gpu/src/renderer.rs`

---

### SA-3: Cross-Crate Trait Vtable Dispatch Fails

**Problem:** `dyn Renderer` across crate boundaries causes linker errors.

**Root cause:** Rust vtable layout is unstable across compilation units.

**Steps:**
1. Replace `Box<dyn Renderer>` with concrete `SurtrRenderer` in all cross-crate usage
2. OR: Add `#[repr(C)]` + manual vtable (complex, defer)
3. Immediate fix: provide `fn renderer() -> &SurtrRenderer` accessor instead of trait object
4. Document in `docs/ARCHITECTURE.md`: "Never use `dyn Renderer` across crate boundaries"

**File:** `docs/ARCHITECTURE.md`

---

### SA-4: gungnir() Glow Too Strong

**Problem:** `gungnir()` creates 8 expanding glow layers with `intensity / (i + 1) * 0.3` — too bright.

**Steps:**
1. In `cvkg-render-gpu/src/api.rs` lines 193-216, reduce intensity:
   ```rust
   let alpha = intensity / (i as f32 + 1.0) * 0.08; // was 0.3
   ```
2. Reduce from 8 layers to 4:
   ```rust
   for i in 0..4 { // was 8
   ```
3. Add `gungnir_soft()` variant with half the intensity
4. Document: use `gungnir()` for critical alerts, `gungnir_soft()` for hover highlights

**File:** `cvkg-render-gpu/src/api.rs`

---

### SA-5: Icon Registry Must Thread via UiParams

**Problem:** Icon registry contributes to `UiParams` god object.

**→ Covered by P2-3 (Icon System)** — the new system eliminates the need to thread icons through UiParams.

---

### SA-6: Resize Events Can Be 0/Oversize
**→ Covered by P1-4 above**

---

### SA-7: set_scene("void") is NO-OP

**Problem:** String-based scene lookup doesn't match the constant name.

**Steps:**
1. In `cvkg-scene/src/lib.rs`, add string-based lookup that normalizes:
   ```rust
   pub fn set_scene_by_name(&mut self, name: &str) {
       let normalized = name.to_lowercase().replace(['-', '_', ' '], "");
       match normalized.as_str() {
           "void" | "empty" | "none" => self.set_scene_preset(SCENE_VOID),
           // ... other presets
           _ => log::warn!("Unknown scene: {}", name),
       }
   }
   ```
2. Keep existing `set_scene_preset()` as canonical API
3. Update docs: string lookup is convenience; use constants for production

**File:** `cvkg-scene/src/lib.rs`

---

### SA-8: ThemeBuilder May Not Expose All Semantic Roles

**Problem:** Builder pattern doesn't expose all `SemanticColors` fields.

**Steps:**
1. Audit `Theme::dark()` vs `Theme::from_seed()` — count semantic roles
2. All 10 semantic roles (primary, secondary, accent, background, surface, error, warning, success, text, text_dim) should be settable via builder
3. Add `ThemeBuilder` with chainable setters:
   ```rust
   let theme = ThemeBuilder::dark()
       .with_error(Color::new(0.8, 0.2, 0.2, 1.0))
       .with_surface(Color::new(0.1, 0.1, 0.12, 1.0))
       .build();
   ```
4. `build()` validates APCA on all pairs

**File:** `cvkg-themes/src/lib.rs`

---

## Implementation Order

```
Week 1 (P0):
  Day 1-2: P0-1 (fix glass pipeline tests) + P0-2 (CI accessibility gate)
  Day 3:   P0-3 (SVG render order) + P0-4 (default canvas bg)
  Day 4:   SA-2 (render_frame auto-flush) + SA-4 (gungnir glow reduce)
  Day 5:   SA-7 (set_scene_by_name) + SA-8 (ThemeBuilder)

Week 2 (P1):
  Day 6-7: P1-1 (per-component glass intensity)
  Day 8:   P1-2 (improved backdrop sampling)
  Day 9:   P1-3 (reduce_motion component audit)
  Day 10:  P1-4 (resize clamping)

Week 3 (P2):
  Day 11-12: P2-1 (animation vocabulary) + P2-5 (design steward doc)
  Day 13-14: P2-2 (density variants)
  Day 15:    P2-4 (theme explorer demo)

Week 4 (THEME-1 through THEME-6):
  Day 16:   THEME-1 (audit + document all hardcoded colors)
  Day 17:   THEME-2 (fix primitive.rs + breadcrumb.rs)
  Day 18:   THEME-3 (fix interactive/ components)
  Day 19:   THEME-4 (fix chrome/ + HUD + inspector components)
  Day 20:   THEME-5 (fix remaining: ai_workflow, multi_agent, collaboration, multimedia, etc.)
  Day 21:   THEME-6 (theme token validation test + docs)

Week 5 (P2/P3 + integration):
  Day 22-23: P2-3 (icon system) + P2-6/7 (research + benchmarks)
  Day 24-25: P3 items (progressive enhancement, touch API, spatial, telemetry design)
  Day 26-27: Full workspace test, version bump to 0.2.13
```

---

## THEME: Make All CVKG-Components Themeable by Default

**Problem:** Many components use hardcoded colors (`Color::WHITE`, `Color::BLACK`, raw `[f32; 4]` arrays) instead of resolving from the theme system. Components don't adapt when the user changes themes (dark ↔ light, custom seed, high-contrast mode).

**Current state:** `cvkg-components/src/theme.rs` provides ~50 semantic token helpers via `StyleResolver::color_array()`. Some components (Button, Input, Checkbox, Badge variant colors) already use them. But many others use hardcoded values.

**Hardcoded color locations found (50+ instances across 12 files):**

| File | Hardcoded Colors | Count |
|------|-----------------|-------|
| `primitive.rs` | `Color::WHITE`, `Color::BLACK` (Text, Divider, Shape defaults) | 3 |
| `breadcrumb.rs` | `Color::GRAY`, `Color::WHITE` | 2 |
| `interactive/select.rs` | `Color::BLACK`, `Color::WHITE`, `Color::RED`, `Color::CYAN` | 4 |
| `qrcode.rs` | `[0.0, 0.0, 0.0, 1.0]` (black cells) | 2 |
| `editable.rs` | `[0.0, 0.0, 0.0, 0.0]` (transparent) | 1 |
| `collaboration.rs` | 4 status colors (away, online, etc.) | 4 |
| `multimedia.rs` | 2 dark backgrounds | 2 |
| `asset_browser.rs` | 1 selection highlight | 1 |
| `gullveig_inspector.rs` | 4 colors (bg, border, accent, warning) | 4 |
| `freyr_inspector.rs` | 2 colors (bg, prop_bg) | 2 |
| `ai_workflow_builder.rs` | ~25 colors (status, type, token, node colors) | ~25 |
| `multi_agent_orchestrator.rs` | ~12 colors (bg, grid, stroke, status) | ~12 |

**Goal:** Zero hardcoded `[f32; 4]` color arrays in component `render()` methods. All colors resolve through `theme::color("token_name")`.

**New theme tokens needed:**

```rust
// Add to cvkg-components/src/theme.rs:

/// Workflow node status colors
pub fn status_running() -> [f32; 4]    { color("accent") }
pub fn status_completed() -> [f32; 4]  { color("success") }
pub fn status_failed() -> [f32; 4]     { color("error") }
pub fn status_waiting() -> [f32; 4]     { color("text_muted") }

/// Inspector/debug colors
pub fn inspector_bg() -> [f32; 4]       { color("surface") }
pub fn inspector_border() -> [f32; 4]   { color("border") }
pub fn inspector_accent() -> [f32; 4]   { color("accent") }
pub fn inspector_warning() -> [f32; 4]  { color("warning") }

/// Collaboration status
pub fn collab_online() -> [f32; 4]     { color("success") }
pub fn collab_away() -> [f32; 4]       { color("warning") }
pub fn collab_offline() -> [f32; 4]    { color("text_muted") }

/// Grid/editor backgrounds
pub fn editor_bg() -> [f32; 4]         { color("surface") }
pub fn editor_grid() -> [f32; 4]       { color("border") }

/// QR/decode colors
pub fn qr_dark() -> [f32; 4]           { color("text") }
pub fn qr_light() -> [f32; 4]          { color("background") }
```

**Theming rules (to be enforced):**
1. Every `render()` method must use `theme::color("token")` or `StyleResolver::color_array("token")` for ALL colors
2. NO hardcoded `[f32; 4]` arrays in render paths (except transparent `[0,0,0,0]`)
3. NO `Color::*` constants (WHITE, BLACK, RED, etc.) in render paths
4. Default component colors use semantic tokens (e.g., `theme::text()` not `Color::WHITE`)
5. Component-specific status colors use dedicated tokens (e.g., `theme::status_running()` not raw green)
6. Builder `.color()` overrides are the ONLY exception — user-provided colors bypass the theme

---

### THEME-1: Audit + Document (Day 16)

**Steps:**
1. Create `doc/THEMING_AUDIT.md` listing every file + line + hardcoded color
2. Categorize: (a) already themed, (b) easy fix, (c) complex fix
3. Define new theme tokens needed (list above)
4. Get user approval on token naming

**File:** `doc/THEMING_AUDIT.md`

---

### THEME-2: Primitive Components (Day 17)

**File:** `cvkg-components/src/primitive.rs`

Changes:
- `Text::new()`: default `color: Color::WHITE` → `color: theme::text()[0..4]`
- `Divider::horizontal()`: default `color: Color::BLACK` → `color: theme::border()[0..4]`
- `Shape::rounded_rect()`: default `fill: Color::BLACK` → `fill: theme::surface_elevated()[0..4]`
- `Skeleton::render()`: hardcoded shimmer → use `theme::skeleton_base()` and `theme::skeleton_highlight()`
- `BadgeVariant::colors()`: already themed — add `Info` variant using `theme::info()`

**File:** `cvkg-components/src/breadcrumb.rs`
- `Color::GRAY` → `theme::text_muted()`
- `Color::WHITE` → `theme::text()`

---

### THEME-3: Interactive Components (Day 18)

**File:** `cvkg-components/src/interactive/select.rs`
- 4 hardcoded colors → theme tokens:
  - `BLACK`/`WHITE` → `theme::text()` / `theme::background()`
  - `RED` → `theme::error_color()`
  - `CYAN` → `theme::accent()`

**File:** `cvkg-components/src/interactive/button.rs`
- Audit all `Color::*` usage → replace with `theme::button_primary_bg()`, etc.

**File:** `cvkg-components/src/interactive/input.rs`
- Hardcoded border/focus colors → `theme::input_border_focus()`, etc.

**File:** `cvkg-components/src/interactive/checkbox.rs`, `toggle.rs`, `slider.rs`
- Hardcoded check/active colors → `theme::checkbox_checked()`, `theme::toggle_active()`, etc.

**Files:** `textarea.rs`, `select.rs` — same pattern

---

### THEME-4: Chrome + HUD + Inspector Components (Day 19)

**Files:** `chrome/valkyrie_toolbar.rs`, `chrome/niflheim_sidebar.rs`, `chrome/niflheim_tab_bar.rs`
- All hardcoded dark grays → `theme::surface()`, `theme::surface_elevated()`, `theme::border()`

**File:** `hud.rs`
- Gauge/alert colors → `theme::accent()`, `theme::error_color()`, `theme::warning()`

**Files:** `gullveig_inspector.rs`, `freyr_inspector.rs`
- 4-6 hardcoded colors each → `theme::inspector_bg()`, `theme::inspector_border()`, `theme::inspector_accent()`, `theme::inspector_warning()`

---

### THEME-5: Remaining Components (Day 20)

**File:** `ai_workflow_builder.rs` (~25 hardcoded colors)
- Status colors → `theme::status_running()`, `theme::status_completed()`, `theme::status_failed()`, `theme::status_waiting()`
- Node type colors → `theme::accent()`, `theme::info()`, `theme::warning()`, create `theme::node_concept()`, `theme::node_entity()`, etc.
- Token type colors → `theme::text()`, `theme::text_muted()`, `theme::accent()`

**File:** `multi_agent_orchestrator.rs` (~12 hardcoded colors)
- Backgrounds → `theme::editor_bg()`, `theme::surface_elevated()`
- Grid → `theme::editor_grid()`
- Strokes → `theme::border_strong()`

**File:** `collaboration.rs` (4 status colors)
- → `theme::collab_online()`, `theme::collab_away()`, `theme::collab_offline()`

**File:** `multimedia.rs` (2 dark backgrounds)
- → `theme::surface()`

**File:** `asset_browser.rs` (1 selection highlight)
- → `theme::hover()` or `theme::list_item_selected()`

**File:** `qrcode.rs` (2 hardcoded blacks)
- → `theme::qr_dark()`, `theme::qr_light()`

**File:** `editable.rs` (transparent → keep as-is, transparent is not a color choice)

---

### THEME-6: Validation Test + Docs (Day 21)

**Test 1: No hardcoded colors in render paths**
```rust
// cvkg-components/tests/theming_test.rs
#[test]
fn all_components_use_theme_tokens() {
    // For each component, create it with default settings, render into a test renderer,
    // and verify that all fill_rect/stroke_rect calls used colors that came from
    // StyleResolver (theme tokens), not hardcoded arrays.
    // This is a structural test — we check that component render() methods don't
    // contain any fill_rect calls with literal [f32; 4] arrays.
}
```

**Test 2: Theme switching changes colors**
```rust
#[test]
fn theme_switch_changes_component_colors() {
    let dark = Theme::dark();
    let light = Theme::light();
    
    // Render Button in both themes, verify pixel output differs
    // Render Text in both themes, verify pixel output differs
    // This catches components that hardcode colors and ignore the theme
}
```

**Test 3: High contrast mode**
```rust
#[test]
fn high_contrast_increases_contrast() {
    let mut overrides = AccessibilityOverrides::default();
    overrides.increase_contrast = true;
    
    let normal_theme = Theme { accessibility: overrides.clone(), ..Theme::dark() };
    // Verify that with increase_contrast, text-background APCA Lc is higher
}
```

**Docs:** Update `docs/ARCHITECTURE.md` with theming rules:
- All colors through theme tokens
- List of all semantic token names
- How to add a new token
- How to add a dark/light variant

---

## Summary: What "Themeable by Default" Means After This Plan

| Before | After |
|--------|-------|
| `Text::new("hello")` renders white on any background | `Text::new("hello")` renders `theme::text()` — adapts to dark/light/custom |
| `Divider::horizontal()` renders black | Renders `theme::border()` — adapts |
| `Button` hardcoded accent | Uses `theme::button_primary_bg()` |
| `ai_workflow_builder.rs` has 25 magic numbers | All map to semantic status/type tokens |
| Inspector panels have fixed dark grays | Use `theme::inspector_*()` tokens |
| Status colors scattered across files | Centralized in `theme::status_*()` and `theme::collab_*()` |
| QR code always black/white | Uses `theme::qr_dark()` / `theme::qr_light()` |
| No validation | `theming_test.rs` catches regressions |
| No documentation | `doc/THEMING_AUDIT.md` + updated `ARCHITECTURE.md` |

**Components already themed (no work needed):** Button, Input, Checkbox, Select, Slider, Toggle, Stepper, Badge (variants), NjordTheme

**Components needing fixes:** Text, Divider, Shape, Skeleton, Breadcrumb, QRCode, Editable, Collaboration, Multimedia, AssetBrowser, GullveigInspector, FreyrInspector, AIWorkflowBuilder, MultiAgentOrchestrator, all chrome components

**New theme tokens needed:** 15-20 new tokens for workflow status, inspector, collaboration, grid, QR

**New tests:** 3 test functions in `cvkg-components/tests/theming_test.rs`
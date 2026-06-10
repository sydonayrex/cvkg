# CVKG Opus
## A Complete Vision for the World's Most Sophisticated Native UI Framework
### Implementation Plan — 2025–2028

---

> *"The best interface is the one that disappears. The second best is one so beautiful users choose not to look away."*

---

## Preamble: What We Are Building

CVKG is not competing with Tauri, Iced, or egui. It is not a "Rust alternative to Electron." CVKG is the only native UI framework that treats the screen as a physical material — glass, metal, ice, fire — rather than a canvas of rectangles. The audit confirms an exceptional foundation: production-quality Kawase blur, physically-grounded SleipnirSolver RK4 spring physics, a true multi-pass Kvasir render graph, and a Norse naming system that gives the codebase a coherent mythological identity that no other framework has.

The gaps identified are not failures. They are the precise delta between "technically impressive" and "visually undeniable." This document closes that delta.

The target: by 2028, CVKG applications should be visually indistinguishable from the best macOS Tahoe apps, while being buildable from Rust, deployable cross-platform, and extensible via a first-class theme and component system that no other framework in any language offers.

This plan is organized into six phases. Each phase is independently shippable. Each phase leaves the framework better than it found it.

---

## Architectural Philosophy

Before any implementation, three principles govern every decision:

**1. The Material Illusion Principle**
Every UI element is a piece of matter with physical properties: refractive index, surface roughness, ambient occlusion, thermal emission. The renderer does not "draw shapes with effects." It simulates light interacting with materials. This distinction changes every API decision.

**2. The Surgical Rule (from CVKG Guidelines §1.3)**
Touch only what is required. Every gap in the audit has a minimal, targeted fix. We do not rebuild what works. The Kawase blur pyramid is excellent — we extend it. The SDF edge rendering is correct — we add convolution on top of it. The Norse naming is distinctive — we deepen it, never dilute it.

**3. The 2028 Standard**
The question is not "does this match macOS Tahoe 2025?" The question is "will this still look exceptional in 2028?" The answer requires: physically-accurate rendering, semantic adaptivity (UI that responds to content, not just pointer position), and a component system designed for the display densities and interaction modalities that will be common in three years — high-DPI touchscreen MacBooks, spatial computing overlays, and voice-augmented workflows.

---

## Current State Assessment (Honest)

| Area | State | Quality |
|------|-------|---------|
| Kawase blur pyramid | Implemented | Production-quality |
| Glass shader (material_glass.wgsl) | Partial | Good but not adaptive |
| Kvasir render graph | Implemented | Sophisticated, production |
| SleipnirSolver RK4 | Implemented | Mathematically correct |
| BifrostModifier | Partial | Disconnected from GPU tint |
| OKLCH theme engine | Implemented | CPU-only, not GPU-wired |
| Component library | Partial | 38+ components, missing chrome |
| MenuBar | Missing | Data model only in cvkg-core |
| Dock | Missing | No component exists |
| Toolbar/TitleBar | Missing | YggdrasilWindow stub only |
| Segmented Control | Missing | — |
| Glass buttons | Missing | No variants |
| Context menu | Missing | RadialMenu only |
| Sidebar chrome | Missing | Flat surface() only |
| SearchBar | Missing | MimirSpotlight has input |
| Berserker preset | Missing | Appears in demo names only |
| Runic ornament system | Missing | — |
| Ambient particles | Missing | — |
| Audio-reactive effects | Missing | AudioEngine trait exists |
| Per-element backdrop blur | Missing | Full-scene only |
| Edge-smear convolution | Missing | — |
| Parallax depth | Missing | — |
| IOR-accurate glass | Missing | Ad-hoc math |

---

## Phase 0 — Architectural Debt Clearance
### Timeline: 1 week | Risk: Low | Impact: Foundational

Before any new features, three existing disconnections must be wired. These are not new work — the pieces exist. The wires are missing.

### 0.1 — Wire OKLCH → GPU

**The Gap:** `Theme::from_seed()` generates OKLCH palettes on CPU. `GlassMaterial.tint_color` exists in `cvkg-themes`. `ColorTheme` in `cvkg-core` has `glass_base: [f32; 4]`. They are never connected.

**The Fix:** Add a single conversion function in `cvkg-themes/src/lib.rs`:

```rust
/// Convert a GlassMaterial's tint into ColorTheme GPU uniforms.
/// Called once per frame when theme changes, or on theme toggle.
pub fn glass_material_to_color_theme_patch(mat: &GlassMaterial) -> GlassThemePatch {
    let tint_rgba = mat.tint_color.to_rgba();
    GlassThemePatch {
        glass_base: [tint_rgba.r, tint_rgba.g, tint_rgba.b, mat.tint_opacity],
        glass_blur_strength: mat.backdrop_blur_radius / 40.0, // normalize to [0,1]
        refraction_index: mat.refraction_index,
    }
}
```

Add `GlassThemePatch` as a sub-struct that `SurtrRenderer::set_theme()` accepts and merges into the active `ColorTheme` uniform. This is a 40-line change. It unblocks every subsequent glass improvement.

### 0.2 — Wire bcs_frosted into EffectRegistry

**The Gap:** `bcs_frosted` in `effects.wgsl` is fully implemented but has no `EffectId` variant and is unreachable via the public API.

**The Fix:** Add `EffectId::BcsFrosted` to the enum. Add the dispatch branch in `EffectRegistry::dispatch()`. Add `Renderer::bcs_frosted(rect, intensity, clear_radius)` to the trait with a default no-op. Wire through `SurtrRenderer`. This is a 25-line change and makes the existing implementation callable.

### 0.3 — Add BackdropRegion to Kvasir

**The Gap:** `BackdropCopyNode` captures the full scene. Per-element blur requires per-element capture.

**The Fix:** Add `BackdropRegionNode` that accepts a `Rect` scissor parameter. Internally it copies only that region from the scene texture, runs two Kawase passes at half resolution, and outputs a texture handle the glass pass can sample. The full-scene `BackdropCopyNode` is preserved — it is still used for the main backdrop. `BackdropRegionNode` is additive.

This requires:
- New `PassId::BackdropRegion(u32)` variant (indexed per-element)
- Region-scissored copy compute pass (16 lines of WGSL)
- A `BlurHandle` type that the glass pipeline binds instead of `t_env`

Estimated: 120 lines across `nodes.rs`, `graph.rs`, and `blur_pyramid.wgsl`.

---

## Phase 1 — Physically Accurate Glass
### Timeline: 2 weeks | Risk: Medium | Impact: Transformative

This is the single most impactful phase. The current glass shader produces a competent frosted-glass approximation. This phase makes it the best glass renderer outside of a game engine.

### 1.1 — Snell's Law Refraction

The current distortion model in `material_glass.wgsl` uses:
```wgsl
let lens = lens_dir * lens_dist * 0.08 * variation;
```

This is directionally correct but physically wrong. Replace with Snell's law refraction:

```wgsl
/// Physically accurate refraction using Snell's law.
/// n1 = 1.0 (air), n2 = IOR parameter from per-instance uniforms.
/// Returns the UV offset for the refracted sample direction.
fn snell_refraction(normal: vec2<f32>, incident: vec2<f32>, ior: f32) -> vec2<f32> {
    let n_ratio = 1.0 / ior;
    let cos_i = -dot(normal, incident);
    let sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
    
    // Total internal reflection
    if sin2_t > 1.0 {
        return reflect(incident, normal);
    }
    
    let cos_t = sqrt(1.0 - sin2_t);
    return n_ratio * incident + (n_ratio * cos_i - cos_t) * normal;
}
```

Add `ior: f32` to the per-instance glass uniform block. Default: 1.45 (borosilicate glass). Berserker mode: 1.85 (dense flint, more dramatic distortion). Midgard mode: 1.0 (no refraction, flat surface).

The visual difference is immediate and dramatic: objects behind glass panels will shift and bend as the viewer's angle changes, which is exactly the behavior Tahoe's liquid glass achieves.

### 1.2 — Adaptive Tint from Backdrop Luminance

Tahoe glass does not have a fixed tint. It samples the average luminance of the backdrop and adjusts its tint toward the dominant color. This creates the "liquid" effect where the glass changes character based on what's behind it.

Implementation: In the glass fragment shader, compute a 4-tap luminance sample of the blurred backdrop at coarse offsets. Derive a tint vector from those samples. Blend with the static `theme.glass_base` tint using a `theme.glass_tint_adapt` weight factor:

```wgsl
// Sample backdrop at 4 coarse positions for dominant color
let s0 = textureSampleLevel(t_env, s_env, uv + vec2(-0.1, -0.1), 6.0).rgb;
let s1 = textureSampleLevel(t_env, s_env, uv + vec2( 0.1, -0.1), 6.0).rgb;
let s2 = textureSampleLevel(t_env, s_env, uv + vec2(-0.1,  0.1), 6.0).rgb;
let s3 = textureSampleLevel(t_env, s_env, uv + vec2( 0.1,  0.1), 6.0).rgb;
let backdrop_dominant = (s0 + s1 + s2 + s3) * 0.25;

// Adaptive tint: mix static theme tint with backdrop-derived tint
let adaptive_tint = mix(theme.glass_base.rgb, backdrop_dominant * 0.3 + 0.7, 
                        theme.glass_tint_adapt);
```

Add `glass_tint_adapt: f32` to `ColorTheme`. Default: 0.35 (subtle, like Tahoe). Vibrant Glass: 0.65 (strong adaptation). Midgard: 0.0 (static).

### 1.3 — Edge Smear Convolution

The characteristic Tahoe glass edge is not a sharp border with a rim light. It is a soft smear where the blurred backdrop bleeds slightly beyond the glass boundary, then snaps to a bright specular edge. This is a two-step effect:

**Step 1 — Smear pass:** A separable 1D convolution along the glass SDF gradient, extending 3px beyond the boundary. Weight: Gaussian envelope with σ=1.5px.

**Step 2 — Crystalline edge:** The SDF distance within 0.5px of the boundary receives an additive bright white contribution scaled by the surface normal's angle to the light source.

```wgsl
// Smear: extend blur slightly beyond the glass edge
let smear_dist = clamp(-d_sdf, 0.0, 3.0) / 3.0;
let smear_contribution = textureSampleLevel(t_env, s_env, 
    uv + lens_dir * smear_dist * 0.01, blur_mip).rgb * 0.15;

// Crystalline edge highlight
let edge_mask = smoothstep(0.5, 0.0, abs(d_sdf));
let crystal_edge = edge_mask * 0.4 * (0.7 + 0.3 * spec);

final_rgb += smear_contribution + crystal_edge;
```

### 1.4 — Sub-Surface Scattering Approximation

Tahoe glass is not uniform — it appears thicker at the edges (darker, more refractive) and thinner at the center (lighter, more transparent). This is sub-surface scattering.

Approximate with a thickness map derived from the SDF:

```wgsl
// Thickness: SDF distance from edge, normalized
// Negative SDF = inside glass. Deeper inside = thinner center.
let thickness = 1.0 - clamp(-d_sdf / (in.size.x * 0.5), 0.0, 1.0);
let sss_tint = mix(vec3(0.92, 0.96, 1.0), vec3(0.7, 0.8, 0.95), thickness);
final_rgb *= sss_tint;

// Alpha model: thicker at edges (more opaque), thinner at center
let sss_alpha = mix(0.06, 0.22, thickness);
let final_alpha = (sss_alpha + fresnel * 0.18) * in.color.a * clip_alpha;
```

### 1.5 — Per-Instance Tint Uniforms

All glass surfaces currently share `ColorTheme`. Tahoe panels have different tints (sidebar is slightly warmer, popovers are cooler, sheets are near-neutral). 

Add a `GlassInstanceUniforms` push-constant block:

```rust
/// Per-draw-call glass instance parameters.
/// Passed as push constants (fast path, no buffer allocation).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlassInstanceUniforms {
    /// Local tint override: [r, g, b, weight].
    /// weight=0 = use theme tint only, weight=1 = use local tint only.
    pub tint_override: [f32; 4],
    /// Per-instance IOR override. 0.0 = use theme default.
    pub ior_override: f32,
    /// Blur strength multiplier. 1.0 = normal, 2.0 = double blur.
    pub blur_multiplier: f32,
    /// Frost intensity override. 0.0 = theme default.
    pub frost_override: f32,
    pub _pad: f32,
}
```

Expose via `Renderer::set_glass_instance(uniforms: GlassInstanceUniforms)` before glass draw calls. The `BifrostModifier` gains `tint_mode`, `fresnel_strength`, and `backdrop_sample_radius` fields, completing the audit gap #19.

---

## Phase 2 — Desktop Chrome Components
### Timeline: 2 weeks | Risk: Low | Impact: Essential

These are the components every macOS-class desktop app requires. Without them, CVKG apps cannot produce a professional application shell. They are built using the glass foundation established in Phase 1.

### 2.1 — GlassMenuBar (NornirBar)

Named after the Nornir (Urd, Verdandi, Skuld) — the three fates who weave destiny, an appropriate name for a horizontal navigation that controls the app's destiny.

**Structure:**

```rust
/// The application menu bar. Renders at the top of the window with
/// glass background, horizontal menu items, and cascading submenus.
///
/// CONTRACT: NornirBar owns the top 28pt of the window's content area.
/// It is always present; hiding it causes layout to reflow upward.
pub struct NornirBar {
    /// The menu items to display left-to-right.
    pub menus: Vec<NornirMenu>,
    /// Glass material for the bar background.
    pub material: GlassMaterial,
    /// Whether the bar floats over content (unified titlebar/toolbar style).
    pub floating: bool,
    /// App icon displayed at leading edge (optional).
    pub app_icon: Option<String>,
    /// Controls rendered at trailing edge (e.g., spotlight, notifications).
    pub trailing_controls: Vec<NornirBarControl>,
}
```

**Rendering:** The bar background uses `Material::Glass` with `GlassInstanceUniforms::tint_override` set to a slightly warm neutral, `ior_override = 1.3`. Submenus appear as floating glass panels with `blur_multiplier = 1.8` (heavier blur for deeper layering).

**Keyboard navigation:** Full arrow-key traversal. `Escape` closes open submenus. `Tab` moves to the next top-level menu. Every `MenuItem::Action` with a `KeyboardShortcut` renders the shortcut in a trailing label using the system monospace font at 11pt, opacity 0.6.

**Submenu positioning:** Uses a `PhaseGate` (portal) to render into the overlay layer, preventing clipping by parent scroll containers. Submenus cascade to the right with a 4pt horizontal overlap; on the trailing edge of the screen they cascade left instead (collision detection via `Renderer::viewport_size()`).

### 2.2 — HeimdallDock

Named after Heimdall, guardian of the Bifrost bridge, who watches all approaches — an apt name for the dock that monitors running applications.

**The magnification algorithm** is the part most frameworks get wrong. The correct implementation is not a simple scale-based-on-distance. It is a smooth Gaussian envelope:

```rust
/// Compute magnified size for a dock item based on pointer proximity.
/// Uses a Gaussian envelope centered on the pointer with σ = 80px.
/// Maximum magnification: 2.0× at zero distance.
/// This matches macOS's parabolic magnification curve.
pub fn dock_item_magnification(
    item_center: f32,    // item's current center x
    pointer_x: f32,     // pointer x in dock coordinates  
    base_size: f32,     // base item size (default: 48pt)
    max_scale: f32,     // maximum scale factor (default: 2.0)
) -> f32 {
    let sigma = 80.0_f32;
    let dist = (item_center - pointer_x).abs();
    let gaussian = (-dist * dist / (2.0 * sigma * sigma)).exp();
    1.0 + (max_scale - 1.0) * gaussian
}
```

The dock layout engine runs this for every item on every pointer-move event and applies the results via `Renderer::push_transform()`. Neighbor items expand symmetrically, pushing items outward from the pointer. This is computed in O(n) per frame and is trivially fast.

**Structure:**

```rust
pub struct HeimdallDock {
    /// Items in the dock, left-to-right.
    pub items: Vec<DockItem>,
    /// Dock position: bottom, left, or right.
    pub position: DockPosition,
    /// Whether the dock auto-hides when the pointer leaves.
    pub auto_hide: bool,
    /// Current auto-hide state (shown/hidden/animating).
    pub hide_state: DockHideState,
    /// Magnification maximum scale factor.
    pub magnification: f32,
    /// Glass platter material.
    pub material: GlassMaterial,
}

pub struct DockItem {
    pub id: String,
    pub icon: String,     // asset name
    pub label: String,
    pub badge: Option<u32>,
    pub is_running: bool,  // shows the running indicator dot
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    pub on_right_click: Option<Arc<dyn Fn() + Send + Sync>>,
}
```

**Auto-hide:** Uses a `SleipnirSolver` with `SleipnirParams::snappy()` to animate the dock off-screen. The trigger is a `PointerLeave` event on the dock's safe-area region. Re-reveal uses a screen-edge hit zone (the bottom 2px of the window) that triggers a `SleipnirSolver` spring toward the visible position.

**Running indicator:** A 4px dot centered below the icon, colored with `theme.primary_neon`, with a `GungnirModifier { radius: 3.0, intensity: 0.8 }`.

**Bounce animation:** On app launch, the dock item bounces using a pre-computed Bezier path keyframe sequence in `cvkg-anim::Animation::Hybrid`. Three keyframes: +40px up (ease-out, 0.15s), back to 0 (overshoot -8px, ease-in-out, 0.2s), settle (spring). Total duration: ~0.5s.

### 2.3 — ValkyrieToolbar

Named after the Valkyries — choosers of the slain, selectors of what matters — the toolbar selects and presents the most important actions.

Tahoe's "unified toolbar" is a floating rounded rectangle that appears to hover between the titlebar and content area. It has no rigid slot layout; items are positioned by a flex-like algorithm with priority rules.

```rust
pub struct ValkyrieToolbar {
    /// Leading items (left-aligned).
    pub leading: Vec<ToolbarItem>,
    /// Center items (displayed centered, pushed aside by leading/trailing).
    pub center: Vec<ToolbarItem>,
    /// Trailing items (right-aligned).
    pub trailing: Vec<ToolbarItem>,
    /// Overflow behavior when items don't fit.
    pub overflow: ToolbarOverflow,
    /// Glass material.
    pub material: GlassMaterial,
    /// Corner radius of the toolbar platter.
    pub radius: f32,
}

pub enum ToolbarItem {
    Button(ToolbarButton),
    SegmentedControl(HrungnirSegment),
    SearchField(MimirSearchBar),
    Spacer,
    FlexSpace,
    Separator,
    Custom(AnyView),
}
```

The toolbar does not use a fixed height. It sizes to its content plus `12pt` vertical padding. It floats with a `4pt` gap below the title bar area. Its glass platter has a distinct `blur_multiplier = 1.4` to separate it visually from both the title bar above and content below.

**Traffic light restyling:** The macOS close/minimize/zoom buttons (traffic lights) are restyled when `floating = true`: they appear as small glass capsules with the standard red/yellow/green fills, but with a subtle neon edge glow in theme color on hover. This is achieved by intercepting the platform's window chrome via `NativeWindowWrapper` and rendering custom controls in the CVKG layer, positioned via `Renderer::viewport_size()`.

### 2.4 — HrungnirSegmented Control

Named after the giant Hrungnir, whose heart was made of stone with three sharp corners — a perfect shape for a segmented control.

```rust
/// A horizontal group of mutually exclusive options with glass styling.
/// This is the primary building block for toolbars, format bars, and filter bars.
pub struct HrungnirSegmented {
    pub segments: Vec<Segment>,
    pub selected: usize,
    pub on_select: Arc<dyn Fn(usize) + Send + Sync>,
    pub style: SegmentedStyle,
}

pub enum SegmentedStyle {
    /// Glass platter with sliding pill indicator.
    Glass,
    /// Capsule buttons that are independent but visually grouped.
    Capsule,
    /// Icon-only segments (minimum width).
    Iconic,
    /// Text labels, flexible width.
    Labeled,
}
```

The sliding pill indicator is the signature interaction: when the selection changes, a `SleipnirSolver { params: SleipnirParams::snappy() }` animates the pill's X position and width from the old segment to the new one. This is the correct way to implement the effect — not a fade transition, not a jump, but a physical spring that gives the UI weight and personality.

The pill itself uses `Material::Glass` with `tint_override = [1.0, 1.0, 1.0, 0.15]` (white tint at low opacity, creating a frosted-highlight effect over the glass platter background).

### 2.5 — Glass Button Variants

The audit correctly identifies that `interactive.rs` buttons lack the Tahoe treatment. Add three new variants to the existing `ButtonStyle` enum:

```rust
pub enum ButtonStyle {
    // Existing variants preserved
    Default,
    Destructive,
    // New variants
    
    /// Glass button: frosted background, no border, subtle backdrop.
    /// Used for secondary actions in toolbars and panels.
    Glass,
    
    /// Tinted glass: glass base with accent color tint.
    /// Used for primary calls-to-action.
    /// color: the accent tint in OKLCH.
    TintedGlass { color: OklchColor },
    
    /// Capsule button: pill-shaped, solid fill, high contrast.
    /// Used for confirmation actions (e.g., "Apply", "Done").
    Capsule { fill: [f32; 4] },
    
    /// Icon button: square glass platter, single icon, no label.
    /// Used in toolbars where space is constrained.
    Icon,
}
```

Each variant has three states rendered via the existing hover/active/disabled infrastructure, with `SleipnirSolver`-driven press animations (scale to 0.96 on press, spring back on release).

The `TintedGlass` variant is particularly important: it is the "primary action" button throughout Tahoe — the blue "Send," green "Accept," etc. Implementation applies `GlassInstanceUniforms::tint_override` with the color's RGBA components and `weight = 0.7` so the glass character is preserved while the tint is visible.

---

## Phase 3 — Panel and Navigation Chrome
### Timeline: 1 week | Risk: Low | Impact: High

### 3.1 — NiflheimSidebar

Named after Niflheim, the realm of ice and mist — sidebars are the frozen, translucent walls of the application.

The current `NavigationSplitView` provides the layout. This phase adds the glass chrome layer:

```rust
/// Glass chrome wrapper for sidebar panels.
/// Applied via `.sidebar_chrome(style)` modifier on any view.
pub struct NiflheimSidebar {
    /// Background glass material (heavier blur than main content).
    pub material: GlassMaterial,
    /// Whether source-list row highlights use translucent fills.
    pub source_list_style: bool,
    /// Edge highlight line on the trailing edge (separator).
    pub separator: SidebarSeparator,
    /// Vibrancy level (affects blur strength and tint weight).
    pub vibrancy: SidebarVibrancy,
}

pub enum SidebarVibrancy {
    /// Subtle frosting, content shows through.
    Translucent,
    /// Standard macOS sidebar vibrancy.
    Standard,
    /// Full vibrancy: heavy blur, strong tint, minimal content bleed.
    Heavy,
}
```

The separator is a 0.5px vertical line on the trailing edge, colored with `theme.border` at 60% opacity, with a 1px inner glow at `theme.glass_edge` at 30% opacity. This replicates the exact Tahoe sidebar edge treatment.

Source-list rows use `fill_rounded_rect` with a `[1.0, 1.0, 1.0, 0.08]` fill for hover, `[1.0, 1.0, 1.0, 0.14]` for selected state on glass backgrounds. On opaque backgrounds (Midgard mode) these revert to the standard theme surface colors.

### 3.2 — RuneInspector (Floating Panel)

Named after the runic tablets used by Norse scholars to record and inspect knowledge.

```rust
/// A detachable floating inspector panel with glass background.
/// Supports snap-to-edge behavior and spring-physics drag.
pub struct RuneInspector {
    pub title: String,
    pub content: AnyView,
    pub position: InspectorPosition,
    pub size: (f32, f32),
    pub material: GlassMaterial,
    pub is_expanded: bool,
    pub on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

pub enum InspectorPosition {
    /// Fixed to the trailing edge of the content area.
    TrailingAttached,
    /// Floating at an absolute position.
    Floating { x: f32, y: f32 },
    /// Snapped to a screen edge.
    SnappedEdge(ScreenEdge),
}
```

Drag behavior: When a user drags the inspector's title bar, a `SleipnirSolver` tracks the pointer with `params: SleipnirParams { stiffness: 280.0, damping: 28.0, mass: 1.0 }` — slightly stiffer than the default fluid preset, giving the panel a heavy, physical feel. On release, if the panel is within 24pt of a screen edge, it snaps to `InspectorPosition::SnappedEdge` via another spring animation.

The glass material for inspectors uses `blur_multiplier = 2.0` and `ior_override = 1.55` — heavier than toolbars, lighter than modal sheets. This creates a visual depth hierarchy: toolbar (1.4×) → inspector (2.0×) → modal (2.8×).

### 3.3 — Context Menu (GaldraMenu)

Named after Galdr — the spoken form of Norse magic, invoked at a specific point to change what is possible.

```rust
/// Right-click context menu with glass styling and keyboard navigation.
pub struct GaldraMenu {
    pub items: Vec<GaldraMenuItem>,
    pub anchor: MenuAnchor,
    pub material: GlassMaterial,
    pub on_dismiss: Arc<dyn Fn() + Send + Sync>,
}

pub enum GaldraMenuItem {
    Action {
        label: String,
        icon: Option<String>,
        shortcut: Option<KeyboardShortcut>,
        enabled: bool,
        action: Arc<dyn Fn() + Send + Sync>,
    },
    Submenu {
        label: String,
        icon: Option<String>,
        items: Vec<GaldraMenuItem>,
    },
    Separator,
    Header(String),
}
```

Context menus render into the portal layer (`Renderer::enter_portal(z_index: 1000)`) and dismiss on `PointerDown` outside their bounds (using the existing `OverlayModifier::on_dismiss` mechanism). They appear with a spring-in animation: scale from 0.9 to 1.0 with `SleipnirParams::snappy()`, opacity from 0 to 1 with a 60ms linear fade. This matches Tahoe exactly.

Keyboard navigation: arrow keys move the selection, `Return` activates, `Escape` dismisses, `Tab`/`Shift+Tab` move between items. The selection highlight uses the same glass pill approach as `HrungnirSegmented`.

### 3.4 — SearchBar (MimirSearch)

The existing `MimirSpotlight` has a full search input internally. Extract and expose it as a standalone component:

```rust
/// Standalone glass search bar with scope buttons and token pills.
pub struct MimirSearch {
    pub query: String,
    pub placeholder: String,
    pub scope: Option<Vec<SearchScope>>,
    pub selected_scope: usize,
    pub tokens: Vec<SearchToken>,
    pub on_change: Arc<dyn Fn(String) + Send + Sync>,
    pub on_submit: Arc<dyn Fn(String) + Send + Sync>,
    pub on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
    pub style: SearchBarStyle,
}

pub enum SearchBarStyle {
    /// Toolbar-sized, compact, rounded pill.
    Compact,
    /// Full-width, taller, with scope bar below.
    Expanded,
    /// Spotlight-style, full-width, very tall, centered.
    Spotlight,
}
```

The clear button (✕) appears via spring animation when `query.len() > 0`. The search icon pulses with a `GungnirPulseModifier { speed: 2.0, radius: 3.0 }` while a search is in progress (indicated via a `searching: bool` prop). Scope buttons render as a `HrungnirSegmented` in `SegmentedStyle::Capsule` style below the input in `Expanded` mode.

---

## Phase 4 — Berserker Theme: The Differentiator
### Timeline: 1.5 weeks | Risk: Low | Impact: Identity-Defining

This is what makes CVKG applications unmistakable. No other framework has anything like this. The berserker theme is not a color palette — it is a complete alternate reality for the UI where glass becomes ice, borders become carved runes, and particles float through the background.

### 4.1 — ColorTheme::berserker()

```rust
pub fn berserker() -> Self {
    Self {
        // Blood-iron neon: warm red with aggressive intensity
        primary_neon: [1.0, 0.08, 0.12, 1.8],
        // Bone-white shatter: cold contrast to the blood red
        shatter_neon: [0.95, 0.92, 0.88, 1.6],
        // Smoked obsidian glass: near-black with iron undertones
        glass_base: [0.03, 0.02, 0.02, 0.88],
        // Forge-edge: hot orange-white at the glass boundary
        glass_edge: [0.8, 0.35, 0.08, 0.7],
        // Elder rune glow: aged amber-gold
        rune_glow: [0.9, 0.72, 0.3, 1.0],
        // Heart of the ember: deep burning orange
        ember_core: [0.98, 0.25, 0.05, 1.0],
        // The void between stars
        background_deep: [0.01, 0.005, 0.005, 1.0],
        // Berserker has no gentle cursor glow — the UI itself blazes
        mani_glow: [0.8, 0.2, 0.05, 0.08],
        // Maximum blur — the glass is thick, ancient, imperfect
        glass_blur_strength: 0.85,
        // Wide shatter edges — violence is visible
        shatter_edge_width: 2.8,
        // Aggressive neon bloom
        neon_bloom_radius: 0.035,
        // Elder runes are always visible in berserker mode
        rune_opacity: 0.85,
    }
}
```

The `glass_tint_adapt` value for berserker is 0.15 — very low. The glass does not adapt to what's behind it. It has its own character. It imposes itself on the content rather than reflecting it.

### 4.2 — Runic Ornament System (ÆttiRunes)

Named after the Ættir — the three groups of eight runes in the Elder Futhark. This is the CVKG ornamental border system.

The key insight: SVG-based ornamental borders cannot be done with `fill_rect` and `stroke_rect`. They require path rendering. CVKG already has `load_svg` and `draw_svg` on the `Renderer` trait. The runic ornament system uses these to apply procedurally-generated rune sequences as frame decorations.

**ÆttiFrame — the ornamental frame component:**

```rust
pub struct ÆttiFrame {
    pub style: RunicStyle,
    pub intensity: f32,
    pub animate: bool,
}

pub enum RunicStyle {
    /// Elder Futhark characters carved into stone, arranged as a border.
    CarvedStone,
    /// Interlocking knotwork pattern, Celtic-Viking fusion.
    Knotwork,
    /// Hammered metal with rivets at corners.
    HammeredMetal,
    /// Dragon-scale tessellation, scales toward corners.
    DragonScale,
    /// Ice crystal formations, growing from corners.
    IceCrystal,
}
```

Each style is pre-generated as a set of SVG path templates that are parameterized by the component's dimensions. The generation runs once on component creation, produces an SVG string, passes it through `Renderer::load_svg()`, and is subsequently rendered via `Renderer::draw_svg()` each frame. The SVG paths are hand-crafted by a designer with a deep knowledge of Norse ornamental patterns and stored as compressed templates in the binary via `include_str!()`.

For `animate = true`, each rune glyph has an individual `GungnirPulseModifier` with a randomized phase offset, creating an organic "breathing" animation where different runes glow at different times. The phase offset is derived from the rune's position in the border sequence using a Perlin-noise-based timing function.

### 4.3 — Material Wear Shaders (ÞrymrSurface)

Named after Þrymr, the frost giant king whose hall was described as ancient and weathered.

These are WGSL shader functions added to `material_opaque.wgsl` as a new material mode 22 (`MATERIAL_WORN`):

```wgsl
/// Apply battle-worn surface damage: scratches, cracks, burn marks.
/// damage_level: [0.0, 1.0] — 0 = pristine, 1 = heavily damaged.
/// damage_seed: per-component random seed for variation.
fn worn_surface(
    uv: vec2<f32>,
    base_color: vec4<f32>,
    damage_level: f32,
    damage_seed: f32,
) -> vec4<f32> {
    var color = base_color;
    
    // Scratches: high-frequency noise along a directional gradient
    let scratch_dir = normalize(vec2(0.7, 0.3) + vec2(damage_seed * 0.2, damage_seed * 0.15));
    let scratch_uv = vec2(dot(uv, scratch_dir), dot(uv, vec2(-scratch_dir.y, scratch_dir.x)));
    let scratch = fbm(scratch_uv * 80.0 + damage_seed * 10.0);
    let scratch_mask = smoothstep(0.72, 0.78, scratch) * damage_level;
    
    // Cracks: larger, branching fractures
    let crack_n = fbm(uv * 12.0 + damage_seed * 7.0);
    let crack_mask = smoothstep(0.68, 0.73, crack_n) * damage_level * 0.6;
    
    // Burn marks: radial dark patches
    let burn_center = vec2(fract(damage_seed * 3.7), fract(damage_seed * 5.3));
    let burn_dist = distance(uv, burn_center);
    let burn_mask = smoothstep(0.3, 0.0, burn_dist) * damage_level * vnoise(uv * 5.0) * 0.7;
    
    // Apply: scratches lighten (exposed metal), cracks and burns darken
    color.rgb += scratch_mask * 0.25;
    color.rgb -= crack_mask * 0.4;
    color.rgb -= burn_mask * 0.5;
    
    return color;
}
```

Exposed via `Renderer::set_material(DrawMaterial::Worn { damage_level: f32, seed: f32 })`.

### 4.4 — Ambient UI Particles (EmberDrift)

The audit identifies this gap (#24): "no persistent ambient particles for the UI chrome itself." This is implemented as a compute-shader particle system that runs independently of the frame render and composites into the background layer.

**Design:** 512 particles maximum. Each particle has position (vec2), velocity (vec2), life (f32), size (f32), and type (u8: ember, snow, rune_fragment, spark). The simulation runs as a compute pass in `PassId::Particles` inserted before `PassId::UI` in the Kvasir graph.

**Particle types by scene:**
- Asgard mode: glowing cyan sparks with slow upward drift
- Berserker mode: orange-red embers with turbulent motion
- Niflheim/ice variants: white crystalline snowflakes with gentle fall
- Scene type `SCENE_YGGDRASIL`: glowing leaf fragments in gold and green

The compute pass is gated: it runs only when `TelemetryData::berserker_rage > 0.1` or when the scene type is ambient-capable. At rage = 0, the compute pass is skipped entirely via `PassNode::disabled(Particles)`. This ensures zero performance cost in standard usage.

```rust
pub struct EmberDriftConfig {
    pub max_particles: u32,
    pub emit_rate: f32,       // particles per second
    pub particle_type: ParticleType,
    pub gravity: f32,
    pub turbulence: f32,
    pub color_seed: [f32; 4], // base color, perturbed per particle
    pub lifetime: f32,        // seconds
}
```

### 4.5 — Seiðr Holographic Scanline Effect

The Seiðr effect exists. It is not applied to glass chrome. This is a one-line fix per component that should have already been done:

In `NiflheimSidebar::render()`, after the glass background, apply:
```rust
if realm == Realm::Asgard && theme == berserker_or_asgard {
    renderer.apply_effect(EffectId::Seiðr { 
        intensity: 0.3, 
        scan_speed: 0.8 
    });
}
```

The effect renders a subtle CRT-style horizontal scan line pattern at low opacity, with a slow downward movement. On glass surfaces, the scan lines are additively blended so they appear as faint light bands rather than dark lines. The visual result is unmistakably cyberpunk.

### 4.6 — Additional MjolnirFrame Variants

Add to the existing `MjolnirFrame` system:

```rust
pub enum MjolnirFrameStyle {
    // Existing
    Standard,
    // New
    
    /// Carved runestone: weathered edges with embedded runes.
    RuneStone { runes: Vec<RuneGlyph> },
    
    /// Hammered metal: irregular forged surface with rivet points at corners.
    HammeredMetal { oxidation: f32 },
    
    /// Dragon scale: interlocking scale tessellation radiating from center.
    DragonScale { scale_size: f32 },
    
    /// Ice crystal: fractal ice growth from corner anchor points.
    IceCrystal { growth_progress: f32 },
    
    /// Void rift: dark energy tearing at the frame boundaries.
    VoidRift { rift_intensity: f32 },
}
```

Each variant is implemented as a WGSL SDF evaluation that replaces the standard border stroke. The `IceCrystal` variant is particularly effective: it uses a recursive SDF approximation that produces branching crystalline patterns that look like frost growing on the frame edges.

---

## Phase 5 — Motion and Physics Refinement
### Timeline: 1 week | Risk: Low | Impact: Polish

### 5.1 — Parallax Depth System

Each UI element receives a `depth: f32` property (range 0.0–1.0, where 0.0 is the background and 1.0 is foreground). During scroll or window drag, elements shift their UV sampling offset proportional to their depth:

```rust
pub struct ParallaxModifier {
    /// Depth in the UI stack. 0.0 = background, 1.0 = foreground.
    /// Glass panels: 0.3–0.5. Toolbars: 0.6. Modals: 0.9.
    pub depth: f32,
    /// Maximum parallax offset in logical pixels.
    pub max_offset: f32,
}
```

The offset is computed from the scroll velocity (stored in `KnowledgeState::pointer_velocity`) and applied as a `push_transform` translation before each element renders. The effect is subtle on normal elements (1–4px movement) but visible and beautiful on glass panels where the backdrop blur sampling shifts independently of the panel geometry.

### 5.2 — Audio-Reactive Visuals

The `AudioEngine` trait exists. The `HapticEngine` trait exists. Neither feeds into the shader pipeline.

Add an `AudioAnalysis` struct to `KnowledgeState`:

```rust
pub struct AudioAnalysis {
    /// Normalized energy in bass frequencies [20–200 Hz], range [0,1].
    pub bass: f32,
    /// Normalized energy in mid frequencies [200–2kHz], range [0,1].
    pub mid: f32,
    /// Normalized energy in treble frequencies [2k–20kHz], range [0,1].
    pub treble: f32,
    /// Instantaneous peak amplitude, range [0,1].
    pub amplitude: f32,
    /// Beat detected this frame (true/false).
    pub beat: bool,
}
```

Add `audio_analysis: AudioAnalysis` to `SceneUniforms` (four floats: bass, mid, treble, amplitude + a beat flag packed into the beat u32).

In the berserker WGSL shaders, `berzerker_rage` is already a scene-level uniform. Augment it with audio-reactivity: when `audio.beat == true`, trigger a brief rage spike (`berzerker_rage = clamp(audio.amplitude * 1.5, 0.0, 1.0)`) that decays via the existing rage decay logic in `about_to_wait`.

The visual result in berserker mode: the glass panels' chromatic aberration pulses on bass hits, the ember particles accelerate and scatter on beats, and the rune glows brighten on treble energy. The UI becomes a live visualization of the audio environment.

### 5.3 — Declarative Animation Primitives

The existing `cvkg-anim` crate is comprehensive but imperative. Add a declarative layer:

```rust
/// Animate a value when a condition changes.
/// Wraps SleipnirSolver in a view-layer primitive.
///
/// Example:
/// ```rust
/// view.animated(
///     AnimationTarget::Opacity,
///     if is_visible { 1.0 } else { 0.0 },
///     SleipnirParams::fluid(),
/// )
/// ```
pub trait AnimatableView: View + Sized {
    fn animated(
        self,
        target: AnimationTarget,
        value: f32,
        params: SleipnirParams,
    ) -> ModifiedView<Self, AnimatedModifier>;
    
    fn spring_scale(self, scale: f32) -> ModifiedView<Self, AnimatedModifier>;
    fn spring_opacity(self, opacity: f32) -> ModifiedView<Self, AnimatedModifier>;
    fn spring_offset(self, x: f32, y: f32) -> ModifiedView<Self, AnimatedModifier>;
}
```

This is implemented via the existing `SleipnirModifier` infrastructure but exposed with a clean API. The `AnimatedModifier` stores the previous value and creates or updates a `SleipnirSolver` in `KnowledgeState::component_states` on each render, which is already the correct pattern.

---

## Phase 6 — Production Hardening and 2028-Readiness
### Timeline: 1 week | Risk: Low | Impact: Longevity

### 6.1 — Accessibility: ShieldWall Completion

The accessibility infrastructure (`accesskit`) is present but thinly used. Every new component in Phases 2–4 must implement `aria_properties()` correctly:

- `NornirBar`: `AriaRole::Menubar`, children are `AriaRole::Menu`
- `HeimdallDock`: `AriaRole::List`, items are `AriaRole::ListItem` with `description = "Running"` if applicable
- `HrungnirSegmented`: `AriaRole::Radiogroup`, items are `AriaRole::Radio` with `checked`
- `GaldraMenu`: `AriaRole::Menu`, items are `AriaRole::Menuitem`
- `MimirSearch`: `AriaRole::Search` with `AriaRole::Textbox` inside

Add `AccessibilityPreferences::should_disable_glass()` check to all glass materials at render time: when the user has enabled "Reduce Transparency," all glass materials fall back to `DrawMaterial::Opaque` with the `glass_base` color used as a flat fill. This is already structurally possible; it just needs to be plumbed into each component's render path.

### 6.2 — Performance Contract

Add a `PerformanceContract` to `cvkg-core` that each component declares:

```rust
pub struct PerformanceContract {
    /// Maximum acceptable render time for this component per frame (microseconds).
    pub max_render_us: u32,
    /// Whether this component uses glass (requires backdrop blur pass).
    pub uses_glass: bool,
    /// Whether this component has ambient animation (requires continuous redraws).
    pub continuous_animation: bool,
    /// GPU tier minimum required for full-quality rendering.
    pub min_tier: RenderTier,
    /// Fallback behavior on Tier3Fallback hardware.
    pub tier3_fallback: Tier3Fallback,
}

pub enum Tier3Fallback {
    /// Render as flat opaque surface.
    FlatOpaque,
    /// Render normally but disable effects.
    NoEffects,
    /// Do not render at all (invisible).
    Hidden,
}
```

The `SurtrRenderer` reads each component's contract (via a registry populated at startup) and adjusts quality accordingly. This is the principled version of the existing `is_over_budget()` check: instead of binary "over/under budget," it makes targeted quality decisions per component.

### 6.3 — Component Composition Protocol

The current component API is render-function-oriented. For complex desktop apps, developers need a composable protocol:

```rust
/// Trait implemented by all CVKG chrome components.
/// Provides a uniform interface for theming, accessibility, and animation.
pub trait ChromeComponent: View + Send {
    /// Returns the component's default glass material.
    fn default_material() -> GlassMaterial;
    
    /// Returns the component's default performance contract.
    fn performance_contract() -> PerformanceContract;
    
    /// Applies the current theme to this component, returning a modified version.
    /// Called automatically by the renderer when the theme changes.
    fn apply_theme(self, theme: &Theme) -> Self;
    
    /// Returns accessibility properties for this component's root element.
    fn root_aria() -> AriaProperties;
}
```

This protocol is implemented by every component in Phases 2–4. It enables a `ThemeProvider` wrapper that walks the component tree and calls `apply_theme()` on all `ChromeComponent` implementors when the theme changes — eliminating the need to manually update every component when toggling dark/light mode or switching between Asgard/Berserker presets.

### 6.4 — Spatial Computing Preparation

By 2028, a significant portion of desktop applications will have spatial/XR variants. CVKG should be ready.

Add to `cvkg-core`:
```rust
/// Hint to the renderer about the target display environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayEnvironment {
    /// Standard flat display (monitor, laptop).
    #[default]
    Flat,
    /// Spatial display (Apple Vision Pro, Meta Quest, etc.).
    /// Elements have physical depth; glass reflects actual environment.
    Spatial,
    /// Head-up display (projected overlay, not a physical screen).
    HeadsUp,
}
```

When `DisplayEnvironment::Spatial`, the glass shader activates its full IOR model and the parallax system uses 3D depth values. The `depth: f32` in `ParallaxModifier` becomes a real Z coordinate. The `BifrostModifier` uses actual environment mapping rather than screen-space backdrop blur.

The framework does not need to implement spatial rendering now. It needs to have the right abstractions so that when the spatial backend is written, it slots in without breaking the existing component API.

### 6.5 — Hot-Reload Protocol

The `cvkg-webkit-server` has an HMR WebSocket. Extend this to native:

```rust
/// Hot-reload server for development.
/// In debug builds, watches component source files and triggers re-renders.
/// In release builds, this entire module is compiled out.
#[cfg(debug_assertions)]
pub struct KvasirHotReload {
    /// Path patterns to watch for changes.
    pub watch_paths: Vec<String>,
    /// Debounce duration before triggering reload.
    pub debounce_ms: u64,
}
```

In the `App::about_to_wait()` loop, check for a hot-reload signal. When received, call `update_system_state()` to increment a generation counter, which triggers a full re-render. This is the minimal hot-reload implementation: state is preserved, layout is recalculated, and the view tree is rebuilt from the new source. No WASM, no scripting, no dynamic linking required.

---

## Component Architecture Overview

After all six phases, the CVKG component hierarchy looks like this:

```
cvkg-components/
├── chrome/                     ← New: Application shell components
│   ├── nornir_bar.rs           ← Menu bar (Phase 2.1)
│   ├── heimdall_dock.rs        ← macOS dock (Phase 2.2)
│   ├── valkyrie_toolbar.rs     ← Floating toolbar (Phase 2.3)
│   ├── niflheim_sidebar.rs     ← Glass sidebar chrome (Phase 3.1)
│   ├── rune_inspector.rs       ← Floating inspector panel (Phase 3.2)
│   └── valkyrie_titlebar.rs    ← Title bar integration
│
├── interactive/                ← Extended: Glass button variants
│   ├── buttons.rs              ← +Glass, TintedGlass, Capsule, Icon variants
│   ├── hrungnir_segment.rs     ← Segmented control (Phase 2.4)
│   ├── galdra_menu.rs          ← Context menu (Phase 3.3)
│   └── mimir_search.rs         ← Standalone search bar (Phase 3.4)
│
├── ornamental/                 ← New: Visual identity components
│   ├── aetti_frame.rs          ← Runic ornament borders (Phase 4.2)
│   ├── ember_drift.rs          ← Ambient particle system (Phase 4.4)
│   └── mjolnir_frame_ext.rs    ← Additional frame variants (Phase 4.6)
│
├── material/                   ← Extended: Material system
│   ├── glass_instance.rs       ← GlassInstanceUniforms (Phase 1.5)
│   ├── thrymr_surface.rs       ← Worn/damaged material (Phase 4.3)
│   └── parallax.rs             ← Depth parallax modifier (Phase 5.1)
│
└── themes/                     ← Extended: Theme system
    ├── berserker.rs            ← Berserker ColorTheme preset (Phase 4.1)
    ├── glass_bridge.rs         ← OKLCH → GPU tint path (Phase 0.1)
    └── chrome_component.rs     ← ChromeComponent trait (Phase 6.3)
```

---

## GPU Pipeline Architecture (Post-Phases 0–2)

The Kvasir render graph after all architectural changes:

```
[Geometry Pass]
    │ scene texture
    ▼
[BackdropCopy Pass]         ← Full-scene copy for global glass
    │ backdrop texture
    ▼
[BackdropBlur Pass]         ← Kawase pyramid (existing, unchanged)
    │ blurred backdrop (mip chain)
    ▼
[BackdropRegion Pass ×N]    ← NEW: Per-element isolated blurs (Phase 0.3)
    │ per-element blur textures
    ▼
[Particles Pass]            ← NEW: EmberDrift compute (Phase 4.4)
    │ particle texture (additive)
    ▼
[Glass Pass]                ← Material 7, using per-element blur textures
    │                       ← NEW: IOR refraction, adaptive tint, edge smear
    ▼
[UI Pass]                   ← All opaque and non-glass UI
    ▼
[BloomExtract Pass]
    ▼
[BloomBlur Pass]
    ▼
[Composite Pass]            ← ACES tonemap, bloom fusion
    ▼
[Accessibility Pass]        ← Color blindness simulation
    ▼
[Present]
```

The critical change is `BackdropRegion ×N` being interleaved between `BackdropBlur` and `Glass`. Each glass element requests a region blur when `GlassInstanceUniforms::blur_multiplier != 1.0`. The scheduler in `CompositorEngine` batches region requests into a single compute pass with multiple output targets, maintaining O(1) GPU submissions regardless of glass element count.

---

## Shader Architecture (Post-Phase 1)

`material_glass.wgsl` becomes three conceptually separate sections:

```wgsl
// ─── Section 1: Geometry and Clipping ───────────────────────────────────────
// SDF clipping, Mjolnir slice, viewport bounds (unchanged from current)

// ─── Section 2: Physical Optics ─────────────────────────────────────────────
// NEW: Snell's law refraction
// NEW: Per-instance IOR from GlassInstanceUniforms
// IMPROVED: Thickness-based alpha model (SSS approximation)
// IMPROVED: Fresnel reflectance with IOR

// ─── Section 3: Adaptive Appearance ─────────────────────────────────────────
// NEW: Adaptive tint from backdrop dominant color
// NEW: Per-instance tint override
// IMPROVED: Edge smear convolution
// IMPROVED: Crystalline edge highlight
// EXISTING: Chromatic aberration (unchanged, already good)

// ─── Section 4: Surface Weathering (Berserker only) ──────────────────────────
// NEW: worn_surface() function for material 22
// NEW: Seiðr scanline overlay at low opacity
```

The glass shader is gated by `theme.glass_tint_adapt` and `instance.tint_override.w` — if both are zero, the entire Section 3 is skipped via early branch, maintaining performance for Midgard (flat) mode.

---

## Testing Strategy

For each phase, the following test classes are required before merge:

**Unit tests (existing `#[test]` pattern):**
- `HeimdallDock::dock_item_magnification()` — verify Gaussian falloff shape
- `HrungnirSegmented` — verify spring animation state machine transitions
- `GaldraMenu` — verify keyboard navigation FSM
- `GlassInstanceUniforms` — verify bytemuck Pod alignment

**Visual regression tests:**
Each new component renders to a PNG via `Renderer::capture_png()` and is compared against a reference image (stored in `test_fixtures/`). Tolerance: 98% pixel match. These run in CI on every PR.

**Performance baseline tests:**
Using the existing `TelemetryData` infrastructure, every new component must demonstrate:
- Glass components: < 0.3ms GPU time per frame on Tier1GPU hardware
- Particle system: < 0.1ms compute time with 512 particles
- Spring animations: < 0.01ms CPU time per frame per animator

**Accessibility audits:**
Every new component runs through the `accessibility_preferences().should_disable_glass()` path in tests, verifying that the opaque fallback renders correctly and that all ARIA properties are correctly set.

---

## The Non-Negotiable Design Standards

These are not suggestions. They are constraints that every piece of code in every phase must satisfy:

1. **No hardcoded color values anywhere in component code.** All colors come from `use_theme()` or `GlassInstanceUniforms`. The berserker theme must be achievable by changing only the `ColorTheme` and `GlassMaterial` — not by touching component render logic.

2. **Every new `pub fn` has a doc comment stating its contract** (WHY and WHAT, not HOW). This is CVKG Guideline #6.

3. **Every animation uses `SleipnirSolver`.** No `lerp()` calls in animation code. No linear interpolation of visible values. Springs only.

4. **Every component degrades gracefully on Tier3 hardware.** The `PerformanceContract::tier3_fallback` must be tested.

5. **Every glass element declares its `depth: f32`** for the parallax system. This is not optional even when parallax is not visually important — it must be correct for when spatial computing support is added in Phase 6.

6. **The Norse naming convention is sacred.** New components get Norse names. If a component cannot be given a meaningful Norse name, it is probably not the right abstraction and should be reconsidered.

---

## What Success Looks Like

When this plan is complete, an application built with CVKG will:

- Open with a `HeimdallDock` that magnifies beautifully under the pointer, items bouncing on launch with physics that feel weighted and real
- Display a `NornirBar` menu whose glass background adapts to the window's content — warmer over warm scenes, cooler over dark backgrounds
- Show floating `RuneInspector` panels that drag with the physical weight of thick glass, snapping to screen edges with a satisfying click
- In berserker mode, pulse with ember particles drifting upward, rune borders breathing with amber light, and the UI itself responding to audio beats with chromatic surges
- On any system running accessibility preferences, render as a clean, high-contrast, fully accessible application with no loss of usability

No other native UI framework in any language will produce applications that look or feel like this.

The code will be maintainable because it is modular: each phase is a set of new files added to the existing structure, with minimal changes to existing code. The Norse naming makes the architecture comprehensible to anyone who takes the time to understand it. The performance contracts make degradation principled and predictable.

This is not an update to CVKG. It is the completion of what CVKG was always meant to be.

---

## Appendix A — Implementation Order Matrix

| Phase | Dependency | Parallel With | Estimated Lines |
|-------|-----------|---------------|-----------------|
| 0.1 OKLCH→GPU | None | 0.2, 0.3 | ~40 |
| 0.2 bcs_frosted wire | None | 0.1, 0.3 | ~25 |
| 0.3 BackdropRegion | None | 0.1, 0.2 | ~120 |
| 1.1 Snell's law | 0.1, 0.3 | 1.2–1.5 | ~60 |
| 1.2 Adaptive tint | 0.1 | 1.1, 1.3–1.5 | ~45 |
| 1.3 Edge smear | 1.1 | 1.2, 1.4–1.5 | ~50 |
| 1.4 SSS approx | 1.1 | 1.2, 1.3, 1.5 | ~40 |
| 1.5 Per-instance uniforms | 0.1 | 1.1–1.4 | ~80 |
| 2.1 NornirBar | 1.5 | 2.2–2.5 | ~400 |
| 2.2 HeimdallDock | 1.5 | 2.1, 2.3–2.5 | ~350 |
| 2.3 ValkyrieToolbar | 2.4 | 2.1, 2.2 | ~300 |
| 2.4 HrungnirSegmented | 1.5 | 2.1–2.3, 2.5 | ~200 |
| 2.5 Glass buttons | 1.5 | 2.1–2.4 | ~150 |
| 3.1 NiflheimSidebar | 1.5 | 3.2–3.4 | ~200 |
| 3.2 RuneInspector | 1.5 | 3.1, 3.3–3.4 | ~250 |
| 3.3 GaldraMenu | 1.5 | 3.1, 3.2, 3.4 | ~300 |
| 3.4 MimirSearch | None | 3.1–3.3 | ~180 |
| 4.1 Berserker preset | 0.1 | 4.2–4.6 | ~60 |
| 4.2 ÆttiRunes | 4.1 | 4.3–4.6 | ~400 |
| 4.3 ÞrymrSurface | 4.1 | 4.2, 4.4–4.6 | ~100 |
| 4.4 EmberDrift | 4.1 | 4.2, 4.3, 4.5–4.6 | ~350 |
| 4.5 Seiðr application | None | 4.1–4.4, 4.6 | ~30 |
| 4.6 MjolnirFrame ext. | 4.1 | 4.2–4.5 | ~200 |
| 5.1 Parallax | 1.5 | 5.2, 5.3 | ~120 |
| 5.2 Audio-reactive | None | 5.1, 5.3 | ~150 |
| 5.3 Declarative anim | None | 5.1, 5.2 | ~200 |
| 6.1 A11y completion | 2.1–3.4 | 6.2–6.5 | ~300 |
| 6.2 Performance contracts | All | 6.3–6.5 | ~150 |
| 6.3 ChromeComponent trait | 2.1–3.4 | 6.1, 6.2, 6.4–6.5 | ~200 |
| 6.4 Spatial prep | 5.1 | 6.1–6.3, 6.5 | ~100 |
| 6.5 Hot-reload | None | 6.1–6.4 | ~150 |

**Total estimated new code: ~5,100 lines**
Total across all phases: approximately 8 weeks of focused engineering work.

---

## Appendix B — The Aesthetic Standard

Every component in CVKG should, when shown to a designer who has never seen the framework, produce one of two reactions:

1. "That's clearly inspired by macOS Tahoe — but it has something extra that macOS doesn't."
2. "I have never seen anything like that in a native application."

The first reaction is for the glass components, the toolbar, the dock. The second is for the berserker mode, the runic ornaments, the ember drift. Both reactions are the goal. Neither is sufficient alone.

The framework exists at the intersection of two design traditions that have never been combined: the precision and restraint of Apple's Human Interface Guidelines, and the aggressive, mythological visual language of Norse/cyberpunk aesthetics. The combination should feel inevitable in retrospect — as if these two traditions were always meant to meet in a Rust UI framework.

This is CVKG's irreplaceable position. No other framework occupies this space. The implementation plan above protects and deepens that position with every phase.

*Valhöll or nothing.*

---

*Document version: 1.0 | Architecture level: Magnum Opus | Author: CVKG Engineering*

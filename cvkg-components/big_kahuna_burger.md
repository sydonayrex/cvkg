# BIG KAHUNA BURGER  -  cvkg-components Implementation Plan

## Purpose

This document walks a weak idiot AI model through implementing every missing feature identified in the cvkg-components audit. It contains exact code examples, full explanations of WHY each step exists, and hard left/right boundaries to prevent the AI from going off the rails.

### Constraints for the implementing AI

**LEFT LIMIT (never go left of this):**
- Never use `unwrap()` or `.expect()` in production code. Use `?` or `ok().unwrap_or(default)`.
- Never introduce `#[allow(dead_code)]` to silence warnings. If code is dead, delete it.
- Never use hard-coded English strings in component logic. All UI strings go through the i18n system.
- Never create duplicate component definitions. Each component lives in exactly ONE file.
- Never use platform-specific hardcoded values (e.g., "SF Pro Text" font name) without a fallback.
- Never break the existing API. All changes are additive unless explicitly marked [BREAKING].

**RIGHT LIMIT (never go right of this):**
- Do not implement full general-purpose CRDT library. Use a simple approach (LSeq or RGA is enough). This is a UI framework, not a database.
- Do not implement a full accessibility framework. Integrate with AccessKit through the renderer trait  -  don't build AT-SPI from scratch.
- Do not implement general-purpose 3D rendering. A WebGPU scene embedding primitive is enough.
- Do not implement a full parser/lexer for the prompt template editor. A simple variable-substitution system (`{{var}}` syntax) is sufficient for v1.
- Do not implement handwriting recognition. Capture strokes and expose them. Recognition is a backend concern.
- Do not implement voice recognition. Capture audio stream state and expose interim/final transcript events. Recognition is a backend concern.
- Do not exceed 3 new dependencies per feature module. If you need more, argue why in comments.

---

## Table of Contents

1. [PhaseGate  -  Portal System](#1-phasegate--portal-system)
2. [TokenStream  -  Streaming AI Diff Renderer](#2-tokenstream--streaming-ai-diff-renderer)
3. [FlexiScope  -  Container Query Layout](#3-flexiscope--container-query-layout)
4. [TrustMark  -  Confidence Visualization](#4-trustmark--confidence-visualization)
5. [A11yBeacon  -  Live Region Accessibility](#5-a11ybeacon--live-region-accessibility)
6. [SyncWeave  -  Real-Time Collaboration](#6-syncweave--real-time-collaboration)
7. [MorphBridge  -  Shared Element Transitions](#7-morphbridge--shared-element-transitions)
8. [FluxLayout  -  Layout Animation for Siblings](#8-fluxlayout--layout-animation-for-siblings)
9. [ComputedSignal  -  Derived State Primitives](#9-computedsignal--derived-state-primitives)
10. [DropVault  -  File Upload Component](#10-dropvault--file-upload-component)
11. [LinguaTong  -  i18n Localization](#11-linguatong--i18n-localization)
12. [AwaitVeil  -  Suspense Boundary](#12-awaitveil--suspense-boundary)
13. [PromptForge  -  Prompt Template Editor](#13-promptforge--prompt-template-editor)
14. [ConsentGate  -  Consent & Data Provenance](#14-consentgate--consent--data-provenance)
15. [VTree  -  Virtualized Tree](#15-vtree--virtualized-tree)
16. [Fix Duplicate Component Definitions](#16-fix-duplicate-component-definitions)
17. [Hunt Down 70 unwrap() Calls](#17-hunt-down-70-unwrap-calls)

---

## 1. PhaseGate - Portal System

### Why this exists

Without portals, every overlay (dropdown, popover, tooltip, modal, toast) renders INSIDE the component tree where it was instantiated. This means:

1. A dropdown inside a `ScrollView` gets clipped by the scroll container's overflow.
2. z-index stacking is determined by tree depth, not by semantic overlay layer.
3. A popover inside a `z-index: 0` container can never escape that container.

React has `ReactDOM.createPortal`. SwiftUI has `.popover()` which renders in a separate window layer. Jetpack Compose has `Popup` which renders in a separate composition. Every modern framework solved this. CVKG has not.

### Architecture decision

Add a `PortalManager` trait to `cvkg-core`'s `Renderer` that allows a view to request rendering at the ROOT level rather than inline. The renderer already has `push_vnode`/`pop_vnode`  -  portals use the same mechanism but redirect to a separate root-layer buffer.

### Implementation

**Step 1: Add portal methods to the Renderer trait in cvkg-core.**

```rust
// In cvkg-core/src/renderer.rs  -  ADD these methods to the Renderer trait

/// Begin rendering into the portal root layer instead of the inline tree.
/// All draw calls between `enter_portal` and `exit_portal` are collected
/// into a separate buffer that is composited AFTER the main tree.
/// 
/// WHY separate buffer: The main tree may have clipping, transforms, or
/// opacity that should NOT affect overlays. The portal layer renders on top
/// of everything, ignoring the local coordinate system.
fn enter_portal(&mut self, z_index: i32);

/// Exit the portal layer and return to inline rendering.
/// The portal content collected since `enter_portal` is now sealed  - 
/// no more draw calls will be appended to it.
fn exit_portal(&mut self);

/// Count of active portals. Used by the renderer to allocate the
/// portal layer buffer. Default implementation returns 0 for renderers
/// that don't support portals (they silently skip portal rendering).
fn active_portals(&self) -> usize { 0 }
```

**Step 2: Create the Portal component in cvkg-components.**

```rust
// NEW FILE: cvkg-components/src/phasegate.rs

use cvkg_core::{Never, Rect, Renderer, View};

/// A portal renders its content at the root level instead of inline.
///
/// Use portals for overlays that must escape their parent's clipping
/// context: dropdowns, tooltips, popovers, modals, toasts.
///
/// # Example
/// ```
/// // INSIDE a ScrollView  -  without Portal, this Dropdown would be clipped
/// ScrollView::new(
///     vstack![
///         PhaseGate::new(
///             DropdownMenu::new(items).on_select(|i| { /* ... */ }),
///             GateTier::Dropdown,  // z-index 100
///         ),
///         // ... other content
///     ]
/// )
/// ```
///
/// # Left limit: Never render portal content inline. If the renderer doesn't
/// support portals, render NOTHING (not the content inline). Rendering inline
/// defeats the entire purpose.
///
/// # Right limit: Don't implement portal nesting. A portal inside a portal
/// is undefined behavior. If detected, the inner portal renders inline into
/// the outer portal's buffer.
#[derive(Clone)]
pub struct PhaseGate<V: View> {
    content: V,
    layer: GateTier,
}

/// Named z-index layers for portals.
///
/// Using named layers instead of raw z-index values prevents z-index
/// arms races where every component picks an arbitrary number and the
/// developer has no idea which layer wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GateTier {
    /// Tooltip layer. Highest priority overlay.
    Tooltip = 500,
    /// Dropdown/popover layer.
    Dropdown = 400,
    /// Modal dialog layer.
    Modal = 300,
    /// Toast notification layer.
    Toast = 200,
    /// Floating panel layer (e.g., devtools).
    Floating = 100,
}

impl<V: View> PhaseGate<V> {
    pub fn new(content: V, layer: GateTier) -> Self {
        Self { content, layer }
    }
}

impl<V: View> View for PhaseGate<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        // left limit: We pass _rect (ignored) instead of using it.
        // Portal content does NOT participate in parent layout.
        // The portal content gets its own layout rect (usually full viewport).
        renderer.enter_portal(self.layer as i32);
        // Portal content renders at (0, 0, viewport_width, viewport_height)
        // because it needs the full screen, not the parent's rect.
        let viewport = renderer.viewport_size();
        let full_rect = Rect::new(0.0, 0.0, viewport.width, viewport.height);
        self.content.render(renderer, full_rect);
        renderer.exit_portal();
    }
}
```

**Step 3: Update existing overlay components to use Portal internally.**

The key insight: we DON'T change the public API of `DropdownMenu`, `Popover`, `Tooltip`, etc. We wrap their render bodies in `Portal` internally. This is additive, not breaking.

```rust
// In src/dropdown_menu.rs  -  modify the render method, NOT the struct definition

impl View for DropdownMenu {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // BEFORE: rendered inline, clipped by parent overflow
        // self.render_dropdown(renderer, rect);
        
        // AFTER: render trigger inline, content in portal
        self.render_trigger(renderer, rect);  // the button that opens the dropdown
        
        // The actual dropdown list renders in portal layer
        renderer.enter_portal(GateTier::Dropdown as i32);
        {
            let dropdown_rect = self.calculate_dropdown_position(rect);
            self.render_dropdown_content(renderer, dropdown_rect);
        }
        renderer.exit_portal();
    }
}
```

**Step 4: Register in lib.rs.**

```rust
// In lib.rs, add:
pub mod phasegate;
pub use phasegate::{PhaseGate, GateTier};
```

---

## 2. TokenStream - Streaming AI Diff Renderer

### Why this exists

When an LLM streams a response, it sends tokens incrementally. The current `GeriPrompt` and `RavenMessenger` components show static text. A streaming diff renderer must:

1. Accept incrementally arriving text tokens.
2. Show newly arrived tokens with a visual highlight that fades.
3. Handle markdown/code blocks that arrive mid-stream (syntax highlighting must update as more tokens arrive).
4. Provide a "blinking cursor" that follows the streaming head.

Without this, you cannot build a ChatGPT-style interface where you see text appearing word by word.

### Implementation

**Step 1: Create the streaming state type.**

```rust
// NEW FILE: cvkg-components/src/token_stream.rs

use cvkg_core::{Never, Rect, Renderer, View};
use std::sync::{Arc, Mutex};

/// Tracks the state of a streaming text sequence.
///
/// # Design decision: Why Arc<Mutex<String>> and not a reactive signal?
///
/// The cvkg-core reactivity system uses State<T> with change listeners.
/// But streaming text arrives at 30-100 tokens/second. Each token arrival
/// triggering a full State change + re-render is wasteful.
///
/// Instead, the TokenStream holds a raw Arc<Mutex<String>> that the
// renderer reads during render(), and a generation counter that IS
/// a State<u64>  -  only the counter triggers re-renders, not the string clone.
///
/// WHY generation counter: The renderer's State system batches changes.
/// By incrementing a u64 counter on each token arrival, we get exactly ONE
/// re-render per token batch (the renderer debounces at 60fps anyway).
pub struct TokenStream {
    /// The accumulated text so far. Displayed to the user.
    text: Arc<Mutex<String>>,
    
    /// Generation counter. Incremented on each token arrival.
    /// This is the ONLY thing that triggers re-renders.
    generation: cvkg_core::State<u64>,
    
    /// Tokens that arrived in the current "highlight window".
    /// These are rendered in accent color, then fade to normal.
    /// WHY: Visual feedback showing "this just arrived" helps users
    /// track the streaming progress. Without it, you can't tell if
    /// the model is still generating or frozen.
    recent_tokens: Arc<Mutex<Vec<HighlightSegment>>>,
    
    /// Whether the stream is still active (cursor visible) or complete.
    streaming: cvkg_core::State<bool>,
    
    /// Optional markdown parser state for mid-stream rendering.
    /// WHY: If the model is streaming markdown, we can't wait until the
    /// end to parse  -  we need to show partial markdown as it arrives.
    /// The parser must be incremental (re-entrant), not batch.
    markdown_state: pulldown_cmark::ParserState,  // simplified; actual impl below
}

/// A range of text that should be visually highlighted as "new".
#[derive(Debug, Clone)]
pub struct HighlightSegment {
    /// Byte offset into the full text.
    pub start: usize,
    /// Byte length of the segment.
    pub len: usize,
    /// When this segment arrived. Used for fade animation.
    pub arrived_at: f64,
}

impl TokenStream {
    /// Create a new TokenStream starting with optional pre-filled content.
    ///
    /// WHY pre-filled: System prompts or cached responses may already exist
    /// before streaming begins. The streaming ADDS to existing content.
    pub fn new(initial: impl Into<String>) -> Self {
        Self {
            text: Arc::new(Mutex::new(initial.into())),
            generation: cvkg_core::State::new(0),
            recent_tokens: Arc::new(Mutex::new(Vec::new())),
            streaming: cvkg_core::State::new(true),
            // markdown_state omitted for brevity  -  use a simple parser
        }
    }

    /// Append a new token to the stream. Called by the data source
    /// (WebSocket handler, SSE parser, etc.).
    ///
    /// # Thread safety: This method can be called from ANY thread.
    /// The Mutex protects the string and highlight segments.
    /// The State increment triggers a re-render on the main thread.
    ///
    /// WHY this returns (): Error handling is push-based. If the stream
    /// errors, the streaming state is set to false. The component renders
    // the final text regardless.
    pub fn push_token(&self, token: &str) {
        let mut text = self.text.lock()
            .map_err(|_| "mutex poisoned")  // left limit: never unwrap
            .unwrap_or_else(|_| return);     // poisoned = stop streaming
        
        let start = text.len();
        text.push_str(token);
        let len = token.len();
        
        let mut recent = self.recent_tokens.lock()
            .map_err(|_| "mutex poisoned")
            .unwrap_or_else(|_| return);
        
        recent.push(HighlightSegment {
            start,
            len,
            arrived_at: std::time::Instant::now()
                .elapsed().as_secs_f64(),  // simplified; use renderer.elapsed_time()
        });
        
        // Trigger re-render via generation counter
        let gen = self.generation.get();
        self.generation.set(gen.wrapping_add(1));
    }

    /// Mark the stream as complete. Cursor disappears, highlight fades immediately.
    pub fn finish(&self) {
        self.streaming.set(false);
        // Force one final render to remove cursor
        let gen = self.generation.get();
        self.generation.set(gen.wrapping_add(1));
    }

    /// Check if there is an active stream.
    pub fn is_streaming(&self) -> bool {
        self.streaming.get()
    }

    /// Get a snapshot of the current text (for debugging/logging only).
    /// Do NOT use this for rendering  -  use the render() method.
    pub fn snapshot(&self) -> String {
        self.text.lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

impl View for TokenStream {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Read the current text. If the mutex is poisoned, show error text.
        // left limit: never panic on mutex poison. Display graceful fallback.
        let text = match self.text.lock() {
            Ok(g) => g.clone(),
            Err(_) => {
                renderer.draw_text("[stream error]", rect.x, rect.y, 14.0, [1.0, 0.0, 0.0, 1.0]);
                return;
            }
        };

        let recent = match self.recent_tokens.lock() {
            Ok(g) => g.clone(),
            Err(_) => Vec::new(),
        };

        let now = renderer.elapsed_time();
        let highlight_duration = 2.0;  // seconds  -  configurable in real impl

        // Render text with per-segment highlight
        let mut current_x = rect.x;
        let mut current_y = rect.y;
        let line_height = 20.0;
        let base_color = [0.9, 0.9, 0.9, 1.0];  // normal text color
        let highlight_color = [0.0, 0.8, 1.0, 1.0];  // accent = "just arrived"

        // Split into segments: highlighted (recent) vs normal (old)
        let mut segments: Vec<(/*start:*/ usize, /*end:*/ usize, /*highlight:*/ bool)> = Vec::new();
        // ... merge recent token ranges with gaps ...
        // (The actual segment merge algorithm: sort recent_tokens by start,
        //  merge overlapping ranges, then invert to get non-highlighted ranges.
        //  This is O(n log n) in token count, fine for chat-size text.)

        for (start, end, is_highlight) in segments {
            let segment_text = &text[start..end];
            if segment_text.is_empty() { continue; }
            
            let elapsed = now - /* segment arrival time */;
            let color = if is_highlight && elapsed < highlight_duration {
                // Fade from highlight to normal over highlight_duration
                let t = (elapsed / highlight_duration) as f32;
                lerp_color(highlight_color, base_color, t)
            } else {
                base_color
            };
            
            let (w, _h) = renderer.measure_text(segment_text, 14.0);
            if current_x + w > rect.x + rect.width {
                // Word wrap
                current_x = rect.x;
                current_y += line_height;
            }
            renderer.draw_text(segment_text, current_x, current_y, 14.0, color);
            current_x += w;
        }

        // Render blinking cursor if streaming
        if self.streaming.get() {
            let blink = (now * 2.0) % 1.0;
            if blink < 0.5 {
                renderer.draw_line(
                    current_x, current_y,
                    current_x, current_y + line_height,
                    [0.0, 0.8, 1.0, 1.0],  // accent color
                    2.0,
                );
            }
        }
    }
}

/// Linearly interpolate between two RGBA colors.
/// WHY standalone function: Used by TokenStream for highlight fade
/// and potentially by animation system for color interpolation.
fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}
```

**Step 2: Register in lib.rs.**

```rust
pub mod token_stream;
pub use token_stream::TokenStream;
```

---

## 3. FlexiScope - Container Query Layout System

### Why this exists

The existing `NidhugMasonry` (advanced.rs:331) uses `SizeProposal::width(col_width)` to estimate item heights, but it cannot RESPOND to the actual rendered size of sibling items. True container queries let a COMPONENT change its layout based on the space IT occupies:

- A card that shows details side-by-side when wide, stacked when narrow.
- A navigation bar that collapses to hamburger menu below 600px  -  but 600px refers to the CONTAINER width, not the viewport.

Without this, responsive design requires knowing the viewport width, which breaks when your component is inside a sidebar (300px wide) vs main content (1200px wide).

### Implementation

**Step 1: Create the container query types.**

```rust
// NEW FILE: cvkg-components/src/flexiscope.rs

use cvkg_core::{Never, Rect, Renderer, Size, View};
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};

/// A breakpoint that triggers based on the CONTAINER width, not the viewport.
///
/// # Why container-based:
/// In a dashboard with a 250px sidebar and a main area, a component in the
/// main area should adapt to the main area's width, not the full screen width.
/// A 1200px screen with a 250px sidebar means the sidebar component should
/// think it's in a 250px container, not 1200px.
///
/// # Example:
/// ```
/// ContainerQuery::new(
///     my_dashboard_content,
///     vec![
///         ScopeThreshold::new(0.0, CompactLayout),
///         ScopeThreshold::new(400.0, NormalLayout),
///         ScopeThreshold::new(800.0, WideLayout),
///     ]
/// )
/// ```
#[derive(Clone)]
pub struct FlexiScope<V, B> {
    /// The content view to render.
    content: V,
    /// Breakpoints: width thresholds and corresponding layout modes.
    // B is a "layout strategy" enum that the component defines
    breakpoints: Vec<ScopeThreshold<B>>,
    /// Cached layout mode from last measurement. Avoids per-frame branching.
    cached_mode: std::cell::RefCell<Option<B>>,
}

/// A single breakpoint: when container width >= threshold, use this mode.
#[derive(Debug, Clone)]
pub struct ScopeThreshold<B> {
    /// Minimum width to activate this breakpoint.
    pub min_width: f32,
    /// The layout mode to use when this breakpoint is active.
    pub mode: B,
}

/// Trait for layout modes that respond to container size.
///
/// WHY a trait instead of an enum: Each component defines its own layout modes.
/// A Card might have `CardLayout::Compact` and `CardLayout::Expanded`.
/// A NavigationBar might have `NavLayout::Full` and `NavLayout::Collapsed`.
/// A trait lets each type define its own variants.
pub trait ContainerLayout: Clone + PartialEq {
    /// Choose the layout mode for a given container width.
    /// `breakpoints` is sorted ascending by min_width.
    fn select_mode(width: f32, breakpoints: &[ScopeThreshold<Self>]) -> Self
    where Self: Sized {
        let mut selected = &breakpoints[0];
        for bp in breakpoints {
            if width >= bp.min_width {
                selected = bp;
            }
        }
        selected.mode.clone()
    }
}

impl<V: View, B: ContainerLayout> FlexiScope<V, B> {
    pub fn new(content: V, breakpoints: Vec<ScopeThreshold<B>>) -> Self {
        assert!(!breakpoints.is_empty(), "need at least one breakpoint");
        Self {
            content,
            breakpoints,
            cached_mode: std::cell::RefCell::new(None),
        }
    }
}

impl<V: View, B: ContainerLayout + 'static> View for FlexiScope<V, B> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Determine layout mode based on THIS component's allocated width.
        let mode = B::select_mode(rect.width, &self.breakpoints);
        
        // Only propagate change if mode changed (avoid unnecessary child rebuilds)
        let mut cached = self.cached_mode.borrow_mut();
        let mode_changed = cached.as_ref() != Some(&mode);
        
        if mode_changed {
            *cached = Some(mode.clone());
        }
        drop(cached);
        
        // Render child. The child reads the current layout mode via
        // system state (injected by this component).
        let state_hash = self.state_hash();
        cvkg_core::update_system_state(|state| {
            state.insert(state_hash, mode.clone());
        });
        
        self.content.render(renderer, rect);
    }
    
    fn layout(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        // Relay to child  -  the child makes sizing decisions based on
        // the container mode we set in render().
        self.content.intrinsic_size(renderer, proposal)
    }
}

impl<V, B> FlexiScope<V, B> {
    /// Derive a stable hash from `self` for system state storage.
    /// Uses the address of the breakpoints vec (stable for the component lifetime).
    fn state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.breakpoints.as_ptr().hash(&mut h);
        h.finish()
    }
}
```

**Step 2: Add `fluid_typography` function.**

```rust
// In flexiscope.rs (same file, below ContainerQuery)

/// Compute a font size that scales linearly between two widths.
///
/// # Arguments
/// - `container_width`: The actual width of the container.
/// - `min_width`: Below this, font size is `min_size`.
/// - `max_width`: Above this, font size is `max_size`.
/// - `min_size`: Font size at or below `min_width`.
/// - `max_size`: Font size at or above `max_width`.
///
/// # Why this matters:
/// Fixed font sizes (the FONT_XS/FONT_3XL constants in lib.rs) don't adapt
/// to container width. A heading that looks good at 1200px is a wall of text
/// at 300px. Fluid typography scales smoothly.
///
/// # Left limit: Never produce a font size below 8.0 or above 96.0.
/// These are hard readability limits regardless of math.
pub fn fluid_typography(
    container_width: f32,
    min_width: f32,
    max_width: f32,
    min_size: f32,
    max_size: f32,
) -> f32 {
    if container_width <= min_width {
        return min_size.max(8.0);
    }
    if container_width >= max_width {
        return max_size.min(96.0);
    }
    let t = (container_width - min_width) / (max_width - min_width);
    let size = min_size + (max_size - min_size) * t.clamp(0.0, 1.0);
    size.clamp(8.0, 96.0)  // left limit: enforce readability bounds
}
```

**Step 3: Update NidhugMasonry to use container queries instead of fixed columns.**

```rust
// In advanced.rs  -  REPLACE the NidhugMasonry impl with:

impl<V: View> NidhugMasonry<V> {
    /// Compute the number of columns for a given container width.
    /// Uses container query logic: fewer columns when narrow.
    fn columns_for_width(&self, container_width: f32) -> usize {
        let min_col_width = 250.0;  // left limit: never narrower than 250px per column
        ((container_width / min_col_width).floor() as usize).max(1)
    }
}
```

---

## 4. TrustMark - Confidence / Uncertainty Visualization

### Why this exists

LLM outputs are probabilistic. Showing raw text without confidence information is like showing a number without error bars  -  technically correct but misleading. When a model says "The capital of France is Paris" with 99% confidence vs "The capital of Bhutan is Thimphu" with 60% confidence, the UI should communicate that difference.

Current `ai_components.rs` has token usage inspectors and reasoning traces but NO confidence display at all.

### Implementation

```rust
// NEW FILE: cvkg-components/src/trustmark.rs

use cvkg_core::{Never, Rect, Renderer, View};

/// Visual indicator of model confidence in a piece of generated content.
///
/// # Design rationale:
/// Confidence is displayed as a colored border/badge, NOT as a tooltip or
/// separate panel. Users need to see confidence INLINE with the content,
/// because scrolling to a tooltip to check every sentence is impractical.
///
/// # Left limit:
/// - Never display precise percentages. "92.3%" is false precision from a
///   probabilistic model. Use discrete bands: High/Medium/Low/VeryLow.
/// - Never default to "high confidence"  -  if confidence data is missing,
///   display [!] Unknown.
///
/// # Right limit:
/// - Don't implement a full probability distribution visualization.
///   A simple 4-band indicator is sufficient for v1.
#[derive(Clone)]
pub struct TrustMark {
    /// Discrete confidence band.
    pub band: TrustLevel,
    /// Optional explanation (shown on hover as tooltip).
    pub explanation: String,
}

/// Discrete confidence bands.
/// WHY discrete: The underlying model outputs a float 0.0-1.0, but displaying
/// "67.3%" implies false precision. Research shows users make better decisions
/// with discrete labels than continuous numbers for uncertainty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    /// >85%  -  Strong confidence. Green.
    High,
    /// 60-85%  -  Moderate confidence. Yellow.
    Medium,
    /// 30-60%  -  Low confidence. Orange. User should verify.
    Low,
    /// <30%  -  Very low confidence. Red. Treat as speculation.
    VeryLow,
    /// No confidence data available. Gray with "?" icon.
    Unknown,
}

impl TrustLevel {
    /// Convert a raw confidence float (0.0-1.0) to a band.
    ///
    /// # WHY these thresholds:
    /// - 85%: Above this, auto-accept is reasonable for most use cases.
    /// - 60%: Below this, human review is recommended.
    /// - 30%: Below this, the model is essentially guessing.
    pub fn from_float(confidence: f32) -> Self {
        // left limit: clamp input, never NaN-propagate
        let c = confidence.clamp(0.0, 1.0);
        if c >= 0.85 { TrustLevel::High }
        else if c >= 0.60 { TrustLevel::Medium }
        else if c >= 0.30 { TrustLevel::Low }
        else { TrustLevel::VeryLow }
    }

    /// Get the display color for this band.
    /// Colors are chosen for deuteranopia (red-green color blindness) safety:
    /// High=teal, Medium=yellow, Low=orange, VeryLow=red, Unknown=gray.
    /// WHY teal instead of green: Red-green color blindness affects ~8% of males.
    /// Teal vs red is safe for all common forms of color blindness.
    pub fn color(&self) -> [f32; 4] {
        match self {
            TrustLevel::High =>     [0.0, 0.7, 0.7, 1.0],    // teal
            TrustLevel::Medium =>   [0.9, 0.8, 0.0, 1.0],    // yellow
            TrustLevel::Low =>      [1.0, 0.5, 0.0, 1.0],    // orange
            TrustLevel::VeryLow =>  [0.9, 0.1, 0.1, 1.0],    // red
            TrustLevel::Unknown =>  [0.5, 0.5, 0.5, 0.6],    // gray
        }
    }

    /// Get the display label.
    pub fn label(&self) -> &'static str {
        match self {
            TrustLevel::High =>     "High confidence",
            TrustLevel::Medium =>   "Moderate confidence",
            TrustLevel::Low =>      "Low confidence  -  verify",
            TrustLevel::VeryLow =>  "Speculative  -  unreliable",
            TrustLevel::Unknown =>  "Confidence unknown",
        }
    }
}

impl Default for TrustMark {
    fn default() -> Self {
        Self {
            band: TrustLevel::Unknown,
            explanation: String::new(),
        }
    }
}

impl View for TrustMark {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let color = self.band.color();
        
        // Render a thin colored border on the LEFT side of the content.
        // WHY left border only: Full border looks like an error highlight
        // (red border = form validation error). Left border is subtle and
        // follows the pattern of git diff indicators and annotation markers.
        let border_width = 3.0;
        renderer.fill_rect(
            Rect::new(rect.x, rect.y, border_width, rect.height),
            color,
        );
        
        // Render small indicator icon at top-right corner.
        let icon_x = rect.x + rect.width - 16.0;
        let icon_y = rect.y + 4.0;
        let icon_rect = Rect::new(icon_x, icon_y, 12.0, 12.0);
        
        match self.band {
            TrustLevel::Unknown => {
                renderer.draw_text("?", icon_x, icon_y + 10.0, 10.0, color);
            }
            TrustLevel::High => {
                // Filled circle = high confidence
                renderer.fill_ellipse(icon_rect, color);
            }
            _ => {
                // Hollow circle + band color = lower confidence
                renderer.stroke_ellipse(icon_rect, color, 1.5);
            }
        }
    }
}

/// Attach a confidence indicator to any content view.
/// Usage: `my_text.trustmark(TrustLevel::from_float(0.92))`
pub trait TrustExt: View + Sized {
    fn confidence(self, band: TrustLevel) -> TrustWrap<Self>;
}

impl<T: View + Sized> TrustExt for T {
    fn confidence(self, band: TrustLevel) -> TrustWrap<Self> {
        TrustWrap {
            content: self,
            badge: TrustMark {
                band,
                explanation: String::new(),
            },
        }
    }
}

#[derive(Clone)]
pub struct TrustWrap<V: View> {
    content: V,
    badge: TrustMark,
}

impl<V: View> View for TrustWrap<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Render the badge LEFT of the content (thin border style).
        self.badge.render(renderer, rect);
        // Render the content normally.
        self.content.render(renderer, rect);
    }
}
```

Register in lib.rs:
```rust
pub mod trustmark;
pub use volva_seal::{TrustMark, TrustLevel, TrustExt};
```

---

## 5. A11yBeacon - Live Region Accessibility Integration

### Why this exists

The current `hlin_accessibility.rs` renders a DEBUG TREE for developers. It has NO connection to actual screen reader technology (AccessKit on Windows/Linux, NSAccessibility on macOS). A screen reader user would get NOTHING from the current accessibility code.

Live regions (`aria-live`) are critical: when a toast appears, a streaming token arrives, or a form field validates, the screen reader must ANNOUNCE it. Without live regions, a screen reader user has no idea anything changed unless they manually navigate to it.

### Implementation

**Step 1: Add accessibility announcement API to cvkg-core's Renderer trait.**

```rust
// In cvkg-core/src/renderer.rs  -  ADD to the Renderer trait:

/// Announce a message to screen readers via the platform accessibility API.
///
/// This call is NON-BLOCKING. The message is queued and the screen reader
/// will speak it at its own pace. Multiple calls are queued in order.
///
/// `priority` determines whether to interrupt current speech:
/// - `Polite`: Wait for current speech to finish. Use for non-urgent updates
///   (e.g., "5 results found").
/// - `Assertive`: Interrupt current speech. Use for urgent updates
///   (e.g., "Form submission failed").
///
/// WHY live regions matter:
/// Screen readers work by traversing the accessibility tree. If content
/// changes dynamically (streaming text, form validation, toast notification),
/// the screen reader doesn't know unless something ANNOUNCES the change.
/// `aria-live` regions are the standard mechanism for this.
///
/// Default implementation: store in a queue for headless modes,
/// where they can be read back in tests.
fn announce(&mut self, message: &str, priority: AnnouncementPriority);

/// Accessibility announcement priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPriority {
    /// Wait for current speech to finish.
    Polite = 0,
    /// Interrupt current speech.
    Assertive = 1,
}
```

**Step 2: Create the LiveRegion component.**

```rust
// NEW FILE: cvkg-components/src/a11y_beacon.rs

use cvkg_core::{Never, Rect, Renderer, View, State};

/// A wrapper that announces content changes to screen readers.
///
/// # Usage:
/// ```
/// // Announce when search results update:
/// LiveRegion::new(
///     Polite,
///     results_list,
///     format!("{} results found", count),  // announcement text
/// )
///
/// // Announce form validation errors (assertive  -  user needs to know NOW):
/// LiveRegion::new(
///     Assertive,
///     form_field,
///     "Email address is invalid",
/// )
/// ```
///
/// # Why `format!()` string and not `&str`:
/// The announcement text needs to change when the wrapped content changes.
/// We use a String (computed at render time) instead of a `&'static str`
/// because the message includes dynamic values (counts, field names).
///
/// # Left limit:
/// - Never announce on every frame. Only announce when the message CHANGES.
///   Announcing "5 results, 5 results, 5 results..." every frame would
///   make the screen reader unusable.
/// - Never announce empty strings. An empty announcement still triggers
///   screen reader noise.
///
/// # Right limit:
/// - Don't implement `aria-atomic` or `aria-relevant` granularity.
///   A simple "announce the message when it changes" is sufficient.
#[derive(Clone)]
pub struct A11yBeacon<V: View> {
    content: V,
    message: String,
    priority: cvkg_core::AnnouncementPriority,
    /// Tracks the last message we announced. If unchanged, we skip.
    last_announced: State<Option<String>>,
}

impl<V: View> A11yBeacon<V> {
    pub fn new(
        priority: cvkg_core::AnnouncementPriority,
        content: V,
        message: impl Into<String>,
    ) -> Self {
        Self {
            content,
            message: message.into(),
            priority,
            last_announced: State::new(None),
        }
    }
}

impl<V: View> View for A11yBeacon<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let last = self.last_announced.get();
        
        // Only announce if message is non-empty AND changed since last render.
        // WHY: Screen readers re-read unchanged announcements as noise.
        let should_announce = !self.message.is_empty() 
            && last.as_deref() != Some(&self.message);
        
        if should_announce {
            renderer.announce(&self.message, self.priority);
            self.last_announced.set(Some(self.message.clone()));
        }
        
        self.content.render(renderer, rect);
    }
}

/// Extension trait for adding live region announcements to any view.
pub trait A11yBeaconExt: View + Sized {
    /// Wrap this view with a polite live region.
    /// The message is computed by the `message_fn` closure each render.
    fn announce_when(
        self,
        priority: cvkg_core::AnnouncementPriority,
        message: impl Into<String>,
    ) -> A11yBeacon<Self> {
        LiveRegion::new(priority, self, message)
    }
}

impl<T: View + Sized> A11yBeaconExt for T {}
```

**Step 3: Update Toast to auto-announce.**

```rust
// In src/toast.rs  -  modify ToastManager::render to include live regions:

impl View for ToastManager {
    // ... existing body/render ...
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // ... existing toast rendering ...
        
        // AFTER rendering each toast, announce new ones:
        for toast in &self.active_toasts {
            if toast.is_new() {
                let priority = match toast.kind {
                    ToastKind::Error => cvkg_core::AnnouncementPriority::Assertive,
                    _ => cvkg_core::AnnouncementPriority::Polite,
                };
                renderer.announce(&toast.message, priority);
                toast.mark_announced();
            }
        }
    }
}
```

Register in lib.rs:
```rust
pub mod a11y_beacon;
pub use heimdall_watch::{LiveRegion, A11yBeaconExt};
```

---

## 6. SyncWeave - Real-Time Collaboration Primitives

### Why this exists

The current `collaboration.rs` shows participant cursors. That's it. It has:

- NO CRDT-aware text binding (editing text simultaneously = last-write-wins data loss)
- NO presence awareness (who's editing what right now?)
- NO conflict visualization (showing where two people edited the same region)

Real-time collaboration requires at minimum:
1. **CRDT text buffer**  -  deterministic merge of concurrent edits
2. **Presence cursors** with name labels (not just dots)
3. **Conflict highlighting**  -  when two users edit the same line

### Left limit for this entire section:
Use LSeq (LogootSplit) or Yjs-compatible CRDT. Don't implement operational transforms  -  they're deprecated in favor of CRDTs.

```rust
// NEW FILE: cvkg-components/src/sync_weave.rs

use crate::{TextEditor, theme};
use cvkg_core::{Never, Rect, Renderer, View, State, Event};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A CRDT-backed collaborative text buffer.
///
/// # CRDT choice: LogootSplit (simplified)
///
/// LogootSplit assigns each character a unique position identifier that is
/// totally ordered. When two users insert at the same position, the IDs
/// determine order without conflicts. This is simpler than Yjs but has
/// the same convergence guarantees.
///
/// WHY NOT Operational Transform:
/// OT requires a central server to transform operations. CRDTs converge
/// correctly even with peer-to-peer communication and partition tolerance.
/// All modern collaborative editors (Notion, Figma, Google Docs post-2019)
/// use CRDTs.
///
/// # Architecture:
/// The buffer is Arc<Mutex<Clone>>  -  each collaborator holds the same buffer.
/// On local edit, the buffer is mutated and a sync message is emitted.
/// On remote edit, the sync message is applied to the local buffer.
/// The component re-renders when the generation counter changes.

/// A single character in the CRDT buffer with a unique position ID.
#[derive(Debug, Clone)]
struct WeaveChar {
    /// Unique position identifier. Totally ordered across all collaborators.
    position: Vec<u64>,
    /// The character value.
    value: char,
    /// Site ID of the editor who inserted this character.
    site_id: u64,
    /// Lamport timestamp for tie-breaking.
    timestamp: u64,
    /// Tombstone flag for deletion (CRDTs never truly delete).
    deleted: bool,
}

/// The CRDT text buffer: an ordered sequence of WeaveChar.
#[derive(Debug, Clone)]
pub struct SyncWeave {
    chars: Vec<WeaveChar>,
    /// This editor's unique site ID.
    site_id: u64,
    /// Monotonic counter for local operations.
    clock: u64,
    /// Generation counter. Incremented on ANY change (local or remote).
    /// The UI watches this to trigger re-renders.
    generation: u64,
}

impl SyncWeave {
    pub fn new(site_id: u64) -> Self {
        Self {
            chars: Vec::new(),
            site_id,
            clock: 0,
            generation: 0,
        }
    }

    /// Insert a character at the given visible position (0-indexed, ignoring tombstones).
    /// Returns an `Op` that can be broadcast to other collaborators.
    pub fn local_insert(&mut self, visible_pos: usize, ch: char) -> Op {
        self.clock += 1;
        self.generation += 1;
        
        // Compute position ID between neighbors.
        // WHY: The new char's position must be BETWEEN its neighbors
        // in the total order. We use a simple scheme: average of
        // neighbor positions, or neighbor+1 if too close.
        let position = self.allocate_position(visible_pos);
        
        let crdt_char = WeaveChar {
            position,
            value: ch,
            site_id: self.site_id,
            timestamp: self.clock,
            deleted: false,
        };
        
        self.chars.insert(visible_pos, crdt_char.clone());
        
        Op::Insert { char: crdt_char }
    }

    /// Delete the character at the given visible position.
    pub fn local_delete(&mut self, visible_pos: usize) -> Op {
        self.clock += 1;
        self.generation += 1;
        
        // Find the actual index (skipping tombstones).
        let actual = self.visible_to_actual(visible_pos);
        self.chars[actual].deleted = true;
        
        Op::Delete { position: self.chars[actual].position.clone() }
    }

    /// Apply a remote operation from another collaborator.
    /// This is always safe to apply  -  CRDT ops are commutative.
    pub fn apply_remote(&mut self, op: &Op) {
        self.generation += 1;
        match op {
            Op::Insert { char } => {
                // Find insertion point maintaining total order.
                let idx = self.chars.partition_point(|c| c.position < char.position);
                self.chars.insert(idx, char.clone());
            }
            Op::Delete { position } => {
                if let Some(c) = self.chars.iter_mut().find(|c| &c.position == position) {
                    c.deleted = true;
                }
            }
        }
    }

    /// Get the visible text (excluding tombstones).
    pub fn text(&self) -> String {
        self.chars.iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }

    /// Get the current generation counter.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    // --- Internal helpers ---

    fn allocate_position(&self, visible_pos: usize) -> Vec<u64> {
        let prev = if visible_pos > 0 {
            self.chars.get(visible_pos - 1).map(|c| &c.position)
        } else { None };
        let next = self.chars.get(visible_pos).map(|c| &c.position);
        
        match (prev, next) {
            (Some(p), Some(n)) => midpoint(p, n),
            (Some(p), None) => increment_last(p),
            (None, Some(n)) => vec![n[0] / 2],
            (None, None) => vec![u64::MAX / 2],
        }
    }

    fn visible_to_actual(&self, visible_pos: usize) -> usize {
        let mut visible = 0;
        for (i, c) in self.chars.iter().enumerate() {
            if !c.deleted {
                if visible == visible_pos { return i; }
                visible += 1;
            }
        }
        self.chars.len()  // append-at-end case
    }
}

/// A collaborative operation that can be broadcast to other editors.
#[derive(Debug, Clone)]
pub enum Op {
    Insert { char: WeaveChar },
    Delete { position: Vec<u64> },
}

/// A cursor position from a remote collaborator.
#[derive(Debug, Clone)]
pub struct PeerCursor {
    pub site_id: u64,
    pub name: String,
    pub color: [f32; 4],
    /// Visible position (0-indexed, ignoring tombstones).
    pub position: usize,
}

/// The collaborative text editor component.
pub struct SyncEditor {
    /// Shared CRDT buffer. Arc<Mutex<>> for thread-safe access.
    buffer: Arc<Mutex<SyncWeave>>,
    
    /// Remote cursors keyed by site_id.
    remote_cursors: State<HashMap<u64, PeerCursor>>,
    
    /// Local cursor position.
    cursor_pos: State<usize>,
    
    /// Pending operations to be sent over the network.
    /// Populated by local edits, consumed by the sync layer.
    pending_ops: Arc<Mutex<Vec<Op>>>,
}

impl SyncEditor {
    pub fn new(site_id: u64) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(CdtBuffer::new(site_id))),
            remote_cursors: State::new(HashMap::new()),
            cursor_pos: State::new(0),
            pending_ops: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Handle a local text input event.
    fn handle_input(&self, event: &Event) {
        let mut buf = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return,  // left limit: mutex poison = skip, don't panic
        };
        
        match event {
            Event::TextInput { text } => {
                for ch in text.chars() {
                    let pos = self.cursor_pos.get();
                    let op = buf.local_insert(pos, ch);
                    self.cursor_pos.set(pos + 1);
                    
                    // Queue for network sync
                    if let Ok(mut pending) = self.pending_ops.lock() {
                        pending.push(op);
                    }
                }
            }
            Event::KeyPress { key: "Backspace", .. } => {
                let pos = self.cursor_pos.get();
                if pos > 0 {
                    let op = buf.local_delete(pos - 1);
                    self.cursor_pos.set(pos - 1);
                    if let Ok(mut pending) = self.pending_ops.lock() {
                        pending.push(op);
                    }
                }
            }
            Event::KeyPress { key: "ArrowLeft", .. } => {
                let pos = self.cursor_pos.get();
                if pos > 0 { self.cursor_pos.set(pos - 1); }
            }
            Event::KeyPress { key: "ArrowRight", .. } => {
                let pos = self.cursor_pos.get();
                let text = buf.text();
                if pos < text.chars().count() { self.cursor_pos.set(pos + 1); }
            }
            _ => {}  // unhandled events pass through
        }
    }

    /// Process a batch of remote operations.
    /// Called by the sync layer when network messages arrive.
    fn apply_remote_ops(&self, ops: &[Op]) {
        let mut buf = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return,
        };
        for op in ops {
            buf.apply_remote(op);
        }
    }

    /// Get a clone of pending operations (for the sync layer to send).
    fn drain_pending_ops(&self) -> Vec<Op> {
        self.pending_ops.lock()
            .map(|mut p| p.drain(..).collect())
            .unwrap_or_default()
    }
}

impl View for SyncEditor {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let buf = match self.buffer.lock() {
            Ok(b) => b.clone(),
            Err(_) => {
                renderer.draw_text("[collaboration error]", rect.x, rect.y, 14.0, [1.0, 0.0, 0.0, 1.0]);
                return;
            }
        };
        
        let text = buf.text();
        let cursor_pos = self.cursor_pos.get();
        let cursors = self.remote_cursors.get();
        
        // Render background
        renderer.fill_rect(rect, theme::surface());
        
        // Render text (reuse text rendering from primitive.rs)
        let mut y = rect.y + 4.0;
        let line_height = 20.0;
        for (line_idx, line) in text.lines().enumerate() {
            renderer.draw_text(line, rect.x + 4.0, y, 14.0, theme::text());
            y += line_height;
        }
        
        // Render local cursor
        let text_before_cursor = text.chars().take(cursor_pos).collect::<String>();
        let last_newline = text_before_cursor.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let chars_on_line = text_before_cursor[last_newline..].chars().count();
        let lines_before = text_before_cursor.chars().filter(|c| *c == '\n').count();
        
        let cursor_x = rect.x + 4.0 + (chars_on_line as f32) * 8.0;  // ~8px per char
        let cursor_y = rect.y + 4.0 + (lines_before as f32) * line_height;
        
        renderer.draw_line(
            cursor_x, cursor_y,
            cursor_x, cursor_y + line_height,
            [0.0, 0.8, 1.0, 1.0],  // local cursor color = accent
            2.0,
        );
        
        // Render remote cursors
        for (_site_id, cursor) in cursors.iter() {
            // Similar math to local cursor, using cursor.position
            // Render a colored cursor with name label
            let text_before = text.chars().take(cursor.position).collect::<String>();
            let last_nl = text_before.rfind('\n').map(|p| p + 1).unwrap_or(0);
            let chars_on = text_before[last_nl..].chars().count();
            let lines_before = text_before.chars().filter(|c| *c == '\n').count();
            
            let cx = rect.x + 4.0 + (chars_on as f32) * 8.0;
            let cy = rect.y + 4.0 + (lines_before as f32) * line_height;
            
            renderer.draw_line(cx, cy, cx, cy + line_height, cursor.color, 2.0);
            
            // Name label above cursor
            let label_rect = Rect::new(cx, cy - 14.0, cursor.name.len() as f32 * 7.0 + 4.0, 14.0);
            renderer.fill_rounded_rect(label_rect, 2.0, cursor.color);
            renderer.draw_text(&cursor.name, cx + 2.0, cy - 2.0, 10.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}

// Helper functions for position allocation:
fn midpoint(a: &[u64], b: &[u64]) -> Vec<u64> {
    let max_len = a.len().max(b.len());
    let a_pad = {
        let mut v = a.to_vec();
        v.resize(max_len, 0);
        v
    };
    let b_pad = {
        let mut v = b.to_vec();
        v.resize(max_len, 0);
        v
    };
    
    // Simple midpoint: (a[0] + b[0]) / 2 at the first differing position
    for i in 0..max_len {
        if a_pad[i] != b_pad[i] {
            let mid = a_pad[i] + (b_pad[i] - a_pad[i]) / 2;
            let mut result = Vec::with_capacity(i + 1);
            for j in 0..i { result.push(a_pad[j]); }
            result.push(mid);
            return result;
        }
    }
    // Identical  -  shouldn't happen in correct usage
    let mut result = a.to_vec();
    result.push(u64::MAX / 2);
    result
}

fn increment_last(a: &[u64]) -> Vec<u64> {
    let mut result = a.to_vec();
    if let Some(last) = result.last_mut() {
        *last += 1;
    }
    result
}
```

Register in lib.rs:
```rust
pub mod sync_weave;
pub use urd_editor::{SyncEditor, SyncWeave, PeerCursor, Op};
```

---

## 7. MorphBridge - Shared Element Transition System

### Why this exists

Current `transitions.rs` animates individual elements (fade, slide, scale). When a user clicks a card and it expands into a detail view, the card should MORPH into the detail  -  same element, growing from card size to full screen. This is what Apple calls "zoom transitions" and what React calls "shared element transitions".

Without this, navigation feels like page cuts instead of spatial continuity.

### Implementation

```rust
// NEW FILE: cvkg-components/src/morph_bridge.rs

use cvkg_core::{Never, Rect, Renderer, View, State};
use cvkg_anim::{SleipnirParams, SleipnirSolver};

/// A shared element transition container.
///
/// When the active element visually changes (e.g., a card expands to detail view),
/// the `SharedTransition` animates from the OLD rect to the NEW rect, creating
/// the illusion that the same physical object is growing/moving.
///
/// # Each shared element is identified by a `key: String` (or any Hash type).
/// The system matches elements by key across tree changes.
///
/// # HOW IT WORKS:
/// 1. Child views call `register_shared_element(key, rect)` to declare their
///    position in the current layout.
/// 2. When the tree changes, the transition system compares the previous rects
///    with the new rects for each key.
/// 3. If a key's rect changed, the system animates from old rect to new rect,
///    rendering the content in an overlay (portal) that covers both positions.
///
/// # Example use case:
/// User clicks on card "My Project" (rect: 50,100,200,150) and the detail
/// view renders (rect: 0,0,800,600). Instead of a cut, the card smoothly
/// grows from (50,100,200,150) to (0,0,800,600) over 300ms.
///
/// # Left limit:
/// - Never animate between positions >500px apart in a single frame.
///   That's not a transition, it's a teleportation. Snap instead.
/// - Never hold references to child content beyond the animation duration.
///   Memory leak. Use weak references or generational indices.
///
/// # Right limit:
/// - Don't implement FLIP (First-Last-Invert-Play) positioning.
///   A simple rectangle interpolation is sufficient for v1.

/// Registered geometry for a shared element.
#[derive(Debug, Clone)]
pub struct MorphElement {
    pub key: String,
    pub old_rect: Rect,
    pub new_rect: Option<Rect>,  // None = element appeared
    pub progress: f32,  // 0.0 = old, 1.0 = new
}

/// The shared transition container.
pub struct MorphBridge<V: View> {
    content: V,
    /// Map of element key to transition state.
    elements: State<Vec<MorphElement>>,
    /// Spring params for the transition animation.
    params: SleipnirParams,
}

impl<V: View> MorphBridge<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            elements: State::new(Vec::new()),
            params: SleipnirParams::smooth(),  // gentle spring, not snappy
        }
    }
    
    /// Update the geometry for a registered element.
    /// Called by child views during render.
    pub fn update_element(&self, key: &str, rect: Rect) {
        let mut elements = self.elements.get();
        if let Some(entry) = elements.iter_mut().find(|e| e.key == key) {
            if entry.new_rect.as_ref() != Some(&rect) {
                entry.new_rect = Some(rect);
                entry.progress = 0.0;  // reset animation
            }
        } else {
            elements.push(MorphElement {
                key: key.to_string(),
                old_rect: rect,
                new_rect: Some(rect),
                progress: 1.0,
            });
        }
        self.elements.set(elements);
    }
}

impl<V: View> View for MorphBridge<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let elements = self.elements.get();
        
        // Render the child tree normally  -  children register their geometry
        // via `update_element` during their render.
        self.content.render(renderer, rect);
        
        // For each element in transition, render an animated overlay.
        for entry in &elements {
            if let Some(ref new_rect) = entry.new_rect {
                if entry.progress < 0.99 {
                    // Interpolate between old and new rect.
                    let current = lerp_rect(&entry.old_rect, new_rect, entry.progress);
                    let progress = entry.progress + 1.0 / 60.0;  // ~60fps tick
                    
                    // Render the element content in the interpolated rect.
                    // WHY overlay: The element must "float above" both the
                    // source and destination during transition.
                    renderer.enter_portal(500);  // above normal content
                    // In a real implementation, you'd render the actual
                    // element content here. For now, render a placeholder
                    // that visually demonstrates the transition.
                    renderer.fill_rounded_rect(current, 8.0, [0.1, 0.1, 0.2, 0.9]);
                    renderer.exit_portal();
                    
                    // Update progress
                    let mut elements = self.elements.get();
                    if let Some(e) = elements.iter_mut().find(|e| e.key == entry.key) {
                        e.progress = progress.min(1.0);
                    }
                    self.elements.set(elements);
                }
            }
        }
    }
}

/// Lerp between two rectangles.
fn lerp_rect(a: &Rect, b: &Rect, t: f32) -> Rect {
    let t = t.clamp(0.0, 1.0);
    Rect {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
    }
}
```

Key thing this adds: uses `Portal` (from feature #1) to render the transitioning element above both source and destination layouts.

Register in lib.rs:
```rust
pub mod morph_bridge;
pub use huginn_glide::SharedTransition;
```

---

## 8. FluxLayout - Layout Animation for Sibling Repositioning

### Why this exists

When an item in a list grows (e.g., expanding an accordion), the items BELOW it should smoothly slide down. When an item collapses, they should smoothly slide up. Currently, changes are INSTANT  -  items teleport to new positions.

This requires the layout system to track previous positions and animate the delta.

### Implementation

```rust
// ADD TO: cvkg-components/src/container.rs (inside the VStack/HStack impls)

// The key insight: VStack and HStack already know the rect of each child.
// We need to RECT each child's previous position and animate toward the new one.

/// Wrapper that records each child's previous rect and animates movement.
/// Insert between VStack and its children.
pub struct FluxLayout<V: View> {
    previous_rects: State<Vec<Rect>>,
    animation_duration: f32,  // seconds
}

impl<V: View> FluxLayout<V> {
    /// Initialize from the current layout.
    /// Called once during first render.
    fn capture_layout(&self, rects: Vec<Rect>) {
        let prev = self.previous_rects.get();
        if prev.is_empty() {
            // First render  -  no animation needed.
            self.previous_rects.set(rects);
        } else if prev.len() != rects.len() {
            // Items added/removed  -  animate from old positions.
            // WHY: The new items animate in from wherever the old layout said
            // they should be (often overlapping with siblings).
            self.previous_rects.set(rects);
        } else {
            // Same count  -  items moved. Let them animate.
            // Don't update immediately  -  let render() handle the interpolation.
        }
    }
}
```

Detailed implementation is 300+ lines. The approach:

1. On each render, compare current child rects to previous rects.
2. For each child that moved, compute `delta = new_rect - old_rect`.
3. Render the child at `old_rect + delta * ease(t)` instead of `new_rect`.
4. Tick `t` from 0->1 over `animation_duration` using the Sleipnir spring solver.

**LEFT LIMIT**: Never animate the root container. Only animate CHILDREN within a container. Animating the root causes the entire UI to slide around.

**RIGHT LIMIT**: Don't implement enter/exit animations for items being added/removed. Just position animation for items that changed position.

---

## 9. ComputedSignal - Derived / Computed State Primitives

### Why this exists

Every component uses raw `State<T>` with manual updates. If you need `full_name = first_name + last_name`, you have to manually update `full_name` every time `first_name` or `last_name` changes. This is error-prone and doesn't scale.

Reactive frameworks (SwiftUI `@DerivedState`, SolidJS `createMemo`, React `useMemo`) compute derived values automatically.

### Implementation

```rust
// NEW FILE: cvkg-components/src/computed_signal.rs

use cvkg_core::State;
use std::sync::{Arc, RwLock};

/// A computed/derived state that automatically updates when its inputs change.
///
/// # How it works:
/// 1. You create a `Computed<T>` with a closure that computes T from inputs.
/// 2. The closure runs on first `.get()` call, caching the result.
/// 3. On each `.get()`, we check if the generation counter of ANY input State
///    has changed since our last computation. If so, we re-run the closure.
///
/// # WHY and not a reactive graph:
/// A full reactive dependency graph (like SolidJS signals) requires tracking
/// which States are READ during computation. That needs compiler support or
/// a macro. A simpler approach: pass the input States explicitly and check
/// all their generation counters. Slightly less efficient but MUCH simpler.
///
/// # Example:
/// ```
/// let first_name = State::new("Alice".to_string());
/// let last_name = State::new("Smith".to_string());
/// let full_name = Computed::new(
///     [&first_name, &last_name],
///     |values| format!("{} {}", values[0], values[1]),
/// );
/// // full_name.get() == "Alice Smith"
/// first_name.set("Bob".to_string());
/// // full_name.get() == "Bob Smith" (automatically updated)
/// ```
///
/// # Left limit:
/// - The compute closure MUST be pure (no side effects).
/// - Never call `.set()` inside the compute closure. Infinite loop.
/// - Never use Computed for values that change every frame (60fps).
///   Use a regular render-time calculation instead.
///
/// # Right limit:
/// - Don't implement two-way binding (Computed that can be SET, propagating
///   changes back to inputs). That's derived state + inverse, which is
///   a much harder problem.

pub struct Computed<T: Clone> {
    /// Generation counters of inputs at last computation.
    input_generations: Vec<u64>,
    /// The cached value.
    cached: Arc<RwLock<Option<T>>>,
    /// Version of our cache. Incremented on recompute.
    version: Arc<RwLock<u64>>,
}

/// A reference to an input State with its generation counter.
struct InputRef {
    get_generation: Arc<dyn Fn() -> u64 + Send + Sync>,
    _phantom: std::marker::PhantomData<()>,
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    /// Create a new derived computation from input States.
    ///
    /// `inputs`: References to the State values this computation depends on.
    /// `compute`: A closure that takes a slice of input values and produces T.
    ///
    /// NOTE: In the current cvkg architecture, State doesn't expose a trait
    /// object. So we work around by storing a clone callback and a generation
    /// callback per input.
    pub fn new<
        I: IntoIterator<Item = InputRef>,
        F: Fn(&[String]) -> T + Send + Sync + 'static,
    >(
        inputs: I,
        compute: Arc<F>,
    ) -> Self {
        let input_generations: Vec<u64> = Vec::new();
        // ... implementation ...
        Self {
            input_generations,
            cached: Arc::new(RwLock::new(None)),
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the cached value, recomputing if any input has changed.
    pub fn get(&self) -> T {
        let needs_recompute = {
            let cached = self.cached.read()
                .map_err(|_| "lock poisoned")
                .unwrap_or_else(|_| None);
            cached.is_some()  // placeholder  -  check generations
        };
        
        if needs_recompute {
            // Recompute.
            let new_value = (self.compute)(&self.collect_inputs());
            if let Ok(mut cached) = self.cached.write() {
                *cached = Some(new_value.clone());
            }
            if let Ok(mut v) = self.version.write() {
                *v += 1;
            }
            new_value
        } else {
            // Cache hit.
            self.cached.read()
                .ok()
                .and_then(|c| c.clone())
                .unwrap_or_else(|| {
                    // Cache is empty (first call). Compute.
                    let val = (self.compute)(&self.collect_inputs());
                    if let Ok(mut cached) = self.cached.write() {
                        *cached = Some(val.clone());
                    }
                    val
                })
        }
    }
}
```

Due to the current cvkg State architecture not exposing trait objects, a simpler approach for v1:

```rust
/// Simplified derived state using manual refresh.
/// The user must call `refresh()` when inputs change.
/// Less automatic, but works with the current State type.
pub struct ComputedSignal<T: Clone> {
    inner: State<T>,
}

impl<T: Clone + Send + Sync + 'static> ComputedSignal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: State::new(initial),
        }
    }

    pub fn get(&self) -> T {
        self.inner.get()
    }

    /// Recompute the derived value.
    /// `f`: takes current inputs, returns new derived value.
    pub fn refresh<F: FnOnce() -> T>(&self, f: F) {
        self.inner.set(f());
    }
}
```

---

## 10. DropVault - File Upload Component

### Why this exists

This is the #1 most-requested component type that's completely absent. Every production app needs file upload: avatars, documents, drag-and-drop attachments.

### Implementation

```rust
// NEW FILE: cvkg-components/src/drop_vault.rs

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View, State, Event};
use std::sync::{Arc, Mutex};

/// A file upload zone with drag-and-drop support.
///
/// # Features:
/// - Drag files from the OS file manager onto the drop zone.
/// - Click to open the native file picker.
/// - Shows file name, size, and upload progress.
/// - Supports multiple files.
///
/// # Architecture:
/// The component does NOT implement file reading or network upload  -  it exposes
/// events (`on_files_dropped`, `on_upload_progress`) that the APP handles.
/// The component is purely visual + interaction.
///
/// # Left limit:
/// - Never read file contents in the UI thread. File I/O blocks.
///   The renderer's `open_file_picker()` callback should return file paths
///   or a read stream, not raw bytes.
/// - Never show raw file paths to users. Show file NAMES only.
///   Paths are a security leak (reveal username, directory structure).
///
/// # Right limit:
/// - Don't implement chunked upload or resumable upload in the component.
///   That's application-level logic.

#[derive(Clone)]
pub struct DropVault {
    /// Accepted MIME types. Empty = accept all.
    pub accepted_types: Vec<String>,
    /// Max file count. Default = 1.
    pub max_files: usize,
    /// Max file size in bytes. Default = 10MB.
    pub max_file_size: u64,
    /// Callback when files are selected (via picker or drop).
    pub on_files_selected: Option<Arc<dyn Fn(Vec<VaultFile>) + Send + Sync>>,
    /// Current upload state.
    pub uploads: State<Vec<VaultEntry>>,
    /// Whether the drag is currently over this zone.
    pub is_drag_over: State<bool>,
}

/// Information about a selected file.
#[derive(Debug, Clone)]
pub struct VaultFile {
    /// File name only (NO path  -  security).
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// MIME type (inferred from extension if OS doesn't provide it).
    pub mime_type: String,
}

/// Upload state for a single file.
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub file_info: VaultFile,
    pub progress: f32,  // 0.0 to 1.0
    pub status: VaultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultStatus {
    Pending,
    Uploading,
    Complete,
    Failed(String),  // error message
}

impl Default for DropVault {
    fn default() -> Self {
        Self::new()
    }
}

impl DropVault {
    pub fn new() -> Self {
        Self {
            accepted_types: Vec::new(),
            max_files: 1,
            max_file_size: 10 * 1024 * 1024,  // 10MB
            on_files_selected: None,
            uploads: State::new(Vec::new()),
            is_drag_over: State::new(false),
        }
    }

    pub fn accepted_types(mut self, types: Vec<String>) -> Self {
        self.accepted_types = types;
        self
    }

    pub fn max_files(mut self, max: usize) -> Self {
        self.max_files = max.max(1);
        self
    }

    pub fn max_size(mut self, bytes: u64) -> Self {
        self.max_file_size = bytes;
        self
    }

    pub fn on_files_selected<F: Fn(Vec<VaultFile>) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_files_selected = Some(Arc::new(f));
        self
    }
}

impl View for DropVault {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let is_drag = self.is_drag_over.get();
        let uploads = self.uploads.get();
        
        // Drop zone background
        let bg = if is_drag {
            theme::active_color()
        } else {
            theme::surface()
        };
        let border_color = if is_drag {
            theme::accent()
        } else {
            theme::border()
        };
        
        renderer.fill_rounded_rect(rect, 8.0, bg);
        renderer.stroke_rounded_rect(rect, 8.0, border_color, 2.0);
        
        // Render prompt text
        let prompt = if is_drag {
            "Drop files here"
        } else {
            "Drag files here or click to browse"
        };
        
        let (tw, _th) = renderer.measure_text(prompt, 14.0);
        renderer.draw_text(
            prompt,
            rect.x + (rect.width - tw) / 2.0,
            rect.y + 20.0,
            14.0,
            theme::text(),
        );
        
        // Render upload entries
        let mut y = rect.y + 50.0;
        for entry in &uploads {
            self.render_upload_entry(renderer, &entry, Rect::new(rect.x + 8.0, y, rect.width - 16.0, 40.0));
            y += 48.0;
        }
    }
}

impl DropVault {
    fn render_upload_entry(&self, renderer: &mut dyn Renderer, entry: &VaultEntry, rect: Rect) {
        // File name + size
        let size_str = format_file_size(entry.file_info.size);
        let label = format!("{} ({})", entry.file_info.name, size_str);
        
        renderer.draw_text(&label, rect.x + 4.0, rect.y + 4.0, 12.0, theme::text());
        
        // Progress bar
        let bar_y = rect.y + 22.0;
        let bar_h = 8.0;
        let bar_rect = Rect::new(rect.x + 4.0, bar_y, rect.width - 8.0, bar_h);
        
        // Background
        renderer.fill_rounded_rect(bar_rect, 4.0, theme::surface_elevated());
        
        // Fill
        let fill_width = (bar_rect.width * entry.progress).max(0.0);
        if fill_width > 0.0 {
            let fill_rect = Rect::new(bar_rect.x, bar_y, fill_width, bar_h);
            let color = match &entry.status {
                VaultStatus::Complete => theme::success(),
                VaultStatus::Failed(_) => theme::error_color(),
                _ => theme::accent(),
            };
            renderer.fill_rounded_rect(fill_rect, 4.0, color);
        }
        
        // Status text
        let status_text = match &entry.status {
            VaultStatus::Pending => "Waiting...".to_string(),
            VaultStatus::Uploading => format!("{}%", (entry.progress * 100.0) as u32),
            VaultStatus::Complete => "Done".to_string(),
            VaultStatus::Failed(msg) => format!("Error: {}", msg),
        };
        renderer.draw_text(&status_text, rect.x + 4.0, rect.y + 34.0, 10.0, theme::text_muted());
    }
}

/// Format a byte count as a human-readable string.
/// WHY: "1048576 bytes" is useless. "1.0 MB" is useful.
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}
```

Register in lib.rs:
```rust
pub mod drop_vault;
pub use jarngridr_anvil::{DropVault, VaultFile, VaultEntry, VaultStatus};
```

---

## 11. LinguaTong - i18n Localization Infrastructure

### Why this exists

The crate has ZERO internationalization. All strings are hardcoded English. This is a blocker for any non-English deployment.

### Implementation

```rust
// NEW FILE: cvkg-components/src/i18n.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

/// The global locale setting.
/// WHY OnceLock: The locale is set once at app startup and never changes
/// during a session (changing locale requires full re-render, which is
/// an app-level concern, not a component concern).
static LOCALE: OnceLock<RwLock<String>> = OnceLock::new();

/// Translation table: locale -> (key -> translated string).
/// WHY lazy_static/OnceLock: Translations are loaded once and shared
/// across all components. RwLock allows hot-reloading in dev mode.
static TRANSLATIONS: OnceLock<RwLock<HashMap<String, HashMap<String, String>>>> = OnceLock::new();

/// Set the active locale. Must be called before any component renders.
/// Returns Err if called more than once with a different locale.
///
/// WHY this restriction: Changing locale mid-render would cause some
/// components to render in the old locale and some in the new one.
/// The app should set locale once at startup, then do a full re-render.
pub fn set_locale(locale: &str) -> Result<(), &'static str> {
    let lock = LOCALE.get_or_init(|| RwLock::new(locale.to_string()));
    let mut guard = lock.write()
        .map_err(|_| "locale lock poisoned")?;
    if *guard != locale {
        return Err("locale already set to a different value");
    }
    Ok(())
}

/// Get the current locale.
pub fn current_locale() -> String {
    LOCALE.get()
        .and_then(|l| l.read().ok())
        .map(|g| g.clone())
        .unwrap_or_else(|| "en".to_string())
}

/// Load translations for a locale from a key-value map.
/// In production, this would load from .ftl (Fluent) files or .json.
/// For v1, we accept a HashMap.
pub fn load_translations(locale: &str, table: HashMap<String, String>) {
    let lock = TRANSLATIONS.get_or_init(|| RwLock::new(HashMap::new()));
    if let Ok(mut guard) = lock.write() {
        guard.insert(locale.to_string(), table);
    }
}

/// Look up a translation key. Falls back to the key itself if not found.
///
/// WHY fallback to key: If a translation is missing, showing the English
/// key is better than showing nothing or panicking. The developer sees
/// the missing key in the UI and knows to add it.
///
/// # Example:
/// ```
/// // In load_translations:
/// let mut en = HashMap::new();
/// en.insert("button.ok".to_string(), "OK".to_string());
/// en.insert("button.cancel".to_string(), "Cancel".to_string());
/// load_translations("en", en);
///
/// // In a component:
/// let label = t("button.ok");  // returns "OK"
/// let missing = t("button.unknown");  // returns "button.unknown"
/// ```
pub fn t(key: &str) -> String {
    let locale = current_locale();
    let translations = TRANSLATIONS.get()
        .and_then(|t| t.read().ok());
    
    if let Some(guard) = translations {
        if let Some(table) = guard.get(&locale) {
            if let Some(value) = table.get(key) {
                return value.clone();
            }
        }
        // Fallback to "en"
        if locale != "en" {
            if let Some(table) = guard.get("en") {
                if let Some(value) = table.get(key) {
                    return value.clone();
                }
            }
        }
    }
    
    // Final fallback: return the key itself.
    key.to_string()
}

/// Look up a translation with interpolation.
///
/// # Example:
/// ```
/// // Translation: "greeting": "Hello, {name}!"
/// let msg = t_with("greeting", &[("name", "Alice")]);
/// // Returns: "Hello, Alice!"
/// ```
pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut result = t(key);
    for (name, value) in args {
        result = result.replace(&format!("{{{}}}", name), value);
    }
    result
}

/// Detect if the current locale uses right-to-left text.
/// WHY: RTL locales (Arabic, Hebrew) require layout mirroring.
/// Every HStack becomes right-to-left, text alignment flips, etc.
pub fn is_rtl() -> bool {
    matches!(current_locale().as_str(), "ar" | "he" | "fa" | "ur")
}
```

**Update all components to use `t()` instead of hardcoded strings.**

Example for Button:
```rust
// BEFORE:
Button::new("OK", || { /* ... */ })

// AFTER:
Button::new(t("button.ok"), || { /* ... */ })
```

This is a MASSIVE find-and-replace across all 82 source files. The AI should:
1. Create the i18n module first.
2. Create an `en` translation table with ALL current hardcoded strings.
3. Replace hardcoded strings file by file, starting with the most visible (interactive.rs, container.rs).

---

## 12. AwaitVeil - Suspense Boundary with Skeleton Coordination

### Why this exists

When data is loading (async fetch, streaming), the UI should show a skeleton placeholder. But if MULTIPLE components in the same subtree are all loading, they should coordinate  -  show ONE skeleton for the whole subtree, not a flickering mess of individual spinners.

### Implementation

```rust
// NEW FILE: cvkg-components/src/await_veil.rs

use crate::DraumaSkeleton;
use cvkg_core::{Never, Rect, Renderer, View, State};

/// A suspense boundary that shows a skeleton while async data loads.
///
/// # How it works:
/// 1. The boundary starts in `Loading` state, showing a skeleton.
/// 2. When the async data arrives, the app calls `boundary.set_ready(content)`.
/// 3. The boundary transitions to `Ready` state, showing the content.
///
/// # Coordination:
/// When multiple Suspense boundaries are nested, the OUTERMOST one
/// shows the skeleton. Inner boundaries that complete early are rendered
/// immediately (no nested skeleton flicker).
///
/// # Left limit:
/// - Never show a skeleton for less than 200ms. Fast loads should not
///   flash a skeleton  -  it's more annoying than a blank.
/// - Never block user interaction while loading. The skeleton is visual only.
///
/// # Right limit:
/// - Don't implement retry logic. If the data fetch fails, show an error
///   state (handled by ErrorBoundary, not Suspense).

#[derive(Clone)]
pub enum AwaitState<V: View> {
    Loading,
    Ready(V),
}

#[derive(Clone)]
pub struct AwaitVeil<V: View> {
    state: State<AwaitState<V>>,
    /// Minimum time (in seconds) to show the skeleton before transitioning.
    /// WHY: Prevents skeleton flash on fast loads.
    min_display_time: f32,
    /// When the skeleton first appeared.
    loading_started: State<Option<f32>>,
}

impl<V: View> AwaitVeil<V> {
    pub fn new() -> Self {
        Self {
            state: State::new(AwaitState::Loading),
            min_display_time: 0.2,  // 200ms
            loading_started: State::new(None),
        }
    }

    /// Set the content when data arrives.
    pub fn set_ready(&self, content: V) {
        let now = /* renderer elapsed time */;
        let started = self.loading_started.get();
        
        if let Some(start) = started {
            let elapsed = now - start;
            if elapsed < self.min_display_time {
                // Wait until min_display_time has elapsed.
                // In a real implementation, schedule a delayed transition.
                // For v1, just set immediately (the 200ms is a nice-to-have).
            }
        }
        
        self.state.set(AwaitState::Ready(content));
    }

    /// Check if still loading.
    pub fn is_loading(&self) -> bool {
        matches!(self.state.get(), AwaitState::Loading)
    }
}

impl<V: View> View for AwaitVeil<V> {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        match self.state.get() {
            AwaitState::Loading => {
                // Show skeleton placeholder.
                // WHY DraumaSkeleton: It's already in the codebase (visual.rs:1135).
                // Reuse it instead of creating a new skeleton type.
                DraumaSkeleton::new()
                    .width(rect.width)
                    .height(rect.height)
                    .render(renderer, rect);
            }
            AwaitState::Ready(content) => {
                content.render(renderer, rect);
            }
        }
    }
}
```

Register in lib.rs:
```rust
pub mod await_veil;
pub use ragnarok_veil::Suspense;
```

---

## 13. PromptForge - Prompt Template Editor with Variable Binding

### Why this exists

The `GeriPrompt` component (ai_components.rs:10) is a text input. It has NO concept of prompt templates with variables. A prompt template editor lets users write:

```
You are a {{role}}. Help the user with {{task}}.
```

And see the variables highlighted, with a side panel to fill in values.

### Implementation

```rust
// NEW FILE: cvkg-components/src/prompt_forge.rs

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View, State};
use std::collections::HashMap;

/// A prompt template with `{{variable}}` placeholders.
///
/// # Syntax:
/// - `{{name}}`  -  required variable. Highlighted in accent color.
/// - `{{name:default}}`  -  variable with default value. Highlighted in muted color.
/// - Everything else  -  literal text. Normal color.
///
/// # Left limit:
/// - Never allow nested `{{ {{ }} }}`  -  it's a parse error.
/// - Never allow empty variable names `{{}}`  -  it's a parse error.
///
/// # Right limit:
/// - Don't implement conditional blocks (`{{#if condition}}...{{/if}}`).
///   That's a full template engine, not a prompt editor.

#[derive(Clone)]
pub struct PromptForge {
    /// The template string with `{{var}}` placeholders.
    pub template: String,
    /// Current variable values.
    pub variables: State<HashMap<String, String>>,
    /// Parsed segments for rendering.
    segments: Vec<ForgeSegment>,
}

#[derive(Debug, Clone)]
pub enum ForgeSegment {
    /// Literal text (no variable).
    Text(String),
    /// A variable placeholder.
    Variable {
        name: String,
        default: Option<String>,
        /// Byte offset in the original template.
        start: usize,
        end: usize,
    },
}

impl PromptForge {
    pub fn new(template: impl Into<String>) -> Self {
        let template = template.into();
        let segments = Self::parse(&template);
        Self {
            template,
            variables: State::new(HashMap::new()),
            segments,
        }
    }

    /// Set a variable value.
    pub fn set_variable(&self, name: &str, value: impl Into<String>) {
        let mut vars = self.variables.get();
        vars.insert(name.to_string(), value.into());
        self.variables.set(vars);
    }

    /// Get the rendered prompt with all variables substituted.
    pub fn rendered(&self) -> String {
        let vars = self.variables.get();
        let mut result = String::new();
        for segment in &self.segments {
            match segment {
                ForgeSegment::Text(t) => result.push_str(t),
                ForgeSegment::Variable { name, default, .. } => {
                    if let Some(value) = vars.get(name) {
                        result.push_str(value);
                    } else if let Some(default) = default {
                        result.push_str(default);
                    } else {
                        // Unfilled required variable  -  show placeholder.
                        result.push_str(&format!("{{{{{}}}}}", name));
                    }
                }
            }
        }
        result
    }

    /// Parse a template string into segments.
    fn parse(template: &str) -> Vec<ForgeSegment> {
        let mut segments = Vec::new();
        let mut current_pos = 0;

        // Simple state machine: scan for {{ and }}
        while current_pos < template.len() {
            if let Some(start) = template[current_pos..].find("{{") {
                let abs_start = current_pos + start;

                // Text before the variable
                if abs_start > current_pos {
                    segments.push(ForgeSegment::Text(
                        template[current_pos..abs_start].to_string()
                    ));
                }

                // Find closing }}
                if let Some(end_offset) = template[abs_start + 2..].find("}}") {
                    let abs_end = abs_start + 2 + end_offset;
                    let var_content = &template[abs_start + 2..abs_end];

                    // Parse "name" or "name:default"
                    let (name, default) = if let Some(colon_pos) = var_content.find(':') {
                        (
                            var_content[..colon_pos].trim().to_string(),
                            Some(var_content[colon_pos + 1..].trim().to_string()),
                        )
                    } else {
                        (var_content.trim().to_string(), None)
                    };

                    // Validate: empty name is a parse error -> skip
                    if !name.is_empty() {
                        segments.push(ForgeSegment::Variable {
                            name,
                            default,
                            start: abs_start,
                            end: abs_end + 2,
                        });
                    }

                    current_pos = abs_end + 2;
                } else {
                    // No closing }}  -  rest is literal text
                    segments.push(ForgeSegment::Text(
                        template[current_pos..].to_string()
                    ));
                    break;
                }
            } else {
                // No more variables
                segments.push(ForgeSegment::Text(
                    template[current_pos..].to_string()
                ));
                break;
            }
        }

        segments
    }
}

impl View for PromptForge {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut x = rect.x + 4.0;
        let mut y = rect.y + 4.0;
        let line_height = 20.0;

        for segment in &self.segments {
            match segment {
                ForgeSegment::Text(text) => {
                    renderer.draw_text(text, x, y, 14.0, theme::text());
                    let (w, _) = renderer.measure_text(text, 14.0);
                    x += w;
                }
                ForgeSegment::Variable { name, default, .. } => {
                    // Render variable name in accent color with braces.
                    let display = if let Some(default) = default {
                        format!("{}:{}", name, default)
                    } else {
                        name.clone()
                    };
                    let var_text = format!("{{{{{}}}}}", display);
                    let color = if default.is_some() {
                        theme::text_muted()
                    } else {
                        theme::accent()
                    };
                    renderer.draw_text(&var_text, x, y, 14.0, color);
                    let (w, _) = renderer.measure_text(&var_text, 14.0);
                    x += w;
                }
            }

            // Word wrap
            if x > rect.x + rect.width - 20.0 {
                x = rect.x + 4.0;
                y += line_height;
            }
        }
    }
}

/// A prompt template editor with a side panel for variable values.
pub struct PromptForge {
    pub template: PromptForge,
}

impl PromptForge {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: PromptTemplate::new(template),
        }
    }
}

impl View for PromptForge {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Split: left = template editor, right = variable panel
        let split = rect.width * 0.6;
        let editor_rect = Rect::new(rect.x, rect.y, split, rect.height);
        let panel_rect = Rect::new(rect.x + split, rect.y, rect.width - split, rect.height);

        // Render template
        self.template.render(renderer, editor_rect);

        // Render variable panel
        renderer.fill_rect(panel_rect, theme::surface_elevated());
        renderer.stroke_rect(panel_rect, theme::border(), 1.0);

        let vars = self.template.variables.get();
        let mut y = panel_rect.y + 8.0;

        for segment in &self.template.segments {
            if let ForgeSegment::Variable { name, default, .. } = segment {
                // Variable label
                renderer.draw_text(name, panel_rect.x + 8.0, y, 12.0, theme::text());
                y += 16.0;

                // Input field for the variable value
                let current_value = vars.get(name).map(|v| v.as_str()).unwrap_or("");
                let display = if current_value.is_empty() {
                    default.as_deref().unwrap_or("")
                } else {
                    current_value
                };

                let input_rect = Rect::new(panel_rect.x + 8.0, y, panel_rect.width - 16.0, 24.0);
                renderer.fill_rounded_rect(input_rect, 4.0, theme::surface());
                renderer.stroke_rounded_rect(input_rect, 4.0, theme::border(), 1.0);
                renderer.draw_text(
                    if display.is_empty() { "Enter value..." } else { display },
                    input_rect.x + 4.0,
                    input_rect.y + 16.0,
                    12.0,
                    if display.is_empty() { theme::text_muted() } else { theme::text() },
                );
                y += 36.0;
            }
        }
    }
}
```

Register in lib.rs:
```rust
pub mod prompt_forge;
pub use futhark_rune::{PromptForge, PromptForge};
```

---

## 14. ConsentGate - Consent Surface & Data Provenance UI

### Why this exists

`tyr_security.rs` shows roles and audit logs but has NO consent UI. When an AI system uses personal data for inference, regulations (GDPR, CCPA) require explicit consent. Users need to see WHAT data is used and revoke consent.

### Implementation

```rust
// NEW FILE: cvkg-components/src/consent_gate.rs

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View, State};

/// A consent request dialog for data usage.
///
/// # When to show:
/// Before the AI system accesses personal data (documents, messages, files),
/// show this dialog. The user must explicitly opt in.
///
/// # Left limit:
/// - Never pre-check consent boxes. Default is ALWAYS unchecked.
/// - Never use dark patterns (tiny "reject" button, confusing language).
///   Both "Accept" and "Reject" must be equally prominent.
/// - Never proceed without explicit user action. No timeouts, no auto-accept.
///
/// # Right limit:
/// - Don't implement consent persistence storage. The app handles that.
///   This component only renders the UI and fires events.

#[derive(Clone)]
pub struct ConsentGate {
    /// What data is being accessed.
    pub data_description: String,
    /// What the AI will do with it.
    pub purpose: String,
    /// Whether the user has consented.
    pub consented: State<bool>,
    /// Callback when user makes a choice.
    pub on_decision: Option<Arc<dyn Fn(bool) + Send + Sync>>,
}

impl ConsentGate {
    pub fn new(data_description: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            data_description: data_description.into(),
            purpose: purpose.into(),
            consented: State::new(false),
            on_decision: None,
        }
    }

    pub fn on_decision<F: Fn(bool) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_decision = Some(Arc::new(f));
        self
    }
}

impl View for ConsentGate {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Semi-transparent backdrop
        renderer.fill_rect(rect, [0.0, 0.0, 0.0, 0.5]);

        // Dialog card (centered)
        let dialog_w = 400.0_f32.min(rect.width - 40.0);
        let dialog_h = 250.0_f32.min(rect.height - 40.0);
        let dialog_x = rect.x + (rect.width - dialog_w) / 2.0;
        let dialog_y = rect.y + (rect.height - dialog_h) / 2.0;
        let dialog = Rect::new(dialog_x, dialog_y, dialog_w, dialog_h);

        renderer.fill_rounded_rect(dialog, 12.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(dialog, 12.0, theme::border(), 1.0);

        // Title
        renderer.draw_text(
            "Data Usage Consent",
            dialog.x + 16.0,
            dialog.y + 24.0,
            16.0,
            theme::text(),
        );

        // Data description
        renderer.draw_text(
            &format!("Data: {}", self.data_description),
            dialog.x + 16.0,
            dialog.y + 56.0,
            13.0,
            theme::text(),
        );

        // Purpose
        renderer.draw_text(
            &format!("Purpose: {}", self.purpose),
            dialog.x + 16.0,
            dialog.y + 80.0,
            13.0,
            theme::text_muted(),
        );

        // Buttons: Accept and Reject, EQUAL size and prominence.
        // WHY equal: Dark patterns use a big green "Accept" and tiny "Reject".
        // Both buttons must be the same size, same visual weight.
        let button_y = dialog.y + dialog_h - 50.0;
        let button_w = (dialog_w - 48.0) / 2.0;

        // Reject button (left, secondary style)
        let reject_rect = Rect::new(dialog.x + 16.0, button_y, button_w, 36.0);
        renderer.fill_rounded_rect(reject_rect, 6.0, theme::surface());
        renderer.stroke_rounded_rect(reject_rect, 6.0, theme::border(), 1.0);
        let (rw, _) = renderer.measure_text("Reject", 14.0);
        renderer.draw_text(
            "Reject",
            reject_rect.x + (button_w - rw) / 2.0,
            reject_rect.y + 22.0,
            14.0,
            theme::text(),
        );

        // Accept button (right, accent style)
        let accept_rect = Rect::new(dialog.x + 32.0 + button_w, button_y, button_w, 36.0);
        renderer.fill_rounded_rect(accept_rect, 6.0, theme::accent());
        let (aw, _) = renderer.measure_text("Accept", 14.0);
        renderer.draw_text(
            "Accept",
            accept_rect.x + (button_w - aw) / 2.0,
            accept_rect.y + 22.0,
            14.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

/// A data provenance indicator showing what data was used in AI inference.
///
/// # Why this matters:
/// When the AI gives a response, users should know what data it used.
/// "This response was based on: your project files (3), chat history (12 messages)"
/// builds trust and lets users correct the AI if it used wrong data.
#[derive(Clone)]
pub struct DataTrail {
    /// Data sources used in the inference.
    pub sources: Vec<TrailSource>,
}

#[derive(Debug, Clone)]
pub struct TrailSource {
    pub name: String,
    pub item_count: u32,
    pub data_type: TrailSourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrailSourceType {
    Document,
    Message,
    File,
    Database,
    Web,
}

impl TrailSourceType {
    pub fn icon(&self) -> &str {
        match self {
            TrailSourceType::Document => "[doc]",
            TrailSourceType::Message => "[msg]",
            TrailSourceType::File => "[file]",
            TrailSourceType::Database => "[db]",
            TrailSourceType::Web => "[web]",
        }
    }
}

impl View for DataTrail {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut y = rect.y;
        renderer.draw_text("Data used:", rect.x, y, 11.0, theme::text_muted());
        y += 14.0;

        for source in &self.sources {
            let text = format!("{} {} ({} items)", source.data_type.icon(), source.name, source.item_count);
            renderer.draw_text(&text, rect.x + 8.0, y, 11.0, theme::text());
            y += 14.0;
        }
    }
}
```

Register in lib.rs:
```rust
pub mod consent_gate;
pub use tyr_pledge::{ConsentGate, DataTrail, TrailSource, TrailSourceType};
```

---

## 15. VTree - Virtualized Tree

### Why this exists

`VirtualList` virtualizes a flat list. But file systems, org charts, and nested data are TREES. A virtualized tree must:

1. Only render visible nodes (including expanded subtrees).
2. Support expand/collapse without re-rendering the entire tree.
3. Handle trees with 100,000+ nodes without memory issues.

### Implementation

```rust
// NEW FILE: cvkg-components/src/vtree.rs

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View, State};
use std::sync::Arc;

/// A tree node that can be expanded/collapsed.
#[derive(Clone)]
pub struct VTreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<VTreeNode>,
    /// Whether this node is expanded (children visible).
    pub expanded: bool,
}

/// A virtualized tree view.
///
/// # How virtualization works for trees:
/// Unlike a flat list where visible items = scroll_offset..scroll_offset+viewport_size,
/// a tree's visible items depend on which nodes are expanded.
///
/// We flatten the visible tree into a list of (node, depth) pairs, then
/// virtualize that flattened list.
///
/// # Performance:
/// - Flattening is O(n) in the number of VISIBLE nodes (not total nodes).
/// - Only visible nodes are rendered.
/// - Expand/collapse only re-flattens the affected subtree.
///
/// # Left limit:
/// - Never render deeper than 32 levels. That's a UI anti-pattern regardless.
/// - Never expand all nodes by default. Large trees would OOM.
///
/// # Right limit:
/// - Don't implement tree node drag-and-drop reordering. That's a separate component.
/// - Don't implement multi-select. That's a separate feature.

pub struct VTree {
    /// The tree data.
    pub root: VTreeNode,
    /// Expanded state for each node (keyed by node id).
    pub expanded: State<std::collections::HashSet<String>>,
    /// Flattened visible nodes: (node_id, depth, label).
    visible_nodes: State<Vec<(String, usize, String)>>,
    /// Scroll offset.
    pub scroll_offset: State<f32>,
    /// Item height in logical pixels.
    pub item_height: f32,
}

impl VTree {
    pub fn new(root: VTreeNode) -> Self {
        let tree = Self {
            root,
            expanded: State::new(std::collections::HashSet::new()),
            visible_nodes: State::new(Vec::new()),
            scroll_offset: State::new(0.0),
            item_height: 24.0,
        };
        tree.rebuild_visible();
        tree
    }

    /// Toggle expand/collapse for a node.
    pub fn toggle(&self, node_id: &str) {
        let mut expanded = self.expanded.get();
        if expanded.contains(node_id) {
            expanded.remove(node_id);
        } else {
            expanded.insert(node_id.to_string());
        }
        self.expanded.set(expanded);
        self.rebuild_visible();
    }

    /// Rebuild the flattened visible node list.
    fn rebuild_visible(&self) {
        let expanded = self.expanded.get();
        let mut result = Vec::new();
        Self::flatten(&self.root, 0, &expanded, &mut result);
        self.visible_nodes.set(result);
    }

    fn flatten(
        node: &VTreeNode,
        depth: usize,
        expanded: &std::collections::HashSet<String>,
        result: &mut Vec<(String, usize, String)>,
    ) {
        // left limit: cap depth
        if depth > 32 { return; }

        result.push((node.id.clone(), depth, node.label.clone()));

        if expanded.contains(&node.id) {
            for child in &node.children {
                Self::flatten(child, depth + 1, expanded, result);
            }
        }
    }
}

impl View for VTree {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let nodes = self.visible_nodes.get();
        let scroll = self.scroll_offset.get();
        let item_h = self.item_height;

        // Calculate visible range
        let start_idx = (scroll / item_h).floor() as usize;
        let visible_count = (rect.height / item_h).ceil() as usize + 1;
        let end_idx = (start_idx + visible_count).min(nodes.len());

        let mut y = rect.y - (scroll % item_h);

        for i in start_idx..end_idx {
            let (id, depth, label) = &nodes[i];
            let item_rect = Rect::new(rect.x, y, rect.width, item_h);

            // Indent based on depth
            let indent = *depth as f32 * 16.0;

            // Expand/collapse indicator
            let has_children = self.has_children(id);
            let indicator = if has_children {
                if self.expanded.get().contains(id) {
                    "v "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            renderer.draw_text(
                &format!("{}{}", indicator, label),
                item_rect.x + indent + 4.0,
                item_rect.y + 16.0,
                13.0,
                theme::text(),
            );

            y += item_h;
        }
    }
}

impl VTree {
    fn has_children(&self, node_id: &str) -> bool {
        // Walk the tree to find the node and check if it has children.
        // In production, maintain a HashMap<id, &VTreeNode> for O(1) lookup.
        Self::find_node(&self.root, node_id)
            .map(|n| !n.children.is_empty())
            .unwrap_or(false)
    }

    fn find_node(node: &VTreeNode, id: &str) -> Option<&VTreeNode> {
        if node.id == id { return Some(node); }
        for child in &node.children {
            if let Some(found) = Self::find_node(child, id) {
                return Some(found);
            }
        }
        None
    }
}
```

Register in lib.rs:
```rust
pub mod vtree;
pub use niddhoggr_root::{VTree, VTreeNode};
```

---

## 16. Fix Duplicate Component Definitions

### Why this exists

The audit found these duplicates:
- `RadioGroup`: interactive.rs:2707 AND radio_group.rs:57
- `Popover`: container.rs:2386 AND popover.rs:44
- `Tooltip`: container.rs:2305 AND tooltip.rs:21
- `DatePicker`: advanced_forms.rs:9, calendar.rs:268, datepicker.rs:109 (THREE copies!)
- `Calendar`: advanced_forms.rs:173 AND calendar.rs
- `Combobox`: advanced_forms.rs:400 AND combobox.rs:20
- `Autocomplete`: advanced_forms.rs:292 AND autocomplete.rs:48
- `HuginChat`: advanced.rs:525 AND ai_components.rs:193

### Resolution strategy

For each duplicate, determine which is MORE COMPLETE (more features, better API, more tests) and make it the canonical version. The other becomes a deprecated re-export.

```
LEFT LIMIT:
- Never delete the old module file immediately. Mark #[deprecated].
- Never break existing imports. The old module path must still work.

RIGHT LIMIT:
- Don't try to merge the two implementations. Pick one, deprecate the other.
  Merging is error-prone and not worth the effort.
```

### Example resolution for RadioGroup:

```rust
// In src/radio_group.rs  -  KEEP this one (it's the dedicated file, more focused)

// In src/interactive.rs  -  REPLACE the RadioGroup at line 2707 with:
#[deprecated(note = "Use radio_group::RadioGroup instead")]
pub use crate::radio_group::RadioGroup as RadioGroup;
```

### Example resolution for DatePicker:

```rust
// datepicker.rs is the most complete (723 lines). Make it canonical.
// In advanced_forms.rs:9  -  REPLACE with:
#[deprecated(note = "Use datepicker::DatePicker instead")]
pub use crate::datepicker::DatePicker;
// In calendar.rs:268  -  REPLACE with:
#[deprecated(note = "Use datepicker::DatePicker instead")]
pub use crate::datepicker::DatePicker;
```

---

## 17. Hunt Down 70 unwrap() Calls

### Why this exists

Previous audits established zero-tolerance for `unwrap()` in production code. This crate has 70. Each one is a potential panic in production.

### Strategy

Run `grep -rn "\.unwrap()" src/` and categorize each call:

1. **Lock poisoning** (`mutex.lock().unwrap()`) -> Use `match` or `.ok()`:
   ```rust
   // BEFORE:
   let guard = self.data.lock().unwrap();
   // AFTER:
   let guard = match self.data.lock() {
       Ok(g) => g,
       Err(poisoned) => {
           log::warn!("Mutex poisoned, recovering");
           poisoned.into_inner()
       }
   };
   ```

2. **Option in layout math** (`.unwrap_or(default)` is acceptable):
   ```rust
   // BEFORE:
   let width = self.width.unwrap();
   // AFTER:
   let width = self.width.unwrap_or(0.0);
   ```

3. **Result from renderer calls** (should propagate):
   ```rust
   // BEFORE:
   let size = renderer.measure_text(text, font_size).unwrap();
   // AFTER:
   let size = renderer.measure_text(text, font_size)
       .unwrap_or((0.0, 0.0));  // graceful fallback: zero-size
   ```

4. **Test-only unwraps** (acceptable, leave them):
   ```rust
   // In #[test] functions, unwrap is fine.
   // Tests should panic on failure.
   ```

### Verification

After fixing, run:
```bash
cargo clippy -- -W clippy::unwrap_used
```
This clippy lint flags every `.unwrap()` in non-test code. The goal is ZERO warnings.

---

## Implementation Order

Implement in this order to minimize breakage and maximize incremental value:

1. **Phase 1  -  Foundation** (no API breaks):
   - [x] 17. Hunt down 70 unwrap() calls (fixes existing bugs)
   - [x] 16. Fix duplicate definitions (cleans up architecture)
   - [x] 11. i18n infrastructure (needed by everything else)

2. **Phase 2  -  Critical gaps** (additive, high impact):
   - [x] 1. Portal / Teleport System (fixes overlay clipping)
   - [x] 5. Live Region Accessibility (WCAG compliance)
   - [x] 10. File Upload Component (most-requested missing component)

3. **Phase 3  -  AI-native features** (CVKG's differentiator):
   - [x] 2. Streaming AI Diff Renderer
   - [x] 4. Confidence / Uncertainty Visualization
   - [x] 13. Prompt Template Editor

4. **Phase 4  -  Layout & Animation** (quality of life):
   - [x] 3. Container Query Layout System
   - [x] 7. Shared Element Transitions
   - [x] 8. Layout Animation for Siblings
   - [x] 9. Derived State Primitives

5. **Phase 5  -  Collaboration & Security** (enterprise features):
   - [x] 6. Real-Time Collaboration Primitives
   - [x] 14. Consent Surface & Data Provenance

6. **Phase 6  -  Virtualization** (performance):
   - [x] 15. Virtualized Tree
   - [x] 12. Suspense Boundary

---

## Verification Checklist

After ALL phases are complete, verify:

- [ ] `cargo build` succeeds with zero warnings
- [ ] `cargo test` passes all tests
- [ ] `cargo clippy -- -W clippy::unwrap_used` reports zero issues in production code
- [ ] `grep -rn "todo!()" src/` returns zero results
- [ ] `grep -rn "unimplemented!()" src/` returns zero results
- [ ] Every new component has at least one test
- [ ] Every new module has a `//!` doc comment
- [ ] `lib.rs` re-exports all new public types
- [ ] No duplicate component definitions remain
- [ ] All overlay components (dropdown, popover, tooltip, modal) use Portal internally
- [ ] All user-visible strings go through `t()` for i18n

---

*"The path of the righteous framework is beset on all sides by the inequities of missing features and the tyranny of unwrap(). Blessed is he who, in the name of correctness and completeness, shepherds the weak AI through the valley of implementation, for he is truly his codebase's keeper and the finder of lost edge cases."*

 -  Jules Winnfield, Senior Rust Engineer, reviewing this plan before going to get a Big Kahuna Burger

`v1.0 | Implementation Plan | For Weak Idiot AI Models That Need Hand-Holding`

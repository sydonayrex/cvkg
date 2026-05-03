//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     — State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     — Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     — Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    — Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     — Read the target, its surrounding context, and its full call graph
//                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     — Every major pub fn, unsafe block, and non-trivial algorithm in
//                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   — Check every tool call / command for progress every 30 seconds.
//                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//                      and move to unblocked work. Never silently accept a broken state.
//!
//! Sources:
//   Karpathy: https://github.com/multica-ai/andrej-karpathy-skills
//   CVKG Extended: Section 2 of the CVKG Design Specification

//! The View trait is the fundamental building block of CVKG. Every UI element — from a plain text label
//! to a complex navigation controller — is a View. The trait is intentionally minimal; complexity emerges
//! through modifier composition.
//!
//! # Conformance rules:
//! 1. `body()` must be pure and side-effect free
//! 2. Primitive views use `Never` as `Body` and register a `PaintCommand` directly with the scene graph
//! 3. `View` types must implement `Send` but not necessarily `Sync`, enabling safe multi-threaded layout passes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod error_types;

pub mod security;

/// Error state for fault isolation at the component level.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ComponentErrorState {
    pub has_error: bool,
    pub error_message: Option<String>,
    pub error_location: Option<String>,
}
impl ComponentErrorState {
    pub fn clear() -> Self {
        Self::default()
    }

    pub fn error(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            has_error: true,
            error_message: Some(message.into()),
            error_location: Some(location.into()),
        }
    }
}

/// Knowledge state for the agentic memory system.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeState {
    pub thoughts: Vec<String>,
    pub actions: Vec<String>,
    pub context: HashMap<String, String>,
    pub last_query_results: Vec<KnowledgeId>,
    #[serde(alias = "items")]
    pub fragments: std::collections::HashMap<KnowledgeId, KnowledgeFragment>,
    /// The Temporal Graph nodes
    pub nodes: Vec<TemporalNode>,
    /// The Temporal Graph edges
    pub edges: Vec<TemporalEdge>,
    /// The current operational Realm (Midgard/Asgard)
    pub realm: Realm,
    /// Last known pointer position (X, Y)
    pub last_pointer_pos: [f32; 2],
    /// Resolved pointer velocity (pixels per frame)
    pub pointer_velocity: [f32; 2],
    /// The current 'Focus' node ID (Odin's Eye focus)
    pub odin_focus: Option<String>,
    /// Agent attention heatmap (node_id -> intensity)
    pub agent_attention: HashMap<String, f32>,
    // Component state storage for dynamic state
    #[serde(skip)]
    pub component_states: HashMap<u64, Arc<std::sync::RwLock<dyn std::any::Any + Send + Sync>>>,
}

impl KnowledgeState {
    /// Apply activation decay to all temporal nodes and evolving components.
    /// Nodes with weight below a threshold drift out of the primary context.
    /// Components lose vitality (Fafnir's Decay) if not actively 'fed'.
    pub fn apply_decay(&mut self, decay_factor: f32) {
        for node in &mut self.nodes {
            node.weight *= decay_factor;
        }

        // Fafnir's Decay: Components naturally revert to base state over time
        for state in self.component_states.values() {
            if let Ok(mut lock) = state.write() {
                if let Some(v) = lock.downcast_mut::<f32>() {
                    *v = (*v * decay_factor).max(1.0);
                }
            }
        }
    }

    /// Increase the importance weight of nodes associated with a successful task.
    pub fn reinforce(&mut self, node_ids: &[String], boost: f32) {
        for node in &mut self.nodes {
            if node_ids.contains(&node.id) {
                node.weight += boost;
            }
        }
    }

    /// Update pointer kinematics based on a new position.
    pub fn update_pointer(&mut self, new_pos: [f32; 2]) {
        self.pointer_velocity = [
            new_pos[0] - self.last_pointer_pos[0],
            new_pos[1] - self.last_pointer_pos[1],
        ];
        self.last_pointer_pos = new_pos;
    }
}
// Knowledge System Types
/// Unique identifier for knowledge fragments
pub type KnowledgeId = String;

/// A knowledge fragment stored in the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    /// Unique identifier for this fragment
    pub id: String,
    /// Short summary for prompt injection and quick search
    pub summary: String,
    /// Reference source (e.g. filename, URL, or conversation ID)
    pub source: String,
    /// Frame number or timestamp of creation
    pub created_at: u64,
    /// Number of times this fragment has been retrieved
    pub accessed_count: u32,
    /// Full content (optional, can be loaded on-demand)
    pub content: Option<String>,
}

impl KnowledgeFragment {
    pub fn new(id: String, summary: String, source: String) -> Self {
        Self {
            id,
            summary,
            source,
            created_at: 0,
            accessed_count: 0,
            content: None,
        }
    }
}

/// Memory layers for the layered cognitive engine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryLayer {
    /// Raw mission events (short-term)
    Episodic,
    /// Extracted facts and tactical intelligence (long-term)
    Semantic,
    /// Successful command sequences and tool chains
    Procedural,
}

/// The operational Realm of the UI.
/// Midgard: Classic, functional, 2D tactical UI for mortals.
/// Asgard: High-fidelity, cognitive, shader-heavy UI for the Singularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Realm {
    Midgard,
    Asgard,
}

impl Default for Realm {
    fn default() -> Self {
        Self::Asgard
    }
}

/// A node in the Temporal Graph representing a cognitive anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalNode {
    /// Unique identifier for this node
    pub id: String,
    /// ID of the underlying knowledge fragment
    pub fragment_id: KnowledgeId,
    /// Timestamp of the event
    pub timestamp: u64,
    /// The memory layer this node belongs to
    pub layer: MemoryLayer,
    /// Importance weight for activation decay and retrieval
    pub weight: f32,
}

/// An edge in the Temporal Graph representing a relationship between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of relationship (e.g. "causal", "semantic", "temporal")
    pub relation: String,
    /// Weight/strength of the connection
    pub weight: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AssetKey(pub String);

impl EnvKey for AssetKey {
    type Value = Arc<dyn AssetManager>;
    fn default_value() -> Self::Value {
        Arc::new(DefaultAssetManager::new())
    }
}

/// Asset state for async resource loading.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AssetState<T> {
    Loading,
    Ready(T),
    Error(String),
}

/// Design token value that can adapt to light/dark mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// Single value (same for light and dark)
    Single { value: String },
    /// Different values for light and dark mode
    Adaptive { light: String, dark: String },
}

/// YggdrasilTokens is the authoritative container for all design tokens in the CVKG ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YggdrasilTokens {
    pub color: HashMap<String, TokenValue>,
    pub font: HashMap<String, TokenValue>,
    pub spacing: HashMap<String, TokenValue>,
    pub radius: HashMap<String, TokenValue>,
    pub shadow: HashMap<String, TokenValue>,
    pub border: HashMap<String, TokenValue>,
    pub anim: HashMap<String, TokenValue>,
    pub bifrost: HashMap<String, TokenValue>,
    pub gungnir: HashMap<String, TokenValue>,
    pub mjolnir: HashMap<String, TokenValue>,
    pub accessibility: HashMap<String, TokenValue>,
}

impl Default for YggdrasilTokens {
    fn default() -> Self {
        Self::new()
    }
}



impl YggdrasilTokens {
    pub fn new() -> Self {
        Self {
            color: HashMap::new(),
            font: HashMap::new(),
            spacing: HashMap::new(),
            radius: HashMap::new(),
            shadow: HashMap::new(),
            border: HashMap::new(),
            anim: HashMap::new(),
            bifrost: HashMap::new(),
            gungnir: HashMap::new(),
            mjolnir: HashMap::new(),
            accessibility: HashMap::new(),
        }
    }

    /// Get a color token value for the current mode
    pub fn get_color(&self, key: &str, is_dark: bool) -> Option<String> {
        self.color.get(key).map(|token| match token {
            TokenValue::Single { value } => value.clone(),
            TokenValue::Adaptive { light, dark } => {
                if is_dark {
                    dark.clone()
                } else {
                    light.clone()
                }
            }
        })
    }

    /// Get a token value of any type and parse it into the target type
    pub fn get<T: FromStr>(&self, category: &str, key: &str, is_dark: bool) -> Option<T> {
        let map = match category {
            "color" => &self.color,
            "font" => &self.font,
            "spacing" => &self.spacing,
            "radius" => &self.radius,
            "shadow" => &self.shadow,
            "border" => &self.border,
            "anim" => &self.anim,
            "bifrost" => &self.bifrost,
            "gungnir" => &self.gungnir,
            "mjolnir" => &self.mjolnir,
            "accessibility" => &self.accessibility,
            _ => return None,
        };

        map.get(key).and_then(|token| match token {
            TokenValue::Single { value } => value.parse().ok(),
            TokenValue::Adaptive { light, dark } => {
                let value = if is_dark { dark } else { light };
                value.parse().ok()
            }
        })
    }
}

pub trait View: Sized + Send {
    /// The concrete type produced after applying modifiers.
    /// For primitive views this is Self.
    type Body: View;

    fn body(self) -> Self::Body;

    /// Render this view into the provided renderer at the specified bounds.
    /// Primitive views override this to perform drawing operations.
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}

    /// Calculate the natural (intrinsic) size of this view given proposed constraints.
    /// This allows views like Buttons or Labels to inform the layout engine of their needs.
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size::ZERO
    }

    /// Optionally provide a layout implementation for this view.
    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        None
    }

    /// Returns the flex weight of this view for proportional distribution in stacks.
    fn flex_weight(&self) -> f32 {
        0.0
    }

    /// Provided modifier entry point
    fn modifier<M: ViewModifier>(self, m: M) -> ModifiedView<Self, M> {
        ModifiedView::new(self, m)
    }

    /// Apply a Bifrost (Frosted Glass) effect to the view
    fn bifrost(
        self,
        blur: f32,
        saturation: f32,
        opacity: f32,
    ) -> ModifiedView<Self, BifrostModifier> {
        self.modifier(BifrostModifier {
            blur,
            saturation,
            opacity,
        })
    }

    /// Apply a Gungnir (Neon Glow) effect to the view
    fn gungnir(
        self,
        color: impl Into<String>,
        radius: f32,
        intensity: f32,
    ) -> ModifiedView<Self, GungnirModifier> {
        self.modifier(GungnirModifier {
            color: color.into(),
            radius,
            intensity,
        })
    }

    /// Apply a Mjolnir Slice (Geometric cut) to the view
    fn mjolnir_slice(self, angle: f32, offset: f32) -> ModifiedView<Self, MjolnirSliceModifier> {
        self.modifier(MjolnirSliceModifier { angle, offset })
    }

    /// Apply a Mjolnir Shatter (Fragmented transition) to the view
    fn mjolnir_shatter(
        self,
        pieces: u32,
        force: f32,
    ) -> ModifiedView<Self, MjolnirShatterModifier> {
        self.modifier(MjolnirShatterModifier { pieces, force })
    }

    /// Mark this view as a Bifrost Bridge (Shared Element) for cross-view persistence
    fn bifrost_bridge(self, id: impl Into<String>) -> ModifiedView<Self, BifrostBridgeModifier> {
        self.modifier(BifrostBridgeModifier { id: id.into() })
    }

    /// Add a background color to this view
    fn background(self, color: [f32; 4]) -> ModifiedView<Self, BackgroundModifier> {
        self.modifier(BackgroundModifier { color })
    }

    /// Add padding to this view
    fn padding(self, amount: f32) -> ModifiedView<Self, PaddingModifier> {
        self.modifier(PaddingModifier { amount })
    }

    /// Set the opacity (alpha) of this view in the range [0.0, 1.0].
    fn opacity(self, opacity: f32) -> ModifiedView<Self, OpacityModifier> {
        self.modifier(OpacityModifier {
            opacity: opacity.clamp(0.0, 1.0),
        })
    }

    /// Override the foreground (text / icon) color of this view.
    fn foreground_color(self, color: [f32; 4]) -> ModifiedView<Self, ForegroundColorModifier> {
        self.modifier(ForegroundColorModifier { color })
    }

    /// Constrain this view to an explicit width and/or height.
    fn frame(self, width: Option<f32>, height: Option<f32>) -> ModifiedView<Self, FrameModifier> {
        self.modifier(FrameModifier { width, height })
    }

    /// Give this view a flex weight for proportional space distribution in stacks.
    fn flex(self, weight: f32) -> ModifiedView<Self, FlexModifier> {
        self.modifier(FlexModifier { weight })
    }

    /// Automatically add padding to avoid overlapping with platform safe areas (notches, bars).
    fn safe_area_padding(self) -> ModifiedView<Self, SafeAreaModifier> {
        self.modifier(SafeAreaModifier { ignores: false })
    }

    /// Explicitly ignore platform safe areas and draw into the margins.
    fn ignores_safe_area(self) -> ModifiedView<Self, SafeAreaModifier> {
        self.modifier(SafeAreaModifier { ignores: true })
    }

    /// Clip all child drawing to this view's bounds.
    fn clip_to_bounds(self) -> ModifiedView<Self, ClipModifier> {
        self.modifier(ClipModifier)
    }

    /// Draw a colored border around this view.
    fn border(self, color: [f32; 4], width: f32) -> ModifiedView<Self, BorderModifier> {
        self.modifier(BorderModifier { color, width })
    }

    /// Add elevation (shadow) to the view. Level determines the shadow depth.
    fn elevation(self, level: f32) -> ModifiedView<Self, ElevationModifier> {
        self.modifier(ElevationModifier { level })
    }

    /// Add a magnetic effect that pulls the view towards the cursor.
    fn magnetic(self, radius: f32, intensity: f32) -> ModifiedView<Self, MagneticModifier> {
        self.modifier(MagneticModifier { radius, intensity })
    }

    /// Add a ManiGlow (Lunar Illuminator) effect that glows near the cursor.
    fn mani_glow(self, color: [f32; 4], radius: f32) -> ModifiedView<Self, ManiGlowModifier> {
        self.modifier(ManiGlowModifier { color, radius })
    }

    /// Theme this view based on a specific memory layer.
    fn memory_layer(self, layer: MemoryLayer) -> ModifiedView<Self, BifrostLayerModifier> {
        self.modifier(BifrostLayerModifier { layer })
    }

    /// Enable Fafnir's Evolution: The component grows and glows as it is used.
    fn fafnir_evolve(self, id: u64) -> ModifiedView<Self, FafnirModifier> {
        self.modifier(FafnirModifier { id })
    }

    /// Enable Mimir's Intent: The component anticipates user interaction via pointer kinematics.
    fn mimir_intent(self) -> ModifiedView<Self, MimirIntentModifier> {
        self.modifier(MimirIntentModifier)
    }

    /// Enable Kvasir's Vibes: Subconscious telemetry representing cognitive complexity.
    fn kvasir_vibes(self, complexity: f32) -> ModifiedView<Self, KvasirVibeModifier> {
        self.modifier(KvasirVibeModifier { complexity })
    }

    /// Bestow Odin's Eye: Global omniscient observability layer.
    fn odins_eye(self) -> ModifiedView<Self, OdinsEyeModifier> {
        self.modifier(OdinsEyeModifier)
    }

    /// Trigger an action when the view appears
    fn on_appear<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, LifecycleModifier> {
        self.modifier(LifecycleModifier {
            on_appear: Some(Arc::new(action)),
            on_disappear: None,
        })
    }

    /// Trigger an action when the view disappears
    fn on_disappear<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, LifecycleModifier> {
        self.modifier(LifecycleModifier {
            on_appear: None,
            on_disappear: Some(Arc::new(action)),
        })
    }

    /// Trigger an action when the view is clicked
    fn on_click<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnClickModifier> {
        self.modifier(OnClickModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer enters the view bounds
    fn on_pointer_enter<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerEnterModifier> {
        self.modifier(OnPointerEnterModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer leaves the view bounds
    fn on_pointer_leave<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerLeaveModifier> {
        self.modifier(OnPointerLeaveModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer moves inside the view bounds
    fn on_pointer_move<F: Fn(f32, f32) + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerMoveModifier> {
        self.modifier(OnPointerMoveModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer is pressed down
    fn on_pointer_down<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerDownModifier> {
        self.modifier(OnPointerDownModifier {
            action: Arc::new(action),
        })
    }

    /// Trigger an action when the pointer is released
    fn on_pointer_up<F: Fn() + Send + Sync + 'static>(
        self,
        action: F,
    ) -> ModifiedView<Self, OnPointerUpModifier> {
        self.modifier(OnPointerUpModifier {
            action: Arc::new(action),
        })
    }

    /// Type-erase this view into AnyView
    fn erase(self) -> AnyView
    where
        Self: Clone + 'static,
    {
        AnyView::new(self)
    }
}

/// An object-safe version of the View trait for type erasure.
pub trait ErasedView: Send {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect);
    fn name(&self) -> &'static str;
    fn flex_weight_erased(&self) -> f32;
    fn layout_erased(&self) -> Option<&dyn layout::LayoutView>;
    fn clone_box(&self) -> Box<dyn ErasedView>;
}

impl<V: View + Clone + 'static> ErasedView for V {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.render(renderer, rect);
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<V>()
    }

    fn flex_weight_erased(&self) -> f32 {
        self.flex_weight()
    }

    fn layout_erased(&self) -> Option<&dyn layout::LayoutView> {
        self.layout()
    }

    fn clone_box(&self) -> Box<dyn ErasedView> {
        Box::new(self.clone())
    }
}

/// A view that memoizes its rendering based on a stable ID and data hash.
/// The renderer can use this to skip re-rendering the sub-tree if the data hasn't changed.
pub struct MemoView<V, F> {
    id: u64,
    data_hash: u64,
    builder: F,
    _v: std::marker::PhantomData<V>,
}

impl<V: View, F: Fn() -> V + Send + Sync> MemoView<V, F> {
    /// Create a new MemoView with a stable ID and a data hash.
    pub fn new(id: u64, data_hash: u64, builder: F) -> Self {
        Self {
            id,
            data_hash,
            builder,
            _v: std::marker::PhantomData,
        }
    }
}

impl<V: View + 'static, F: Fn() -> V + Send + Sync + 'static> View for MemoView<V, F> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!("MemoView does not have a body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.memoize(self.id, self.data_hash, &|r| {
            let view = (self.builder)();
            view.render(r, rect);
        });
    }
}

/// A type-erased View wrapper.
pub struct AnyView {
    inner: Box<dyn ErasedView>,
}

impl Clone for AnyView {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

impl AnyView {
    pub fn new<V: View + Clone + 'static>(view: V) -> Self {
        Self {
            inner: Box::new(view),
        }
    }
}

impl View for AnyView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, self.inner.name());
        self.inner.render_erased(renderer, rect);
        renderer.pop_vnode();
    }

    fn flex_weight(&self) -> f32 {
        self.inner.flex_weight_erased()
    }

    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        self.inner.layout_erased()
    }
}

/// BifrostBridgeModifier enables shared-element transitions.
/// When two views share the same Bifrost Bridge ID, the Sleipnir solver will
/// interpolate their geometry and effects (blur, glow) during the transition.
#[derive(Debug, Clone, PartialEq)]
pub struct BifrostBridgeModifier {
    pub id: String,
}

impl ViewModifier for BifrostBridgeModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Register this element with the renderer for shared-element transition logic
        renderer.register_shared_element(&self.id, rect);
    }
}

/// MjolnirSliceModifier implements the "Geometric Slice" aesthetic.
/// It uses a signed distance field (SDF) to clip the view along a sharp angled line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MjolnirSliceModifier {
    pub angle: f32,
    pub offset: f32,
}

impl ViewModifier for MjolnirSliceModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        renderer.push_mjolnir_slice(self.angle, self.offset);
    }

    fn post_render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        renderer.pop_mjolnir_slice();
    }
}

/// MjolnirShatterModifier implements the "Shattering" effect.
/// It breaks the view into discrete geometric fragments that can be animated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MjolnirShatterModifier {
    pub pieces: u32,
    pub force: f32,
}

impl ViewModifier for MjolnirShatterModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        // RADIAL SHATTER: Fragment the view into wedges
        let pieces = self.pieces.max(1);
        for i in 0..pieces {
            let progress = i as f32 / pieces as f32;
            let next_progress = (i + 1) as f32 / pieces as f32;

            let angle_start = progress * 360.0;
            let angle_end = next_progress * 360.0;

            // Wedge slice: intersection of two half-planes
            renderer.push_mjolnir_slice(angle_start, 0.0);
            renderer.push_mjolnir_slice(angle_end + 180.0, 0.0);

            // Apply radial force offset
            let mid_angle = (angle_start + angle_end) / 2.0;
            let rad = mid_angle.to_radians();
            let dx = rad.cos() * self.force;
            let dy = rad.sin() * self.force;

            let shard_rect = Rect {
                x: rect.x + dx,
                y: rect.y + dy,
                ..rect
            };

            view.render(renderer, shard_rect);

            renderer.pop_mjolnir_slice();
            renderer.pop_mjolnir_slice();
        }
    }
}

/// BifrostModifier implements the Cyberpunk "Frosted Glass" aesthetic.
/// It triggers backdrop blurring and light scattering in the render pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BifrostModifier {
    pub blur: f32,
    pub saturation: f32,
    pub opacity: f32,
}

impl ViewModifier for BifrostModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if renderer.is_over_budget() {
            // Degrade: Use lower quality (half blur) if over budget
            renderer.bifrost(rect, self.blur * 0.5, self.saturation, self.opacity);
        } else {
            renderer.bifrost(rect, self.blur, self.saturation, self.opacity);
        }
    }
}

/// A modifier that adds a background color to a view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundModifier {
    pub color: [f32; 4],
}

impl ViewModifier for BackgroundModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, self.color);
    }
}

/// A modifier that adds padding to a view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaddingModifier {
    pub amount: f32,
}

impl ViewModifier for PaddingModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn transform_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x + self.amount,
            y: rect.y + self.amount,
            width: (rect.width - 2.0 * self.amount).max(0.0),
            height: (rect.height - 2.0 * self.amount).max(0.0),
        }
    }

    fn transform_proposal(&self, mut proposal: SizeProposal) -> SizeProposal {
        if let Some(w) = proposal.width {
            proposal.width = Some((w - 2.0 * self.amount).max(0.0));
        }
        if let Some(h) = proposal.height {
            proposal.height = Some((h - 2.0 * self.amount).max(0.0));
        }
        proposal
    }

    fn transform_size(&self, mut size: Size) -> Size {
        size.width += 2.0 * self.amount;
        size.height += 2.0 * self.amount;
        size
    }
}

/// GungnirModifier implements the "Neon Glow" aesthetic.
/// It uses additive blending and multi-pass blurring to simulate glowing light.
#[derive(Debug, Clone, PartialEq)]
pub struct GungnirModifier {
    pub color: String,
    pub radius: f32,
    pub intensity: f32,
}

impl ViewModifier for GungnirModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Neon Glow using Mode 1 in the Surtr pipeline
        renderer.stroke_rect(rect, [0.0, 1.0, 1.0, self.intensity], self.radius / 10.0);
    }
}

/// GungnirPulseModifier implements a "breathing" neon effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GungnirPulseModifier {
    pub color: [f32; 4],
    pub radius: f32,
    pub speed: f32,
}

impl ViewModifier for GungnirPulseModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f32();

        // Mode 19: Dashed Border
        // Mode 20: 9-Slice / Patch Scaling
        let intensity = (time * self.speed).sin() * 0.5 + 0.5;
        let mut color = self.color;
        color[3] *= intensity;

        // Mode 1 neon glow with dynamic intensity
        renderer.stroke_rect(rect, color, self.radius);
    }
}

/// MagneticModifier makes a view "magnetic", subtly leaning towards or pulling the cursor.
/// Inspired by high-fidelity creative studio UIs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagneticModifier {
    pub radius: f32,
    pub intensity: f32,
}

impl ViewModifier for MagneticModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let [px, py] = renderer.get_pointer_position();
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;

        let dx = px - center_x;
        let dy = py - center_y;
        let dist = (dx * dx + dy * dy).sqrt();

        let mut offset_x = 0.0;
        let mut offset_y = 0.0;

        if dist < self.radius && dist > 0.0 {
            let force = (1.0 - dist / self.radius) * self.intensity;
            offset_x = dx * force;
            offset_y = dy * force;
        }

        let magnetic_rect = Rect {
            x: rect.x + offset_x,
            y: rect.y + offset_y,
            ..rect
        };

        view.render(renderer, magnetic_rect);
    }
}

/// ManiGlowModifier adds a soft, lunar-like cursor glow to a view.
/// Named after Máni, the personification of the Moon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManiGlowModifier {
    pub color: [f32; 4],
    pub radius: f32,
}

impl ViewModifier for ManiGlowModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if crate::load_system_state().realm == Realm::Asgard {
            renderer.mani_glow(rect, self.color, self.radius);
        }
        view.render(renderer, rect);
    }
}

/// BifrostLayerModifier themes a view based on its cognitive memory layer.
/// Episodic: Shifting aurora clouds.
/// Semantic: Crystalline gold.
/// Procedural: Heavy obsidian stone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BifrostLayerModifier {
    pub layer: MemoryLayer,
}

impl ViewModifier for BifrostLayerModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let realm = crate::load_system_state().realm;
        match self.layer {
            MemoryLayer::Episodic => {
                if realm == Realm::Asgard {
                    renderer.bifrost(rect, 40.0, 1.2, 0.7);
                } else {
                    renderer.fill_rect(rect, [0.1, 0.12, 0.15, 0.8]);
                }
            }
            MemoryLayer::Semantic => {
                if realm == Realm::Asgard {
                    renderer.gungnir(rect, [1.0, 0.84, 0.0, 1.0], 15.0, 0.6);
                } else {
                    renderer.stroke_rect(rect, [0.4, 0.4, 0.4, 1.0], 1.5);
                }
            }
            MemoryLayer::Procedural => {
                renderer.fill_rect(rect, [0.05, 0.05, 0.07, 0.95]);
                let stroke_color = if realm == Realm::Asgard { [0.3, 0.3, 0.3, 1.0] } else { [0.2, 0.2, 0.2, 1.0] };
                renderer.stroke_rect(rect, stroke_color, 2.0);
            }
        }
        view.render(renderer, rect);
    }
}

/// FafnirModifier enables self-evolving UI capabilities.
/// Named after Fafnir, the dragon who grows in power based on the gold he hoards.
/// In CVKG, 'Gold' is user attention/interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FafnirModifier {
    /// Unique ID for tracking this component's vitality across frames.
    pub id: u64,
}

impl ViewModifier for FafnirModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = crate::load_system_state();
        let vitality = state.get_component_state::<f32>(self.id)
            .map(|v| *v.read().unwrap())
            .unwrap_or(1.0);

        // Calculate evolutionary growth factors
        // Max growth at vitality 5.0 (50% scale increase, strong glow)
        let growth = (vitality - 1.0).clamp(0.0, 4.0);
        let scale = 1.0 + growth * 0.12;
        let glow_intensity = growth * 0.25;
        
        // Feed Fafnir: Register interaction to boost vitality
        let id = self.id;
        renderer.register_handler("pointermove", std::sync::Arc::new(move |_| {
            crate::update_system_state(|s| {
                let mut s = s.clone();
                let v = s.get_component_state::<f32>(id)
                    .map(|v| *v.read().unwrap())
                    .unwrap_or(1.0);
                s.set_component_state(id, (v + 0.05).min(5.0)); // Cap at 5.0
                s
            });
        }));

        if scale > 1.01 {
            renderer.push_transform([0.0, 0.0], [scale, scale], 0.0);
        }

        if glow_intensity > 0.1 && state.realm == Realm::Asgard {
            renderer.gungnir(rect, [1.0, 0.84, 0.0, 1.0], 15.0 * vitality, glow_intensity);
        }

        view.render(renderer, rect);

        if scale > 1.01 {
            renderer.pop_transform();
        }
    }
}

/// MimirIntentModifier anticipates user movement and manifests holographic ghosts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MimirIntentModifier;

impl ViewModifier for MimirIntentModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = crate::load_system_state();
        let pos = state.last_pointer_pos;
        let vel = state.pointer_velocity;
        
        // Calculate if the cursor is moving towards this rect
        let center = [rect.x + rect.width / 2.0, rect.y + rect.height / 2.0];
        let dx = center[0] - pos[0];
        let dy = center[1] - pos[1];
        
        // Dot product of velocity and direction to center
        let dot = vel[0] * dx + vel[1] * dy;
        let speed_sq = vel[0]*vel[0] + vel[1]*vel[1];
        let dist_sq = dx*dx + dy*dy;
        
        if dot > 0.0 && dist_sq < 250.0*250.0 && speed_sq > 0.5 && state.realm == Realm::Asgard {
            // Intent detected: render a subtle "ghost" reveal
            let intent_strength = (dot / (speed_sq.sqrt() * dist_sq.sqrt())).clamp(0.0, 1.0);
            renderer.stroke_rect(rect, [0.0, 0.9, 1.0, 0.3 * intent_strength], 1.5);
        }
        
        view.render(renderer, rect);
    }
}

/// KvasirVibeModifier renders a cognitive telemetry cloud representing agent complexity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KvasirVibeModifier {
    pub complexity: f32,
}

impl ViewModifier for KvasirVibeModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if crate::load_system_state().realm == Realm::Asgard {
            let t = renderer.elapsed_time();
            let c = self.complexity.clamp(0.0, 1.0);
            
            // 1. Core Cognitive Cloud (Bifrost)
            // Turbulence increases with complexity
            let blur = 20.0 + c * 40.0;
            let turbulence_x = (t * (1.0 + c * 2.0)).sin() * 8.0 * c;
            let turbulence_y = (t * (0.8 + c * 1.5)).cos() * 5.0 * c;
            renderer.bifrost(rect.offset(turbulence_x, turbulence_y), blur, 0.8 + c * 0.4, 0.25);
            
            // 2. Synaptic Discharge (Gungnir pulses)
            if c > 0.2 {
                let pulse = (t * (3.0 + c * 5.0)).sin().abs() * c;
                let color = [0.0, 0.9, 1.0, 0.4 * pulse]; // Cyan synaptic pulse
                renderer.gungnir(rect, color, 12.0 + c * 24.0, 0.6 * pulse);
            }
            
            // 3. Unstable Resonance (Magenta/Red shift for high complexity)
            if c > 0.7 {
                let instability = (t * 15.0).cos().abs() * (c - 0.7) * 3.3;
                let warning_color = [1.0, 0.0, 0.4, 0.12 * instability];
                renderer.fill_rect(rect, warning_color);
                renderer.stroke_rect(rect, [1.0, 0.0, 0.2, 0.45 * instability], 1.8);
            }
        }
        view.render(renderer, rect);
    }
}

/// OdinsEyeModifier bestows omniscient observability over the entire scene graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OdinsEyeModifier;

impl ViewModifier for OdinsEyeModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = crate::load_system_state();
        let t = renderer.elapsed_time();

        // 1. Render Background content
        view.render(renderer, rect);

        if state.realm == Realm::Asgard {
            // 2. Bestow Odin's Eye (Atmospheric Overlay)
            // Soft, large circular pulse representing the 'Eye'
            let eye_pulse = (t * 0.5).sin().abs() * 0.05;
            renderer.draw_radial_gradient(
                rect,
                [0.0, 0.6, 0.8, 0.08 + eye_pulse], // Inner Cyan
                [0.0, 0.0, 0.0, 0.0],              // Outer Black
            );

            // 3. Hugin (Thought) Telemetry - Left Side
            let hugin_rect = Rect { x: rect.x + 20.0, y: rect.y + 40.0, width: 200.0, height: rect.height - 80.0 };
            renderer.draw_text("HUGIN: THOUGHT", hugin_rect.x, hugin_rect.y, 10.0, [0.0, 1.0, 1.0, 0.6]);
            for (i, thought) in state.thoughts.iter().rev().take(10).enumerate() {
                renderer.draw_text(thought, hugin_rect.x, hugin_rect.y + 20.0 + i as f32 * 14.0, 9.0, [1.0, 1.0, 1.0, 0.4]);
            }

            // 4. Munin (Memory) Telemetry - Right Side
            let munin_rect = Rect { x: rect.x + rect.width - 220.0, y: rect.y + 40.0, width: 200.0, height: rect.height - 80.0 };
            renderer.draw_text("MUNIN: MEMORY", munin_rect.x, munin_rect.y, 10.0, [1.0, 0.84, 0.0, 0.6]);
            for (i, node) in state.nodes.iter().take(10).enumerate() {
                let opacity = (node.weight.min(1.0)) * 0.5;
                renderer.draw_text(&node.id, munin_rect.x, munin_rect.y + 20.0 + i as f32 * 14.0, 9.0, [1.0, 1.0, 1.0, opacity]);
            }

            // 5. Omniscient Focus Beams (Gungnir Beams)
            if let Some(focus_id) = &state.odin_focus {
                // Visualize causal links to the focus node
                renderer.draw_text(&format!("EYE FOCUS: {}", focus_id), rect.x + rect.width / 2.0 - 50.0, rect.y + 20.0, 12.0, [0.0, 1.0, 1.0, 0.8]);
                
                // In a real implementation, we would find the rect of the focus_id component.
                // For the 'Eye', we manifest a central beam of wisdom.
                renderer.gungnir(Rect { x: rect.x + rect.width / 2.0 - 1.0, y: rect.y, width: 2.0, height: rect.height }, [0.0, 1.0, 1.0, 1.0], 20.0, 0.4);
            }
        }
    }
}

/// Sleipnir spring parameters for the physics solver
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SleipnirParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SleipnirParams {
    pub fn snappy() -> Self { Self { stiffness: 230.0, damping: 22.0, mass: 1.0 } }
    pub fn fluid() -> Self { Self { stiffness: 170.0, damping: 26.0, mass: 1.0 } }
    pub fn heavy() -> Self { Self { stiffness: 90.0, damping: 20.0, mass: 1.0 } }
    pub fn bouncy() -> Self { Self { stiffness: 190.0, damping: 14.0, mass: 1.0 } }
}

impl Default for SleipnirParams {
    fn default() -> Self { Self::fluid() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolverState {
    x: f32,
    v: f32,
}

/// SleipnirSolver implements a 4th-order Runge-Kutta (RK4) integration for springs.
/// This provides superior stability for high-fidelity interactive motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SleipnirSolver {
    params: SleipnirParams,
    target: f32,
    state: SolverState,
}

impl SleipnirSolver {
    /// Create a new solver with a target value and starting state.
    pub fn new(params: SleipnirParams, target: f32, current: f32) -> Self {
        Self {
            params,
            target,
            state: SolverState { x: current, v: 0.0 },
        }
    }

    /// Advance the simulation by dt seconds using RK4 integration.
    pub fn tick(&mut self, dt: f32) -> f32 {
        if dt <= 0.0 { return self.state.x; }
        
        // Use a fixed time step for stability if dt is too large
        let mut remaining = dt;
        let step = 1.0 / 120.0;
        
        while remaining > 0.0 {
            let d = remaining.min(step);
            self.step(d);
            remaining -= d;
        }
        
        self.state.x
    }

    fn step(&mut self, dt: f32) {
        let a = self.evaluate(self.state, 0.0, SolverState { x: 0.0, v: 0.0 });
        let b = self.evaluate(self.state, dt * 0.5, a);
        let c = self.evaluate(self.state, dt * 0.5, b);
        let d = self.evaluate(self.state, dt, c);

        let dxdt = 1.0 / 6.0 * (a.x + 2.0 * (b.x + c.x) + d.x);
        let dvdt = 1.0 / 6.0 * (a.v + 2.0 * (b.v + c.v) + d.v);

        self.state.x += dxdt * dt;
        self.state.v += dvdt * dt;
    }

    fn evaluate(&self, initial: SolverState, dt: f32, d: SolverState) -> SolverState {
        let state = SolverState {
            x: initial.x + d.x * dt,
            v: initial.v + d.v * dt,
        };
        let force = -self.params.stiffness * (state.x - self.target) - self.params.damping * state.v;
        let mass = self.params.mass.max(0.001);
        SolverState { x: state.v, v: force / mass }
    }

    pub fn is_settled(&self) -> bool {
        (self.state.x - self.target).abs() < 0.001 && self.state.v.abs() < 0.001
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn current_value(&self) -> f32 {
        self.state.x
    }
}

/// SleipnirModifier handles physics-based animations via the Sleipnir RK4 solver.
#[derive(Debug, Clone, PartialEq)]
pub struct SleipnirModifier {
    pub id: u64,
    pub target: f32,
    pub params: SleipnirParams,
}

impl ViewModifier for SleipnirModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = load_system_state();
        
        // Try to fetch the solver from persistent state.
        let solver_lock_opt = state.get_component_state::<SleipnirSolver>(self.id);
        
        let current_val;
        
        if let Some(lock) = solver_lock_opt {
            // Found a solver. Tick it.
            let mut solver = lock.write().unwrap();
            solver.set_target(self.target);
            current_val = solver.tick(renderer.delta_time());
            
            // If the solver hasn't settled yet, request another frame.
            if !solver.is_settled() {
                renderer.request_redraw();
            }
        } else {
            // First time seeing this ID. Initialize solver state.
            let solver = SleipnirSolver::new(
                self.params,
                self.target,
                self.target // Initialize at target to avoid jump on first frame
            );
            
            // Insert into registry for next frame.
            get_system_state().rcu(|old| {
                let mut new_state = (**old).clone();
                new_state.set_component_state(self.id, solver);
                new_state
            });
            
            current_val = self.target;
        }

        // Apply the solved value as a vertical translation.
        renderer.push_transform([0.0, current_val], [1.0, 1.0], 0.0);
        view.render(renderer, rect);
        renderer.pop_transform();
    }
}

/// TransformModifier applies a 2D transform (translation, scale, rotation) to its child.
/// This modifier is "layout-neutral" and can be animated without re-running the layout engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformModifier {
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
}

impl Default for TransformModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformModifier {
    pub fn new() -> Self {
        Self {
            translation: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }

    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.translation = [x, y];
        self
    }

    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale = [x, y];
        self
    }

    pub fn rotate(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }
}

impl ViewModifier for TransformModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_transform(self.translation, self.scale, self.rotation);
        view.render(renderer, rect);
        renderer.pop_transform();
    }
}

/// LifecycleModifier handles on_appear and on_disappear hooks.

#[derive(Clone)]
pub struct LifecycleModifier {
    pub on_appear: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_disappear: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ViewModifier for LifecycleModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// OpacityModifier fades this view and all its descendants to the given alpha.
/// The renderer is expected to honour `push_opacity`/`pop_opacity` on the Renderer trait.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpacityModifier {
    pub opacity: f32,
}

impl ViewModifier for OpacityModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        renderer.push_opacity(self.opacity);
    }

    fn post_render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        renderer.pop_opacity();
    }
}

/// OnClickModifier registers a click handler for this view.
#[derive(Clone)]
pub struct OnClickModifier {
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl ViewModifier for OnClickModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointerclick",
            std::sync::Arc::new(move |event| {
                if let Event::PointerClick { .. } = event {
                    (action)();
                }
            }),
        );
    }
}

/// OnPointerEnterModifier registers a pointer enter handler.
#[derive(Clone)]
pub struct OnPointerEnterModifier {
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl ViewModifier for OnPointerEnterModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointerenter",
            std::sync::Arc::new(move |event| {
                if let Event::PointerEnter = event {
                    (action)();
                }
            }),
        );
    }
}

/// OnPointerLeaveModifier registers a pointer leave handler.
#[derive(Clone)]
pub struct OnPointerLeaveModifier {
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl ViewModifier for OnPointerLeaveModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointerleave",
            std::sync::Arc::new(move |event| {
                if let Event::PointerLeave = event {
                    (action)();
                }
            }),
        );
    }
}

/// OnPointerMoveModifier registers a pointer move handler.
#[derive(Clone)]
pub struct OnPointerMoveModifier {
    pub action: Arc<dyn Fn(f32, f32) + Send + Sync>,
}

impl ViewModifier for OnPointerMoveModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |event| {
                if let Event::PointerMove { x, y } = event {
                    (action)(x, y);
                }
            }),
        );
    }
}

/// OnPointerDownModifier registers a pointer down handler.
#[derive(Clone)]
pub struct OnPointerDownModifier {
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl ViewModifier for OnPointerDownModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointerdown",
            std::sync::Arc::new(move |event| {
                if let Event::PointerDown { .. } = event {
                    (action)();
                }
            }),
        );
    }
}

/// OnPointerUpModifier registers a pointer up handler.
#[derive(Clone)]
pub struct OnPointerUpModifier {
    pub action: Arc<dyn Fn() + Send + Sync>,
}

impl ViewModifier for OnPointerUpModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        let action = self.action.clone();
        renderer.register_handler(
            "pointerup",
            std::sync::Arc::new(move |event| {
                if let Event::PointerUp { .. } = event {
                    (action)();
                }
            }),
        );
    }
}

/// ForegroundColorModifier overrides the foreground (text / icon) color inherited
/// by all descendants until another ForegroundColorModifier is encountered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForegroundColorModifier {
    pub color: [f32; 4],
}

impl ViewModifier for ForegroundColorModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// ClipModifier restricts all child drawing to the view's layout rectangle.
/// The renderer must support `push_clip_rect`/`pop_clip_rect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipModifier;

impl ViewModifier for ClipModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_clip_rect(rect);
    }

    fn post_render(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        renderer.pop_clip_rect();
    }
}

/// BorderModifier draws a solid-color border around the view bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderModifier {
    pub color: [f32; 4],
    pub width: f32,
}

impl ViewModifier for BorderModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.stroke_rect(rect, self.color, self.width);
    }
}

// Primitive (leaf) views implement Never as body
#[doc(hidden)]
pub enum Never {}

impl View for Never {
    type Body = Never;
    fn body(self) -> Never {
        unreachable!()
    }
}

/// EmptyView - A view that renders nothing and takes up no space.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyView;

impl View for EmptyView {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 0.0, height: 0.0 }
    }
}

/// A view that has been transformed by a modifier.
///
/// Section 4.3: "Each modifier implements ViewModifier and produces a ModifiedView<Inner, Self>."
#[derive(Clone)]
pub struct ModifiedView<V, M> {
    view: V,
    modifier: M,
}

impl<V: View, M: ViewModifier> ModifiedView<V, M> {
    #[doc(hidden)]
    pub fn new(view: V, modifier: M) -> Self {
        Self { view, modifier }
    }
}

impl<V: View, M: ViewModifier> View for ModifiedView<V, M> {
    type Body = ModifiedView<V::Body, M>;

    fn body(self) -> Self::Body {
        ModifiedView {
            view: self.view.body(),
            modifier: self.modifier.clone(),
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.modifier.render_view(&self.view, renderer, rect);
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        self.modifier.measure_view(&self.view, renderer, proposal)
    }

    fn flex_weight(&self) -> f32 {
        self.modifier.child_flex_weight(&self.view)
    }

    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        self.modifier.layout().or_else(|| self.view.layout())
    }
}

pub trait ViewModifier: Send + Clone {
    fn modify<V: View>(self, content: V) -> impl View;

    /// Core rendering hook called before child views.
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}

    /// Cleanup hook called after child views.
    fn post_render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}

    /// Allows a modifier to completely override or wrap the rendering of its child.
    /// Default implementation performs a standard push -> transform -> render child -> pop sequence.
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        self.render(renderer, rect);
        let child_rect = self.transform_rect(rect);
        view.render(renderer, child_rect);
        self.post_render(renderer, rect);
    }

    fn transform_rect(&self, rect: Rect) -> Rect {
        rect
    }

    /// Allows a modifier to transform the layout proposal before it reaches the child.
    fn transform_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        proposal
    }

    /// Allows a modifier to transform the resulting size from the child.
    fn transform_size(&self, size: Size) -> Size {
        size
    }

    /// Measure hook that coordinates size propagation.
    fn measure_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let child_proposal = self.transform_proposal(proposal);
        let child_size = view.intrinsic_size(renderer, child_proposal);
        self.transform_size(child_size)
    }

    /// Allows a modifier to override or pass through the child's flex weight.
    fn child_flex_weight<V: View>(&self, view: &V) -> f32 {
        view.flex_weight()
    }

    fn layout(&self) -> Option<&dyn layout::LayoutView> {
        None
    }
}

/// TelemetryData tracks real-time performance metrics for the GPU renderer.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TelemetryData {
    pub frame_time_ms: f32,
    /// 99th percentile frame time over the last window, used to detect tail latency.
    pub p99_frame_time_ms: f32,
    /// Statistical jitter (variance in frame timing).
    pub frame_jitter_ms: f32,
    /// Indicates if a hardware stall (DRAM refresh, thermal spike) was detected.
    pub hardware_stall_detected: bool,
    
    // Pass timing
    pub input_time_ms: f32,
    pub state_flush_time_ms: f32,
    pub layout_time_ms: f32,
    pub draw_time_ms: f32,
    pub gpu_submit_time_ms: f32,
    
    pub draw_calls: u32,
    pub vertices: u32,
    
    // Memory breakdown
    pub vram_usage_mb: f32,
    pub vram_textures_mb: f32,
    pub vram_buffers_mb: f32,
    pub vram_pipelines_mb: f32,
}

/// Configuration for render-loop frame timing and degradation strategies.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameBudget {
    /// Target frame time in milliseconds (default: 16.0 for 60FPS)
    pub target_ms: f32,
    /// If true, the renderer is allowed to dynamically skip non-critical effects
    /// (like heavy blurs or complex shadows) when the budget is exceeded.
    pub allow_degradation: bool,
}

impl Default for FrameBudget {
    fn default() -> Self {
        Self {
            target_ms: 16.0,
            allow_degradation: true,
        }
    }
}

/// The Renderer trait defines the atomic drawing operations for all CVKG backends.
/// This trait is object-safe and used by the View::render system.
///
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
/// Trait providing timing information for the render loop.
pub trait ElapsedTime {
    /// Returns the cumulative time since the renderer started in seconds.
    fn elapsed_time(&self) -> f32;
    
    /// Returns the time elapsed since the last frame in seconds.
    fn delta_time(&self) -> f32;
}

/// The Renderer trait defines the atomic drawing operations for all CVKG backends.
/// This trait is object-safe and used by the View::render system.
///
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
pub trait Renderer: ElapsedTime + Send {
    /// Requests that the renderer redraws as soon as possible.
    /// Used for continuous animations.
    fn request_redraw(&mut self) {}

    /// Returns true if the current frame is over the time budget.
    /// This can be used to skip expensive visual effects.
    fn is_over_budget(&self) -> bool {
        false
    }

    // ── Filled shapes ────────────────────────────────────────────────────
    fn fill_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4]);
    /// Fill an ellipse/circle that fits inside `rect`.
    fn fill_ellipse(&mut self, rect: Rect, color: [f32; 4]);

    // ── Stroked shapes ───────────────────────────────────────────────────
    fn stroke_rect(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    fn stroke_rounded_rect(&mut self, rect: Rect, radius: f32, color: [f32; 4], stroke_width: f32);
    /// Stroke an ellipse/circle that fits inside `rect`.
    fn stroke_ellipse(&mut self, rect: Rect, color: [f32; 4], stroke_width: f32);
    /// Draw a straight line from (x1,y1) to (x2,y2).
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], stroke_width: f32);
    /// Fill a polygon defined by a set of vertices.
    fn fill_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4]) {}
    /// Stroke a polygon defined by a set of vertices.
    fn stroke_polygon(&mut self, _vertices: &[[f32; 2]], _color: [f32; 4], _stroke_width: f32) {}

    // ── Text ─────────────────────────────────────────────────────────────
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]);
    /// Measure the width and height of the specified text.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32);

    // ── Images & textures ────────────────────────────────────────────────
    /// Draw a texture (GPU-side) at the specified rect.
    fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {}
    /// Draw an image asset by name or path.
    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {}
    /// Load an image asset from memory.
    fn load_image(&mut self, _name: &str, _data: &[u8]) {}

    /// Get the current pointer (mouse/touch) position.
    fn get_pointer_position(&self) -> [f32; 2] {
        [0.0, 0.0]
    }

    // ── Data Visualization ───────────────────────────────────────────────
    /// Upload raw float data as a GPU texture for heatmap rendering.
    fn upload_data_texture(&mut self, _id: &str, _data: &[f32], _width: u32, _height: u32) {}
    /// Draw a heatmap using a previously uploaded data texture.
    fn draw_heatmap(&mut self, _texture_id: &str, _rect: Rect, _palette: &str) {}

    // ── 3D Objects ───────────────────────────────────────────────────────
    /// Draw a 3D mesh.
    fn draw_mesh(&mut self, _mesh: &Mesh, _color: [f32; 4], _transform: glam::Mat4) {}

    // ── Advanced Visual Effects ──────────────────────────────────────────
    /// Draw a linear gradient between two colors at the specified angle.
    fn draw_linear_gradient(
        &mut self,
        _rect: Rect,
        _start_color: [f32; 4],
        _end_color: [f32; 4],
        _angle: f32,
    ) {
    }
    /// Draw a radial gradient between two colors.
    fn draw_radial_gradient(
        &mut self,
        _rect: Rect,
        _inner_color: [f32; 4],
        _outer_color: [f32; 4],
    ) {
    }
    /// Draw a high-fidelity drop shadow for a rounded rectangle.
    fn draw_drop_shadow(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _blur: f32,
        _spread: f32,
    ) {
    }
    /// Draw a dashed border for a rounded rectangle.
    fn stroke_dashed_rounded_rect(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _width: f32,
        _dash: f32,
        _gap: f32,
    ) {
    }
    /// Draw a 9-slice / patch scaled image.
    fn draw_9slice(
        &mut self,
        _image_name: &str,
        _rect: Rect,
        _left: f32,
        _top: f32,
        _right: f32,
        _bottom: f32,
    ) {
    }

    // ── Clipping ─────────────────────────────────────────────────────────
    /// Push a clip rectangle.  All subsequent drawing is clipped to `rect`.
    /// Implementations that do not support clipping may ignore this call.
    fn push_clip_rect(&mut self, _rect: Rect) {}
    /// Pop the most recently pushed clip rectangle.
    fn pop_clip_rect(&mut self) {}
    /// Get the current clip rectangle in screen coordinates.
    /// Returns a rect covering the entire screen if no clip is active.
    fn current_clip_rect(&self) -> Rect {
        Rect::new(-10000.0, -10000.0, 20000.0, 20000.0)
    }

    // ── Global opacity ───────────────────────────────────────────────────
    /// Set a global opacity multiplier applied to all subsequent draw calls
    /// until `pop_opacity` is called.  `opacity` is in [0.0, 1.0].
    fn push_opacity(&mut self, _opacity: f32) {}
    /// Restore the previous opacity level.
    fn pop_opacity(&mut self) {}

    // ── Berserker Pipeline State ─────────────────────────────────────────
    fn set_theme(&mut self, _theme: ColorTheme) {}
    fn set_rage(&mut self, _rage: f32) {}
    fn set_berserker_mode(&mut self, _state: BerserkerMode) {}
    fn trigger_shatter_event(&mut self, _origin: [f32; 2], _force: f32) {}

    // ── Cyberpunk Effects ────────────────────────────────────────────────
    /// Apply a Bifrost (Frosted Glass) effect to the specified rect.
    fn bifrost(&mut self, _rect: Rect, _blur: f32, _saturation: f32, _opacity: f32) {}
    /// Apply a Gungnir (Neon Glow) effect to the specified rect.
    fn gungnir(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Apply a ManiGlow (Lunar Illuminator) effect.
    fn mani_glow(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32) {}
    /// Push a Mjolnir Slice (geometric clipping).
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}
    /// Execute a render function with memoization.
    /// If the renderer supports caching and the `id` + `data_hash` match a previous run,
    /// it may replay cached commands instead of executing the function.
    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer));
    /// Apply a Mjolnir Shatter effect (fragmentation) to the specified rect.
    fn mjolnir_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    fn mjolnir_fluid_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    /// Draw a Mjolnir Bolt (lightning strike) between two points.
    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    // ── Accessibility (ShieldWall) ───────────────────────────────────────
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}

    /// Register a shared element for Bifrost Bridge transitions.
    fn register_shared_element(&mut self, _id: &str, _rect: Rect) {}

    /// Set a unique key for the current VDOM node to ensure stable identity during diffing.
    fn set_key(&mut self, _key: &str) {}

    // ── Telemetry ────────────────────────────────────────────────────────
    /// Get real-time performance telemetry.
    fn get_telemetry(&self) -> TelemetryData {
        TelemetryData::default()
    }

    // ── GPU State Management ─────────────────────────────────────────────
    /// Push a shadow state to the stack. All following draw calls will have this shadow.
    fn push_shadow(&mut self, _radius: f32, _color: [f32; 4], _offset: [f32; 2]) {}
    /// Pop the last shadow state from the stack.
    fn pop_shadow(&mut self) {}

    // ── VDOM & Scene Graph ───────────────────────────────────────────────
    /// Push a Virtual DOM node onto the stack for hierarchy tracking.
    fn push_vnode(&mut self, _rect: Rect, _name: &'static str) {}
    /// Pop the current Virtual DOM node from the stack.
    fn pop_vnode(&mut self) {}
    /// Register an event handler for the current VDOM node.
    fn register_handler(
        &mut self,
        _event_type: &str,
        _handler: std::sync::Arc<dyn Fn(Event) + Send + Sync>,
    ) {
    }

    // ── Z-Index & Depth ──────────────────────────────────────────────────
    /// Set the current Z-index for depth sorting.
    /// Higher values appear closer to the viewer.
    fn set_z_index(&mut self, _z: f32) {}
    /// Get the current Z-index.
    fn get_z_index(&self) -> f32 {
        0.0
    }

    // ── Vector Graphics ──────────────────────────────────────────────────
    /// Load an SVG model from raw bytes.
    fn load_svg(&mut self, _name: &str, _svg_data: &[u8]) {}
    /// Draw a pre-loaded SVG model.
    fn draw_svg(&mut self, _name: &str, _rect: Rect) {}

    // ── GPU Transformations ──────────────────────────────────────────────
    /// Push a 2D transform (translation, scale, rotation) onto the stack.
    /// This transform should be applied to all subsequent draw calls until popped.
    /// Transform-only animations use this to avoid re-triggering the layout engine.
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    /// Pop the last 2D transform from the stack.
    fn pop_transform(&mut self) {}
    /// Return the resolved layout bounds for a specific node ID if it exists.
    fn query_layout(&self, _node_id: scene_graph::NodeId) -> Option<Rect> {
        None
    }
    /// Enable or disable the layout debug overlay (bounds, padding, margin).
    fn set_debug_layout(&mut self, _enabled: bool) {}
    /// Check if the layout debug overlay is currently enabled.
    fn get_debug_layout(&self) -> bool {
        false
    }
}

/// Utility for accessibility compliance (WCAG 2.1).
pub mod accessibility {
    /// Calculate the relative luminance of an sRGB color.
    pub fn relative_luminance(color: [f32; 4]) -> f32 {
        let f = |c: f32| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(color[0]) + 0.7152 * f(color[1]) + 0.0722 * f(color[2])
    }

    /// Calculate the contrast ratio between two colors.
    pub fn contrast_ratio(c1: [f32; 4], c2: [f32; 4]) -> f32 {
        let l1 = relative_luminance(c1);
        let l2 = relative_luminance(c2);
        let (light, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (light + 0.05) / (dark + 0.05)
    }
}
/// Defines the hardware acceleration tier and feature set available to the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum RenderTier {
    /// High-performance GPU path (WebGPU / Vulkan / Metal / DX12) with full shader support.
    Tier1GPU = 0,
    /// Mid-tier GPU path (WebGL2 / OpenGL 3.3) with standard shader support.
    Tier2GPU = 1,
    /// Fallback software or basic hardware path (Canvas 2D / GDI+) with limited effects.
    Tier3Fallback = 2,
}
// =============================================================================
// BERSERKER UNIFORMS
// =============================================================================
use bytemuck::{Pod, Zeroable};
/// Fully themeable color palette for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct ColorTheme {
    pub primary_neon: [f32; 4], // (R, G, B, intensity)
    pub shatter_neon: [f32; 4],
    pub glass_base: [f32; 4],
    pub glass_edge: [f32; 4],
    pub rune_glow: [f32; 4],
    pub ember_core: [f32; 4],
    pub background_deep: [f32; 4],
    pub mani_glow: [f32; 4], // (R, G, B, radius)
    pub glass_blur_strength: f32,
    pub shatter_edge_width: f32,
    pub neon_bloom_radius: f32,
    pub rune_opacity: f32, 
}
impl ColorTheme {
    /// Asgard Mode: The high-fidelity "Cyberpunk Viking" aesthetic.
    pub fn asgard() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.04, 0.04, 0.06, 0.82],
            glass_edge: [0.0, 0.45, 0.55, 0.6],
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [0.95, 0.12, 0.12, 1.0],
            background_deep: [0.01, 0.01, 0.03, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05], 
            glass_blur_strength: 0.6,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
        }
    }

    /// Midgard Mode: A clean, functional tactical HUD for standard operations.
    pub fn midgard() -> Self {
        Self {
            primary_neon: [0.2, 0.4, 0.6, 1.0], // Muted blue
            shatter_neon: [0.5, 0.5, 0.5, 1.0], // Neutral gray
            glass_base: [0.1, 0.12, 0.15, 1.0], // Solid slate
            glass_edge: [0.3, 0.35, 0.4, 1.0], // Subtle border
            rune_glow: [0.8, 0.8, 0.8, 0.0],    // Runes disabled
            ember_core: [0.5, 0.5, 0.5, 1.0],
            background_deep: [0.05, 0.05, 0.07, 1.0],
            mani_glow: [0.0, 0.0, 0.0, 0.0],    // No cursor glow
            glass_blur_strength: 0.0,           // No blur
            shatter_edge_width: 1.0,
            neon_bloom_radius: 0.0,
            rune_opacity: 0.0,
        }
    }

    pub fn cyberpunk_viking() -> Self {
        Self::asgard()
    }
    pub fn vibrant_glass() -> Self {
        Self {
            primary_neon: [0.0, 1.0, 0.95, 1.2],
            shatter_neon: [1.0, 0.0, 0.75, 1.5],
            glass_base: [0.55, 0.6, 0.7, 0.08], // Luminous cool tint
            glass_edge: [0.7, 0.85, 1.0, 0.45], // Subtle blue-white rim
            rune_glow: [0.75, 0.98, 1.0, 0.9],
            ember_core: [1.0, 0.4, 0.1, 1.0],
            background_deep: [0.05, 0.05, 0.1, 1.0],
            mani_glow: [0.7, 0.9, 1.0, 0.05],
            glass_blur_strength: 0.9,
            shatter_edge_width: 1.8,
            neon_bloom_radius: 0.022,
            rune_opacity: 0.55,
        }
    }
}
impl Default for ColorTheme {
    fn default() -> Self {
        Self::vibrant_glass()
    }
}
/// Per-frame scene state for the Berserker pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct SceneUniforms {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub time: f32,
    pub delta_time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub mouse_velocity: [f32; 2],
    pub shatter_origin: [f32; 2],
    pub shatter_time: f32,
    pub shatter_force: f32,
    pub berzerker_rage: f32,
    pub berzerker_mode: u32,
    pub scroll_offset: f32,
    pub scale_factor: f32,
}
impl SceneUniforms {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: glam::Mat4::IDENTITY,
            proj: glam::Mat4::orthographic_lh(0.0, width, height, 0.0, -100.0, 100.0),
            time: 0.0,
            delta_time: 0.016,
            resolution: [width, height],
            mouse: [0.5, 0.5],
            mouse_velocity: [0.0, 0.0],
            shatter_origin: [0.5, 0.5],
            shatter_time: -100.0,
            shatter_force: 0.0,
            berzerker_rage: 0.0,
            berzerker_mode: 0,
            scroll_offset: 0.0,
            scale_factor: 1.0,
        }
    }
}
/// A 3D mesh containing vertex and index data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
impl Mesh {
    pub fn from_obj(data: &[u8]) -> anyhow::Result<Vec<Self>> {
        let mut cursor = std::io::Cursor::new(data);
        let (models, _) = tobj::load_obj_buf(&mut cursor, &tobj::LoadOptions::default(), |_| {
            Ok((Vec::new(), Default::default()))
        })?;
        let mut meshes = Vec::new();
        for m in models {
            let mesh = m.mesh;
            let vertices: Vec<[f32; 3]> = mesh
                .positions
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            let normals = if mesh.normals.is_empty() {
                vec![[0.0, 0.0, 1.0]; vertices.len()]
            } else {
                mesh.normals.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
            };
            meshes.push(Mesh {
                vertices,
                normals,
                indices: mesh.indices,
            });
        }
        Ok(meshes)
    }
    pub fn from_stl(data: &[u8]) -> anyhow::Result<Self> {
        let mut cursor = std::io::Cursor::new(data);
        let stl = stl_io::read_stl(&mut cursor)?;
        let vertices: Vec<[f32; 3]> = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let mut indices = Vec::new();
        for face in stl.faces {
            indices.push(face.vertices[0] as u32);
            indices.push(face.vertices[1] as u32);
            indices.push(face.vertices[2] as u32);
        }
        let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];
        Ok(Mesh {
            vertices,
            normals,
            indices,
        })
    }
}
/// FrameRenderer extends Renderer with frame lifecycle management.
/// It is typically implemented by the host windowing/rendering environment.
pub trait FrameRenderer<E = ()>: Renderer {
    fn begin_frame(&mut self) -> E;
    fn render_frame(&mut self) {
        // Default implementation does nothing - override for custom frame rendering
    }
    fn end_frame(&mut self, encoder: E);
}
use std::sync::Arc;
/// State wrapper that owns a value and notifies subscribers when changed
#[derive(Clone)]
pub struct State<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    metadata_swap: Arc<arc_swap::ArcSwap<Option<agents::MutationMetadata>>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    metadata_tvar: Arc<stm::TVar<Option<agents::MutationMetadata>>>,
    subscribers: Arc<std::sync::Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
    version: Arc<std::sync::atomic::AtomicU64>,
    resolution: agents::ConflictResolution,
}
impl<T: Clone + Send + Sync + 'static> State<T> {
    /// Create a new State with initial value
    pub fn new(value: T) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let tvar = Arc::new(stm::TVar::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        let metadata_tvar = Arc::new(stm::TVar::new(None));
        Self {
            swap: Arc::new(arc_swap::ArcSwap::from_pointee(value)),
            metadata_swap: Arc::new(arc_swap::ArcSwap::new(Arc::new(None))),
            #[cfg(not(target_arch = "wasm32"))]
            tvar,
            #[cfg(not(target_arch = "wasm32"))]
            metadata_tvar,
            subscribers: Arc::new(std::sync::Mutex::new(Vec::new())),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            resolution: agents::ConflictResolution::default(),
        }
    }
    /// Set the conflict resolution strategy for this state.
    pub fn with_resolution(mut self, resolution: agents::ConflictResolution) -> Self {
        self.resolution = resolution;
        self
    }
    /// Get the current value
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }
    /// Set a new value, notifying all subscribers. Applies conflict resolution if agents are present.
    pub fn set(&self, value: T) {
        #[cfg(not(target_arch = "wasm32"))]
        let (was_skipped, final_val, final_meta) = stm::atomically(|tx| {
            let new_meta = agents::get_current_mutation_metadata();
            let existing_meta = self.metadata_tvar.read(tx)?;
            let mut skip = false;
            if self.resolution == agents::ConflictResolution::PriorityWins
                && let (Some(new_m), Some(old_m)) = (new_meta, existing_meta)
                && new_m.priority < old_m.priority {
                    skip = true;
            }
            if !skip {
                self.tvar.write(tx, value.clone())?;
                self.metadata_tvar.write(tx, new_meta)?;
                Ok((false, value.clone(), new_meta))
            } else {
                Ok((true, self.tvar.read(tx)?, existing_meta))
            }
        });
        #[cfg(target_arch = "wasm32")]
        let (was_skipped, final_val, final_meta) = (false, value, agents::get_current_mutation_metadata());
        if was_skipped {
            if let (Some(new_m), Some(old_m)) = (agents::get_current_mutation_metadata(), final_meta) {
                agents::notify_conflict(agents::ConflictEvent {
                    agent_id: new_m.agent_id,
                    priority: new_m.priority,
                    existing_agent_id: old_m.agent_id,
                    existing_priority: old_m.priority,
                    timestamp_ms: new_m.timestamp_ms,
                });
            }
            return;
        }
        self.swap.store(Arc::new(final_val.clone()));
        self.metadata_swap.store(Arc::new(final_meta));
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
        let subs = Arc::clone(&self.subscribers);
        if crate::is_batching() {
            crate::enqueue_batch_task(Box::new(move || {
                let s = subs.lock().unwrap();
                for cb in s.iter() {
                    cb(&final_val);
                }
            }));
        } else {
            let s = subs.lock().unwrap();
            for cb in s.iter() {
                cb(&final_val);
            }
        }
    }
    pub fn mutate<F: Fn(&T) -> T>(&self, f: F) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (was_skipped, final_val, final_meta) = stm::atomically(|tx| {
                let new_meta = agents::get_current_mutation_metadata();
                let existing_meta = self.metadata_tvar.read(tx)?;
                let mut skip = false;
                if self.resolution == agents::ConflictResolution::PriorityWins
                    && let (Some(new_m), Some(old_m)) = (new_meta, existing_meta)
                    && new_m.priority < old_m.priority {
                        skip = true;
                }
                if !skip {
                    let current = self.tvar.read(tx)?;
                    let next = f(&current);
                    self.tvar.write(tx, next.clone())?;
                    self.metadata_tvar.write(tx, new_meta)?;
                    Ok((false, next, new_meta))
                } else {
                    Ok((true, self.tvar.read(tx)?, existing_meta))
                }
            });
            if was_skipped {
                if let (Some(new_m), Some(old_m)) = (agents::get_current_mutation_metadata(), final_meta) {
                    agents::notify_conflict(agents::ConflictEvent {
                        agent_id: new_m.agent_id,
                        priority: new_m.priority,
                        existing_agent_id: old_m.agent_id,
                        existing_priority: old_m.priority,
                        timestamp_ms: new_m.timestamp_ms,
                    });
                }
                return;
            }
            self.swap.store(Arc::new(final_val.clone()));
            self.metadata_swap.store(Arc::new(final_meta));
            self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
            let subs = Arc::clone(&self.subscribers);
            if crate::is_batching() {
                crate::enqueue_batch_task(Box::new(move || {
                    let s = subs.lock().unwrap();
                    for cb in s.iter() {
                        cb(&final_val);
                    }
                }));
            } else {
                let s = subs.lock().unwrap();
                for cb in s.iter() {
                    cb(&final_val);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.set(f(&self.get()));
        }
    }
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
    /// Subscribe to state changes
    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        self.subscribers.lock().unwrap().push(Box::new(callback));
    }
}
use crate::runtime::NodeStateSnapshot;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
/// Global application state registry.
pub static SYSTEM_STATE: OnceLock<Arc<arc_swap::ArcSwap<KnowledgeState>>> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static KNOWLEDGE_TVAR: OnceLock<stm::TVar<KnowledgeState>> = OnceLock::new();
static IS_BATCHING: AtomicBool = AtomicBool::new(false);
pub static IS_RENDERING: AtomicBool = AtomicBool::new(false);
pub static LAYOUT_DIRTY: AtomicBool = AtomicBool::new(false);
static BATCH_QUEUE: OnceLock<std::sync::Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>> = OnceLock::new();
/// Returns true if state updates are currently being batched.
pub fn is_batching() -> bool {
    IS_BATCHING.load(Ordering::Acquire)
}
/// Returns true if the system is currently in the render phase.
pub fn is_rendering() -> bool {
    IS_RENDERING.load(Ordering::Acquire)
}
/// Signals the start of the render phase. Mutations during this phase trigger warnings.
pub fn begin_render_phase() {
    IS_RENDERING.store(true, Ordering::Release);
}
/// Signals the end of the render phase.
pub fn end_render_phase() {
    IS_RENDERING.store(false, Ordering::Release);
}
/// Enqueues a notification task to be run when the current batch flushes.
pub fn enqueue_batch_task(task: Box<dyn FnOnce() + Send + Sync>) {
    let mut queue = BATCH_QUEUE
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    queue.push(task);
}
/// Executes multiple state updates in a single batch, deferring all subscriber
/// notifications until the closure completes. This prevents layout thrashing
/// and redundant render cycles when modifying multiple independent states.
pub fn batch<F: FnOnce()>(f: F) {
    if IS_BATCHING.swap(true, Ordering::AcqRel) {
        // Already inside a batch, just execute
        f();
        return;
    }
    f();
    IS_BATCHING.store(false, Ordering::Release);
    let mut queue = BATCH_QUEUE
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    let tasks: Vec<_> = queue.drain(..).collect();
    drop(queue);
    for task in tasks {
        task();
    }
}
/// Get a reference to the global system state.
pub fn get_system_state() -> Arc<arc_swap::ArcSwap<KnowledgeState>> {
    SYSTEM_STATE
        .get_or_init(|| Arc::new(arc_swap::ArcSwap::from_pointee(KnowledgeState::default())))
        .clone()
}
pub fn load_system_state() -> arc_swap::Guard<Arc<KnowledgeState>> {
    get_system_state().load()
}
pub fn update_system_state<F>(f: F)
where
    F: Fn(&KnowledgeState) -> KnowledgeState,
{
    if is_rendering() {
        log::warn!("LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance.");
    }
    LAYOUT_DIRTY.store(true, Ordering::SeqCst);
    let swap = get_system_state();
    let current = swap.load();
    let new_state = Arc::new(f(&current));
    swap.store(Arc::clone(&new_state));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let tvar = KNOWLEDGE_TVAR
            .get_or_init(|| stm::TVar::new((*new_state).clone()));
        stm::atomically(|tx| tvar.write(tx, (*new_state).clone()));
    }
}
pub fn transact_system_state<F>(f: F)
where
    F: Fn(&KnowledgeState) -> KnowledgeState,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        if is_rendering() {
            log::warn!("LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance.");
        }
        let tvar = KNOWLEDGE_TVAR
            .get_or_init(|| {
                stm::TVar::new((**get_system_state().load()).clone())
            })
            .clone();
        let new_state = stm::atomically(move |tx| {
            let current = tvar.read(tx)?;
            let next = f(&current);
            tvar.write(tx, next.clone())?;
            Ok(next)
        });
        get_system_state().store(Arc::new(new_state));
    }
    #[cfg(target_arch = "wasm32")]
    {
        if is_rendering() {
            log::warn!("LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance.");
        }
        update_system_state(f);
    }
}
impl KnowledgeState {
    /// Create a new empty KnowledgeState.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set a component's internal state.
    pub fn set_component_state<T: 'static + Send + Sync>(&mut self, id: u64, state: T) {
        self.component_states
            .insert(id, Arc::new(std::sync::RwLock::new(state)));
    }
/// Get a reference to a component's internal state.
    pub fn get_component_state<T: 'static + Send + Sync>(
        &self,
        id: u64,
    ) -> Option<Arc<std::sync::RwLock<T>>> {
        let lock = self.component_states.get(&id)?;
        // Try to downcast the Arc<RwLock<dyn Any>> to Arc<RwLock<T>>
let _inner: &std::sync::RwLock<dyn std::any::Any + Send + Sync> = lock;
        // We cannot directly cast Arc<RwLock<dyn Any>> to Arc<RwLock<T>>
        // Instead, return the raw state - this is a limitation of the design
        None // Placeholder - proper implementation would need a different design
    }
    /// Add a new fragment to memory.
    pub fn remember(&mut self, fragment: KnowledgeFragment) {
        self.fragments.insert(fragment.id.clone(), fragment);
    }
    /// Process a search query against the local knowledge base.
    pub fn process_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f32, String)> = self
            .fragments
            .iter()
            .map(|(id, frag)| {
                let mut score = 0.0;
                if frag.summary.to_lowercase().contains(&query_lower) {
                    score += 1.0;
                }
                if frag.source.to_lowercase().contains(&query_lower) {
                    score += 0.5;
                }
                (score, id.clone())
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();
        // Sort by relevance score
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        self.last_query_results = results.into_iter().map(|(_, id)| id).take(5).collect();
    }
    /// Captures a snapshot of the current state for debugging and hot-reloading.
    pub fn snapshot(&self) -> Vec<NodeStateSnapshot> {
        let mut snapshots = Vec::new();
        // Snapshots of agentic fragments
        for frag in self.fragments.values() {
            if let Ok(val) = serde_json::to_value(frag) {
                snapshots.push(NodeStateSnapshot { id: 0, state: val });
            }
        }
        snapshots
    }
}
/// A read/write projection into a `State<T>` owned elsewhere.
#[derive(Clone)]
pub struct Binding<T: Clone + Send + Sync + 'static> {
    swap: Arc<arc_swap::ArcSwap<T>>,
    #[cfg(not(target_arch = "wasm32"))]
    tvar: Arc<stm::TVar<T>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}
impl<T: Clone + Send + Sync + 'static> Binding<T> {
    /// Create a binding from a State
    pub fn from_state(state: &State<T>) -> Self {
        Self {
            swap: Arc::clone(&state.swap),
            #[cfg(not(target_arch = "wasm32"))]
            tvar: Arc::clone(&state.tvar),
            version: Arc::clone(&state.version),
        }
    }
    /// Get the current value
    pub fn get(&self) -> T {
        (**self.swap.load()).clone()
    }
    /// Set a new value
    pub fn set(&self, value: T) {
        self.swap.store(Arc::new(value.clone()));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tvar = Arc::clone(&self.tvar);
            let v = value.clone();
            stm::atomically(move |tx| tvar.write(tx, v.clone()));
        }
        self.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    }
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Acquire)
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub fn transact_pair<A, B, F>(state_a: &State<A>, state_b: &State<B>, f: F)
where
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
    F: Fn(&A, &B) -> (A, B),
{
    let tvar_a = Arc::clone(&state_a.tvar);
    let tvar_b = Arc::clone(&state_b.tvar);
    let (new_a, new_b) = stm::atomically(move |tx| {
        let a = tvar_a.read(tx)?;
        let b = tvar_b.read(tx)?;
        let (na, nb) = f(&a, &b);
        tvar_a.write(tx, na.clone())?;
        tvar_b.write(tx, nb.clone())?;
        Ok((na, nb))
    });
    state_a.swap.store(Arc::new(new_a.clone()));
    state_b.swap.store(Arc::new(new_b.clone()));
    state_a.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    state_b.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    let subs_a = Arc::clone(&state_a.subscribers);
    let subs_b = Arc::clone(&state_b.subscribers);
    if crate::is_batching() {
        crate::enqueue_batch_task(Box::new(move || {
            {
                let s = subs_a.lock().unwrap();
                for cb in s.iter() { cb(&new_a); }
            }
            {
                let s = subs_b.lock().unwrap();
                for cb in s.iter() { cb(&new_b); }
            }
        }));
    } else {
        {
            let s = subs_a.lock().unwrap();
            for cb in s.iter() { cb(&new_a); }
        }
        {
            let s = subs_b.lock().unwrap();
            for cb in s.iter() { cb(&new_b); }
        }
    }
}
use std::any::TypeId;
use std::sync::Mutex;
/// Global environment storage using TypeId as keys.
pub(crate) static ENVIRONMENT: OnceLock<
    Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>,
> = OnceLock::new();
/// Environment key type for accessing ambient values
///
/// Implement this trait to define a new environment key.
pub trait EnvKey: 'static + Send + Sync {
    /// The type of value stored in the environment
    type Value: Clone + Send + Sync + 'static;
    /// Get a default value for this key
    fn default_value() -> Self::Value;
}
/// Key for accessing the Yggdrasil design token tree
pub struct YggdrasilKey;
impl EnvKey for YggdrasilKey {
    type Value = YggdrasilTokens;
    fn default_value() -> Self::Value {
        default_tokens()
    }
}
// Duplicate AssetKey removed - original definition at line 63
/// System appearance (Light/Dark mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Light,
    Dark,
}
/// Orientation for layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
/// Cross-axis alignment for layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Center,
    Leading,
    Trailing,
    Top,
    Bottom,
}
/// Main-axis distribution for linear layout containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Distribution {
    #[default]
    Fill,
    Center,
    Leading,
    Trailing,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
/// A color represented by RGBA components in the [0.0, 1.0] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const VIKING_GOLD: Color = Color {
        r: 1.0,
        g: 0.84,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA_LIQUID: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TACTICAL_OBSIDIAN: Color = Color {
        r: 0.05,
        g: 0.05,
        b: 0.07,
        a: 1.0,
    };
    /// Calculate the relative luminance of the color as defined by WCAG 2.x
    pub fn relative_luminance(&self) -> f32 {
        fn res(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * res(self.r) + 0.7152 * res(self.g) + 0.0722 * res(self.b)
    }
    /// Calculate the contrast ratio between this color and another color
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        if l1 > l2 {
            (l1 + 0.05) / (l2 + 0.05)
        } else {
            (l2 + 0.05) / (l1 + 0.05)
        }
    }
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };
    /// Create a new color from RGBA components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    /// Convert the color to a [r, g, b, a] array.
    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
impl View for Color {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, self.as_array());
    }
}
/// Key for accessing the current system appearance
pub struct AppearanceKey;
impl EnvKey for AppearanceKey {
    type Value = Appearance;
    fn default_value() -> Self::Value {
        Appearance::Dark // Default to Dark (Ginnungagap) for Berserker aesthetic
    }
}
/// StyleResolver provides high-level access to themed values from the environment.
pub struct StyleResolver;
impl StyleResolver {
    /// Resolve a color from the current environment
    pub fn color(key: &str) -> String {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens
            .get_color(key, is_dark)
            .unwrap_or_else(|| "#FF00FF".to_string()) // Default to MuspelMagenta on failure
    }
    /// Resolve a generic token value
    pub fn get<T: FromStr>(category: &str, key: &str) -> Option<T> {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens.get(category, key, is_dark)
    }
}
/// The authoritative Cyberpunk Viking default tokens
pub fn default_tokens() -> YggdrasilTokens {
    let mut tokens = YggdrasilTokens::new();
    // Core Norse Colorways
    tokens.color.insert(
        "background".to_string(),
        TokenValue::Single {
            value: "#000000".to_string(), // Ginnungagap (The Void)
        },
    );
    tokens.color.insert(
        "primary".to_string(),
        TokenValue::Single {
            value: "#00FFFF".to_string(), // NiflCyan (Aesir Primary)
        },
    );
    tokens.color.insert(
        "secondary".to_string(),
        TokenValue::Single {
            value: "#FF00FF".to_string(), // MuspelMagenta (Berserker Secondary)
        },
    );
    tokens.color.insert(
        "surface".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#121212".to_string(),
        },
    );
    tokens.color.insert(
        "text".to_string(),
        TokenValue::Adaptive {
            light: "#000000".to_string(),
            dark: "#FFFFFF".to_string(),
        },
    );
    // Bifrost (Glassmorphism) - Frosted Style
    tokens.bifrost.insert(
        "blur".to_string(),
        TokenValue::Single {
            value: "25.0".to_string(),
        },
    );
    tokens.bifrost.insert(
        "saturation".to_string(),
        TokenValue::Single {
            value: "1.2".to_string(),
        },
    );
    tokens.bifrost.insert(
        "opacity".to_string(),
        TokenValue::Single {
            value: "0.65".to_string(),
        },
    );
    // Gungnir (Neon Glow)
    tokens.gungnir.insert(
        "intensity".to_string(),
        TokenValue::Single {
            value: "1.0".to_string(),
        },
    );
    tokens.gungnir.insert(
        "radius".to_string(),
        TokenValue::Single {
            value: "15.0".to_string(),
        },
    );
    // Mjolnir (Sharp Geometry)
    tokens.mjolnir.insert(
        "clip_angle".to_string(),
        TokenValue::Single {
            value: "12.0".to_string(),
        },
    );
    tokens.mjolnir.insert(
        "border_width".to_string(),
        TokenValue::Single {
            value: "2.0".to_string(),
        },
    );
    // Sleipnir (Spring Animation)
    tokens.anim.insert(
        "stiffness".to_string(),
        TokenValue::Single {
            value: "170.0".to_string(),
        },
    );
    tokens.anim.insert(
        "damping".to_string(),
        TokenValue::Single {
            value: "26.0".to_string(),
        },
    );
    tokens.anim.insert(
        "mass".to_string(),
        TokenValue::Single {
            value: "1.0".to_string(),
        },
    );
    // Accessibility
    tokens.accessibility.insert(
        "reduce_motion".to_string(),
        TokenValue::Single {
            value: "false".to_string(),
        },
    );
    tokens
}
/// Environment wrapper for accessing ambient values
pub struct Environment<K: EnvKey> {
    _marker: std::marker::PhantomData<K>,
}
impl<K: EnvKey> Default for Environment<K> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: EnvKey> Environment<K> {
    /// Create a new Environment
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
    /// Get the current value from the environment
    pub fn get(&self) -> K::Value {
        if let Some(env_store) = ENVIRONMENT.get() {
            let env_lock = env_store.lock().unwrap();
            if let Some(val) = env_lock.get(&std::any::TypeId::of::<K>()) {
                if let Some(typed_val) = val.downcast_ref::<K::Value>() {
                    return typed_val.clone();
                } else {
                    log::warn!("Environment: Downcast failed for key type {:?}", std::any::type_name::<K>());
                }
            } else {
                log::debug!("Environment: Key not found: {:?}. Returning default.", std::any::type_name::<K>());
            }
        } else {
            log::debug!("Environment: Store not initialized. Key: {:?}. Returning default.", std::any::type_name::<K>());
        }
        K::default_value()
    }
}
/// Ambient environment management
pub mod env {
    /// Insert a value into the environment
    pub fn insert<K: super::EnvKey>(value: K::Value) {
        let store = super::ENVIRONMENT.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        let mut env_map = store.lock().unwrap();
        env_map.insert(std::any::TypeId::of::<K>(), Box::new(value));
    }
    /// Remove a value from the environment.
    pub fn remove<K: super::EnvKey>() {
        if let Some(store) = super::ENVIRONMENT.get() {
            let mut env_map = store.lock().unwrap();
            env_map.remove(&std::any::TypeId::of::<K>());
        }
    }
}
/// Geometry modifiers
/// Size of the view in logical pixels
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };

    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Insets for padding
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub leading: f32,
    pub bottom: f32,
    pub trailing: f32,
}

impl EdgeInsets {
    /// Equal insets on all edges
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            leading: value,
            bottom: value,
            trailing: value,
        }
    }

    /// Vertical insets (top and bottom)
    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            leading: 0.0,
            bottom: value,
            trailing: 0.0,
        }
    }

    /// Horizontal insets (leading and trailing)
    pub fn horizontal(value: f32) -> Self {
        Self {
            top: 0.0,
            leading: value,
            bottom: 0.0,
            trailing: value,
        }
    }
}

/// Modifier to set the size of a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameModifier {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for FrameModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameModifier {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

impl ViewModifier for FrameModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to set the flex weight of a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexModifier {
    pub weight: f32,
}

impl ViewModifier for FlexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn child_flex_weight<V: View>(&self, _view: &V) -> f32 {
        self.weight
    }
}

/// Modifier to offset a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetModifier {
    pub x: f32,
    pub y: f32,
}

impl OffsetModifier {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl ViewModifier for OffsetModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to set the z-index of a view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZIndexModifier {
    pub z_index: i32,
}

impl ZIndexModifier {
    pub fn new(z_index: i32) -> Self {
        Self { z_index }
    }
}

impl ViewModifier for ZIndexModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Layout constraints for views
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

/// Modifier to set layout constraints
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutModifier {
    pub constraints: LayoutConstraints,
}

impl LayoutModifier {
    pub fn new(constraints: LayoutConstraints) -> Self {
        Self { constraints }
    }
}

impl ViewModifier for LayoutModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to handle platform safe areas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeAreaModifier {
    pub ignores: bool,
}

impl ViewModifier for SafeAreaModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }
}

/// Modifier to add elevation (shadow) to a view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationModifier {
    pub level: f32,
}

impl ViewModifier for ElevationModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if self.level > 0.0 {
            let radius = self.level * 2.0;
            let offset_y = self.level * 0.5;
            let shadow_color = [0.0, 0.0, 0.0, 0.3];
            renderer.push_shadow(radius, shadow_color, [0.0, offset_y]);
            view.render(renderer, rect);
            renderer.pop_shadow();
        } else {
            view.render(renderer, rect);
        }
    }
}

// Layout subsystem
pub mod layout {
    use super::*;

    // Layout pass scratch space
    pub struct LayoutCache {
        pub safe_area: SafeArea,
        size_cache: HashMap<(u64, u32, u32), Size>, // (ViewHash, ProposalW, ProposalH)
    }

    impl Default for LayoutCache {
        fn default() -> Self {
            Self::new()
        }
    }

    impl LayoutCache {
        pub fn new() -> Self {
            Self {
                safe_area: SafeArea::default(),
                size_cache: HashMap::new(),
            }
        }

        pub fn clear(&mut self) {
            self.safe_area = SafeArea::default();
            self.size_cache.clear();
        }

        pub fn get_size(&self, view_hash: u64, proposal: SizeProposal) -> Option<Size> {
            let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
            let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
            self.size_cache.get(&(view_hash, pw, ph)).copied()
        }

        pub fn set_size(&mut self, view_hash: u64, proposal: SizeProposal, size: Size) {
            let pw = (proposal.width.unwrap_or(-1.0) * 100.0) as u32;
            let ph = (proposal.height.unwrap_or(-1.0) * 100.0) as u32;
            self.size_cache.insert((view_hash, pw, ph), size);
        }

        /// Remove all cached size entries for a specific view hash.
        pub fn invalidate_view(&mut self, view_hash: u64) {
            self.size_cache.retain(|&(hash, _, _), _| hash != view_hash);
        }
    }

    /// Proposed size from parent view
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct SizeProposal {
        pub width: Option<f32>,
        pub height: Option<f32>,
    }

    impl SizeProposal {
        pub fn unspecified() -> Self {
            Self {
                width: None,
                height: None,
            }
        }

        pub fn width(width: f32) -> Self {
            Self {
                width: Some(width),
                height: None,
            }
        }

        pub fn height(height: f32) -> Self {
            Self {
                width: None,
                height: Some(height),
            }
        }

        pub fn tight(width: f32, height: f32) -> Self {
            Self {
                width: Some(width),
                height: Some(height),
            }
        }

        pub fn new(width: Option<f32>, height: Option<f32>) -> Self {
            Self { width, height }
        }
    }

    /// A view that can participate in layout
    pub trait LayoutView: Send {
        /// Propose a size for this view given the available space
        fn size_that_fits(
            &self,
            proposal: SizeProposal,
            subviews: &[&dyn LayoutView],
            cache: &mut LayoutCache,
        ) -> Size;

        /// Place subviews within the given bounds
        fn place_subviews(
            &self,
            bounds: Rect,
            subviews: &mut [&mut dyn LayoutView],
            cache: &mut LayoutCache,
        );

        /// Returns the flex weight of this view (default is 0.0, which means fixed/intrinsic)
        fn flex_weight(&self) -> f32 {
            0.0
        }
    }
    /// Edge insets for padding, margins, and safe areas
    #[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
    pub struct EdgeInsets {
        pub top: f32,
        pub leading: f32,
        pub bottom: f32,
        pub trailing: f32,
    }

    impl EdgeInsets {
        pub fn new(top: f32, leading: f32, bottom: f32, trailing: f32) -> Self {
            Self { top, leading, bottom, trailing }
        }

        pub fn all(value: f32) -> Self {
            Self {
                top: value,
                leading: value,
                bottom: value,
                trailing: value,
            }
        }
    }

    /// SafeArea constraints provided by the platform
    #[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
    pub struct SafeArea {
        pub insets: EdgeInsets,
    }

    /// Rectangle in logical pixels
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Rect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    impl Rect {
        pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                x,
                y,
                width,
                height,
            }
        }

        pub fn inset(&self, amount: f32) -> Self {
            Self {
                x: self.x + amount,
                y: self.y + amount,
                width: (self.width - amount * 2.0).max(0.0),
                height: (self.height - amount * 2.0).max(0.0),
            }
        }

        pub fn offset(&self, dx: f32, dy: f32) -> Self {
            Self {
                x: self.x + dx,
                y: self.y + dy,
                ..*self
            }
        }

        pub fn zero() -> Self {
            Self {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }
        }

        pub fn contains(&self, x: f32, y: f32) -> bool {
            x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
        }

        pub fn size(&self) -> Size {
            Size {
                width: self.width,
                height: self.height,
            }
        }

        /// Split the rect horizontally into N equal pieces
        pub fn split_horizontal(&self, n: usize) -> Vec<Rect> {
            if n == 0 {
                return vec![];
            }
            let item_width = self.width / n as f32;
            (0..n)
                .map(|i| Rect {
                    x: self.x + i as f32 * item_width,
                    y: self.y,
                    width: item_width,
                    height: self.height,
                })
                .collect()
        }

        /// Split the rect vertically into N equal pieces
        pub fn split_vertical(&self, n: usize) -> Vec<Rect> {
            if n == 0 {
                return vec![];
            }
            let item_height = self.height / n as f32;
            (0..n)
                .map(|i| Rect {
                    x: self.x,
                    y: self.y + i as f32 * item_height,
                    width: self.width,
                    height: item_height,
                })
                .collect()
        }
    }
}

// Re-export layout items for convenience
pub use layout::{LayoutCache, LayoutView, Rect, SizeProposal};
// Size and FrameRenderer are pub items in this module; no re-export alias needed.

pub mod runtime;
pub mod scene_graph;
pub mod agents;
pub mod material;


pub use scene_graph::{NodeId, bifrost_registry};

// Duplicate AssetState removed - original definition at line 67

/// AssetManager defines the interface for loading and caching external resources.
pub trait AssetManager: Send + Sync {
    /// Request an image asset. Returns the current state (Loading, Ready, or Error).
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>>;

    /// Pre-load an image into the cache.
    fn preload_image(&self, url: &str);
}

/// User input event types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    PointerDown { x: f32, y: f32 },
    PointerUp { x: f32, y: f32 },
    PointerMove { x: f32, y: f32 },
    PointerClick { x: f32, y: f32 },
    PointerEnter,
    PointerLeave,
    KeyDown { key: String },
    KeyUp { key: String },
    /// Input Method Editor event (e.g. CJK character composition)
    Ime(String),
}

impl Event {
    /// Returns the canonical string name of the event for lookup in handler maps.
    pub fn name(&self) -> &'static str {
        match self {
            Self::PointerDown { .. } => "pointerdown",
            Self::PointerUp { .. } => "pointerup",
            Self::PointerMove { .. } => "pointermove",
            Self::PointerClick { .. } => "pointerclick",
            Self::PointerEnter => "pointerenter",
            Self::PointerLeave => "pointerleave",
            Self::KeyDown { .. } => "keydown",
            Self::KeyUp { .. } => "keyup",
            Self::Ime(_) => "ime",
        }
    }
}

/// Response from an event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    Handled,
    Ignored,
}

/// A basic implementation of AssetManager that can be overridden by platform backends.
pub struct DefaultAssetManager {
    cache: Arc<arc_swap::ArcSwap<HashMap<String, AssetState<Arc<Vec<u8>>>>>>,
}

impl Default for DefaultAssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAssetManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(arc_swap::ArcSwap::from_pointee(HashMap::new())),
        }
    }
}

impl AssetManager for DefaultAssetManager {
    fn load_image(&self, url: &str) -> AssetState<Arc<Vec<u8>>> {
        if let Some(state) = self.cache.load().get(url) {
            return state.clone();
        }

        self.cache.rcu(|map| {
            let mut m = (**map).clone();
            m.entry(url.to_string()).or_insert(AssetState::Loading);
            m
        });
        AssetState::Loading
    }

    fn preload_image(&self, _url: &str) {}
}

use std::future::Future;

/// Suspense wrapper for asynchronous state management.
/// Integrates with State<T> to provide loading/error/ready states for async operations.
pub struct Suspense<T: Clone + Send + Sync + 'static> {
    inner: State<AssetState<T>>,
}

impl<T: Clone + Send + Sync + 'static> Default for Suspense<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> Suspense<T> {
    pub fn new() -> Self {
        Self {
            inner: State::new(AssetState::Loading),
        }
    }

    pub fn new_async<F>(future: F) -> Self
    where
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let suspense = Self::new();
        let suspense_clone = suspense.clone();
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Try to use an existing tokio runtime, or fallback to a dedicated thread
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let result = future.await;
                    match result {
                        Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                        Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                    }
                });
            } else {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    rt.block_on(async {
                        let result = future.await;
                        match result {
                            Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                            Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                        }
                    });
                });
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let result = future.await;
                match result {
                    Ok(val) => suspense_clone.inner.set(AssetState::Ready(val)),
                    Err(err) => suspense_clone.inner.set(AssetState::Error(err)),
                }
            });
        }
        
        suspense
    }

    pub fn ready(value: T) -> Self {
        Self {
            inner: State::new(AssetState::Ready(value)),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            inner: State::new(AssetState::Error(message.into())),
        }
    }

    pub fn get(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn get_ref(&self) -> AssetState<T> {
        self.inner.get()
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.get(), AssetState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.get(), AssetState::Ready(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.get(), AssetState::Error(_))
    }

    pub fn ready_value(&self) -> Option<T> {
        match self.get() {
            AssetState::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn error_message(&self) -> Option<String> {
        match self.get() {
            AssetState::Error(message) => Some(message),
            _ => None,
        }
    }

    pub fn subscribe<F: Fn(&AssetState<T>) + Send + Sync + 'static>(&self, callback: F) {
        self.inner.subscribe(callback)
    }

    pub fn inner_state(&self) -> &State<AssetState<T>> {
        &self.inner
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Suspense<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<T> for Suspense<T> {
    fn from(value: T) -> Self {
        Self::ready(value)
    }
}

impl<T: Clone + Send + Sync + 'static> From<Result<T, String>> for Suspense<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(value) => Self::ready(value),
            Err(error) => Self::error(error),
        }
    }
}

#[cfg(test)]
mod phase1_test;

/// Berserker mode states for the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BerserkerMode {
    Normal,
    Rage,     // Red tint, slight shake
    Frenzy,   // Heavy red tint, motion blur, aggressive shake
    GodMode,  // Golden aura, lightning arcs
}

/// Seer trait for AI-assisted UI components (inspired by Argmax OSS).
/// Allows components to receive "prophecies" (predictions) from an AI backend.
pub trait Seer: Send + Sync {
    /// Provide a prediction for the next user action or content.
    fn predict(&self, context: &str) -> String;
    /// Stream real-time "whispers" (transcriptions/intent).
    fn whispers(&self) -> Vec<String>;
}

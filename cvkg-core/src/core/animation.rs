// =============================================================================
// REDUCED MOTION
// =============================================================================

/// Detects OS-level reduced motion preference via [`AccessibilityPreferences`].
///
/// This delegates to `AccessibilityPreferences::detect_from_system()` which
/// queries the correct OS API on macOS, Linux, and Windows.
pub fn is_reduced_motion() -> bool {
    AccessibilityPreferences::detect_from_system().reduce_motion
}

/// Returns effective animation duration (0.0 if reduced motion is active).
pub fn effective_duration(secs: f32) -> f32 {
    if is_reduced_motion() { 0.0 } else { secs }
}

/// An object-safe version of the View trait for type erasure.
pub trait ErasedView: Send {
    fn render_erased(&self, renderer: &mut dyn Renderer, rect: Rect);
    fn name(&self) -> &'static str;
    fn flex_weight_erased(&self) -> f32;
    fn layout_erased(&self) -> Option<&dyn layout::LayoutView>;
    fn grid_placement_erased(&self) -> Option<GridPlacement>;
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

    fn grid_placement_erased(&self) -> Option<GridPlacement> {
        self.get_grid_placement()
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
        // SAFETY: `Never` is uninhabitable (zero variants). MemoView renders via
        // `render()` using the memoized builder closure and never exposes a body.
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
        // SAFETY: `Never` is uninhabitable. AnyView is a type-erased wrapper that
        // renders via `render_erased()` and never exposes a composable body.
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

    fn get_grid_placement(&self) -> Option<GridPlacement> {
        self.inner.grid_placement_erased()
    }
}

/// SharedElementModifier enables shared-element transitions.
/// When two views share the same Bifrost Bridge ID, the Sleipnir solver will
/// interpolate their geometry and effects (blur, glow) during the transition.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedElementModifier {
    pub id: String,
}

impl ViewModifier for SharedElementModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Register this element with the renderer for shared-element transition logic
        renderer.register_shared_element(&self.id, rect);
    }
}

/// GeometricClipModifier implements the "Geometric Slice" aesthetic.
/// It uses a signed distance field (SDF) to clip the view along a sharp angled line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometricClipModifier {
    pub angle: f32,
    pub offset: f32,
}

impl ViewModifier for GeometricClipModifier {
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

/// FragmentModifier implements the "Shattering" effect.
/// It breaks the view into discrete geometric fragments that can be animated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FragmentModifier {
    pub pieces: u32,
    pub force: f32,
}

impl ViewModifier for FragmentModifier {
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

/// FrostedGlassModifier implements the Cyberpunk "Frosted Glass" aesthetic.
/// It triggers backdrop blurring and light scattering in the render pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrostedGlassModifier {
    pub blur: f32,
    pub saturation: f32,
    pub opacity: f32,
    /// Fresnel strength multiplier. 0.0 = no fresnel, 1.0 = full.
    pub fresnel_strength: f32,
}

impl ViewModifier for FrostedGlassModifier {
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

/// NeonGlowModifier implements the "Neon Glow" aesthetic.
/// It uses additive blending and multi-pass blurring to simulate glowing light.
#[derive(Debug, Clone, PartialEq)]
pub struct NeonGlowModifier {
    pub color: String,
    pub radius: f32,
    pub intensity: f32,
}

impl ViewModifier for NeonGlowModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Neon Glow using Mode 1 in the Surtr pipeline
        renderer.stroke_rect(rect, [0.0, 1.0, 1.0, self.intensity], self.radius / 10.0);
    }
}

/// PulsingGlowModifier implements a "breathing" neon effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulsingGlowModifier {
    pub color: [f32; 4],
    pub radius: f32,
    pub speed: f32,
}

impl ViewModifier for PulsingGlowModifier {
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

/// MagneticPullModifier makes a view "magnetic", subtly leaning towards or pulling the cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagneticPullModifier {
    pub radius: f32,
    pub intensity: f32,
}

impl ViewModifier for MagneticPullModifier {
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

/// CursorGlowModifier adds a soft, lunar-like cursor glow to a view.
/// Named after Máni, the personification of the Moon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorGlowModifier {
    pub color: [f32; 4],
    pub radius: f32,
}

impl ViewModifier for CursorGlowModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if crate::load_system_state().realm == UiFidelityLevel::Asgard {
            renderer.mani_glow(rect, self.color, self.radius);
        }
        view.render(renderer, rect);
    }
}

/// MemoryLayerModifier themes a view based on its cognitive memory layer.
/// Episodic: Shifting aurora clouds.
/// Semantic: Crystalline gold.
/// Procedural: Heavy obsidian stone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryLayerModifier {
    pub layer: MemoryLayer,
}

impl ViewModifier for MemoryLayerModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let realm = crate::load_system_state().realm;
        match self.layer {
            MemoryLayer::Episodic => {
                if realm == UiFidelityLevel::Asgard {
                    renderer.bifrost(rect, 40.0, 1.2, 0.7);
                } else {
                    renderer.fill_rect(rect, [0.1, 0.12, 0.15, 0.8]);
                }
            }
            MemoryLayer::Semantic => {
                if realm == UiFidelityLevel::Asgard {
                    renderer.gungnir(rect, [1.0, 0.84, 0.0, 1.0], 15.0, 0.6);
                } else {
                    renderer.stroke_rect(rect, [0.4, 0.4, 0.4, 1.0], 1.5);
                }
            }
            MemoryLayer::Procedural => {
                renderer.fill_rect(rect, [0.05, 0.05, 0.07, 0.95]);
                let stroke_color = if realm == UiFidelityLevel::Asgard {
                    [0.3, 0.3, 0.3, 1.0]
                } else {
                    [0.2, 0.2, 0.2, 1.0]
                };
                renderer.stroke_rect(rect, stroke_color, 2.0);
            }
        }
        view.render(renderer, rect);
    }
}

/// EvolvingInteractionModifier enables self-evolving UI capabilities.
/// Named after Fafnir, the dragon who grows in power based on the gold he hoards.
/// In CVKG, 'Gold' is user attention/interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvolvingInteractionModifier {
    /// Unique ID for tracking this component's vitality across frames.
    pub id: u64,
}

impl ViewModifier for EvolvingInteractionModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = crate::load_system_state();
        let vitality = state
            .get_component_state::<f32>(self.id)
            .map(|v| *v.read().unwrap())
            .unwrap_or(1.0);

        // Calculate evolutionary growth factors
        // Max growth at vitality 5.0 (50% scale increase, strong glow)
        let growth = (vitality - 1.0).clamp(0.0, 4.0);
        let scale = 1.0 + growth * 0.12;
        let glow_intensity = growth * 0.25;

        // Feed Fafnir: Register interaction to boost vitality
        let id = self.id;
        renderer.register_handler(
            "pointermove",
            std::sync::Arc::new(move |_| {
                crate::update_system_state(|s| {
                    let mut s = s.clone();
                    let v = s
                        .get_component_state::<f32>(id)
                        .map(|v| *v.read().unwrap())
                        .unwrap_or(1.0);
                    s.set_component_state(id, (v + 0.05).min(5.0)); // Cap at 5.0
                    s
                });
            }),
        );

        if scale > 1.01 {
            renderer.push_transform([0.0, 0.0], [scale, scale], 0.0);
        }

        if glow_intensity > 0.1 && state.realm == UiFidelityLevel::Asgard {
            renderer.gungnir(rect, [1.0, 0.84, 0.0, 1.0], 15.0 * vitality, glow_intensity);
        }

        view.render(renderer, rect);

        if scale > 1.01 {
            renderer.pop_transform();
        }
    }
}

/// IntentPredictionModifier anticipates user movement and manifests holographic ghosts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntentPredictionModifier;

impl ViewModifier for IntentPredictionModifier {
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
        let speed_sq = vel[0] * vel[0] + vel[1] * vel[1];
        let dist_sq = dx * dx + dy * dy;

        if dot > 0.0 && dist_sq < 250.0 * 250.0 && speed_sq > 0.5 && state.realm == UiFidelityLevel::Asgard {
            // Intent detected: render a subtle "ghost" reveal
            let intent_strength = (dot / (speed_sq.sqrt() * dist_sq.sqrt())).clamp(0.0, 1.0);
            renderer.stroke_rect(rect, [0.0, 0.9, 1.0, 0.3 * intent_strength], 1.5);
        }

        view.render(renderer, rect);
    }
}

/// ComplexityTelemetryModifier renders a cognitive telemetry cloud representing agent complexity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexityTelemetryModifier {
    pub complexity: f32,
}

impl ViewModifier for ComplexityTelemetryModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        if crate::load_system_state().realm == UiFidelityLevel::Asgard {
            let t = renderer.elapsed_time();
            let c = self.complexity.clamp(0.0, 1.0);

            // 1. Core Cognitive Cloud (Bifrost)
            // Turbulence increases with complexity
            let blur = 20.0 + c * 40.0;
            let turbulence_x = (t * (1.0 + c * 2.0)).sin() * 8.0 * c;
            let turbulence_y = (t * (0.8 + c * 1.5)).cos() * 5.0 * c;
            renderer.bifrost(
                rect.offset(turbulence_x, turbulence_y),
                blur,
                0.8 + c * 0.4,
                0.25,
            );

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

/// ObservabilityOverlayModifier bestows omniscient observability over the entire scene graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObservabilityOverlayModifier;

impl ViewModifier for ObservabilityOverlayModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = crate::load_system_state();
        let t = renderer.elapsed_time();

        // 1. Render Background content
        view.render(renderer, rect);

        if state.realm == UiFidelityLevel::Asgard {
            // 2. Bestow Odin's Eye (Atmospheric Overlay)
            // Soft, large circular pulse representing the 'Eye'
            let eye_pulse = (t * 0.5).sin().abs() * 0.05;
            renderer.draw_radial_gradient(
                rect,
                [0.0, 0.6, 0.8, 0.08 + eye_pulse], // Inner Cyan
                [0.0, 0.0, 0.0, 0.0],              // Outer Black
            );

            // 3. Hugin (Thought) Telemetry - Left Side
            let hugin_rect = Rect {
                x: rect.x + 20.0,
                y: rect.y + 40.0,
                width: 200.0,
                height: rect.height - 80.0,
            };
            renderer.draw_text(
                "HUGIN: THOUGHT",
                hugin_rect.x,
                hugin_rect.y,
                10.0,
                [0.0, 1.0, 1.0, 0.6],
            );
            for (i, thought) in state.thoughts.iter().rev().take(10).enumerate() {
                renderer.draw_text(
                    thought,
                    hugin_rect.x,
                    hugin_rect.y + 20.0 + i as f32 * 14.0,
                    9.0,
                    [1.0, 1.0, 1.0, 0.4],
                );
            }

            // 4. Munin (Memory) Telemetry - Right Side
            let munin_rect = Rect {
                x: rect.x + rect.width - 220.0,
                y: rect.y + 40.0,
                width: 200.0,
                height: rect.height - 80.0,
            };
            renderer.draw_text(
                "MUNIN: MEMORY",
                munin_rect.x,
                munin_rect.y,
                10.0,
                [1.0, 0.84, 0.0, 0.6],
            );
            for (i, node) in state.nodes.iter().take(10).enumerate() {
                let opacity = (node.weight.min(1.0)) * 0.5;
                renderer.draw_text(
                    &node.id,
                    munin_rect.x,
                    munin_rect.y + 20.0 + i as f32 * 14.0,
                    9.0,
                    [1.0, 1.0, 1.0, opacity],
                );
            }

            // 5. Omniscient Focus Beams (Gungnir Beams)
            if let Some(focus_id) = &state.odin_focus {
                // Visualize causal links to the focus node
                renderer.draw_text(
                    &format!("EYE FOCUS: {}", focus_id),
                    rect.x + rect.width / 2.0 - 50.0,
                    rect.y + 20.0,
                    12.0,
                    [0.0, 1.0, 1.0, 0.8],
                );

                // In a real implementation, we would find the rect of the focus_id component.
                // For the 'Eye', we manifest a central beam of wisdom.
                renderer.gungnir(
                    Rect {
                        x: rect.x + rect.width / 2.0 - 1.0,
                        y: rect.y,
                        width: 2.0,
                        height: rect.height,
                    },
                    [0.0, 1.0, 1.0, 1.0],
                    20.0,
                    0.4,
                );
            }
        }
    }
}

/// Sleipnir spring parameters for the physics solver
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SpringParams {
    pub fn snappy() -> Self {
        Self {
            stiffness: 230.0,
            damping: 22.0,
            mass: 1.0,
        }
    }
    pub fn fluid() -> Self {
        Self {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
    pub fn heavy() -> Self {
        Self {
            stiffness: 90.0,
            damping: 20.0,
            mass: 1.0,
        }
    }
    pub fn bouncy() -> Self {
        Self {
            stiffness: 190.0,
            damping: 14.0,
            mass: 1.0,
        }
    }
}

impl Default for SpringParams {
    fn default() -> Self {
        Self::fluid()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolverState {
    x: f32,
    v: f32,
}

/// SpringSolver implements a 4th-order Runge-Kutta (RK4) integration for springs.
/// This provides superior stability for high-fidelity interactive motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSolver {
    params: SpringParams,
    target: f32,
    state: SolverState,
}

impl SpringSolver {
    /// Create a new solver with a target value and starting state.
    pub fn new(params: SpringParams, target: f32, current: f32) -> Self {
        Self {
            params,
            target,
            state: SolverState { x: current, v: 0.0 },
        }
    }

    /// Advance the simulation by dt seconds using RK4 integration.
    pub fn tick(&mut self, dt: f32) -> f32 {
        if dt <= 0.0 {
            return self.state.x;
        }

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
        let force =
            -self.params.stiffness * (state.x - self.target) - self.params.damping * state.v;
        let mass = self.params.mass.max(0.001);
        SolverState {
            x: state.v,
            v: force / mass,
        }
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

/// SpringAnimationModifier handles physics-based animations via the Sleipnir RK4 solver.
#[derive(Debug, Clone, PartialEq)]
pub struct SpringAnimationModifier {
    pub id: u64,
    pub target: f32,
    pub params: SpringParams,
}

impl ViewModifier for SpringAnimationModifier {
    fn modify<V: View>(self, content: V) -> impl View {
        ModifiedView::new(content, self)
    }

    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        let state = load_system_state();

        // Try to fetch the solver from persistent state.
        let solver_lock_opt = state.get_component_state::<SpringSolver>(self.id);

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
            let solver = SpringSolver::new(
                self.params,
                self.target,
                self.target, // Initialize at target to avoid jump on first frame
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
                if let Event::PointerMove { x, y, .. } = event {
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
        // SAFETY: `Never` is an uninhabitable enum (zero variants), so this function
        // can never be called with a valid value and can never return.
        unreachable!()
    }
}

/// EmptyView - A view that renders nothing and takes up no space.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyView;

impl View for EmptyView {
    type Body = Never;
    fn body(self) -> Self::Body {
        // SAFETY: `Never` is uninhabitable. EmptyView renders nothing and has no
        // composable body -- it registers zero paint commands.
        unreachable!()
    }
    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {}
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// A view that has been transformed by a modifier.
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

    fn get_grid_placement(&self) -> Option<GridPlacement> {
        self.modifier
            .get_grid_placement()
            .or_else(|| self.view.get_grid_placement())
    }
}

pub trait ViewModifier: Send + Clone {
    fn modify<V: View>(self, content: V) -> impl View;

    /// Returns the grid placement configuration if this modifier defines one.
    fn get_grid_placement(&self) -> Option<GridPlacement> {
        None
    }

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
    fn measure_view<V: View>(
        &self,
        view: &V,
        renderer: &mut dyn Renderer,
        proposal: SizeProposal,
    ) -> Size {
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

/// Captures the depth of every stack-pushing operation on the `Renderer`.
///
/// Created via `Renderer::snapshot_render_state()` and consumed by
/// `Renderer::restore_render_state()`. The renderer uses this to recover
/// from mid-render panics -- any items pushed beyond the snapshot point
/// are popped so sibling views drawn afterward don't inherit leaked
/// clip / opacity / transform / shadow / vnode / mjolnir-slice state.
///
/// Frame-scoped: the renderer resets all stacks in `begin_frame()` so a
/// snapshot taken in one frame is meaningless in another.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderStateSnapshot {
    pub clip_depth: u32,
    pub opacity_depth: u32,
    pub slice_depth: u32,
    pub shadow_depth: u32,
    pub transform_depth: u32,
    pub vnode_depth: u32,
}

/// TelemetryData tracks real-time performance metrics for the GPU renderer.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TelemetryData {
    pub frame_time_ms: f32,
    /// Total frame budget in milliseconds for the active policy.
    pub frame_budget_ms: f32,
    /// Remaining frame budget after the frame completed; negative means over budget.
    pub frame_budget_remaining_ms: f32,
    /// Remaining layout budget after layout completed; negative means the layout slice was exceeded.
    pub layout_budget_remaining_ms: f32,
    /// Whether the frame exceeded the total budget.
    pub frame_over_budget: bool,
    /// Whether the layout phase exceeded its budget slice.
    pub layout_over_budget: bool,
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

    /// Global Berserker Pipeline Intensity (0.0 - 1.0+)
    pub berserker_rage: f32,

    // Memory breakdown
    pub vram_usage_mb: f32,
    pub vram_textures_mb: f32,
    pub vram_buffers_mb: f32,
    pub vram_pipelines_mb: f32,
    /// Indicates if the Mega-Atlas or VRAM pools are at capacity.
    pub vram_exhausted: bool,
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
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
///    Trait providing timing information for the render loop.
pub trait ElapsedTime {
    /// Returns the cumulative time since the renderer started in seconds.
    fn elapsed_time(&self) -> f32;

    /// Returns the time elapsed since the last frame in seconds.
    fn delta_time(&self) -> f32;
}

/// The Renderer trait defines the atomic drawing operations for all CVKG backends.
/// This trait is object-safe and used by the View::render system.
/// # Implementation Requirements
/// 1. Coordinate system is origin-top-left (0,0) with Y increasing downwards.
/// 2. Colors are [R, G, B, A] in the [0.0, 1.0] range.
/// 3. All operations must be batchable by the underlying backend.
///
/// Sub-traits in `renderer/mod.rs` (RendererCore, RendererShapes, etc.) provide
/// logical groupings for consumer code. Backends implement this monolithic trait;
/// the sub-traits are aspirational documentation and NOT enforced as supertraits
/// to avoid method ambiguity (sub-traits re-declare the same methods).
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

    /// Draw a background image that fills the entire rect.
    /// This is a convenience wrapper around `draw_image` for the common case
    /// of a full-rect background. The image must have been pre-warmed via
    /// `prewarm_vram` before the first frame.
    fn draw_background_image(&mut self, image_name: &str, rect: Rect) {
        Renderer::draw_image(self, image_name, rect);
    }

    /// Fill a rounded rect with glass material for frosted backdrop effect.
    /// This is the proper way to render glass cards for macOS Tahoe-style blur.
    /// The blur_radius controls the intensity of the backdrop blur.
    fn fill_glass_rect(&mut self, rect: Rect, radius: f32, blur_radius: f32) {
        // Default no-op implementation; GPU backend overrides
        let _ = (rect, radius, blur_radius);
    }
    /// Fill a rounded rect with glass material with explicit intensity control.
    /// `glass_intensity` ranges from 0.0 (solid) to 1.0 (full glass). Default: 1.0.
    fn fill_glass_rect_with_intensity(&mut self, rect: Rect, radius: f32, blur_radius: f32, glass_intensity: f32) {
        let _ = (rect, radius, blur_radius, glass_intensity);
    }
    /// Fill a rounded rect with glass material with explicit tint color and intensity.
    /// `tint_color` is the glass fill color (RGBA). `glass_intensity` ranges from 0.0 (solid) to 1.0 (full glass).
    fn fill_glass_rect_with_tint(&mut self, rect: Rect, radius: f32, blur_radius: f32, tint_color: [f32; 4], glass_intensity: f32) {
        // Default: delegate to intensity-only version using tint color as a simple fill
        let _ = (rect, radius, blur_radius, tint_color, glass_intensity);
    }
    /// Fill a rounded rect with glass material, modulated by touch pressure.
    /// `pressure` ranges from 0.0 (no touch) to 1.0 (full pressure).
    /// When pressure > 0, refraction distortion is scaled by pressure amount.
    /// Desktop stub: pressure is always 1.0 for mouse clicks, 0.0 otherwise.
    fn fill_glass_rect_with_pressure(&mut self, rect: Rect, radius: f32, blur_radius: f32, pressure: f32) {
        // Default: delegate to standard glass with intensity = pressure
        Renderer::fill_glass_rect_with_intensity(self, rect, radius, blur_radius, pressure);
    }

    /// Fill a squircle (superellipse) for Apple-style icon silhouettes.
    /// `n` controls the squareness: 2.0 = rounded rect, 4.0 = classic squircle, higher = more square.
    fn fill_squircle(&mut self, rect: Rect, _n: f32, color: [f32; 4]) {
        // Default fallback to rounded rect
        Renderer::fill_rounded_rect(self, rect, rect.width.min(rect.height) * 0.22, color);
    }

    /// Stroke a squircle (superellipse) outline.
    fn stroke_squircle(&mut self, rect: Rect, _n: f32, color: [f32; 4], stroke_width: f32) {
        Renderer::stroke_rounded_rect(self, rect, rect.width.min(rect.height) * 0.22, color, stroke_width);
    }

    /// Draw a focus ring around a rect (for keyboard navigation accessibility).
    /// `offset` is the gap between the rect and the ring, `width` is the ring thickness.
    fn draw_focus_ring(&mut self, rect: Rect, radius: f32, offset: f32, width: f32, color: [f32; 4]) {
        // Default fallback to a stroked rounded rect
        let ring_rect = Rect {
            x: rect.x - offset,
            y: rect.y - offset,
            width: rect.width + 2.0 * offset,
            height: rect.height + 2.0 * offset,
        };
        Renderer::stroke_rounded_rect(self, ring_rect, radius + offset, color, width);
    }

    /// Draw a high-fidelity 3D cube inside the given rectangle using specialized shader logic.
    /// `rotation` is [pitch, yaw, roll] in radians.
    fn draw_3d_cube(&mut self, _rect: Rect, _color: [f32; 4], _rotation: [f32; 3]) {}

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
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        let span = cvkg_runic_text::TextSpan::new(
            text,
            cvkg_runic_text::TextStyle {
                family: "Inter".to_string(),
                font_size: size,
                color: [(color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8, (color[3]*255.0) as u8],
                fallback_families: vec![
                    "SF Pro".to_string(),
                    "SF Pro Text".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ],
                ..Default::default()
            },
        );
        if let Some(shaped) = Renderer::shape_rich_text(
            self,
            &[span],
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            Renderer::draw_shaped_text(self, &shaped, x, y);
        }
    }

    /// Measure the width and height of the specified text.
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        let span = cvkg_runic_text::TextSpan::new(
            text,
            cvkg_runic_text::TextStyle {
                family: "Inter".to_string(),
                font_size: size,
                fallback_families: vec![
                    "SF Pro".to_string(),
                    "SF Pro Text".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ],
                ..Default::default()
            },
        );
        if let Some(shaped) = Renderer::shape_rich_text(
            self,
            &[span],
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            let scale = self.text_scale_factor().max(1.0);
            (shaped.width / scale, shaped.height / scale)
        } else {
            (0.0, 0.0)
        }
    }

    /// Return the baseline offset (ascent) for the given text and size.
    /// This is the distance from the text origin (y in draw_text) to the baseline.
    /// Default returns 0.0; override in renderers that support text shaping.
    fn measure_text_baseline(&mut self, text: &str, size: f32) -> f32 {
        let span = cvkg_runic_text::TextSpan::new(
            text,
            cvkg_runic_text::TextStyle {
                family: "Inter".to_string(),
                font_size: size,
                fallback_families: vec![
                    "SF Pro".to_string(),
                    "SF Pro Text".to_string(),
                    "Helvetica Neue".to_string(),
                    "Helvetica".to_string(),
                    "Arial".to_string(),
                    "sans-serif".to_string(),
                ],
                ..Default::default()
            },
        );
        if let Some(shaped) = Renderer::shape_rich_text(
            self,
            &[span],
            None,
            cvkg_runic_text::TextAlign::Start,
            cvkg_runic_text::TextOverflow::Visible,
        ) {
            shaped.ascent / self.text_scale_factor().max(1.0)
        } else {
            0.0
        }
    }

    /// Scale factor used by text measurement helpers.
    ///
    /// Renderers that shape text in device pixels should return their current
    /// device scale so `measure_text` and `measure_text_baseline` stay in logical pixels.
    fn text_scale_factor(&self) -> f32 {
        1.0
    }

    fn shape_rich_text(
        &mut self,
        _spans: &[cvkg_runic_text::TextSpan],
        _max_width: Option<f32>,
        _align: cvkg_runic_text::TextAlign,
        _overflow: cvkg_runic_text::TextOverflow,
    ) -> Option<cvkg_runic_text::ShapedText> {
        None
    }

    fn draw_shaped_text(&mut self, _text: &cvkg_runic_text::ShapedText, _x: f32, _y: f32) {}

    // ── Images & textures ────────────────────────────────────────────────
    /// Draw a texture (GPU-side) at the specified rect.
    fn draw_texture(&mut self, _texture_id: u32, _rect: Rect) {}
    /// Draw an image asset by name or path.
    fn draw_image(&mut self, _image_name: &str, _rect: Rect) {}
    /// Load an image asset from memory.
    fn load_image(&mut self, _name: &str, _data: &[u8]) {}
    /// Pre-warm the renderer with assets. Implementations can use this
    /// to populate texture atlases or warm up shader caches.
    fn prewarm_vram(&mut self, _assets: Vec<(String, Vec<u8>)>) {}

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

    /// Draw a 3D mesh with full material and transform support.
    fn draw_mesh_3d(&mut self, _mesh: &Mesh, _material: &Material3D, _transform: &Transform3D) {}

    /// Set the 3D camera for perspective/orthographic projection.
    /// If not called, rendering defaults to the 2D orthographic projection.
    fn set_camera_3d(&mut self, _camera: &Camera3D) {}

    /// Push a 3D transform onto the transform stack.
    /// All subsequent drawing is affected until `pop_transform_3d`.
    fn push_transform_3d(&mut self, _transform: &Transform3D) {}

    /// Pop the most recently pushed 3D transform.
    fn pop_transform_3d(&mut self) {}

    /// Render a 3D scene graph node. Reads position_3d, rotation_3d, scale_3d
    /// from the node and emits the appropriate draw call.
    /// Default implementation is a no-op; 3D renderers override this.
    ///
    /// `position`: [x, y, z] world-space position
    /// `rotation`: [x, y, z, w] quaternion rotation
    /// `scale`: [x, y, z] scale factors
    /// `color`: [r, g, b, a] base color for unlit rendering
    fn render_scene_node_3d(
        &mut self,
        _position: [f32; 3],
        _rotation: [f32; 4],
        _scale: [f32; 3],
        _color: [f32; 4],
        _meshes: &[Mesh],
    ) {
        // Default no-op: 2D renderers ignore 3D scene nodes
    }

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
    fn set_berserker_mode(&mut self, _state: RenderIntensityMode) {}
    fn trigger_shatter_event(&mut self, _origin: [f32; 2], _force: f32) {}
    /// Set the fireball position for dynamic glass specular highlights.
    fn set_fireball_pos(&mut self, _pos: [f32; 2]) {}
    /// Set the desktop scene preset (Aurora, Void, Nebula, Glitch, Yggdrasil).
    fn set_scene(&mut self, _scene: &str) {}
    /// Set the desktop scene by name. Case-insensitive.
    /// Supports: "aurora", "void", "nebula", "glitch", "yggdrasil".
    /// Aliases: "empty", "none", "blank" → Void.
    fn set_scene_by_name(&mut self, name: &str) {
        if let Some(preset) = resolve_scene_by_name(name) {
            Renderer::set_scene_preset(self, preset);
        }
    }

    // ── Export & Print ───────────────────────────────────────────────────
    /// Capture the current frame as a PNG byte buffer.
    fn capture_png(&mut self) -> Vec<u8> {
        Vec::new()
    }
    /// Trigger a native print dialog or spooling operation.
    fn print(&mut self) {}

    fn set_scene_preset(&mut self, _preset: u32) {}

    // ── Cyberpunk Effects ────────────────────────────────────────────────
    /// Apply a Bifrost (Frosted Glass) effect to the specified rect.
    fn bifrost(&mut self, _rect: Rect, _blur: f32, _saturation: f32, _opacity: f32) {}
    /// Apply a Gungnir (Neon Glow) effect to the specified rect.
    fn gungnir(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Soft glow variant -- half the intensity of gungnir(). Use for hover highlights.
    fn gungnir_soft(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32, _intensity: f32) {}
    /// Set the default background color for the canvas (RGBA).
    /// Used when the app does not draw its own background.
    fn set_default_background_color(&mut self, _color: [f32; 4]) {}
    /// Apply a ManiGlow (Lunar Illuminator) effect.
    fn mani_glow(&mut self, _rect: Rect, _color: [f32; 4], _radius: f32) {}
    /// Push a Mjolnir Slice (geometric clipping).
    fn push_mjolnir_slice(&mut self, _angle: f32, _offset: f32) {}
    fn pop_mjolnir_slice(&mut self) {}
    /// Execute a render function with memoization.
    /// If the renderer supports caching and the `id` + `data_hash` match a previous run,
    /// it may replay cached commands instead of executing the function.
    fn memoize(&mut self, id: u64, data_hash: u64, render_fn: &dyn Fn(&mut dyn Renderer));
    /// Capture current renderer stack depths for later panic recovery.
    /// The default implementation returns `RenderStateSnapshot::default()`,
    /// which is safe but does nothing useful -- backends with stack state
    /// must override this to record their actual depths.
    fn snapshot_render_state(&self) -> RenderStateSnapshot {
        RenderStateSnapshot::default()
    }
    /// Restore renderer stack state by popping items pushed beyond the
    /// snapshot point. Used by `ErrorBoundary` to recover from mid-render
    /// panics so sibling views don't inherit leaked clip/opacity/transform
    /// state. Idempotent: a no-op if stacks are already at or below the
    /// snapshot depths. Default implementation is a no-op for backends
    /// that have no stack state.
    fn restore_render_state(&mut self, _snap: RenderStateSnapshot) {}
    /// Apply a Mjolnir Shatter effect (fragmentation) to the specified rect.
    fn mjolnir_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    fn mjolnir_fluid_shatter(&mut self, _rect: Rect, _pieces: u32, _force: f32, _color: [f32; 4]) {}
    fn draw_mjolnir_bolt(&mut self, _from: [f32; 2], _to: [f32; 2], _color: [f32; 4]) {}

    // ── Futuristic UI Compute & Volumetric ───────────────────────────────
    /// Dispatches a burst of GPU particles (e.g. fireworks, data streams).
    fn dispatch_particles(
        &mut self,
        _origin: [f32; 2],
        _count: u32,
        _effect_type: &str,
        _color: [f32; 4],
    ) {
    }

    /// Draws a volumetric hologram into the specified bounding rectangle.
    fn draw_hologram(&mut self, _rect: Rect, _hologram_id: &str, _time: f32) {}

    // ── Accessibility (ShieldWall) ───────────────────────────────────────
    fn set_aria_role(&mut self, _role: &str) {}
    fn set_aria_label(&mut self, _label: &str) {}
    fn set_aria_valuemin(&mut self, _min: f32) {}
    fn set_aria_valuemax(&mut self, _max: f32) {}
    fn set_aria_valuenow(&mut self, _now: f32) {}

    /// Push a focus trap onto the stack. While active, keyboard focus is
    /// trapped within the specified element and its children.
    /// Returns a trap ID that must be passed to `pop_focus_trap`.
    fn push_focus_trap(&mut self, _element_id: &str) -> u64 { 0 }

    /// Pop the most recently pushed focus trap.
    fn pop_focus_trap(&mut self, _trap_id: u64) {}

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
    /// Draw a pre-loaded SVG model with a per-instance animation time offset.
    /// The offset shifts the animation phase, allowing multiple draws of the same
    /// SVG to animate independently. Default delegates to draw_svg (no offset).
    fn draw_svg_with_offset(&mut self, name: &str, rect: Rect, _animation_time_offset: f32) {
        Renderer::draw_svg(self, name, rect);
    }
    /// Draw a pre-loaded SVG model with explicit draw_order for z-sorting.
    /// draw_order=200 renders above UI chrome (draw_order=0).
    fn draw_svg_with_order(&mut self, name: &str, rect: Rect, _draw_order: i32) {
        Renderer::draw_svg(self, name, rect);
    }
    /// Serialize a pre-loaded SVG model back to SVG XML markup.
    /// Returns the serialized SVG string, or an error if the model is not loaded
    /// or serialization is not supported by this renderer.
    fn serialize_svg(&mut self, _name: &str) -> Result<String, String> {
        Err("SVG serialization not supported by this renderer".into())
    }
    /// Apply an SVG filter to a pre-loaded SVG model by filter element ID.
    /// The filter is evaluated and the result composited back into the SVG.
    /// Returns the filtered SVG as XML, or an error if not supported.
    fn apply_svg_filter(
        &mut self,
        _name: &str,
        _filter_id: &str,
        _region: Rect,
    ) -> Result<String, String> {
        Err("SVG filter not supported by this renderer".into())
    }

    // ── GPU Transformations ──────────────────────────────────────────────
    /// Push a 2D transform (translation, scale, rotation) onto the stack.
    /// This transform should be applied to all subsequent draw calls until popped.
    /// Transform-only animations use this to avoid re-triggering the layout engine.
    fn push_transform(&mut self, _translation: [f32; 2], _scale: [f32; 2], _rotation: f32) {}
    /// Push a raw 2D affine transform matrix [a, b, c, d, e, f] corresponding to
    /// [m11, m12, m21, m22, tx, ty].
    fn push_affine(&mut self, _transform: [f32; 6]) {}
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

    // ── Material Routing ─────────────────────────────────────────────────
    /// Set the active material for subsequent draw calls.
    /// Controls which pass a draw call is routed to in the multi-pass pipeline.
    fn set_material(&mut self, _material: crate::material::DrawMaterial) {}
    /// Return the currently active material (defaults to Opaque).
    fn current_material(&self) -> crate::material::DrawMaterial {
        crate::material::DrawMaterial::Opaque
    }

    // ── Vili Interaction Paradigm ──────────────────────────────────────────
    /// Compute the user's velocity/intent vector.
    fn mimir_intent(&self) -> [f32; 2] {
        [0.0, 0.0]
    }
    /// Calculate magnetic coordinate warp towards an anchor.
    fn magnetic_warp(&self, pointer: [f32; 2], anchor_rect: Rect, strength: f32) -> [f32; 2] {
        if strength <= 0.0 {
            return pointer;
        }
        let cx = anchor_rect.x + anchor_rect.width / 2.0;
        let cy = anchor_rect.y + anchor_rect.height / 2.0;
        let dx = pointer[0] - cx;
        let dy = pointer[1] - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let radius = 120.0;
        if dist < radius && dist > 0.0 {
            let force = (1.0 - dist / radius) * strength;
            [pointer[0] - dx * force, pointer[1] - dy * force]
        } else {
            pointer
        }
    }
    /// Calculate kinematic glow intensity based on proximity.
    fn mani_glow_intensity(&self, pointer: [f32; 2], bounds: Rect, radius: f32) -> f32 {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let dist = ((pointer[0] - cx).powi(2) + (pointer[1] - cy).powi(2)).sqrt();
        if dist < radius {
            (1.0 - dist / radius).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
    /// Calculate dynamic element attention (scaling/morphing) statelessly per frame.
    fn fafnir_evolve(&self, pointer: [f32; 2], bounds: Rect, max_scale: f32) -> f32 {
        let prox = self.mani_glow_intensity(pointer, bounds, 120.0);
        1.0 + (max_scale - 1.0) * prox
    }
    /// Sets the precise Vili SDF Shape boundary for hit-testing.
    fn set_sdf_shape(&mut self, _shape: crate::layout::SdfShape) {}

    // -- Portal / PhaseGate rendering -----------------------------------------

    /// Begin rendering into the portal root layer instead of the inline tree.
    /// All draw calls between `enter_portal` and `exit_portal` are collected
    /// into a separate buffer that is composited AFTER the main tree.
    ///
    /// WHY separate buffer: The main tree may have clipping, transforms, or
    /// opacity that should NOT affect overlays. The portal layer renders on top
    /// of everything, ignoring the local coordinate system.
    fn enter_portal(&mut self, _z_index: i32) {}

    /// Exit the portal layer and return to inline rendering.
    /// The portal content collected since `enter_portal` is now sealed --
    /// no more draw calls will be appended to it.
    fn exit_portal(&mut self) {}

    /// Get the current viewport size in logical pixels.
    /// Used by portal content to size itself to the full screen.
    fn viewport_size(&self) -> Rect {
        Rect::new(0.0, 0.0, 1920.0, 1080.0)
    }

    // -- Accessibility announcements -----------------------------------------

    /// Announce a message to screen readers via the platform accessibility API.
    /// This call is non-blocking. The message is queued and the screen reader
    /// will speak it at its own pace.
    fn announce(&mut self, _message: &str, _priority: AnnouncementPriority) {}
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RenderTier {
    /// High-performance GPU path (WebGPU / Vulkan / Metal / DX12) with full shader support.
    Tier1GPU = 0,
    /// Mid-tier GPU path (WebGL2 / OpenGL 3.3) with standard shader support.
    Tier2GPU = 1,
    /// Fallback software or basic hardware path (Canvas 2D / GDI+) with limited effects.
    Tier3Fallback = 2,
}

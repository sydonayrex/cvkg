use crate::*;
use std::sync::Arc;

#[derive(Clone)]
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

        if dot > 0.0
            && dist_sq < 250.0 * 250.0
            && speed_sq > 0.5
            && state.realm == UiFidelityLevel::Asgard
        {
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
            renderer.draw_text_raw(
                "HUGIN: THOUGHT",
                hugin_rect.x,
                hugin_rect.y,
                10.0,
                [0.0, 1.0, 1.0, 0.6],
            );
            for (i, thought) in state.thoughts.iter().rev().take(10).enumerate() {
                renderer.draw_text_raw(
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
            renderer.draw_text_raw(
                "MUNIN: MEMORY",
                munin_rect.x,
                munin_rect.y,
                10.0,
                [1.0, 0.84, 0.0, 0.6],
            );
            for (i, node) in state.nodes.iter().take(10).enumerate() {
                let opacity = (node.weight.min(1.0)) * 0.5;
                renderer.draw_text_raw(
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
                renderer.draw_text_raw(
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
        Some(self)
    }

    fn get_grid_placement(&self) -> Option<GridPlacement> {
        self.modifier
            .get_grid_placement()
            .or_else(|| self.view.get_grid_placement())
    }
}

impl<V: View, M: ViewModifier> layout::LayoutView for ModifiedView<V, M> {
    fn size_that_fits(
        &self,
        proposal: layout::SizeProposal,
        _subviews: &[&dyn layout::LayoutView],
        cache: &mut layout::LayoutCache,
    ) -> Size {
        let child_proposal = self.modifier.transform_proposal(proposal);
        let child_size = if let Some(layout) = self.view.layout() {
            layout.size_that_fits(child_proposal, &[], cache)
        } else {
            Size::ZERO
        };
        self.modifier.transform_size(child_size)
    }

    fn place_subviews(
        &self,
        bounds: Rect,
        _subviews: &mut [&mut dyn layout::LayoutView],
        cache: &mut layout::LayoutCache,
    ) {
        let child_rect = self.modifier.transform_rect(bounds);
        if let Some(layout) = self.view.layout() {
            layout.place_subviews(child_rect, &mut [], cache);
        }
    }

    fn flex_weight(&self) -> f32 {
        self.modifier.child_flex_weight(&self.view)
    }

    fn view_hash(&self) -> u64 {
        self.view.view_id().unwrap_or(0)
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

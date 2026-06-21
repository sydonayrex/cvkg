use cvkg_anim::{SpringParams, SpringSolver};
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
    load_system_state, update_system_state,
};

/// State storage for the layout morphing engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorphState {
    current_rect: Rect,
    solver_x: Option<SpringSolver>,
    solver_y: Option<SpringSolver>,
    solver_w: Option<SpringSolver>,
    solver_h: Option<SpringSolver>,
}

impl Default for MorphState {
    fn default() -> Self {
        Self {
            current_rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            solver_x: None,
            solver_y: None,
            solver_w: None,
            solver_h: None,
        }
    }
}

/// A view wrapper/modifier that smoothly animates its layout boundary changes.
///
/// # Contract
/// - Interpolates the x, y, width, and height positions of the view using spring physics.
/// - Triggers continuous frame redraws until boundaries have fully settled.
#[derive(Clone)]
pub struct Morphed<V: View> {
    pub content: V,
    pub morph_id: u64,
    pub spring_params: SpringParams,
}

impl<V: View> Morphed<V> {
    /// Create a new layout morphing view wrapper with a unique identifier.
    pub fn new(content: V, morph_id: u64) -> Self {
        Self {
            content,
            morph_id,
            spring_params: SpringParams::fluid(),
        }
    }

    /// Set custom spring parameters for the layout interpolation.
    pub fn spring_params(mut self, params: SpringParams) -> Self {
        self.spring_params = params;
        self
    }
}

impl<V: View> View for Morphed<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.morph_id == 0 {
            self.content.render(renderer, rect);
            return;
        }

        let dt = renderer.delta_time();

        // Load the previous state from system map
        let needs_init = {
            let state = load_system_state();
            state
                .get_component_state::<MorphState>(self.morph_id)
                .is_none()
        };

        if needs_init {
            let initial = MorphState {
                current_rect: rect,
                solver_x: Some(SpringSolver::new(self.spring_params, rect.x, rect.x)),
                solver_y: Some(SpringSolver::new(self.spring_params, rect.y, rect.y)),
                solver_w: Some(SpringSolver::new(
                    self.spring_params,
                    rect.width,
                    rect.width,
                )),
                solver_h: Some(SpringSolver::new(
                    self.spring_params,
                    rect.height,
                    rect.height,
                )),
            };
            update_system_state(|s| {
                let mut ns = s.clone();
                ns.set_component_state(self.morph_id, initial);
                ns
            });
        }

        let mut current_rect = rect;
        let mut is_moving = false;

        update_system_state(|s| {
            let mut ns = s.clone();
            if let Some(guard) = ns.get_component_state::<MorphState>(self.morph_id)
                && let Ok(r_guard) = guard.read()
            {
                let mut state = *r_guard;

                // Get or create solvers, updating targets if layout proposal changed
                let mut sx = state.solver_x.unwrap_or_else(|| {
                    SpringSolver::new(self.spring_params, rect.x, state.current_rect.x)
                });
                sx.set_target(rect.x);
                current_rect.x = sx.tick(dt);
                if !sx.is_settled() {
                    is_moving = true;
                }
                state.solver_x = Some(sx);

                let mut sy = state.solver_y.unwrap_or_else(|| {
                    SpringSolver::new(self.spring_params, rect.y, state.current_rect.y)
                });
                sy.set_target(rect.y);
                current_rect.y = sy.tick(dt);
                if !sy.is_settled() {
                    is_moving = true;
                }
                state.solver_y = Some(sy);

                let mut sw = state.solver_w.unwrap_or_else(|| {
                    SpringSolver::new(self.spring_params, rect.width, state.current_rect.width)
                });
                sw.set_target(rect.width);
                current_rect.width = sw.tick(dt);
                if !sw.is_settled() {
                    is_moving = true;
                }
                state.solver_w = Some(sw);

                let mut sh = state.solver_h.unwrap_or_else(|| {
                    SpringSolver::new(self.spring_params, rect.height, state.current_rect.height)
                });
                sh.set_target(rect.height);
                current_rect.height = sh.tick(dt);
                if !sh.is_settled() {
                    is_moving = true;
                }
                state.solver_h = Some(sh);

                state.current_rect = current_rect;
                ns.set_component_state(self.morph_id, state);
            }
            ns
        });

        // Request next frame render pass while spring is active
        if is_moving {
            renderer.request_redraw();
        }

        renderer.push_vnode(rect, "Morphed");
        self.content.render(renderer, current_rect);
        renderer.pop_vnode();
    }
}

impl<V: View> LayoutView for Morphed<V> {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        subviews: &[&dyn LayoutView],
        cache: &mut LayoutCache,
    ) -> Size {
        if let Some(layout_child) = subviews.first() {
            layout_child.size_that_fits(proposal, &subviews[1..], cache)
        } else {
            Size {
                width: 100.0,
                height: 100.0,
            }
        }
    }
    fn place_subviews(
        &self,
        bounds: Rect,
        subviews: &mut [&mut dyn LayoutView],
        cache: &mut LayoutCache,
    ) {
        if let Some((layout_child, rest)) = subviews.split_first_mut() {
            layout_child.place_subviews(bounds, rest, cache);
        }
    }
}

/// Extension trait to easily chain the morph modifier onto any view.
pub trait ViewMorphExt: View + Sized {
    /// Smoothly morphs layout changes for this view.
    fn morphed(self, morph_id: u64) -> Morphed<Self> {
        Morphed::new(self, morph_id)
    }
}

impl<V: View> ViewMorphExt for V {}

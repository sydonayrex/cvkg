use cvkg_anim::{SpringParams, SpringSolver};
use cvkg_core::{Never, Rect, Renderer, View, load_system_state, update_system_state};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

pub fn eval_easing(easing: Easing, t: f32) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => t * (2.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    Fade,
    Slide(SlideDirection),
    Scale,
    SlideFade(SlideDirection),
}

/// A unified animation component that supports both looping time-based transitions
/// and state-based spring physics interpolations.
#[derive(Clone)]
pub struct Animated<V: View> {
    pub content: V,

    // Time-based transition
    pub transition: Option<Transition>,
    pub easing: Easing,
    pub duration: f32,

    // Spring physics target
    pub state_hash: Option<u64>,
    pub spring_params: SpringParams,
    pub target_translate_x: Option<f32>,
    pub target_translate_y: Option<f32>,
    pub target_scale: Option<f32>,
}

impl<V: View> Animated<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            transition: None,
            easing: Easing::EaseInOut,
            duration: 0.3,
            state_hash: None,
            spring_params: SpringParams::snappy(),
            target_translate_x: None,
            target_translate_y: None,
            target_scale: None,
        }
    }

    pub fn transition(mut self, t: Transition) -> Self {
        self.transition = Some(t);
        self
    }

    pub fn easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }

    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d.max(0.01);
        self
    }

    pub fn spring(mut self, hash: u64, params: SpringParams) -> Self {
        self.state_hash = Some(hash);
        self.spring_params = params;
        self
    }

    pub fn translate_x(mut self, tx: f32) -> Self {
        self.target_translate_x = Some(tx);
        self
    }

    pub fn translate_y(mut self, ty: f32) -> Self {
        self.target_translate_y = Some(ty);
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.target_scale = Some(s.max(0.0));
        self
    }
}

impl<V: View> View for Animated<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut final_tx = 0.0;
        let mut final_ty = 0.0;
        let mut final_scale = 1.0;
        let mut final_opacity = 1.0;

        // Apply time-based looping transition
        if let Some(trans) = &self.transition {
            let elapsed = renderer.elapsed_time();
            let duration = self.duration.max(0.01);
            let raw = (elapsed % duration) / duration;
            let t = eval_easing(self.easing, raw);

            match trans {
                Transition::Fade => {
                    final_opacity *= t;
                }
                Transition::Slide(direction) => {
                    let (tx, ty) = match direction {
                        SlideDirection::Up => (0.0, -rect.height * (1.0 - t)),
                        SlideDirection::Down => (0.0, rect.height * (1.0 - t)),
                        SlideDirection::Left => (-rect.width * (1.0 - t), 0.0),
                        SlideDirection::Right => (rect.width * (1.0 - t), 0.0),
                    };
                    final_tx += tx;
                    final_ty += ty;
                }
                Transition::Scale => {
                    final_scale *= t;
                }
                Transition::SlideFade(direction) => {
                    let (tx, ty) = match direction {
                        SlideDirection::Up => (0.0, -rect.height * (1.0 - t)),
                        SlideDirection::Down => (0.0, rect.height * (1.0 - t)),
                        SlideDirection::Left => (-rect.width * (1.0 - t), 0.0),
                        SlideDirection::Right => (rect.width * (1.0 - t), 0.0),
                    };
                    final_tx += tx;
                    final_ty += ty;
                    final_opacity *= t;
                }
            }
        }

        // Apply state-based spring physics
        if let Some(hash) = self.state_hash {
            let dt = renderer.delta_time();

            let needs_init = {
                let state = load_system_state();
                state.get_component_state::<SpringSolver>(hash).is_none()
            };

            if needs_init {
                update_system_state(|s| {
                    let mut ns = s.clone();
                    if let Some(target) = self.target_translate_x {
                        ns.set_component_state(
                            hash,
                            SpringSolver::new(self.spring_params, target, 0.0),
                        );
                    }
                    if let Some(target) = self.target_translate_y {
                        ns.set_component_state(
                            hash + 1,
                            SpringSolver::new(self.spring_params, target, 0.0),
                        );
                    }
                    if let Some(target) = self.target_scale {
                        ns.set_component_state(
                            hash + 2,
                            SpringSolver::new(self.spring_params, target, 1.0),
                        );
                    }
                    ns
                });
            }

            let mut tx = 0.0f32;
            let mut ty = 0.0f32;
            let mut scale = 1.0f32;
            let mut is_moving = false;

            update_system_state(|s| {
                let mut ns = s.clone();

                for (i, target) in [
                    self.target_translate_x,
                    self.target_translate_y,
                    self.target_scale,
                ]
                .iter()
                .enumerate()
                {
                    if target.is_none() {
                        continue;
                    }
                    let h = hash + i as u64;
                    if let Some(solver) = ns.get_component_state::<SpringSolver>(h)
                        && let Ok(guard) = solver.read()
                    {
                        let mut solver = *guard;
                        let val = solver.tick(dt);
                        if !solver.is_settled() {
                            is_moving = true;
                        }
                        match i {
                            0 => tx = val,
                            1 => ty = val,
                            2 => scale = val,
                            _ => {}
                        }
                        ns.set_component_state(h, solver);
                    }
                }
                ns
            });

            if self.target_translate_x.is_some() {
                final_tx += tx;
            }
            if self.target_translate_y.is_some() {
                final_ty += ty;
            }
            if self.target_scale.is_some() {
                final_scale *= scale;
            }

            if is_moving {
                renderer.request_redraw();
            }
        }

        renderer.push_vnode(rect, "Animated");
        if final_opacity < 1.0 {
            renderer.push_opacity(final_opacity);
        }
        if final_tx != 0.0 || final_ty != 0.0 || final_scale != 1.0 {
            renderer.push_transform([final_tx, final_ty], [final_scale, final_scale], 0.0);
        }

        self.content.render(renderer, rect);

        if final_tx != 0.0 || final_ty != 0.0 || final_scale != 1.0 {
            renderer.pop_transform();
        }
        if final_opacity < 1.0 {
            renderer.pop_opacity();
        }
        renderer.pop_vnode();
    }
}

pub trait ViewAnimExt: View + Sized {
    fn animated(self) -> Animated<Self> {
        Animated::new(self)
    }
}
impl<V: View> ViewAnimExt for V {}

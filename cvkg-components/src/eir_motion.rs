//! Eir Motion - Animation and motion framework
//!
//! Eir the Aesir goddess governs healing and motion - this animation system
//! provides declarative animations, physics-based motion, and timeline controls.

use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Animation keyframes
#[derive(Debug, Clone)]
pub struct MotionKeyframe {
    pub time: f32,
    pub value: f32,
    pub easing: MotionEasing,
}

/// Easing functions for motion animations.
#[derive(Debug, Clone, Copy)]
pub enum MotionEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

/// Animation definition
#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub keyframes: Vec<MotionKeyframe>,
    pub loop_count: u32,
}

/// Physics-based motion parameters
#[derive(Debug, Clone)]
pub struct PhysicsMotion {
    pub velocity: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub bounce: f32,
}

/// Eir Motion Engine for animations
pub struct EirMotion {
    pub animations: Vec<Animation>,
    pub physics: Vec<PhysicsMotion>,
    pub state_machine: StateMachine,
}

/// State machine for animations
#[derive(Debug, Clone)]
pub struct StateMachine {
    pub current_state: String,
    pub transitions: Vec<StateMachineTransition>,
}

#[derive(Debug, Clone)]
pub struct StateMachineTransition {
    pub from: String,
    pub to: String,
    pub condition: String,
}

impl Default for EirMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl EirMotion {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            physics: Vec::new(),
            state_machine: StateMachine {
                current_state: "idle".to_string(),
                transitions: Vec::new(),
            },
        }
    }

    pub fn animation(mut self, name: &str, duration: f32) -> Self {
        self.animations.push(Animation {
            name: name.to_string(),
            duration,
            keyframes: Vec::new(),
            loop_count: 1,
        });
        self
    }

    pub fn keyframe(mut self, anim_name: &str, time: f32, value: f32, easing: MotionEasing) -> Self {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.name == anim_name) {
            anim.keyframes.push(MotionKeyframe {
                time,
                value,
                easing,
            });
        }
        self
    }

    pub fn physics(mut self, velocity: f32, acceleration: f32) -> Self {
        self.physics.push(PhysicsMotion {
            velocity,
            acceleration,
            friction: 0.95,
            bounce: 0.8,
        });
        self
    }

    pub fn state(mut self, state: &str) -> Self {
        self.state_machine.current_state = state.to_string();
        self
    }
}

impl View for EirMotion {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.08, 0.06, 0.12, 1.0]);
        renderer.draw_text(
            "Eir Motion Engine",
            rect.x + 10.0,
            rect.y + 20.0,
            14.0,
            [0.7, 0.8, 1.0, 1.0],
        );

        // State indicator
        renderer.draw_text(
            &format!("State: {}", self.state_machine.current_state),
            rect.x + 120.0,
            rect.y + 20.0,
            10.0,
            [0.5, 0.8, 1.0, 1.0],
        );

        // Animation list
        let mut y = rect.y + 45.0;
        for anim in &self.animations {
            renderer.fill_rect(
                Rect {
                    x: rect.x + 10.0,
                    y,
                    width: rect.width - 20.0,
                    height: 25.0,
                },
                [0.06, 0.06, 0.08, 1.0],
            );

            let _progress = format!(
                "{:.0}%",
                (anim.keyframes.len() as f32 / (anim.duration / 0.1)) * 100.0
            );
            renderer.draw_text(
                &anim.name,
                rect.x + 15.0,
                y + 8.0,
                11.0,
                [0.8, 0.9, 1.0, 1.0],
            );
            renderer.draw_text(
                &format!("{:.1}s", anim.duration),
                rect.x + rect.width - 60.0,
                y + 8.0,
                9.0,
                [0.6, 0.7, 0.9, 1.0],
            );
            y += 28.0;
        }

        // Physics visualization
        if !self.physics.is_empty() {
            let physics_y = rect.y + rect.height - 80.0;
            renderer.draw_text(
                "Physics Systems:",
                rect.x + 10.0,
                physics_y,
                10.0,
                [0.7, 0.8, 1.0, 1.0],
            );

            for (i, p) in self.physics.iter().enumerate() {
                let bar_width = (p.velocity.abs() * 20.0).min(rect.width - 40.0);
                renderer.fill_rect(
                    Rect {
                        x: rect.x + 15.0,
                        y: physics_y + 15.0 + i as f32 * 15.0,
                        width: bar_width,
                        height: 8.0,
                    },
                    [0.4, 0.6, 0.9, 1.0],
                );
            }
        }
    }
}

impl LayoutView for EirMotion {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 300.0,
            height: 100.0 + self.animations.len() as f32 * 28.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

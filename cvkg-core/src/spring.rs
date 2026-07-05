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

/// A 3D spring solver using individual 1D spring solvers for each axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSolver3D {
    pub params: SpringParams,
    pub x: SpringSolver,
    pub y: SpringSolver,
    pub z: SpringSolver,
}

impl SpringSolver3D {
    /// Create a new 3D spring solver with target and current positions.
    pub fn new(params: SpringParams, target: glam::Vec3, current: glam::Vec3) -> Self {
        Self {
            params,
            x: SpringSolver::new(params, target.x, current.x),
            y: SpringSolver::new(params, target.y, current.y),
            z: SpringSolver::new(params, target.z, current.z),
        }
    }

    /// Set the target position.
    pub fn set_target(&mut self, t: glam::Vec3) {
        self.x.set_target(t.x);
        self.y.set_target(t.y);
        self.z.set_target(t.z);
    }

    /// Advance the simulation by dt seconds and return the new position.
    pub fn tick(&mut self, dt: f32) -> glam::Vec3 {
        glam::Vec3::new(
            self.x.tick(dt),
            self.y.tick(dt),
            self.z.tick(dt),
        )
    }

    /// Check if all three spring solvers have settled.
    pub fn is_settled(&self) -> bool {
        self.x.is_settled() && self.y.is_settled() && self.z.is_settled()
    }
}

/// A quaternion rotation spring solver using angular velocity and log-map torque.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSolverQuat {
    pub params: SpringParams,
    pub current: glam::Quat,
    pub angular_vel: glam::Vec3,
}

impl SpringSolverQuat {
    /// Create a new quaternion spring solver.
    pub fn new(params: SpringParams, current: glam::Quat) -> Self {
        Self {
            params,
            current,
            angular_vel: glam::Vec3::ZERO,
        }
    }

    /// Advance the simulation by dt seconds toward target and return the new rotation.
    pub fn tick(&mut self, dt: f32, target: glam::Quat) -> glam::Quat {
        if dt <= 0.0 {
            return self.current;
        }

        let mut remaining = dt;
        let step = 1.0 / 120.0;

        while remaining > 0.0 {
            let d = remaining.min(step);
            self.step(d, target);
            remaining -= d;
        }

        self.current
    }

    fn step(&mut self, dt: f32, target: glam::Quat) {
        let q_error = target * self.current.inverse();
        let q_error = if q_error.w < 0.0 {
            glam::Quat::from_xyzw(-q_error.x, -q_error.y, -q_error.z, -q_error.w)
        } else {
            q_error
        };

        // Safe conversion of quaternion to axis-angle rotation vector
        let w = q_error.w.clamp(-1.0, 1.0);
        let angle = 2.0 * w.acos();
        let sin_half = (1.0 - w * w).sqrt();
        let axis = if sin_half > 1e-5 {
            glam::Vec3::new(q_error.x, q_error.y, q_error.z) / sin_half
        } else {
            glam::Vec3::X
        };
        let error_rot_vec = axis * angle;

        let torque = error_rot_vec * self.params.stiffness - self.angular_vel * self.params.damping;
        let mass = self.params.mass.max(0.001);
        let accel = torque / mass;

        self.angular_vel += accel * dt;
        let angle_step = self.angular_vel * dt;
        let rot_step = if angle_step.length_squared() > 1e-8 {
            glam::Quat::from_scaled_axis(angle_step)
        } else {
            glam::Quat::IDENTITY
        };
        self.current = (rot_step * self.current).normalize();
    }

    /// Check if the rotation spring has settled at the target.
    pub fn is_settled(&self, target: glam::Quat) -> bool {
        let q_error = target * self.current.inverse();
        let w = q_error.w.clamp(-1.0, 1.0);
        let angle = 2.0 * w.acos();
        angle.abs() < 0.001 && self.angular_vel.length() < 0.001
    }
}

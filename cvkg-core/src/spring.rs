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

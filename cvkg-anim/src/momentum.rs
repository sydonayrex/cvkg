pub struct DecaySolver {
    pub velocity: f32,
    pub friction: f32,
    pub position: f32,
}

impl DecaySolver {
    pub fn new(velocity: f32, friction: f32, initial_position: f32) -> Self {
        Self {
            velocity,
            friction,
            position: initial_position,
        }
    }

    /// Advance the simulation by dt seconds.
    pub fn tick(&mut self, dt: f32) -> f32 {
        self.position += self.velocity * dt;
        self.velocity *= self.friction.powf(dt * 60.0); // Assuming 60Hz baseline for friction param
        self.position
    }

    /// Calculate the resting position mathematically (infinite time projection)
    pub fn project_rest_position(&self) -> f32 {
        // v = v0 * f^t => integral is v0 / (1 - f) for discrete ticks
        let decay_rate = 1.0 - self.friction;
        if decay_rate <= 0.001 {
            return self.position;
        }
        self.position + (self.velocity / (decay_rate * 60.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_projection() {
        let solver = DecaySolver::new(1000.0, 0.95, 0.0);
        let projection = solver.project_rest_position();
        assert!(projection > 0.0);
    }
}

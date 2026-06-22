/// Budget for offscreen render targets.
/// Prevents OOM on mobile GPUs by enforcing a maximum number of concurrent
/// offscreen targets and a maximum total pixel count.
#[derive(Clone, Debug)]
pub struct OffscreenBudget {
    /// Maximum number of concurrent offscreen targets.
    pub max_targets: usize,
    /// Maximum total pixel count across all offscreen targets.
    pub max_total_pixels: u64,
    /// Current total pixel count.
    pub current_pixels: u64,
    /// Current number of allocated targets.
    pub current_targets: usize,
}

impl Default for OffscreenBudget {
    fn default() -> Self {
        Self {
            max_targets: 8,
            // 4x 1080p frames = ~8.3M pixels
            max_total_pixels: 1920u64 * 1080 * 4,
            current_pixels: 0,
            current_targets: 0,
        }
    }
}

impl OffscreenBudget {
    /// Create a budget with mobile-friendly defaults (lower limits).
    pub fn mobile() -> Self {
        Self {
            max_targets: 4,
            // 2x 720p frames = ~1.8M pixels
            max_total_pixels: 1280u64 * 720 * 2,
            current_pixels: 0,
            current_targets: 0,
        }
    }

    /// Check if a new target of the given size can be allocated.
    pub fn can_allocate(&self, width: u32, height: u32) -> bool {
        let pixels = width as u64 * height as u64;
        self.current_targets < self.max_targets
            && self.current_pixels + pixels <= self.max_total_pixels
    }

    /// Register a new offscreen target.
    pub fn register(&mut self, width: u32, height: u32) {
        self.current_pixels += width as u64 * height as u64;
        self.current_targets += 1;
    }

    /// Release an offscreen target.
    pub fn release(&mut self, width: u32, height: u32) {
        self.current_pixels = self.current_pixels.saturating_sub(width as u64 * height as u64);
        self.current_targets = self.current_targets.saturating_sub(1);
    }

    /// Reset the budget (e.g., on frame boundary).
    pub fn reset(&mut self) {
        self.current_pixels = 0;
        self.current_targets = 0;
    }

    /// Returns true if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.current_targets >= self.max_targets
    }
}

#[cfg(test)]
mod p1_27_offscreen_budget_tests {
    use super::OffscreenBudget;

    #[test]
    fn default_budget_allows_allocation() {
        let budget = OffscreenBudget::default();
        assert!(budget.can_allocate(1920, 1080));
    }

    #[test]
    fn mobile_budget_has_lower_limits() {
        let budget = OffscreenBudget::mobile();
        assert!(budget.can_allocate(1280, 720));
        assert!(!budget.can_allocate(3840, 2160)); // 4K exceeds mobile budget
    }

    #[test]
    fn budget_tracks_registration() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        assert_eq!(budget.current_targets, 1);
        assert_eq!(budget.current_pixels, 1920u64 * 1080);
    }

    #[test]
    fn budget_enforces_max_targets() {
        let mut budget = OffscreenBudget {
            max_targets: 2,
            max_total_pixels: u64::MAX,
            current_pixels: 0,
            current_targets: 0,
        };
        budget.register(100, 100);
        budget.register(100, 100);
        assert!(!budget.can_allocate(100, 100)); // 3rd target exceeds max
        assert!(budget.is_exhausted());
    }

    #[test]
    fn budget_enforces_pixel_limit() {
        let mut budget = OffscreenBudget {
            max_targets: 100,
            max_total_pixels: 1000,
            current_pixels: 0,
            current_targets: 0,
        };
        assert!(budget.can_allocate(10, 10)); // 100 pixels
        budget.register(10, 10);
        assert!(!budget.can_allocate(100, 10)); // 1000 pixels would exceed
    }

    #[test]
    fn release_frees_budget() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        budget.release(1920, 1080);
        assert_eq!(budget.current_targets, 0);
        assert_eq!(budget.current_pixels, 0);
    }

    #[test]
    fn reset_clears_all() {
        let mut budget = OffscreenBudget::default();
        budget.register(1920, 1080);
        budget.register(1280, 720);
        budget.reset();
        assert_eq!(budget.current_targets, 0);
        assert_eq!(budget.current_pixels, 0);
    }
}

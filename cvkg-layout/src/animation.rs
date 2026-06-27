use cvkg_core::{LayoutCache, LayoutView, Rect};
use std::collections::HashMap;

/// Manages active physics transitions for layout bounding boxes.
pub struct AnimationEngine {
    pub active_transitions: HashMap<u64, cvkg_anim::physics::ViscousSpring>,
    /// Generation counter for transition eviction.
    pub eviction_generation: u64,
    /// Tracks which generation each transition was last touched in.
    pub transition_generation: HashMap<u64, u64>,
    /// Number of generations a transition can go untouched before eviction.
    pub eviction_threshold: u64,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationEngine {
    /// Creates a new AnimationEngine.
    pub fn new() -> Self {
        Self {
            active_transitions: HashMap::new(),
            eviction_generation: 0,
            transition_generation: HashMap::new(),
            eviction_threshold: 300,
        }
    }

    /// Retrieves or initializes the AnimationEngine in the layout cache.
    pub fn get_or_insert_engine(cache: &mut LayoutCache) -> &mut Self {
        if cache.animators.is_none() {
            cache.animators = Some(Box::new(AnimationEngine::new()));
        }
        cache
            .animators
            .as_mut()
            .unwrap()
            .downcast_mut::<AnimationEngine>()
            .unwrap()
    }

    /// Evict settled transitions that haven't been touched for N generations.
    pub fn evict_stale_transitions(&mut self) {
        self.eviction_generation += 1;
        let threshold = self.eviction_threshold;
        let current_gen = self.eviction_generation;
        self.active_transitions.retain(|hash, spring| {
            let recent = self
                .transition_generation
                .get(hash)
                .is_some_and(|g| current_gen - *g < threshold);
            let unsettled =
                spring.velocity_a.length_sq() > 0.0001 || spring.velocity_b.length_sq() > 0.0001;
            recent || unsettled
        });
        self.transition_generation
            .retain(|hash, _| self.active_transitions.contains_key(hash));
    }
}

/// Applies view transitions to calculated layout rects.
pub fn apply_layout_animations(
    rects: Vec<Rect>,
    subviews: &mut [&mut dyn LayoutView],
    cache: &mut LayoutCache,
) {
    let mut transitions_to_update = Vec::new();

    for (child, target_rect) in subviews.iter().zip(&rects) {
        let hash = child.view_hash();
        if hash != 0 {
            if let Some(prev) = cache.previous_rects.get(&hash) {
                let dx = (prev.x - target_rect.x).abs();
                let dy = (prev.y - target_rect.y).abs();
                let dw = (prev.width - target_rect.width).abs();
                let dh = (prev.height - target_rect.height).abs();
                let epsilon = 1e-3;
                if dx > epsilon || dy > epsilon || dw > epsilon || dh > epsilon {
                    transitions_to_update.push((hash, *prev, *target_rect));
                }
            }
            cache.previous_rects.insert(hash, *target_rect);
            cache
                .previous_rects_generation
                .insert(hash, cache.eviction_generation);
        }
    }

    let mut interpolated_rects = HashMap::new();
    let delta = cache.delta_time;
    let scale = cache.scale_factor;
    let anim_engine = AnimationEngine::get_or_insert_engine(cache);

    for (hash, prev, target_rect) in transitions_to_update {
        let mut spring = if let Some(mut existing) = anim_engine.active_transitions.remove(&hash) {
            existing.position_b =
                cvkg_anim::physics::Vec3::new(target_rect.x, target_rect.y, target_rect.width);
            existing
        } else {
            cvkg_anim::physics::ViscousSpring::new(
                cvkg_anim::physics::Vec3::new(prev.x, prev.y, prev.width),
                cvkg_anim::physics::Vec3::new(target_rect.x, target_rect.y, target_rect.width),
                0.9,
                1000.0,
            )
        };
        spring.step(delta);

        let speed = (spring.velocity_a.length_sq() + spring.velocity_b.length_sq()).sqrt();
        let snap = |v: f32| (v * scale).round() / scale;

        let (rx, ry, rw) = if speed < 0.05 {
            (
                snap(spring.position_a.x),
                snap(spring.position_a.y),
                snap(spring.position_a.z),
            )
        } else {
            (
                spring.position_a.x,
                spring.position_a.y,
                spring.position_a.z,
            )
        };

        interpolated_rects.insert(
            hash,
            Rect {
                x: rx,
                y: ry,
                width: rw,
                height: target_rect.height,
            },
        );
        anim_engine.active_transitions.insert(hash, spring);
        anim_engine
            .transition_generation
            .insert(hash, anim_engine.eviction_generation);
    }

    cache.evict_stale_entries();

    let anim_engine = AnimationEngine::get_or_insert_engine(cache);
    anim_engine.evict_stale_transitions();

    for (child, mut target_rect) in subviews.iter_mut().zip(rects) {
        let hash = child.view_hash();
        if let Some(interp) = interpolated_rects.get(&hash) {
            target_rect = *interp;
        }
        let is_visible = if let Some(viewport) = cache.viewport {
            target_rect.intersects(&viewport)
        } else {
            true
        };
        if is_visible {
            crate::with_layout_cycle_guard_void(hash, || {
                child.place_subviews(target_rect, &mut [], cache);
            });
        }
    }
}

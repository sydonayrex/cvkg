//! Screen-space SDF shadow types and raymarching parameters.
//!
//! Provides the data structures needed to render screen-space signed distance
//! field shadows. Nodes cast shadows onto the layers below them by rendering
//! their shape into an SDF texture, then raymarching against it during the
//! compositing pass.

use crate::Rect;

/// Parameters for a single screen-space SDF shadow pass.
///
/// The shadow is computed by:
/// 1. Rendering occluder shapes into an SDF texture (distance to nearest edge).
/// 2. Raymarching from each pixel toward the light direction, checking SDF values.
/// 3. Accumulating shadow density based on occlusion distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SdfShadowParams {
    /// Direction the shadow is cast (unit vector pointing away from the light).
    pub light_dir: [f32; 2],
    /// Shadow color (RGB + alpha density).
    pub shadow_color: [f32; 4],
    /// Maximum raymarch distance in pixels.
    pub max_distance: f32,
    /// Number of raymarch steps (more = smoother, more expensive).
    pub step_count: u32,
    /// Softness of the shadow edge (higher = softer).
    pub softness: f32,
    /// Offset of the shadow from the occluder center.
    pub offset: [f32; 2],
    /// Opacity multiplier for the entire shadow.
    pub opacity: f32,
}

impl SdfShadowParams {
    /// Creates a default downward shadow (light from above).
    pub fn downward() -> Self {
        Self {
            light_dir: [0.0, 1.0],
            shadow_color: [0.0, 0.0, 0.0, 0.35],
            max_distance: 64.0,
            step_count: 32,
            softness: 4.0,
            offset: [0.0, 2.0],
            opacity: 1.0,
        }
    }

    /// Creates a default upward shadow (light from below).
    pub fn upward() -> Self {
        Self {
            light_dir: [0.0, -1.0],
            shadow_color: [0.0, 0.0, 0.0, 0.35],
            max_distance: 64.0,
            step_count: 32,
            softness: 4.0,
            offset: [0.0, -2.0],
            opacity: 1.0,
        }
    }

    /// Creates a directional shadow with a custom light angle (in radians).
    pub fn with_angle(angle: f32) -> Self {
        Self {
            light_dir: [angle.sin(), angle.cos()],
            ..Self::downward()
        }
    }

    /// Sets the shadow color.
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.shadow_color = [r, g, b, a];
        self
    }

    /// Sets the maximum raymarch distance.
    pub fn with_max_distance(mut self, d: f32) -> Self {
        self.max_distance = d;
        self
    }

    /// Sets the raymarch step count.
    pub fn with_steps(mut self, n: u32) -> Self {
        self.step_count = n;
        self
    }

    /// Sets the shadow softness.
    pub fn with_softness(mut self, s: f32) -> Self {
        self.softness = s;
        self
    }

    /// Sets the shadow offset from the occluder.
    pub fn with_offset(mut self, dx: f32, dy: f32) -> Self {
        self.offset = [dx, dy];
        self
    }

    /// Sets the shadow opacity.
    pub fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o.clamp(0.0, 1.0);
        self
    }
}

impl Default for SdfShadowParams {
    fn default() -> Self {
        Self::downward()
    }
}

/// An occluder shape that casts an SDF shadow.
///
/// Represents a single shape to be rendered into the SDF texture
/// before the raymarching pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdfShape {
    /// A rectangle occluder.
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// A rounded rectangle occluder.
    RoundedRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
    /// A circle occluder.
    Circle { cx: f32, cy: f32, radius: f32 },
}

impl SdfShape {
    /// Returns the bounding box of this shape.
    pub fn bounds(&self) -> Rect {
        match *self {
            SdfShape::Rect {
                x,
                y,
                width,
                height,
            } => Rect::new(x, y, width, height),
            SdfShape::RoundedRect {
                x,
                y,
                width,
                height,
                ..
            } => Rect::new(x, y, width, height),
            SdfShape::Circle { cx, cy, radius } => {
                Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0)
            }
        }
    }

    /// Creates a rounded rectangle from a `Rect` and corner radius.
    pub fn from_rect(rect: Rect, radius: f32) -> Self {
        SdfShape::RoundedRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            radius,
        }
    }
}

/// A single shadow instance to be rendered.
///
/// Combines an SDF shape with shadow parameters for one shadow-casting object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowInstance {
    /// The occluder shape.
    pub shape: SdfShape,
    /// Shadow rendering parameters.
    pub params: SdfShadowParams,
}

impl ShadowInstance {
    /// Creates a new shadow instance from a shape and parameters.
    pub fn new(shape: SdfShape, params: SdfShadowParams) -> Self {
        Self { shape, params }
    }

    /// Creates a shadow instance for a rectangle with default downward shadow.
    pub fn rect_shadow(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            shape: SdfShape::Rect {
                x,
                y,
                width,
                height,
            },
            params: SdfShadowParams::downward(),
        }
    }

    /// Creates a shadow instance for a rounded rectangle with default downward shadow.
    pub fn rounded_rect_shadow(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        Self {
            shape: SdfShape::RoundedRect {
                x,
                y,
                width,
                height,
                radius,
            },
            params: SdfShadowParams::downward(),
        }
    }

    /// Creates a shadow instance for a circle with default downward shadow.
    pub fn circle_shadow(cx: f32, cy: f32, radius: f32) -> Self {
        Self {
            shape: SdfShape::Circle { cx, cy, radius },
            params: SdfShadowParams::downward(),
        }
    }

    /// Returns the bounding box of the shadow (occluder bounds + max distance + offset).
    pub fn shadow_bounds(&self) -> Rect {
        let bounds = self.shape.bounds();
        let d = self.params.max_distance;
        let ox = self.params.offset[0];
        let oy = self.params.offset[1];
        Rect::new(
            bounds.x + ox - d,
            bounds.y + oy - d,
            bounds.width + d * 2.0,
            bounds.height + d * 2.0,
        )
    }
}

/// A batch of shadow instances for efficient GPU submission.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowBatch {
    pub instances: Vec<ShadowInstance>,
}

impl ShadowBatch {
    /// Creates an empty shadow batch.
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    /// Creates a batch pre-allocated for `capacity` instances.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instances: Vec::with_capacity(capacity),
        }
    }

    /// Adds a shadow instance to the batch.
    pub fn push(&mut self, instance: ShadowInstance) {
        self.instances.push(instance);
    }

    /// Returns the number of shadow instances.
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Returns true if the batch has no instances.
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Clears all instances while retaining capacity.
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Computes the combined bounding box of all shadow instances.
    pub fn total_bounds(&self) -> Option<Rect> {
        if self.instances.is_empty() {
            return None;
        }
        let first = self.instances[0].shadow_bounds();
        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x + first.width;
        let mut max_y = first.y + first.height;

        for inst in &self.instances[1..] {
            let b = inst.shadow_bounds();
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x + b.width);
            max_y = max_y.max(b.y + b.height);
        }

        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

impl Default for ShadowBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdf_shadow_params_default() {
        let p = SdfShadowParams::default();
        assert_eq!(p.light_dir, [0.0, 1.0]);
        assert_eq!(p.step_count, 32);
    }

    #[test]
    fn sdf_shadow_params_with_angle() {
        let p = SdfShadowParams::with_angle(std::f32::consts::PI / 4.0);
        assert!((p.light_dir[0] - (std::f32::consts::PI / 4.0_f32).sin()).abs() < 0.001);
        assert!((p.light_dir[1] - (std::f32::consts::PI / 4.0_f32).cos()).abs() < 0.001);
    }

    #[test]
    fn sdf_shape_bounds() {
        let rect = SdfShape::Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        let b = rect.bounds();
        assert_eq!(b.x, 10.0);
        assert_eq!(b.y, 20.0);
        assert_eq!(b.width, 100.0);
        assert_eq!(b.height, 50.0);
    }

    #[test]
    fn sdf_shape_circle_bounds() {
        let circle = SdfShape::Circle {
            cx: 50.0,
            cy: 50.0,
            radius: 20.0,
        };
        let b = circle.bounds();
        assert_eq!(b.x, 30.0);
        assert_eq!(b.y, 30.0);
        assert_eq!(b.width, 40.0);
        assert_eq!(b.height, 40.0);
    }

    #[test]
    fn shadow_instance_bounds() {
        let inst = ShadowInstance::rect_shadow(0.0, 0.0, 100.0, 50.0);
        let b = inst.shadow_bounds();
        // Should extend by max_distance (64) in all directions
        assert!(b.x < 0.0);
        assert!(b.y < 0.0);
        assert!(b.width > 100.0);
        assert!(b.height > 50.0);
    }

    #[test]
    fn shadow_batch_total_bounds() {
        let mut batch = ShadowBatch::new();
        assert!(batch.total_bounds().is_none());

        batch.push(ShadowInstance::rect_shadow(0.0, 0.0, 100.0, 50.0));
        batch.push(ShadowInstance::rect_shadow(200.0, 200.0, 50.0, 50.0));

        let bounds = batch.total_bounds().unwrap();
        assert!(bounds.x <= 0.0);
        assert!(bounds.y <= 0.0);
        assert!(bounds.width >= 250.0);
        assert!(bounds.height >= 250.0);
    }

    #[test]
    fn shadow_batch_clear() {
        let mut batch = ShadowBatch::new();
        batch.push(ShadowInstance::rect_shadow(0.0, 0.0, 10.0, 10.0));
        assert_eq!(batch.len(), 1);
        batch.clear();
        assert!(batch.is_empty());
    }
}

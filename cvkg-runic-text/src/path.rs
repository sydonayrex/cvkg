/// A Bezier spline path for positioning and rotating glyphs along arbitrary curves.
///
/// # Contract
/// The path is constructed from control points. The `sample` method interpolates
/// along the path at normalized parameter `t` (0.0 to 1.0) and returns the 2D position
/// and the tangent rotation angle in radians for orienting characters correctly.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPath {
    /// Control points for the Bezier spline segments.
    pub control_points: Vec<(f32, f32)>,
}

impl TextPath {
    /// Create a new text path from control points.
    pub fn new(control_points: Vec<(f32, f32)>) -> Self {
        TextPath { control_points }
    }

    /// Sample the position and tangent rotation angle (radians) at normalized parameter `t` (0.0..=1.0).
    pub fn sample(&self, t: f32) -> ((f32, f32), f32) {
        if self.control_points.is_empty() {
            return ((0.0, 0.0), 0.0);
        }
        let n = self.control_points.len();
        if n == 1 {
            return (self.control_points[0], 0.0);
        }
        if n == 3 {
            // Quadratic Bezier interpolation
            let p0 = self.control_points[0];
            let p1 = self.control_points[1];
            let p2 = self.control_points[2];
            let u = 1.0 - t;
            let tt = t * t;
            let uu = u * u;
            let x = uu * p0.0 + 2.0 * u * t * p1.0 + tt * p2.0;
            let y = uu * p0.1 + 2.0 * u * t * p1.1 + tt * p2.1;
            let tx = 2.0 * u * (p1.0 - p0.0) + 2.0 * t * (p2.0 - p1.0);
            let ty = 2.0 * u * (p1.1 - p0.1) + 2.0 * t * (p2.1 - p1.1);
            let angle = ty.atan2(tx);
            ((x, y), angle)
        } else if n == 4 {
            // Cubic Bezier interpolation
            let p0 = self.control_points[0];
            let p1 = self.control_points[1];
            let p2 = self.control_points[2];
            let p3 = self.control_points[3];
            let u = 1.0 - t;
            let tt = t * t;
            let uu = u * u;
            let uuu = uu * u;
            let ttt = tt * t;
            let x = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
            let y = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;
            let tx =
                3.0 * uu * (p1.0 - p0.0) + 6.0 * u * t * (p2.0 - p1.0) + 3.0 * tt * (p3.0 - p2.0);
            let ty =
                3.0 * uu * (p1.1 - p0.1) + 6.0 * u * t * (p2.1 - p1.1) + 3.0 * tt * (p3.1 - p2.1);
            let angle = ty.atan2(tx);
            ((x, y), angle)
        } else {
            // Fallback: Linear polyline interpolation
            let segments = n - 1;
            let scaled_t = t * segments as f32;
            let idx = (scaled_t.floor() as usize).min(segments - 1);
            let local_t = scaled_t - idx as f32;
            let p0 = self.control_points[idx];
            let p1 = self.control_points[idx + 1];
            let x = p0.0 + (p1.0 - p0.0) * local_t;
            let y = p0.1 + (p1.1 - p0.1) * local_t;
            let tx = p1.0 - p0.0;
            let ty = p1.1 - p0.1;
            let angle = ty.atan2(tx);
            ((x, y), angle)
        }
    }
}

/// Boundary shapes used for non-rectangular text wrapping.
///
/// # Contract
/// Represents geometric limits within which text flows are allowed or clipped.
/// The layouter checks collision with boundaries during the line reflow calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutBoundary {
    /// Circular boundary: center x, center y, radius.
    Circle {
        /// Center X coordinate.
        cx: f32,
        /// Center Y coordinate.
        cy: f32,
        /// Radius of boundary circle.
        r: f32,
    },
    /// Convex polygon boundary defined by a set of clockwise vertices.
    Polygon {
        /// Vertices (x, y) defining the polygon boundary.
        vertices: Vec<(f32, f32)>,
    },
}

impl LayoutBoundary {
    /// Compute the allowed horizontal span `[x_min, x_max]` at a vertical coordinate `y`.
    ///
    /// # Contract
    /// Checks intersection of a horizontal line at `y` with the boundary shape.
    /// Returns `Some((x_min, x_max))` if the line intersects the boundary, otherwise `None`.
    pub fn allowed_span(&self, y: f32) -> Option<(f32, f32)> {
        match self {
            LayoutBoundary::Circle { cx, cy, r } => {
                let dy = y - cy;
                if dy.abs() < *r {
                    let dx = (r * r - dy * dy).sqrt();
                    Some((cx - dx, cx + dx))
                } else {
                    None
                }
            }
            LayoutBoundary::Polygon { vertices } => {
                if vertices.len() < 3 {
                    return None;
                }
                let mut intersections = Vec::new();
                for i in 0..vertices.len() {
                    let p0 = vertices[i];
                    let p1 = vertices[(i + 1) % vertices.len()];
                    let y_min = p0.1.min(p1.1);
                    let y_max = p0.1.max(p1.1);
                    if y >= y_min && y <= y_max && (p1.1 - p0.1).abs() > 1e-5 {
                        let t = (y - p0.1) / (p1.1 - p0.1);
                        let x = p0.0 + t * (p1.0 - p0.0);
                        intersections.push(x);
                    }
                }
                if intersections.len() >= 2 {
                    intersections.sort_by(|a, b| a.total_cmp(b));
                    Some((intersections[0], intersections[intersections.len() - 1]))
                } else {
                    None
                }
            }
        }
    }
}

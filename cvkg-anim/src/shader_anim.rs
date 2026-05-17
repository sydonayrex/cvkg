use glam::{Vec2, Vec3, Vec4};

// ============================================================================
// Reaction-Diffusion (Gray-Scott Model)
// ============================================================================

/// Gray-Scott reaction-diffusion simulation using ping-pong double-buffered grids.
///
/// Each cell stores two chemical concentrations (U, V) in a `Vec4` where:
///   x = U concentration, y = V concentration, z = laplacian(U), w = laplacian(V)
///
/// The simulation uses a 3x3 Laplacian convolution kernel:
///   [0.05, 0.2, 0.05]
///   [0.2, -1.0, 0.2]
///   [0.05, 0.2, 0.05]
///
/// Feed rate `f` and kill rate `k` control the pattern formation.
pub struct ReactionDiffusion {
    width: u32,
    height: u32,
    feed: f32,
    kill: f32,
    /// Double-buffered grids: current and next state.
    grid_a: Vec<Vec4>,
    grid_b: Vec<Vec4>,
    /// Diffusion rates for U and V chemicals.
    du: f32,
    dv: f32,
    /// Whether grid_a is currently the source buffer.
    ping_pong_a: bool,
}

impl ReactionDiffusion {
    /// Creates a new Gray-Scott reaction-diffusion grid.
    ///
    /// * `width` - Grid width in cells.
    /// * `height` - Grid height in cells.
    /// * `feed` - Feed rate (f) controlling how chemical U is replenished.
    /// * `kill` - Kill rate (k) controlling how chemical V is removed.
    ///
    /// Typical values: feed=0.055, kill=0.062 for mitosis-like patterns.
    pub fn new(width: u32, height: u32, feed: f32, kill: f32) -> Self {
        let size = (width * height) as usize;
        let mut grid_a = vec![Vec4::ZERO; size];
        let grid_b = vec![Vec4::ZERO; size];

        // Seed with a central blob of V chemical
        let cx = (width / 2) as i32;
        let cy = (height / 2) as i32;
        let radius = std::cmp::min(width, height) as i32 / 8;

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy < radius * radius {
                    let idx = (y as u32 * width + x as u32) as usize;
                    // U starts at 1.0, V starts at 1.0 in the seed region
                    grid_a[idx] = Vec4::new(1.0, 1.0, 0.0, 0.0);
                } else {
                    let idx = (y as u32 * width + x as u32) as usize;
                    // U=1, V=0 everywhere else
                    grid_a[idx] = Vec4::new(1.0, 0.0, 0.0, 0.0);
                }
            }
        }

        Self {
            width,
            height,
            feed,
            kill,
            grid_a,
            grid_b,
            du: 1.0,
            dv: 0.5,
            ping_pong_a: true,
        }
    }

    /// Advances the simulation by `dt` seconds using the Gray-Scott equations:
    ///
    ///   dU/dt = Du * ∇²U - U*V² + f*(1 - U)
    ///   dV/dt = Dv * ∇²V + U*V² - (f + k)*V
    pub fn update(&mut self, dt: f32) {
        let w = self.width as i32;
        let h = self.height as i32;
        let (src, dst) = if self.ping_pong_a {
            (&self.grid_a, &mut self.grid_b)
        } else {
            (&self.grid_b, &mut self.grid_a)
        };

        let f = self.feed;
        let k = self.kill;
        let du = self.du;
        let dv = self.dv;

        for y in 0..h {
            for x in 0..w {
                let idx = (y as u32 * self.width + x as u32) as usize;
                let cell = src[idx];
                let u = cell.x;
                let v = cell.y;

                // 3x3 Laplacian convolution with wrapping boundaries (inlined to avoid double borrow)
                let lap_u =
                    ReactionDiffusion::laplacian_static(src, self.width, self.height, x, y, 0);
                let lap_v =
                    ReactionDiffusion::laplacian_static(src, self.width, self.height, x, y, 1);

                let uvv = u * v * v;

                let new_u = u + dt * (du * lap_u - uvv + f * (1.0 - u));
                let new_v = v + dt * (dv * lap_v + uvv - (f + k) * v);

                dst[idx] = Vec4::new(new_u.clamp(0.0, 1.0), new_v.clamp(0.0, 1.0), 0.0, 0.0);
            }
        }

        self.ping_pong_a = !self.ping_pong_a;
    }

    /// Samples a grid cell with toroidal wrapping.
    fn sample_static(
        grid: &[Vec4],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        component: usize,
    ) -> f32 {
        let w = width as i32;
        let h = height as i32;
        let wx = x.rem_euclid(w) as u32;
        let wy = y.rem_euclid(h) as u32;
        let idx = (wy * width + wx) as usize;
        if component == 0 {
            grid[idx].x
        } else {
            grid[idx].y
        }
    }

    /// Computes the Laplacian for a single cell using a 3x3 stencil (static version).
    /// `component` 0 = U (x), 1 = V (y).
    fn laplacian_static(
        grid: &[Vec4],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        component: usize,
    ) -> f32 {
        let center_val = Self::sample_static(grid, width, height, x, y, component);
        let adj_sum = Self::sample_static(grid, width, height, x - 1, y, component)
            + Self::sample_static(grid, width, height, x + 1, y, component)
            + Self::sample_static(grid, width, height, x, y - 1, component)
            + Self::sample_static(grid, width, height, x, y + 1, component);
        let diag_sum = Self::sample_static(grid, width, height, x - 1, y - 1, component)
            + Self::sample_static(grid, width, height, x + 1, y - 1, component)
            + Self::sample_static(grid, width, height, x - 1, y + 1, component)
            + Self::sample_static(grid, width, height, x + 1, y + 1, component);
        0.2 * adj_sum + 0.05 * diag_sum - center_val
    }

    /// Returns a reference to the current front buffer.
    pub fn current_grid(&self) -> &[Vec4] {
        if self.ping_pong_a {
            &self.grid_a
        } else {
            &self.grid_b
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Sets the feed rate.
    pub fn set_feed(&mut self, feed: f32) {
        self.feed = feed;
    }

    /// Sets the kill rate.
    pub fn set_kill(&mut self, kill: f32) {
        self.kill = kill;
    }

    /// Sets diffusion rates for U and V chemicals.
    pub fn set_diffusion(&mut self, du: f32, dv: f32) {
        self.du = du;
        self.dv = dv;
    }
}

// ============================================================================
// Vertex Animation Texture (VAT)
// ============================================================================

/// Vertex Animation Texture system that bakes per-frame vertex positions and
/// normals into an RGBA32F texture atlas for GPU playback.
///
/// Texture layout (each row is one frame, each column is one vertex):
///   RGBA channel 0 (R,G,B,A) = vertex position (x, y, z, 1.0)
///   RGBA channel 1 (R,G,B,A) = vertex normal   (x, y, z, 0.0)
///
/// The texture atlas has `frame_count` rows and `vertices_per_frame` columns.
/// Each pixel stores one Vec4; positions and normals are stored in alternating
/// column pairs.
pub struct VertexAnimationTexture {
    frame_count: u32,
    vertices_per_frame: u32,
    /// Flat texture data: width = vertices_per_frame, height = frame_count * 2
    /// (position rows + normal rows).
    texture_data: Vec<Vec4>,
    /// Texture dimensions
    tex_width: u32,
    tex_height: u32,
    /// Animation duration in seconds
    duration: f32,
}

/// Result of a VAT playback query.
pub struct VatFrame {
    /// UV coordinates for the position row of this frame.
    pub position_uv: Vec2,
    /// UV coordinates for the normal row of this frame.
    pub normal_uv: Vec2,
    /// The integer frame index.
    pub frame_index: u32,
    /// Interpolation factor between this frame and the next.
    pub blend: f32,
}

impl VertexAnimationTexture {
    /// Creates a new Vertex Animation Texture.
    ///
    /// * `frame_count` - Number of animation frames.
    /// * `vertices_per_frame` - Number of vertices per frame.
    /// * `positions` - Flat slice of positions: frame_count * vertices_per_frame Vec3s.
    /// * `normals` - Flat slice of normals: frame_count * vertices_per_frame Vec3s.
    /// * `duration` - Total animation duration in seconds.
    pub fn new(
        frame_count: u32,
        vertices_per_frame: u32,
        positions: &[Vec3],
        normals: &[Vec3],
        duration: f32,
    ) -> Self {
        let expected = (frame_count * vertices_per_frame) as usize;
        assert_eq!(
            positions.len(),
            expected,
            "positions length must equal frame_count * vertices_per_frame"
        );
        assert_eq!(
            normals.len(),
            expected,
            "normals length must equal frame_count * vertices_per_frame"
        );

        let tex_width = vertices_per_frame;
        let tex_height = frame_count * 2; // position rows + normal rows
        let mut texture_data = vec![Vec4::ZERO; (tex_width * tex_height) as usize];

        // Fill position rows (rows 0..frame_count)
        for frame in 0..frame_count {
            for v in 0..vertices_per_frame {
                let src_idx = (frame * vertices_per_frame + v) as usize;
                let dst_idx = (frame * tex_width + v) as usize;
                let p = positions[src_idx];
                texture_data[dst_idx] = Vec4::new(p.x, p.y, p.z, 1.0);
            }
        }

        // Fill normal rows (rows frame_count..frame_count*2)
        for frame in 0..frame_count {
            for v in 0..vertices_per_frame {
                let src_idx = (frame * vertices_per_frame + v) as usize;
                let dst_idx = ((frame_count + frame) * tex_width + v) as usize;
                let n = normals[src_idx];
                texture_data[dst_idx] = Vec4::new(n.x, n.y, n.z, 0.0);
            }
        }

        Self {
            frame_count,
            vertices_per_frame,
            texture_data,
            tex_width,
            tex_height,
            duration,
        }
    }

    /// Samples the animation at the given time and returns UV coordinates
    /// and frame info for GPU texture lookups.
    ///
    /// The returned `VatFrame` contains:
    ///   - `position_uv`: UV for the position row of the current frame
    ///   - `normal_uv`: UV for the normal row of the current frame
    ///   - `frame_index`: the current integer frame
    ///   - `blend`: interpolation factor [0,1) toward the next frame
    pub fn playback(&self, time: f32) -> VatFrame {
        let t = if self.duration > 0.0 {
            time.rem_euclid(self.duration) / self.duration
        } else {
            0.0
        };

        let frame_f = t * self.frame_count as f32;
        let frame_index = (frame_f as u32).min(self.frame_count - 1);
        let blend = frame_f - frame_index as f32;

        // UV y-coordinate: center of the texel row
        // Position rows are at y = 0..frame_count-1
        // Normal rows are at y = frame_count..frame_count*2-1
        let pos_y = (frame_index as f32 + 0.5) / self.tex_height as f32;
        let norm_y = (frame_index as f32 + 0.5 + self.frame_count as f32) / self.tex_height as f32;

        VatFrame {
            position_uv: Vec2::new(0.5 / self.tex_width as f32, pos_y),
            normal_uv: Vec2::new(0.5 / self.tex_width as f32, norm_y),
            frame_index,
            blend,
        }
    }

    /// Returns the raw texture data as a flat slice of Vec4.
    pub fn texture_data(&self) -> &[Vec4] {
        &self.texture_data
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn vertices_per_frame(&self) -> u32 {
        self.vertices_per_frame
    }

    pub fn texture_width(&self) -> u32 {
        self.tex_width
    }

    pub fn texture_height(&self) -> u32 {
        self.tex_height
    }

    /// Returns the position of a specific vertex in a specific frame.
    pub fn position(&self, frame: u32, vertex: u32) -> Vec3 {
        let idx = (frame * self.tex_width + vertex) as usize;
        let v = self.texture_data[idx];
        Vec3::new(v.x, v.y, v.z)
    }

    /// Returns the normal of a specific vertex in a specific frame.
    pub fn normal(&self, frame: u32, vertex: u32) -> Vec3 {
        let idx = ((self.frame_count + frame) * self.tex_width + vertex) as usize;
        let v = self.texture_data[idx];
        Vec3::new(v.x, v.y, v.z)
    }
}

// ============================================================================
// SDF Animator
// ============================================================================

/// Signed Distance Function animator supporting primitive shapes and
/// boolean operations with smooth blending.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SdfPrimitive {
    Sphere {
        radius: f32,
    },
    Box {
        half_extents: Vec3,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
}

/// An SDF node that can be a primitive or a composite operation.
#[derive(Debug, Clone)]
pub enum SdfNode {
    Primitive(SdfPrimitive),
    SmoothUnion {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    SmoothIntersection {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    SmoothSubtraction {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        k: f32,
    },
    Morph {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
        t: f32,
    },
}

/// SDF Animator that evaluates signed distance fields with support for
/// primitive shapes, smooth boolean operations, and morphing.
pub struct SdfAnimator {
    root: Option<SdfNode>,
}

impl SdfAnimator {
    /// Creates a new SDF Animator with no root node.
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Creates a new SDF Animator with the given root node.
    pub fn with_root(root: SdfNode) -> Self {
        Self { root: Some(root) }
    }

    /// Sets the root SDF node.
    pub fn set_root(&mut self, root: SdfNode) {
        self.root = Some(root);
    }

    /// Evaluates the SDF at the given point, returning the signed distance.
    /// Negative values indicate the point is inside the shape.
    pub fn evaluate(&self, point: Vec3) -> f32 {
        match &self.root {
            Some(node) => Self::eval_node(node, point),
            None => f32::MAX,
        }
    }

    fn eval_node(node: &SdfNode, p: Vec3) -> f32 {
        match node {
            SdfNode::Primitive(primitive) => match primitive {
                SdfPrimitive::Sphere { radius } => p.length() - *radius,
                SdfPrimitive::Box { half_extents } => {
                    let q = Vec3::new(
                        p.x.abs() - half_extents.x,
                        p.y.abs() - half_extents.y,
                        p.z.abs() - half_extents.z,
                    );
                    let outside = Vec3::new(q.x.max(0.0), q.y.max(0.0), q.z.max(0.0));
                    outside.length() + q.x.max(q.y.max(q.z)).min(0.0)
                }
                SdfPrimitive::Torus {
                    major_radius,
                    minor_radius,
                } => {
                    let q = Vec2::new(Vec2::new(p.x, p.z).length() - *major_radius, p.y);
                    q.length() - *minor_radius
                }
            },
            SdfNode::SmoothUnion { a, b, k } => {
                let d1 = Self::eval_node(a, p);
                let d2 = Self::eval_node(b, p);
                Self::smooth_min(d1, d2, *k)
            }
            SdfNode::SmoothIntersection { a, b, k } => {
                let d1 = Self::eval_node(a, p);
                let d2 = Self::eval_node(b, p);
                Self::smooth_max(d1, d2, *k)
            }
            SdfNode::SmoothSubtraction { a, b, k } => {
                let d1 = Self::eval_node(a, p);
                let d2 = Self::eval_node(b, p);
                Self::smooth_max(d1, -d2, *k)
            }
            SdfNode::Morph { a, b, t } => {
                let d1 = Self::eval_node(a, p);
                let d2 = Self::eval_node(b, p);
                d1 * (1.0 - t) + d2 * t
            }
        }
    }

    /// Smooth minimum (polynomial) between two distances.
    fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
        if k <= 0.0 {
            return a.min(b);
        }
        let h = (k - (a - b).abs()).max(0.0) / k;
        a.min(b) - h * h * k * 0.25
    }

    /// Smooth maximum (polynomial) between two distances.
    fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
        if k <= 0.0 {
            return a.max(b);
        }
        let h = (k - (a - b).abs()).max(0.0) / k;
        a.max(b) + h * h * k * 0.25
    }

    /// Creates a morph (linear interpolation) between two SDF nodes.
    ///
    /// * `a` - The source SDF node.
    /// * `b` - The target SDF node.
    /// * `t` - Interpolation factor [0.0, 1.0].
    pub fn morph(a: SdfNode, b: SdfNode, t: f32) -> SdfNode {
        SdfNode::Morph {
            a: Box::new(a),
            b: Box::new(b),
            t: t.clamp(0.0, 1.0),
        }
    }

    /// Creates a smooth union of two SDF nodes.
    pub fn smooth_union(a: SdfNode, b: SdfNode, k: f32) -> SdfNode {
        SdfNode::SmoothUnion {
            a: Box::new(a),
            b: Box::new(b),
            k,
        }
    }

    /// Creates a smooth intersection of two SdfNodes.
    pub fn smooth_intersection(a: SdfNode, b: SdfNode, k: f32) -> SdfNode {
        SdfNode::SmoothIntersection {
            a: Box::new(a),
            b: Box::new(b),
            k,
        }
    }

    /// Creates a smooth subtraction (a minus b) of two SdfNodes.
    pub fn smooth_subtraction(a: SdfNode, b: SdfNode, k: f32) -> SdfNode {
        SdfNode::SmoothSubtraction {
            a: Box::new(a),
            b: Box::new(b),
            k,
        }
    }

    /// Creates a sphere primitive SDF node.
    pub fn sphere(radius: f32) -> SdfNode {
        SdfNode::Primitive(SdfPrimitive::Sphere { radius })
    }

    /// Creates a box primitive SDF node.
    pub fn box_sdf(half_extents: Vec3) -> SdfNode {
        SdfNode::Primitive(SdfPrimitive::Box { half_extents })
    }

    /// Creates a torus primitive SDF node.
    pub fn torus(major_radius: f32, minor_radius: f32) -> SdfNode {
        SdfNode::Primitive(SdfPrimitive::Torus {
            major_radius,
            minor_radius,
        })
    }
}

impl Default for SdfAnimator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_diffusion_creation() {
        let rd = ReactionDiffusion::new(64, 64, 0.055, 0.062);
        assert_eq!(rd.width(), 64);
        assert_eq!(rd.height(), 64);
        assert_eq!(rd.current_grid().len(), 64 * 64);
    }

    #[test]
    fn test_reaction_diffusion_update() {
        let mut rd = ReactionDiffusion::new(32, 32, 0.055, 0.062);
        rd.update(1.0);
        // After update, grid should still be valid
        let grid = rd.current_grid();
        for cell in grid.iter() {
            assert!(cell.x >= 0.0 && cell.x <= 1.0);
            assert!(cell.y >= 0.0 && cell.y <= 1.0);
        }
    }

    #[test]
    fn test_vat_creation() {
        let frame_count = 4;
        let verts = 8;
        let positions = vec![Vec3::ZERO; (frame_count * verts) as usize];
        let normals = vec![Vec3::Y; (frame_count * verts) as usize];

        let vat = VertexAnimationTexture::new(frame_count, verts, &positions, &normals, 2.0);
        assert_eq!(vat.frame_count(), frame_count);
        assert_eq!(vat.vertices_per_frame(), verts);
        assert_eq!(vat.texture_width(), verts);
        assert_eq!(vat.texture_height(), frame_count * 2);
    }

    #[test]
    fn test_vat_playback() {
        let frame_count = 4;
        let verts = 8;
        let positions = vec![Vec3::ZERO; (frame_count * verts) as usize];
        let normals = vec![Vec3::Y; (frame_count * verts) as usize];

        let vat = VertexAnimationTexture::new(frame_count, verts, &positions, &normals, 2.0);
        let frame = vat.playback(0.5);
        assert!(frame.frame_index < frame_count);
        assert!(frame.blend >= 0.0 && frame.blend < 1.0);
    }

    #[test]
    fn test_sdf_sphere() {
        let animator = SdfAnimator::with_root(SdfAnimator::sphere(1.0));
        // Point on surface
        assert!((animator.evaluate(Vec3::X) - 0.0).abs() < 1e-5);
        // Point inside
        assert!(animator.evaluate(Vec3::ZERO) < 0.0);
        // Point outside
        assert!(animator.evaluate(Vec3::new(3.0, 0.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_sdf_box() {
        let animator = SdfAnimator::with_root(SdfAnimator::box_sdf(Vec3::ONE));
        // Point on surface
        assert!((animator.evaluate(Vec3::X) - 0.0).abs() < 1e-5);
        // Point inside
        assert!(animator.evaluate(Vec3::ZERO) < 0.0);
        // Point outside
        assert!(animator.evaluate(Vec3::new(3.0, 0.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_sdf_torus() {
        let animator = SdfAnimator::with_root(SdfAnimator::torus(1.0, 0.3));
        // Point at center should be inside the hole
        let d = animator.evaluate(Vec3::ZERO);
        // The center of the torus hole: distance to the torus tube center circle is 1.0,
        // minus tube radius 0.3 = 0.7 (positive = outside the solid)
        assert!(d > 0.0);
    }

    #[test]
    fn test_sdf_morph() {
        let a = SdfAnimator::sphere(1.0);
        let b = SdfAnimator::sphere(2.0);
        let morphed = SdfAnimator::morph(a, b, 0.5);
        let animator = SdfAnimator::with_root(morphed);
        // At t=0.5, the distance at (1.5,0,0) should be ~0.0 (midway between r=1 and r=2)
        let d = animator.evaluate(Vec3::new(1.5, 0.0, 0.0));
        assert!(d.abs() < 1e-4);
    }

    #[test]
    fn test_sdf_smooth_union() {
        let union =
            SdfAnimator::smooth_union(SdfAnimator::sphere(1.0), SdfAnimator::sphere(1.0), 0.5);
        let animator = SdfAnimator::with_root(union);
        // At origin, both spheres give -1.0, union should be around -1.0
        let d = animator.evaluate(Vec3::ZERO);
        assert!(d < -0.5);
    }

    #[test]
    fn test_sdf_empty_animator() {
        let animator = SdfAnimator::new();
        assert_eq!(animator.evaluate(Vec3::ZERO), f32::MAX);
    }
}

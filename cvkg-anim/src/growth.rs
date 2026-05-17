//! Growth and destruction procedural animation systems.
//!
//! Provides L-system based organic growth animation and Voronoi-based
//! fracturing/destruction animation for the Sleipnir animation engine.

use glam::Vec2;
use std::collections::HashMap;

// ============================================================================
// L-System — Organic Growth via Turtle Graphics
// ============================================================================

/// A single stochastic rule: maps a predecessor to a successor with a probability.
#[derive(Debug, Clone)]
pub struct LRule {
    /// The replacement string.
    pub successor: String,
    /// Probability weight for this rule (0.0..1.0). When multiple rules share
    /// the same predecessor key, weights are normalised and one is chosen at
    /// random on each expansion.
    pub probability: f32,
}

impl LRule {
    /// Create a deterministic rule (probability = 1.0, single rule per symbol).
    pub fn new(successor: impl Into<String>) -> Self {
        Self {
            successor: successor.into(),
            probability: 1.0,
        }
    }

    /// Create a stochastic rule with an explicit probability weight.
    pub fn with_probability(successor: impl Into<String>, probability: f32) -> Self {
        Self {
            successor: successor.into(),
            probability: probability.max(0.0),
        }
    }
}

/// A 2D line segment produced by the turtle interpreter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub start: Vec2,
    pub end: Vec2,
}

/// State saved/restored by the `[` / `]` turtle stack operations.
#[derive(Debug, Clone, Copy)]
struct TurtleState {
    position: Vec2,
    angle: f32,
}

/// Turtle graphics L-system with axiom, production rules, and configurable
/// angle/step.  Supports stochastic rules with probability weights, bracket
/// branching, and arbitrary symbol-to-instruction mappings.
///
/// # Example: Koch curve
/// ```
/// use cvkg_anim::growth::LSystem;
/// let mut lsys = LSystem::new("F", 90.0, 1.0);
/// lsys.add_rule('F', "F+F-F-F+F");
/// let lines = lsys.generate_with_depth(3);
/// // `lines` now contains the Koch curve segments.
/// ```
#[derive(Debug, Clone)]
pub struct LSystem {
    /// The initial axiom string.
    pub axiom: String,
    /// Production rules keyed by predecessor character.  Each entry holds one
    /// or more `LRule`; when multiple they are treated as stochastic
    /// alternatives.
    pub rules: HashMap<char, Vec<LRule>>,
    /// Turn angle in degrees for `+` (left) and `-` (right).
    pub angle: f32,
    /// Forward step length for each `F` draw command.
    pub step: f32,
    /// Number of iteration passes applied by `generate`.
    pub depth: usize,
    /// Symbols that trigger a forward draw (default: `['F']`).
    pub draw_symbols: Vec<char>,
    /// Symbols that trigger a forward move without drawing (default: `['f']`).
    pub move_symbols: Vec<char>,
    /// Symbol for left turn (default: `'+'`).
    pub turn_left: char,
    /// Symbol for right turn (default: `'-'`).
    pub turn_right: char,
    /// Push stack symbol (default: `'['`).
    pub push_symbol: char,
    /// Pop stack symbol (default: `']'`).
    pub pop_symbol: char,
    /// Optional RNG seed for deterministic stochastic expansion.
    pub seed: Option<u64>,
}

impl LSystem {
    /// Create a new L-system with the given axiom, turn angle (degrees), and
    /// forward step length.  Depth defaults to 0.
    pub fn new(axiom: impl Into<String>, angle: f32, step: f32) -> Self {
        Self {
            axiom: axiom.into(),
            rules: HashMap::new(),
            angle,
            step,
            depth: 0,
            draw_symbols: vec!['F'],
            move_symbols: vec!['f'],
            turn_left: '+',
            turn_right: '-',
            push_symbol: '[',
            pop_symbol: ']',
            seed: None,
        }
    }

    /// Set a deterministic seed for stochastic rule selection.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Add a single (deterministic) rule for `predecessor`.
    ///
    /// If multiple rules are added for the same predecessor they become
    /// stochastic alternatives chosen by normalised probability.
    pub fn add_rule(&mut self, predecessor: char, successor: impl Into<String>) -> &mut Self {
        let entry = self.rules.entry(predecessor).or_default();
        entry.push(LRule::new(successor));
        self
    }

    /// Add a stochastic rule with an explicit probability weight.
    pub fn add_stochastic_rule(
        &mut self,
        predecessor: char,
        successor: impl Into<String>,
        probability: f32,
    ) -> &mut Self {
        let entry = self.rules.entry(predecessor).or_default();
        entry.push(LRule::with_probability(successor, probability));
        self
    }

    /// Set the iteration depth (number of expansion passes).
    pub fn set_depth(&mut self, depth: usize) -> &mut Self {
        self.depth = depth;
        self
    }

    /// Expand the axiom by applying production rules for `iterations`
    /// rounds.  Returns the resulting string (the internal state is **not**
    /// mutated).
    pub fn iterate(&self, iterations: usize) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..iterations {
            current = self.expand_once(&current);
        }
        current
    }

    /// Convenience: expand using `self.depth` rounds and return the result.
    pub fn expanded(&self) -> String {
        self.iterate(self.depth)
    }

    /// Expand using a specific depth, overriding `self.depth`.
    pub fn expanded_with_depth(&self, depth: u32) -> String {
        self.iterate(depth as usize)
    }

    /// Generate line segments by expanding the axiom `self.depth` times and
    /// interpreting the result with the turtle graphics engine.
    pub fn generate(&self) -> Vec<LineSegment> {
        let lsystem_string = self.expanded();
        self.interpret(&lsystem_string)
    }

    /// Generate line segments with a specific expansion depth, overriding `self.depth`.
    pub fn generate_with_depth(&self, depth: u32) -> Vec<LineSegment> {
        let lsystem_string = self.expanded_with_depth(depth);
        self.interpret(&lsystem_string)
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Single expansion pass over `input`.
    fn expand_once(&self, input: &str) -> String {
        // Pre-compute total capacity to avoid repeated reallocations.
        let mut output = String::with_capacity(input.len() * 2);

        for ch in input.chars() {
            if let Some(rules) = self.rules.get(&ch) {
                if rules.len() == 1 {
                    // Fast path: deterministic single rule.
                    output.push_str(&rules[0].successor);
                } else {
                    // Stochastic: pick by normalised probability.
                    let chosen = self.pick_stochastic(rules);
                    output.push_str(&chosen.successor);
                }
            } else {
                // No rule → identity (the character passes through unchanged).
                output.push(ch);
            }
        }

        output
    }

    /// Pick a rule from a slice using weighted random selection.
    fn pick_stochastic<'a>(&'a self, rules: &'a [LRule]) -> &'a LRule {
        let total_weight: f32 = rules.iter().map(|r| r.probability).sum();
        if total_weight <= 0.0 {
            return &rules[0];
        }

        // Use seed-derived RNG or fallback to a simple deterministic hash.
        let mut rng = self.make_rng();
        let threshold = rng.next_f32() * total_weight;

        let mut cumulative = 0.0;
        for rule in rules {
            cumulative += rule.probability;
            if cumulative >= threshold {
                return rule;
            }
        }

        // Fallback (floating point edge case).
        rules.last().unwrap()
    }

    /// Build a simple deterministic PRNG from `self.seed` or a derivation
    /// thereof.
    fn make_rng(&self) -> SimpleRng {
        match self.seed {
            Some(s) => SimpleRng::new(s),
            None => {
                // Fallback: hash the axiom bytes as a pseudo-seed so identical
                // axioms give identical results even without an explicit seed.
                let hash = self.axiom.bytes().fold(0u64, |acc, b| {
                    acc.wrapping_mul(6364136223846793005).wrapping_add(b as u64)
                });
                SimpleRng::new(hash)
            }
        }
    }

    /// Turtle-graphics interpreter over `program`.
    fn interpret(&self, program: &str) -> Vec<LineSegment> {
        let mut segments = Vec::new();
        let mut state = TurtleState {
            position: Vec2::ZERO,
            angle: 0.0_f32.to_radians(), // start facing +X
        };
        let mut stack: Vec<TurtleState> = Vec::new();
        let turn = self.angle.to_radians();

        for ch in program.chars() {
            if self.draw_symbols.contains(&ch) {
                let delta = Vec2::new(state.angle.cos(), state.angle.sin()) * self.step;
                let end = state.position + delta;
                segments.push(LineSegment {
                    start: state.position,
                    end,
                });
                state.position = end;
            } else if self.move_symbols.contains(&ch) {
                let delta = Vec2::new(state.angle.cos(), state.angle.sin()) * self.step;
                state.position += delta;
            } else if ch == self.turn_left {
                state.angle += turn;
            } else if ch == self.turn_right {
                state.angle -= turn;
            } else if ch == self.push_symbol {
                stack.push(state);
            } else if ch == self.pop_symbol {
                if let Some(restored) = stack.pop() {
                    state = restored;
                }
            }
            // All other symbols are no-ops (used as constants to control branching).
        }

        segments
    }
}

// ============================================================================
// Simple deterministic PRNG (xorshift64*)
// ============================================================================

/// Minimal deterministic PRNG used internally by the L-system.  Not exposed
/// publicly — lives here to avoid a `rand` dependency for the L-system alone.
#[derive(Debug, Clone)]
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        // Ensure state is never zero (xorshift requirement).
        let state = if seed == 0 { 0xDEADBEEF } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Returns a value in `[0.0, 1.0)`.
    fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32;
        v / (1u64 << 24) as f32
    }
}

// ============================================================================
// Voronoi Fracture — Destruction via Voronoi Partitioning
// ============================================================================

/// An axis-aligned bounding box used as the sampling domain for Voronoi seeds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    /// Create a bounding box from min and max corners.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create a bounding box from center and half-extents.
    pub fn from_center_half(center: Vec2, half: Vec2) -> Self {
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Width of the bounding box.
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height of the bounding box.
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Center of the bounding box.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Check if a point is contained in the bounds (inclusive).
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

/// A single Voronoi cell: polygon vertices in CCW order plus metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct VoronoiCell {
    /// Vertices of the cell polygon (CCW winding).
    pub vertices: Vec<Vec2>,
    /// The seed point that generated this cell.
    pub site: Vec2,
    /// Centroid of the cell polygon.
    pub centroid: Vec2,
}

/// A fractured fragment: polygon with physics properties for animation.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    /// Vertices of the fragment polygon (CCW winding).
    pub vertices: Vec<Vec2>,
    /// Centroid of the fragment.
    pub centroid: Vec2,
    /// Outward velocity for animation (points away from fracture center).
    pub velocity: Vec2,
    /// Area of the fragment polygon.
    pub area: f32,
}

/// Voronoi-based fracturing system for procedural destruction animation.
///
/// Generates Voronoi cells from random seed points and clips an input polygon
/// against those cells to produce independently animatable fragments.
///
/// Uses an approximation of Fortune's algorithm based on discrete sampling for
/// robustness and simplicity, producing visually plausible Voronoi
/// decompositions suitable for real-time animation.
///
/// # Pipeline
/// 1. `generate(seed_count, bounds)` — place random seeds and compute cells.
/// 2. `clip_polygon(polygon)` — clip an arbitrary polygon against each cell.
/// 3. `fracture(polygon)` — runs the full pipeline and returns `Fragment`s.
///
/// # Example
/// ```
/// use cvkg_anim::growth::{VoronoiFracture, Bounds};
/// use glam::Vec2;
/// let mut vf = VoronoiFracture::new();
/// let bounds = Bounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
/// let polygon = vec![
///     Vec2::new(10.0, 10.0),
///     Vec2::new(90.0, 10.0),
///     Vec2::new(90.0, 90.0),
///     Vec2::new(10.0, 90.0),
/// ];
/// let fragments = vf.fracture(bounds, &polygon, 12);
/// // `fragments` contains individually animatable pieces.
/// ```
#[derive(Debug, Clone)]
pub struct VoronoiFracture {
    /// Generated Voronoi cells after `generate` is called.
    pub cells: Vec<VoronoiCell>,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Internal RNG state.
    rng: SimpleRng,
    /// Grid resolution for Voronoi approximation (higher = more accurate).
    pub resolution: usize,
}

impl VoronoiFracture {
    /// Create a new Voronoi fracture system with default settings.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            seed: 42,
            rng: SimpleRng::new(42),
            resolution: 128,
        }
    }

    /// Set the seed for deterministic fracture patterns.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self.rng = SimpleRng::new(seed);
        self
    }

    /// Set the grid resolution for Voronoi computation.  Higher values give
    /// more accurate boundaries at the cost of performance.  Default: 128.
    pub fn with_resolution(mut self, res: usize) -> Self {
        self.resolution = res.max(8);
        self
    }

    /// Generate `seed_count` random seed points within `bounds` and compute the
    /// Voronoi diagram.
    ///
    /// Each seed becomes the site of one Voronoi cell.  The cell boundaries
    /// are approximated via a discrete closest-site grid which is then refined
    /// into polygonal cells.
    pub fn generate(&mut self, seed_count: usize, bounds: Bounds) {
        self.cells.clear();

        // 1. Generate random seed sites within bounds.
        let sites = self.generate_sites(seed_count, bounds);

        // 2. Compute Voronoi diagram via discrete grid approximation.
        let grid = self.compute_voronoi_grid(&sites, bounds);

        // 3. Extract polygonal cells from the grid.
        self.cells = self.extract_cells(&sites, &grid, bounds);
    }

    /// Clip a convex or concave polygon against the generated Voronoi cells.
    ///
    /// Returns a vector of `(polygon_vertices, centroid)` tuples for every
    /// intersection of the input polygon with a Voronoi cell.  If fewer than
    /// one cell exists the input polygon is returned unchanged.
    ///
    /// The input polygon should be in CCW winding order.
    pub fn clip_polygon(&self, polygon: &[Vec2]) -> Vec<(Vec<Vec2>, Vec2)> {
        if self.cells.is_empty() {
            let centroid = polygon_centroid(polygon);
            return vec![(polygon.to_vec(), centroid)];
        }

        let mut results = Vec::with_capacity(self.cells.len());

        for cell in &self.cells {
            if let Some(clipped) = Self::sutherland_hodgman(polygon, &cell.vertices) {
                if clipped.len() >= 3 {
                    let centroid = polygon_centroid(&clipped);
                    results.push((clipped, centroid));
                }
            }
        }

        results
    }

    /// Full fracture pipeline: generate cells and clip the polygon.
    ///
    /// - `bounds`: domain for seed placement.
    /// - `polygon`: the shape to fracture (CCW winding).
    /// - `seed_count`: number of Voronoi seeds / fragments.
    ///
    /// Returns `Vec<Fragment>` ready for physics-based animation.  Each
    /// fragment gets a velocity pointing from the fracture center (bounds
    /// center) through the fragment centroid.
    pub fn fracture(
        &mut self,
        bounds: Bounds,
        polygon: &[Vec2],
        seed_count: usize,
    ) -> Vec<Fragment> {
        self.generate(seed_count, bounds);
        let fracture_center = bounds.center();

        self.clip_polygon(polygon)
            .into_iter()
            .filter_map(|(verts, centroid)| {
                if verts.len() < 3 {
                    return None;
                }
                let area = polygon_area(&verts);
                let direction = (centroid - fracture_center).normalize_or_zero();
                let speed = direction * (area.sqrt() * 0.5); // larger pieces fly faster
                Some(Fragment {
                    vertices: verts,
                    centroid,
                    velocity: speed,
                    area,
                })
            })
            .collect()
    }

    // ------------------------------------------------------------------
    // Internal: site generation
    // ------------------------------------------------------------------

    fn generate_sites(&mut self, count: usize, bounds: Bounds) -> Vec<Vec2> {
        let mut sites = Vec::with_capacity(count);
        for _ in 0..count {
            let x = bounds.min.x + self.rng.next_f32() * bounds.width();
            let y = bounds.min.y + self.rng.next_f32() * bounds.height();
            sites.push(Vec2::new(x, y));
        }
        sites
    }

    // ------------------------------------------------------------------
    // Internal: Voronoi grid computation
    // ------------------------------------------------------------------

    /// Compute a discrete closest-site grid.  Returns a 2D grid of indices
    /// (into `sites`) with the grid stored as a flat `Vec<usize>`.
    fn compute_voronoi_grid(&self, sites: &[Vec2], bounds: Bounds) -> Vec<usize> {
        let res = self.resolution;
        let mut grid = vec![0usize; res * res];

        for gy in 0..res {
            for gx in 0..res {
                let point = Vec2::new(
                    bounds.min.x + (gx as f32 + 0.5) / res as f32 * bounds.width(),
                    bounds.min.y + (gy as f32 + 0.5) / res as f32 * bounds.height(),
                );

                let mut best_idx = 0;
                let mut best_dist = f32::MAX;
                for (i, site) in sites.iter().enumerate() {
                    let d = point.distance_squared(*site);
                    if d < best_dist {
                        best_dist = d;
                        best_idx = i;
                    }
                }
                grid[gy * res + gx] = best_idx;
            }
        }

        grid
    }

    // ------------------------------------------------------------------
    // Internal: cell polygon extraction from grid
    // ------------------------------------------------------------------

    /// For each site, gather the grid region belonging to it and extract an
    /// outer polygon via a marching-squares-like boundary walk, scaled back to
    /// world-space.
    fn extract_cells(&self, sites: &[Vec2], grid: &[usize], bounds: Bounds) -> Vec<VoronoiCell> {
        let res = self.resolution;
        let cell_width = bounds.width() / res as f32;
        let cell_height = bounds.height() / res as f32;

        sites
            .iter()
            .enumerate()
            .filter_map(|(site_idx, &site)| {
                // Collect grid cells belonging to this site.
                let mut boundary_points: Vec<Vec2> = Vec::new();

                for gy in 0..res {
                    for gx in 0..res {
                        if grid[gy * res + gx] != site_idx {
                            continue;
                        }

                        // Check if this grid cell is on a boundary (adjacent to a different cell).
                        let is_boundary = self.is_grid_boundary(grid, gx, gy, site_idx, res);

                        // Convert grid cell corners to world coords.
                        let x0 = bounds.min.x + gx as f32 * cell_width;
                        let y0 = bounds.min.y + gy as f32 * cell_height;
                        let x1 = x0 + cell_width;
                        let y1 = y0 + cell_height;

                        if is_boundary {
                            // Emit all four corners; we'll compute convex hull.
                            boundary_points.push(Vec2::new(x0, y0));
                            boundary_points.push(Vec2::new(x1, y0));
                            boundary_points.push(Vec2::new(x1, y1));
                            boundary_points.push(Vec2::new(x0, y1));
                        } else {
                            // Interior cell — emit centre for robustness.
                            boundary_points.push(Vec2::new((x0 + x1) * 0.5, (y0 + y1) * 0.5));
                        }
                    }
                }

                if boundary_points.is_empty() {
                    return None;
                }

                // Compute convex hull of boundary points to get cell polygon.
                let hull = convex_hull_graham(&boundary_points);
                if hull.len() < 3 {
                    return None;
                }

                let centroid = polygon_centroid(&hull);

                Some(VoronoiCell {
                    vertices: hull,
                    site,
                    centroid,
                })
            })
            .collect()
    }

    /// Check whether the grid cell at (gx, gy) borders a different Voronoi site.
    fn is_grid_boundary(
        &self,
        grid: &[usize],
        gx: usize,
        gy: usize,
        site_idx: usize,
        res: usize,
    ) -> bool {
        let neighbours: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in &neighbours {
            let nx = gx as isize + dx;
            let ny = gy as isize + dy;
            if nx < 0 || nx >= res as isize || ny < 0 || ny >= res as isize {
                return true; // Edge of domain counts as boundary.
            }
            if grid[ny as usize * res + nx as usize] != site_idx {
                return true;
            }
        }
        false
    }

    // ------------------------------------------------------------------
    // Internal: Sutherland-Hodgman polygon clipping
    // ------------------------------------------------------------------

    /// Clip `subject` polygon against `clip` polygon (both CCW).  Returns the
    /// intersection polygon or `None` if there is no overlap.
    fn sutherland_hodgman(subject: &[Vec2], clip: &[Vec2]) -> Option<Vec<Vec2>> {
        if subject.len() < 3 || clip.len() < 3 {
            return None;
        }

        let mut output = subject.to_vec();
        let clip_len = clip.len();

        for i in 0..clip_len {
            if output.is_empty() {
                return None;
            }

            let edge_start = clip[i];
            let edge_end = clip[(i + 1) % clip_len];
            let input = output;
            output = Vec::with_capacity(input.len());

            for j in 0..input.len() {
                let current = input[j];
                let next = input[(j + 1) % input.len()];

                let current_inside = is_inside_edge(current, edge_start, edge_end);
                let next_inside = is_inside_edge(next, edge_start, edge_end);

                if current_inside {
                    output.push(current);
                    if !next_inside {
                        if let Some(inter) = line_intersection(current, next, edge_start, edge_end)
                        {
                            output.push(inter);
                        }
                    }
                } else if next_inside {
                    if let Some(inter) = line_intersection(current, next, edge_start, edge_end) {
                        output.push(inter);
                    }
                }
            }
        }

        if output.len() < 3 { None } else { Some(output) }
    }
}

impl Default for VoronoiFracture {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Geometric utility functions
// ===========================================================================///

/// Winding-signed area of a polygon (CCW = positive).
fn polygon_area(polygon: &[Vec2]) -> f32 {
    let n = polygon.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i].x * polygon[j].y;
        area -= polygon[j].x * polygon[i].y;
    }
    area.abs() * 0.5
}

/// Centroid (geometric center) of a polygon.
fn polygon_centroid(polygon: &[Vec2]) -> Vec2 {
    let n = polygon.len();
    if n == 0 {
        return Vec2::ZERO;
    }
    if n == 1 {
        return polygon[0];
    }
    if n == 2 {
        return (polygon[0] + polygon[1]) * 0.5;
    }

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut signed_area = 0.0;

    for i in 0..n {
        let j = (i + 1) % n;
        let cross = polygon[i].x * polygon[j].y - polygon[j].x * polygon[i].y;
        signed_area += cross;
        cx += (polygon[i].x + polygon[j].x) * cross;
        cy += (polygon[i].y + polygon[j].y) * cross;
    }

    signed_area *= 0.5;
    if signed_area.abs() < 1e-10 {
        // Degenerate: fall back to average.
        let sum: Vec2 = polygon.iter().copied().sum();
        return sum / n as f32;
    }

    let factor = 1.0 / (6.0 * signed_area);
    Vec2::new(cx * factor, cy * factor)
}

/// Check whether `point` is on the inside (left side) of the directed edge
/// from `edge_start` to `edge_end`.
fn is_inside_edge(point: Vec2, edge_start: Vec2, edge_end: Vec2) -> bool {
    let edge = edge_end - edge_start;
    let to_point = point - edge_start;
    // 2D cross product (z-component): positive = left side = inside for CCW.
    edge.x * to_point.y - edge.y * to_point.x >= -1e-6
}

/// Compute the intersection point of line segment (p1→p2) with the infinite
/// line through (e1→e2).  Returns `None` if the lines are parallel.
fn line_intersection(p1: Vec2, p2: Vec2, e1: Vec2, e2: Vec2) -> Option<Vec2> {
    let d1 = p2 - p1;
    let d2 = e2 - e1;
    let cross = d1.x * d2.y - d1.y * d2.x;

    if cross.abs() < 1e-10 {
        return None; // Parallel.
    }

    let t = ((e1.x - p1.x) * d2.y - (e1.y - p1.y) * d2.x) / cross;
    Some(p1 + d1 * t)
}

/// Convex hull via Graham scan.
///
/// Returns vertices in CCW order.
fn convex_hull_graham(points: &[Vec2]) -> Vec<Vec2> {
    let n = points.len();
    if n <= 3 {
        return points.to_vec();
    }

    // Find the point with the lowest y (and lowest x in case of tie).
    let mut min_idx = 0;
    for i in 1..n {
        if points[i].y < points[min_idx].y
            || (points[i].y == points[min_idx].y && points[i].x < points[min_idx].x)
        {
            min_idx = i;
        }
    }

    let pivot = points[min_idx];

    // Sort remaining points by polar angle relative to pivot.
    let mut sorted: Vec<(f32, Vec2)> = points
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != min_idx)
        .map(|(_, &p)| {
            let angle = (p.y - pivot.y).atan2(p.x - pivot.x);
            (angle, p)
        })
        .collect();

    sorted.sort_by(|a, b| {
        a.0.partial_cmp(&b.0).unwrap().then_with(|| {
            let da = (a.1 - pivot).length_squared();
            let db = (b.1 - pivot).length_squared();
            da.partial_cmp(&db).unwrap()
        })
    });

    // Remove duplicate angles (keep farthest — last in sorted order).
    let mut filtered: Vec<Vec2> = Vec::with_capacity(sorted.len());
    for i in 0..sorted.len() {
        if i + 1 < sorted.len() && (sorted[i + 1].0 - sorted[i].0).abs() < 1e-10 {
            // Next point has same angle — skip this one (keep the farther one).
            continue;
        }
        filtered.push(sorted[i].1);
    }

    if filtered.len() < 2 {
        return vec![pivot];
    }

    // Graham scan.
    let mut hull: Vec<Vec2> = vec![pivot, filtered[0]];

    for i in 1..filtered.len() {
        while hull.len() >= 2 {
            let top = hull[hull.len() - 1];
            let next = hull[hull.len() - 2];
            let cross = (top - next).perp_dot(filtered[i] - top);
            if cross <= 1e-6 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(filtered[i]);
    }

    hull
}

// Extension trait for perp_dot since not all glam versions have it.
#[allow(dead_code)]
trait Vec2Ext {
    fn perp_dot(self, other: Vec2) -> f32;
}

impl Vec2Ext for Vec2 {
    fn perp_dot(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- LSystem tests ---

    #[test]
    fn test_lsystem_deterministic() {
        let mut lsys = LSystem::new("F", 90.0, 1.0);
        lsys.add_rule('F', "F+F-F-F+F");
        let result = lsys.iterate(1);
        assert_eq!(result, "F+F-F-F+F");
    }

    #[test]
    fn test_lsystem_depth_2() {
        let mut lsys = LSystem::new("F", 90.0, 1.0);
        lsys.add_rule('F', "F+F-F-F+F");
        let result = lsys.iterate(2);
        // Each F in "F+F-F-F+F" is replaced by the same pattern.
        assert!(result.contains("F+F-F-F+F"));
    }

    #[test]
    fn test_lsystem_with_seed_deterministic() {
        let mut lsys1 = LSystem::new("F", 90.0, 1.0).with_seed(123);
        lsys1.add_stochastic_rule('F', "F+F", 0.5);
        lsys1.add_stochastic_rule('F', "F-F", 0.5);

        let mut lsys2 = LSystem::new("F", 90.0, 1.0).with_seed(123);
        lsys2.add_stochastic_rule('F', "F+F", 0.5);
        lsys2.add_stochastic_rule('F', "F-F", 0.5);

        // Same seed should produce same result.
        assert_eq!(lsys1.expanded(), lsys2.expanded());
    }

    #[test]
    fn test_lsystem_branches_preserved() {
        let mut lsys = LSystem::new("F", 90.0, 1.0);
        lsys.add_rule('F', "F[+F]F[-F]F");
        // Brackets and +/- should be preserved in the string.
        let result = lsys.iterate(1);
        assert!(result.contains('['));
        assert!(result.contains(']'));
        assert!(result.contains('+'));
        assert!(result.contains('-'));
    }

    #[test]
    fn test_lsystem_generate_produces_segments() {
        let mut lsys = LSystem::new("F", 60.0, 1.0);
        lsys.add_rule('F', "FF+[+F-F-F]-[-F+F+F]");
        lsys.set_depth(0);
        let segments = lsys.generate();
        assert_eq!(segments.len(), 1); // Single F in axiom.
    }

    #[test]
    fn test_lsystem_no_rule_identity() {
        let lsys = LSystem::new("F+F--F+F", 90.0, 1.0);
        // No rules added — identity expansion.
        assert_eq!(lsys.expanded(), "F+F--F+F");
    }

    #[test]
    fn test_lsystem_stochastic_probability_zero() {
        let mut lsys = LSystem::new("F", 90.0, 1.0).with_seed(0);
        lsys.add_stochastic_rule('F', "AB", 0.0);
        lsys.add_stochastic_rule('F', "XY", 1.0);
        // Weight 0 vs 1 should always pick XY.
        let result = lsys.iterate(1);
        assert_eq!(result, "XY");
    }

    /// Verify that L-system generates segments with correct start/end
    /// connectivity (turtle moves forward by `step`).
    #[test]
    fn test_lsystem_segment_geometry() {
        let lsys = LSystem::new("FF", 90.0, 2.0);
        let segments = lsys.generate();
        assert_eq!(segments.len(), 2);
        // Both segments start at origin for depth 0.
        assert_eq!(segments[0].start, Vec2::ZERO);
        assert_eq!(segments[0].end, Vec2::new(2.0, 0.0));
        // Second segment starts where first ended.
        assert_eq!(segments[1].start, Vec2::new(2.0, 0.0));
    }

    #[test]
    fn test_lsystem_moves_no_draw() {
        let lsys = LSystem::new("fF", 90.0, 1.0);
        let segments = lsys.generate();
        assert_eq!(segments.len(), 1); // Only 'F' draws.
        // Turtle moved via 'f' first, so F is at (1,0).
        assert_eq!(segments[0].start, Vec2::new(1.0, 0.0));
    }

    // --- VoronoiFracture tests ---

    #[test]
    fn test_voronoi_generate_creates_cells() {
        let mut vf = VoronoiFracture::new().with_seed(42).with_resolution(32);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let fragments = vf.fracture(
            bounds,
            &[
                Vec2::new(10.0, 10.0),
                Vec2::new(90.0, 10.0),
                Vec2::new(90.0, 90.0),
                Vec2::new(10.0, 90.0),
            ],
            5,
        );
        assert!(!fragments.is_empty());
        // Each fragment should have at least 3 vertices.
        for frag in &fragments {
            assert!(frag.vertices.len() >= 3);
            assert!(frag.area > 0.0);
        }
    }

    #[test]
    fn test_voronoi_deterministic() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let polygon = vec![
            Vec2::new(10.0, 10.0),
            Vec2::new(90.0, 10.0),
            Vec2::new(90.0, 90.0),
            Vec2::new(10.0, 90.0),
        ];

        let mut vf1 = VoronoiFracture::new().with_seed(7).with_resolution(32);
        let f1 = vf1.fracture(bounds, &polygon, 4);

        let mut vf2 = VoronoiFracture::new().with_seed(7).with_resolution(32);
        let f2 = vf2.fracture(bounds, &polygon, 4);

        assert_eq!(f1.len(), f2.len());
        for (a, b) in f1.iter().zip(f2.iter()) {
            assert_eq!(a.vertices.len(), b.vertices.len());
            assert!((a.velocity - b.velocity).length() < 1e-6);
        }
    }

    #[test]
    fn test_voronoi_single_seed() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let mut vf = VoronoiFracture::new().with_seed(0).with_resolution(32);
        vf.generate(1, bounds);

        assert_eq!(vf.cells.len(), 1);
    }

    #[test]
    fn test_voronoi_zero_seeds() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let mut vf = VoronoiFracture::new();
        vf.generate(0, bounds);
        assert!(vf.cells.is_empty());
    }

    #[test]
    fn test_voronoi_no_cells_returns_input() {
        let vf = VoronoiFracture::new();
        let polygon = vec![Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let result = vf.clip_polygon(&polygon);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, polygon);
    }

    // --- Utility function tests ---

    #[test]
    fn test_polygon_area_triangle() {
        let tri = vec![Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(0.0, 3.0)];
        let area = polygon_area(&tri);
        assert!((area - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_polygon_area_square() {
        let sq = vec![
            Vec2::ZERO,
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        assert!((polygon_area(&sq) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_polygon_centroid_square() {
        let sq = vec![
            Vec2::ZERO,
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let c = polygon_centroid(&sq);
        assert!((c - Vec2::new(1.0, 1.0)).length() < 1e-6);
    }

    #[test]
    fn test_convex_hull_square() {
        let points = vec![
            Vec2::ZERO,
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.5, 0.5), // interior point
        ];
        let hull = convex_hull_graham(&points);
        eprintln!("hull: {:?}", hull);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn test_bounds_contains() {
        let b = Bounds::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(b.contains(Vec2::new(5.0, 5.0)));
        assert!(b.contains(Vec2::ZERO));
        assert!(b.contains(Vec2::new(10.0, 10.0)));
        assert!(!b.contains(Vec2::new(11.0, 5.0)));
    }

    #[test]
    fn test_sutherland_hodgman_full_overlap() {
        let subject = vec![
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let clip = subject.clone();
        let result = VoronoiFracture::sutherland_hodgman(&subject, &clip);
        assert!(result.is_some());
    }

    #[test]
    fn test_sutherland_hodgman_partial_overlap() {
        let subject = vec![
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let clip = vec![
            Vec2::new(5.0, -5.0),
            Vec2::new(15.0, -5.0),
            Vec2::new(15.0, 5.0),
            Vec2::new(5.0, 5.0),
        ];
        let result = VoronoiFracture::sutherland_hodgman(&subject, &clip);
        assert!(result.is_some());
        let clipped = result.unwrap();
        assert!(clipped.len() >= 3);
    }

    #[test]
    fn test_sutherland_hodgman_no_overlap() {
        let subject = vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)];
        let clip = vec![
            Vec2::new(10.0, 10.0),
            Vec2::new(20.0, 10.0),
            Vec2::new(20.0, 20.0),
        ];
        assert!(VoronoiFracture::sutherland_hodgman(&subject, &clip).is_none());
    }

    #[test]
    fn test_simple_rng_deterministic() {
        let mut r1 = SimpleRng::new(99);
        let mut r2 = SimpleRng::new(99);
        for _ in 0..100 {
            assert_eq!(r1.next_u64(), r2.next_u64());
        }
    }

    #[test]
    fn test_simple_rng_f32_range() {
        let mut r = SimpleRng::new(1);
        for _ in 0..1000 {
            let v = r.next_f32();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    /// End-to-end: fracture a large polygon and verify fragments have
    /// velocities pointing away from the fracture centre.
    #[test]
    fn test_fracture_velocities_point_outward() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(200.0, 200.0));
        let center = bounds.center();
        let polygon = vec![
            Vec2::new(50.0, 50.0),
            Vec2::new(150.0, 50.0),
            Vec2::new(150.0, 150.0),
            Vec2::new(50.0, 150.0),
        ];

        let mut vf = VoronoiFracture::new().with_seed(1).with_resolution(64);
        let fragments = vf.fracture(bounds, &polygon, 6);

        assert!(!fragments.is_empty());
        for frag in &fragments {
            let outward = (frag.centroid - center).normalize_or_zero();
            if frag.velocity.length_squared() > 0.0 && outward.length_squared() > 0.0 {
                let alignment = frag.velocity.normalize().dot(outward);
                // Velocity should generally point outward (positive dot).
                assert!(
                    alignment > -0.5,
                    "Fragment velocity not outward: {}",
                    alignment
                );
            }
        }
    }
}

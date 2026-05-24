use cvkg_core::{Never, Rect, Renderer, View};

/// MjolnirFrame - A geometric, non-rectangular UI frame with chromatic aberration.
/// Section 4.5: "Mjolnir's Edge — Geometric slicing and destructive visual feedback."
pub struct MjolnirFrame {
    /// Color of the main geometric border.
    pub border_color: [f32; 4],
    /// Width of the drawn border stroke.
    pub border_width: f32,
    /// Size of the cut-off/beveled corners.
    pub bevel_size: f32,
    /// Amplitude of chromatic aberration color shift.
    pub glitch_intensity: f32,
}

impl Default for MjolnirFrame {
    /// Creates a default MjolnirFrame instance.
    ///
    /// # Contract
    /// - Uses default color, border width, bevel size, and glitch intensity.
    fn default() -> Self {
        Self::new()
    }
}

impl MjolnirFrame {
    /// Creates a new MjolnirFrame with default geometric and visual styling.
    ///
    /// # Contract
    /// - Defaults to a cyan border, 1.5 width, 20.0 bevel size, and 0.1 glitch intensity.
    pub fn new() -> Self {
        Self {
            border_color: [0.0, 1.0, 1.0, 0.8], // Cyan Default
            border_width: 1.5,
            bevel_size: 20.0,
            glitch_intensity: 0.1,
        }
    }

    /// Sets the color of the frame border.
    ///
    /// # Contract
    /// - Returns the modified Self.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    /// Sets the glitch intensity/amplitude.
    ///
    /// # Contract
    /// - Returns the modified Self.
    pub fn with_glitch(mut self, intensity: f32) -> Self {
        self.glitch_intensity = intensity;
        self
    }

    /// Check if the point `(px, py)` lies inside the beveled frame polygon.
    /// This represents the CPU-side point-in-polygon (SDF) hit-testing calculation
    /// to ensure perfect alignment with GPU clipping and rendering boundaries.
    ///
    /// # Contract
    /// - Returns `true` if `(px, py)` is inside the boundaries of the beveled frame.
    /// - Returns `false` if it is outside, including inside the cut-off corners.
    pub fn contains_point(&self, rect: Rect, px: f32, py: f32) -> bool {
        let bevel = self.bevel_size;
        // Check if the point lies inside the main bounding box first
        if px < rect.x || px > rect.x + rect.width || py < rect.y || py > rect.y + rect.height {
            return false;
        }

        // If the bevel is non-positive or too large, fall back to simple bounding box containment.
        if bevel <= 0.0 {
            return true;
        }

        let bevel = bevel.min(rect.width / 2.0).min(rect.height / 2.0);

        // Check top-left bevel: (px - rect.x) + (py - rect.y) >= bevel in top-left region
        if (px - rect.x) < bevel && (py - rect.y) < bevel {
            if (px - rect.x) + (py - rect.y) < bevel {
                return false;
            }
        }

        // Check top-right bevel: ((rect.x + rect.width) - px) + (py - rect.y) >= bevel in top-right region
        if ((rect.x + rect.width) - px) < bevel && (py - rect.y) < bevel {
            if ((rect.x + rect.width) - px) + (py - rect.y) < bevel {
                return false;
            }
        }

        // Check bottom-right bevel: ((rect.x + rect.width) - px) + ((rect.y + rect.height) - py) >= bevel in bottom-right region
        if ((rect.x + rect.width) - px) < bevel && ((rect.y + rect.height) - py) < bevel {
            if ((rect.x + rect.width) - px) + ((rect.y + rect.height) - py) < bevel {
                return false;
            }
        }

        // Check bottom-left bevel: (px - rect.x) + ((rect.y + rect.height) - py) >= bevel in bottom-left region
        if (px - rect.x) < bevel && ((rect.y + rect.height) - py) < bevel {
            if (px - rect.x) + ((rect.y + rect.height) - py) < bevel {
                return false;
            }
        }

        true
    }
}

impl View for MjolnirFrame {
    type Body = Never;

    /// The body of primitive components like MjolnirFrame returns `Never` as they are rendered
    /// directly by the pipeline.
    ///
    /// # Contract
    /// - Always panics with unreachable, as this component overrides the `render` method directly.
    fn body(self) -> Self::Body {
        unreachable!()
    }

    /// Renders the MjolnirFrame by drawing a beveled border, applying time-varying chromatic aberration glitch
    /// effects, and filling the interior with a scanline glow animation.
    ///
    /// # Contract
    /// - `rect` specifies the assigned bounds of the frame.
    /// - Line drawings and fill operations are dispatched to the `renderer`.
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let bevel = self.bevel_size;

        // 1. Calculate path points for a beveled rectangle
        // TL, TR, BR, BL with slices
        let points = [
            (rect.x + bevel, rect.y),
            (rect.x + rect.width - bevel, rect.y),
            (rect.x + rect.width, rect.y + bevel),
            (rect.x + rect.width, rect.y + rect.height - bevel),
            (rect.x + rect.width - bevel, rect.y + rect.height),
            (rect.x + bevel, rect.y + rect.height),
            (rect.x, rect.y + rect.height - bevel),
            (rect.x, rect.y + bevel),
        ];

        // 2. Main Border
        self.draw_beveled_path(renderer, &points, self.border_color, self.border_width);

        // 3. Chromatic Aberration (Glitch)
        if self.glitch_intensity > 0.0 {
            let offset = (t * 10.0).sin() * self.glitch_intensity * 2.0;

            // Red Shift
            let mut red_points = points;
            for p in &mut red_points {
                p.0 += offset;
            }
            self.draw_beveled_path(renderer, &red_points, [1.0, 0.0, 0.0, 0.4], 1.0);

            // Blue Shift
            let mut blue_points = points;
            for p in &mut blue_points {
                p.0 -= offset;
            }
            self.draw_beveled_path(renderer, &blue_points, [0.0, 0.0, 1.0, 0.4], 1.0);
        }

        // 4. Inner "Scanline" Glow
        let alpha = (t * 2.0).sin().abs() * 0.1 + 0.05;
        renderer.fill_rect(
            rect,
            [
                self.border_color[0],
                self.border_color[1],
                self.border_color[2],
                alpha,
            ],
        );
    }
}

impl MjolnirFrame {
    /// Renders individual line segments connecting the vertices of the beveled polygon.
    ///
    /// # Contract
    /// - Iterates and draws lines sequentially from point to point, closing the loop.
    fn draw_beveled_path(
        &self,
        renderer: &mut dyn Renderer,
        points: &[(f32, f32); 8],
        color: [f32; 4],
        width: f32,
    ) {
        for i in 0..8 {
            let p1 = points[i];
            let p2 = points[(i + 1) % 8];
            renderer.draw_line(p1.0, p1.1, p2.0, p2.1, color, width);
        }
    }
}

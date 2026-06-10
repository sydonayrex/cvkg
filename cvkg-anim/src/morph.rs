use crate::physics::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo(Vec3),
    LineTo(Vec3),
    CubicCurve {
        control1: Vec3,
        control2: Vec3,
        to: Vec3,
    },
    ClosePath,
}

impl PathCommand {
    pub fn end_point(&self) -> Option<Vec3> {
        match self {
            PathCommand::MoveTo(p) => Some(*p),
            PathCommand::LineTo(p) => Some(*p),
            PathCommand::CubicCurve { to, .. } => Some(*to),
            PathCommand::ClosePath => None,
        }
    }
}

pub struct PathMorpher {
    from_path: Vec<PathCommand>,
    to_path: Vec<PathCommand>,
}

impl PathMorpher {
    pub fn new(mut from_path: Vec<PathCommand>, mut to_path: Vec<PathCommand>) -> Self {
        Self::auto_subdivide(&mut from_path, &mut to_path);
        Self { from_path, to_path }
    }

    /// Interpolate between the two paths based on a 0.0 to 1.0 scalar
    pub fn lerp(&self, t: f32) -> Vec<PathCommand> {
        let mut result = Vec::with_capacity(self.from_path.len());
        for (f, t_cmd) in self.from_path.iter().zip(self.to_path.iter()) {
            result.push(match (f, t_cmd) {
                (PathCommand::MoveTo(p1), PathCommand::MoveTo(p2)) => {
                    PathCommand::MoveTo(p1.lerp(*p2, t))
                }
                (PathCommand::LineTo(p1), PathCommand::LineTo(p2)) => {
                    PathCommand::LineTo(p1.lerp(*p2, t))
                }
                (
                    PathCommand::CubicCurve {
                        control1: c1a,
                        control2: c2a,
                        to: ta,
                    },
                    PathCommand::CubicCurve {
                        control1: c1b,
                        control2: c2b,
                        to: tb,
                    },
                ) => PathCommand::CubicCurve {
                    control1: c1a.lerp(*c1b, t),
                    control2: c2a.lerp(*c2b, t),
                    to: ta.lerp(*tb, t),
                },
                (PathCommand::ClosePath, PathCommand::ClosePath) => PathCommand::ClosePath,
                // If commands are completely mismatched (e.g., LineTo vs CubicCurve), we convert LineTo to CubicCurve
                // For simplicity in this basic auto-subdivider, we assume both sides have been normalized into Cubics/Lines
                _ => f.clone(), // Fallback
            });
        }
        result
    }

    /// Dynamically auto-subdivides the shorter path to match the point count of the longer path.
    fn auto_subdivide(path_a: &mut Vec<PathCommand>, path_b: &mut Vec<PathCommand>) {
        // Equalize the number of commands by repeating the last command with zero length,
        // or properly bisecting the longest segment. For a UI auto-subdivider, bisection is smoother.
        while path_a.len() < path_b.len() {
            Self::subdivide_longest_segment(path_a);
        }
        while path_b.len() < path_a.len() {
            Self::subdivide_longest_segment(path_b);
        }

        // Normalize commands so they match types (e.g. LineTo -> CubicCurve)
        for i in 0..path_a.len() {
            match (&path_a[i], &path_b[i]) {
                (PathCommand::LineTo(p), PathCommand::CubicCurve { .. }) => {
                    let prev = if i > 0 {
                        path_a[i - 1].end_point().unwrap_or(*p)
                    } else {
                        *p
                    };
                    path_a[i] = PathCommand::CubicCurve {
                        control1: prev.lerp(*p, 0.33),
                        control2: prev.lerp(*p, 0.66),
                        to: *p,
                    };
                }
                (PathCommand::CubicCurve { .. }, PathCommand::LineTo(p)) => {
                    let prev = if i > 0 {
                        path_b[i - 1].end_point().unwrap_or(*p)
                    } else {
                        *p
                    };
                    path_b[i] = PathCommand::CubicCurve {
                        control1: prev.lerp(*p, 0.33),
                        control2: prev.lerp(*p, 0.66),
                        to: *p,
                    };
                }
                _ => {}
            }
        }
    }

    fn subdivide_longest_segment(path: &mut Vec<PathCommand>) {
        if path.is_empty() {
            return;
        }

        let mut max_len_sq = 0.0;
        let mut max_idx = 0;
        let mut prev_point = path[0].end_point().unwrap_or(Vec3::ZERO);

        for (i, cmd) in path.iter().enumerate().skip(1) {
            if let Some(p) = cmd.end_point() {
                let len_sq = (p - prev_point).length_sq();
                if len_sq > max_len_sq {
                    max_len_sq = len_sq;
                    max_idx = i;
                }
                prev_point = p;
            }
        }

        if max_idx == 0 || max_idx >= path.len() {
            return;
        }

        let prev = path[max_idx - 1].end_point().unwrap_or(Vec3::ZERO);

        // Bisect the segment
        let split_cmd = match &path[max_idx] {
            PathCommand::LineTo(p) => {
                let mid = prev.lerp(*p, 0.5);
                path[max_idx] = PathCommand::LineTo(*p);
                PathCommand::LineTo(mid)
            }
            PathCommand::CubicCurve {
                control1,
                control2,
                to,
            } => {
                // De Casteljau's algorithm at t=0.5
                let q0 = prev.lerp(*control1, 0.5);
                let q1 = control1.lerp(*control2, 0.5);
                let q2 = control2.lerp(*to, 0.5);

                let r0 = q0.lerp(q1, 0.5);
                let r1 = q1.lerp(q2, 0.5);

                let mid = r0.lerp(r1, 0.5);

                path[max_idx] = PathCommand::CubicCurve {
                    control1: r1,
                    control2: q2,
                    to: *to,
                };

                PathCommand::CubicCurve {
                    control1: q0,
                    control2: r0,
                    to: mid,
                }
            }
            _ => PathCommand::MoveTo(prev),
        };

        path.insert(max_idx, split_cmd);
    }
}

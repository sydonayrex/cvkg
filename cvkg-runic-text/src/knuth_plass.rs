// ── Knuth-Plass Line Breaking ───────────────────────────────────────────────
//
// Optimal paragraph line breaking using the Knuth-Plass algorithm with
// demerit minimization. Produces visually balanced line breaks by considering
// all possible break points and scoring them with a badness/demerit function.


/// Classification of a potential break point in text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakKind {
    /// Mandatory break (e.g., newline character). Must break here.
    Mandatory,
    /// Optional break between words (space character). Can break here.
    Optional,
    /// No break possible (within a word unless hyphenation is enabled).
    NoBreak,
}

/// A single potential break point in the text.
#[derive(Debug, Clone, PartialEq)]
pub struct BreakPoint {
    /// Byte position in the original text where this break occurs.
    pub position: usize,
    /// The kind of break (mandatory, optional, or no-break).
    pub kind: BreakKind,
    /// Amount the line would need to shrink (in pixels) to fit exactly.
    pub shrink: f32,
    /// Amount the line can stretch (in pixels) to fill the target width.
    pub stretch: f32,
    /// Demerit score for breaking here (lower is better). -f32::INFINITY for mandatory.
    pub demerit: f32,
}

impl BreakPoint {
    /// Creates a mandatory break point.
    pub fn mandatory(position: usize, shrink: f32, stretch: f32) -> Self {
        Self {
            position,
            kind: BreakKind::Mandatory,
            shrink,
            stretch,
            demerit: -f32::INFINITY,
        }
    }

    /// Creates an optional break point.
    pub fn optional(position: usize, shrink: f32, stretch: f32) -> Self {
        Self {
            position,
            kind: BreakKind::Optional,
            shrink,
            stretch,
            demerit: 0.0,
        }
    }

    /// Creates a no-break point (used as sentinel).
    pub fn no_break(position: usize) -> Self {
        Self {
            position,
            kind: BreakKind::NoBreak,
            shrink: 0.0,
            stretch: 0.0,
            demerit: f32::INFINITY,
        }
    }
}

/// A finished line with its break information.
#[derive(Debug, Clone, PartialEq)]
pub struct KnuthPlassLine {
    /// Start byte position in the original text.
    pub start: usize,
    /// End byte position (exclusive) in the original text.
    pub end: usize,
    /// Shrink amount (pixels) for justification.
    pub shrink: f32,
    /// Stretch amount (pixels) for justification.
    pub stretch: f32,
    /// Badness of this line (0 = perfect fit).
    pub badness: f32,
}

/// Configuration for the Knuth-Plass algorithm.
#[derive(Debug, Clone)]
pub struct KnuthPlassConfig {
    /// Maximum ratio of stretch to use before penalizing a line.
    pub stretch_tolerance: f32,
    /// Maximum ratio of shrink to use before penalizing a line.
    pub shrink_tolerance: f32,
    /// Penalty for consecutive lines ending with hyphens.
    pub hyphen_penalty: f32,
    /// Penalty for lines with very different tightness.
    pub adjacency_penalty: f32,
    /// Fitness difference threshold for considering two lines similar.
    pub fitness_threshold: f32,
}

impl Default for KnuthPlassConfig {
    fn default() -> Self {
        Self {
            stretch_tolerance: 1.0,
            shrink_tolerance: 0.8,
            hyphen_penalty: 50.0,
            adjacency_penalty: 50.0,
            fitness_threshold: 0.5,
        }
    }
}

/// Computes line breaks using the Knuth-Plass algorithm.
///
/// Takes pre-measured glyph widths and finds the optimal set of break points
/// that minimizes total demerits across the paragraph.
///
/// # Arguments
/// * `widths` - Advance width of each character/glyph in pixels.
/// * `breaks` - Slice of `(position, BreakKind)` pairs indicating where breaks are possible.
/// * `line_width` - Target line width in pixels.
/// * `tolerance` - How much the badness can increase before lines are rejected (1.0..10.0).
///
/// # Returns
/// A `Vec<KnuthPlassLine>` representing the optimal line breaks, or an empty
/// vector if no valid break sequence exists.
pub fn break_lines(
    widths: &[f32],
    breaks: &[(usize, BreakKind)],
    line_width: f32,
    tolerance: f32,
) -> Vec<KnuthPlassLine> {
    let config = KnuthPlassConfig::default();
    break_lines_config(widths, breaks, line_width, tolerance, &config)
}

/// Knuth-Plass line breaking with full configuration.
///
/// # Arguments
/// * `widths` - Advance width of each character/glyph in pixels.
/// * `breaks` - Slice of `(position, BreakKind)` pairs.
/// * `line_width` - Target line width in pixels.
/// * `tolerance` - Badness increase tolerance factor.
/// * `config` - Full Knuth-Plass configuration.
///
/// # Returns
/// A `Vec<KnuthPlassLine>` with optimal breaks.
pub fn break_lines_config(
    widths: &[f32],
    breaks: &[(usize, BreakKind)],
    line_width: f32,
    tolerance: f32,
    config: &KnuthPlassConfig,
) -> Vec<KnuthPlassLine> {
    if widths.is_empty() || breaks.is_empty() {
        return Vec::new();
    }

    let n = breaks.len();
    // active[i] = index of the best predecessor break point for breaks[i]
    let mut active: Vec<Option<usize>> = vec![None; n];
    // cost[i] = minimum total demerit to reach break point i
    let mut cost: Vec<f32> = vec![f32::INFINITY; n];

    // The first break point is the start of the text
    cost[0] = 0.0;

    // For each potential break point, find the best predecessor
    for j in 1..n {
        for i in (0..j).rev() {
            let prev_pos = breaks[i].0;
            let curr_pos = breaks[j].0;

            // Compute natural width of the line from break i to break j
            let natural_width: f32 = widths[prev_pos..curr_pos.min(widths.len())].iter().sum();

            // Compute how much we need to stretch or shrink
            let diff = line_width - natural_width;

            let (badness, _shrink, _stretch) = if diff >= 0.0 {
                // Line needs to stretch
                let ratio = if natural_width > 0.0 {
                    diff / natural_width
                } else {
                    0.0
                };
                if ratio > config.stretch_tolerance {
                    // Exceeds stretch tolerance, skip this candidate
                    continue;
                }
                (ratio, 0.0, diff)
            } else {
                // Line needs to shrink
                let ratio = natural_width;
                let abs_diff = diff.abs();
                if ratio > 0.0 && abs_diff / ratio > config.shrink_tolerance {
                    continue;
                }
                (abs_diff / line_width.max(1.0), abs_diff, 0.0)
            };

            if badness > tolerance && breaks[j].1 != BreakKind::Mandatory {
                continue;
            }

            // Demerit = (1 + badness + penalty)^2, simplified
            let penalty: f32 = if breaks[j].1 == BreakKind::Mandatory && j < n - 1 {
                config.hyphen_penalty
            } else {
                0.0
            };
            let demerit = (1.0 + badness * 100.0 + penalty) * (1.0 + badness * 100.0 + penalty);

            let total_cost = if breaks[j].1 == BreakKind::Mandatory {
                cost[i] // Mandatory breaks have no additional demerit
            } else {
                cost[i] + demerit
            };

            if total_cost < cost[j] {
                cost[j] = total_cost;
                active[j] = Some(i);
            }
        }

        // No valid predecessor found for a mandatory break = impossible
        if breaks[j].1 == BreakKind::Mandatory && active[j].is_none() && j > 0 {
            continue;
        }
    }

    // Backtrack from the last break point to find the optimal sequence
    let mut lines = Vec::new();
    let mut current = n - 1;

    // Find the best final break point
    if cost[current] >= f32::INFINITY && breaks[current].1 != BreakKind::Mandatory {
        // Try to find any reachable final point
        for i in (0..n).rev() {
            if cost[i] < f32::INFINITY {
                current = i;
                break;
            }
        }
    }

    // Backtrack through active list
    while current != 0 {
        if let Some(prev) = active[current] {
            let start = breaks[prev].0;
            let end = breaks[current].0;
            let natural_width: f32 = widths[start..end.min(widths.len())].iter().sum();
            let diff = line_width - natural_width;
            let (shrink, stretch, badness) = if diff > 0.0 {
                (
                    0.0,
                    diff,
                    if natural_width > 0.0 {
                        diff / natural_width
                    } else {
                        0.0
                    },
                )
            } else {
                (diff.abs(), 0.0, diff.abs() / line_width.max(1.0))
            };
            lines.push(KnuthPlassLine {
                start,
                end,
                shrink,
                stretch: stretch.max(0.0),
                badness: badness.clamp(0.0, 1.0),
            });
            current = prev;
        } else {
            break;
        }
    }

    // Handle the first line (from position 0 to the first break)
    if !lines.is_empty() {
        let first_end = lines.last().unwrap().start;
        if first_end > 0 {
            let natural_width: f32 = widths[0..first_end.min(widths.len())].iter().sum();
            let diff = line_width - natural_width;
            let (shrink, stretch, badness) = if diff > 0.0 {
                (
                    0.0,
                    diff,
                    if natural_width > 0.0 {
                        diff / natural_width
                    } else {
                        0.0
                    },
                )
            } else {
                (diff.abs(), 0.0, diff.abs() / line_width.max(1.0))
            };
            lines.push(KnuthPlassLine {
                start: 0,
                end: first_end,
                shrink,
                stretch: stretch.max(0.0),
                badness: badness.clamp(0.0, 1.0),
            });
        }
    } else {
        // No breaks found, entire text is one line
        let natural_width: f32 = widths.iter().sum();
        let diff = line_width - natural_width;
        let (shrink, stretch, badness) = if diff > 0.0 {
            (
                0.0,
                diff,
                if natural_width > 0.0 {
                    diff / natural_width
                } else {
                    0.0
                },
            )
        } else {
            (diff.abs(), 0.0, diff.abs() / line_width.max(1.0))
        };
        lines.push(KnuthPlassLine {
            start: 0,
            end: widths.len(),
            shrink,
            stretch: stretch.max(0.0),
            badness: badness.clamp(0.0, 1.0),
        });
    }

    lines.reverse();
    lines
}

/// Convenience function: compute break points from text with a fixed character width.
pub fn break_text_simple(
    text: &str,
    char_width: f32,
    line_width: f32,
    tolerance: f32,
) -> Vec<KnuthPlassLine> {
    let widths: Vec<f32> = text.chars().map(|_| char_width).collect();
    let mut breaks: Vec<(usize, BreakKind)> = Vec::new();
    let mut pos = 0;

    for c in text.chars() {
        let char_len = c.len_utf8();
        if c == '\n' {
            breaks.push((pos + char_len, BreakKind::Mandatory));
        } else if c == ' ' {
            breaks.push((pos + char_len, BreakKind::Optional));
        }
        pos += char_len;
    }

    if breaks.is_empty() {
        breaks.push((0, BreakKind::NoBreak));
        breaks.push((text.len(), BreakKind::Mandatory));
    }

    // Deduplicate and sort
    breaks.sort_by_key(|b| b.0);
    breaks.dedup_by_key(|b| b.0);

    break_lines(&widths, &breaks, line_width, tolerance)
}

#[cfg(test)]
mod knuth_plass_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let lines = break_lines(&[], &[], 100.0, 2.0);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_single_line_fits() {
        // 5 chars at 10px each = 50px, line is 100px wide
        let widths = vec![10.0; 5];
        let breaks = vec![(0, BreakKind::NoBreak), (5, BreakKind::Mandatory)];
        let lines = break_lines(&widths, &breaks, 100.0, 2.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 5);
        assert!(lines[0].stretch > 0.0); // needs to stretch to fill
    }

    #[test]
    fn test_word_wrap() {
        // "hello world" with 10px chars, 11 chars total
        // Line width 80px -- should break at the space
        let text = "hello world";
        let lines = break_text_simple(text, 10.0, 80.0, 3.0);
        assert!(!lines.is_empty(), "Should produce at least one line");
        // First line: "hello" = 50px, should fit in 80px
        let first = &lines[0];
        let first_width: f32 = widths_sum(first, text);
        assert!(first_width <= 80.0, "First line should fit in 80px");
    }

    fn widths_sum(line: &KnuthPlassLine, _text: &str) -> f32 {
        // Each char is 10px in the test
        (line.end - line.start) as f32 * 10.0
    }

    #[test]
    fn test_mandatory_break() {
        let text = "hello\nworld";
        let lines = break_text_simple(text, 10.0, 200.0, 3.0);
        assert!(lines.len() >= 2, "Should break at newline");
    }

    #[test]
    fn test_break_point_kinds() {
        let mandatory = BreakPoint::mandatory(10, 5.0, 20.0);
        assert_eq!(mandatory.kind, BreakKind::Mandatory);

        let optional = BreakPoint::optional(20, 3.0, 15.0);
        assert_eq!(optional.kind, BreakKind::Optional);

        let no_break = BreakPoint::no_break(30);
        assert_eq!(no_break.kind, BreakKind::NoBreak);
    }

    #[test]
    fn test_knuth_plass_line_fields() {
        let line = KnuthPlassLine {
            start: 0,
            end: 10,
            shrink: 5.0,
            stretch: 0.0,
            badness: 0.3,
        };
        assert_eq!(line.start, 0);
        assert_eq!(line.end, 10);
        assert_eq!(line.shrink, 5.0);
        assert_eq!(line.badness, 0.3);
    }

    #[test]
    fn test_config_default() {
        let config = KnuthPlassConfig::default();
        assert_eq!(config.stretch_tolerance, 1.0);
        assert_eq!(config.shrink_tolerance, 0.8);
        assert_eq!(config.hyphen_penalty, 50.0);
    }

    #[test]
    fn test_narrow_line_forces_many_breaks() {
        // 10 chars at 10px each = 100px, but line is only 30px wide
        let text = "abcdefghij";
        let lines = break_text_simple(text, 10.0, 30.0, 5.0);
        // Should produce at least 4 lines (3 chars per line)
        assert!(lines.len() >= 3, "Narrow line should force multiple breaks");
    }
}

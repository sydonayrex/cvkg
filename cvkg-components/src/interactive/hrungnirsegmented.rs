use cvkg_core::{Never, Rect, Renderer, View};

// Named after Hrungnir, whose heart was stone with three sharp corners.

/// A horizontal group of mutually exclusive options with glass styling.
pub struct HrungnirSegmented {
    pub segments: Vec<String>,
    pub selected: usize,
    pub style: SegmentedStyle,
    pill_x: f32,
    pill_width: f32,
    #[allow(dead_code)]
    anim: cvkg_anim::SleipnirSolver,
}

pub enum SegmentedStyle {
    Glass,
    Capsule,
    Iconic,
    Labeled,
}

impl HrungnirSegmented {
    pub fn new(segments: Vec<String>, selected: usize) -> Self {
        Self {
            segments,
            selected,
            style: SegmentedStyle::Glass,
            pill_x: 0.0,
            pill_width: 0.0,
            anim: cvkg_anim::SleipnirSolver::new(cvkg_anim::SleipnirParams::snappy(), 0.0, 0.0),
        }
    }

    pub fn style(mut self, style: SegmentedStyle) -> Self {
        self.style = style;
        self
    }

    pub fn on_select(&mut self, index: usize) {
        self.selected = index;
    }
}

impl View for HrungnirSegmented {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass platter background
        let radius = match self.style {
            SegmentedStyle::Capsule => rect.height / 2.0,
            _ => 8.0,
        };
        renderer.fill_rounded_rect(rect, radius, [0.1, 0.1, 0.12, 0.85]);

        // Sliding pill indicator (white tint at low opacity)
        let pill_rect = cvkg_core::Rect {
            x: rect.x + self.pill_x,
            y: rect.y + 2.0,
            width: self.pill_width,
            height: rect.height - 4.0,
        };
        renderer.fill_rounded_rect(pill_rect, 6.0, [1.0, 1.0, 1.0, 0.15]);

        // Segment labels
        let item_width = rect.width / self.segments.len().max(1) as f32;
        for (i, label) in self.segments.iter().enumerate() {
            let x = rect.x + i as f32 * item_width + item_width / 2.0;
            let w = renderer.measure_text(label, 12.0).0;
            let color = if i == self.selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.7, 0.7, 0.75, 1.0]
            };
            renderer.draw_text(label, x - w / 2.0, rect.y + 8.0, 12.0, color);
        }
    }
}

#[cfg(test)]
mod hrungnir_tests {
    use super::*;

    #[test]
    fn test_segmented_new() {
        let seg = HrungnirSegmented::new(vec!["A".into(), "B".into()], 0);
        assert_eq!(seg.segments.len(), 2);
        assert_eq!(seg.selected, 0);
        assert!(matches!(seg.style, SegmentedStyle::Glass));
    }

    #[test]
    fn test_segmented_select() {
        let mut seg = HrungnirSegmented::new(vec!["A".into(), "B".into(), "C".into()], 0);
        seg.on_select(2);
        assert_eq!(seg.selected, 2);
    }

    #[test]
    fn test_segmented_style() {
        let seg = HrungnirSegmented::new(vec!["A".into()], 0).style(SegmentedStyle::Capsule);
        assert!(matches!(seg.style, SegmentedStyle::Capsule));
    }

    #[test]
    fn test_segmented_single() {
        let seg = HrungnirSegmented::new(vec!["Only".into()], 0);
        assert_eq!(seg.segments.len(), 1);
    }

    #[test]
    fn test_segmented_many() {
        let seg = HrungnirSegmented::new(
            vec!["One".into(), "Two".into(), "Three".into(), "Four".into()],
            3,
        );
        assert_eq!(seg.segments.len(), 4);
        assert_eq!(seg.selected, 3);
    }
}

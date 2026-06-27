use crate::RADIUS_SM;
use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

const RUNES: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ', 'ᛒ', 'ᛖ',
    'ᛗ', 'ᛚ', 'ᛜ', 'ᛞ', 'ᛟ',
];

/// RuneScript - A text component that reveals itself with a runic "deciphering" animation.
/// Formerly ScanningText, renamed for Norse-themed tactical alignment.
#[derive(Clone)]
pub struct RuneScript {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub speed: f32, // Characters per second
}

impl RuneScript {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 14.0,
            color: theme::progress_fill(), // Cyan
            speed: 20.0,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for RuneScript {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let revealed_count = (t * self.speed) as usize;
        let mut display_text = String::new();

        let chars: Vec<char> = self.text.chars().collect();
        for i in 0..chars.len() {
            if i < revealed_count {
                display_text.push(chars[i]);
            } else if i < revealed_count + 4 {
                let rune_idx = ((t * 30.0 + i as f32) as usize) % RUNES.len();
                display_text.push(RUNES[rune_idx]);
            } else {
                break;
            }
        }

        if !display_text.is_empty() {
            renderer.draw_text(
                &display_text,
                rect.x,
                rect.y + self.font_size,
                self.font_size,
                self.color,
            );
        }
    }

    fn intrinsic_size(
        &self,
        renderer: &mut dyn Renderer,
        _proposal: cvkg_core::SizeProposal,
    ) -> cvkg_core::Size {
        let (w, h) = renderer.measure_text(&self.text, self.font_size);
        cvkg_core::Size {
            width: w,
            height: h,
        }
    }
}

/// SleipnirGait - A container that staggers the reveal of its children.
/// Named after Odin's 8-legged horse, known for its rapid and coordinated gait.
#[doc(alias = "StepIndicator")]
#[derive(Clone)]
pub struct SleipnirGait {
    pub children: Vec<cvkg_core::AnyView>,
    pub stagger_delay: f32, // Delay between child reveals in seconds
}

impl SleipnirGait {
    pub fn new(stagger_delay: f32) -> Self {
        Self {
            children: Vec::new(),
            stagger_delay,
        }
    }

    pub fn child<V: View + Clone + 'static>(mut self, view: V) -> Self {
        self.children.push(view.erase());
        self
    }
}

impl View for SleipnirGait {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let child_height = rect.height / self.children.len().max(1) as f32;

        for (i, child) in self.children.iter().enumerate() {
            let start_time = i as f32 * self.stagger_delay;
            if t < start_time {
                continue;
            }

            // Apply reveal opacity based on how long since its start time
            let opacity = ((t - start_time) * 4.0).min(1.0);
            renderer.push_opacity(opacity);

            let child_rect = Rect {
                x: rect.x,
                y: rect.y + i as f32 * child_height,
                width: rect.width,
                height: child_height,
            };
            child.render(renderer, child_rect);

            renderer.pop_opacity();
        }
    }
}

/// VölvaScan - A container that renders "runic noise" before revealing its content.
/// Named after the Völva (seers) who saw through the veil of time.
#[derive(Clone)]
pub struct VölvaScan<V: View> {
    pub content: V,
    pub duration: f32,
}

impl<V: View> VölvaScan<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            duration: 1.5,
        }
    }
}

impl<V: View> View for VölvaScan<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();

        if t < self.duration {
            // Render Runic Noise
            let mut noise = String::new();
            let char_count = (rect.width * rect.height / 200.0) as usize;
            for i in 0..char_count {
                let rune_idx = ((t * 50.0 + i as f32) as usize) % RUNES.len();
                noise.push(RUNES[rune_idx]);
                if i % 10 == 0 {
                    noise.push('\n');
                }
            }
            renderer.draw_text(
                &noise,
                rect.x,
                rect.y + 10.0,
                10.0,
                theme::with_alpha(theme::accent(), 0.4),
            );
        } else {
            // Reveal Content
            let opacity = ((t - self.duration) * 2.0).min(1.0);
            renderer.push_opacity(opacity);
            self.content.render(renderer, rect);
            renderer.pop_opacity();
        }
    }
}

/// RunicTooltip - A contextual tooltip for providing hidden wisdom (information).
/// Named after the Runes, which encode secret knowledge.
#[doc(alias = "Tooltip")]
#[derive(Clone)]
pub struct RunicTooltip<V: View> {
    pub content: V,
    pub text: String,
    pub is_visible: bool,
}

impl<V: View> RunicTooltip<V> {
    /// Creates a new RunicTooltip wrapping the given content.
    pub fn new(content: V, text: impl Into<String>) -> Self {
        Self {
            content,
            text: text.into(),
            is_visible: false,
        }
    }

    /// Sets whether the tooltip is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }
}

impl<V: View> View for RunicTooltip<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn layout(&self) -> Option<&dyn cvkg_core::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        self.content.intrinsic_size(renderer, proposal)
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RunicTooltip");

        // 1. Render Base Content
        self.content.render(renderer, rect);

        // 2. Render Tooltip if visible — rendered below the content so it
        //    stays within the gallery preview rect and is never clipped.
        if self.is_visible {
            let font_size = 12.0;
            let (tw, _th) = renderer.measure_text(&self.text, font_size);
            let tip_h = font_size + 10.0; // reliable fixed height — measure_text height is unreliable
            let tip_rect = Rect {
                x: rect.x + (rect.width - (tw + 16.0)) / 2.0,
                // Place below the "Hover target" label, inside the preview rect
                y: rect.y + 26.0,
                width: tw + 16.0,
                height: tip_h,
            };

            renderer.set_z_index(200.0);
            // Solid dark background
            renderer.fill_rounded_rect(tip_rect, RADIUS_SM, [0.10, 0.05, 0.15, 0.97]);
            renderer.stroke_rounded_rect(tip_rect, RADIUS_SM, [0.70, 0.50, 0.90, 0.70], 1.0);

            // Vertically center text using font_size, not th (th is unreliable)
            renderer.draw_text(
                &self.text,
                tip_rect.x + 8.0,
                tip_rect.y + (tip_h - font_size) / 2.0,
                font_size,
                [1.0, 1.0, 1.0, 1.0],
            );
            renderer.set_z_index(0.0);
        }

        renderer.pop_vnode();
    }
}

impl<V: View> cvkg_core::LayoutView for RunicTooltip<V> {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::SizeProposal,
        _subviews: &[&dyn cvkg_core::LayoutView],
        cache: &mut cvkg_core::LayoutCache,
    ) -> cvkg_core::Size {
        if let Some(layout) = self.content.layout() {
            layout.size_that_fits(proposal, &[], cache)
        } else {
            cvkg_core::Size { width: 0.0, height: 0.0 }
        }
    }

    fn place_subviews(
        &self,
        _bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) {}
}

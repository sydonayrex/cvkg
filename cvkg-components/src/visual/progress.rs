use crate::RADIUS_SM;
use crate::integration::{CompanionBundle, WorldSpaceConfig};
use crate::theme;
use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// SkollProgress indicator component.
///
/// # Examples
/// ```
/// use cvkg_components::{SkollProgress, ProgressVariant};
/// let progress = SkollProgress::new(0.75)
///     .variant(ProgressVariant::Linear);
/// ```
#[doc(alias = "Progress")]
#[derive(Clone)]
pub struct SkollProgress {
    pub value: f32,
    pub variant: ProgressVariant,
    pub height: f32,
    pub background: [f32; 4],
    pub fill: [f32; 4],
    pub border_radius: f32,
    pub animated: bool,
    /// VDOM companion bundle — ARIA progress semantics.
    pub companions: CompanionBundle,
    /// Optional 3D world-space placement.
    pub world: WorldSpaceConfig,
}

/// Visual variants for progress indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressVariant {
    #[default]
    Linear,
    Circular,
    Segmented,
}

impl SkollProgress {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            variant: ProgressVariant::Linear,
            height: 8.0,
            background: theme::surface(),
            fill: theme::accent(),
            border_radius: RADIUS_SM,
            animated: true,
            companions: CompanionBundle::focusable()
                .with_role("progressbar")
                .with_label(format!("Progress {}%", (value.clamp(0.0, 1.0) * 100.0) as u32)),
            world: WorldSpaceConfig::default(),
        }
    }

    /// Opt this progress indicator into 3D world-space rendering.
    pub fn world(mut self, config: WorldSpaceConfig) -> Self {
        self.world = config;
        self
    }

    pub fn variant(mut self, v: ProgressVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn fill(mut self, color: [f32; 4]) -> Self {
        self.fill = color;
        self
    }
}

impl View for SkollProgress {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn companion_states(&self) -> Vec<Box<dyn cvkg_core::Companion>> {
        let mut bundle = self.companions.clone();
        let label = format!("Progress {}%", (self.value.clamp(0.0, 1.0) * 100.0) as u32);
        bundle = bundle.with_label(label);
        bundle.to_vec()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let world_id = {
            use std::hash::{Hash, Hasher};
            let mut s = std::collections::hash_map::DefaultHasher::new();
            "skoll_progress".hash(&mut s);
            (self.value as i32).hash(&mut s);
            s.finish()
        };

        // Push a dedicated VNode so companions + 3D world-space attach to the node.
        renderer.push_vnode_with_companions(rect, "Progress", self.companions.to_vec());

        self.world.begin(renderer, world_id);

        let bg_rect = rect;
        renderer.fill_rounded_rect(bg_rect, self.border_radius, self.background);

        let fill_width = rect.width * self.value;
        if fill_width > 0.0 {
            let fill_rect = Rect {
                x: rect.x,
                y: rect.y,
                width: fill_width,
                height: rect.height,
            };
            renderer.fill_rounded_rect(fill_rect, self.border_radius, self.fill);
        }

        if self.world.is_enabled() {
            renderer.end_world_space_panel(world_id);
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: 150.0,
            height: self.height,
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for SkollProgress {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 150.0,
            height: self.height,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// A horizontal status bar for system indicators.
#[derive(Clone)]
pub struct StatusBar {
    pub segments: Vec<StatusSegment>,
    pub height: f32,
}

#[derive(Clone)]
pub struct StatusSegment {
    pub label: String,
    pub value: f32,
    pub color: [f32; 4],
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            height: 24.0,
        }
    }

    pub fn segment(mut self, label: &str, value: f32, color: [f32; 4]) -> Self {
        self.segments.push(StatusSegment {
            label: label.to_string(),
            value,
            color,
        });
        self
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StatusBar {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!("Primitive view has no body")
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, theme::surface());
        let seg_width = rect.width / self.segments.len().max(1) as f32;
        for (i, seg) in self.segments.iter().enumerate() {
            let seg_rect = Rect {
                x: rect.x + i as f32 * seg_width,
                y: rect.y,
                width: seg_width,
                height: rect.height,
            };
            renderer.fill_rect(seg_rect, seg.color);
            let (tw, th) = renderer.measure_text(&seg.label, 10.0);
            renderer.draw_text_raw(
                &seg.label,
                seg_rect.x + (seg_width - tw) / 2.0,
                seg_rect.y + (rect.height - th) / 2.0,
                10.0,
                theme::text(),
            );
        }
    }
}

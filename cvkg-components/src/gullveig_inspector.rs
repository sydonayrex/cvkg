//! Gullveig Inspector - Component inspection and debugging
//!
//! Gullveig the Vanir witch was known for her prophetic powers and ability
//! to see into the nature of things - this inspector reveals component internals.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Component attribute for inspection
#[derive(Debug, Clone)]
pub struct ComponentAttr {
    pub name: String,
    pub value: String,
    pub editable: bool,
}

/// Gullveig Inspector for deep component analysis
pub struct GullveigInspector {
    pub(crate) component_name: String,
    pub(crate) attributes: Vec<ComponentAttr>,
    pub(crate) render_stats: RenderStats,
}

/// Rendering performance statistics
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub render_time_ms: f32,
    pub memory_bytes: usize,
    pub update_count: u32,
}

impl GullveigInspector {
    pub fn new(name: &str) -> Self {
        Self {
            component_name: name.to_string(),
            attributes: Vec::new(),
            render_stats: RenderStats::default(),
        }
    }

    pub fn attr(mut self, name: &str, value: &str, editable: bool) -> Self {
        self.attributes.push(ComponentAttr {
            name: name.to_string(),
            value: value.to_string(),
            editable,
        });
        self
    }

    pub fn stats(mut self, render_ms: f32, memory: usize, updates: u32) -> Self {
        self.render_stats = RenderStats {
            render_time_ms: render_ms,
            memory_bytes: memory,
            update_count: updates,
        };
        self
    }
}

impl View for GullveigInspector {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Header
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 26.0,
            },
            theme::inspector_bg(),
        );
        renderer.draw_text_raw(
            &format!("Inspecting: {}", self.component_name),
            rect.x + 8.0,
            rect.y + 8.0,
            12.0,
            theme::inspector_accent(),
        );

        // Stats section
        let stats_y = rect.y + 32.0;
        renderer.draw_text_raw(
            &format!(
                "Render: {:.1}ms | Mem: {}KB | Updates: {}",
                self.render_stats.render_time_ms,
                self.render_stats.memory_bytes / 1024,
                self.render_stats.update_count
            ),
            rect.x + 10.0,
            stats_y,
            10.0,
            theme::info(),
        );

        // Attributes
        let mut current_y = rect.y + 52.0;
        for attr in &self.attributes {
            let bg = if current_y as usize % 56 < 28 {
                theme::surface()
            } else {
                theme::inspector_bg()
            };
            renderer.fill_rect(
                Rect {
                    x: rect.x,
                    y: current_y,
                    width: rect.width,
                    height: 22.0,
                },
                bg,
            );

            renderer.draw_text_raw(
                &attr.name,
                rect.x + 8.0,
                current_y + 6.0,
                10.0,
                theme::text(),
            );
            renderer.draw_text_raw(
                &attr.value,
                rect.x + 100.0,
                current_y + 6.0,
                10.0,
                theme::inspector_accent(),
            );

            if attr.editable {
                renderer.draw_text_raw(
                    "✏",
                    rect.x + rect.width - 20.0,
                    current_y + 5.0,
                    10.0,
                    theme::inspector_warning(),
                );
            }
            current_y += 22.0;
        }
    }
}

impl LayoutView for GullveigInspector {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 280.0,
            height: 54.0 + self.attributes.len() as f32 * 22.0,
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

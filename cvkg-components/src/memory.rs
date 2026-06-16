use crate::theme;
use cvkg_core::{KnowledgeState, Never, Rect, Renderer, State, View};

/// MemoryView provides a tactical interface for inspecting an agent's KnowledgeState.
///
/// Section 2.3: "Prompt Integration... Only inject summaries/references."
pub struct MemoryView {
    /// The knowledge state to display
    pub state: State<KnowledgeState>,
}

impl MemoryView {
    /// Create a new MemoryView for the given state.
    pub fn new(state: State<KnowledgeState>) -> Self {
        Self { state }
    }
}

impl View for MemoryView {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let state = self.state.get();

        // Aesthetic: Sleipnir-inspired scrolling list placeholder
        // Title
        renderer.draw_text(
            "YGGDRASIL_RECOLLECTION",
            rect.x,
            rect.y + 15.0,
            16.0,
            theme::accent(),
        );

        let mut y_offset = rect.y + 40.0;

        if state.last_query_results.is_empty() {
            renderer.draw_text(
                "NO_ACTIVE_RECALL",
                rect.x + 10.0,
                y_offset,
                12.0,
                theme::text_muted(),
            );
        } else {
            for id in &state.last_query_results {
                if let Some(frag) = state.fragments.get(id) {
                    let item_rect = Rect {
                        x: rect.x,
                        y: y_offset,
                        width: rect.width,
                        height: 45.0,
                    };

                    // Fragment background
                    renderer.fill_rect(item_rect, theme::surface());
                    renderer.stroke_rect(item_rect, theme::with_alpha(theme::accent(), 0.4), 1.0);

                    // Summary
                    renderer.draw_text(
                        &frag.summary,
                        rect.x + 8.0,
                        y_offset + 18.0,
                        13.0,
                        theme::text(),
                    );

                    // Metadata
                    let meta = format!("REF: {} | ACCESS: {}", frag.source, frag.accessed_count);
                    renderer.draw_text(
                        &meta,
                        rect.x + 8.0,
                        y_offset + 35.0,
                        10.0,
                        theme::text_muted(),
                    );

                    y_offset += 55.0;

                    // Stop if we exceed the rect height
                    if y_offset > rect.y + rect.height {
                        break;
                    }
                }
            }
        }
    }
}

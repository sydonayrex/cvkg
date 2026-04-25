use cvkg_core::{ComponentErrorState, Never, Rect, Renderer, State, View};

/// ErrorBoundary provides fault isolation for a component subtree.
///
/// If the associated `ComponentErrorState` has `has_error` set to true,
/// the ErrorBoundary will suppress the normal UI and render a high-visibility
/// error placeholder instead.
pub struct ErrorBoundary<V: View> {
    pub error_state: State<ComponentErrorState>,
    pub content: V,
}

impl<V: View> ErrorBoundary<V> {
    /// Create a new ErrorBoundary wrapping the given content.
    pub fn new(error_state: State<ComponentErrorState>, content: V) -> Self {
        Self {
            error_state,
            content,
        }
    }
}

impl<V: View> View for ErrorBoundary<V> {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let err = self.error_state.get();
        if err.has_error {
            // Render Fallback UI - Muspelheim Aesthetic
            let msg = err.error_message.as_deref().unwrap_or("Unknown Error");
            let loc = err.error_location.as_deref().unwrap_or("unknown_loc");

            // Background - Deep void red
            renderer.fill_rect(rect, [0.08, 0.02, 0.02, 1.0]);

            // Hazard border
            renderer.stroke_rect(rect, [1.0, 0.2, 0.0, 1.0], 1.5);

            // Error Details
            renderer.draw_text(
                &format!("× FATAL: {}", msg),
                rect.x + 8.0,
                rect.y + 20.0,
                14.0,
                [1.0, 0.3, 0.2, 1.0],
            );

            renderer.draw_text(
                &format!("@ {}", loc),
                rect.x + 8.0,
                rect.y + 40.0,
                10.0,
                [0.6, 0.6, 0.7, 0.8],
            );
        } else {
            // Normal execution path
            self.content.render(renderer, rect);
        }
    }
}

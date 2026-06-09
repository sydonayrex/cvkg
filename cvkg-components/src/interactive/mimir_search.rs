//! MimirSearch — Standalone glass search bar.
//! Named after Mimir's well of wisdom.

use cvkg_core::{Rect, Renderer, View, Never};

pub struct MimirSearch {
    pub query: String,
    pub placeholder: String,
    pub style: SearchBarStyle,
    pub searching: bool,
}

pub enum SearchBarStyle {
    Compact,
    Expanded,
}

impl MimirSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            placeholder: "Search...".to_string(),
            style: SearchBarStyle::Compact,
            searching: false,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn style(mut self, style: SearchBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }
}

impl Default for MimirSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl View for MimirSearch {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Glass background (pill shape)
        let radius = match self.style {
            SearchBarStyle::Compact => rect.height / 2.0,
            SearchBarStyle::Expanded => 8.0,
        };

        renderer.bifrost(rect, 15.0, 1.0, 0.5);
        renderer.fill_rounded_rect(rect, radius, [0.1, 0.1, 0.12, 0.85]);

        // Search icon
        renderer.draw_text("\u{1F50D}", rect.x + 10.0, rect.y + 6.0, 14.0, [0.6, 0.6, 0.65, 0.8]);

        // Query text or placeholder
        let text = if self.query.is_empty() { &self.placeholder } else { &self.query };
        let color = if self.query.is_empty() { [0.5, 0.5, 0.55, 0.6] } else { [0.9, 0.9, 0.92, 1.0] };
        renderer.draw_text(text, rect.x + 32.0, rect.y + 7.0, 13.0, color);

        // Clear button (if query is non-empty)
        if !self.query.is_empty() {
            let clear_x = rect.x + rect.width - 24.0;
            renderer.draw_text("\u{2715}", clear_x, rect.y + 6.0, 12.0, [0.6, 0.6, 0.65, 0.8]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_new() {
        let search = MimirSearch::new();
        assert_eq!(search.query, "");
        assert_eq!(search.placeholder, "Search...");
        assert!(matches!(search.style, SearchBarStyle::Compact));
        assert!(!search.searching);
    }

    #[test]
    fn test_search_placeholder() {
        let search = MimirSearch::new().placeholder("Find files...");
        assert_eq!(search.placeholder, "Find files...");
    }

    #[test]
    fn test_search_style() {
        let search = MimirSearch::new().style(SearchBarStyle::Expanded);
        assert!(matches!(search.style, SearchBarStyle::Expanded));
    }

    #[test]
    fn test_search_query() {
        let search = MimirSearch::new().query("test query");
        assert_eq!(search.query, "test query");
    }

    #[test]
    fn test_search_default() {
        let search: MimirSearch = Default::default();
        assert_eq!(search.query, "");
    }
}
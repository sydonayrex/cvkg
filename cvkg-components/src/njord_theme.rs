//! Njord Theme - Theme engine for dynamic styling
//!
//! The Vanir god Njord governs prosperity and well-being - this theme engine
//! provides dynamic theming capabilities for the CVKG framework.

use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::collections::HashMap;

/// Design token for theme values
#[derive(Debug, Clone)]
pub struct DesignToken {
    pub name: String,
    pub value: String,
}

/// Theme variant definition
#[derive(Debug, Clone)]
pub struct ThemeVariant {
    pub name: String,
    pub colors: HashMap<String, [f32; 4]>,
}

/// Njord Theme Engine for dynamic theming
pub struct NjordTheme {
    pub(crate) tokens: Vec<DesignToken>,
    pub(crate) variants: Vec<ThemeVariant>,
    pub(crate) active_variant: String,
}

impl Default for NjordTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl NjordTheme {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            variants: Vec::new(),
            active_variant: "default".to_string(),
        }
    }

    /// Add a design token
    pub fn token(mut self, name: &str, value: &str) -> Self {
        self.tokens.push(DesignToken {
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }

    /// Add a theme variant
    pub fn variant(mut self, name: &str, colors: HashMap<String, [f32; 4]>) -> Self {
        self.variants.push(ThemeVariant {
            name: name.to_string(),
            colors,
        });
        self
    }

    /// Set active variant
    pub fn active(mut self, name: &str) -> Self {
        self.active_variant = name.to_string();
        self
    }

    /// Get color from active variant
    pub fn color(&self, name: &str) -> Option<[f32; 4]> {
        self.variants
            .iter()
            .find(|v| v.name == self.active_variant)
            .and_then(|v| v.colors.get(name).copied())
    }
}

impl View for NjordTheme {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.05, 0.05, 0.08, 1.0]);
        renderer.draw_text(
            "Njord Theme Engine",
            rect.x + 10.0,
            rect.y + 20.0,
            14.0,
            [0.8, 0.9, 1.0, 1.0],
        );

        let mut y = rect.y + 45.0;
        for token in &self.tokens {
            renderer.draw_text(
                &format!("{} = {}", token.name, token.value),
                rect.x + 15.0,
                y,
                11.0,
                [0.7, 0.8, 0.9, 1.0],
            );
            y += 20.0;
        }
    }
}

impl LayoutView for NjordTheme {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 240.0,
            height: 50.0 + self.tokens.len() as f32 * 20.0,
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

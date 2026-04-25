//! Theme engine and Norse-inspired skinning system for CVKG
//!
//! Provides the `Theme` instance and specific skin definitions built on top of Yggdrasil tokens.

use cvkg_core::YggdrasilTokens;

/// A resolved Theme instance, providing concrete values for the current mode
#[derive(Debug, Clone)]
pub struct Theme {
    tokens: YggdrasilTokens,
    is_dark: bool,
}

impl Theme {
    /// Create a new Theme instance from tokens and mode
    pub fn new(tokens: YggdrasilTokens, is_dark: bool) -> Self {
        Self { tokens, is_dark }
    }

    /// Create a theme with default Norse tokens
    pub fn default(is_dark: bool) -> Self {
        Self::new(cvkg_core::default_tokens(), is_dark)
    }

    /// Get a color value from the theme
    pub fn get_color(&self, key: &str) -> Option<String> {
        self.tokens.get_color(key, self.is_dark)
    }

    /// Access the underlying Yggdrasil tokens
    pub fn tokens(&self) -> &YggdrasilTokens {
        &self.tokens
    }

    /// Check if the theme is currently in dark mode
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }
}

//! Asset management types and design tokens.
//!
//! Extracted from lib.rs (P1-13).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::AssetManager;
use crate::EnvKey;

/// Key for accessing the asset manager in the environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetKey(pub String);

impl EnvKey for AssetKey {
    type Value = Arc<dyn AssetManager>;
    fn default_value() -> Self::Value {
        Arc::new(crate::DefaultAssetManager::new())
    }
}

/// Asset state for async resource loading.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AssetState<T> {
    Loading,
    Ready(T),
    Error(String),
}

/// Design token value that can adapt to light/dark mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// Single value (same for light and dark)
    Single { value: String },
    /// Different values for light and dark mode
    Adaptive { light: String, dark: String },
}

/// DesignTokens is the authoritative container for all design tokens in the CVKG ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokens {
    pub color: HashMap<String, TokenValue>,
    pub font: HashMap<String, TokenValue>,
    pub spacing: HashMap<String, TokenValue>,
    pub radius: HashMap<String, TokenValue>,
    pub shadow: HashMap<String, TokenValue>,
    pub border: HashMap<String, TokenValue>,
    pub anim: HashMap<String, TokenValue>,
    pub bifrost: HashMap<String, TokenValue>,
    pub gungnir: HashMap<String, TokenValue>,
    pub mjolnir: HashMap<String, TokenValue>,
    pub accessibility: HashMap<String, TokenValue>,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self::new()
    }
}

impl DesignTokens {
    pub fn new() -> Self {
        Self {
            color: HashMap::new(),
            font: HashMap::new(),
            spacing: HashMap::new(),
            radius: HashMap::new(),
            shadow: HashMap::new(),
            border: HashMap::new(),
            anim: HashMap::new(),
            bifrost: HashMap::new(),
            gungnir: HashMap::new(),
            mjolnir: HashMap::new(),
            accessibility: HashMap::new(),
        }
    }

    /// Get a color token value for the current mode
    pub fn get_color(&self, key: &str, is_dark: bool) -> Option<String> {
        self.color.get(key).map(|token| match token {
            TokenValue::Single { value } => value.clone(),
            TokenValue::Adaptive { light, dark } => {
                if is_dark {
                    dark.clone()
                } else {
                    light.clone()
                }
            }
        })
    }

    /// Get a token value of any type and parse it into the target type
    pub fn get<T: FromStr>(&self, category: &str, key: &str, is_dark: bool) -> Option<T> {
        let map = match category {
            "color" => &self.color,
            "font" => &self.font,
            "spacing" => &self.spacing,
            "radius" => &self.radius,
            "shadow" => &self.shadow,
            "border" => &self.border,
            "anim" => &self.anim,
            "bifrost" => &self.bifrost,
            "gungnir" => &self.gungnir,
            "mjolnir" => &self.mjolnir,
            "accessibility" => &self.accessibility,
            _ => return None,
        };

        map.get(key).and_then(|token| match token {
            TokenValue::Single { value } => value.parse().ok(),
            TokenValue::Adaptive { light, dark } => {
                let value = if is_dark { dark } else { light };
                value.parse().ok()
            }
        })
    }
}

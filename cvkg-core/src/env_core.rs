use crate::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Global environment storage using TypeId as keys.
pub static ENVIRONMENT: OnceLock<Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>> =
    OnceLock::new();

pub trait EnvKey: 'static + Send + Sync {
    /// The type of value stored in the environment
    type Value: Clone + Send + Sync + 'static;
    /// Get a default value for this key
    fn default_value() -> Self::Value;
}
/// Key for accessing the Yggdrasil design token tree
pub struct YggdrasilKey;
impl EnvKey for YggdrasilKey {
    type Value = DesignTokens;
    fn default_value() -> Self::Value {
        default_tokens()
    }
}
// Duplicate AssetKey removed - original definition at line 63

/// A color represented by RGBA components in the [0.0, 1.0] range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const VIKING_GOLD: Color = Color {
        r: 1.0,
        g: 0.84,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA_LIQUID: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TACTICAL_OBSIDIAN: Color = Color {
        r: 0.05,
        g: 0.05,
        b: 0.07,
        a: 1.0,
    };
    /// Calculate the relative luminance of the color as defined by WCAG 2.x
    pub fn relative_luminance(&self) -> f32 {
        fn res(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * res(self.r) + 0.7152 * res(self.g) + 0.0722 * res(self.b)
    }
    /// Calculate the contrast ratio between this color and another color
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        if l1 > l2 {
            (l1 + 0.05) / (l2 + 0.05)
        } else {
            (l2 + 0.05) / (l1 + 0.05)
        }
    }
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };

    /// Parse a HEX color string (e.g., "#FF6B35" or "FF6B35") into a Color.
    /// Returns None if the string is not a valid 6-digit HEX color.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        Some(Color { r, g, b, a: 1.0 })
    }
    /// Create a new color from RGBA components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    /// Convert the color to a [r, g, b, a] array.
    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Return a new color with lightness increased by `amount`.
    ///
    /// Adds `amount` to each RGB channel and clamps to [0.0, 1.0].
    /// This is a simple sRGB lightness adjustment, not perceptually uniform.
    /// For perceptually uniform adjustments, use OKLCH via cvkg-themes.
    pub fn lighten(&self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).clamp(0.0, 1.0),
            g: (self.g + amount).clamp(0.0, 1.0),
            b: (self.b + amount).clamp(0.0, 1.0),
            a: self.a,
        }
    }

    /// Return a new color with lightness decreased by `amount`.
    pub fn darken(&self, amount: f32) -> Self {
        Self {
            r: (self.r - amount).clamp(0.0, 1.0),
            g: (self.g - amount).clamp(0.0, 1.0),
            b: (self.b - amount).clamp(0.0, 1.0),
            a: self.a,
        }
    }
}
impl View for Color {
    type Body = Never;
    fn body(self) -> Self::Body {
        // SAFETY: `Never` is uninhabitable. Color is a primitive view that fills a
        // rectangle directly in `render()` and never exposes a composable body.
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, self.as_array());
    }
}
/// Key for accessing the current system appearance
pub struct AppearanceKey;
impl EnvKey for AppearanceKey {
    type Value = Appearance;
    fn default_value() -> Self::Value {
        Appearance::Dark // Default to Dark (Ginnungagap) for Berserker aesthetic
    }
}

/// Key for accessing the current text direction
pub struct DirectionKey;
impl EnvKey for DirectionKey {
    type Value = Direction;
    fn default_value() -> Self::Value {
        Direction::LTR
    }
}

/// StyleResolver provides high-level access to themed values from the environment.
pub struct StyleResolver;
impl StyleResolver {
    /// Resolve a color from the current environment
    pub fn color(key: &str) -> String {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens
            .get_color(key, is_dark)
            .unwrap_or_else(|| "#FF00FF".to_string()) // Default to MuspelMagenta on failure
    }
    /// Resolve a generic token value
    pub fn get<T: FromStr>(category: &str, key: &str) -> Option<T> {
        let tokens = Environment::<YggdrasilKey>::new().get();
        let appearance = Environment::<AppearanceKey>::new().get();
        let is_dark = appearance == Appearance::Dark;
        tokens.get(category, key, is_dark)
    }
    /// Resolve a color from the current environment as a [f32; 4] RGBA array.
    /// Returns the color value for the current appearance (light/dark).
    /// Falls back to magenta (#FF00FF) if the key is not found.
    pub fn color_array(key: &str) -> [f32; 4] {
        let hex = Self::color(key);
        parse_hex_color(&hex)
    }
}

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into [f32; 4] RGBA.
fn parse_hex_color(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        let a = if hex.len() >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        [r, g, b, a]
    } else {
        [1.0, 0.0, 1.0, 1.0] // Magenta fallback
    }
}

/// The authoritative Cyberpunk Viking default tokens
pub fn default_tokens() -> DesignTokens {
    let mut tokens = DesignTokens::new();
    // Core Norse Colorways
    tokens.color.insert(
        "background".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(), // Light mode: white background
            dark: "#05050A".to_string(),  // Dark mode: Ginnungagap (The Void, deeper flat black)
        },
    );
    tokens.color.insert(
        "primary".to_string(),
        TokenValue::Adaptive {
            light: "#007B8A".to_string(), // Light mode: muted cyan
            dark: "#E60012".to_string(),  // Dark mode: Berserker Red
        },
    );
    tokens.color.insert(
        "secondary".to_string(),
        TokenValue::Adaptive {
            light: "#8A008A".to_string(), // Light mode: muted magenta
            dark: "#FF8C00".to_string(),  // Dark mode: Ember Orange
        },
    );
    tokens.color.insert(
        "surface".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#0C0C12".to_string(),  // Darker flat blood-iron surface
        },
    );
    tokens.color.insert(
        "text".to_string(),
        TokenValue::Adaptive {
            light: "#000000".to_string(),
            dark: "#FFFFFF".to_string(),
        },
    );
    // Semantic component tokens
    tokens.color.insert(
        "surface_elevated".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#14141E".to_string(),
        },
    );
    tokens.color.insert(
        "surface_overlay".to_string(),
        TokenValue::Adaptive {
            light: "#FFFFFF".to_string(),
            dark: "#1A1A26".to_string(),
        },
    );
    tokens.color.insert(
        "border".to_string(),
        TokenValue::Adaptive {
            light: "#D0D0D8".to_string(),
            dark: "#3A1A1E".to_string(),  // Subtle red-grey border
        },
    );
    tokens.color.insert(
        "border_strong".to_string(),
        TokenValue::Adaptive {
            light: "#A0A0B0".to_string(),
            dark: "#5A252A".to_string(),
        },
    );
    tokens.color.insert(
        "text_muted".to_string(),
        TokenValue::Adaptive {
            light: "#606070".to_string(),
            dark: "#A08085".to_string(),
        },
    );
    tokens.color.insert(
        "text_dim".to_string(),
        TokenValue::Adaptive {
            light: "#9090A0".to_string(),
            dark: "#705055".to_string(),
        },
    );
    tokens.color.insert(
        "accent".to_string(),
        TokenValue::Adaptive {
            light: "#007B8A".to_string(), // Light mode: muted cyan
            dark: "#FF1E27".to_string(),  // Dark mode: Berserker Neon Red
        },
    );
    tokens.color.insert(
        "accent_hover".to_string(),
        TokenValue::Adaptive {
            light: "#00A0B0".to_string(), // Light mode: lighter muted cyan
            dark: "#FF5E66".to_string(),  // Dark mode: brighter red
        },
    );
    tokens.color.insert(
        "success".to_string(),
        TokenValue::Single {
            value: "#00E676".to_string(),
        },
    );
    tokens.color.insert(
        "warning".to_string(),
        TokenValue::Single {
            value: "#FFB300".to_string(),
        },
    );
    tokens.color.insert(
        "error".to_string(),
        TokenValue::Single {
            value: "#FF5252".to_string(),
        },
    );
    tokens.color.insert(
        "info".to_string(),
        TokenValue::Single {
            value: "#448AFF".to_string(),
        },
    );
    tokens.color.insert(
        "hover".to_string(),
        TokenValue::Adaptive {
            light: "#F0F0F5".to_string(),
            dark: "#251214".to_string(),
        },
    );
    tokens.color.insert(
        "active".to_string(),
        TokenValue::Adaptive {
            light: "#E0E0EB".to_string(),
            dark: "#3A1A1E".to_string(),
        },
    );
    tokens.color.insert(
        "disabled".to_string(),
        TokenValue::Adaptive {
            light: "#E8E8F0".to_string(),
            dark: "#1A1A28".to_string(),
        },
    );
    tokens.color.insert(
        "disabled_text".to_string(),
        TokenValue::Adaptive {
            light: "#B0B0C0".to_string(),
            dark: "#604045".to_string(),
        },
    );
    tokens.color.insert(
        "focus_ring".to_string(),
        TokenValue::Single {
            value: "#FF1E27".to_string(),
        },
    );
    tokens.color.insert(
        "shadow".to_string(),
        TokenValue::Adaptive {
            light: "#00000020".to_string(),
            dark: "#00000060".to_string(),
        },
    );
    tokens.color.insert(
        "code_bg".to_string(),
        TokenValue::Adaptive {
            light: "#F5F5FA".to_string(),
            dark: "#0D0D18".to_string(),
        },
    );
    // Bifrost (Glassmorphism) - Frosted Style
    tokens.bifrost.insert(
        "blur".to_string(),
        TokenValue::Single {
            value: "25.0".to_string(),
        },
    );
    tokens.bifrost.insert(
        "saturation".to_string(),
        TokenValue::Single {
            value: "1.2".to_string(),
        },
    );
    tokens.bifrost.insert(
        "opacity".to_string(),
        TokenValue::Single {
            value: "0.65".to_string(),
        },
    );
    // Gungnir (Neon Glow)
    tokens.gungnir.insert(
        "intensity".to_string(),
        TokenValue::Single {
            value: "0.25".to_string(), // Reduced glow intensity for legibility
        },
    );
    tokens.gungnir.insert(
        "radius".to_string(),
        TokenValue::Single {
            value: "6.0".to_string(), // Tighter glow radius for legibility
        },
    );
    // Mjolnir (Sharp Geometry)
    tokens.mjolnir.insert(
        "clip_angle".to_string(),
        TokenValue::Single {
            value: "12.0".to_string(),
        },
    );
    tokens.mjolnir.insert(
        "border_width".to_string(),
        TokenValue::Single {
            value: "2.0".to_string(),
        },
    );
    // Sleipnir (Spring Animation)
    tokens.anim.insert(
        "stiffness".to_string(),
        TokenValue::Single {
            value: "170.0".to_string(),
        },
    );
    tokens.anim.insert(
        "damping".to_string(),
        TokenValue::Single {
            value: "26.0".to_string(),
        },
    );
    tokens.anim.insert(
        "mass".to_string(),
        TokenValue::Single {
            value: "1.0".to_string(),
        },
    );
    // Accessibility
    tokens.accessibility.insert(
        "reduce_motion".to_string(),
        TokenValue::Single {
            value: "false".to_string(),
        },
    );
    tokens
}
/// Environment wrapper for accessing ambient values
pub struct Environment<K: EnvKey> {
    _marker: std::marker::PhantomData<K>,
}
impl<K: EnvKey> Default for Environment<K> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: EnvKey> Environment<K> {
    /// Create a new Environment
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
    /// Get the current value from the environment
    pub fn get(&self) -> K::Value {
        if let Some(env_store) = ENVIRONMENT.get() {
            let env_lock = env_store.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(val) = env_lock.get(&std::any::TypeId::of::<K>()) {
                if let Some(typed_val) = val.downcast_ref::<K::Value>() {
                    return typed_val.clone();
                } else {
                    log::warn!(
                        "Environment: Downcast failed for key type {:?}",
                        std::any::type_name::<K>()
                    );
                }
            } else {
                // Lowered to trace to avoid terminal logging overhead under standard debug runs
                log::trace!(
                    "Environment: Key not found: {:?}. Returning default.",
                    std::any::type_name::<K>()
                );
            }
        } else {
            // Lowered to trace to avoid terminal logging overhead under standard debug runs
            log::trace!(
                "Environment: Store not initialized. Key: {:?}. Returning default.",
                std::any::type_name::<K>()
            );
        }
        K::default_value()
    }
}

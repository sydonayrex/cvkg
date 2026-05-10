# cvkg-themes

**cvkg-themes** manages the "Berserker" design system, including semantic colors, typography scales, and motion tokens.

## What This Crate Does

- Provides `Theme` struct with semantic colors and design tokens
- Defines `SemanticColors` for UI states (primary, secondary, background, error, etc.)
- Provides `TypographyScale` for consistent text sizing
- Provides `SpacingScale` for layout consistency
- Provides `MotionScale` for standardized animation physics

## What This Crate Does NOT Do

- Does not provide rendering functionality
- Does not handle user preferences or system theme detection
- Does not persist theme settings

## Public API Overview

### Theme

```rust
/// Semantic colors for the Berserker Design System
pub struct SemanticColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
}

/// Typography scale for consistent rhythmic text
pub struct TypographyScale {
    pub hero: f32,
    pub h1: f32,
    pub h2: f32,
    pub body: f32,
    pub caption: f32,
    pub code: f32,
}

/// Spacing scale for layout consistency
pub struct SpacingScale {
    pub xs: f32,
    pub s: f32,
    pub m: f32,
    pub l: f32,
    pub xl: f32,
}

/// Motion scale for standardized animation physics
pub struct MotionScale {
    pub snappy: SleipnirParams,
    pub fluid: SleipnirParams,
    pub heavy: SleipnirParams,
    pub bouncy: SleipnirParams,
}

/// A resolved Theme instance
pub struct Theme {
    pub colors: SemanticColors,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub motion: MotionScale,
}
impl Theme {
    /// Create a dark theme with Norse tokens
    pub fn dark() -> Self;
    
    /// Check if theme is in dark mode
    pub fn is_dark(&self) -> bool;
    
    /// Validate accessibility against WCAG 2.1
    pub fn validate_accessibility(&self) -> Vec<String>;
}
```

### Color Utilities

```rust
use cvkg_core::Color;

// Color constants (defined in cvkg-core)
Color::VIKING_GOLD
Color::MAGENTA_LIQUID
Color::TACTICAL_OBSIDIAN
```

## Usage Example

```rust
use cvkg_themes::Theme;

let theme = Theme::dark();

println!("Primary color: {:?}", theme.colors.primary);
println!("Body text size: {}", theme.typography.body);
println!("Default spacing: {}", theme.spacing.m);

// Validate accessibility
let warnings = theme.validate_accessibility();
for warning in warnings {
    eprintln!("Accessibility warning: {}", warning);
}
```

## Known Limitations

- Only dark theme is provided by default
- Color contrast validation only checks primary/background, text/background, and text/surface combinations
- No runtime theme switching support in core
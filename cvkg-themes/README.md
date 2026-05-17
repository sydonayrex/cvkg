# cvkg-themes

![CVKG Component Showcase](../docs/images/cvkg_component_showcase.png)

`cvkg-themes` defines the authoritative "Berserker Design System" tokens, enabling consistent aesthetics, rhythmic typography, and accessible color palettes across the CVKG ecosystem.

## Boundaries and Responsibilities

This crate provides the data structures for design tokens. It does NOT apply styles directly (delegated to components). Its responsibilities include:
- Defining semantic color palettes (`Primary`, `Secondary`, `Accent`, `Surface`, `Background`).
- Establishing typography scales for hero headers, body text, and captions.
- Standardizing spacing and motion (animation physics) scales.
- Providing built-in themes like the default "Norse Dark" mode.
- Validating theme contrast against WCAG 2.1 accessibility standards.

## Public API Overview

### Theme Structure
- `Theme`: The root container for all design tokens.
- `SemanticColors`: Context-aware color definitions.
- `TypographyScale`: Rhythmic font sizes.
- `SpacingScale`: Standardized padding and margin values.
- `MotionScale`: Presets for Sleipnir animation solvers.

### Authoritative Tokens
- `Color::VIKING_GOLD`: The primary brand color.
- `Color::TACTICAL_OBSIDIAN`: The authoritative background for UI surfaces.
- `Color::MAGENTA_LIQUID`: The primary accent color for active states.

### Methods
- `Theme::dark()`: Returns the default high-fidelity dark theme.
- `Theme::validate_accessibility()`: Checks for contrast violations and returns a list of warnings.

## Usage Example

```rust
use cvkg_themes::Theme;

let theme = Theme::dark();
let bg_color = theme.colors.background;

// Check for accessibility compliance
for warning in theme.validate_accessibility() {
    eprintln!("A11y Warning: {}", warning);
}
```

## Known Limitations
- Dynamic runtime theme switching requires state management at the application level (typically via `EnvironmentValue`).
- Color validation is based on standard LTR contrast formulas; complex gradients require manual review.
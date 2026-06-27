# How to Generate a Theme

## Goal

Create a complete theme from a brand color using `cvkg-themes`.

## Prerequisites

- `cvkg-themes` in your project dependencies

## Steps

### 1. Generate from a Seed Color

```rust
use cvkg_themes::Theme;

let theme = Theme::from_seed(0x3B82F6); // Blue brand color
```

This generates a full theme with light/dark variants, semantic colors, typography scale, spacing, and radius tokens.

### 2. Build from a Brand Hex

```rust
use cvkg_themes::ThemeBuilder;

let theme = ThemeBuilder::from_brand_hex(0x3B82F6)
    .density(cvkg_themes::Density::Compact)
    .build();
```

### 3. Customize Specific Tokens

```rust
use cvkg_themes::{Theme, SpacingScale};

let mut theme = Theme::from_seed(0x3B82F6);
theme.spacing = SpacingScale {
    xs: 2.0,
    sm: 4.0,
    md: 8.0,
    lg: 16.0,
    xl: 32.0,
};
```

### 4. Validate Accessibility

```rust
use cvkg_themes::ApcaResult;

let result = theme.validate_contrast();
match result {
    ApcaResult::Pass => println!("All text meets APCA contrast requirements"),
    ApcaResult::Fail { element, .. } => println!("{} fails contrast", element),
}
```

## Expected Output

A `Theme` object with complete design tokens ready for use by components.

## What Can Go Wrong

- **Clamping**: OKLCH chroma is clamped to [0.0, ~0.4]. Highly saturated seed colors may produce slightly different hues.
- **Float equality**: Theme comparison uses float equality. Avoid comparing themes computed from different code paths.
- **Empty materials**: `ThemeBuilder::build()` with no materials produces a no-op theme. Add at least one material.

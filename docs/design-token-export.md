# Design Token Export

CVKG's theme engine can export design tokens to multiple formats for use in
design tools, CSS, and other frameworks.

## CLI Command

```bash
# Export to CSS custom properties (for Tailwind v4, plain CSS, etc.)
cvkg tokens --format css --output tokens.css

# Export to Figma Tokens JSON format
cvkg tokens --format figma --output tokens.json

# Export to plain JSON
cvkg tokens --format json --output tokens.json

# Export to Swift constants
cvkg tokens --format swift --output Theme.swift
```

## Supported Formats

### CSS (`--format css`)

Outputs CSS custom properties on `:root`:

```css
:root {
  --color-surface: rgb(250, 250, 250);
  --color-primary: rgb(51, 51, 64);
  --color-text: rgb(26, 26, 31);
  /* ... */
}
```

**Tailwind v4 compatibility**: Tailwind v4 consumes CSS custom properties directly
via its `@theme` directive. The output is ready to use with Tailwind v4's
`@theme { --color-x: ... }` convention.

### Figma (`--format figma`)

Outputs a JSON file compatible with Figma Tokens plugin:

```json
{
  "colors": {
    "background": {"r": 255, "g": 255, "b": 255, "a": 1.0},
    "primary": {"r": 51, "g": 51, "b": 64, "a": 1.0}
  }
}
```

### JSON (`--format json`)

Outputs a plain JSON representation of all theme tokens.

### Swift (`--format swift`)

Outputs a Swift file with color constants for iOS/macOS development.

## Theme Source

All exports are generated from the current `Theme::default()` which uses
OKLCH-based perceptually uniform colors. To export a custom theme, modify
the `Theme::from_seed()` call in `cvkg-cli/src/token_export.rs`.

## Files

- `cvkg-cli/src/token_export.rs` -- Token export engine (364 lines)
- `cvkg-cli/src/main.rs` -- CLI dispatch (see `Tokens` variant)

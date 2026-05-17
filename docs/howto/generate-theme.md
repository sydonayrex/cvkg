# How to Generate a Theme from Design Tokens

Goal: Generate a type-safe Rust styling theme module from a JSON design tokens specification file.

## Prerequisites
- Rust compiler and Cargo setup active in the CVKG workspace.
- A JSON file containing valid design color token entries (RGBA float arrays).

---

## Steps

### 1. Create a Token JSON File
Draft a `tokens.json` file containing your custom palette coordinates:
```json
{
  "viking_gold": [0.85, 0.65, 0.12, 1.0],
  "tactical_obsidian": [0.03, 0.03, 0.05, 1.0],
  "bifrost_teal": [0.0, 0.8, 0.8, 0.6]
}
```

### 2. Generate the Rust Source Theme
Run the CVKG theme compiler, directing the inputs and compiling the output theme file:
```bash
cargo run -p cvkg-cli -- theme --input ./tokens.json --output ./src/theme.rs
```

### 3. Integrate in Your View Tree
Import the compiled Rust output in your view file and apply it using environment variables:
```rust
mod theme;
use cvkg::prelude::*;

fn render_hud() -> impl View {
    VStack::new(10.0) {
        Text::new("Status System")
            .foregroundColor(theme::CUSTOM_THEME.viking_gold);
    }
    .background(theme::CUSTOM_THEME.tactical_obsidian)
}
```

---

## Expected Output
The generator writes the Rust file `src/theme.rs` containing a struct with your defined token fields and a static constant:
```rust
/// Generated CVKG Theme
pub struct Theme {
    pub viking_gold: [f32; 4],
    pub tactical_obsidian: [f32; 4],
    pub bifrost_teal: [f32; 4],
}

pub const CUSTOM_THEME: Theme = Theme {
    viking_gold: [0.85, 0.65, 0.12, 1.00],
    tactical_obsidian: [0.03, 0.03, 0.05, 1.00],
    bifrost_teal: [0.00, 0.80, 0.80, 0.60],
};
```

---

## Recovery and Debugging

### Compilation Fails on Invalid JSON Structure
If the parsing engine reports that the tokens format is corrupt:
1. Verify that all values are structured precisely as an array of four numeric values.
2. Confirm there are no trailing commas in the JSON keys.
3. Validate syntax using JSON parsing linters:
   ```bash
   python3 -m json.tool tokens.json
   ```

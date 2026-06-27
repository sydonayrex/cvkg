# Settings Page Recipe

A scrollable settings panel with grouped controls.

```rust
use cvkg::prelude::*;
use cvkg_layout::{ScrollView, VStack, HStack};
use cvkg_components::{Card, Text, Switch, Divider};

fn settings_page() -> impl View {
    ScrollView {
        VStack
            .gap(24.0)
            .padding(24.0)
            .children([
                Text::new("Settings")
                    .font_size(28.0)
                    .font_weight(FontWeight::Bold),

                // Account Section
                Text::new("Account")
                    .font_size(12.0)
                    .color(Color::GRAY)
                    .uppercase(true),
                Card::new(
                    VStack {
                        HStack {
                            Text::new("Email")
                            Text::new("user@example.com")
                                .color(Color::GRAY)
                        }
                        .justify(JustifyContent::SpaceBetween)
                        Divider::new()
                        HStack {
                            Text::new("Notifications")
                            Switch::new(true)
                        }
                        .justify(JustifyContent::SpaceBetween)
                    }
                    .gap(16.0)
                    .padding(16.0),
                ),

                // Appearance Section
                Text::new("Appearance")
                    .font_size(12.0)
                    .color(Color::GRAY)
                    .uppercase(true),
                Card::new(
                    HStack {
                        Text::new("Dark Mode")
                        Switch::new(false)
                    }
                    .justify(JustifyContent::SpaceBetween)
                    .gap(16.0)
                    .padding(16.0),
                ),
            ])
    }
}
```

**When to use:** Preferences, account management, configuration screens.
**Note:** Group related settings under section headers with dividers between rows.

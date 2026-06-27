# Master-Detail View Recipe

A classic pattern: a list on the left, detail view on the right.

```rust
use cvkg::prelude::*;
use cvkg_layout::{HStack, VStack};

#[derive(Clone)]
struct Item {
    title: String,
    description: String,
}

fn master_detail_view(items: Vec<Item>) -> impl View {
    HStack {
        // Master (left panel)
        VStack
            .flex(1.0)
            .gap(8.0)
            .padding(16.0)
            .children(
                items.iter().map(|item| {
                    Card::new(
                        Text::new(&item.title)
                            .font_size(14.0)
                            .color(Color::BLACK),
                    )
                    .padding(12.0)
                }),
            )

        // Detail (right panel)
        VStack
            .flex(2.0)
            .gap(8.0)
            .padding(16.0)
            .children([
                Text::new("Select an item")
                    .font_size(18.0)
                    .color(Color::GRAY),
            ]),
    }
    .gap(1.0)
    .background(Color::WHITE)
}
```

**When to use:** File managers, email clients, settings panels.
**Minimum width:** 600px for comfortable side-by-side layout.

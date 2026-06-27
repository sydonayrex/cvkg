# Async Loading Pattern Recipe

Display content after an async operation with loading and error states.

```rust
use cvkg::prelude::*;
use cvkg_layout::VStack;
use cvkg_components::{Text, Spinner, Button};
use cvkg_core::{Binding, BindingExt, Suspense};

#[derive(Clone)]
enum LoadState<T> {
    Loading,
    Ready(T),
    Error(String),
}

async fn fetch_user_data(user_id: u64) -> Result<String, String> {
    // Simulate async work
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(format!("User #{} data loaded", user_id))
}

fn async_content_view(user_id: u64) -> impl View {
    let result = Binding::new(LoadState::Loading);

    // Spawn async work
    let result_clone = result.clone();
    cvkg_core::spawn(async move {
        match fetch_user_data(user_id).await {
            Ok(data) => result_clone.set(LoadState::Ready(data)),
            Err(e) => result_clone.set(LoadState::Error(e)),
        }
    });

    VStack
        .gap(16.0)
        .children([match result.get() {
            LoadState::Loading => Spinner::new()
                .size(32.0)
                .center(true),
            LoadState::Ready(data) => Text::new(&data)
                .font_size(16.0),
            LoadState::Error(msg) => VStack
                .gap(8.0)
                .children([
                    Text::new("Failed to load")
                        .color(Color::RED),
                    Text::new(msg)
                        .font_size(12.0)
                        .color(Color::GRAY),
                    Button::new("Retry", || {
                        // Retry logic
                    })
                    .variant(ButtonVariant::Secondary),
                ]),
        }])
}
```

**When to use:** Data fetching, file loading, any async I/O.
**Note:** Always show a loading indicator so the user knows work is in progress.

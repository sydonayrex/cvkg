# Login with Validation Recipe

A login form with email/password validation and error display.

```rust
use cvkg::prelude::*;
use cvkg_layout::{VStack, HStack};
use cvkg_components::{Card, Text, TextInput, Button, FormField};
use cvkg_core::{Binding, BindingExt};

#[derive(Clone)]
struct LoginForm {
    email: Binding<String>,
    password: Binding<String>,
    error: Binding<Option<String>>,
}

impl LoginForm {
    fn new() -> Self {
        Self {
            email: Binding::new(String::new()),
            password: Binding::new(String::new()),
            error: Binding::new(None),
        }
    }

    fn validate(&self) -> bool {
        let email = self.email.get();
        let password = self.password.get();

        if !email.contains('@') {
            self.error.set(Some("Invalid email format".to_string()));
            return false;
        }
        if password.len() < 8 {
            self.error.set(Some("Password must be 8+ characters".to_string()));
            return false;
        }
        true
    }
}

fn login_view(form: &LoginForm) -> impl View {
    Card::new(
        VStack
            .gap(16.0)
            .padding(24.0)
            .children([
                Text::new("Sign In")
                    .font_size(24.0)
                    .font_weight(FontWeight::Bold),

                // Error display
                if form.error.get().is_some() {
                    Text::new(form.error.get().unwrap())
                        .color(Color::RED)
                        .font_size(12.0)
                },

                FormField::new("Email")
                    .text_input(form.email.clone())
                    .placeholder("you@example.com"),

                FormField::new("Password")
                    .text_input(form.password.clone())
                    .placeholder("Enter password")
                    .secure(true),

                Button::new("Sign In", || {
                    if form.validate() {
                        println!("Login successful");
                    }
                })
                .variant(ButtonVariant::Primary)
                .fill(true),
            ]),
    )
    .max_width(400.0)
}
```

**When to use:** Authentication screens, sign-up flows.
**Note:** Show inline errors below or beside the invalid field, not in a modal.

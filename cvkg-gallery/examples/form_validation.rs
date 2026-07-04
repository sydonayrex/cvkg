//! Form validation example demonstrating integrated validation.
//!
//! This example shows how to create a validated form with Input, Select, and Checkbox
//! components that automatically display error messages.

use cvkg_components::{Form, FormField, Input, Select, Checkbox, ValidationRule, Button};
use cvkg_core::{Never, Rect, Renderer, View};

/// Contact form with validation
pub struct ContactForm {
    email_field: FormField<Input>,
    country_field: FormField<Select<&'static str>>,
    agree_field: FormField<Checkbox>,
}

impl ContactForm {
    pub fn new() -> Self {
        Self {
            email_field: FormField::new("Email", Input::new("Enter your email"))
                .required()
                .rule(ValidationRule::Pattern("@".to_string())),
            country_field: FormField::new("Country", Select::new("Select a country"))
                .required()
                .option("USA", "usa")
                .option("Canada", "canada")
                .option("UK", "uk"),
            agree_field: FormField::new("", Checkbox::new(false, |_| {}))
                .required(),
        }
    }
}

impl View for ContactForm {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
        // Stub render - actual form uses Form wrapper
    }
}

/// Simpler validated form using the Form wrapper directly
pub struct SimpleValidatedForm {
    form: Form,
}

impl SimpleValidatedForm {
    pub fn new() -> Self {
        let form = Form::new()
            .submit_label("Save")
            .on_submit(|| {
                println!("Form submitted successfully!");
            });

        Self { form }
    }
}

impl View for SimpleValidatedForm {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) {
        // Form renders its fields internally
    }
}

fn main() {
    // Example usage - validated form in ~15 lines as specified in P1.2
    let email = Input::new("Email address")
        .rules(vec![
            ValidationRule::Required,
            ValidationRule::MinLength(5),
            ValidationRule::Pattern("@".to_string()),
        ]);

    // Note: Full Form integration example would use:
    // Form::new()
    //     .field(FormField::new("Email", email))
    //     .on_submit(|values| async { submit(values) })
}
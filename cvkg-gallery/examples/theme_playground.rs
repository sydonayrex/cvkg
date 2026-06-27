use cvkg::prelude::*;
use cvkg_components::{Badge, BadgeVariant, ButtonVariant};
/// A simple theme playground that demonstrates Theme::from_seed().
/// Cycles through preset seed colors to show how the theme changes.
struct ThemePlayground {
    selected_seed: usize,
}

const SEED_COLORS: &[(&str, [f32; 4])] = &[
    ("Cyan", [0.0, 0.8, 1.0, 1.0]),
    ("Magenta", [1.0, 0.0, 0.8, 1.0]),
    ("Orange", [1.0, 0.5, 0.0, 1.0]),
    ("Green", [0.0, 0.8, 0.2, 1.0]),
    ("Purple", [0.5, 0.0, 1.0, 1.0]),
    ("Red", [1.0, 0.1, 0.1, 1.0]),
];

impl ThemePlayground {
    fn new() -> Self {
        Self { selected_seed: 0 }
    }
}

impl View for ThemePlayground {
    type Body = VStack;

    fn body(self) -> Self::Body {
        let (name, color) = SEED_COLORS[self.selected_seed];

        VStack::new(16.0)
            .child(
                Text::new(format!("Theme Playground -- Seed: {}", name))
                    .font_size(20.0)
                    .color(color),
            )
            .child(
                Text::new(
                    "Select a seed color to see how Theme::from_seed() generates a complete theme.",
                )
                .font_size(12.0)
                .color([0.6, 0.6, 0.7, 1.0]),
            )
            .child(
                HStack::new(8.0)
                    .child(Button::new("Cyan", || {}).disabled(self.selected_seed == 0))
                    .child(Button::new("Magenta", || {}).disabled(self.selected_seed == 1))
                    .child(Button::new("Orange", || {}).disabled(self.selected_seed == 2))
                    .child(Button::new("Green", || {}).disabled(self.selected_seed == 3))
                    .child(Button::new("Purple", || {}).disabled(self.selected_seed == 4))
                    .child(Button::new("Red", || {}).disabled(self.selected_seed == 5)),
            )
            .child(
                HStack::new(12.0)
                    .child(Button::new("Primary Action", || {}))
                    .child(Button::new("Secondary", || {}).variant(ButtonVariant::Secondary))
                    .child(Button::new("Ghost", || {}).variant(ButtonVariant::Ghost)),
            )
            .child(
                VStack::new(8.0)
                    .child(
                        Text::new("Form Controls")
                            .font_size(14.0)
                            .color([0.7, 0.7, 0.8, 1.0]),
                    )
                    .child(Input::new("Sample input"))
                    .child(Checkbox::new(true, |_| {}))
                    .child(Slider::new(0.5, 0.0..=1.0, |_v| {})),
            )
            .child(
                HStack::new(8.0)
                    .child(Progress::new(0.7))
                    .child(Spinner::new().size(16.0)),
            )
            .child(
                HStack::new(4.0)
                    .child(Badge::new("Default"))
                    .child(Badge::new("Info").variant(BadgeVariant::Secondary))
                    .child(Badge::new("Outline").variant(BadgeVariant::Outline)),
            )
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(ThemePlayground::new(), None);
}

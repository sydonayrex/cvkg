use cvkg::prelude::*;

struct Dashboard;

impl View for Dashboard {
    type Body = VStack;

    fn body(self) -> Self::Body {
        VStack::new(16.0)
            .child(
                Text::new("Dashboard")
                    .font_size(28.0)
                    .color([1.0, 1.0, 1.0, 1.0]),
            )
            .child(
                HStack::new(12.0)
                    .child(Button::new("Refresh", || {}))
                    .child(Button::new("Export", || {})),
            )
            .child(Progress::new(0.7))
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(Dashboard, None);
}

use cvkg::prelude::*;

struct HoverDemo;

impl View for HoverDemo {
    type Body = VStack;

    fn body(self) -> Self::Body {
        VStack::new(16.0)
            .child(
                Text::new("Hover over the button below")
                    .font_size(14.0)
                    .color([0.7, 0.7, 0.7, 1.0]),
            )
            .child(
                Button::new("Hover me", || {})
                    .on_hover(TriggerSpring::hover_scale()),
            )
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(HoverDemo, None);
}

use cvkg::prelude::*;

#[view_component]
fn App() -> impl View {
    VStack::new(10.0)
        .child(Text::new("Hello Cyber Viking"))
        .child(Button::new("Click Me", || println!("Clicked!")))
        .padding(20.0)
        .background([0.05, 0.05, 0.1, 1.0])
}

fn main() {
    cvkg::native::NativeRenderer::run(App());
}

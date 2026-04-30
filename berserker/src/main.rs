use cvkg::prelude::*;

#[derive(View)]
struct App;

impl App {
    fn body(&self) -> impl View {
        VStack::new()
            .push(Text::new("Hello Cyber Viking"))
            .push(Button::new("Click Me", || println!("Clicked!")))
            .padding(20.0)
            .background(Color::rgb(0.05, 0.05, 0.1))
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(App);
}

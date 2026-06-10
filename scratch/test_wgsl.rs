fn main() {
    let source = include_str!("../cvkg-render-gpu/src/shaders/effects.wgsl");
    let mut frontend = naga::front::wgsl::Frontend::new();
    match frontend.parse(source) {
        Ok(_) => println!("WGSL parsed successfully!"),
        Err(e) => {
            println!("WGSL parsing failed:");
            println!("{}", e.emit_to_string(source));
        }
    }
}

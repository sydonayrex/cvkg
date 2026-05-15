use accesskit::Action;

fn main() {
    // This is just to trigger a compilation error that lists variants if it fails,
    // or I can try to use it.
    let _ = Action::Click;
}

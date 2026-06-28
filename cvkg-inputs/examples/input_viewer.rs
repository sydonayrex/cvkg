//! Example: input viewer.
//!
//! Prints live gamepad, keyboard, and mouse state to the terminal.
//! Run with: cargo run -p cvkg-inputs --example input_viewer --features gilrs

use cvkg_inputs::backend::{NoopBackend, InputBackend};
#[cfg(feature = "gilrs")]
use cvkg_inputs::backend::GilrsBackend;
use cvkg_inputs::{InputSystem, InputState};

fn main() {
    let mut system = InputSystem::new();

    #[cfg(feature = "gilrs")]
    {
        match GilrsBackend::new() {
            Ok(backend) => {
                println!("Initialized gilrs backend");
                system.add_backend(Box::new(backend));
            }
            Err(e) => {
                eprintln!("Failed to init gilrs: {e}");
                system.add_backend(Box::new(NoopBackend::new()));
            }
        }
    }

    #[cfg(not(feature = "gilrs"))]
    {
        println!("gilrs feature disabled, using noop backend");
        system.add_backend(Box::new(NoopBackend::new()));
    }

    println!("Polling input system (Ctrl+C to exit)...");

    loop {
        match system.poll() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Poll error: {e}");
                break;
            }
        }

        let state = system.state();
        if let Ok(guard) = state.read() {
            print_state(&guard);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn print_state(state: &cvkg_inputs::InputState) {
    if !state.gamepads.is_empty() {
        println!("Gamepads: {}", state.gamepads.len());
        for (id, gp) in &state.gamepads {
            println!("  [{:?}] connected={}", id, gp.connected);
        }
    }
}

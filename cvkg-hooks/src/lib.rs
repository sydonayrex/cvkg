//! Lightweight lifecycle hooks.

type Callback = Box<dyn FnMut()>;

/// An ordered set of callbacks invoked by `run`.
#[derive(Default)]
pub struct Hook {
    callbacks: Vec<Callback>,
}

impl Hook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe<F: FnMut() + 'static>(&mut self, callback: F) {
        self.callbacks.push(Box::new(callback));
    }

    pub fn run(&mut self) {
        for callback in &mut self.callbacks {
            callback();
        }
    }
}

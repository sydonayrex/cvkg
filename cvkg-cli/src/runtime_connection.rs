//! Runtime Connection Layer
//! Handles communication between CLI and running app

use tokio::sync::mpsc;

/// Runtime message types
#[derive(Debug, Clone)]
pub enum RuntimeMessage {
    Patch(super::patch_engine::RuntimePatch),
    State(super::dev_runtime::RuntimeStateSnapshot),
    Event(super::dev_runtime::RuntimeEvent),
}

/// Runtime connection handles communication between CLI and running app
pub struct RuntimeConnection {
    sender: mpsc::Sender<RuntimeMessage>,
}

impl RuntimeConnection {
    /// Create a new RuntimeConnection
    pub fn new() -> (Self, mpsc::Receiver<RuntimeMessage>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { sender }, receiver)
    }

    /// Send a message to the runtime
    pub async fn send(&self, msg: RuntimeMessage) {
        let _ = self.sender.send(msg).await;
    }
}

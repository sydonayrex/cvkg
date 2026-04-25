//! Agent Replay Module
//! Load and replays agent traces for debugging

use serde_json;
use std::fs;

use crate::dev_runtime::AgentEvent;

/// Load an agent trace from a JSON file
pub fn load_agent_trace(path: &str) -> Vec<AgentEvent> {
    let data =
        fs::read_to_string(path).expect(&format!("Failed to read agent trace file: {}", path));

    serde_json::from_str(&data).expect(&format!("Failed to parse agent trace JSON from: {}", path))
}

/// Replay an agent trace by sending events to the runtime
pub fn replay_agent_trace<F>(events: Vec<AgentEvent>, mut inject_event: F)
where
    F: FnMut(AgentEvent),
{
    for event in events {
        inject_event(event);
        // In a real implementation, we might want to delay between events
        // to simulate real-time behavior
    }
}

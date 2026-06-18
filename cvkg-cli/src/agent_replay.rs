//! Agent Replay Module
//! Load and replays agent traces for debugging

use crate::dev_runtime::AgentEvent;

/// Load an agent trace from a JSON file.
///
/// # Errors
/// Returns an error if the file cannot be read or the JSON is invalid.
pub fn load_agent_trace(path: &str) -> anyhow::Result<Vec<AgentEvent>> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read agent trace file '{}': {}", path, e))?;
    let events = serde_json::from_str(&data)
        .map_err(|e| anyhow::anyhow!("Failed to parse agent trace JSON from '{}': {}", path, e))?;
    Ok(events)
}

/// Replay an agent trace by sending events to the runtime.
///
/// # Arguments
/// * `events` -- The agent events to replay.
/// * `inject_event` -- Callback invoked for each event.
pub fn replay_agent_trace<F>(events: Vec<AgentEvent>, mut inject_event: F)
where
    F: FnMut(AgentEvent),
{
    for event in events {
        inject_event(event);
    }
}

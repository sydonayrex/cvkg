//! # CVKG Multi-Agent Conflict Resolution Subsystem
//!
//! This module implements Priority 3.1 of the CVKG roadmap.
//! It provides identity and priority tracking for UI state mutations,
//! enabling predictable resolution when multiple AI agents contend for the same state.

use serde::{Serialize, Deserialize};

/// Unique identifier for an AI agent or system component issuing state mutations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

impl AgentId {
    /// Reserved ID for the core framework.
    pub const SYSTEM: Self = Self(0);
    /// Reserved ID for direct user input.
    pub const USER: Self = Self(1);
    /// Starting ID for dynamic agents.
    pub const AGENT_START: Self = Self(100);
}

/// Priority level for conflict resolution. Higher values take precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentPriority(pub u32);

impl AgentPriority {
    pub const LOW: Self = Self(0);
    pub const NORMAL: Self = Self(100);
    pub const HIGH: Self = Self(1000);
    pub const CRITICAL: Self = Self(u32::MAX);
}

/// Metadata describing the context of a state mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationMetadata {
    pub agent_id: AgentId,
    pub priority: AgentPriority,
    pub timestamp_ms: u64,
}

/// Strategies for resolving concurrent writes to the same piece of state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictResolution {
    /// The most recent write always wins, regardless of priority.
    #[default]
    LastWriterWins,
    /// Writes with higher priority overwrite lower priority ones.
    /// If priorities are equal, the last writer wins.
    PriorityWins,
    /// Indicates that a semantic merge should be attempted (requires custom implementation).
    Merge,
}

thread_local! {
    static CURRENT_AGENT: std::cell::RefCell<Option<MutationMetadata>> = std::cell::RefCell::new(None);
}

/// Executes a closure within the context of a specific agent.
/// Any State mutations performed within this block will be attributed to this agent.
pub fn with_agent<F, R>(agent_id: AgentId, priority: AgentPriority, f: F) -> R
where
    F: FnOnce() -> R,
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    let meta = MutationMetadata {
        agent_id,
        priority,
        timestamp_ms: now,
    };
    
    let prev = CURRENT_AGENT.with(|a| a.replace(Some(meta)));
    let result = f();
    CURRENT_AGENT.with(|a| a.replace(prev));
    result
}

/// A transactional unit of work issued by an AI agent.
pub struct AgentTransaction<F> {
    pub agent_id: AgentId,
    pub priority: AgentPriority,
    pub mutation: F,
}

impl<F, R> AgentTransaction<F>
where
    F: FnOnce() -> R,
{
    pub fn new(agent_id: AgentId, priority: AgentPriority, mutation: F) -> Self {
        Self { agent_id, priority, mutation }
    }

    pub fn execute(self) -> R {
        with_agent(self.agent_id, self.priority, self.mutation)
    }
}

/// Description of a conflict between two agents.
#[derive(Debug, Clone)]
pub struct ConflictEvent {
    pub agent_id: AgentId,
    pub priority: AgentPriority,
    pub existing_agent_id: AgentId,
    pub existing_priority: AgentPriority,
    pub timestamp_ms: u64,
}

static CONFLICT_HANDLERS: once_cell::sync::Lazy<std::sync::Arc<std::sync::Mutex<Vec<Box<dyn Fn(ConflictEvent) + Send + Sync>>>>> =
    once_cell::sync::Lazy::new(|| std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));

/// Register a global handler for agentic conflicts.
pub fn on_conflict<F>(handler: F)
where
    F: Fn(ConflictEvent) + Send + Sync + 'static,
{
    CONFLICT_HANDLERS.lock().unwrap().push(Box::new(handler));
}

pub(crate) fn notify_conflict(event: ConflictEvent) {
    let handlers = CONFLICT_HANDLERS.lock().unwrap();
    for handler in handlers.iter() {
        handler(event.clone());
    }
}

/// A stable API surface for AI agents to interact with the CVKG framework.
pub trait AgentSurface: Send + Sync {
    /// Retrieve the unique identifier of the agent using this surface.
    fn agent_id(&self) -> AgentId;

    /// Query the current value of a piece of state.
    fn query<T: Clone + Send + Sync + 'static>(&self, state: &crate::State<T>) -> T;

    /// Update a piece of state with a specific priority.
    fn update<T: Clone + Send + Sync + 'static>(
        &self,
        state: &crate::State<T>,
        value: T,
        priority: AgentPriority,
    );

    /// Execute a set of mutations atomically under this agent's identity.
    fn transact<R, F>(&self, priority: AgentPriority, f: F) -> R
    where
        F: FnOnce() -> R;
}

/// Standard implementation of the AgentSurface.
pub struct DefaultAgentSurface {
    id: AgentId,
}

impl DefaultAgentSurface {
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

impl AgentSurface for DefaultAgentSurface {
    fn agent_id(&self) -> AgentId {
        self.id
    }

    fn query<T: Clone + Send + Sync + 'static>(&self, state: &crate::State<T>) -> T {
        state.get()
    }

    fn update<T: Clone + Send + Sync + 'static>(
        &self,
        state: &crate::State<T>,
        value: T,
        priority: AgentPriority,
    ) {
        with_agent(self.id, priority, || {
            state.set(value);
        });
    }

    fn transact<R, F>(&self, priority: AgentPriority, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        with_agent(self.id, priority, f)
    }
}

/// Internal helper to retrieve the current agent context.
pub(crate) fn get_current_mutation_metadata() -> Option<MutationMetadata> {
    CURRENT_AGENT.with(|a| *a.borrow())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::State;

    #[test]
    fn test_conflict_resolution() {
        let state = State::new(0).with_resolution(ConflictResolution::PriorityWins);
        let surface_a = DefaultAgentSurface::new(AgentId(101));
        let surface_b = DefaultAgentSurface::new(AgentId(102));

        // Agent A sets with NORMAL priority
        surface_a.update(&state, 10, AgentPriority::NORMAL);
        assert_eq!(state.get(), 10);

        // Agent B tries to set with LOW priority - should be ignored
        surface_b.update(&state, 20, AgentPriority::LOW);
        assert_eq!(state.get(), 10); // Still 10

        // Agent B sets with HIGH priority - should win
        surface_b.update(&state, 30, AgentPriority::HIGH);
        assert_eq!(state.get(), 30);
    }
}

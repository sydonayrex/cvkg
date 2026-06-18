//! Knowledge state for the agentic memory system.
//!
//! Extracted from lib.rs (P1-13).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::Notification;
use crate::UndoManager;

/// Knowledge state for the agentic memory system.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeState {
    pub thoughts: Vec<String>,
    pub actions: Vec<String>,
    pub context: HashMap<String, String>,
    pub last_query_results: Vec<KnowledgeId>,
    #[serde(alias = "items")]
    pub fragments: std::collections::HashMap<KnowledgeId, KnowledgeFragment>,
    /// The Temporal Graph nodes
    pub nodes: Vec<TemporalNode>,
    /// The Temporal Graph edges
    pub edges: Vec<TemporalEdge>,
    /// The current operational Realm (Midgard/Asgard)
    pub realm: Realm,
    /// Last known pointer position (X, Y)
    pub last_pointer_pos: [f32; 2],
    /// Resolved pointer velocity (pixels per frame)
    pub pointer_velocity: [f32; 2],
    /// The current 'Focus' node ID (Odin's Eye focus)
    pub odin_focus: Option<String>,
    /// Agent attention heatmap (node_id -> intensity)
    pub agent_attention: HashMap<String, f32>,
    // Component state storage for dynamic state
    #[serde(skip)]
    pub component_states: HashMap<u64, Arc<std::sync::RwLock<dyn std::any::Any + Send + Sync>>>,
    /// Global undo/redo manager tracking document and input states.
    #[serde(skip)]
    pub undo_manager: UndoManager,
    /// Active notification list.
    #[serde(default)]
    pub notifications: Vec<Notification>,
    /// Flag indicating whether the notification center panel is visible.
    #[serde(default)]
    pub notification_center_visible: bool,
    /// Modifier key state: shift key pressed.
    #[serde(default)]
    pub modifiers_shift: bool,
    /// Modifier key state: control key pressed.
    #[serde(default)]
    pub modifiers_ctrl: bool,
    /// Modifier key state: alt/option key pressed.
    #[serde(default)]
    pub modifiers_alt: bool,
    /// Modifier key state: logo/command/windows key pressed.
    #[serde(default)]
    pub modifiers_logo: bool,
    /// Whether the performance profiling overlay (Cmd+Shift+P) is currently visible.
    #[serde(default)]
    pub performance_overlay_visible: bool,
}

impl KnowledgeState {
    /// Apply activation decay to all temporal nodes and evolving components.
    /// Nodes with weight below a threshold drift out of the primary context.
    /// Components lose vitality (Fafnir's Decay) if not actively 'fed'.
    pub fn apply_decay(&mut self, decay_factor: f32) {
        for node in &mut self.nodes {
            node.weight *= decay_factor;
        }

        // Fafnir's Decay: Components naturally revert to base state over time
        for state in self.component_states.values() {
            if let Ok(mut lock) = state.write()
                && let Some(v) = lock.downcast_mut::<f32>()
            {
                *v = (*v * decay_factor).max(1.0);
            }
        }
    }

    /// Increase the importance weight of nodes associated with a successful task.
    pub fn reinforce(&mut self, node_ids: &[String], boost: f32) {
        for node in &mut self.nodes {
            if node_ids.contains(&node.id) {
                node.weight += boost;
            }
        }
    }

    /// Update pointer kinematics based on a new position.
    pub fn update_pointer(&mut self, new_pos: [f32; 2]) {
        self.pointer_velocity = [
            new_pos[0] - self.last_pointer_pos[0],
            new_pos[1] - self.last_pointer_pos[1],
        ];
        self.last_pointer_pos = new_pos;
    }
}
// Knowledge System Types
/// Unique identifier for knowledge fragments
pub type KnowledgeId = String;

/// A knowledge fragment stored in the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFragment {
    /// Unique identifier for this fragment
    pub id: String,
    /// Short summary for prompt injection and quick search
    pub summary: String,
    /// Reference source (e.g. filename, URL, or conversation ID)
    pub source: String,
    /// Frame number or timestamp of creation
    pub created_at: u64,
    /// Number of times this fragment has been retrieved
    pub accessed_count: u32,
    /// Full content (optional, can be loaded on-demand)
    pub content: Option<String>,
}

impl KnowledgeFragment {
    pub fn new(id: String, summary: String, source: String) -> Self {
        Self {
            id,
            summary,
            source,
            created_at: 0,
            accessed_count: 0,
            content: None,
        }
    }
}

/// Memory layers for the layered cognitive engine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryLayer {
    /// Raw mission events (short-term)
    Episodic,
    /// Extracted facts and tactical intelligence (long-term)
    Semantic,
    /// Successful command sequences and tool chains
    Procedural,
}

/// The operational Realm of the UI.
/// Midgard: Classic, functional, 2D tactical UI for mortals.
/// Asgard: High-fidelity, cognitive, shader-heavy UI for the Singularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Realm {
    Midgard,
    #[default]
    Asgard,
}

/// Priority for screen reader announcements via `Renderer::announce`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPriority {
    /// Wait for current speech to finish before announcing.
    Polite,
    /// Interrupt current speech to announce immediately.
    Assertive,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalNode {
    /// Unique identifier for this node
    pub id: String,
    /// ID of the underlying knowledge fragment
    pub fragment_id: KnowledgeId,
    /// Timestamp of the event
    pub timestamp: u64,
    /// The memory layer this node belongs to
    pub layer: MemoryLayer,
    /// Importance weight for activation decay and retrieval
    pub weight: f32,
}

/// An edge in the Temporal Graph representing a relationship between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of relationship (e.g. "causal", "semantic", "temporal")
    pub relation: String,
    /// Weight/strength of the connection
    pub weight: f32,
}

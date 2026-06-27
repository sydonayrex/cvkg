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
pub struct AppState {
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
    /// The current operational UiFidelityLevel (Midgard/Asgard)
    pub realm: UiFidelityLevel,
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
    pub component_states: HashMap<u64, Arc<dyn std::any::Any + Send + Sync>>,
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

impl AppState {
    /// Apply activation decay to all temporal nodes and evolving components.
    /// Nodes with weight below a threshold drift out of the primary context.
    /// Components lose vitality (Fafnir's Decay) if not actively 'fed'.
    /// Apply activation decay to all temporal nodes and evolving components.
    /// Nodes with weight below a threshold drift out of the primary context.
    /// Components lose vitality (Fafnir's Decay) if not actively 'fed'.
    pub fn apply_decay(&mut self, decay_factor: f32) {
        for node in &mut self.nodes {
            node.weight *= decay_factor;
        }

        // Fafnir's Decay: Components naturally revert to base state over time
        for state in self.component_states.values() {
            if let Ok(rw_arc) = state.clone().downcast::<std::sync::RwLock<f32>>()
                && let Ok(mut lock) = rw_arc.write()
            {
                *lock = (*lock * decay_factor).max(1.0);
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

    /// Get a reference to a component's internal state by ID.
    /// Uses `Arc::downcast` for safe type recovery without unsafe pointer casts.
    pub fn get_component_state<T: 'static + Send + Sync>(
        &self,
        id: u64,
    ) -> Option<Arc<std::sync::RwLock<T>>> {
        let stored = self.component_states.get(&id)?;
        let any_arc: &Arc<dyn std::any::Any + Send + Sync> = stored;
        let rw_arc: Arc<dyn std::any::Any + Send + Sync> = any_arc.clone();
        rw_arc.downcast::<std::sync::RwLock<T>>().ok()
    }

    /// Set component state by ID (for hooks system).
    pub fn set_component_state<T: 'static + Send + Sync>(&mut self, id: u64, value: T) {
        let rw: Arc<std::sync::RwLock<T>> = Arc::new(std::sync::RwLock::new(value));
        let any_arc: Arc<dyn std::any::Any + Send + Sync> = rw;
        self.component_states.insert(id, any_arc);
    }

    /// Captures a snapshot of the current state for debugging and hot-reloading.
    pub fn snapshot(&self) -> Vec<crate::runtime::NodeStateSnapshot> {
        let mut snapshots = Vec::new();
        for frag in self.fragments.values() {
            if let Ok(val) = serde_json::to_value(frag) {
                snapshots.push(crate::runtime::NodeStateSnapshot { id: 0, state: val });
            }
        }
        snapshots
    }

    /// Add a new fragment to memory.
    pub fn remember(&mut self, fragment: KnowledgeFragment) {
        self.fragments.insert(fragment.id.clone(), fragment);
    }

    /// Process a search query against the local knowledge base.
    pub fn process_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f32, String)> = self
            .fragments
            .iter()
            .filter(|(_, frag)| {
                frag.summary.to_lowercase().contains(&query_lower)
                    || frag
                        .content
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&query_lower))
            })
            .map(|(id, frag)| {
                let score = frag.accessed_count as f32 + frag.created_at as f32 * 0.001;
                (score, id.clone())
            })
            .collect();
        results.sort_by(|a, b| b.0.total_cmp(&a.0));
        self.last_query_results = results.into_iter().map(|(_, id)| id).take(5).collect();
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

/// The operational UiFidelityLevel of the UI.
/// Midgard: Classic, functional, 2D tactical UI for mortals.
/// Asgard: High-fidelity, cognitive, shader-heavy UI for the Singularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum UiFidelityLevel {
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

// ── Global Application State ──────────────────────────────────────────────
// System-wide reactive state for agentic UI features.
// Extracted from core/uniforms.rs during modularization.

use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// Global application state registry.
pub static SYSTEM_STATE: OnceLock<Arc<arc_swap::ArcSwap<AppState>>> = OnceLock::new();

/// Flag indicating whether the renderer is currently in the rendering phase.
#[allow(dead_code)]
static IS_BATCHING: AtomicBool = AtomicBool::new(false);

/// Flag set whenever system state is mutated, triggering a layout recalculation.
pub static LAYOUT_DIRTY: AtomicBool = AtomicBool::new(false);
static RENDERING_FLAG: AtomicBool = AtomicBool::new(false);

/// Check if the renderer is currently in its rendering phase.
pub fn is_rendering() -> bool {
    RENDERING_FLAG.load(Ordering::Relaxed)
}

/// Mark the start of the rendering phase.
pub fn set_rendering(rendering: bool) {
    RENDERING_FLAG.store(rendering, Ordering::Relaxed);
}

/// Get a reference to the global system state.
pub fn get_system_state() -> Arc<arc_swap::ArcSwap<AppState>> {
    SYSTEM_STATE
        .get_or_init(|| Arc::new(arc_swap::ArcSwap::from_pointee(AppState::default())))
        .clone()
}

/// Signals the start of the render phase. Mutations during this phase trigger warnings.
pub fn begin_render_phase() {
    set_rendering(true);
}

/// Signals the end of the render phase.
pub fn end_render_phase() {
    set_rendering(false);
}

/// Load the current system state snapshot.
pub fn load_system_state() -> arc_swap::Guard<Arc<AppState>> {
    get_system_state().load()
}

/// Snapshot the current system state (returns a cheap clone of the Arc).
pub fn snapshot_system_state() -> AppState {
    let guard = load_system_state();
    (**guard).clone()
}

/// Update the system state via a mutation closure.
pub fn update_system_state<F>(f: F)
where
    F: FnOnce(&AppState) -> AppState,
{
    use std::sync::Mutex;
    static STATE_WRITE_MUTEX: Mutex<()> = Mutex::new(());

    let _lock = STATE_WRITE_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    if is_rendering() {
        log::warn!(
            "LAYOUT THRASH DETECTED: System state mutated during render phase. This may trigger redundant layout passes and impact performance."
        );
    }
    LAYOUT_DIRTY.store(true, Ordering::SeqCst);
    let swap = get_system_state();
    let current = swap.load();
    let new_state = Arc::new(f(&current));
    drop(current);
    swap.store(new_state);
}

// ── Fallback Runtime ───────────────────────────────────────────────────────
// Extracted from core/uniforms.rs

#[cfg(not(target_arch = "wasm32"))]
static FALLBACK_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static FALLBACK_WORKER_COUNT: OnceLock<usize> = OnceLock::new();

/// Get the fallback tokio runtime for spawning background tasks.
/// Used by suspense, async data loading, and off-main-thread computation.
#[cfg(not(target_arch = "wasm32"))]
pub fn fallback_runtime() -> &'static tokio::runtime::Runtime {
    FALLBACK_RUNTIME.get_or_init(|| {
        let worker_count = *FALLBACK_WORKER_COUNT.get_or_init(|| {
            let available = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(2);
            available.saturating_sub(1).clamp(1, 8)
        });
        tokio::runtime::Builder::new_current_thread()
            .worker_threads(worker_count)
            .thread_name("cvkg-fallback-rt")
            .enable_all()
            .build()
            .expect("failed to build fallback tokio runtime")
    })
}

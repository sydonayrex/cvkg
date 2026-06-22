pub mod types;
pub mod engine;
pub mod visual;

pub use types::*;
pub use engine::*;

use std::collections::HashMap;
use std::time::Instant;

use crate::theme;
use cvkg_core::{
    Renderer, Size, View, Never,
    layout::{LayoutCache, LayoutView, Rect, SizeProposal},
};

// ═══════════════════════════════════════════════════════════════════════════
// Orchestrator State
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct OrchestratorState {
    pub nodes: Vec<OrchestratorNode>,
    pub edges: Vec<OrchestratorEdge>,
    pub current_run: Option<OrchestratorRun>,
    pub run_history: Vec<OrchestratorRun>,
    pub selected_node: Option<String>,
    pub inspected_node: Option<String>,
    pub show_log_panel: bool,
    pub show_metrics_panel: bool,
    pub show_template_library: bool,
    pub show_run_comparison: bool,
    pub templates: Vec<WorkflowTemplate>,
    pub viewport_offset: (f32, f32),
    pub viewport_zoom: f32,
    pub is_executing: bool,
    pub execution_speed: f32,
    pub log_level_filter: Option<LogLevel>,
    pub log_search: String,
    pub run_counter: u64,
    pub active_run_id: Option<String>,
    pub skill_registry: SkillRegistry,
    pub undo_redo: UndoRedo,
    pub validation_result: ValidationResult,
    pub search: SearchState,
    pub recurring_runs: Vec<RecurringRun>,
    pub is_paused: bool,
    pub step_mode: bool,
    pub message_log: Vec<AgentMessage>,
    pub validation_errors: Vec<ValidationError>,
    pub show_output_panel: bool,
    pub show_message_panel: bool,
    pub show_validation_panel: bool,
    pub show_skills_panel: bool,
    pub show_webhook_panel: bool,
    pub show_schedule_panel: bool,
    pub show_recurring_panel: bool,
    pub show_minimap: bool,
}

impl OrchestratorState {
    pub fn demo() -> Self {
        let nodes = vec![
            OrchestratorNode::webhook("trigger", "Webhook Trigger").at(50.0, 100.0),
            OrchestratorNode::agent("agent_1", "Research Agent")
                .at(300.0, 50.0)
                .with_agent_config(AgentConfig {
                    model: "gpt-4".to_string(),
                    system_prompt: "You are a research assistant.".to_string(),
                    prompt_template: "Research: {{input}}".to_string(),
                    ..Default::default()
                }),
            OrchestratorNode::agent("agent_2", "Analysis Agent")
                .at(300.0, 200.0)
                .with_agent_config(AgentConfig {
                    model: "claude-3".to_string(),
                    system_prompt: "You are an analyst.".to_string(),
                    prompt_template: "Analyze: {{input}}".to_string(),
                    ..Default::default()
                }),
            OrchestratorNode::conditional("branch_1", "Quality Check").at(550.0, 120.0),
            OrchestratorNode::agent("agent_3", "Writer Agent")
                .at(800.0, 50.0)
                .with_agent_config(AgentConfig {
                    model: "gpt-4".to_string(),
                    system_prompt: "You are a writer.".to_string(),
                    prompt_template: "Write: {{input}}".to_string(),
                    ..Default::default()
                }),
            OrchestratorNode::loop_node("loop_1", "Refine Loop").at(800.0, 200.0),
            OrchestratorNode::parallel("parallel_1", "Fan Out").at(1050.0, 120.0),
            OrchestratorNode::output_sink("output_1", "Save Output").at(1300.0, 120.0),
        ];

        let edges = vec![
            OrchestratorEdge::new("e1", "trigger", "out", "agent_1", "in"),
            OrchestratorEdge::new("e2", "trigger", "out", "agent_2", "in"),
            OrchestratorEdge::new("e3", "agent_1", "out", "branch_1", "in"),
            OrchestratorEdge::new("e4", "agent_2", "out", "branch_1", "in"),
            OrchestratorEdge::new("e5", "branch_1", "true", "agent_3", "in")
                .with_condition("quality >= 0.8"),
            OrchestratorEdge::new("e6", "branch_1", "false", "loop_1", "in")
                .with_condition("quality < 0.8"),
            OrchestratorEdge::new("e7", "loop_1", "done", "agent_3", "in"),
            OrchestratorEdge::new("e8", "agent_3", "out", "parallel_1", "in"),
            OrchestratorEdge::new("e9", "parallel_1", "branch_1", "output_1", "in"),
        ];

        Self {
            nodes,
            edges,
            current_run: None,
            run_history: Vec::new(),
            selected_node: None,
            inspected_node: None,
            show_log_panel: true,
            show_metrics_panel: false,
            show_template_library: false,
            show_run_comparison: false,
            templates: Vec::new(),
            viewport_offset: (0.0, 0.0),
            viewport_zoom: 1.0,
            is_executing: false,
            execution_speed: 1.0,
            log_level_filter: None,
            log_search: String::new(),
            run_counter: 0,
            active_run_id: None,
            skill_registry: SkillRegistry::new(),
            undo_redo: UndoRedo::default(),
            validation_result: ValidationResult::default(),
            search: SearchState::default(),
            recurring_runs: vec![],
            show_recurring_panel: false,
            is_paused: false,
            step_mode: false,
            message_log: vec![],
            validation_errors: vec![],
            show_output_panel: false,
            show_message_panel: false,
            show_validation_panel: false,
            show_skills_panel: false,
            show_webhook_panel: false,
            show_schedule_panel: false,
            show_minimap: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Multi-Agent Orchestrator Component
// ═══════════════════════════════════════════════════════════════════════════

/// The main orchestrator component -- a `View` that renders the workflow graph,
/// execution logs, metrics panel, and controls.
#[derive(Debug, Clone)]
pub struct MultiAgentOrchestrator {
    /// The orchestrator state.
    pub state: OrchestratorState,
    /// The execution engine.
    pub engine: ExecutionEngine,
    /// Unique ID for this instance (used for state hashing).
    pub instance_id: u64,
    /// Last tick time for simulation.
    pub last_tick: Option<Instant>,
    /// Whether the user is dragging a node.
    pub dragging: Option<String>,
    /// Drag offset (pointer position minus node origin).
    pub drag_offset: (f32, f32),
    /// Pending edge creation (source node + port).
    pub pending_edge: Option<(String, String)>,
    /// Pointer position in graph space.
    pub pointer_pos: (f32, f32),
}

impl Default for MultiAgentOrchestrator {
    fn default() -> Self {
        Self {
            state: OrchestratorState::demo(),
            engine: ExecutionEngine::default(),
            instance_id: 0xDEAD_BEEF,
            last_tick: None,
            dragging: None,
            drag_offset: (0.0, 0.0),
            pending_edge: None,
            pointer_pos: (0.0, 0.0),
        }
    }
}

impl MultiAgentOrchestrator {
    /// Create a new orchestrator with the demo workflow.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start execution of the workflow.
    pub fn start_execution(&mut self) {
        if self.state.is_executing {
            return;
        }
        self.state.is_executing = true;
        self.state.current_run = Some(OrchestratorRun {
            id: format!("run-{}", self.engine.total_steps()),
            started_at: Instant::now(),
            ended_at: None,
            status: RunStatus::Running,
            node_states: HashMap::new(),
            total_usage: TokenUsage::default(),
            total_cost: 0.0,
            duration: None,
            logs: Vec::new(),
            metadata: HashMap::new(),
        });
        self.engine.plan_execution(&self.state);
        self.last_tick = Some(Instant::now());
    }

    /// Stop execution.
    pub fn stop_execution(&mut self) {
        self.state.is_executing = false;
        self.engine.reset();
        if let Some(run) = &mut self.state.current_run {
            run.status = RunStatus::Cancelled;
            run.ended_at = Some(Instant::now());
        }
    }

    /// Advance the simulation by one tick.
    pub fn simulation_tick(&mut self) {
        if !self.state.is_executing {
            return;
        }

        if let Some(step) = self.engine.tick() {
            self.apply_step(&step);
        }

        if self.engine.is_complete() {
            self.state.is_executing = false;
            if let Some(run) = &mut self.state.current_run {
                run.status = RunStatus::Completed;
                run.ended_at = Some(Instant::now());
                self.state.run_history.push(run.clone());
            }
        }
    }

    /// Apply a single execution step to the state.
    fn apply_step(&mut self, step: &ExecutionStep) {
        let run = match &mut self.state.current_run {
            Some(r) => r,
            None => return,
        };

        match step {
            ExecutionStep::StartNode(node_id) => {
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.status = NodeExecutionStatus::Running;
                node_state.started_at = Some(Instant::now());
            }
            ExecutionStep::CompleteNode(node_id, output) => {
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.status = NodeExecutionStatus::Completed;
                node_state.output = Some(output.clone());
                node_state.ended_at = Some(Instant::now());
            }
            ExecutionStep::FailNode(node_id, error) => {
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.status = NodeExecutionStatus::Failed;
                node_state.error = Some(error.clone());
                node_state.ended_at = Some(Instant::now());
            }
            ExecutionStep::RetryNode(node_id, attempt) => {
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.status = NodeExecutionStatus::Retrying(*attempt);
                node_state.retry_count = *attempt;
            }
            ExecutionStep::SkipNode(node_id) => {
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.status = NodeExecutionStatus::Skipped;
            }
            ExecutionStep::Log(node_id, level, message) => {
                let log_entry = OrchestratorLog {
                    timestamp: Instant::now(),
                    node_id: node_id.clone(),
                    level: level.clone(),
                    message: message.clone(),
                    data: None,
                };
                run.logs.push(log_entry.clone());
                let node_state = run
                    .node_states
                    .entry(node_id.clone())
                    .or_insert_with(NodeExecutionState::default);
                node_state.logs.push(log_entry);
            }
            ExecutionStep::ActivateEdge(edge_id) => {
                for edge in &mut self.state.edges {
                    if edge.id == *edge_id {
                        edge.is_active = true;
                        break;
                    }
                }
            }
            ExecutionStep::DeactivateEdge(edge_id) => {
                for edge in &mut self.state.edges {
                    if edge.id == *edge_id {
                        edge.is_active = false;
                        break;
                    }
                }
            }
            ExecutionStep::Wait(_) => {
                // Wait steps are handled by the simulation tick timing.
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// View Implementation -- Rendering
// ═══════════════════════════════════════════════════════════════════════════

impl View for MultiAgentOrchestrator {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // ── Background ──────────────────────────────────────────────────
        renderer.fill_rect(rect, theme::surface());

        // ── Layout: graph canvas (left) + side panels (right) ──────────
        let log_panel_w = if self.state.show_log_panel {
            320.0
        } else {
            0.0
        };
        let metrics_panel_w = if self.state.show_metrics_panel {
            260.0
        } else {
            0.0
        };
        let graph_w = rect.width - log_panel_w - metrics_panel_w;

        let graph_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: graph_w.max(100.0),
            height: rect.height,
        };

        // ── Render graph canvas ─────────────────────────────────────────
        self.render_graph_canvas(renderer, graph_rect);

        // ── Render log panel ────────────────────────────────────────────
        if self.state.show_log_panel {
            let log_rect = Rect {
                x: rect.x + graph_w,
                y: rect.y,
                width: log_panel_w,
                height: rect.height,
            };
            self.render_log_panel(renderer, log_rect);
        }

        // ── Render metrics panel ────────────────────────────────────────
        if self.state.show_metrics_panel {
            let metrics_x = rect.x + graph_w + log_panel_w;
            let metrics_rect = Rect {
                x: metrics_x,
                y: rect.y,
                width: metrics_panel_w,
                height: rect.height,
            };
            self.render_metrics_panel(renderer, metrics_rect);
        }

        // ── Render template library overlay ──────────────────────────────
        if self.state.show_template_library {
            self.render_template_library(renderer, rect);
        }

        // ── Render run comparison overlay ────────────────────────────────
        if self.state.show_run_comparison {
            self.render_run_comparison(renderer, rect);
        }

        // ── Render restored overlay panels ───────────────────────────────
        let panel_width = 300.0;
        let mut floating_y = rect.y + 10.0;
        let floating_x = rect.x + 10.0;

        if self.state.show_output_panel {
            self.render_output_panel(renderer, floating_x, floating_y, panel_width, 150.0);
            floating_y += 160.0;
        }
        if self.state.show_message_panel {
            self.render_message_panel(renderer, floating_x, floating_y, panel_width, 200.0);
            floating_y += 210.0;
        }
        if self.state.show_validation_panel {
            self.render_validation_panel(renderer, floating_x, floating_y, panel_width, 150.0);
            floating_y += 160.0;
        }
        if self.state.show_skills_panel {
            self.render_skills_panel(renderer, floating_x, floating_y, panel_width, 200.0);
            floating_y += 210.0;
        }
        if self.state.show_webhook_panel {
            self.render_webhook_panel(renderer, floating_x, floating_y, panel_width, 100.0);
            floating_y += 110.0;
        }
        if self.state.show_schedule_panel {
            self.render_schedule_panel(renderer, floating_x, floating_y, panel_width, 100.0);
            floating_y += 110.0;
        }
        if self.state.show_recurring_panel {
            self.render_recurring_panel(renderer, floating_x, floating_y, panel_width, 150.0);
            floating_y += 160.0;
        }
        if self.state.show_minimap {
            let minimap_w = 200.0;
            let minimap_h = 150.0;
            self.render_minimap(
                renderer,
                rect.x + rect.width - minimap_w - 20.0,
                rect.y + rect.height - minimap_h - 20.0,
                minimap_w,
                minimap_h,
            );
        }

        let _ = floating_y;

        // ── Event handlers ──────────────────────────────────────────────
        self.register_event_handlers(renderer, rect);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Event Handler Registration
// ═══════════════════════════════════════════════════════════════════════════

impl MultiAgentOrchestrator {
    fn register_event_handlers(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let instance_id = self.instance_id;

        // ── Run/Stop button ─────────────────────────────────────────────
        let btn_x = rect.x + rect.width - 100.0;
        let btn_y = rect.y + 6.0;
        let btn_w = 88.0;
        let btn_h = 24.0;
        let btn_rect = Rect {
            x: btn_x,
            y: btn_y,
            width: btn_w,
            height: btn_h,
        };
        renderer.register_handler(
            "pointerdown:runstop",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerDown { x, y, .. } = event
                    && x >= btn_rect.x
                    && x <= btn_rect.x + btn_rect.width
                    && y >= btn_rect.y
                    && y <= btn_rect.y + btn_rect.height
                {
                    cvkg_core::update_system_state(move |s| {
                        let s = s.clone();
                        if let Some(guard) = s.get_component_state::<OrchestratorState>(instance_id)
                        {
                            let mut new_state =
                                guard.read().ok().map(|g| g.clone()).unwrap_or_default();
                            if new_state.is_executing {
                                new_state.is_executing = false;
                            } else {
                                new_state.is_executing = true;
                                new_state.run_counter += 1;
                                new_state.active_run_id =
                                    Some(format!("run-{}", new_state.run_counter));
                            }
                            *guard.write().unwrap_or_else(|e| e.into_inner()) = new_state;
                        }
                        s
                    });
                }
            }),
        );

        // ── Log panel toggle ────────────────────────────────────────────
        renderer.register_handler(
            "pointerdown:togglelog",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerDown { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let s = s.clone();
                        if let Some(guard) = s.get_component_state::<OrchestratorState>(instance_id)
                        {
                            let mut new_state =
                                guard.read().ok().map(|g| g.clone()).unwrap_or_default();
                            new_state.show_log_panel = !new_state.show_log_panel;
                            *guard.write().unwrap_or_else(|e| e.into_inner()) = new_state;
                        }
                        s
                    });
                }
            }),
        );

        // ── Metrics panel toggle ────────────────────────────────────────
        renderer.register_handler(
            "pointerdown:togglemetrics",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerDown { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let s = s.clone();
                        if let Some(guard) = s.get_component_state::<OrchestratorState>(instance_id)
                        {
                            let mut new_state =
                                guard.read().ok().map(|g| g.clone()).unwrap_or_default();
                            new_state.show_metrics_panel = !new_state.show_metrics_panel;
                            *guard.write().unwrap_or_else(|e| e.into_inner()) = new_state;
                        }
                        s
                    });
                }
            }),
        );
    }
}

impl LayoutView for MultiAgentOrchestrator {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 1200.0,
            height: 700.0,
        }
    }

    fn place_subviews(
        &self,
        _rect: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

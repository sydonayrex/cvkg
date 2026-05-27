//! Multi-Agent Orchestrator
//!
//! A full workflow orchestration engine with:
//! - Visual node graph editor (agent, conditional, loop, parallel, webhook, schedule nodes)
//! - Real-time execution with log streaming
//! - Node output inspection panel
//! - Cost/token tracking per node and aggregate
//! - Retry logic and timeout config per node
//! - Conditional branching and loop nodes
//! - Parallel execution visualization
//! - Template library and versioning
//! - Run comparison and agent metrics dashboard

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::theme;
use cvkg_core::{
    Renderer, Size, View,
    layout::{LayoutCache, LayoutView, Rect, SizeProposal},
};

// ═══════════════════════════════════════════════════════════════════════════
// Color helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Convert a `[u8; 4]` RGBA color to `[f32; 4]`.
fn c(color: [u8; 4]) -> [f32; 4] {
    [
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        color[3] as f32 / 255.0,
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// Node Types
// ═══════════════════════════════════════════════════════════════════════════

/// The kind of node in the workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrchestratorNodeType {
    Agent,
    Conditional,
    Loop,
    Parallel,
    Webhook,
    Schedule,
    DataSource,
    Sink,
}

impl OrchestratorNodeType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Agent => "Agent",
            Self::Conditional => "Conditional",
            Self::Loop => "Loop",
            Self::Parallel => "Parallel",
            Self::Webhook => "Webhook",
            Self::Schedule => "Schedule",
            Self::DataSource => "Data Source",
            Self::Sink => "Sink",
        }
    }

    pub fn color(&self) -> [f32; 4] {
        c(match self {
            Self::Agent => [0x4C, 0xC9, 0xF0, 0xFF],
            Self::Conditional => [0xFF, 0xAB, 0x40, 0xFF],
            Self::Loop => [0xAB, 0x47, 0xBC, 0xFF],
            Self::Parallel => [0x66, 0xBB, 0x6A, 0xFF],
            Self::Webhook => [0xEC, 0x40, 0x7A, 0xFF],
            Self::Schedule => [0xFF, 0xEE, 0x58, 0xFF],
            Self::DataSource => [0x78, 0x90, 0x9C, 0xFF],
            Self::Sink => [0x8D, 0x6E, 0x63, 0xFF],
        })
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Agent => "A",
            Self::Conditional => "?",
            Self::Loop => "L",
            Self::Parallel => "P",
            Self::Webhook => "W",
            Self::Schedule => "S",
            Self::DataSource => "D",
            Self::Sink => "O",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Execution Status
// ═══════════════════════════════════════════════════════════════════════════

/// The current execution status of a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying(u32),
    TimedOut,
    Cancelled,
}

impl NodeExecutionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Completed => "Done",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::Retrying(_) => "Retrying",
            Self::TimedOut => "Timeout",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn color(&self) -> [f32; 4] {
        c(match self {
            Self::Pending => [0x90, 0xA4, 0xAE, 0xFF],
            Self::Running => [0x42, 0xA5, 0xF5, 0xFF],
            Self::Completed => [0x4C, 0xAF, 0x50, 0xFF],
            Self::Failed => [0xF4, 0x43, 0x36, 0xFF],
            Self::Skipped => [0x78, 0x90, 0x9C, 0xFF],
            Self::Retrying(_) => [0xFF, 0x98, 0x00, 0xFF],
            Self::TimedOut => [0xFF, 0x57, 0x22, 0xFF],
            Self::Cancelled => [0x61, 0x61, 0x61, 0xFF],
        })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Skipped | Self::TimedOut | Self::Cancelled
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Retrying(_))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Node Definition
// ═══════════════════════════════════════════════════════════════════════════

/// A node in the workflow graph.
#[derive(Debug, Clone)]
pub struct OrchestratorNode {
    pub id: String,
    pub name: String,
    pub node_type: OrchestratorNodeType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub agent_config: Option<AgentConfig>,
    pub retry_config: RetryConfig,
    pub timeout: Option<Duration>,
    pub metadata: HashMap<String, String>,
    pub webhook_config: WebhookConfig,
    pub schedule_config: ScheduleConfig,
    pub node_metrics: NodeMetrics,
}

impl Default for OrchestratorNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            node_type: OrchestratorNodeType::Agent,
            position: (0.0, 0.0),
            size: (200.0, 100.0),
            inputs: vec!["in".to_string()],
            outputs: vec!["out".to_string()],
            agent_config: None,
            retry_config: RetryConfig::default(),
            timeout: Some(Duration::from_secs(300)),
            metadata: HashMap::new(),
            webhook_config: WebhookConfig::default(),
            schedule_config: ScheduleConfig::default(),
            node_metrics: NodeMetrics::default(),
        }
    }
}

impl OrchestratorNode {
    pub fn agent(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Agent,
            ..Default::default()
        }
    }

    pub fn conditional(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Conditional,
            outputs: vec!["true".to_string(), "false".to_string()],
            ..Default::default()
        }
    }

    pub fn loop_node(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Loop,
            outputs: vec!["body".to_string(), "done".to_string()],
            ..Default::default()
        }
    }

    pub fn parallel(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Parallel,
            outputs: vec![
                "branch_1".to_string(),
                "branch_2".to_string(),
                "branch_3".to_string(),
            ],
            ..Default::default()
        }
    }

    pub fn webhook(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Webhook,
            inputs: vec![],
            ..Default::default()
        }
    }

    pub fn schedule(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Schedule,
            inputs: vec![],
            ..Default::default()
        }
    }

    pub fn data_source(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::DataSource,
            inputs: vec![],
            ..Default::default()
        }
    }

    pub fn output_sink(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: OrchestratorNodeType::Sink,
            outputs: vec![],
            ..Default::default()
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = Some(config);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry_config = retry;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Agent Configuration
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for an AI agent node.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub model: String,
    pub system_prompt: String,
    pub prompt_template: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub stop_sequences: Vec<String>,
    pub skills: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            system_prompt: String::new(),
            prompt_template: String::new(),
            temperature: 0.7,
            max_tokens: 4096,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
            skills: Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Retry Configuration
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f32,
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            exponential_backoff: true,
        }
    }
}

impl RetryConfig {
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if !self.exponential_backoff || attempt == 0 {
            return self.initial_delay;
        }
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let delay = self.initial_delay.mul_f32(multiplier);
        std::cmp::min(delay, self.max_delay)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Definition
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct OrchestratorEdge {
    pub id: String,
    pub source_node: String,
    pub source_port: String,
    pub target_node: String,
    pub target_port: String,
    pub condition: Option<String>,
    pub is_active: bool,
}

impl OrchestratorEdge {
    pub fn new(
        id: impl Into<String>,
        source_node: impl Into<String>,
        source_port: impl Into<String>,
        target_node: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_node: source_node.into(),
            source_port: source_port.into(),
            target_node: target_node.into(),
            target_port: target_port.into(),
            condition: None,
            is_active: false,
        }
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Log Entry
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct OrchestratorLog {
    pub timestamp: Instant,
    pub node_id: String,
    pub level: LogLevel,
    pub message: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Debug => "DBG",
            Self::Info => "INF",
            Self::Warn => "WRN",
            Self::Error => "ERR",
        }
    }

    pub fn color(&self) -> [f32; 4] {
        c(match self {
            Self::Debug => [0x90, 0xA4, 0xAE, 0xFF],
            Self::Info => [0x42, 0xA5, 0xF5, 0xFF],
            Self::Warn => [0xFF, 0x98, 0x00, 0xFF],
            Self::Error => [0xF4, 0x43, 0x36, 0xFF],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Node Execution State
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct NodeExecutionState {
    pub status: NodeExecutionStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<Instant>,
    pub ended_at: Option<Instant>,
    pub retry_count: u32,
    pub token_usage: TokenUsage,
    pub logs: Vec<OrchestratorLog>,
    pub children: Vec<String>,
}

impl Default for NodeExecutionState {
    fn default() -> Self {
        Self {
            status: NodeExecutionStatus::Pending,
            output: None,
            error: None,
            started_at: None,
            ended_at: None,
            retry_count: 0,
            token_usage: TokenUsage::default(),
            logs: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl NodeExecutionState {
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Token Usage / Cost Tracking
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
}

impl TokenUsage {
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
        self.estimated_cost += other.estimated_cost;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Run
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct OrchestratorRun {
    pub id: String,
    pub started_at: Instant,
    pub ended_at: Option<Instant>,
    pub status: RunStatus,
    pub node_states: HashMap<String, NodeExecutionState>,
    pub total_usage: TokenUsage,
    pub total_cost: f64,
    pub duration: Option<std::time::Duration>,
    pub logs: Vec<OrchestratorLog>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn color(&self) -> [f32; 4] {
        c(match self {
            Self::Pending => [0x90, 0xA4, 0xAE, 0xFF],
            Self::Running => [0x42, 0xA5, 0xF5, 0xFF],
            Self::Completed => [0x4C, 0xAF, 0x50, 0xFF],
            Self::Failed => [0xF4, 0x43, 0x36, 0xFF],
            Self::Cancelled => [0x61, 0x61, 0x61, 0xFF],
        })
    }
}

impl Default for OrchestratorRun {
    fn default() -> Self {
        Self {
            id: String::new(),
            started_at: Instant::now(),
            ended_at: None,
            status: RunStatus::Pending,
            node_states: HashMap::new(),
            total_usage: TokenUsage::default(),
            total_cost: 0.0,
            duration: None,
            logs: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Template
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<OrchestratorNode>,
    pub edges: Vec<OrchestratorEdge>,
    pub version: String,
    pub tags: Vec<String>,
}

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
    // New fields for remaining features
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
// Execution Engine
// ═══════════════════════════════════════════════════════════════════════════

/// A step in the execution simulation.
#[derive(Debug, Clone)]
pub enum ExecutionStep {
    /// Start executing a node.
    StartNode(String),
    /// Complete a node with output.
    CompleteNode(String, String),
    /// Fail a node with an error.
    FailNode(String, String),
    /// Retry a node.
    RetryNode(String, u32),
    /// Skip a node.
    SkipNode(String),
    /// Log a message from a node.
    Log(String, LogLevel, String),
    /// Mark an edge as active (data flowing).
    ActivateEdge(String),
    /// Mark an edge as inactive.
    DeactivateEdge(String),
    /// Wait (for simulation timing).
    Wait(Duration),
}

/// The execution engine drives workflow execution as a state machine.
///
/// It processes nodes in topological order, handles retries with exponential
/// backoff, enforces timeouts, and produces a stream of `ExecutionStep` events
/// that the UI consumes to render real-time progress.
#[derive(Debug, Clone, Default)]
pub struct ExecutionEngine {
    /// The pending steps in the simulation.
    steps: Vec<ExecutionStep>,
    /// Current step index.
    current_step: usize,
    /// Whether the engine has been initialized.
    initialized: bool,
}

impl ExecutionEngine {
    /// Build the execution plan for a given workflow state.
    pub fn plan_execution(&mut self, state: &OrchestratorState) {
        self.steps.clear();
        self.current_step = 0;
        self.initialized = true;

        if state.nodes.is_empty() {
            return;
        }

        // Topological sort of nodes based on edges.
        let order = Self::topological_sort(&state.nodes, &state.edges);

        // Produce simulation steps for each node.
        for node_id in &order {
            let node = match state.nodes.iter().find(|n| &n.id == node_id) {
                Some(n) => n,
                None => continue,
            };

            // Activate incoming edges.
            for edge in &state.edges {
                if edge.target_node == *node_id {
                    self.steps
                        .push(ExecutionStep::ActivateEdge(edge.id.clone()));
                }
            }

            match node.node_type {
                OrchestratorNodeType::Agent => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Agent '{}' starting execution...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));

                    // Simulate token usage.
                    let input_tokens = 150 + (node.name.len() as u64 * 10);
                    let output_tokens = 300 + (node.name.len() as u64 * 20);
                    let cost = (input_tokens as f64 * 0.00003) + (output_tokens as f64 * 0.00006);

                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Debug,
                        format!(
                            "Tokens: {} in, {} out, ${:.4} est.",
                            input_tokens, output_tokens, cost
                        ),
                    ));

                    // Simulate processing time.
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(800)));

                    // Check retry config — simulate a retry if max_retries > 1.
                    if node.retry_config.max_retries > 1 {
                        self.steps.push(ExecutionStep::Log(
                            node.id.clone(),
                            LogLevel::Warn,
                            "Simulating retry scenario...".to_string(),
                        ));
                        self.steps
                            .push(ExecutionStep::RetryNode(node.id.clone(), 1));
                        self.steps
                            .push(ExecutionStep::Wait(Duration::from_millis(400)));
                    }

                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        format!("Output from {}: processed successfully.", node.name),
                    ));
                }
                OrchestratorNodeType::Conditional => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Evaluating condition at '{}'...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(300)));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Condition evaluated: true branch taken.".to_string(),
                    ));
                }
                OrchestratorNodeType::Loop => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Starting loop '{}' (3 iterations)...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    for i in 1..=3 {
                        self.steps.push(ExecutionStep::Log(
                            node.id.clone(),
                            LogLevel::Debug,
                            format!("Loop iteration {}/3", i),
                        ));
                        self.steps
                            .push(ExecutionStep::Wait(Duration::from_millis(300)));
                    }
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Loop completed after 3 iterations.".to_string(),
                    ));
                }
                OrchestratorNodeType::Parallel => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Parallel execution '{}' — fanning out...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(500)));
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        "All parallel branches completed.".to_string(),
                    ));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Parallel execution completed.".to_string(),
                    ));
                }
                OrchestratorNodeType::Webhook => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Webhook '{}' triggered.", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(100)));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Webhook received and acknowledged.".to_string(),
                    ));
                }
                OrchestratorNodeType::Schedule => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Schedule '{}' triggered.", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(100)));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Scheduled trigger fired.".to_string(),
                    ));
                }
                OrchestratorNodeType::DataSource => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Loading data from '{}'...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(200)));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Data loaded successfully.".to_string(),
                    ));
                }
                OrchestratorNodeType::Sink => {
                    self.steps.push(ExecutionStep::Log(
                        node.id.clone(),
                        LogLevel::Info,
                        format!("Writing output to '{}'...", node.name),
                    ));
                    self.steps.push(ExecutionStep::StartNode(node.id.clone()));
                    self.steps
                        .push(ExecutionStep::Wait(Duration::from_millis(200)));
                    self.steps.push(ExecutionStep::CompleteNode(
                        node.id.clone(),
                        "Output saved successfully.".to_string(),
                    ));
                }
            }

            // Deactivate incoming edges.
            for edge in &state.edges {
                if edge.target_node == *node_id {
                    self.steps
                        .push(ExecutionStep::DeactivateEdge(edge.id.clone()));
                }
            }
        }
    }

    /// Advance the execution by one step. Returns the step if there is one.
    pub fn tick(&mut self) -> Option<ExecutionStep> {
        if self.current_step >= self.steps.len() {
            return None;
        }
        let step = self.steps[self.current_step].clone();
        self.current_step += 1;
        Some(step)
    }

    /// Whether the execution is complete.
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.steps.len()
    }

    /// Total number of steps.
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Current step index.
    pub fn current_step_index(&self) -> usize {
        self.current_step
    }

    /// Reset the engine.
    pub fn reset(&mut self) {
        self.steps.clear();
        self.current_step = 0;
        self.initialized = false;
    }

    /// Topological sort of nodes based on edges.
    fn topological_sort(nodes: &[OrchestratorNode], edges: &[OrchestratorEdge]) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in nodes {
            in_degree.entry(node.id.clone()).or_insert(0);
            adj.entry(node.id.clone()).or_default();
        }

        for edge in edges {
            *in_degree.entry(edge.target_node.clone()).or_insert(0) += 1;
            adj.entry(edge.source_node.clone())
                .or_default()
                .push(edge.target_node.clone());
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        // Sort queue for deterministic order.
        queue.sort();

        let mut result = Vec::new();
        while let Some(id) = queue.pop() {
            result.push(id.clone());
            if let Some(neighbors) = adj.get(&id) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
            queue.sort();
        }

        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Multi-Agent Orchestrator Component
// ═══════════════════════════════════════════════════════════════════════════

/// The main orchestrator component — a `View` that renders the workflow graph,
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
// View Implementation — Rendering
// ═══════════════════════════════════════════════════════════════════════════

use cvkg_core::Never;

impl View for MultiAgentOrchestrator {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // ── Background ──────────────────────────────────────────────────
        renderer.fill_rect(rect, [0.04, 0.03, 0.06, 1.0]);

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
        let state = self.state.clone();

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
        let _run_state = state.clone();
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
                            let mut new_state = guard.read().unwrap().clone();
                            if new_state.is_executing {
                                new_state.is_executing = false;
                            } else {
                                new_state.is_executing = true;
                                new_state.run_counter += 1;
                                new_state.active_run_id =
                                    Some(format!("run-{}", new_state.run_counter));
                            }
                            *guard.write().unwrap() = new_state;
                        }
                        s
                    });
                }
            }),
        );

        // ── Log panel toggle ────────────────────────────────────────────
        let _log_state = state.clone();
        renderer.register_handler(
            "pointerdown:togglelog",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerDown { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let s = s.clone();
                        if let Some(guard) = s.get_component_state::<OrchestratorState>(instance_id)
                        {
                            let mut new_state = guard.read().unwrap().clone();
                            new_state.show_log_panel = !new_state.show_log_panel;
                            *guard.write().unwrap() = new_state;
                        }
                        s
                    });
                }
            }),
        );

        // ── Metrics panel toggle ────────────────────────────────────────
        let _metrics_state = state.clone();
        renderer.register_handler(
            "pointerdown:togglemetrics",
            std::sync::Arc::new(move |event| {
                if let cvkg_core::Event::PointerDown { .. } = event {
                    cvkg_core::update_system_state(move |s| {
                        let s = s.clone();
                        if let Some(guard) = s.get_component_state::<OrchestratorState>(instance_id)
                        {
                            let mut new_state = guard.read().unwrap().clone();
                            new_state.show_metrics_panel = !new_state.show_metrics_panel;
                            *guard.write().unwrap() = new_state;
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

// ═══════════════════════════════════════════════════════════════════════════
// Rendering Helpers
// ═══════════════════════════════════════════════════════════════════════════

impl MultiAgentOrchestrator {
    /// Render the main graph canvas with nodes and edges.
    fn render_graph_canvas(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, [0.05, 0.04, 0.08, 1.0]);

        // Grid
        self.render_grid(renderer, rect);

        // Clip to graph area
        renderer.push_clip_rect(rect);

        // Render edges (behind nodes)
        self.render_edges(renderer, rect);

        // Render nodes
        for node in &self.state.nodes {
            self.render_node(renderer, node, rect);
        }

        // Render pending edge (if any)
        if let Some((ref node_id, ref port)) = self.pending_edge {
            self.render_pending_edge(renderer, node_id, port, rect);
        }

        renderer.pop_clip_rect();

        // ── Toolbar ─────────────────────────────────────────────────────
        self.render_toolbar(renderer, rect);
    }

    /// Render the background grid.
    fn render_grid(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let grid_spacing = 40.0 * self.state.viewport_zoom;
        let offset_x = self.state.viewport_offset.0 % grid_spacing;
        let offset_y = self.state.viewport_offset.1 % grid_spacing;
        let grid_color = [0.08, 0.07, 0.12, 1.0];

        let mut x = rect.x + offset_x;
        while x < rect.x + rect.width {
            renderer.draw_line(x, rect.y, x, rect.y + rect.height, grid_color, 0.5);
            x += grid_spacing;
        }

        let mut y = rect.y + offset_y;
        while y < rect.y + rect.height {
            renderer.draw_line(rect.x, y, rect.x + rect.width, y, grid_color, 0.5);
            y += grid_spacing;
        }
    }

    /// Render all edges as Bézier curves.
    fn render_edges(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        for edge in &self.state.edges {
            let source = match self.state.nodes.iter().find(|n| n.id == edge.source_node) {
                Some(n) => n,
                None => continue,
            };
            let target = match self.state.nodes.iter().find(|n| n.id == edge.target_node) {
                Some(n) => n,
                None => continue,
            };

            let sx = source.position.0 + source.size.0;
            let sy = source.position.1 + source.size.1 / 2.0;
            let tx = target.position.0;
            let ty = target.position.1 + target.size.1 / 2.0;

            let dx = (tx - sx).abs();
            let cp_offset = dx * 0.4;

            // Glow (wider, translucent)
            let glow_color = if edge.is_active {
                [0.0, 0.8, 1.0, 0.15]
            } else {
                [0.2, 0.3, 0.4, 0.1]
            };
            self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, glow_color, 6.0);

            // Main edge
            let edge_color = if edge.is_active {
                [0.0, 0.8, 1.0, 0.9]
            } else {
                [0.3, 0.4, 0.5, 0.7]
            };
            self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, edge_color, 2.0);

            // Arrow head
            self.draw_arrow_head(renderer, tx, ty, tx - cp_offset, ty, edge_color);

            // Condition label
            if let Some(ref cond) = edge.condition {
                let mid_x = (sx + tx) / 2.0;
                let mid_y = (sy + ty) / 2.0 - 12.0;
                let tw = renderer.measure_text(cond, 9.0);
                renderer.fill_rounded_rect(
                    Rect {
                        x: mid_x - tw.0 / 2.0 - 4.0,
                        y: mid_y - 6.0,
                        width: tw.0 + 8.0,
                        height: 14.0,
                    },
                    3.0,
                    [0.08, 0.06, 0.12, 0.85],
                );
                renderer.draw_text(
                    cond,
                    mid_x - tw.0 / 2.0,
                    mid_y + 3.0,
                    9.0,
                    [0.7, 0.8, 0.9, 0.9],
                );
            }
        }
    }

    /// Draw a cubic Bézier curve as a series of line segments.
    fn draw_bezier_edge(
        &self,
        renderer: &mut dyn Renderer,
        x0: f32,
        y0: f32,
        x3: f32,
        y3: f32,
        cp_offset: f32,
        color: [f32; 4],
        width: f32,
    ) {
        let x1 = x0 + cp_offset;
        let y1 = y0;
        let x2 = x3 - cp_offset;
        let y2 = y3;

        let segments = 24;
        let mut prev_x = x0;
        let mut prev_y = y0;

        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            let x = mt3 * x0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * x3;
            let y = mt3 * y0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * y3;

            renderer.draw_line(prev_x, prev_y, x, y, color, width);
            prev_x = x;
            prev_y = y;
        }
    }

    /// Draw an arrow head at the end of an edge.
    fn draw_arrow_head(
        &self,
        renderer: &mut dyn Renderer,
        tip_x: f32,
        tip_y: f32,
        from_x: f32,
        from_y: f32,
        color: [f32; 4],
    ) {
        let dx = tip_x - from_x;
        let dy = tip_y - from_y;
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let nx = dx / len;
        let ny = dy / len;

        let arrow_len = 10.0;
        let arrow_width = 5.0;

        let left_x = tip_x - nx * arrow_len - ny * arrow_width;
        let left_y = tip_y - ny * arrow_len + nx * arrow_width;
        let right_x = tip_x - nx * arrow_len + ny * arrow_width;
        let right_y = tip_y - ny * arrow_len - nx * arrow_width;

        renderer.draw_line(tip_x, tip_y, left_x, left_y, color, 2.0);
        renderer.draw_line(tip_x, tip_y, right_x, right_y, color, 2.0);
        renderer.draw_line(left_x, left_y, right_x, right_y, color, 2.0);
    }

    /// Render a single node.
    fn render_node(&self, renderer: &mut dyn Renderer, node: &OrchestratorNode, _rect: Rect) {
        let nx = node.position.0 + self.state.viewport_offset.0;
        let ny = node.position.1 + self.state.viewport_offset.1;
        let nw = node.size.0;
        let nh = node.size.1;

        let node_rect = Rect {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
        };

        let is_selected = self.state.selected_node.as_ref() == Some(&node.id);
        let node_color = node.node_type.color();

        // Drop shadow
        renderer.fill_rounded_rect(
            Rect {
                x: nx + 3.0,
                y: ny + 3.0,
                width: nw,
                height: nh,
            },
            6.0,
            theme::shadow(),
        );

        // Node body
        renderer.fill_rounded_rect(node_rect, 6.0, [0.08, 0.07, 0.12, 0.95]);

        // Selection highlight
        if is_selected {
            renderer.stroke_rounded_rect(node_rect, 6.0, theme::accent(), 2.0);
        }

        // Title bar
        let title_h = 28.0;
        let title_rect = Rect {
            x: nx,
            y: ny,
            width: nw,
            height: title_h,
        };
        renderer.fill_rounded_rect(title_rect, 6.0, node_color);
        // Cover bottom corners of title bar
        renderer.fill_rect(
            Rect {
                x: nx,
                y: ny + title_h - 8.0,
                width: nw,
                height: 8.0,
            },
            node_color,
        );

        // Icon
        renderer.draw_text(
            node.node_type.icon(),
            nx + 8.0,
            ny + 7.0,
            14.0,
            [0.0, 0.0, 0.0, 0.9],
        );

        // Name
        renderer.draw_text(&node.name, nx + 28.0, ny + 7.0, 12.0, theme::surface());

        // Status indicator (if running)
        if let Some(ref run) = self.state.current_run
            && let Some(node_state) = run.node_states.get(&node.id)
        {
            let status_color = node_state.status.color();
            let status_label = node_state.status.label();
            let tw = renderer.measure_text(status_label, 9.0);
            renderer.draw_text(
                status_label,
                nx + nw - tw.0 - 8.0,
                ny + 9.0,
                9.0,
                status_color,
            );
        }

        // Separator line
        renderer.draw_line(
            nx + 6.0,
            ny + title_h,
            nx + nw - 6.0,
            ny + title_h,
            [0.15, 0.12, 0.2, 0.5],
            1.0,
        );

        // Ports
        self.render_ports(renderer, node, nx, ny, nw, nh);
    }

    /// Render input and output ports for a node.
    fn render_ports(
        &self,
        renderer: &mut dyn Renderer,
        node: &OrchestratorNode,
        nx: f32,
        ny: f32,
        nw: f32,
        nh: f32,
    ) {
        let port_radius = 5.0;
        let title_h = 28.0;
        let content_h = nh - title_h;
        let input_count = node.inputs.len().max(1);
        let output_count = node.outputs.len().max(1);
        let spacing_in = content_h / (input_count as f32 + 1.0);
        let spacing_out = content_h / (output_count as f32 + 1.0);

        // Input ports (left side)
        for (i, port_name) in node.inputs.iter().enumerate() {
            let py = ny + title_h + spacing_in * (i as f32 + 1.0);
            let px = nx;

            renderer.fill_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                [0.0, 0.8, 1.0, 0.9],
            );
            renderer.stroke_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                [0.4, 0.6, 0.8, 0.6],
                1.0,
            );
            renderer.draw_text(port_name, px + 10.0, py - 4.0, 9.0, [0.6, 0.7, 0.8, 0.9]);
        }

        // Output ports (right side)
        for (i, port_name) in node.outputs.iter().enumerate() {
            let py = ny + title_h + spacing_out * (i as f32 + 1.0);
            let px = nx + nw;

            renderer.fill_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                [0.0, 1.0, 0.6, 0.9],
            );
            renderer.stroke_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                [0.4, 0.8, 0.6, 0.6],
                1.0,
            );
            let tw = renderer.measure_text(port_name, 9.0);
            renderer.draw_text(
                port_name,
                px - 10.0 - tw.0,
                py - 4.0,
                9.0,
                [0.6, 0.8, 0.7, 0.9],
            );
        }
    }

    /// Render a pending edge being created.
    fn render_pending_edge(
        &self,
        renderer: &mut dyn Renderer,
        node_id: &str,
        _port: &str,
        _rect: Rect,
    ) {
        let source = match self.state.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => return,
        };

        let sx = source.position.0 + source.size.0;
        let sy = source.position.1 + source.size.1 / 2.0;
        let tx = self.pointer_pos.0;
        let ty = self.pointer_pos.1;

        let dx = (tx - sx).abs();
        let cp_offset = dx * 0.4;

        // Dashed preview
        let preview_color = [0.0, 0.8, 1.0, 0.5];
        self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, preview_color, 1.5);

        // Source dot
        renderer.fill_ellipse(
            Rect {
                x: sx - 6.0,
                y: sy - 6.0,
                width: 12.0,
                height: 12.0,
            },
            theme::accent(),
        );
    }

    /// Render the toolbar at the top of the graph canvas.
    fn render_toolbar(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let toolbar_h = 36.0;
        let toolbar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: toolbar_h,
        };

        renderer.fill_rect(toolbar_rect, [0.06, 0.05, 0.1, 0.95]);
        renderer.draw_line(
            rect.x,
            rect.y + toolbar_h,
            rect.x + rect.width,
            rect.y + toolbar_h,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        // Title
        renderer.draw_text(
            "Orchestrator",
            rect.x + 12.0,
            rect.y + 10.0,
            14.0,
            [0.8, 0.7, 1.0, 1.0],
        );

        // Run/Stop button
        let btn_x = rect.x + rect.width - 100.0;
        let btn_y = rect.y + 6.0;
        let btn_w = 88.0;
        let btn_h = 24.0;

        if self.state.is_executing {
            renderer.fill_rounded_rect(
                Rect {
                    x: btn_x,
                    y: btn_y,
                    width: btn_w,
                    height: btn_h,
                },
                4.0,
                [0.8, 0.2, 0.2, 0.9],
            );
            renderer.draw_text("■ Stop", btn_x + 12.0, btn_y + 6.0, 11.0, theme::text());
        } else {
            renderer.fill_rounded_rect(
                Rect {
                    x: btn_x,
                    y: btn_y,
                    width: btn_w,
                    height: btn_h,
                },
                4.0,
                [0.0, 0.7, 0.4, 0.9],
            );
            renderer.draw_text("▶ Run", btn_x + 16.0, btn_y + 6.0, 11.0, theme::text());
        }

        // Execution progress
        if self.state.is_executing {
            let total = self.engine.total_steps().max(1);
            let current = self.engine.current_step_index();
            let progress = current as f32 / total as f32;
            let bar_x = rect.x + 140.0;
            let bar_y = rect.y + 14.0;
            let bar_w = 120.0;
            let bar_h = 8.0;

            renderer.fill_rounded_rect(
                Rect {
                    x: bar_x,
                    y: bar_y,
                    width: bar_w,
                    height: bar_h,
                },
                3.0,
                [0.1, 0.08, 0.15, 1.0],
            );
            renderer.fill_rounded_rect(
                Rect {
                    x: bar_x,
                    y: bar_y,
                    width: bar_w * progress,
                    height: bar_h,
                },
                3.0,
                [0.0, 0.8, 1.0, 0.9],
            );
            let pct_text = format!("{}/{}", current, total);
            let tw = renderer.measure_text(&pct_text, 9.0);
            renderer.draw_text(
                &pct_text,
                bar_x + bar_w / 2.0 - tw.0 / 2.0,
                bar_y - 1.0,
                9.0,
                [0.6, 0.7, 0.8, 0.8],
            );
        }
    }

    /// Render the log panel on the right side.
    fn render_log_panel(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, [0.06, 0.05, 0.1, 0.95]);

        // Left border
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        // Header
        let header_h = 32.0;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: header_h,
            },
            [0.08, 0.06, 0.12, 1.0],
        );
        renderer.draw_text(
            "Execution Log",
            rect.x + 10.0,
            rect.y + 9.0,
            13.0,
            [0.8, 0.7, 1.0, 1.0],
        );
        renderer.draw_line(
            rect.x,
            rect.y + header_h,
            rect.x + rect.width,
            rect.y + header_h,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        // Log entries
        let logs: Vec<&OrchestratorLog> = match &self.state.current_run {
            Some(run) => run.logs.iter().rev().take(50).collect(),
            None => Vec::new(),
        };

        if logs.is_empty() {
            renderer.draw_text(
                "No logs yet. Click ▶ Run to start.",
                rect.x + 10.0,
                rect.y + header_h + 20.0,
                11.0,
                [0.4, 0.4, 0.5, 0.8],
            );
            return;
        }

        let mut y = rect.y + header_h + 6.0;
        let line_h = 16.0;
        let max_y = rect.y + rect.height;

        for log in logs.into_iter().rev() {
            if y + line_h > max_y {
                break;
            }

            let level_color = log.level.color();
            let level_label = log.level.label();

            // Level badge
            renderer.draw_text(level_label, rect.x + 8.0, y, 9.0, level_color);

            // Message
            renderer.draw_text(&log.message, rect.x + 36.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);

            y += line_h;
        }
    }

    /// Render the metrics panel.
    fn render_metrics_panel(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, [0.06, 0.05, 0.1, 0.95]);

        // Left border
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        // Header
        let header_h = 32.0;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: header_h,
            },
            [0.08, 0.06, 0.12, 1.0],
        );
        renderer.draw_text(
            "Metrics",
            rect.x + 10.0,
            rect.y + 9.0,
            13.0,
            [0.8, 0.7, 1.0, 1.0],
        );
        renderer.draw_line(
            rect.x,
            rect.y + header_h,
            rect.x + rect.width,
            rect.y + header_h,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        let mut y = rect.y + header_h + 12.0;
        let line_h = 20.0;

        if let Some(ref run) = self.state.current_run {
            // Run status
            let status_color = run.status.color();
            renderer.draw_text("Status:", rect.x + 10.0, y, 11.0, [0.6, 0.6, 0.7, 0.9]);
            y += line_h;
            renderer.draw_text(run.status.label(), rect.x + 16.0, y, 11.0, status_color);
            y += line_h + 6.0;

            // Node count
            let total_nodes = run.node_states.len();
            let completed = run
                .node_states
                .values()
                .filter(|s| s.status == NodeExecutionStatus::Completed)
                .count();
            let failed = run
                .node_states
                .values()
                .filter(|s| s.status == NodeExecutionStatus::Failed)
                .count();
            let running = run
                .node_states
                .values()
                .filter(|s| s.status.is_active())
                .count();

            renderer.draw_text("Nodes:", rect.x + 10.0, y, 11.0, [0.6, 0.6, 0.7, 0.9]);
            y += line_h;
            let node_summary = format!(
                "{} total, {} done, {} running, {} failed",
                total_nodes, completed, running, failed
            );
            renderer.draw_text(&node_summary, rect.x + 16.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            y += line_h + 6.0;

            // Token usage
            renderer.draw_text("Tokens:", rect.x + 10.0, y, 11.0, [0.6, 0.6, 0.7, 0.9]);
            y += line_h;
            let token_text = format!(
                "In: {}  Out: {}  Total: {}",
                run.total_usage.input_tokens,
                run.total_usage.output_tokens,
                run.total_usage.total_tokens
            );
            renderer.draw_text(&token_text, rect.x + 16.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            y += line_h;

            // Cost
            renderer.draw_text("Cost:", rect.x + 10.0, y, 11.0, [0.6, 0.6, 0.7, 0.9]);
            y += line_h;
            let cost_text = format!("${:.4}", run.total_usage.estimated_cost);
            renderer.draw_text(&cost_text, rect.x + 16.0, y, 10.0, [0.0, 0.9, 0.5, 0.9]);
            y += line_h + 6.0;

            // Per-node breakdown
            renderer.draw_text("Per-Node:", rect.x + 10.0, y, 11.0, [0.6, 0.6, 0.7, 0.9]);
            y += line_h;

            for node in &self.state.nodes {
                if let Some(node_state) = run.node_states.get(&node.id) {
                    if y + line_h > rect.y + rect.height {
                        break;
                    }
                    let status_color = node_state.status.color();
                    let line = format!(
                        "{}: {} ({}toks)",
                        node.name,
                        node_state.status.label(),
                        node_state.token_usage.total_tokens
                    );
                    renderer.draw_text(&line, rect.x + 16.0, y, 9.0, status_color);
                    y += line_h;
                }
            }
        } else {
            renderer.draw_text(
                "No run data yet.",
                rect.x + 10.0,
                y,
                11.0,
                [0.4, 0.4, 0.5, 0.8],
            );
        }
    }

    /// Render the template library overlay panel.
    fn render_template_library(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let panel_w = 380.0;
        let panel_h = rect.height - 80.0;
        let panel_x = rect.x + (rect.width - panel_w) / 2.0;
        let panel_y = rect.y + 40.0;

        // Background overlay
        renderer.fill_rect(rect, theme::shadow());

        // Panel
        let panel_rect = Rect {
            x: panel_x,
            y: panel_y,
            width: panel_w,
            height: panel_h,
        };
        renderer.fill_rounded_rect(panel_rect, 8.0, [0.08, 0.06, 0.12, 0.95]);
        renderer.stroke_rounded_rect(panel_rect, 8.0, [0.2, 0.15, 0.3, 0.8], 1.0);

        // Title
        renderer.draw_text(
            "Template Library",
            panel_x + 16.0,
            panel_y + 14.0,
            14.0,
            [0.8, 0.7, 1.0, 1.0],
        );
        renderer.draw_line(
            panel_x,
            panel_y + 32.0,
            panel_x + panel_w,
            panel_y + 32.0,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        // Template list
        let templates = &self.state.templates;
        if templates.is_empty() {
            renderer.draw_text(
                "No templates saved yet.",
                panel_x + 16.0,
                panel_y + 50.0,
                11.0,
                [0.5, 0.5, 0.6, 0.8],
            );
            renderer.draw_text(
                "Save your current workflow as a template.",
                panel_x + 16.0,
                panel_y + 66.0,
                10.0,
                [0.4, 0.4, 0.5, 0.7],
            );
        } else {
            let mut y = panel_y + 42.0;
            for template in templates.iter().take(20) {
                // Template card
                let card_rect = Rect {
                    x: panel_x + 10.0,
                    y,
                    width: panel_w - 20.0,
                    height: 52.0,
                };
                renderer.fill_rounded_rect(card_rect, 4.0, [0.1, 0.08, 0.15, 0.9]);
                renderer.draw_text(
                    &template.name,
                    card_rect.x + 8.0,
                    card_rect.y + 10.0,
                    11.0,
                    [0.8, 0.8, 0.9, 0.9],
                );
                let desc = if template.description.len() > 40 {
                    &template.description[..40]
                } else {
                    &template.description
                };
                renderer.draw_text(
                    desc,
                    card_rect.x + 8.0,
                    card_rect.y + 26.0,
                    9.0,
                    [0.5, 0.5, 0.6, 0.8],
                );
                let meta = format!("v{} · {} nodes", template.version, template.nodes.len());
                renderer.draw_text(
                    &meta,
                    card_rect.x + 8.0,
                    card_rect.y + 38.0,
                    8.0,
                    [0.4, 0.4, 0.5, 0.7],
                );
                y += 58.0;
            }
        }

        // Close hint
        renderer.draw_text(
            "Click outside to close",
            panel_x + 16.0,
            panel_y + panel_h - 16.0,
            9.0,
            [0.4, 0.4, 0.5, 0.6],
        );
    }

    /// Render the run comparison overlay panel.
    fn render_run_comparison(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let panel_w = 500.0;
        let panel_h = rect.height - 80.0;
        let panel_x = rect.x + (rect.width - panel_w) / 2.0;
        let panel_y = rect.y + 40.0;

        // Background overlay
        renderer.fill_rect(rect, theme::shadow());

        // Panel
        let panel_rect = Rect {
            x: panel_x,
            y: panel_y,
            width: panel_w,
            height: panel_h,
        };
        renderer.fill_rounded_rect(panel_rect, 8.0, [0.08, 0.06, 0.12, 0.95]);
        renderer.stroke_rounded_rect(panel_rect, 8.0, [0.2, 0.15, 0.3, 0.8], 1.0);

        // Title
        renderer.draw_text(
            "Run Comparison",
            panel_x + 16.0,
            panel_y + 14.0,
            14.0,
            [0.8, 0.7, 1.0, 1.0],
        );
        renderer.draw_line(
            panel_x,
            panel_y + 32.0,
            panel_x + panel_w,
            panel_y + 32.0,
            [0.15, 0.12, 0.2, 0.6],
            1.0,
        );

        let history = &self.state.run_history;
        if history.len() < 2 {
            renderer.draw_text(
                "Need at least 2 runs to compare.",
                panel_x + 16.0,
                panel_y + 50.0,
                11.0,
                [0.5, 0.5, 0.6, 0.8],
            );
            renderer.draw_text(
                "Run the workflow multiple times.",
                panel_x + 16.0,
                panel_y + 66.0,
                10.0,
                [0.4, 0.4, 0.5, 0.7],
            );
        } else {
            let mut y = panel_y + 42.0;
            // Compare last two runs
            let run_a = &history[history.len() - 2];
            let run_b = &history[history.len() - 1];

            renderer.draw_text(
                "Run A (previous)",
                panel_x + 16.0,
                y,
                11.0,
                [0.6, 0.8, 0.6, 0.9],
            );
            renderer.draw_text(
                "Run B (latest)",
                panel_x + 260.0,
                y,
                11.0,
                [0.6, 0.8, 1.0, 0.9],
            );
            y += 20.0;

            // Duration comparison
            renderer.draw_text("Duration:", panel_x + 16.0, y, 10.0, [0.6, 0.6, 0.7, 0.9]);
            let dur_a = format!("{:?}", run_a.duration);
            let dur_b = format!("{:?}", run_b.duration);
            renderer.draw_text(&dur_a, panel_x + 100.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            renderer.draw_text(&dur_b, panel_x + 260.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            y += 16.0;

            // Token comparison
            renderer.draw_text("Tokens:", panel_x + 16.0, y, 10.0, [0.6, 0.6, 0.7, 0.9]);
            let tok_a = format!("{}", run_a.total_usage.total_tokens);
            let tok_b = format!("{}", run_b.total_usage.total_tokens);
            renderer.draw_text(&tok_a, panel_x + 100.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            renderer.draw_text(&tok_b, panel_x + 260.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            y += 16.0;

            // Cost comparison
            renderer.draw_text("Cost:", panel_x + 16.0, y, 10.0, [0.6, 0.6, 0.7, 0.9]);
            let cost_a = format!("${:.4}", run_a.total_cost);
            let cost_b = format!("${:.4}", run_b.total_cost);
            renderer.draw_text(&cost_a, panel_x + 100.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            renderer.draw_text(&cost_b, panel_x + 260.0, y, 10.0, [0.7, 0.7, 0.8, 0.9]);
            y += 24.0;

            // Per-node comparison
            renderer.draw_text(
                "Per-node token usage:",
                panel_x + 16.0,
                y,
                10.0,
                [0.6, 0.6, 0.7, 0.9],
            );
            y += 16.0;

            for (node_id, state_a) in &run_a.node_states {
                if let Some(state_b) = run_b.node_states.get(node_id) {
                    let tok_a = state_a.token_usage.total_tokens;
                    let tok_b = state_b.token_usage.total_tokens;
                    let diff = if tok_b > tok_a {
                        format!("+{}", tok_b - tok_a)
                    } else {
                        format!("{}", tok_b - tok_a)
                    };
                    let line = format!("  {}: {} → {} ({})", node_id, tok_a, tok_b, diff);
                    let color = if tok_b > tok_a {
                        [0.9, 0.5, 0.3, 0.9]
                    } else {
                        [0.5, 0.9, 0.5, 0.9]
                    };
                    renderer.draw_text(&line, panel_x + 16.0, y, 9.0, color);
                    y += 14.0;
                }
            }
        }

        // Close hint
        renderer.draw_text(
            "Click outside to close",
            panel_x + 16.0,
            panel_y + panel_h - 16.0,
            9.0,
            [0.4, 0.4, 0.5, 0.6],
        );
    }
}

// === FEATURE: SkillRegistry ===
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: std::collections::HashMap<String, SkillDef>,
}

#[derive(Debug, Clone)]
pub struct SkillDef {
    pub name: String,
    pub base_token_cost: u32,
    pub base_duration_ms: u32,
    pub description: String,
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut skills = std::collections::HashMap::new();
        skills.insert(
            "text_generation".to_string(),
            SkillDef {
                name: "text_generation".to_string(),
                base_token_cost: 1500,
                base_duration_ms: 2000,
                description: "General text generation".to_string(),
            },
        );
        skills.insert(
            "code_analysis".to_string(),
            SkillDef {
                name: "code_analysis".to_string(),
                base_token_cost: 3000,
                base_duration_ms: 4000,
                description: "Analyze and review code".to_string(),
            },
        );
        skills.insert(
            "summarization".to_string(),
            SkillDef {
                name: "summarization".to_string(),
                base_token_cost: 800,
                base_duration_ms: 1500,
                description: "Summarize long text".to_string(),
            },
        );
        Self { skills }
    }

    pub fn get(&self, name: &str) -> Option<&SkillDef> {
        self.skills.get(name)
    }

    pub fn token_cost_for(&self, skill_names: &[String]) -> u32 {
        skill_names
            .iter()
            .filter_map(|n| self.get(n))
            .map(|s| s.base_token_cost)
            .sum()
    }

    pub fn duration_for(&self, skill_names: &[String]) -> u32 {
        skill_names
            .iter()
            .filter_map(|n| self.get(n))
            .map(|s| s.base_duration_ms)
            .sum::<u32>()
            .max(500)
    }

    pub fn list_skills(&self) -> Vec<&SkillDef> {
        self.skills.values().collect()
    }
}

// === FEATURE: WebhookConfig ===
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body_template: String,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "POST".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body_template: "{}".to_string(),
        }
    }
}

// === FEATURE: ScheduleConfig ===
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub cron_expression: String,
    pub interval_seconds: f32,
    pub next_trigger_time: f32,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            cron_expression: "0 * * * *".to_string(),
            interval_seconds: 3600.0,
            next_trigger_time: 0.0,
        }
    }
}

// === FEATURE: RecurringRun ===
#[derive(Debug, Clone)]
pub struct RecurringRun {
    pub id: u64,
    pub name: String,
    pub interval_seconds: f32,
    pub last_run_time: f32,
    pub next_run_time: f32,
    pub enabled: bool,
    pub run_count: u32,
}

impl Default for RecurringRun {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Recurring Run".to_string(),
            interval_seconds: 3600.0,
            last_run_time: 0.0,
            next_run_time: 3600.0,
            enabled: false,
            run_count: 0,
        }
    }
}

// === FEATURE: Undo/Redo ===
#[derive(Debug, Clone)]
pub enum GraphAction {
    AddNode {
        node: OrchestratorNode,
    },
    RemoveNode {
        node_id: u64,
    },
    MoveNode {
        node_id: u64,
        old_x: f32,
        old_y: f32,
    },
    AddEdge {
        edge: OrchestratorEdge,
    },
    RemoveEdge {
        edge_id: u64,
    },
}

#[derive(Debug, Clone, Default)]
pub struct UndoRedo {
    undo_stack: Vec<GraphAction>,
    redo_stack: Vec<GraphAction>,
}

impl UndoRedo {
    pub fn push(&mut self, action: GraphAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<GraphAction> {
        self.undo_stack.pop()
    }

    pub fn redo(&mut self) -> Option<GraphAction> {
        self.redo_stack.pop()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// === FEATURE: ValidationResult ===
#[derive(Debug, Clone, Default)]
pub struct ValidationError {
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub issues: Vec<ValidationError>,
    pub is_valid: bool,
}

// === FEATURE: SearchState ===
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub is_active: bool,
    pub matched_node_ids: Vec<u64>,
}

// === FEATURE: Workflow Validation ===
pub fn validate_workflow(
    nodes: &[OrchestratorNode],
    edges: &[OrchestratorEdge],
) -> Vec<ValidationError> {
    let mut issues = Vec::new();

    if nodes.is_empty() {
        issues.push(ValidationError {
            message: "Workflow has no nodes".to_string(),
            is_error: true,
        });
        return issues;
    }

    let connected_ids: std::collections::HashSet<&str> = edges
        .iter()
        .flat_map(|e| vec![e.source_node.as_str(), e.target_node.as_str()])
        .collect();

    for node in nodes {
        if !connected_ids.contains(node.id.as_str()) {
            issues.push(ValidationError {
                message: format!("Node '{}' is disconnected", node.name),
                is_error: false,
            });
        }
    }

    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for edge in edges {
        adj.entry(edge.source_node.as_str())
            .or_default()
            .push(edge.target_node.as_str());
    }

    let mut visited = std::collections::HashSet::new();
    let mut in_stack = std::collections::HashSet::new();

    fn has_cycle<'a>(
        node: &'a str,
        adj: &std::collections::HashMap<&'a str, Vec<&'a str>>,
        visited: &mut std::collections::HashSet<&'a str>,
        in_stack: &mut std::collections::HashSet<&'a str>,
    ) -> bool {
        visited.insert(node);
        in_stack.insert(node);
        if let Some(neighbors) = adj.get(node) {
            for &n in neighbors {
                if !visited.contains(n) {
                    if has_cycle(n, adj, visited, in_stack) {
                        return true;
                    }
                } else if in_stack.contains(n) {
                    return true;
                }
            }
        }
        in_stack.remove(node);
        false
    }

    for node in nodes {
        if !visited.contains(node.id.as_str())
            && has_cycle(node.id.as_str(), &adj, &mut visited, &mut in_stack)
        {
            issues.push(ValidationError {
                message: "Workflow contains a cycle".to_string(),
                is_error: true,
            });
            break;
        }
    }

    let has_input: std::collections::HashSet<&str> =
        edges.iter().map(|e| e.target_node.as_str()).collect();

    let start_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| !has_input.contains(n.id.as_str()))
        .collect();

    if start_nodes.is_empty() && !nodes.is_empty() {
        issues.push(ValidationError {
            message: "No start node found (all nodes have inputs)".to_string(),
            is_error: false,
        });
    }

    issues
}

// === FEATURE: Zoom to fit ===
pub fn zoom_to_fit(
    nodes: &[OrchestratorNode],
    viewport_w: f32,
    viewport_h: f32,
) -> (f32, f32, f32) {
    if nodes.is_empty() {
        return (0.0, 0.0, 1.0);
    }
    let min_x = nodes
        .iter()
        .map(|n| n.position.0)
        .fold(f32::INFINITY, f32::min);
    let max_x = nodes
        .iter()
        .map(|n| n.position.0 + n.size.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = nodes
        .iter()
        .map(|n| n.position.1)
        .fold(f32::INFINITY, f32::min);
    let max_y = nodes
        .iter()
        .map(|n| n.position.1 + n.size.1)
        .fold(f32::NEG_INFINITY, f32::max);
    let content_w = max_x - min_x + 80.0;
    let content_h = max_y - min_y + 80.0;
    let zoom = (viewport_w / content_w)
        .min(viewport_h / content_h)
        .min(2.0)
        .max(0.1);
    let offset_x = (viewport_w - content_w * zoom) / 2.0 - min_x * zoom + 40.0 * zoom;
    let offset_y = (viewport_h - content_h * zoom) / 2.0 - min_y * zoom + 40.0 * zoom;
    (offset_x, offset_y, zoom)
}

// ============================================================
// AgentMessage — inter-agent message passing visualization
// ============================================================

#[derive(Clone, Debug)]
pub struct AgentMessage {
    pub from_node: String,
    pub to_node: String,
    pub content: String,
    pub timestamp: f64,
    pub message_type: MessageType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    Request,
    Response,
    Error,
    Info,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Request => write!(f, "REQ"),
            MessageType::Response => write!(f, "RSP"),
            MessageType::Error => write!(f, "ERR"),
            MessageType::Info => write!(f, "INF"),
        }
    }
}

impl Default for AgentMessage {
    fn default() -> Self {
        Self {
            from_node: String::new(),
            to_node: String::new(),
            content: String::new(),
            timestamp: 0.0,
            message_type: MessageType::Info,
        }
    }
}

// ============================================================
// NodeMetrics — per-node performance metrics
// ============================================================

#[derive(Clone, Debug, Default)]
pub struct NodeMetrics {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub total_cost: f64,
    pub avg_duration_ms: f64,
    pub last_duration_ms: f64,
    pub node_id: String,
    pub max_duration_ms: f64,
}

// ============================================================
// Helper: make a Rect
// ============================================================
fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

// ============================================================
// Restored Rendering Methods
// ============================================================
impl MultiAgentOrchestrator {
    fn render_output_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text("Node Output", x + 12.0, y + 8.0, 14.0, theme::text());
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                let output_text = if node.outputs.is_empty() {
                    "No output yet."
                } else {
                    &node.outputs[0]
                };
                renderer.draw_text(output_text, x + 12.0, y + 30.0, 12.0, theme::text_muted());
            }
        } else {
            renderer.draw_text(
                "Select a node to inspect output.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    fn render_message_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text("Agent Messages", x + 12.0, y + 8.0, 14.0, theme::text());
        let mut cy = y + 30.0;
        for msg in state.message_log.iter().rev().take(20) {
            let color = match msg.message_type {
                MessageType::Request => theme::accent(),
                MessageType::Response => [0.3, 1.0, 0.5, 1.0],
                MessageType::Error => theme::error_color(),
                MessageType::Info => theme::text_muted(),
            };
            let label = format!(
                "[{}] {} -> {}",
                msg.message_type, msg.from_node, msg.to_node
            );
            renderer.draw_text(&label, x + 12.0, cy, 11.0, color);
            cy += 16.0;
            renderer.draw_text(&msg.content, x + 20.0, cy, 10.0, theme::text_muted());
            cy += 20.0;
            if cy > y + h - 20.0 {
                break;
            }
        }
    }

    fn render_validation_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text("Validation", x + 12.0, y + 8.0, 14.0, theme::text());
        if state.validation_errors.is_empty() {
            renderer.draw_text(
                "No issues found.",
                x + 12.0,
                y + 30.0,
                12.0,
                [0.3, 1.0, 0.5, 1.0],
            );
        } else {
            let mut cy = y + 30.0;
            for err in &state.validation_errors {
                let color = if err.is_error {
                    theme::error_color()
                } else {
                    [1.0, 0.8, 0.2, 1.0]
                };
                renderer.draw_text(&err.message, x + 12.0, cy, 11.0, color);
                cy += 18.0;
            }
        }
    }

    fn render_skills_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text(
            "Skills Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text(
            "Available skills:",
            x + 12.0,
            y + 30.0,
            12.0,
            theme::text_muted(),
        );
        let mut cy = y + 50.0;
        for skill in state.skill_registry.list_skills() {
            renderer.draw_text(
                &format!("• {}", skill.name),
                x + 20.0,
                cy,
                11.0,
                theme::info(),
            );
            cy += 16.0;
            if cy > y + h - 20.0 {
                break;
            }
        }
    }

    fn render_webhook_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text(
            "Webhook Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                renderer.draw_text(
                    &format!("URL: {}", node.webhook_config.url),
                    x + 12.0,
                    y + 30.0,
                    11.0,
                    theme::text_muted(),
                );
                renderer.draw_text(
                    &format!("Method: {}", node.webhook_config.method),
                    x + 12.0,
                    y + 48.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        } else {
            renderer.draw_text(
                "Select a webhook node.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    fn render_schedule_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text(
            "Schedule Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                renderer.draw_text(
                    &format!("Cron: {}", node.schedule_config.cron_expression),
                    x + 12.0,
                    y + 30.0,
                    11.0,
                    theme::text_muted(),
                );
                renderer.draw_text(
                    &format!("Interval: {}s", node.schedule_config.interval_seconds),
                    x + 12.0,
                    y + 48.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        } else {
            renderer.draw_text(
                "Select a schedule node.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    fn render_recurring_panel(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 6.0, [0.08, 0.08, 0.12, 0.95]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 6.0, [0.3, 0.3, 0.5, 1.0], 1.0);
        renderer.draw_text("Recurring Runs", x + 12.0, y + 8.0, 14.0, theme::text());
        if state.recurring_runs.is_empty() {
            renderer.draw_text(
                "No recurring runs configured.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        } else {
            let mut cy = y + 30.0;
            for run in &state.recurring_runs {
                renderer.draw_text(
                    &format!("Every {}s ({} runs)", run.interval_seconds, run.run_count),
                    x + 12.0,
                    cy,
                    11.0,
                    theme::text_muted(),
                );
                cy += 18.0;
            }
        }
    }

    fn render_minimap(&self, renderer: &mut dyn Renderer, x: f32, y: f32, w: f32, h: f32) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), 4.0, [0.06, 0.06, 0.1, 0.9]);
        renderer.stroke_rounded_rect(r(x, y, w, h), 4.0, [0.25, 0.25, 0.4, 1.0], 1.0);
        let scale = 0.1;
        for node in &state.nodes {
            let nx = x + node.position.0 * scale;
            let ny = y + node.position.1 * scale;
            let nw = node.size.0 * scale;
            let nh = node.size.1 * scale;
            renderer.fill_rounded_rect(
                r(nx, ny, nw.max(2.0), nh.max(2.0)),
                1.0,
                [0.4, 0.4, 0.6, 0.8],
            );
        }
        let vp_x = x + state.viewport_offset.0 * scale;
        let vp_y = y + state.viewport_offset.1 * scale;
        let vp_w = 200.0 * state.viewport_zoom * scale;
        let vp_h = 150.0 * state.viewport_zoom * scale;
        renderer.stroke_rounded_rect(r(vp_x, vp_y, vp_w, vp_h), 1.0, [0.0, 1.0, 0.5, 0.5], 1.0);
    }
}

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::theme;

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
    /// Returns the user-facing label for the node type.
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

    /// Returns the theme color for this node type.
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Agent => theme::node_concept(),
            Self::Conditional => theme::node_relation(),
            Self::Loop => theme::node_context(),
            Self::Parallel => theme::node_entity(),
            Self::Webhook => theme::secondary(),
            Self::Schedule => theme::warning(),
            Self::DataSource => theme::text_muted(),
            Self::Sink => theme::text_muted(),
        }
    }

    /// Returns the symbol or abbreviation representing the node type.
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
    /// Returns the user-facing status label.
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

    /// Returns the status color.
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Pending => theme::status_waiting(),
            Self::Running => theme::status_running(),
            Self::Completed => theme::status_completed(),
            Self::Failed => theme::status_failed(),
            Self::Skipped => theme::text_muted(),
            Self::Retrying(_) => theme::warning(),
            Self::TimedOut => theme::error_color(),
            Self::Cancelled => theme::disabled(),
        }
    }

    /// Returns whether the status represents a final execution state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Skipped | Self::TimedOut | Self::Cancelled
        )
    }

    /// Returns whether the status represents an active execution state.
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
        match self {
            Self::Debug => theme::text_muted(),
            Self::Info => theme::info(),
            Self::Warn => theme::warning(),
            Self::Error => theme::error_color(),
        }
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
        match self {
            Self::Pending => theme::status_waiting(),
            Self::Running => theme::status_running(),
            Self::Completed => theme::status_completed(),
            Self::Failed => theme::status_failed(),
            Self::Cancelled => theme::text_muted(),
        }
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

// ============================================================
// AgentMessage -- inter-agent message passing visualization
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
// NodeMetrics -- per-node performance metrics
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

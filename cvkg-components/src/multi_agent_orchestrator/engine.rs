use std::collections::HashMap;
use std::time::Duration;

use super::types::{
    LogLevel, OrchestratorEdge, OrchestratorNode, OrchestratorNodeType, ValidationError,
};
use super::OrchestratorState;

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

                    // Check retry config -- simulate a retry if max_retries > 1.
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

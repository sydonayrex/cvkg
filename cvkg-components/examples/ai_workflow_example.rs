// CVKG AI Workflow Builder Example
// Demonstrates multi-agent orchestration and prompt chain visualization
//
// Run with: cargo run --example ai_workflow_example

use cvkg_components::{
    AIExecutionDebugger, AIWorkflowBuilder, AgentStatus, MemoryGraphViewer, MultiAgentPanel,
    NodeType, PromptChainVisualizer, PromptStatus, ReasoningTraceInspector, WorkflowNodeType,
};
use cvkg_core::{Rect, Renderer};

fn main() {
    println!("CVKG AI Workflow Builder Example");
    println!("================================\n");

    // Create a multi-agent panel showing agent orchestration
    let multi_agent = MultiAgentPanel::new("Research Team")
        .agent("agent_1", "Data Collector", AgentStatus::Running)
        .agent("agent_2", "Analyzer", AgentStatus::Running)
        .agent("agent_3", "Synthesizer", AgentStatus::Completed)
        .progress("agent_1", 0.75, "Collecting research papers");

    println!("Multi-Agent Panel: {}", multi_agent.title);
    println!("Agents: {}", multi_agent.agents.len());

    // Create a prompt chain visualizer
    let prompt_chain = PromptChainVisualizer::new()
        .step("s1", "System Prompt")
        .step("s2", "User Query")
        .step("s3", "Chain of Thought")
        .running("s2", 150)
        .completed("s1", 50, 245.5);

    println!("Prompt Chain Steps: {}", prompt_chain.prompts.len());

    // Create a memory graph viewer
    let memory_graph = MemoryGraphViewer::new()
        .node("n1", "Research", 0.8, NodeType::Concept)
        .node("n2", "Data", 0.6, NodeType::Entity)
        .node("n3", "Insights", 0.9, NodeType::Entity)
        .edge("n1", "n2", 0.7)
        .edge("n2", "n3", 0.9);

    println!("Memory Graph Nodes: {}", memory_graph.nodes.len());

    // Create a reasoning trace inspector
    let reasoning = ReasoningTraceInspector::new()
        .step(
            "step_1",
            "Problem Identification",
            0.95,
            "Identified the core issue",
        )
        .step(
            "step_2",
            "Approach Selection",
            0.85,
            "Selected analytical approach",
        );

    println!("Reasoning Steps: {}", reasoning.steps.len());

    // Create an AI workflow builder
    let workflow = AIWorkflowBuilder::new()
        .node("input", "Input", WorkflowNodeType::Input, (100.0, 200.0))
        .node(
            "process",
            "Process",
            WorkflowNodeType::Process,
            (300.0, 200.0),
        )
        .node("output", "Output", WorkflowNodeType::Output, (500.0, 200.0))
        .edge("input", "process")
        .edge("process", "output");

    println!("Workflow Nodes: {}", workflow.nodes.len());

    // Create an execution debugger
    let debugger = AIExecutionDebugger::new()
        .breakpoint("line_42")
        .breakpoint("line_87");

    println!("Breakpoints: {}", debugger.breakpoints.len());

    println!("\n=== AI Workflow Components Created Successfully ===");
}

# CVKG Evolution: Memory & Visuals Implementation Plan

This document outlines the strategic implementation path for enhancing the **CVKG** and **Agent Ulfhednar** ecosystems with advanced cognitive memory and high-fidelity tactical telemetry.

---

## 🧠 Phase 2: Cognitive Memory OS (TemporalGraph)

Based on the **Mnemosyne** and **PentAGI** architectures, this phase focuses on transforming flat memory fragments into a multi-layered cognitive engine.

### 1. Data Layer Infrastructure
- **PGVector Integration**: Transition from flat JSON storage to PostgreSQL with `pgvector` for high-performance semantic retrieval.
- **Layered Storage**:
    - **Episodic**: Raw mission events (short-term).
    - **Semantic**: Extracted facts and tactical intelligence (long-term).
    - **Procedural**: Successful command sequences and tool chains.

### 2. The Temporal Graph
- **Node-Edge Protocol**: Implement `TemporalNode` and `TemporalEdge` in `cvkg-core`.
- **Relationship Extraction**: Auto-generate edges between entities (e.g., *Service* -> *Vulnerability* -> *Exploit*).
- **Temporal Anchoring**: Allow the agent to "rewind" mission state by traversing nodes chronologically.

### 3. Cognitive Features
- **Activation Decay**: Implement a weighted decay algorithm where unused lore fragments drift out of the primary context window.
- **Reinforcement Learning**: Increase the "Importance Weight" of memory fragments that contribute to successful `RaidTask` completions.

---

## 📊 Phase 4: Visual Excellence & Telemetry

Inspired by **BugHunter-AI** and **Vibe-Kanban**, this phase elevates the operator interface to editorial-grade "Cyberpunk Viking" standards.

### 1. The Neural Vortex (Mimir's Well V2)
- **Force-Directed Layout**: Replace circular Lore Fragments with a dynamic graph visualization.
- **Bifrost Paths**: Implement animated glow-paths between connected memory nodes.
- **Gravitational Clustering**: Visually pull related tactical fragments together using a 2D physics engine.

### 2. Tactical HUD Gauges
- **Resource Probes**: Implement real-time CPU/RAM/Network monitoring for active agent processes.
- **Kinetics Dashboard**: Add high-fidelity gauges for:
    - **Neural Latency**: Round-trip time for LLM inferences.
    - **Mission Velocity**: Task completion rate over time.
    - **Context Saturation**: How much of the agent's context window is currently utilized.

### 3. Design System Consolidation
- **Tokens**: Formalize `Tactical Obsidian`, `Viking Gold`, and `Magenta Liquid` into a global `cvkg-theme` crate.
- **Clipped-Corner Nodes**: Replace legacy rectangles with tactical, editorial-grade SVG components.

---

> [!IMPORTANT]
> This plan is deferred until Phase 1 (Architecture) and Phase 3 (Self-Improvement) are stabilized.

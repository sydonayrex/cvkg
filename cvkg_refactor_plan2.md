# CVKG Agentic Refactor Plan - Improved

## 🔧 Core Problems Identified

### 1. No Error Boundaries (Critical UI Resilience Gap)
- Components must self-handle errors
- No automatic subtree fallback
- No isolation of failures

➡️ Leads to cascading UI failure in production

---

### 2. FAISS-Based Memory System (Scaling Bottleneck)
- Query latency grows with index size
- Prompt injection causes exponential growth
- Not aligned with CVKG frame pipeline
- External dependency creates integration complexity

➡️ Will break first at scale

---

# 🧠 Agentic Refactor Strategy (CVKG-Aligned)

## Phase 1 — Formalize Invariants

### Required System Invariants
- All mutations go through `state_queue`
- Frame-based deterministic execution
- No hidden side effects outside scheduler
- UI = pure function of state

### Agent Tasks
- Build machine-readable spec of invariants
- Scan codebase for violations
- Tag unsafe subsystems

---

# 🧱 FIX 1: Error Boundary System (CVKG-Patterned)

## Goal
Introduce fault isolation at component level using CVKG's state-driven approach

## Implementation Plan

### Step 1: Define Error State via Existing Patterns
```rust
// Use existing state pattern rather than new enum
#[state]
struct ComponentErrorState {
    has_error: bool,
    error_message: Option<String>,
    error_location: Option<String>,
}
```

### Step 2: Error Handling Through State Updates
- Components catch errors locally and update their error state
- Errors flow through normal state update mechanisms
- No direct mutation or panic propagation

### Step 3: Fallback UI Through Conditional Rendering
```rust
#[view]
fn MyComponent(state: State<MyComponentState>) -> impl View {
    if state.error_state.has_error {
        state.error_state.error_message.map(|msg| {
            Text(format!("Error: {}", msg))
                .color(Color::red())
        })
    } else {
        // Normal UI
        View::normal()
    }
}
```

### Step 4: Scheduler Integration (Natural Flow)
- Error state updates go through normal state update pipeline
- Benefit from batching, coalescing, frame alignment
- No special scheduler integration needed

### Step 5: DevTools Integration (Leverage Existing)
- Use existing State Inspector to view error states
- Time-travel debugging to see error introduction
- Frame Profiler to see impact on rendering

## Validation
- Component errors don't crash application
- Error states update deterministically
- Recovery works through normal state updates

---

# 🧠 FIX 2: State-Native Memory System (EXTRAS-Aligned)

## Migration Strategy
- Introduce `MemoryManager` as specialized state
- Use existing `[state]` and `[binding]` mechanisms
- Leverage frame scheduler for query processing

## Implementation Plan

### Step 1: Define Memory as Specialized State
```rust
// Use CVKG's existing state pattern
#[state]
struct KnowledgeState {
    fragments: HashMap<MemoryId, KnowledgeFragment>,
}

#[state]
struct KnowledgeFragment {
    summary: String,           // For prompt injection
    source: String,            // Original source/reference
    created_at: u64,           // Frame or timestamp
    accessed_count: u32,
    // Optional: compressed full content for on-demand retrieval
}
```

### Step 2: Frame-Aligned Memory Pipeline
```rust
// Memory query follows standard async pattern
spawn_scoped(view_id, async move {
    let query = prepare_memory_query(user_input);
    
    // Enqueue through scheduler - goes into state_queue
    scheduler.enqueue(move || {
        state.knowledge_state.process_query(query)
    });
});

// Processing happens in frame pipeline
impl KnowledgeState {
    fn process_query(&mut self, query: MemoryQuery) {
        // Search local fragments (could be enhanced with indexing)
        let results = self.fragments
            .iter()
            .filter(|(_, frag)| frag.is_relevant_to(&query))
            .take(5)  // Limit results
            .cloned()
            .collect::<Vec<_>>();
        
        // Store results as state update
        self.last_query_results = results;
    }
}
```

### Step 3: Prompt Integration (Sized Appropriately)
```rust
#[view]
fn AIComponent(state: State<AIState>) -> impl View {
    // Only inject summaries/references, not full content
    let memory_refs: Vec<String> = state.knowledge_state
        .last_query_results
        .iter()
        .map(|frag| format!("[Memory:{}]", frag.source))
        .collect();
    
    VStack {
        Text("Knowledge: ".to_string() + &memory_refs.join(" "))
        // Main AI content...
    }
}
```

### Step 4: On-Demand Full Retrieval
```rust
// When user wants details, use normal async flow
.on_click(|_| {
    spawn_scoped(view_id, async move {
        if let Some(fragment) = state.knowledge_state.get_selected() {
            // Could fetch full content from storage if needed
            let full_content = fetch_full_content(&fragment.source).await;
            scheduler.enqueue(move || {
                state.preview_state.set_content(full_content);
            });
        }
    });
})
```

### Step 5: Multi-Window Integration (CVKG Pattern)
```rust
// Global knowledge cache
#[global_state]
struct AppState {
    knowledge_cache: HashMap<MemoryId, String>,  // Summaries only
}

// Window-specific working state
#[state]
struct WindowState {
    active_queries: Vec<MemoryId>,
    previewed_fragment: Option<MemoryId>,
}
```

### Step 6: Leverage Existing Infrastructure
- **Inspector**: View knowledge state through State Inspector
- **Time-Travel**: See how knowledge state evolved
- **Frame Profiler**: Monitor memory query performance
- **Snapshot System**: Automatic persistence through existing mechanisms

## Validation
- Memory queries scale with frame batching
- Prompt size bounded by summary length
- Knowledge persistence through existing snapshots
- Full integration with DevTools for debugging

This approach eliminates the FAISS dependency while leveraging CVKG's proven state management, frame scheduling, and DevTools infrastructure for a more scalable, integrated solution.
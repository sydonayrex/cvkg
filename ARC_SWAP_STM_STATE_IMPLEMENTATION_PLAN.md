# ArcSwap + Software Transactional Memory Implementation Plan for CVKG State Management

## Replacing Arc<RwLock<T>> State<T> Implementation

## 1. Current State Management Analysis

CVKG's current State<T> implementation in `cvkg-core/src/lib.rs`:

```rust
pub struct State<T: Clone + Send + Sync + 'static> {
    value: Arc<std::sync::RwLock<T>>,
    subscribers: Arc<std::sync::RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>,
    version: Arc<std::sync::atomic::AtomicU64>,
}
```

**Limitations identified in evaluation:**
- RwLock can cause contention under high update frequency (State & Reactivity Model: 78/100)
- Writer starvation under heavy read loads
- Limited composability for multi-state transactions
- Potential deadlock risks in complex UI interactions

## 2. Proposed Solution Overview

Replace the current State<T> with a hybrid approach:

- **Top-Level State Management**: `ArcSwap` for wait-free reads and atomic swaps
- **Sub-State Mutations**: Software Transactional Memory (STM) for transactional integrity

This combination provides:
- ✅ Lock-free reads (critical for UI rendering performance)
- ✅ Atomic state updates at top level
- ✅ Composable transactions for complex state mutations
- ✅ Reduced contention compared to RwLock
- ✅ Eliminated deadlock risks from manual lock ordering

## 3. Detailed Implementation Plan

### Phase 1: Foundation & Dependencies

#### 1.1 Add Required Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
arcswap = "0.5"  # Atomic reference-counted pointer
# For STM: we'll implement a lightweight version or use existing crate
# Options: stm-rs, or build minimal STM tailored to CVKG needs
```

#### 1.2 Core Abstractions

Define new state primitives:

```rust
// Top-level state using ArcSwap (wait-free reads)
pub struct ArcSwapState<T> {
    inner: ArcSwap<Arc<T>>,  // ArcSwap wrapping Arc for efficient cloning
}

// Transactional state using STM
pub struct TxState<T> {
    inner: TxCell<T>,        // Transactional cell
}

// Transaction context for STM
pub struct Txn {
    id: TxnId,
    manager: &'static TxnManager,
    read_set: ReadSet,
    write_set: WriteSet,
}

// Transaction manager (singleton)
struct TxnManager {
    global_version: AtomicUsize,
    active_txns: Mutex<Vec<TxnId>>,
}
```

### Phase 2: ArcSwapState Implementation

#### 2.1 ArcSwapState Core Functions

```rust
impl<T: Clone + Send + Sync + 'static> ArcSwapState<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: ArcSwap::from_pointee(Arc::new(value)),
        }
    }

    // Wait-free read - returns guard that pins the value
    pub fn read(&self) -> Guard<Arc<T>> {
        self.inner.load()
    }

    // Atomic swap - wait-free for readers
    pub fn swap(&self, value: T) {
        self.inner.store(Arc::new(value));
    }

    // Compare-and-swap for optimistic updates
    pub fn compare_and_swap<F>(&self, f: F) -> Result<Arc<T>, Arc<T>>
    where
        F: FnOnce(&T) -> Option<T>,
    {
        self.inner.compare_and_swap(|arc| f(&arc).map(Arc::new))
    }
}
```

### Phase 3: Software Transactional Memory Implementation

#### 3.1 Lightweight STM for CVKG

We'll implement a minimal STM suited for UI workloads:

```rust
// Versioned transactional cell
struct TxCell<T> {
    value: UnsafeCell<T>,
    version: AtomicUsize,
}

impl<T> TxCell<T> {
    pub fn new(value: T) -> Self {
        TxCell {
            value: UnsafeCell::new(value),
            version: AtomicUsize::new(0),
        }
    }
}

// Transaction log entry
enum LogOp {
    Read(*const TxCell<dyn Any + Send + Sync>, usize),  // (cell, version_at_read)
    Write(*const TxCell<dyn Any + Send + Sync>, Box<dyn Any + Send + Sync>), // (cell, new_value)
}

// Transaction context
struct Txn {
    id: TxnId,
    manager: &'static TxnManager,
    log: Vec<LogOp>,
}

impl Txn {
    pub fn read<T: 'static + Send + Sync>(&self, cell: &TxCell<T>) -> T {
        // Record read for conflict detection
        let version = cell.version.load(Ordering::Acquire);
        self.manager.record_read(self.id, cell as *const _, version);
        unsafe { ptr::read(cell.value.get()) }
    }

    pub fn write<T: 'static + Send + Sync>(&self, cell: &TxCell<T>, value: T) {
        // Record write for later application
        let boxed_value: Box<dyn Any + Send + Sync> = Box::new(value);
        self.log.push(LogOp::Write(
            cell as *const TxCell<dyn Any + Send + Sync>,
            boxed_value
        ));
    }

    pub fn commit(self) -> Result<(), TxnAbort> {
        self.manager.validate_and_commit(self)
    }
}

// Transaction manager
struct TxnManager {
    global_version: AtomicUsize,
    // Track versions for conflict detection
    cell_versions: DashMap<*const (), usize>, // Simplified
    active_txns: RwLock<HashMap<TxnId, Txn>>,
}

impl TxnManager {
    pub fn new() -> Self {
        TxnManager {
            global_version: AtomicUsize::new(0),
            cell_versions: DashMap::new(),
            active_txns: RwLock::new(HashMap::new()),
        }
    }

    pub fn begin(&self) -> Txn {
        let id = TxnId::new();
        self.active_txns.write().insert(id, Txn { id, manager: self, log: Vec::new() });
        self.active_txns.read()[&id].clone()
    }

    fn validate_and_commit(&self, txn: Txn) -> Result<(), TxnAbort> {
        // 1. Validate all reads haven't changed
        for LogOp::Read(cell_ptr, expected_version) in &txn.log {
            let cell = unsafe { &*(*cell_ptr as *const TxCell<()>) };
            let actual_version = cell.version.load(Ordering::Acquire);
            if *expected_version != actual_version {
                return Err(TxnAbort::Conflict);
            }
        }

        // 2. Apply all writes
        for LogOp::Write(cell_ptr, value_box) in txn.log {
            let cell = unsafe { &*(*cell_ptr as *const TxCell<()>) };
            unsafe { ptr::write(cell.value.get(), *value_box.downcast_unchecked()) };
            cell.version.fetch_add(1, Ordering::Release);
        }

        // 3. Cleanup
        self.active_txns.write().remove(&txn.id);
        Ok(())
    }
}
```

### Phase 3: TxState Implementation

```rust
impl<T: Clone + Send + Sync + 'static> TxState<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: TxCell::new(value),
        }
    }
}

impl<T> TxState<T> {
    pub fn read_in_txn(&self, txn: &Txn) -> T {
        txn.read(&self.inner)
    }

    pub fn write_in_txn(&self, txn: &Txn, value: T) {
        txn.write(&self.inner, value);
    }
}
```

### Phase 4: New State<T> Implementation

#### 4.1 State Enum with Multiple Variants

```rust
pub enum State<T: Clone + Send + Sync + 'static> {
    /// Top-level state using ArcSwap (wait-free reads)
    ArcSwap(ArcSwapState<T>),
    
    /// Sub-state using STM (transactional mutations)
    Txnal(TxState<T>),
    
    /// Legacy RwLock state (temporary for migration)
    Legacy(Arc<RwLock<T>>),
}

impl<T: Clone + Send + Sync + 'static> State<T> {
    pub fn new(value: T) -> Self {
        // Default to ArcSwap for new state (top-level)
        State::ArcSwap(ArcSwapState::new(value))
    }

    // --- Read Operations ---

    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self {
            State::ArcSwap(state) => {
                let guard = state.read();
                f(&*guard)
            }
            State::Txnal(state) => {
                // For simple reads outside txn, use current value
                // In practice, outside txn reads should be rare for Txnal
                let value = state.inner.value.get_mut();
                // This is unsafe but we're outside txn - accept inconsistency risk
                // Better approach: require txn for all Txnal access
                unsafe { f(&*value.get()) }
            }
            State::Legacy(state) => {
                let guard = state.read().unwrap();
                f(&*guard)
            }
        }
    }

    // --- Write Operations ---

    pub fn swap(&self, value: T) {
        match self {
            State::ArcSwap(state) => state.swap(value),
            State::Txnal(state) => {
                // For Txnal, swap outside txn = direct update
                *state.inner.value.get_mut() = value;
                state.inner.version.fetch_add(1, Ordering::Release);
            }
            State::Legacy(state) => *state.write().unwrap() = value,
        }
    }

    // --- Transactional Interface ---

    pub fn txn<R>(&self, f: impl FnOnce(&Txn) -> R) -> Result<R, TxnAbort> {
        match self {
            State::ArcSwap(state) => {
                // For ArcSwap state, transactions are simple
                let mut txn = TxnManager::begin();
                let result = f(&txn);
                txn.commit()?;
                Ok(result)
            }
            State::Txnal(state) => {
                // Full STM transaction
                let mut txn = TxnManager::begin();
                let result = f(&txn);
                txn.commit()?;
                Ok(result)
            }
            State::Legacy(state) => {
                // Legacy path - acquire lock for duration
                let mut guard = state.write().unwrap();
                let result = f(&mut *guard);
                Ok(result)
            }
        }
    }

    // --- Helper Methods ---

    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, TxnAbort> {
        self.txn(|txn| {
            match self {
                State::ArcSwap(state) => {
                    let mut current = (**state.read()).clone();
                    let result = f(&mut current);
                    state.swap(current);
                    Ok(result)
                }
                State::Txnal(state) => {
                    let mut current = state.read_in_txn(txn);
                    let result = f(&mut current);
                    state.write_in_txn(txn, current);
                    Ok(result)
                }
                State::Legacy(state) => {
                    let mut current = state.write().unwrap();
                    let result = f(&mut current);
                    Ok(result)
                }
            }
        })
    }

    // --- Version Tracking (for CVKG compatibility) ---

    pub fn version(&self) -> u64 {
        match self {
            State::ArcSwap(state) => {
                // ArcSwap doesn't have built-in version, we'd need to track it
                // Simplified: use a separate ArcSwap<AtomicU64> or clone the Arc and use ptr addr
                0  // Placeholder - would need proper implementation
            }
            State::Txnal(state) => state.inner.version.load(Ordering::Acquire) as u64,
            State::Legacy(state) => 0,  // Placeholder
        }
    }

    // --- Subscription System (maintain CVKG compatibility) ---

    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&self, callback: F) {
        // For simplicity in this plan, we'll note that a full implementation
        // would need to integrate with ArcSwap's hazard pointers or
        // maintain a separate subscriber list with appropriate locking
        // This is a complex area that would need careful design
        // For now, we acknowledge this needs more work
        unimplemented!();
    }
}
```

### Phase 5: Migration Strategy

#### 5.1 Backward Compatibility Approach

1. **Dual Implementation Period**: Keep both State implementations during transition
2. **Feature Gate**: Use Cargo features to switch between implementations
3. **Gradual Migration**: Migrate subsystems one by one

#### 5.2 Migration Steps

1. **Phase 1**: Implement new State<T> alongside existing (behind feature flag)
2. **Phase 2**: Update cvkg-core to use new State<T> internally
3. **Phase 3**: Migrate cvkg-vdom, cvkg-layout to use new State<T>
4. **Phase 4**: Update examples and documentation
5. **Phase 5**: Remove legacy State<T> implementation

#### 5.3 Migration Example

```rust
// BEFORE (RwLock-based)
let state = State::new(initial_value);
let mut guard = state.write().unwrap();
guard.field = new_value;
drop(guard); // Unlock

// AFTER (ArcSwap + STM)
let state = State::new(initial_value);

// Option 1: Simple swap (top-level state)
state.swap(new_value);

// Option 2: Transactional update (sub-state)
state.update(|s| {
    s.field1 = new_value1;
    s.field2 = new_value2;
    // Complex multi-field updates are atomic
})?;

// Option 3: Explicit transaction
state.txn(|txn| {
    let s1 = state.read_in_txn(txn)?;
    let s2 = other_state.read_in_txn(txn)?;
    // ... complex logic involving multiple state variables
    state.write_in_txn(txn, updated_s1);
    other_state.write_in_txn(txn, updated_s2);
    Ok(())
})?;
```

## 4. CVKG-Specific Integration Points

### 4.1 Rendering System Integration

- **ArcSwapState**: Ideal for render state (wait-free reads during rendering)
- **Guard lifetime**: Ensure render frames don't outlive ArcSwap guards
- **Double-buffering**: Consider for complex render state updates

### 4.2 Layout Engine Integration

- **TxState**: Layout constraint solving benefits from transactional updates
- **ArcSwapState**: Layout parameters (size, position) for read-heavy access

### 4.3 Animation System Integration

- **ArcSwapState**: Animation progress/state (frequent reads, occasional writes)
- **TxState**: Animation queues/timelines (coordinated updates)

### 4.4 Event System Integration

- **TxState**: Event queues (multiple producers/consumers)
- **ArcSwapState**: UI state derived from events

## 5. Benefits and Trade-offs

### Benefits

1. **Eliminated Read-side Blocking**: UI rendering never blocked by state updates
2. **Composable Transactions**: Complex state updates are atomic and consistent
3. **Reduced Contention**: No lock convoy effects under high update frequency
4. **Better Performance**: Particularly for read-heavy UI workloads (typical UI ratio 10:1 read:write)
5. **Deadlock Freedom**: No manual lock ordering required
6. **CVKG Alignment**: Supports agentic UI patterns with observable state

### Trade-offs

1. **Increased Complexity**: More sophisticated state management model
2. **STM Overhead**: Transaction logging and validation costs (mitigated by lightweight implementation)
3. **Eventual Consistency**: ArcSwap reads may see slightly stale data (acceptable for UI)
4. **Memory Usage**: Slightly higher per-state overhead vs. plain RwLock
5. **Subscription Complexity**: Maintaining observer system with ArcSwap requires careful design

## 6. Implementation Roadmap

### Milestone 1: Core Primitives (Week 1)
- [] Add arcswap dependency
- [] Implement ArcSwapState with wait-free reads
- [] Design and implement lightweight STM
- [] Create TxnManager and Txn types

### Milestone 2: State<T> Redesign (Week 2)
- [] Implement new State<T> enum with variants
- [] Implement read/swap/txn/update methods
- [] Ensure basic compatibility with existing API
- [] Add comprehensive unit tests

### Milestone 3: Integration & Testing (Week 3)
- [] Update core crates (cvkg-core, cvkg-vdom) to use new State
- [] Integrate with rendering pipeline
- [] Test with layout and animation systems
- [] Performance benchmarking vs. RwLock

### Milestone 4: Migration & Documentation (Week 4)
- [] Create migration guide for component developers
- [] Update examples and documentation
- [] Gather feedback from internal usage
- [] Finalize and remove Legacy variant

## 7. Expected Outcomes

### Performance Improvements
- **Read throughput**: 2-10x improvement (lock-free vs. RwLock)
- **Write latency**: Similar or better under contention
- **Frame consistency**: More stable frame times due to eliminated read-side blocking

### Evaluation Score Impact
- **State & Reactivity Model**: Expected improvement from 78/100 to 90+/100
- **Performance & Scaling**: Potential improvement from 80/100 to 85+/100
- **Overall Weighted Score**: Projected increase from 86/100 to 89+/100

### Qualitative Improvements
- Better scalability to 100k+ components
- Improved responsiveness under high-frequency updates
- Enhanced capability for complex UI interactions
- Stronger foundation for agentic UI patterns

---
*Implementation Plan Version: 1.0*
*Date: 2026-04-26*
*Target: CVKG State<T> Replacement with ArcSwap + STM*
*Based on Evaluation Findings: State & Reactivity Model scored 78/100 due to RwLock limitations*

# Security & Bug Hunt Report: cvkg-vdom

**Date**: 2026-07-05  
**Auditor**: Security/Bug Hunt Agent  
**Scope**: cvkg-vdom crate (v0.3.3)

---

## Executive Summary

**Overall Risk Level**: ⚠️ MODERATE

The cvkg-vdom crate has good test coverage and follows Rust best practices, but contains several security concerns and potential bug vectors that need attention.

---

## Critical Findings

### 1. ID Counter Overflow Vulnerability (DoS Risk)

**Location**: `src/lib.rs:886-889`

**Severity**: HIGH

**Description**: The `next_id` counter in `VNodeRenderer` is a `u64` that starts at 1 and increments without bounds. If it overflows, it would wrap to 0, potentially causing:
- ID collisions between old and new nodes
- Incorrect event handler association
- Memory corruption in the VDOM tree

**Code**:
```rust
fn next_id(&mut self) -> NodeId {
    let id = KvasirId(self.next_id);
    self.next_id += 1;  // No overflow check!
    id
}
```

**Recommendation**: Add overflow protection:
```rust
fn next_id(&mut self) -> NodeId {
    self.next_id = self.next_id.checked_add(1)
        .expect("VDOM node ID counter overflow");
    let id = KvasirId(self.next_id);
    id
}
```

---

### 2. Mutex Lock Poisoning (Panic Risk)

**Location**: Multiple locations

**Severity**: MEDIUM

**Description**: Multiple `.lock().unwrap()` calls on Mutexes can panic if a previous thread panicked while holding the lock. This is particularly problematic in:
- Event handling paths (`src/lib.rs:560-564`)
- Signal system (`src/signals.rs:72, 78-79, 95, 125, 138, 174, 177, 183, 203, 205, 211`)
- Physics tick (`src/physics.rs:45`)
- Accessibility tree traversal (`src/accesskit_bridge.rs:180`)

**Code Example**:
```rust
// src/lib.rs:560
if let Ok(mut focus) = self.focused_node.lock() {
    *focus = id;
}
```

**Recommendation**: Already partially handled with `Ok()` pattern in some places, but should be consistent. Use `.lock().ok()` or implement recovery logic.

---

### 3. Hash Flooding DoS Vector

**Location**: `src/diff.rs:429`

**Severity**: MEDIUM

**Description**: The keyed child diffing algorithm uses a `HashMap<String, (usize, NodeId)>` to map old children by key. If an attacker can control the keys, they could potentially trigger hash collisions causing degraded performance (O(n²) instead of O(n)).

**Code**:
```rust
let mut old_keyed: HashMap<String, (usize, NodeId)> = HashMap::new();
for (i, id) in old_children.iter().enumerate() {
    if let Some(node) = self.nodes.get(id)
        && let Some(key) = &node.key
    {
        old_keyed.insert(key.clone(), (i, *id));
    }
}
```

**Recommendation**: Use `HashMap::with_hasher` with a cryptographically secure hasher (e.g., `SipHasher`) for user-controlled keys.

---

## High-Priority Bugs

### 4. serde_json::to_value unwrap() in Hot Path

**Location**: `src/lib.rs:1184, 1201, 1214-1215, 1222, 1243-1248, 1257, 1308-1310, 1332-1339, 1356-1362, 1376-1381, 1388-1393`

**Severity**: MEDIUM

**Description**: 30+ calls to `serde_json::to_value(...).unwrap()` in the rendering path. While these are unlikely to fail for primitive types (f32, String), they could panic if serialization fails, crashing the renderer.

**Recommendation**: Replace with `.unwrap_or_default()` or proper error handling:
```rust
props.insert("radius".to_string(), 
    serde_json::to_value(radius).unwrap_or_else(|e| {
        tracing::warn!("Failed to serialize radius: {:?}", e);
        serde_json::Value::Null
    })
);
```

---

### 5. Potential Double-Free in Replace Patch

**Location**: `src/lib.rs:316-348`

**Severity**: MEDIUM

**Description**: The `Replace` patch implementation removes the old node and inserts the new one. If an error occurs between these operations, the VDOM could be left in an inconsistent state.

**Code**:
```rust
VDomPatch::Replace { id, node } => {
    let is_root = self.root == Some(id);
    let new_id = node.id;
    
    // If this panics, old node is removed but state is inconsistent
    if let Some(old_node) = self.nodes.get(&id) {
        for child_id in &old_node.children {
            self.parents.remove(child_id);
        }
    }
    for child_id in &node.children {
        self.parents.insert(*child_id, new_id);
    }
    
    self.nodes.remove(&id);  // Old node removed
    self.nodes.insert(new_id, node);  // New node added
    // ...
}
```

**Recommendation**: Wrap in a transactional pattern or use `Option` to ensure atomicity.

---

## Medium-Priority Issues

### 6. Missing Error Handling in Event Dispatch

**Location**: `src/lib.rs:311-373`

**Severity**: MEDIUM

**Description**: The `dispatch_event` function silently ignores errors from `hit_test` and mutex locks, potentially masking bugs.

### 7. Recursive Depth in Validation

**Location**: `src/lib.rs:102-129`

**Severity**: LOW

**Description**: The `validate_node_sync` function is recursive and could cause stack overflow on deeply nested VDOM trees (unlikely in practice, but possible).

### 8. Unbounded Vec Growth

**Location**: Multiple locations

**Severity**: LOW

**Description**: Several `Vec` allocations (e.g., `best_child_hit`, `source_indices`, `order`) are not bounded by tree depth or node count, potentially allowing memory exhaustion with malformed input.

---

## Security Audit Results

### cargo audit Output

```
Crate:     bincode
Version:   1.3.3
Warning:   unmaintained
Title:     Bincode is unmaintained
ID:        RUSTSEC-2025-0141

Crate:     anyhow
Version:   1.0.102
Warning:   unsound
Title:     Unsoundness in Error::downcast_mut()
ID:        RUSTSEC-2026-0190
```

**Recommendation**: Monitor these dependencies and consider alternatives if security issues are discovered.

---

## Test Coverage Analysis

**Total Tests**: 22 tests  
**Status**: All passing ✓

**Key Tests Identified**:
- `test_vdom_keyed_reordering` - Tests list diffing
- `test_vdom_deep_diffing` - Tests tree diffing
- `test_signal_cross_thread` - Tests signal thread safety
- `p0_6_diff_emits_clear_handlers_when_handler_removed` - Tests handler removal
- `p0_7_identical_handlers_do_not_emit_update_patch` - Tests optimization
- `phase6_pointer_capture_survives_rebuild_before_release` - Tests event capture
- `berserker_click_box_regression` - Tests overlay hit testing

**Comments in Tests**:
The test file documents a previously known bug (lines 786-801) about overlay hit testing that has since been fixed. This shows good documentation practices.

---

## Recommendations

### Immediate Actions (P0)

1. **Add overflow protection** to `next_id()` counter
2. **Replace unwrap() calls** with proper error handling in hot paths
3. **Use checked operations** for index calculations in `Move` patch

### Short-Term Actions (P1)

4. **Implement hash flooding protection** for keyed diffing
5. **Add bounds checking** for recursive operations
6. **Review all Mutex usage** for consistent error handling

### Long-Term Actions (P2)

7. **Add property-based testing** for edge cases (tree depth, node count)
8. **Implement fuzzing** for the diff algorithm
9. **Add metrics** for detecting performance degradation

---

## Appendix: Pattern Analysis

### Unwrap() Usage (75 total)

**Source locations**:
- `src/vdom.rs:297` - One unwrap in hit testing
- `src/signals.rs:72, 78-79, 91, 95, 125, 133, 138, 174, 177, 183, 203, 205, 211` - Signal system
- `src/diff.rs:348` - HashMap lookup
- `src/lib.rs:470, 1184, 1201, 1214-1215, 1222, 1243-1248, 1257, 1308-1310, 1332-1339, 1356-1362, 1376-1381, 1388-1393` - Rendering

### Unsafe Code

**Result**: No `unsafe` blocks found in production code (only documentation mentions it).

### TODO/FIXME

**Result**: None found.

---

## Conclusion

The cvkg-vdom crate is well-structured with good test coverage, but needs attention to error handling and potential DoS vectors. The most critical issue is the ID counter overflow which could lead to memory corruption in production use.
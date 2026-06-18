//! Task scheduling primitives for CVKG's frame pipeline.
//!
//! # Why this module exists
//! Without a centralized task queue, every CVKG subsystem (VDOM, Layout, Animation,
//! Physics, Render, Telemetry) invokes work immediately and in arbitrary order.
//! This module provides a priority-ordered queue so that Critical tasks always
//! run before High, Normal, and Idle tasks — matching the documented frame pipeline.

use cvkg_core::KvasirId;

// =============================================================================
// Priority
// =============================================================================

/// Execution priority for a scheduled task.
///
/// # Why four levels?
/// The four levels map directly onto the four stages of the CVKG frame pipeline:
/// - `Critical` blocks the frame — input and state resolution must finish first.
/// - `High` is required for visual correctness (layout/animation feed the render).
/// - `Normal` is unconstrained general work (component state updates, etc.).
/// - `Idle` is deferred work that runs in leftover frame time (telemetry, prefetch).
///
/// The `u8` discriminant allows numeric comparison: lower value = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Priority {
    /// Must complete before the frame proceeds (input processing, state resolution).
    Critical = 0,
    /// Required for visual correctness — layout and animation.
    High = 1,
    /// General work — component updates, event handling.
    Normal = 2,
    /// Deferred work — telemetry flush, asset prefetch.
    Idle = 3,
}

// =============================================================================
// Task
// =============================================================================

/// A single unit of scheduled work with an associated priority.
///
/// # Why `Box<dyn FnOnce() + Send>`?
/// Tasks are submitted by heterogeneous callers (VDOM, Layout, Physics, etc.) at
/// different priority levels. The trait object allows arbitrary closures to be stored
/// in the same queue without generics, while `Send` ensures the scheduler can hand
/// tasks to a worker thread if needed in the future.
///
/// # Contract
/// - `work` is consumed exactly once when the task is executed.
/// - `id` is globally unique (via `KvasirId::new()`).
pub struct Task {
    /// Globally unique task identifier. Used for cancellation.
    pub id: KvasirId,
    /// Execution priority — lower discriminant runs first.
    pub priority: Priority,
    /// Human-readable label for debugging and profiling.
    pub name: &'static str,
    /// The closure to execute. Consumed on first call.
    pub work: Box<dyn FnOnce() + Send>,
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("name", &self.name)
            .finish()
    }
}

// =============================================================================
// TaskHandle
// =============================================================================

/// An opaque handle returned by [`TaskScheduler::submit`] that allows the caller
/// to cancel a pending task before it runs.
///
/// # Why a separate handle type?
/// The handle decouples the cancellation token from the task itself so callers
/// cannot accidentally double-cancel or inspect internal task state. Only the `id`
/// field is needed for the scheduler to locate and remove the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskHandle {
    /// The `KvasirId` of the task this handle refers to.
    pub id: KvasirId,
}

// =============================================================================
// TaskScheduler
// =============================================================================

/// A priority-ordered task queue that executes work in the correct frame order.
///
/// # Why a `Vec` with sort-on-drain rather than a `BinaryHeap`?
/// `BinaryHeap` would give O(log n) insertion and O(log n) pop, but heap elements
/// need `Ord` on the whole `Task` struct, which requires comparing `Box<dyn Fn…>`.
/// A `Vec` lets us store heterogeneous closures directly and sort once on drain —
/// acceptable for the typical frame budget of < 64 tasks per flush.
///
/// # Contract
/// - Tasks are not re-ordered within the same priority level (stable sort preserves
///   insertion order for equal priorities).
/// - Cancelled tasks are removed in O(n) time.
/// - `run_all` leaves the queue empty after it returns.
#[derive(Default)]
pub struct TaskScheduler {
    /// Pending tasks. Sorted by priority during drain.
    queue: Vec<Task>,
}

impl TaskScheduler {
    /// Create an empty `TaskScheduler`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a new task to the queue and return a handle for potential cancellation.
    ///
    /// # Why `impl FnOnce() + Send + 'static`?
    /// The bound matches `Box<dyn FnOnce() + Send>` requirements while letting
    /// the compiler reject accidentally non-`Send` captures at the call site.
    ///
    /// # Contract
    /// The returned `TaskHandle::id` is unique and matches the stored task's `id`.
    pub fn submit(
        &mut self,
        priority: Priority,
        name: &'static str,
        work: impl FnOnce() + Send + 'static,
    ) -> TaskHandle {
        let id = KvasirId::new();
        self.queue.push(Task {
            id,
            priority,
            name,
            work: Box::new(work),
        });
        TaskHandle { id }
    }

    /// Drain and execute all pending tasks in priority order (Critical → Idle).
    ///
    /// # Why sort-on-drain?
    /// Tasks are submitted throughout the frame tick; sorting once at flush time
    /// is cheaper than maintaining heap invariants on every submit.
    ///
    /// # Contract
    /// After this call the queue is empty. If a task panics, subsequent tasks
    /// in the drained batch are still lost (caller should use catch_unwind if needed).
    pub fn run_all(&mut self) {
        // Stable sort: equal-priority tasks run in insertion order.
        self.queue.sort_by_key(|t| t.priority);
        let tasks: Vec<Task> = std::mem::take(&mut self.queue);
        for task in tasks {
            log::trace!("scheduler: running task '{}' ({:?})", task.name, task.priority);
            (task.work)();
        }
    }

    /// Drain and execute only tasks whose priority is at or above `min_priority`.
    ///
    /// # Why this exists?
    /// Some frame phases only need to flush Critical + High work and want to
    /// defer Normal and Idle tasks to a later phase. This avoids a full flush
    /// while still running time-sensitive tasks on schedule.
    ///
    /// # Contract
    /// Tasks not matching the filter remain in the queue and will be executed
    /// on a subsequent `run_all` or `run_priority` call.
    pub fn run_priority(&mut self, min_priority: Priority) {
        // Partition: extract matching tasks, keep the rest.
        let mut remaining = Vec::with_capacity(self.queue.len());
        let mut to_run = Vec::new();

        for task in std::mem::take(&mut self.queue) {
            if task.priority <= min_priority {
                to_run.push(task);
            } else {
                remaining.push(task);
            }
        }

        self.queue = remaining;

        to_run.sort_by_key(|t| t.priority);
        for task in to_run {
            log::trace!(
                "scheduler: running priority-filtered task '{}' ({:?})",
                task.name,
                task.priority
            );
            (task.work)();
        }
    }

    /// Return the number of tasks currently pending in the queue.
    ///
    /// # Contract
    /// Returns 0 immediately after `run_all` returns.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Remove a pending task by its handle without executing it.
    ///
    /// # Why cancellation?
    /// Long-lived systems (e.g. a streaming data source) may submit speculative
    /// Idle tasks during a frame and need to retract them if the source becomes
    /// unavailable before the frame flushes.
    ///
    /// # Contract
    /// If the task has already been executed (removed by `run_all`), this is a no-op.
    /// If multiple tasks somehow share the same id (should not happen), all are removed.
    pub fn cancel(&mut self, handle: &TaskHandle) {
        self.queue.retain(|t| t.id != handle.id);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_submit_increments_pending_count() {
        let mut s = TaskScheduler::new();
        assert_eq!(s.pending_count(), 0);
        s.submit(Priority::Normal, "a", || {});
        assert_eq!(s.pending_count(), 1);
        s.submit(Priority::High, "b", || {});
        assert_eq!(s.pending_count(), 2);
    }

    #[test]
    fn test_run_all_executes_in_priority_order() {
        let mut s = TaskScheduler::new();
        let order: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        let o1 = order.clone();
        s.submit(Priority::Idle, "idle", move || o1.lock().unwrap().push("idle"));

        let o2 = order.clone();
        s.submit(Priority::Critical, "critical", move || o2.lock().unwrap().push("critical"));

        let o3 = order.clone();
        s.submit(Priority::Normal, "normal", move || o3.lock().unwrap().push("normal"));

        let o4 = order.clone();
        s.submit(Priority::High, "high", move || o4.lock().unwrap().push("high"));

        s.run_all();

        let result = order.lock().unwrap().clone();
        assert_eq!(result, vec!["critical", "high", "normal", "idle"]);
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn test_run_priority_only_runs_matching_tasks() {
        let mut s = TaskScheduler::new();
        let ran: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        let r1 = ran.clone();
        s.submit(Priority::Critical, "c", move || r1.lock().unwrap().push("critical"));

        let r2 = ran.clone();
        s.submit(Priority::Idle, "i", move || r2.lock().unwrap().push("idle"));

        // Only run Critical and above.
        s.run_priority(Priority::Critical);

        let result = ran.lock().unwrap().clone();
        assert_eq!(result, vec!["critical"]);
        // Idle task still pending.
        assert_eq!(s.pending_count(), 1);
    }

    #[test]
    fn test_cancel_removes_task_before_run() {
        let mut s = TaskScheduler::new();
        let ran = Arc::new(Mutex::new(false));

        let r = ran.clone();
        let handle = s.submit(Priority::Normal, "cancelme", move || {
            *r.lock().unwrap() = true;
        });

        assert_eq!(s.pending_count(), 1);
        s.cancel(&handle);
        assert_eq!(s.pending_count(), 0);

        s.run_all();
        assert!(!*ran.lock().unwrap(), "cancelled task should not have run");
    }

    #[test]
    fn test_cancel_noop_after_run() {
        let mut s = TaskScheduler::new();
        let handle = s.submit(Priority::Normal, "already_run", || {});
        s.run_all();
        // Should be a no-op, not panic.
        s.cancel(&handle);
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        // Verify that Priority enum ordering is correct (Critical < High < Normal < Idle).
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Idle);
    }
}

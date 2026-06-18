//! Frame scheduling for CVKG's per-frame pipeline.
//!
//! # Why this module exists
//! A frame in CVKG is not a single monolithic update — it is a sequence of
//! ordered phases (Input → State → Layout → Animation → Render → Composite → PostFrame).
//! Violating this order causes visual tearing (render before layout), stale data
//! (animation before state), or wasted GPU work (composite before render).
//!
//! `FrameScheduler` owns a phase-keyed work queue and ensures each phase's tasks
//! run exactly once when `flush_current_phase` is called for that phase.
//!
//! # Design note
//! Phase tasks are stored directly in the `FrameScheduler`'s own `phase_queue`
//! (not routed through the inner `TaskScheduler`) because cancelling from the
//! `TaskScheduler` would irrecoverably consume the closure. The inner
//! `TaskScheduler` is exposed as a convenience for callers that want to submit
//! ad-hoc priority-ordered work within a phase flush.

use crate::task::{Priority, TaskHandle, TaskScheduler};
use cvkg_core::KvasirId;

// =============================================================================
// FramePhase
// =============================================================================

/// The ordered phases of a single CVKG render frame.
///
/// # Why enumerate phases?
/// Explicit phases give each subsystem a named slot in the frame timeline.
/// Code that submits work for `Layout` cannot accidentally run before `State`
/// is resolved, and code that submits for `Render` cannot run before `Layout`
/// and `Animation` are complete.
///
/// # Ordering contract
/// Phases advance monotonically within a frame via [`FrameScheduler::advance_phase`].
/// Returning to an earlier phase within the same frame is not permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum FramePhase {
    /// Raw input events are collected and dispatched (pointer, keyboard, touch).
    Input,
    /// Application state is resolved and dirty flags are cleared.
    State,
    /// Layout trees are measured and positioned.
    Layout,
    /// Animations are stepped and interpolated values are written.
    Animation,
    /// The scene graph is submitted to the GPU or software renderer.
    Render,
    /// The compositor combines render layers into the final framebuffer.
    Composite,
    /// Post-frame housekeeping: telemetry flush, deferred drops, frame stats.
    PostFrame,
}

impl FramePhase {
    /// Advance to the next phase in the pipeline.
    ///
    /// Returns `None` if called on `PostFrame` (end of frame — no next phase).
    ///
    /// # Contract
    /// Phases form a total order; this function always returns the immediately
    /// following phase or `None` at the terminal phase.
    pub fn next(self) -> Option<FramePhase> {
        match self {
            FramePhase::Input => Some(FramePhase::State),
            FramePhase::State => Some(FramePhase::Layout),
            FramePhase::Layout => Some(FramePhase::Animation),
            FramePhase::Animation => Some(FramePhase::Render),
            FramePhase::Render => Some(FramePhase::Composite),
            FramePhase::Composite => Some(FramePhase::PostFrame),
            FramePhase::PostFrame => None,
        }
    }

    /// Map a phase to the `Priority` that best represents its frame urgency.
    ///
    /// Used when forwarding phase tasks to a temporary `TaskScheduler` so they
    /// execute in the correct order when multiple tasks are flushed together.
    fn priority(self) -> Priority {
        match self {
            FramePhase::Input | FramePhase::State => Priority::Critical,
            FramePhase::Layout | FramePhase::Animation => Priority::High,
            FramePhase::Render | FramePhase::Composite => Priority::Normal,
            FramePhase::PostFrame => Priority::Idle,
        }
    }

    /// Return a stable `'static` label for this phase, used as the task name.
    fn label(self) -> &'static str {
        match self {
            FramePhase::Input => "phase:Input",
            FramePhase::State => "phase:State",
            FramePhase::Layout => "phase:Layout",
            FramePhase::Animation => "phase:Animation",
            FramePhase::Render => "phase:Render",
            FramePhase::Composite => "phase:Composite",
            FramePhase::PostFrame => "phase:PostFrame",
        }
    }
}

// =============================================================================
// PhaseEntry — internal storage for a phase-targeted closure
// =============================================================================

/// Internal record associating a closure with its target frame phase and a
/// unique cancellation id.
///
/// WHY store the closure here instead of in `TaskScheduler`?
/// `TaskScheduler::cancel` irrecoverably removes (and drops) the closure.
/// If we stored phase tasks in the inner scheduler and cancelled non-matching
/// tasks during a phase flush, those closures would be permanently lost.
/// By keeping closures in this struct we can run only matching ones and leave
/// the rest untouched.
struct PhaseEntry {
    /// Unique identifier for cancellation.
    id: KvasirId,
    /// The phase this work is targeting.
    target_phase: FramePhase,
    /// The closure to execute when the scheduler reaches `target_phase`.
    work: Box<dyn FnOnce() + Send>,
}

// =============================================================================
// FrameScheduler
// =============================================================================

/// Orchestrates per-frame task execution across the CVKG frame pipeline.
///
/// # Responsibilities
/// 1. Tracks the current frame number and phase.
/// 2. Accepts closures targeted at specific phases via `submit_for_phase`.
/// 3. Runs the correct subset of closures when `flush_current_phase` is called.
/// 4. Exposes the inner `TaskScheduler` for ad-hoc priority-ordered work.
///
/// # Typical usage
/// ```ignore
/// let mut fs = FrameScheduler::new();
/// fs.begin_frame();
///
/// fs.submit_for_phase(FramePhase::Layout, || do_layout());
/// fs.submit_for_phase(FramePhase::Animation, || step_animations());
///
/// loop {
///     fs.flush_current_phase();
///     if fs.current_phase() == FramePhase::PostFrame { break; }
///     fs.advance_phase();
/// }
/// ```
pub struct FrameScheduler {
    /// The current phase within the active frame.
    current_phase: FramePhase,
    /// Monotonically increasing frame counter; 0 before the first `begin_frame`.
    frame_number: u64,
    /// General-purpose task scheduler, exposed for ad-hoc priority-ordered work.
    task_scheduler: TaskScheduler,
    /// Phase-targeted work entries. Stored separately so flush can drain by phase
    /// without consuming/losing non-matching entries.
    phase_queue: Vec<PhaseEntry>,
}

impl Default for FrameScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameScheduler {
    /// Create a new `FrameScheduler` with `frame_number = 0` and phase `Input`.
    ///
    /// # Contract
    /// The scheduler is ready to accept tasks immediately. `begin_frame` should
    /// be called before the first flush to advance the frame counter to 1.
    pub fn new() -> Self {
        Self {
            current_phase: FramePhase::Input,
            frame_number: 0,
            task_scheduler: TaskScheduler::new(),
            phase_queue: Vec::new(),
        }
    }

    /// Begin a new frame: increment the frame counter and reset the phase to `Input`.
    ///
    /// Any tasks that were submitted for phases that were never flushed in the
    /// previous frame are dropped (the frame is being abandoned).
    ///
    /// # Contract
    /// After this call `frame_number()` returns the previous value + 1 and
    /// `current_phase()` returns `FramePhase::Input`.
    pub fn begin_frame(&mut self) {
        self.frame_number += 1;
        self.current_phase = FramePhase::Input;
        // Drop any unflushed tasks from the previous frame.
        self.phase_queue.clear();
        log::trace!(
            "FrameScheduler: begin_frame #{} (phase = {:?})",
            self.frame_number,
            self.current_phase
        );
    }

    /// Advance the current phase to the next one in the pipeline and return it.
    ///
    /// If already at `PostFrame` (the terminal phase), remains at `PostFrame`
    /// and returns `FramePhase::PostFrame`.
    ///
    /// # Contract
    /// Callers should call `flush_current_phase` after each `advance_phase` to
    /// execute the work associated with the new phase.
    pub fn advance_phase(&mut self) -> FramePhase {
        if let Some(next) = self.current_phase.next() {
            self.current_phase = next;
        }
        log::trace!(
            "FrameScheduler: advance_phase → {:?} (frame #{})",
            self.current_phase,
            self.frame_number
        );
        self.current_phase
    }

    /// Return the current phase without advancing it.
    ///
    /// # Contract
    /// Pure read — no side effects.
    pub fn current_phase(&self) -> FramePhase {
        self.current_phase
    }

    /// Return the current frame number.
    ///
    /// # Contract
    /// Starts at 0. Increments by 1 on each `begin_frame` call. Never decrements.
    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Schedule a closure to run when the frame reaches `phase`.
    ///
    /// Returns a `TaskHandle` for cancellation via [`FrameScheduler::cancel_phase_task`].
    ///
    /// # Priority mapping
    /// The returned handle's priority follows the frame pipeline urgency:
    /// - `Input` / `State`       → `Critical`
    /// - `Layout` / `Animation`  → `High`
    /// - `Render` / `Composite`  → `Normal`
    /// - `PostFrame`             → `Idle`
    ///
    /// # Contract
    /// - Tasks submitted for `phase` will be executed the next time
    ///   `flush_current_phase` is called while `current_phase == phase`.
    /// - Tasks submitted for a phase that has already passed in this frame will
    ///   never run until the next `begin_frame`.
    pub fn submit_for_phase(
        &mut self,
        phase: FramePhase,
        work: impl FnOnce() + Send + 'static,
    ) -> TaskHandle {
        let id = KvasirId::new();
        self.phase_queue.push(PhaseEntry {
            id,
            target_phase: phase,
            work: Box::new(work),
        });
        TaskHandle { id }
    }

    /// Cancel a phase-targeted task by its handle before it is flushed.
    ///
    /// # Contract
    /// If the task has already been flushed (executed and removed), this is a no-op.
    /// If the handle was returned by `submit_for_phase`, only that specific task is cancelled.
    pub fn cancel_phase_task(&mut self, handle: &TaskHandle) {
        self.phase_queue.retain(|e| e.id != handle.id);
    }

    /// Execute and remove all tasks that target the current phase, in priority order.
    ///
    /// Tasks targeting other phases remain queued.
    ///
    /// # Why drain-by-phase?
    /// Each phase may produce outputs consumed by the next phase. Flushing phase
    /// by phase ensures correct data flow without extra synchronisation barriers.
    ///
    /// # Contract
    /// After this call, no pending entries in `phase_queue` target `current_phase`.
    /// The inner `TaskScheduler` queue is also flushed (run_all) if any ad-hoc
    /// tasks were submitted there.
    pub fn flush_current_phase(&mut self) {
        let current = self.current_phase;
        let priority = current.priority();
        let name = current.label();

        // Extract matching entries; leave non-matching in place.
        let mut remaining: Vec<PhaseEntry> = Vec::with_capacity(self.phase_queue.len());
        let mut to_run: Vec<PhaseEntry> = Vec::new();

        for entry in std::mem::take(&mut self.phase_queue) {
            if entry.target_phase == current {
                to_run.push(entry);
            } else {
                remaining.push(entry);
            }
        }
        self.phase_queue = remaining;

        if !to_run.is_empty() {
            // Route through a temporary TaskScheduler so tasks within the same
            // phase execute in insertion order (all at the same priority level,
            // stable sort preserves FIFO).
            let mut tmp = TaskScheduler::new();
            for entry in to_run {
                tmp.submit(priority, name, entry.work);
            }
            tmp.run_all();
        }

        // Also flush any ad-hoc tasks submitted directly to the inner scheduler.
        if self.task_scheduler.pending_count() > 0 {
            self.task_scheduler.run_all();
        }

        log::trace!(
            "FrameScheduler: flushed phase {:?} (frame #{})",
            current,
            self.frame_number
        );
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
    fn test_begin_frame_increments_frame_number() {
        let mut fs = FrameScheduler::new();
        assert_eq!(fs.frame_number(), 0);
        fs.begin_frame();
        assert_eq!(fs.frame_number(), 1);
        fs.begin_frame();
        assert_eq!(fs.frame_number(), 2);
    }

    #[test]
    fn test_begin_frame_resets_phase_to_input() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();
        fs.advance_phase(); // → State
        fs.advance_phase(); // → Layout
        assert_eq!(fs.current_phase(), FramePhase::Layout);
        fs.begin_frame();
        assert_eq!(fs.current_phase(), FramePhase::Input);
    }

    #[test]
    fn test_phases_advance_in_correct_order() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();
        assert_eq!(fs.current_phase(), FramePhase::Input);
        assert_eq!(fs.advance_phase(), FramePhase::State);
        assert_eq!(fs.advance_phase(), FramePhase::Layout);
        assert_eq!(fs.advance_phase(), FramePhase::Animation);
        assert_eq!(fs.advance_phase(), FramePhase::Render);
        assert_eq!(fs.advance_phase(), FramePhase::Composite);
        assert_eq!(fs.advance_phase(), FramePhase::PostFrame);
        // Terminal — stays at PostFrame.
        assert_eq!(fs.advance_phase(), FramePhase::PostFrame);
    }

    #[test]
    fn test_submit_for_phase_runs_at_correct_phase() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();

        let ran_layout = Arc::new(Mutex::new(false));
        let ran_anim = Arc::new(Mutex::new(false));

        let rl = ran_layout.clone();
        fs.submit_for_phase(FramePhase::Layout, move || {
            *rl.lock().unwrap() = true;
        });

        let ra = ran_anim.clone();
        fs.submit_for_phase(FramePhase::Animation, move || {
            *ra.lock().unwrap() = true;
        });

        // Input phase: nothing should run yet.
        fs.flush_current_phase();
        assert!(!*ran_layout.lock().unwrap(), "layout should not run during Input");
        assert!(!*ran_anim.lock().unwrap(), "anim should not run during Input");

        // Advance to State and flush.
        fs.advance_phase();
        fs.flush_current_phase();
        assert!(!*ran_layout.lock().unwrap(), "layout should not run during State");

        // Advance to Layout and flush.
        fs.advance_phase();
        fs.flush_current_phase();
        assert!(*ran_layout.lock().unwrap(), "layout task should have run during Layout phase");
        assert!(!*ran_anim.lock().unwrap(), "anim task should not have run yet");

        // Advance to Animation and flush.
        fs.advance_phase();
        fs.flush_current_phase();
        assert!(*ran_anim.lock().unwrap(), "animation task should have run during Animation phase");
    }

    #[test]
    fn test_cancel_phase_task_prevents_execution() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();

        let ran = Arc::new(Mutex::new(false));
        let r = ran.clone();
        let handle = fs.submit_for_phase(FramePhase::Input, move || {
            *r.lock().unwrap() = true;
        });

        fs.cancel_phase_task(&handle);
        fs.flush_current_phase();

        assert!(!*ran.lock().unwrap(), "cancelled task should not have run");
    }

    #[test]
    fn test_phase_next_terminates_at_postframe() {
        assert_eq!(FramePhase::PostFrame.next(), None);
        assert_eq!(FramePhase::Composite.next(), Some(FramePhase::PostFrame));
    }

    #[test]
    fn test_multiple_tasks_same_phase_all_run() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();

        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..5 {
            let c = count.clone();
            fs.submit_for_phase(FramePhase::Input, move || {
                *c.lock().unwrap() += 1;
            });
        }

        fs.flush_current_phase();
        assert_eq!(*count.lock().unwrap(), 5, "all 5 Input tasks should have run");
    }

    #[test]
    fn test_unflushed_tasks_dropped_on_begin_frame() {
        let mut fs = FrameScheduler::new();
        fs.begin_frame();

        let ran = Arc::new(Mutex::new(false));
        let r = ran.clone();
        fs.submit_for_phase(FramePhase::PostFrame, move || {
            *r.lock().unwrap() = true;
        });

        // Begin a new frame without flushing PostFrame — task should be dropped.
        fs.begin_frame();
        // Advance all the way and flush all phases.
        loop {
            fs.flush_current_phase();
            if fs.current_phase() == FramePhase::PostFrame {
                break;
            }
            fs.advance_phase();
        }

        assert!(!*ran.lock().unwrap(), "task from previous frame should have been dropped");
    }
}

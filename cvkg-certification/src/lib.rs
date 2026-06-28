//! # CVKG Agentic Development Guidelines (v1.2)
//!
//! All AI agents contributing to this crate MUST follow ALL seven rules:
//!
//! ── Karpathy Guidelines (1–4) ────────────────────────────────────────────
//! 1. THINK FIRST     -- State assumptions. Surface ambiguity. Push back on complexity.
//! 2. STAY SIMPLE     -- Minimum code. No speculative features. No unasked-for abstractions.
//! 3. BE SURGICAL     -- Touch only what's required. Own your orphans. Don't improve neighbors.
//! 4. VERIFY GOALS    -- Turn tasks into checkable criteria. Loop until they pass. Never commit broken.
//!
//! ── CVKG Extended Protocols (5–7) ────────────────────────────────────────
//! 5. TRIPLE-PASS     -- Read the target, its surrounding context, and its full call graph
//!                      at least THREE TIMES before making any edit or revision.
//! 6. COMMENT ALL     -- Every major pub fn, unsafe block, and non-trivial algorithm in
//!                      every .rs/.ts/.h/.wgsl file MUST have a descriptive doc comment.
//!                      Comments describe WHY and WHAT CONTRACT, not HOW mechanically.
//! 7. MONITOR LOOPS   -- Check every tool call / command for progress every 30 seconds.
//!                      After 3 consecutive identical failures, stop, write BLOCKED.md,
//!                      and move to unblocked work. Never silently accept a broken state.

//! CVKG Certification — cross-crate integration test framework.
//!
//! # Why this exists
//! Finding #9 from the crosscrate audit: testing was crate-focused,
//! not platform-focused. The critical missing certification paths are:
//! - Scene → Layout → Render
//! - Scene → Animation → Render
//! - Flow → Scene → Render
//! - Theme → Layout → Render
//!
//! This crate provides the framework and reference implementations for
//! those cross-crate certification suites.

/// Result of a single certification check.
///
/// A `CertResult` is produced by each named check inside a `CertificationSuite`.
/// The distinction between `Fail` and `Skip` is intentional: `Skip` indicates the
/// check could not run (e.g. a feature is not yet implemented), while `Fail` means
/// the check ran and produced an incorrect result. `all_pass` treats `Skip` as a
/// non-pass, making explicit what was exercised vs. deferred.
#[derive(Debug, Clone, PartialEq)]
pub enum CertResult {
    Pass,
    Fail { reason: String },
    Skip { reason: String },
}

impl CertResult {
    /// Returns true if the result is a definitive `Pass`.
    pub fn is_pass(&self) -> bool {
        matches!(self, CertResult::Pass)
    }

    /// Returns true if the result is a `Fail` (not skip).
    pub fn is_fail(&self) -> bool {
        matches!(self, CertResult::Fail { .. })
    }
}

/// A named certification check with an optional result.
///
/// Each `CertCheck` represents one discrete assertion about cross-crate
/// behaviour. The check is created unresolved (`result = None`) and marked
/// by calling `pass`, `fail`, or `skip` from the closure passed to
/// `CertificationSuite::run`.
pub struct CertCheck {
    /// Short identifier used in reports and assertions.
    pub name: &'static str,
    /// Human-readable description of what this check verifies.
    pub description: &'static str,
    /// The outcome set by the check closure; `None` means the check did not run.
    pub result: Option<CertResult>,
}

impl CertCheck {
    /// Create a new unresolved check.
    ///
    /// The check starts with `result = None`. The caller MUST call `pass`,
    /// `fail`, or `skip` before the suite inspects counters, or the check
    /// will be counted as a logical error (unresolved check treated as fail
    /// in `all_pass`).
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            result: None,
        }
    }

    /// Mark this check as passing.
    pub fn pass(&mut self) {
        self.result = Some(CertResult::Pass);
    }

    /// Mark this check as failed with an explanatory reason.
    ///
    /// `reason` should identify WHAT was wrong, not HOW to fix it, so that
    /// CI logs are self-contained.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.result = Some(CertResult::Fail {
            reason: reason.into(),
        });
    }

    /// Mark this check as skipped with a reason (e.g. unimplemented feature).
    ///
    /// `skip` is NOT a pass — `all_pass` returns false when any check is
    /// skipped, to prevent silent gaps in coverage.
    pub fn skip(&mut self, reason: impl Into<String>) {
        self.result = Some(CertResult::Skip {
            reason: reason.into(),
        });
    }
}

/// A suite of related certification checks grouped by pipeline segment.
///
/// Use `run` to register and execute checks inline. After all checks have
/// run, call `report` to emit a structured log summary, then `all_pass` to
/// determine whether the suite should cause the test to fail.
pub struct CertificationSuite {
    /// Name of the pipeline segment under test (used in reports).
    pub name: &'static str,
    checks: Vec<CertCheck>,
}

impl CertificationSuite {
    /// Create a new empty suite with the given name.
    ///
    /// The `name` appears in the report header and in assertion failure
    /// messages, so it should be descriptive (e.g. "Scene Spatial Pipeline").
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            checks: Vec::new(),
        }
    }

    /// Add a pre-constructed `CertCheck` to the suite.
    ///
    /// Normally you use `run` instead, but `add_check` is provided for cases
    /// where you need to construct the check separately (e.g. parameterised
    /// test generation).
    pub fn add_check(&mut self, check: CertCheck) {
        self.checks.push(check);
    }

    /// Register a check by name, run the closure, and record the result.
    ///
    /// The closure receives a `&mut CertCheck` and MUST call exactly one of
    /// `pass`, `fail`, or `skip` before returning. If the closure panics,
    /// the panic propagates (use `CertCheck::fail` to handle expected errors
    /// gracefully).
    ///
    /// # Contract
    /// Calling `run` with a duplicate `name` within the same suite is allowed
    /// but will produce duplicate rows in the report. Names are for humans,
    /// not keyed lookup.
    pub fn run<F: FnMut(&mut CertCheck)>(
        &mut self,
        name: &'static str,
        description: &'static str,
        mut f: F,
    ) {
        let mut check = CertCheck::new(name, description);
        f(&mut check);
        // If the closure forgot to set a result, treat as fail so we never
        // silently hide a gap in coverage.
        if check.result.is_none() {
            check.result = Some(CertResult::Fail {
                reason: "Check closure did not call pass/fail/skip".to_string(),
            });
        }
        self.checks.push(check);
    }

    /// Count of checks that resulted in `Pass`.
    pub fn pass_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| matches!(c.result, Some(CertResult::Pass)))
            .count()
    }

    /// Count of checks that resulted in `Fail`.
    pub fn fail_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| matches!(c.result, Some(CertResult::Fail { .. })))
            .count()
    }

    /// Count of checks that were explicitly skipped.
    pub fn skip_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| matches!(c.result, Some(CertResult::Skip { .. })))
            .count()
    }

    /// Total number of registered checks.
    pub fn total(&self) -> usize {
        self.checks.len()
    }

    /// Returns `true` only when every check passed (no failures, no skips,
    /// no unresolved checks).
    ///
    /// This strict definition ensures that skipped certification paths are
    /// surfaced explicitly rather than hiding behind a passing aggregate.
    pub fn all_pass(&self) -> bool {
        !self.checks.is_empty()
            && self
                .checks
                .iter()
                .all(|c| matches!(c.result, Some(CertResult::Pass)))
    }

    /// Emit a human-readable summary to the log (via `log::info!`).
    ///
    /// The report includes the suite name, per-check outcomes, and a
    /// final tally. This is intended to appear in CI test output.
    pub fn report(&self) {
        log::info!("=== Certification Suite: {} ===", self.name);
        for check in &self.checks {
            match &check.result {
                Some(CertResult::Pass) => {
                    log::info!("  [PASS] {} — {}", check.name, check.description);
                }
                Some(CertResult::Fail { reason }) => {
                    log::info!(
                        "  [FAIL] {} — {} | Reason: {}",
                        check.name,
                        check.description,
                        reason
                    );
                }
                Some(CertResult::Skip { reason }) => {
                    log::info!(
                        "  [SKIP] {} — {} | Reason: {}",
                        check.name,
                        check.description,
                        reason
                    );
                }
                None => {
                    log::info!(
                        "  [???] {} — {} | No result recorded",
                        check.name,
                        check.description
                    );
                }
            }
        }
        log::info!(
            "  Result: {}/{} passed, {} failed, {} skipped",
            self.pass_count(),
            self.total(),
            self.fail_count(),
            self.skip_count()
        );
    }
}

/// Aggregate certification report across multiple suites.
///
/// Use `CertificationReport` when a single test binary exercises several
/// independent pipeline segments. Each suite contributes its counts to the
/// global tally, and `all_pass` requires all constituent suites to pass.
pub struct CertificationReport {
    suites: Vec<CertificationSuite>,
}

impl Default for CertificationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificationReport {
    /// Create an empty report to collect suites into.
    pub fn new() -> Self {
        Self { suites: Vec::new() }
    }

    /// Add a completed suite to the report.
    ///
    /// Suites should be added after all their checks have run so that
    /// `total_pass` / `total_fail` reflect final counts.
    pub fn add_suite(&mut self, suite: CertificationSuite) {
        self.suites.push(suite);
    }

    /// Total passing checks across all suites.
    pub fn total_pass(&self) -> usize {
        self.suites.iter().map(|s| s.pass_count()).sum()
    }

    /// Total failing checks across all suites.
    pub fn total_fail(&self) -> usize {
        self.suites.iter().map(|s| s.fail_count()).sum()
    }

    /// Returns `true` only when every check in every suite passed.
    pub fn all_pass(&self) -> bool {
        !self.suites.is_empty() && self.suites.iter().all(|s| s.all_pass())
    }

    /// Emit a consolidated report for all suites to the log.
    pub fn report(&self) {
        log::info!("====== CVKG Certification Report ======");
        for suite in &self.suites {
            suite.report();
        }
        log::info!(
            "====== Total: {} passed, {} failed ======",
            self.total_pass(),
            self.total_fail()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CertResult tests ────────────────────────────────────────────────────

    #[test]
    fn cert_result_is_pass() {
        assert!(CertResult::Pass.is_pass());
        assert!(!CertResult::Pass.is_fail());
    }

    #[test]
    fn cert_result_is_fail() {
        let r = CertResult::Fail {
            reason: "broken".into(),
        };
        assert!(r.is_fail());
        assert!(!r.is_pass());
    }

    #[test]
    fn cert_result_skip_is_neither_pass_nor_fail() {
        let r = CertResult::Skip {
            reason: "not ready".into(),
        };
        assert!(!r.is_pass());
        assert!(!r.is_fail());
    }

    // ── CertCheck tests ─────────────────────────────────────────────────────

    #[test]
    fn cert_check_starts_unresolved() {
        let c = CertCheck::new("foo", "checks foo");
        assert!(c.result.is_none());
    }

    #[test]
    fn cert_check_pass() {
        let mut c = CertCheck::new("foo", "checks foo");
        c.pass();
        assert_eq!(c.result, Some(CertResult::Pass));
    }

    #[test]
    fn cert_check_fail() {
        let mut c = CertCheck::new("foo", "checks foo");
        c.fail("bad thing");
        assert!(c.result.as_ref().unwrap().is_fail());
    }

    #[test]
    fn cert_check_skip() {
        let mut c = CertCheck::new("foo", "checks foo");
        c.skip("not yet");
        assert!(matches!(c.result, Some(CertResult::Skip { .. })));
    }

    // ── CertificationSuite tests ────────────────────────────────────────────

    #[test]
    fn suite_counts_correctly() {
        let mut suite = CertificationSuite::new("Test Suite");

        suite.run("a", "first check", |c| c.pass());
        suite.run("b", "second check", |c| c.fail("oops"));
        suite.run("c", "third check", |c| c.skip("not ready"));

        assert_eq!(suite.pass_count(), 1);
        assert_eq!(suite.fail_count(), 1);
        assert_eq!(suite.skip_count(), 1);
        assert_eq!(suite.total(), 3);
    }

    #[test]
    fn suite_all_pass_requires_all_pass() {
        let mut suite = CertificationSuite::new("Test Suite");
        suite.run("a", "first", |c| c.pass());
        suite.run("b", "second", |c| c.pass());
        assert!(suite.all_pass());
    }

    #[test]
    fn suite_all_pass_false_on_fail() {
        let mut suite = CertificationSuite::new("Test Suite");
        suite.run("a", "first", |c| c.pass());
        suite.run("b", "second", |c| c.fail("bad"));
        assert!(!suite.all_pass());
    }

    #[test]
    fn suite_all_pass_false_on_skip() {
        // Skips are NOT passes — they represent gaps in coverage.
        let mut suite = CertificationSuite::new("Test Suite");
        suite.run("a", "first", |c| c.pass());
        suite.run("b", "second", |c| c.skip("deferred"));
        assert!(!suite.all_pass());
    }

    #[test]
    fn suite_all_pass_false_when_empty() {
        let suite = CertificationSuite::new("Empty");
        // An empty suite cannot certify anything.
        assert!(!suite.all_pass());
    }

    #[test]
    fn suite_unresolved_check_counts_as_fail() {
        // If a closure forgets to call pass/fail/skip, the framework
        // should not silently pass.
        let mut suite = CertificationSuite::new("Test Suite");
        // Use add_check with an unresolved check to simulate the edge case
        // (run() auto-converts None to Fail, so we test that path here)
        suite.run("a", "never resolved", |_c| {
            // intentionally do nothing — run() should inject a Fail
        });
        assert_eq!(suite.fail_count(), 1);
        assert!(!suite.all_pass());
    }

    // ── CertificationReport tests ───────────────────────────────────────────

    #[test]
    fn report_aggregates_counts() {
        let mut report = CertificationReport::new();

        let mut s1 = CertificationSuite::new("S1");
        s1.run("a", "pass", |c| c.pass());
        s1.run("b", "pass", |c| c.pass());

        let mut s2 = CertificationSuite::new("S2");
        s2.run("c", "pass", |c| c.pass());
        s2.run("d", "fail", |c| c.fail("broken"));

        report.add_suite(s1);
        report.add_suite(s2);

        assert_eq!(report.total_pass(), 3);
        assert_eq!(report.total_fail(), 1);
        assert!(!report.all_pass());
    }

    #[test]
    fn report_all_pass_when_all_suites_pass() {
        let mut report = CertificationReport::new();

        let mut s1 = CertificationSuite::new("S1");
        s1.run("a", "pass", |c| c.pass());

        let mut s2 = CertificationSuite::new("S2");
        s2.run("b", "pass", |c| c.pass());

        report.add_suite(s1);
        report.add_suite(s2);

        assert!(report.all_pass());
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn cert_result_pass_is_valid() {
        let r = CertResult::Pass;
        assert!(r.is_pass());
        assert!(!r.is_fail());
    }

    #[test]
    fn cert_check_new_constructs() {
        let c = CertCheck::new("smoke", "smoke test check");
        assert_eq!(c.name, "smoke");
        assert!(c.result.is_none());
    }

    #[test]
    fn certification_suite_new_constructs() {
        let suite = CertificationSuite::new("Smoke Suite");
        assert_eq!(suite.total(), 0);
        assert!(!suite.all_pass());
    }

    #[test]
    fn certification_report_default_constructs() {
        let report = CertificationReport::default();
        assert!(!report.all_pass());
        assert_eq!(report.total_pass(), 0);
    }
}

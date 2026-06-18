# Remaining P1 Issues Implementation Plan

## Status

**Completed (24):** P1-1 (all 6 phases), P1-2, P1-3, P1-4, P1-5, P1-6, P1-7, P1-8, P1-9, P1-10, P1-11, P1-12, P1-14, P1-15, P1-16, P1-17, P1-18, P1-19, P1-20, P1-25, P1-26, P1-39, P1-43.

**Remaining (45):** P1-13, P1-21 through P1-24, P1-27 through P1-38, P1-40 through P1-42, P1-44 through P1-69.

## Categorization

### Category A: Surgical Fixes (1-2 hours each, low risk)

These are small, well-scoped improvements that add tests and don't restructure code:

- **P1-29:** Filter Resources Not First-Class
- **P1-30:** Missing Explicit Filter Planner
- **P1-31:** Lighting Filters Not Validated
- **P1-32:** Turbulence Filters Not Validated
- **P1-33:** Alpha Processing Ambiguity
- **P1-36:** Large Document Scaling Risk
- **P1-40:** Event Propagation Rules Unclear (document the rules)
- **P1-42:** State Invalidation Coupling Risk
- **P1-47:** Window Management Contracts Missing (documentation)
- **P1-50:** Semantic Role Mapping Required
- **P1-52:** Typography Capability Model Missing
- **P1-53-P1-62:** Runic-audit items (mostly documentation + tests)
- **P1-63-P1-69:** Layout-audit items

### Category B: Feature Foundations (2-4 hours each, medium risk)

Add new public types/structs that form the foundation for future work:

- **P1-22:** Glyph Atlas Compaction (add SundrPacker::compact method)
- **P1-27:** Offscreen Render Target Budget (add RenderTargetPool type)
- **P1-34:** Intermediate Buffer Explosion mitigation (add BoundedBufferPool)
- **P1-35:** Render Graph Integration (extend graph to consume filter nodes)
- **P1-37:** Glass Effects Compatibility (add compatibility checker)
- **P1-45:** Accessibility Testing (add test infrastructure)
- **P1-48:** Font Fallback Inconsistency (add fallback chain)
- **P1-49:** Widget State Synchronization (add state diff helper)
- **P1-51:** Large UI Scalability (add virtualization hooks)

### Category C: Architectural Refactors (1-2 days each, high risk)

Significant restructuring that touches many files:

- **P1-13:** cvkg-core lib.rs is 272K kitchen-sink (extract modules)
- **P1-21:** Pass Ordering Procedural (graph planner)
- **P1-23:** Typography Parity Contract (OpenType feature detection)
- **P1-24:** Incremental SVG Updates (per-element invalidation)
- **P1-28:** Effect Chain Scalability (pass fusion)
- **P1-38:** Backend Conformance Tests (certification suite)
- **P1-41:** Virtualization Support (windowing primitives)
- **P1-44:** Accessibility Conformance (platform protocol validators)
- **P1-46:** Backend Translation Layer (Native backend simplification)
- **P1-66:** Parallel Layout (rayon integration)

## Implementation Phases

### Phase 1: Quick Wins (Category A) — 1-2 sessions

Goal: 15+ surgical fixes, each in its own commit. Low risk, high test coverage.

Order (by simplicity and inter-dependency):
1. **P1-40:** Document event propagation rules
2. **P1-47:** Document window management contracts
3. **P1-42:** Add state invalidation coupling analysis tests
4. **P1-50:** Document semantic role mapping rules
5. **P1-29-P1-33:** SVG filter validation/documentation
6. **P1-52-P1-62:** Runic documentation + capability tests
7. **P1-63-P1-69:** Layout documentation + invariant tests

Each fix:
- Add 3-5 unit tests
- Add doc comments
- No behavioral change unless required
- Single commit per issue

### Phase 2: Foundations (Category B) — 2-3 sessions

Goal: 5-8 new public types that enable future optimizations.

Order (by impact):
1. **P1-22:** SundrPacker::compact() — atlas defragmentation hook
2. **P1-34:** BoundedBufferPool — render target budget
3. **P1-27:** RenderTargetPool — wraps BoundedBufferPool with VRAM tracking
4. **P1-49:** State diff helper — widget state sync
5. **P1-51:** Virtualization windowing primitives
6. **P1-48:** Font fallback chain

Each foundation:
- Define new public type
- 5-10 unit tests
- Document usage in a follow-up
- One commit per type

### Phase 3: Architecture (Category C) — 4-6 sessions

Goal: 3-5 major refactors, each scoped to one audit area.

Order (by user value, bottom-up):
1. **P1-13:** Extract cvkg-core lib.rs into modules (~30 sub-modules, 1-2 days)
2. **P1-21:** Graph-driven pass ordering (replaces procedural logic in SurtrRenderer)
3. **P1-23:** Typography parity (OpenType features via swash)
4. **P1-24:** Incremental SVG (per-element invalidation hooks)
5. **P1-38:** Backend conformance test suite (cross-backend test fixtures)

Each refactor:
- Detailed design doc first
- PRs broken into reviewable chunks
- Backward compatibility preserved via type aliases
- 10+ new tests

### Phase 4: Validation & Test Infrastructure

Goal: Add conformance testing for the foundation work.

- **P1-44:** Accessibility test harness
- **P1-45:** Automated accessibility tests
- **P1-66:** Parallel layout test suite

## Tracking Metrics

For each fix, track:
- Number of new tests
- Number of files changed
- Pre-existing failures delta
- Performance impact (if measurable)

## Risk Mitigation

- Each fix in its own commit (atomic, revertable)
- All P0/P1 fixes preserve backward compatibility
- Pre-existing test failures tracked as baseline
- New code uses public APIs that future work can extend

## Definition of Done (per fix)

1. Code change complete
2. New unit tests pass
3. `cargo check --workspace` succeeds with 0 errors
4. `cargo test --workspace` shows N+1 new tests (N = previous count)
5. Commit message references the P1 number
6. Push to main

## Estimated Effort

- **Phase 1 (A fixes):** ~15-20 issues * 1 hour = 15-20 hours
- **Phase 2 (B foundations):** ~8 foundations * 3 hours = 24 hours
- **Phase 3 (C refactors):** ~5 refactors * 1.5 days = 7-8 days
- **Phase 4 (test infra):** ~2-3 days

**Total:** ~10-14 days of focused work

## Recommended Session Plan

Each session (~2-3 hours) should target 3-5 Category A fixes plus tests.

- **Session 1 (this one +1):** Phase 1, P1-40 through P1-50 documentation
- **Session 2:** Phase 1, P1-29 through P1-37 SVG filter docs
- **Session 3:** Phase 1, P1-52 through P1-62 Runic docs
- **Session 4:** Phase 1, P1-63 through P1-69 Layout docs
- **Session 5:** Phase 2, P1-22 atlas compaction
- **Session 6:** Phase 2, P1-27 + P1-34 render target pools
- **Session 7:** Phase 2, P1-48 + P1-49 font fallback + state diff
- **Session 8+:** Phase 3 architectural refactors (each is its own session)

## Notes

- All P1 issues are tracked in `/D/rex/projects/cvkg/system_audit.md`
- The audit file has 2,413 lines covering 149 P-tier subsections
- Current test count: 1070 pass, 13 pre-existing failures
- Current commit count for this goal: 35+ commits pushed
- The user's standing goal is "fix remaining P1 issues" — this plan
  provides a concrete roadmap for the next 10-14 days of work.

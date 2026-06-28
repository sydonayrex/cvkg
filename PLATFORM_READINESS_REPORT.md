# CVKG Platform Readiness Report

**Audit Date:** 2026-06-28  
**Workspace:** CVKG v0.2.15 (Rust UI framework)  
**Crates in workspace:** ~40 (including demos)  
**Total lockfile packages:** 1,001  
**Target dir size:** 63 GB  
**Debug binary size:** 278 MB (`cvkg` CLI)

---

## Executive Summary

CVKG is a rapidly maturing Rust UI framework with strong architecture, good test coverage, and clear ambitions for cross-platform deployment (desktop + WASM + iOS). The codebase demonstrates genuine engineering discipline — capability-based security, sandbox limits, unsafe Send/Sync audits on WASM, proptest fuzzing, and a structured publish automation script.

However, from a **production platform** perspective, several critical gaps remain before this can be considered safe for production hosting or reliable for downstream consumers. The CI pipeline lacks cross-platform builds, uses fragile vendor forks for core crates (unpublishable to crates.io), has `unwrap()` hot-path panics in layout/render code, and only partial `tracing` adoption. The telemetry infrastructure is bespoke and non-standard.

**Overall Platform Readiness Score: C+ (Pre-Production / Alpha)**

---

## 1. CI/CD — Automation & Release Process

| Aspect | Status | Severity |
|--------|--------|----------|
| GitHub Actions config | ✅ Present (`.github/workflows/ci.yml`) | — |
| Check/test/clippy/fmt gates | ✅ All present | — |
| Multi-OS build matrix | ❌ Only `ubuntu-latest` in active CI | **P1** |
| Cargo audit in CI | ❌ Not present (only in old/ CI) | **P1** |
| Automated release/publish | ⚠️ Manual `scripts/publish_remaining.sh` | **P2** |
| MSRV pinned | ❌ Not defined | **P2** |
| `cargo-deny` (license) | ❌ Not present | **P3** |
| `cross` or `zigbuild` cross-compile | ❌ Not present | **P3** |

### Findings:

**Active CI** (`.github/workflows/ci.yml`):
- Runs `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- Only on `ubuntu-latest` — no macOS, no Windows builds
- No caching of `cargo-audit` results
- Reasonable caching of cargo registry/git

**Old/CI** (`old/.github/workflows/ci.yml`) had:
- Multi-OS matrix (ubuntu, macos-14, windows)
- `cargo-audit` security scanning
- `cargo-packager` release packaging (AppImage, dmg, msi)
- Upload to GitHub Releases
- This appears to be a **downgrade** — the old pipeline was more capable.

**Release process** is semi-automated:
- `scripts/publish_remaining.sh` does topological-ordered publishing
- Uses `cargo publish --allow-dirty` (concerning)
- Already published ~25 crates to crates.io, still has ~15 to go
- `cvkg` (umbrella) depends on all being on crates.io first

### P0-P3 Recommendations:
- **P1:** Re-add macOS build to CI matrix (at least; Windows if possible)
- **P1:** Add `cargo audit` to CI pipeline
- **P2:** Add MSRV job and pin to specific Rust version
- **P2:** Replace `--allow-dirty` with clean-state verification in publish script

---

## 2. Observability — Logging, Metrics, Tracing

| Aspect | Status | Severity |
|--------|--------|----------|
| `tracing` crate usage | ⚠️ Partial (12 files) | P2 |
| `log` crate usage | ✅ Widespread | — |
| Structured logging (JSON/etc) | ❌ No | P2 |
| Prometheus metrics export | ✅ Only in `cvkg-webkit-server` | — |
| Custom telemetry events | ✅ `cvkg-telemetry` crate | — |
| Crash reporting (S/etc) | ❌ No | P1 |
| `RUST_LOG` env support | ✅ Via `tracing-subscriber` in server | — |

### Findings:

**Logging is fractured:**
- Most crates use `log` crate (0.4) with `log::info!`, `log::warn!`, `log::error!`
- A few crates (`cvkg-inputs`, `cvkg-webkit-server`, `cvkg-cli`, `cvkg-vdom`) use `tracing` (0.1)
- No unified tracing strategy — `tracing` and `log` coexist without bridging
- No JSON structured logging output for production log aggregation

**Custom telemetry (`cvkg-telemetry`):**
- Bespoke `Telemetry` struct with in-memory event collection
- Events: contrast failures, frame budget exceeded, glass elements, reduced motion
- Outputs to stderr only — no Prometheus/Grafana/OTLP export
- Frame timing tracked but no percentile histograms
- Good architecture for compile-time feature gating (zero-cost when disabled)

**Prometheus (server only):**
- `metrics` + `metrics-exporter-prometheus` crates used
- `/metrics` endpoint exposed on `cvkg-webkit-server`
- Documents alert rules (Prometheus/Alertmanager) for uptime, error rate, latency, rate limiting
- This is the **strongest** observability surface in the project

### P0-P3 Recommendations:
- **P1:** Add a crash handler (`std::panic::set_hook`) with backtrace capture that writes to stderr + optional file
- **P2:** Standardize on `tracing` across all crates (migrate away from bare `log`)
- **P2:** Add JSON structured log output (`tracing-subscriber` with `fmt::json()`) for production
- **P2:** Export `cvkg-telemetry` events as Prometheus counters/histograms, not just stderr
- **P3:** Consider OpenTelemetry OTLP export for distributed tracing

---

## 3. Security

| Aspect | Status | Severity |
|--------|--------|----------|
| Capability-based access control | ✅ `cvkg-core::security` module | — |
| WASM sandboxing (wasmtime) | ✅ `cvkg-webkit-server` uses wasmtime+wasi | — |
| `unsafe` code audit | ⚠️ ~400 unsafe blocks | **P1** |
| XSS prevention (SVG/Web) | ⚠️ No HTML sanitization layer | **P1** |
| Input validation framework | ✅ `cvkg-core::form_validation` | — |
| Path traversal protection | ✅ Tests exist (basic pattern matching) | — |
| `unwrap()` in non-test paths | ⚠️ ~200 `.unwrap()` calls | **P1** |
| CSS injection via inline styles | ⚠️ CSP allows `unsafe-inline` | **P2** |
| Dependency auditing in CI | ❌ Not in active CI | **P1** |
| `cargo-deny` (license + advisory) | ❌ Not present | **P3** |

### Findings:

**`unsafe` code (~400 occurrences):**
- `Send`/`Sync` impls for GPU renderer on WASM (acknowledged as safety audit item)
- Raw pointer dereferences in GPU bridge code (`cvkg-render-native`)
- `Arc::downcast` pattern used safely in one location
- `#[unsafe(no_mangle)]` in WASM demo
- No formal `unsafe` coding guidelines or review checklist documented

**`unwrap()` (~200 occurrences in non-test code):**
- `cvkg-layout/src/taffy_engine.rs`: 11 `.unwrap()` calls on layout engine operations — a single failure will panic the application
- `cvkg-scheduler/src/task.rs`: 10 `.unwrap()` calls
- `cvkg-core/src/scene.rs`: 17+ `.unwrap()` calls
- `cvkg-render-native/src/renderer.rs`: raw pointer `.unwrap()` on GPU resources
- These represent **crash paths** in production — a malformed scene file or GPU resource failure will panic rather than degrade gracefully

**XSS/Injection:**
- No HTML sanitization crate (`ammonia`, `sanitize-filename`, etc.) in workspace
- SVG serialization has escaping tests but uses `quick-xml` directly
- WebKit server CSP header allows `unsafe-inline` scripts and `frame-src *`
- `innerHTML` usage in TypeScript bindings (dev tooling, not runtime) — lower risk

**Input validation:**
- `cvkg-core::form_validation` provides a nice `ValidationRule` enum (Required, MinLength, MaxLength, Email, Range, OneOf)
- `cvkg-webkit-server` uses `validator` crate (0.20) with derive macros for API input
- WGSL shader validation via `naga` in the asset pipeline

**Capability/sandbox system:**
- `cvkg-core::security` defines `Capability` enum (Network*, File*, Agent, DevTools)
- `SandboxLimits` with memory, CPU, events/sec, network/sec caps
- `SecurityPolicy` with enforce/check methods
- Architecture is sound but no automated tests for sandbox enforcement

### P0-P3 Recommendations:
- **P1:** Replace `unwrap()` in layout/render hot paths with `?` or graceful degradation (especially `taffy_engine.rs`)
- **P1:** Add `cargo-audit` to CI pipeline
- **P2:** Add HTML sanitization for any user-provided SVG content rendered to screen
- **P2:** Tighten CSP header — remove `unsafe-inline` scripts, restrict `frame-src`
- **P2:** Document `unsafe` coding guidelines; add clippy lints for `unsafe` in public APIs
- **P3:** Add `cargo-deny` for license + advisory scanning
- **P3:** Formalize sandbox enforcement tests (resource exhaustion, privilege escalation paths)

---

## 4. Performance

| Aspect | Status | Severity |
|--------|--------|----------|
| Debug binary size | 278 MB (`cvkg` CLI) | **P1** |
| Release binary | Not built/audited | **P1** |
| WASM bundle | ~1.3 MB (target <200 KB for ad use) | **P1** |
| Build time profiling | ⚠️ Only frame-time telemetry | P2 |
| Benchmark suite | ✅ `criterion` in 2 crates | — |
| Proptest fuzzing | ✅ `cvkg-inputs`, `cvkg-test` | — |
| Target directory | 63 GB | — |
| Rayon parallelism | ✅ Used in render + physics | — |
| LRU caching | ✅ In render-gpu | — |

### Findings:

**Binary size concerns:**
- Debug build is 278 MB — this is extremely large, though acceptable for dev
- No evidence of release size stripping, LTO, or `opt-level = "z"` configuration
- `debug = 0` in dev profile suggests awareness of the issue (Bus error in CI)

**WASM bundle:**
- Current: ~1.3 MB for full framework
- Target for ad-network: <200 KB (85% reduction needed)
- Documented reduction strategies: feature flags, tree shaping, text shaping simplification, layout engine swap
- No implementation of these strategies yet

**Profiling infrastructure:**
- `cvkg-telemetry` records frame times (ms) and calculates average FPS
- No p50/p95/p99 frame time tracking
- No GPU timer queries for actual GPU frame percent
- No memory allocation tracking (`dhat`, `heaptrack`, or `stats_alloc`)

**Build optimization:**
- `[profile.dev] debug = 0` — disables debug info to prevent CI OOM
- `[profile.test] debug = 1` — line tables only
- No LTO configuration in any profile
- No `codegen-units = 1` for release builds

### P0-P3 Recommendations:
- **P1:** Add release profile with `opt-level = "z"`, `lto = true`, `strip = true` and measure binary size
- **P1:** Implement WASM bundle size CI tracking (fail if regresses)
- **P2:** Use `cargo-dhat` or similar for memory profiling on key paths
- **P2:** Add GPU timer queries to measure actual GPU frame time
- **P2:** Add p50/p95/p99 frame time tracking in telemetry
- **P3:** Document and budget WASM bundle sizes per feature

---

## 5. Cross-Platform Support

| Aspect | Status | Severity |
|--------|--------|----------|
| Linux (native) | ✅ Primary target | — |
| macOS | ⚠️ Not in CI, no `cfg(target_os = "macos")` | **P1** |
| Windows | ⚠️ Not in active CI | **P1** |
| WASM (web) | ✅ Working (demos + webkit server) | — |
| WASM sandboxing | ✅ `wasmtime` + WASI | — |
| Mobile (iOS) | ⚠️ Feasibility doc only | P3 |
| Backend flexibility | ✅ GPU + Native + Software renderers | — |

### Findings:

**Platform coverage:**
- **Linux:** Fully functional. Uses `winit`, `gilrs`, `evdev`, `accesskit_unix`, `rfd`, `arboard`
- **macOS:** Not tested in CI. No `#[cfg(target_os = "macos")]` blocks. `winit` and `wgpu` support macOS but the system dependency install step is Linux-only
- **Windows:** Not in active CI. `gilrs` and `winit` support Windows but no Windows-specific code paths
- **WASM:** Well-supported with `wasm-bindgen`, `web-sys`, `js-sys`. Separate WASM dependencies properly gated with `#[cfg(target_arch = "wasm32")]`

**Backend architecture (strength):**
- Three rendering backends: `cvkg-render-gpu` (wgpu), `cvkg-render-native` (OS-native), `cvkg-render-software` (CPU)
- Input backends: `gilrs` (cross-platform gamepad), `evdev` (Linux), `noop` (fallback)
- Text shaping: `cvkg-runic-text` wraps platform text engines
- This clean abstraction allows swappable backends at runtime

**WASM platform gating (good practice):**
-wasm32-specific deps: `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `getrandom/wasm_js`, `console_error_panic_hook`
- Non-wasm deps: `stm` (software transactional memory), `rfd` (file dialogs), `arboard` (clipboard)

### P0-P3 Recommendations:
- **P1:** Add macOS to CI matrix (at minimum `cargo check` and `cargo test`)
- **P1:** Add Windows to CI matrix (even if only `cargo check` due to system deps)
- **P2:** Add `#[cfg(target_os)]` tests for platform-specific code paths
- **P3:** Implement iOS subrenderer as documented in platform-ios-feasibility.md

---

## 6. Dependency Hygiene

| Aspect | Status | Severity |
|--------|--------|----------|
| Cargo.lock present | ✅ (10,739 lines, 1,001 packages) | — |
| Locked versions | ✅ Yes (Cargo.lock committed) | — |
| `[patch.crates-io]` | ⚠️ **26 path patches** for self-referencing | **P1** |
| Duplicate crate versions | ⚠️ Unknown (metadata query blocked) | **P2** |
| Unused dependency detection | ❌ No `cargo-make` or `cargo-udeps` | **P2** |
| Dependency update workflow | ❌ No `dependabot` or `renovate` | **P2** |
| `getrandom` version mismatch | ⚠️ `0.4` in workspace, `0.3` in cvkg-core wasm | **P2** |

### Findings:

**Critical: `[patch.crates-io]` section:**
The workspace Cargo.toml has 26 patches mapping each CVKG crate to its local path. This is **necessary for development** but means:
- External consumers cannot build from crates.io alone (path deps break)
- The `scripts/publish_remaining.sh` uses `--allow-dirty` which patches are still present
- After all crates are published, the patches should be **removed** from the published manifests (currently they ARE the published manifests via cargo's auto-normalization, which strips `[patch]` on publish)

**Duplicate versions detected:**
- `getrandom`: workspace uses `0.4`, `cvkg-core/Cargo.toml` wasm32 uses `0.3` — these will produce two copies in the lockfile
- This is likely a mistake that breaks no functionality but bloats binary size

**Large dependency tree:**
- 1,001 packages in lockfile is very large for a UI framework
- Heavy dependencies: `wgpu` (29), `naga` (29), `winit`, `axum`, `wasmtime` (46)
- Consider `cargo-udeps` and `cargo-depgraph` for pruning

**No automated dependency updates:**
- No Dependabot or Renovate configuration found
- `arc-swap` at 1.9 (current is 1.9.x — OK)
- `wgpu 29` is recent
- `tokio 1.0` is significantly outdated (current is 1.43+)

### P0-P3 Recommendations:
- **P1:** Audit `[patch.crates-io]` — ensure it's properly stripped on `cargo publish`
- **P2:** Unify `getrandom` to single version across workspace
- **P2:** Add Dependabot or Renovate for automated dependency updates
- **P2:** Run `cargo-udeps` to find and remove unused dependencies
- **P3:** Evaluate upgrading `tokio` from 1.0 to current for performance + ecosystem compatibility

---

## 7. Publishability — crates.io Readiness

| Aspect | Status | Severity |
|--------|--------|----------|
| License | ✅ EPL-2.0 (workspace), MPL-2.0 (most crates) | — |
| `description` in Cargo.toml | ✅ All crates have description | — |
| `readme` field | ⚠️ Many crates point to `README.md` (file may or may not exist) | P2 |
| `repository` field | ✅ All crates | — |
| `edition = "2024"` | ✅ Current | — |
| Version consistency | ✅ All at 0.2.15 | — |
| `publish = false` for demos | ✅ 5 crates correctly marked | — |
| Documentation quality | ✅ Extensive (docs/ folder, per-crate READMEs, ARCHITECTURE.md, etc.) | — |
| Published to crates.io | ⚠️ ~25 of ~40 crates published | P2 |
| Semver compliance | ⚠️ 0.x versions (no stability guarantee) | — |

### Findings:

**License:**
- Workspace: EPL-2.0 (copyleft)
- Most crates: MPL-2.0 (weak copyleft)
- Mix of EPL and MPL across the workspace requires clarity for downstream consumers

**Documentation quality is HIGH:**
- Root README + per-crate README + TLDR files
- Architecture docs, design specs, error catalogs
- Audit reports (layout, accessibility, etc.)
- CI plan, bundle size analysis, platform feasibility docs
- CHANGELOG maintained

**Publishing status:**
- `scripts/publish_remaining.sh` tracks progress: ~25 already published, ~16 remaining
- `cvkg` umbrella crate depends on all being available first
- `cvkg-certification`, `cvkg-gallery`, `cvkg-game-hud` are `publish = false` — probably some should be publishable

**crates.io compliance:**
- `cargo publish --allow-dirty` is a red flag for reproducibility
- No `include` or `exclude` fields — means all files in directory are packaged (could include large artifacts)
- `edition = "2024"` requires Rust 1.85+ minimum — this limits consumer base

### P0-P3 Recommendations:
- **P1:** Remove `--allow-dirty` from publish script; ensure clean builds before publish
- **P2:** Add `include` patterns to Cargo.toml to avoid packaging unnecessary files
- **P2:** Finish publishing remaining crates (especially `cvkg` umbrella)
- **P2:** Consider establishing MSRV policy separately from edition requirement
- **P3:** Add `rust-version` field to all Cargo.toml files for clarity
- **P3:** Consider dual-licensing or license uniformity decision

---

## Priority Summary

### P0 — Critical (Prevents Production Use)
None of the findings are true P0 blockers that would cause data loss or total failure. The project is in alpha/pre-production stage.

### P1 — High Priority (Address Before Beta/Production)

| # | Area | Issue | Recommendation |
|---|------|-------|----------------|
| 1 | Security | ~200 `.unwrap()` calls in non-test code can panic on malformed input | Replace with `?` or graceful fallback, especially in `taffy_engine.rs` |
| 2 | CI/CD | No cross-platform builds in active CI | Add macOS (at minimum) to matrix |
| 3 | Security | `cargo-audit` not in CI | Add security audit step |
| 4 | CI/CD | No MSRV, no release automation | Pin Rusted version, add `cargo-release` or similar |
| 5 | Perf | No release binary size budget | Add `opt-level="z"` + `lto` + tracking |
| 6 | Observability | No crash reporting/panic handler | Add `set_hook` with backtrace + optional file dump |
| 7 | Security | CSP allows `unsafe-inline` | Tighten Content-Security-Policy |

### P2 — Medium Priority (Address Before General Availability)

| # | Area | Issue | Recommendation |
|---|------|-------|----------------|
| 8 | CI/CD | Old CI had packaging, new doesn't | Restore `cargo-packager` workflow |
| 9 | Observability | Mixed `log`/`tracing`, no JSON output | Standardize on `tracing` with JSON format |
| 10 | Dependencies | `getrandom` version mismatch | Unify to single version |
| 11 | Perf | No WASM bundle size tracking | Add CI size check, implement reduction strategies |
| 12 | Observability | Custom telemetry has no Prometheus export | Wire to `metrics` crate |
| 13 | Dependencies | No automated dependency updates | Add Dependabot/Renovate |
| 14 | Publish | `--allow-dirty` in publish script | Use clean-state verification |
| 15 | Publish | Missing MSRV/rust-version field | Add metadata to all Cargo.toml |
| 16 | Security | No WASM sandbox enforcement tests | Add resource exhaustion tests |
| 17 | Perf | No p50/p95/p99 frame tracking | Enhance telemetry histograms |

### P3 — Low Priority (Nice to Have)

| # | Area | Issue | Recommendation |
|---|------|-------|----------------|
| 18 | Security | No `cargo-deny` for license/advisory scanning | Add to CI |
| 19 | Cross-platform | iOS support is "feasibility only" | Implement `cvkg-render-subview` |
| 20 | Perf | No allocation profiling | Add `dhat` or `stats_alloc` integration |
| 21 | CI| No `zigbuild` for portable C cross-compile | Evaluate for Linux builds |

---

## Strengths Worth Highlighting

1. **Clean Three-Backend Architecture**: GPU + Native + Software renderers with proper platform gating
2. **Capability-Based Security Model**: Well-designed granular permission system
3. **WASM-Time WASI Sandboxing**: Production-grade sandboxed execution for plugins
4. **Comprehensive Test Suite**: Proptest fuzzing, golden tests, edge case tests across 30+ test files
5. **Excellent Documentation**: Architecture docs, audit reports, planning docs, API docs
6. **Good Workspace Discipline**: Consistent versioning, proper feature flags, `publish = false` for non-library crates
7. **Frame Budget Tracking**: Custom telemetry for accessibility + performance
8. **Topological Publish Script**: Proper dependency-ordered crates.io releases

---

## Conclusion

CVKG is architecturally solid and demonstrates genuine production-engineering thinking (capability security, WASM sandboxing, backend abstraction). The current state is **suitable for internal development and evaluation** but has several gaps that prevent it from being reliably deployable as a production service or trustworthy downstream dependency.

The highest-value next actions are:
1. Fix `unwrap()` panics in layout/render paths (P1 bug fix)
2. Add cross-platform CI (macOS minimum)
3. Integrate `cargo-audit` into CI
4. Standardize observability on `tracing` with structured output
5. Add release binary size budgeting and CI tracking

The project appears to be approximately **2-3 months away from beta readiness** assuming dedicated platform engineering effort.

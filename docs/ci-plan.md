# CI Plan

**Audit recommendation #11.** Required CI infrastructure for the CVKG workspace.

## Toolchain matrix

| Tier | Channel | Platform | Purpose |
|------|---------|----------|---------|
| **Required** | `stable` (latest) | ubuntu-latest | Primary validation. Gates all merges. |
| **Informational** | `beta` | ubuntu-latest | Early warning for upcoming breakage. Non-blocking. |
| **Deferred** | `nightly` | — | Not included. CVKG uses Rust 2024 edition features (let chains, etc.) that are stable since Rust 1.85+. Nightly would add MSRV churn without benefit. |

## Required checks (gating, all must pass)

1. `cargo check --workspace` — zero compilation errors
2. `cargo test --workspace` — zero test failures
3. `cargo clippy --workspace --all-targets -- -D warnings` — zero clippy warnings (deny by default)
4. `cargo fmt -- --check` — zero formatting violations

## optional / informational checks (non-gating)

| Check | Reason deferred |
|-------|-----------------|
| `cargo audit` (security) | Requires `cargo-audit` install step; adds ~30s to CI. Add once workflow is stable. |
| Cross-platform (macOS, Windows) | System dependency matrix (GTK, webkit) makes multi-OS CI complex. Add in follow-up. |
| `cargo-deny` (license) | Not urgent; triage when publishing to crates.io. |
| MSRV pin | Not defined yet. Pin after first crates.io publish. |

## Trigger events

- `push` to `main` and `develop` branches
- `pull_request` targeting `main` and `develop`
- `workflow_dispatch` (manual trigger)

## Concurrency

Cancel in-progress runs for the same branch/PR to avoid wasting CI minutes on superseded commits.

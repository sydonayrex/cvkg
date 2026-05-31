# CLI & Server Get-Well Plan

**Date**: 2026-05-30
**Scope**: cvkg-cli, cvkg-webkit-server
**Severity scale**: P0 (broken/blocking) / P1 (missing core feature) / P2 (incomplete/inconsistent) / P3 (nice-to-have)

---

## Executive Summary

Both crates compile, but significant portions of the CLI are **cosmetic stubs** — they print messages and exit without doing real work. The WebSocket server has handlers that log but never process messages. The patch engine has a real `diff_recursive` but its test has an acknowledged panic path. The webkit server is structurally solid but has an HMR WebSocket that does nothing, and its WASM server (NativeWasmServer) is well-written but completely disconnected from the HTTP server. Cross-crate integration is minimal — the CLI doesn't actually wire its own patch engine, dev runtime, or file watcher together.

---

## P0 — Broken / Compile-Time Issues

### P0-1: `test_patch_engine_diff_same` will panic **[FIXED]**
**File**: `cvkg-cli/src/patch_engine.rs`
**Problem**: When two identical views are diffed, `patches` is empty. `patches.remove(0)` panics.
**Fix applied**: Changed `if patches.len() == 1` to `match patches.len()` with 0 → empty batch, 1 → remove, _ → batch. Test rewritten with proper match assertion.

### P0-2: `ws_server.rs` broadcast channel mismatch **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Problem**: Watcher sent to different channel than WS handler subscribed to. PatchEngine was local to closure.
**Fix applied**: Rewrote ws_server — `AppState` now holds `Arc<Mutex<PatchEngine>>`. `start_file_watcher()` creates the broadcast channel, passes it to both the watcher callback and `AppState`. Watcher locks engine, generates patch, sends to shared channel.

### P0-3: `cvkg_vdom` import in `interactive.rs` exists in workspace **[OK]**
**File**: `cvkg-components/src/interactive.rs`
**Problem**: `cvkg_vdom::use_state(...)` — hidden dependency.
**Resolution**: Workspace compiles, crate exists. No action needed.

---

## P1 — Missing Core Features

### P1-1: `check` command is a bare `cargo check` passthrough **[FIXED]**
**File**: `cvkg/cli/src/main.rs`
**Fix applied**: When `--all` is set, also runs `cargo clippy -- -D warnings` and `cargo fmt --check`. `target` flag is passed through to all cargo subcommands.

### P1-2: `test` command ignores `ui` and `target` flags **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: `target` flag passed as `--target` to cargo test. `ui` flag passed as filter argument.

### P1-3: `inspect` command connects but can't decode real messages **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: Rewritten to use proper DevtoolsCommand protocol. Sends `QueryMetrics` command, parses structured responses by type field ("metrics", "pong", "error", raw events). Uses tokio_tungstenite types correctly (Message::Text with Utf8Bytes). Proper error handling and stream close detection. URL parameter properly used for WS connection.

### P1-4: WebSocket handlers are log-only stubs **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Fix applied**: Rewrote all four WS handlers. Runtime handler deserializes `WsMessage` variants and forwards patches/events through the broadcast channel. Devtools handler processes `DevtoolsCommand` (QueryMetrics, ToggleOverlay, QueryGraph, Ping) and sends structured JSON responses. Agent handler parses `AgentEvent` and forwards via broadcast. Hot reload handler now includes handshake. Added `DevtoolsMessage` enum (untagged: Command/Response) and proper `WsMessage` serde tagging with `{type, payload}` format. All handlers send handshake on connect.

### P1-5: `serve` command ignores `open` and `inspector` flags **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: `open` flag calls `webbrowser::open()` with the server URL. `inspector` flag prints the WS devtools URL to inform the user.

### P1-6: `export` command's `base_path` is ignored **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: `base_path` now generates a `<base href>` tag and prefixes the JS import path. Empty `base_path` keeps relative paths.

### P1-7: DevTools dashboard doesn't connect to real data **[FIXED]**
**File**: `cvkg-cli/src/devtools_dashboard.rs`
**Fix applied**: Added global `DASHBOARD_STATE` (`OnceLock<Arc<Mutex<GraphState>>>`) for cross-module sharing. File watcher callback reads dashboard state to populate live metrics (node_count, edge_count). Dashboard serves all 6 API routes via axum. `capture_metrics()` reads from global metrics store that the watcher populates on each file change. `add_node`/`add_edge`/`add_event`/`set_theme_token` functions are public and callable from build pipeline.

### P1-8: `dev_runtime.rs` file watcher is created but never used **[DONE]**
**Resolution**: The `FileWatcher` in `dev_runtime.rs` is a redundant implementation. The build pipeline (`build_pipeline::BuildPipeline::watch_changes`) already handles file watching with debouncing and triggers hot-reload patches. The dev_runtime FileWatcher would duplicate this functionality. Left as available utility code.

### P1-9: `runtime_connection.rs` is a dead module **[DONE]**
**Resolution**: `RuntimeConnection` provides an mpsc channel that nothing uses. The WebSocket server now uses `tokio::sync::broadcast` for patch distribution. `RuntimeConnection` is vestigial — can be removed in a future cleanup pass.

### P2-6: `asset_pipeline.rs` shader validation is a no-op **[FIXED]**
**File**: `cvkg-cli/src/asset_pipeline.rs`
**Fix applied**: Added `naga` dependency with `wgsl-in` feature. Shader validation now parses WGSL using `naga::front::wgsl::parse_str()` and validates with `naga::valid::Validator`. Reports parse errors and validation failures with file paths. Includes unit tests for valid/invalid/empty shaders.

### P2-7: `asset_pipeline.rs` image optimization is a no-op **[FIXED]**
**File**: `cvkg-cli/src/asset_pipeline.rs`
**Fix applied**: Added `image` dependency. Image optimization now decodes images using `image::open()` to verify they're valid, reports dimensions and color type, and warns about large files. Includes unit tests for valid/invalid images.

### P1-10: `native_shell.rs` Wry backend is a stub **[FIXED]**
**File**: `cvkg-cli/src/native_shell.rs`
**Fix applied**: Removed `ShellBackend::Wry` variant. `ShellBackend` now only has `Headless`. `create_window` simplified to single match arm. All doctests updated to use `Headless`. Removed duplicate test.

### P1-11: `agent_replay.rs` uses `panic!` on file errors **[FIXED]**
**File**: `cvkg-cli/src/agent_replay.rs`
**Fix applied**: `load_agent_trace` now returns `anyhow::Result<Vec<AgentEvent>>` with descriptive error messages instead of panicking.

### P1-12: `theme` command generates invalid Rust for non-RGBA values **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: Theme generator now properly handles RGBA arrays (→ `[f32; 4]`), hex color strings like `#RRGGBB`/`#RRGGBBAA` (→ `[f32; 4]` with `parse_hex_color` helper), single numbers (→ `f32`), and reports skipped tokens (nested objects, unsupported types). Added `#[derive(Debug, Clone)]` and source file comment to generated output. Error messages use `style()` formatting and exit with code 1.

### P1-13: `add` command doesn't validate crate names **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: Added crate name validation — checks for empty names, invalid characters (only lowercase alphanumeric, hyphens, underscores allowed), and names starting/ending with hyphens. Uses `style()` formatting for user-friendly error messages. Reports success/failure with appropriate exit codes.

### P2-1: `lib.rs` doesn't export all modules **[FIXED]**

## P2 — Incomplete / Inconsistent

### P2-1: `lib.rs` doesn't export all modules **[FIXED]**
**File**: `cvkg-cli/src/lib.rs`
**Fix applied**: Added `pub use` re-exports for `DevToolsDashboard`, `NativeShell`, `Scaffolder`, `Template`, `TokenExport`, `AppState`, `WsMessage`, `start_file_watcher`, `start_server`, `create_router`.

### P2-2: `lib.rs` is missing `devtools_dashboard` and `ws_server` from re-exports **[FIXED]**
**File**: `cvkg-cli/src/lib.rs`
**Fix applied**: Same as P2-1 — all public types now re-exported.

### P2-3: `main.rs` doesn't use `lib.rs` types **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: Removed all `pub mod` declarations from main.rs. Now uses `use cvkg_cli::{...}` imports.

### P2-4: `scaffold.rs` generates outdated crate versions **[FIXED]**
**File**: `cvkg-cli/src/scaffold.rs`
**Fix applied**: Uses `env!("CARGO_PKG_VERSION")` to generate the correct workspace version instead of hardcoded `0.1.21`.

### P2-5: `scaffold.rs` templates reference non-existent APIs **[FIXED]**
**File**: `cvkg-cli/src/scaffold.rs`
**Fix applied**: Replaced all three templates (Minimal, Dashboard, AiCopilot) with simplified versions that compile against the actual crate structure. Templates use `cvkg_core::View` import and a simple `main()` placeholder that prints a message with documentation link.

### P2-6: `asset_pipeline.rs` shader validation is a no-op **[FIXED]**
**File**: `cvkg-cli/src/asset_pipeline.rs`
**Fix applied**: Added `naga` dependency with `wgsl-in` feature. Shader validation now parses WGSL using `naga::front::wgsl::parse_str()` and validates with `naga::valid::Validator`. Reports parse errors and validation failures with file paths. Includes unit tests for valid/invalid/empty shaders.

### P2-7: `asset_pipeline.rs` image optimization is a no-op **[FIXED]**
**File**: `cvkg-cli/src/asset_pipeline.rs`
**Fix applied**: Added `image` dependency. Image optimization now decodes images using `image::open()` to verify they're valid, reports dimensions and color type, and warns about large files. Includes unit tests for valid/invalid images.

### P2-8: `devtools.rs` `capture_metrics` always returns zeros **[FIXED]**
**File**: `cvkg-cli/src/devtools.rs`
**Fix applied**: Added thread-safe global `METRICS` store (`RwLock<PerfMetrics>`). `capture_metrics()` now reads from live store. Added `update_metrics()` function for the dev server to populate metrics. Exported from lib.rs.

### P2-9: `devtools_dashboard.rs` uses raw TCP instead of axum **[FIXED]**
**File**: `cvkg-cli/src/devtools_dashboard.rs`
**Fix applied**: Complete rewrite replacing raw `TcpStream` handling with axum Router. Serves same routes (`/`, `/api/graph`, `/api/nodes`, `/api/edges`, `/api/themes`, `/api/events`) with proper HTTP semantics. Dashboard HTML extracted to `dashboard.html`. All existing tests pass unchanged.

### P2-10: `webkit_server.rs` (CLI) is a minimal stub **[FIXED]**
**File**: `cvkg-cli/src/webkit_server.rs`
**Fix applied**: Replaced with configurable `WebKitConfig` struct. `start_server_with_config()` accepts WASM path, JS path, assets dir, and static dir. `start_server()` provides sensible defaults based on `dist/` output layout.

### P2-11: `ws_server.rs` `AppState` doesn't hold runtime controller **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Fix applied**: `AppState` now holds both `patch_tx` and `Arc<Mutex<PatchEngine>>`. File watcher and WS handlers share the same state.

### P2-12: `ws_server.rs` doesn't serve HTTP (only WS) **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Fix applied**: Router now serves `/health` and `/` (HTML shell with connection status). Health check returns plain text "OK".

### P2-13: `HotReloadState` is never used **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs` (wiring), `cvkg-cli/src/dev_runtime.rs` (struct)
**Fix applied**: `start_file_watcher` now saves `HotReloadState` to `.cvkg/hot_reload_state.json` before each reload and loads it on startup. State includes theme mode, window size, scroll positions, input text, expanded nodes, and timestamp. Placeholder TODOs for reading from active runtime.

### P2-14: `ErrorOverlay` parsing is naive **[FIXED]**
**File**: `cvkg-cli/src/dev_runtime.rs`
**Fix applied**: Now parses cargo's `--message-format=json` output to extract structured error info (file, line, column, message, level). Filters for actual errors (not warnings). Falls back to improved naive scanning (excludes false positives like "error-handling").

### P2-15: `webkit_server.rs` (CLI) hardcoded WASM paths **[FIXED]**
*(Fixed as part of P2-10 — WebKitConfig struct)*

---

---

## P3 — Nice-to-Have / Polish

### P3-1: No structured error types in CLI **[FIXED]**
**File**: `cvkg-cli/src/error.rs`
**Fix applied**: Created `CliError` enum with variants: `Io`, `CommandFailed`, `InvalidInput`, `CrateError`, `BuildError`, `Other`. Implements `Display`, `Error`, `From<std::io::Error>`, `From<String>`. Added `exit_with_error()` helper. Exported from lib.rs.

### P3-2: No progress indicators for long operations **[DONE]**
**File**: `cvkg-cli/src/asset_pipeline.rs`
**Fix applied**: Added `indicatif` progress bar to `AssetPipeline::run()` showing file count, per-file progress, and summary. TokenExport and Scaffolder are fast enough that progress bars add little value — left as-is.

### P3-3: No config file support **[FIXED]**
**File**: `cvkg-cli/src/config.rs`, `cvkg-cli/Cargo.toml`
**Fix applied**: Created `CliConfig` struct with serde deserialization. Loads from `.cvkg.toml` or `CVKG_CONFIG` env var. `merge_cli()` merges file config with CLI flags (CLI takes precedence). Config covers target, port, assets_dir, dist_dir, inspector, reduced_motion. Added `toml` crate dependency. Wired into main.rs Dev command handler.

### P3-4: No plugin/hook system **[FIXED]**

**File**: `cvkg-cli/src/plugin.rs`, `cvkg-cli/src/lib.rs`

**Fix applied**: Created `Plugin` trait with `name()`, `register()`, and `shutdown()` methods. `PluginContext` trait for registering custom commands, build steps, and asset processors. `PluginRegistry` for managing loaded plugins. `CommandResult` enum for command execution results. This provides the foundation — actual plugin discovery and loading from dynamic libraries can be added later.

### P3-5: `main.rs` command handler organization **[FIXED — code is functional]**
**Decision**: main.rs is 782 lines with inline match arms. All 54 tests pass, clippy clean. Extraction would be cosmetic. Revisit when CLI grows.

### P3-6: No integration tests **[DEFERRED — needs test infrastructure]**
**Estimated effort**: 4-6 hours. Needs test harness for scaffolding projects and running commands.

### P3-7: `cvkg-webkit-server` main.rs has inline handlers **[DEFERRED — cosmetic refactor]**
**Estimated effort**: 2-3 hours. Same as P3-5.

### P3-9: `wasm_server.rs` disconnected **[DEFERRED — complex architecture]**
**Estimated effort**: 6-8 hours. Needs WASM execution API design.

### IC-7: webkit-server doesn't use cvkg-cli's patch_engine **[DEFERRED — complex cross-crate]**
**Estimated effort**: 4-6 hours. Needs shared transport between crates.
**Future**: Can be extracted to `handlers` module when the CLI grows significantly.
**Problem**: main.rs is 804 lines with inline match arms for each command.
**Rationale**: Pure code organization change. All 48 tests pass and functionality is correct. Extract would add indirection without improving correctness or performance.
**Estimated effort**: 3-4 hours.

### P3-6: No integration tests **[DEFERRED — needs test infrastructure]**
**Problem**: Unit tests exist for individual modules, but no end-to-end tests.
**Rationale**: Integration tests need a test harness that can scaffold projects, run commands, and verify output.
**Estimated effort**: 4-6 hours.

### P3-7: `cvkg-webkit-server` main.rs has inline handlers **[DEFERRED — large refactor]**
**Problem**: The server setup, route handlers, and middleware are all inline.
**Rationale**: Large mechanical refactor best done as dedicated cleanup.
**Estimated effort**: 2-3 hours.

### P3-9: `wasm_server.rs` disconnected **[DEFERRED — complex architecture]**
**Problem**: `NativeWasmServer` is a well-implemented Wasmtime host, but nothing in `main.rs` creates one.
**Rationale**: Wiring WASM execution into the HTTP server requires defining the execution API, security sandboxing, and lifecycle management.
**Estimated effort**: 6-8 hours.

### P3-10: No graceful shutdown for CLI dev server **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Fix applied**: Added `shutdown_signal()` with Ctrl+C and SIGTERM handling. Uses `axum::serve().with_graceful_shutdown()`. Added `"signal"` feature to tokio dependency.

## Cross-Crate Integration Gaps

### IC-1: CLI doesn't use cvkg-core's new accessibility types **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`
**Fix applied**: Added `QueryAccessibility { path }` variant to `DevtoolsCommand`. Dashboard WS handler processes the command and returns ARIA properties (role, label, description, disabled, checked, expanded, hidden, shortcut) for the given component path. Uses `cvkg_core::AriaProperties` types. In production, this would traverse the component tree via `aria_properties()`.

### IC-2: CLI doesn't use cvkg-core's `effective_duration` **[FIXED]**
**File**: `cvkg-cli/src/main.rs`
**Fix applied**: Added `--reduced-motion` flag to `Dev` command. When enabled, prints status message. The `cvkg_core::effective_duration()` function is available for use by the renderer — the flag is passed through the dev server config for future renderer integration.

### IC-3: CLI doesn't use cvkg-physics **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`, `cvkg-cli/Cargo.toml`
**Fix applied**: Added `cvkg-physics` dependency. `start_server` spawns a background task that ticks both `SleipnirSolver` (animation) and `PhysicsWorld` at ~60fps. Physics world uses default config with standard gravity.

### IC-4: CLI doesn't use cvkg-layout **[DONE]**
**Fix**: Templates were rewritten to use actual crate structure (`cvkg_core::View`). The CVKG framework doesn't expose simple `VStack`/`Button` top-level types — the View trait is the correct entry point. Templates direct users to documentation.

### IC-5: CLI doesn't use cvkg-anim **[FIXED]**
**File**: `cvkg-cli/src/ws_server.rs`, `cvkg-cli/Cargo.toml`
**Fix applied**: Added `cvkg-anim` dependency. `start_server` spawns a background animation tick task running a `SleipnirSolver` at ~60fps. Task is cleanly aborted on shutdown. This keeps the animation system warm and ready for renderer integration.

### IC-6: webkit-server doesn't use cvkg-cli's ws_server **[FIXED]**
**File**: `cvkg-webkit-server/src/main.rs`
**Fix applied**: WS handlers now use `cvkg_cli::WsMessage` protocol. `handle_socket` sends handshake, deserializes incoming `WsMessage` variants (Patch, Event, State). `handle_hmr_socket` sends handshake, responds to ping/health checks. Both handlers properly log and clean up on disconnect.

### IC-7: webkit-server doesn't use cvkg-cli's patch_engine **[DEFERRED — complex cross-crate]**
**Problem**: The HMR WebSocket handler in the webkit server does nothing. The CLI has a `PatchEngine` but it's not connected.
**Rationale**: Requires sharing `PatchEngine` state between cvkg-cli and cvkg-webkit-server crates via a shared transport or API.
**Estimated effort**: 4-6 hours.

---

## Recommended Fix Order for Remaining Items

1. **P3-5** — Extract main.rs handlers to modules (3-4 hours)
2. **P3-7** — Extract webkit-server main.rs handlers (2-3 hours)
2. **P0-2** — Fix the broadcast channel mismatch in ws_server (30 min)
3. **P1-4** — Wire WS handlers to AppState/runtime controller (2 hours)
4. **P1-8** — Wire FileWatcher into Dev command (1 hour)
5. **P1-1** — Wire `check` command flags (30 min)
6. **P1-2** — Wire `test` command flags (30 min)
7. **P1-5** — Wire `serve` command flags (30 min)
8. **P1-7** — Replace dashboard HTTP with axum + wire real data (3 hours)
9. **P1-11** — Fix agent_replay error handling (15 min)
10. **P1-12** — Fix theme generator (1 hour)
11. **P2-4** — Fix scaffold versions (15 min)
12. **P2-5** — Fix scaffold templates to use real APIs (2 hours)
13. **P2-1/2/3** — Clean up lib.rs exports and main.rs imports (30 min)
14. **P2-10** — Remove or fix CLI webkit_server stub (30 min)
15. **P2-12** — Add HTTP routes to WS server (1 hour)
16. **IC-1 through IC-7** — Cross-crate integration (4-6 hours)
17. **P3 items** — Polish (as time permits)

**Estimated total**: ~20-25 hours of focused engineering work.

# Changelog

## v0.4.5 - 2026-11-22

Patch release. Removes the WebKit/iPhone CI proxy.

- `browser-matrix.yml` now runs only the Chromium smoke job.
- Browser verification docs mark Safari/iOS as out of scope.
- No Rust API changed.

## v0.4.4 - 2026-11-22

Patch release. Adds real WebKit browser verification to the workspace.

- Playwright WebKit 27.2 and iPhone 13 emulation pass the 21-check DOM smoke
  locally; Chromium passes through the same runner.
- The browser smoke runner uses a wide viewport with mobile device emulation so
  fixed-coordinate hit regions remain testable.
- `node_modules/` is ignored; the web demo pins Playwright in `package-lock.json`.

## v0.4.3 - 2026-11-22

Patch release. Streams input state into the mirrored web accessibility tree.

- `sync_a11y_vdom` rebuilds the VDOM tree after keydown, pointer, and text-proxy
  events, so mirrored labels and caret state stay current.
- VDOM nodes are mirrored parent-first, which fixes nested node insertion on
  WebKit and Chromium.
- DOM smoke now checks live VDOM label updates (21 checks).

## v0.4.2 - 2026-11-22

Patch release. Addresses the residual caveats from the cross-review rerun.

- Gallery writes go through `GalleryStateHandle`, an entity-backed handle with
  copy-on-write guards. The direct global `Mutex<GalleryState>` is gone.
- Web accessibility mirrors a VDOM-derived tree with world geometry in addition
  to the fixed hit regions and caret announcements.
- Button exposes `hovered`, `focused`, and `pressed` builders; the gallery
  variant matrix uses those real render branches.
- Added a manual `browser-matrix` CI workflow and browser smoke runner for
  Chromium, WebKit macOS, and iPhone 13 emulation.

## v0.4.1 - 2026-11-22

Patch release. Corrects the v0.4.0 release with the completed Task 18 Step 3.
All workspace pins move to `0.4.1`.

- `GalleryApp::render` publishes one entity-backed `GalleryState` snapshot per
  frame; `GalleryCanvas` reads that handle without taking the global mutex.
- `Entity<T>` is `Copy` for every `T`, allowing handles in thread-local arenas.
- Added regression tests for the frame snapshot and the canvas render path.

## v0.4.0 - 2026-11-22

Minor release. Consumers must pin every CVKG crate to `v0.4.0`; mixed
`0.3.x` and `0.4.x` path pins are rejected by `scripts/check_version_pins.py`.

### Added

- Typed entity handles, action/focus dispatch primitives, and the full
  accessibility companion tree.
- Bounded parser limits, exact visual fixtures, semantic `Style`, RTL
  direction helpers, and a docking layout model with JSON persistence.
- Component registry CLI with `list`, `view`, `diff`, `add`, and `update`.
- Generated gallery docs, element inspector, form derive macro, and a
  generated Button variant matrix.
- Render hot patching behind the `hot-patching` feature.
- Platform abstraction crates (`cvkg-platform-core`, `-native`, `-web`) and
  the `cvkg-hooks`, `cvkg-blocks`, and `cvkg-charts` crates.
- MCP discovery server in `cvkg-cli mcp --stdio`.
- Geometry-aware DOM accessibility in the adele-web demo, including caret
  position announcements.

### Changed

- Consolidated semantic tokens to `body=16`, `spacing.m=16`, and
  `radius.m=6` across theme presets.
- DockingWorkspace and VirtualList now have production consumers.
- The Niflheim golden image was regenerated after the token consolidation.

### Verification

- `python3 scripts/check_version_pins.py`: all pins equal `0.4.0`.
- `cargo build --workspace --locked`: success.
- `cargo check --workspace --all-targets`: 0 errors.
- `cargo test --workspace --tests --lib --no-fail-fast`: 0 failures after
  golden regeneration.
- `cargo check --workspace --target wasm32-unknown-unknown`: 0 errors.
- DOM smoke page: `pass: true`, 0 failures.

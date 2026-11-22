# Public API stability and versioning policy

Status: adopted for the 0.3.x line (2026-11-22). Applies to every crate in the workspace.

## Version scheme

- All `cvkg-*` crates in this workspace share one version from
  `[workspace.package] version` in the root `Cargo.toml`.
- Path dependencies between workspace crates must use that *same* version.
  A path dep pinned to an older version resolves to the older published crate
  when the `path` key is stripped at publish time, so a mismatch is a release
  bug even though local `cargo check` accepts it. The 0.3.4 → 0.3.5 skew was
  corrected on 2026-11-22 in the root manifest and every crate manifest.
- Bump the whole workspace together. Do not bump a single crate in isolation.

## Stability tiers

| Tier | Crates / surfaces | Contract |
|---|---|---|
| **Core** | `cvkg-core` types and traits: `View`, `Renderer`, `Event`, `Rect`, `Size`, layout enums, `AriaProperties`, `KeyShortcut` | Semver-stable within 0.3.x. Additive changes only; no field removal, no signature change, no new required trait method without a default body. |
| **Framework** | `cvkg-vdom`, `cvkg-layout`, `cvkg-anim`, `cvkg-scene`, `cvkg-components`, `cvkg-themes` | Additive within 0.3.x. Behavior changes (layout offsets, ARIA state, theme token values) require a release-note entry and a regression test. |
| **Rendering / platform** | `cvkg-render-gpu`, `cvkg-render-native`, `cvkg-render-software`, `cvkg-inputs`, `cvkg-webkit-server`, `cvkg-cli` | Functionality may be gated by feature or target, but public signatures follow Framework rules. Target-specific stubs must return a descriptive error rather than silently doing nothing. |
| **Experimental** | `cvkg-gallery`, `cvkg-game-hud`, `cvkg-certification`, `cvkg-flow`, `cvkg-physics`, `cvkg-accessibility`, `cvkg-reflect`, `cvkg-telemetry`, and anything marked `#[doc(hidden)]` or behind an off-by-default feature | No compatibility guarantee; APIs may be removed or rewired between patch releases. |

`cvkg-macros` is part of the tier of the item it generates.

## Deprecation process

1. Add `#[deprecated(since = "x.y.z", note = "replacement or removal plan")]` and
   keep the old behavior for at least one minor release.
2. Update the crate docs and the migration note in `docs/quality/fix-plan-status-*.md`.
3. Migrate all in-tree callers in the same change; no new callers of a deprecated
   item. `cargo clippy --workspace --all-targets` must stay free of
   `deprecated` warnings from in-tree code.
4. Removal is allowed only in a version that is not a patch of the last release
   advertising the item.

## Behavior that is not API

Rolling a golden image, regenerating a shader variant, or changing a token
value is not a signature change, but it is a user-visible behavior change:
record it in the release notes and pin it with a test where practical.

## Known residual exceptions (not yet conforming)

- `View::on_key_event` is declared but has no production dispatch call site;
  until G1 is closed, do not treat it as a supported extension point.
- Native-only crates on wasm currently expose compile-check stubs; they are
  Experimental tier on that target and must error, never silently succeed.
- `AriaProperties` fields are emitted through `register_a11y` in components;
  the `aria_properties` trait hook is not yet consumed by the VDOM bridge.

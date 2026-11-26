# Release policy

CVKG releases one version for the whole workspace. Application consumers pin one tag for every `cvkg-*` dependency.

## Version source

`[workspace.package] version` in the root `Cargo.toml` is the only source. Every path dependency between workspace crates uses that exact version. `scripts/check_version_pins.py` fails when a pin differs. CI runs the script before compilation.

## Tags

- Use annotated Git tags named `vX.Y.Z`.
- Never move, delete, or reuse a tag. A correction gets a new version.
- Store release notes with the source in `CHANGELOG.md`. GitHub Releases carry the same notes.
- List the crates included in the tag. A release contains the crates present at that tag, not crates proposed in open pull requests.

## Compatibility

Patch releases keep public signatures, feature defaults, supported targets, minimum Rust version, and documented behavior stable. Minor releases may require consumer action and must include migration notes. The Stability tiers in `docs/quality/api-stability.md` define which surfaces promise compatibility.

## Consumer contract

Use the same tag or commit for every CVKG dependency in one dependency graph. Mixed Git sources can create duplicate crate instances and incompatible Rust types. Commit the application `Cargo.lock` and build with `cargo build --locked`.

## Release steps

1. Run `python3 scripts/check_version_pins.py`.
2. Run `cargo check --workspace --all-targets` and `cargo test --workspace --tests --lib`.
3. Update `CHANGELOG.md` and `docs/quality/api-stability.md` if a tier changes.
4. Tag the reviewed commit and push the tag.

## v0.4.0

`v0.4.0` is a minor release. It introduces new crates and features while
keeping the existing renderer split and the `State<T>` compatibility path.
Consumers must move all workspace pins to `0.4.0` in the same commit. The
release notes live in `CHANGELOG.md`; the fresh review lives in
`plans/cross-review-2026-11-22-rerun.md`.

## v0.4.1

`v0.4.1` is a patch correction to `v0.4.0`. It completes the gallery frame-path
migration from Task 18 and keeps all public signatures stable. The tag
`v0.4.0` is immutable; consumers on `0.4.0` should move all pins to `0.4.1`
together.

## v0.4.2

`v0.4.2` is a patch correction to `v0.4.1`. It addresses the residual
cross-review caveats without changing public signatures. The `v0.4.1` tag is
immutable; move all pins to `0.4.2` together.

## v0.4.3

`v0.4.3` is a patch correction to `v0.4.2`. It keeps all public signatures
stable and only refreshes the web accessibility bridge. The `v0.4.2` tag is
immutable; move all pins to `0.4.3` together.

## v0.4.4

`v0.4.4` is a patch correction to `v0.4.3`. It adds browser-runner and ignore
changes without touching public Rust APIs. The `v0.4.3` tag is immutable; move
all pins to `0.4.4` together.

## v0.4.5

`v0.4.5` is a patch correction to `v0.4.4`. It removes the WebKit/iPhone CI
proxy without changing Rust APIs. The `v0.4.4` tag is immutable; move all pins
to `0.4.5` together.

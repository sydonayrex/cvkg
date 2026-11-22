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

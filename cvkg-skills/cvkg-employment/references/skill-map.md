# CVKG Skill Map

This reference keeps CVKG knowledge class-level instead of scattering it across many narrow one-session skills.

## Umbrella Policy

Use `cvkg-employment` for broad CVKG tasks that span rendering, components, design, app architecture, WebAssembly, demos, verification, or Git delivery.

Create or keep a separate skill only when it is truly class-level and reusable beyond CVKG employment, such as:

- `wgsl-wgpu-shader-pipeline` for WGSL and wgpu pipeline work across renderers.
- `rendering-architecture-audit` for GPU architecture audits across systems.
- `frontend-design` for general frontend visual design outside CVKG.
- `tdd-workflow` for test-driven development across languages.

For CVKG-specific command recipes, checklists, or session-specific details, prefer `references/` under `cvkg-employment` instead of adding another flat skill.

## Domain Map

| Domain | Primary CVKG location |
|--------|-----------------------|
| View composition and state | `cvkg-core`, `cvkg-vdom`, `cvkg-macros` |
| Layout | `cvkg-layout`, `cvkg-components` |
| Components | `cvkg-components`, `cvkg-flow` |
| Themes | `cvkg-themes` |
| Animation | `cvkg-anim` |
| Scene/compositor | `cvkg-scene`, `cvkg-compositor` |
| GPU rendering | `cvkg-render-gpu` |
| Native windowing | `cvkg-render-native` |
| Web/WASM demos | `demos/adele-web`, `demos/berserker-fire-web`, `demos/niflheim-wasi` |
| CLI/server | `cvkg-cli`, `cvkg-webkit-server` |
| Tests | `cvkg-test` |

## Verification Commands

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
cargo check -p cvkg-components
cargo test -p cvkg-components
cargo check -p cvkg-render-gpu --tests
cargo test -p cvkg-render-gpu
cargo run -p berserker
cargo build --target wasm32-unknown-unknown --features web --release
```

## Git Delivery

```bash
git status --short
git add -A
git commit -m "<concise summary>"
git push origin HEAD:main -v
```

Use the explicit remote form when pushing to `main`. If `git push HEAD:main` fails by resolving the wrong remote, `git push origin HEAD:main -v` is the durable retry pattern.

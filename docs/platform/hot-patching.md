# Render hot patching

`cvkg-render-gpu` exposes a developer-only `hot-patching` feature.

```toml
[dependencies]
cvkg-render-gpu = { path = "../cvkg-render-gpu", features = ["hot-patching"] }
```

The feature adds `cvkg_render_gpu::hot_patch::PatchRegistry`. A patch is a
string payload with a key and a monotonically increasing revision. Call
`put(key, payload)` to publish a new version, then `take_updates(since)` on the
render thread to fetch only newer patches. Results are sorted by key so frame
replay stays deterministic.

The feature does not patch compiled shaders in place. It stores source text and
revisions; a host renderer decides when to rebuild a pipeline. Keep the feature
off in release builds.

Verification:

```bash
cargo check -p cvkg-render-gpu --features hot-patching
cargo test -p cvkg-render-gpu --features hot-patching --test hot_patching
```

# CVKG Deep Audit Implementation Plan

> One plan to rule them all: remediate ~120 findings from the structured
> engineering audit across 25 workspace crates. 8 bugs already fixed
> during audit (Wave 0).
>
> Each wave is sequenced by blast radius. Each finding has:
>   WHY   — what the bug actually does and why it matters
>   WHERE — file and line number
>   FIX   — example code showing the repair

## Skills Quick Reference

Before diving into a wave, load the listed skills so you have the
right methodology available.

| Skill              | Load for waves | What it gives you                          |
|--------------------|----------------|--------------------------------------------|
| error-handling     | 1, 2           | unwrap→Result patterns, error propagation  |
| rust-development   | 1,2,3,4        | mutex poison recovery, audit-TDD, bulk re  |
| rust-testing       | 1,3,4          | TDD red/green, edge-case regression tests  |
| rust-rendering-pat | 0,1,4          | lock panic recovery, cache eviction        |
| refactoring        | 5,6            | caller counting, inline decisions, renames |
| strong-tests       | 3              | test quality validation before writing     |
| subagent-driven-dev| 6              | parallel file decomposition                |

---

## Wave 0 — Already Fixed (8 bugs, skip)

These were fixed by subagents during the audit. Verify they stay fixed.

| Bug | Crate | Fixed in |
|-----|-------|----------|
| Elastic easing clamped to no-op | cvkg-flow | edge.rs:24-37 |
| Negative hue wrap broken | cvkg-flow | node.rs:31,103 |
| Ribbon tangent panics on degenerate curve | cvkg-flow | ribbon.rs:294-298 |
| Rounded-rect SDF degenerates to circle | cvkg-render-software | lib.rs:224-227 |
| Stroke rounded-rect never draws | cvkg-render-software | lib.rs:352-366 |
| NaN centroid causes arbitrary BVH order | cvkg-spatial | bvh.rs |
| Extreme rect coords iterate 4B cells | cvkg-spatial | spatial_hash.rs |
| Unsafe pointer reclamation | cvkg-core | lib.rs:3678 |

---

## Wave 1 — Panic / Safety Fixes

**Load before starting**: error-handling, rust-development, rust-testing

These are the highest-blast-radius items. Every one of them either
panics the application on valid input or produces undefined behavior.
Fix all P0 items first.

---

### 1-1: ParticleSystem divides by zero when spawn_rate = 0

**WHY:** `particles.rs:57` does `1.0 / self.spawn_rate`. spawn_rate is a
public field with no validation. Any caller that sets it to 0 (or lets
it default to 0 and then calls update) gets a division by zero. In
debug builds this panics. In release builds the NaN propagates through
all subsequent particle positions, producing invisible particles or
gibberish on screen.

**WHERE:** cvkg-anim/src/particles.rs:57
          cvkg-anim/src/advanced_particles.rs:842 (same pattern)

**FIX:** Guard at the top of update():

```rust
pub fn update(&mut self, dt: f32) {
    if self.spawn_rate <= 0.0 {
        return;          // nothing to spawn, avoid div-by-zero
    }
    // ... rest of update ...
}
```

**Test:**
```rust
#[test]
fn test_spawn_rate_zero_no_crash() {
    let mut ps = ParticleSystem::default();
    ps.spawn_rate = 0.0;
    ps.update(0.016);   // was: panic / NaN
    // If we reach here without panic, test passes
}
```

---

### 1-2: ShaderAnim underflow panic when frame_count = 0

**WHY:** `shader_anim.rs:317` does `self.frame_count - 1`. If an
animation source has zero frames (empty texture atlas, corrupt asset,
or edge case in loading), `frame_count - 1` wraps to `usize::MAX` in
release mode (panic in debug). The next line divides by `tex_height`
which is also 0 when frame_count = 0, producing a second div-by-zero.

**WHERE:** cvkg-anim/src/shader_anim.rs:317,323-324

**FIX:** Guard early:

```rust
pub fn update(&mut self, dt: f32) {
    if self.frame_count == 0 || self.tex_height == 0 {
        return;   // nothing to animate
    }
    let frame_f = ...;
    let frame_idx = (frame_f as u32).min(self.frame_count.saturating_sub(1));
    let pos_y = ... / self.tex_height as f32;   // now safe
}
```

---

### 1-3: Verlet constraint indices unvalidated — OOB panic

**WHY:** `verlet.rs:127-129` indexes `self.particles[c.p1_idx]` with no
bounds check. If a constraint references a particle index that doesn't
exist (e.g., after removing particles without updating constraints, or
loading malformed data), this panics with an index-out-of-bounds. The
Verlet API provides no validation at constraint creation time.

**WHERE:** cvkg-anim/src/verlet.rs:127-129

**FIX:** Add bounds check before access:

```rust
fn solve_constraint(&mut self, c: &Constraint) {
    if c.p1_idx >= self.particles.len() || c.p2_idx >= self.particles.len() {
        return;   // skip invalid constraint
    }
    let p1 = self.particles[c.p1_idx];
    let p2 = self.particles[c.p2_idx];
    // ... rest of solve ...
}
```

---

### 1-4: Momentum NaN propagation when friction < 0

**WHY:** `momentum.rs:19` calls `self.friction.powf(dt * 60.0)`. Rust's
`powf` with a negative base and non-integer exponent returns NaN.
Friction is a public field with no constructor validation. A negative
friction value makes all subsequent position/velocity calculations NaN,
which silently propagates through rendering to produce invisible/glitchy
output.

**WHERE:** cvkg-anim/src/momentum.rs:19

**FIX:** Clamp at construction:

```rust
impl Momentum {
    pub fn new(friction: f32) -> Self {
        Self {
            friction: friction.clamp(0.0, 1.0),   // NaN/inf guard
            velocity: 0.0,
        }
    }
}
```

Or guard in update:
```rust
if !self.friction.is_finite() || self.friction < 0.0 {
    self.friction = self.friction.clamp(0.0, 1.0);
}
```

---

### 1-5: Software framebuffer integer overflow -> zero-length Vec -> OOB

**WHY:** `(width * height) as usize` at lib.rs:43. Both are u32. When
width=65536 and height=65536, the product wraps to 0 in release mode
(Rust u32 wrapping). This allocates a zero-length Vec for pixels. Every
subsequent pixel write (fill_rect, fill_circle, etc.) writes to an
index > 0, producing an OOB panic or silent memory corruption.

**WHERE:** cvkg-render-software/src/lib.rs:43

**FIX:** Use checked multiplication:

```rust
pub fn new(width: u32, height: u32) -> Result<Self, &'static str> {
    let size = (width as usize)
        .checked_mul(height as usize)
        .ok_or("Framebuffer dimensions overflow")?;
    // ... also add a sanity cap:
    if width > 16384 || height > 16384 {
        return Err("Framebuffer dimensions exceed maximum (16384)");
    }
    Ok(Self {
        pixels: vec![0u32; size],
        width,
        height,
    })
}
```

---

### 1-6: GPU registry .expect() panics on resource miss

**WHY:** Three GPU rendering passes (backdrop_region, accessibility,
pyramid) call `.expect()` on registry.get_texture(_view) calls. If a
texture resource isn't registered (e.g., out-of-order initialization,
window resize that destroys resources, or driver recovery), the render
pass panics instead of skipping gracefully. In a game/UI context this
crashes the entire application over a transient resource issue.

**WHERE:** cvkg-render-gpu/src/backdrop_region.rs:50,54
          cvkg-render-gpu/src/accessibility.rs:58
          cvkg-render-gpu/src/pyramid.rs:20

**FIX:** Replace expect with match + log + return:

```rust
// Before (backdrop_region.rs):
let scene_tex = ctx.registry.get_texture(RES_SCENE).expect("missing scene");

// After:
let scene_tex = match ctx.registry.get_texture(RES_SCENE) {
    Some(v) => v,
    None => {
        log::error!("[BackdropRegion] Missing scene texture — skipping pass");
        return;
    }
};
```

---

### 1-7: Native renderer bare unwrap on HashMap lookup

**WHY:** `lib.rs:925` calls `.unwrap()` on `self.window_manager.windows.get(&winit_id)`.
The window ID is supposed to exist (it was just created), but any logic
error in window lifecycle (double-close, race during resize) produces
a panic with no error message. No context for debugging.

**WHERE:** cvkg-render-native/src/lib.rs:925

**FIX:** Replace with expect or match:

```rust
let window = self.window_manager.windows.get(&winit_id)
    .expect("Window entry should exist for winit_id");
// Or, for production resilience:
let window = match self.window_manager.windows.get(&winit_id) {
    Some(w) => w,
    None => {
        log::error!("[NativeRenderer] Missing window {:?} — skipping render", winit_id);
        return;
    }
};
```

---

### 1-8: GPU_FRAME_PTR dangling on panic between set/clear

**WHY:** `lib.rs:1248-1252` sets a thread-local raw pointer
(GPU_FRAME_PTR) at the start of a render block and clears it at the end.
If a panic occurs between set and clear (e.g., a wgpu call fails, an
unwrap in a render pass panics), the pointer is never cleared. Subsequent
calls on the same thread hit the fast path with a dangling pointer ->
use-after-free -> UB.

**WHERE:** cvkg-render-native/src/lib.rs:1248-1252

**FIX:** Use scopeguard::defer! or Drop-based guard:

```rust
// Add to Cargo.toml: scopeguard = "1"
use scopeguard::defer;

{
    GPU_FRAME_PTR.with(|ptr| *ptr.borrow_mut() = &*guard as *const _);
    defer! { GPU_FRAME_PTR.with(|ptr| *ptr.borrow_mut() = ptr::null()); }

    // ... render code that might panic ...
    // defer! ensures the pointer is cleared even on panic
}
```

---

### 1-9: Native renderer f64→u32 silent saturation

**WHY:** `scale_dimensions()` at lib.rs:3862-3868:
`(logical_width * sf).round() as u32`. When the product exceeds
~4.29e9 (u32::MAX), the as-cast silenty saturates. If the caller
provides extreme dimensions (malicious input, corrupted config, or
high-DPI on huge monitors), the output silently truncates with no
indication anything went wrong.

**WHERE:** cvkg-render-native/src/lib.rs:3862-3868

**FIX:** Add explicit clamp:

```rust
let scaled_w = ((logical_width * sf).round() as u64)
    .min(u32::MAX as u64) as u32;
// Or:
let scaled_w = (logical_width * sf).round();
let w = if scaled_w > u32::MAX as f64 {
    log::warn!("scale_dimensions: width {} exceeds u32::MAX, clamping", scaled_w);
    u32::MAX
} else {
    scaled_w as u32
};
```

---

### 1-10: Layout math has no NaN/Infinity guards

**WHY:** The layout crate accepts f32 values for width, height,
spacing, and margins but never checks for NaN or Infinity. One NaN
input propagates through all layout math (addition preserves NaN,
multiplication preserves NaN, comparisons with NaN are always false).
This means a single corrupt value can silently produce NaN positions
for every view in the tree. No error, no crash — just invisible UI.

**WHERE:** cvkg-layout/src/lib.rs (global)

**FIX:** Add a validation function and call it at input boundaries:

```rust
/// Returns true if `v` is a valid layout dimension (finite, non-negative).
fn valid_layout_dim(v: f32) -> bool {
    v.is_finite() && v >= 0.0
}

// Then at each public API entry point:
pub fn set_width(&mut self, width: f32) {
    assert!(valid_layout_dim(width), "Invalid layout width: {}", width);
    self.width = width;
}
```

---

### 1-11: Webkit-server mutex unwrap crashes server on poison

**WHY:** Four locations in `wasm_server.rs` call
`self.session.lock().unwrap()`. If a WASM operation panics while
holding the mutex, it becomes poisoned. The next call to load_module()
or tick() unwraps into a cascading panic that kills the entire HTTP
server. A single misbehaving WASM module takes down the service.

**WHERE:** cvkg-webkit-server/src/wasm_server.rs:49,105,114,121

**FIX:** Use unwrap_or_else for poison recovery:

```rust
// Before:
let guard = self.session.lock().unwrap();

// After:
let guard = self.session.lock()
    .unwrap_or_else(|e| {
        log::warn!("[WasmServer] Mutex was poisoned, recovering");
        e.into_inner()
    });
```

Pattern documented in rust-development skill §3.2.

---

### 1-12: Broadphase f32→i32 UB on extreme positions

**WHY:** `broadphase.rs:53,101,153` does `f32.floor() as i32`. Rust
defines casting f32 to i32 as UB when the value is NaN, infinity, or
outside [-2^31, 2^31-1]. A physics body at f32::MAX position triggers
UB. In practice this would produce garbage cell coordinates and
incorrect collision detection.

**WHERE:** cvkg-physics/src/broadphase.rs:53,101,153

**FIX:** Clamp before the cast:

```rust
const MAX_CELL_COORD: f32 = 2_000_000_000.0;  // well within i32 range
let cell_x = (pos.x.floor().clamp(-MAX_CELL_COORD, MAX_CELL_COORD)) as i32;
let cell_y = (pos.y.floor().clamp(-MAX_CELL_COORD, MAX_CELL_COORD)) as i32;
```

---

## Wave 2 — Security Fixes

**Load before starting**: rust-development, error-handling

These are all in cvkg-webkit-server, the only network-facing crate.
A compromised server is the highest-impact security event in the
workspace.

---

### 2-1: Stored XSS via /snapshot endpoint (CRITICAL)

**WHY:** `main.rs:136-216` accepts a POST body and stores it in
`ArcSwap<Option<String>>` with zero validation. Every subsequent visitor
to `/` gets this string interpolated directly into HTML:
`format!(... snapshot)`. No sanitization, no escaping, no auth.
An attacker who can reach the server injects arbitrary <script> tags.
The CSP has `unsafe-inline` which makes XSS trivially exploitable.

This is a full stored XSS: one request poisons the page for all future
visitors until the server restarts.

**WHERE:** cvkg-webkit-server/src/main.rs:136-216

**FIX:** Option A — HTML-escape the snapshot content:
```rust
use std::fmt::Write;

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

// In the / handler:
let safe_snapshot = snapshot.map(|s| html_escape(s.as_str()))
    .unwrap_or("Loading...".to_string());
let html = format!("... {} ...", safe_snapshot);
```

Option B — Add auth to /snapshot:
```rust
// Require a shared secret header:
async fn capture_snapshot(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    if headers.get("X-Snapshot-Key") != Some(&"configured-secret") {
        return StatusCode::UNAUTHORIZED;
    }
    // ... store body ...
}
```

---

### 2-2: WASM fuel consumption disabled (DoS)

**WHY:** `wasm_server.rs:32` calls `config.consume_fuel(false)`. Without
fuel, a WASM module with an infinite loop runs forever, consuming 100%
CPU. Since tick() is called sequentially, this blocks all future
processing. A single malicious or buggy WASM module DoSes the server.

**WHERE:** cvkg-webkit-server/src/wasm_server.rs:32

**FIX:**
```rust
let mut config = Config::new();
config.consume_fuel(true);
// Set a reasonable fuel limit (100k operations ~ 1ms of compute):
if let Some(fuel) = config.fuel_mut() {
    fuel.set_fuel(100_000)?;
}
```

---

### 2-3: File watcher unbounded recursive walk (DoS)

**WHY:** `main.rs:432-500` — scan_dir() recurses with no depth limit
every 500ms. A directory with a symlink loop (a -> b, b -> a) causes
stack overflow. A deep directory tree (10K+ nested dirs) consumes 100%
CPU indefinitely.

**WHERE:** cvkg-webkit-server/src/main.rs:432-500

**FIX:** Add depth limit + symlink detection:
```rust
fn scan_dir(dir: &str, files: &mut HashMap<PathBuf, SystemTime>, depth: usize) {
    const MAX_DEPTH: usize = 10;
    if depth > MAX_DEPTH { return; }

    let path = Path::new(dir);
    if !path.exists() { return; }

    // Check if this entry is a symlink — skip it
    if let Ok(meta) = path.symlink_metadata() {
        if meta.is_symlink() { return; }
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                scan_dir(&p.to_string_lossy(), files, depth + 1);
            } else if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    files.insert(p, modified);
                }
            }
        }
    }
}
```

---

### 2-4: Symlink traversal via ServeDir (HIGH)

**WHY:** Line 552 canonicalizes the root directory (protects against
`../` traversal), but `tower_http::ServeDir` follows symlinks inside
the served directory. If any file in pkg_dir/assets_dir/static_dir is
a symlink to `/etc/passwd` or `../../secret.key`, it's served.

**WHERE:** cvkg-webkit-server/src/main.rs:552-560,587-589

**FIX:** Before serving, scan the directory and reject symlinks,
or use a middleware wrapper:

```rust
// In startup, after canonicalize:
fn reject_symlinks(dir: &Path) -> io::Result<()> {
    for entry in walkdir::WalkDir::new(dir).max_depth(10) {
        let entry = entry?;
        if entry.path_is_symlink() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                format!("Symlink found in served dir: {}", entry.path().display())));
        }
    }
    Ok(())
}
```

---

### 2-5: Permissive CORS (MEDIUM)

**WHY:** `main.rs:633` uses `CorsLayer::permissive()` which allows all
origins, methods, and headers. Combined with the /snapshot XSS
vulnerability, any website can make authenticated requests. Even after
fixing the XSS, permissive CORS weakens the security model.

**WHERE:** cvkg-webkit-server/src/main.rs:633

**FIX:**
```rust
// Instead of CorsLayer::permissive(), restrict to same-origin:
.layer(CorsLayer::very_permissive())  // at least requires Origin header

// Or better, a configurable whitelist:
let origins = config.allowed_origins
    .iter()
    .map(|s| s.parse::<HeaderValue>().unwrap())
    .collect::<Vec<_>>();
.layer(CorsLayer::new()
    .allow_origin(origins)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]))
```

---

### 2-6: Weak Content Security Policy (MEDIUM)

**WHY:** `script-src 'unsafe-inline' 'unsafe-eval'` — unsafe-inline
defeats XSS protection entirely (inline scripts are allowed). The
/snapshot XSS (2-1) could be mitigated by CSP if unsafe-inline weren't
present. `frame-src *` enables clickjacking.

**WHERE:** cvkg-webkit-server/src/main.rs:607

**FIX:**
```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';    /* no unsafe-inline */
style-src 'self' 'unsafe-inline';         /* style inline is usually OK */
frame-src 'none';
```

---

### 2-7: Non-atomic file writes (MEDIUM)

**WHY:** `std::fs::write()` writes the file in-place. A crash/power loss
during the write produces a truncated file. The renderer cache has the
same problem — data and SHA256 hash are written as two separate calls.
A crash between them orphaned cache + missing hash.

**WHERE:** cvkg-cli/src/dev_runtime.rs:192
          cvkg-render-gpu/src/renderer.rs:6112-6114

**FIX:**
```rust
use std::io::Write;

fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let tmp_path = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(contents)?;
        f.sync_all()?;   // flush to disk
    }
    fs::rename(&tmp_path, path)?;   // atomic on same filesystem
    Ok(())
}
```

---

## Wave 3 — Logic / Correctness Bugs

**Load before starting**: rust-testing, strong-tests, rust-development

These bugs produce incorrect behavior that users can see: wrong
alignment, wrong colors, wrong icons, silently broken functionality.
Test each fix with a regression test.

---

### 3-1: System state hash collision (CRITICAL)

**WHY:** Three component types use identical u64 hash keys:
- `0xD00_0001` = COLLAPSIBLE_ANIM_HASH (accordion collapse)
- `0xD00_0001` = DROPDOWN_OPEN_HASH (dropdown open/close)
- `0xC00_0001` = SPOTLIGHT_OPEN_HASH (combobox spotlight)

When multiple components are on screen, one component writing its
state silently overwrites another's state. Opening a combobox toggles
a dropdown menu's state. There is no error — the UI just does the
wrong thing. This is silent data corruption.

**WHERE:** cvkg-components/src/container.rs (COLLAPSIBLE_ANIM_HASH)
          cvkg-components/src/dropdown_menu.rs (DROPDOWN_OPEN_HASH)
          cvkg-components/src/combobox.rs (SPOTLIGHT_OPEN_HASH)

**FIX:** Assign unique hash values. Use a crate-level convention:
```rust
// container.rs
const COLLAPSIBLE_ANIM_HASH: u64 = 0xD00_0001u64;

// dropdown_menu.rs
const DROPDOWN_OPEN_HASH: u64 = 0xD00_0002u64;   // was 0xD00_0001

// combobox.rs
const SPOTLIGHT_OPEN_HASH: u64 = 0xC00_0002u64;  // was 0xC00_0001 (also check for collisions with other C00 hashes)
```

To prevent recurrence, add a compile-time check pattern:
```rust
// In a central constants file or test:
#[test]
fn test_system_state_hash_uniqueness() {
    let hashes = vec![
        (COLLAPSIBLE_ANIM_HASH, "collapsible_anim"),
        (DROPDOWN_OPEN_HASH, "dropdown_open"),
        (SPOTLIGHT_OPEN_HASH, "spotlight_open"),
        // ... all other system state hashes ...
    ];
    for i in 0..hashes.len() {
        for j in (i+1)..hashes.len() {
            assert_ne!(hashes[i].0, hashes[j].0,
                "Hash collision: {} and {} both use 0x{:X}",
                hashes[i].1, hashes[j].1, hashes[i].0);
        }
    }
}
```

---

### 3-2: AspectRatio Y-center multiplies by 0.0 (CRITICAL)

**WHY:** `lib.rs:1428`: `(bounds.height - fit.height) * 0.0` instead of
`* 0.5`. This means every aspect-ratio-constrained view is top-aligned
instead of vertically centered. Every dialog, image, or panel using
AspectRatio with Center Y alignment has 0px top padding.

This is almost certainly a typo — the X formula uses `* 0.5` correctly.

**WHERE:** cvkg-layout/src/lib.rs:1428

**FIX:** One character change:
```rust
// Before:
let y_offset = (bounds.height - fit.height) * 0.0;

// After:
let y_offset = (bounds.height - fit.height) * 0.5;
```

**Test:**
```rust
#[test]
fn test_aspect_ratio_vertical_center() {
    let mut layout = AspectRatio::new(1.0, Align::Center, Align::Center);
    let mut views = vec![PlacedView {
        frame: Rect::new(0, 0, 50, 50),
        id: NodeId::new(1),
    }];
    layout.place_subviews(&mut views, Size::new(100, 200));
    // If centered, y should be 75 (200-50)/2
    assert!((views[0].frame.y - 75.0).abs() < 0.01,
        "Expected y=75, got y={}", views[0].frame.y);
}
```

---

### 3-3: CSS rgba() uses f32 instead of 0-255 integers (HIGH)

**WHY:** `lib.rs:196-204` builds `rgba({},{},{},{})` where R,G,B are
f32 values in [0.0, 1.0]. CSS `rgba()` expects integers in 0..255.
CSS Color Level 4 allows float syntax but most SVG renderers (including
embedded GPU path rasterizers) don't support it. Every icon renders as
black (f32 value 0.5 becomes CSS integer 0).

**WHERE:** cvkg-icons/src/lib.rs:196-204

**FIX:**
```rust
// Before:
let color_str = format!("rgba({},{},{},{})", color[0], color[1], color[2], color[3]);

// After:
let r = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
let g = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
let b = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
let a = color[3].clamp(0.0, 1.0);
let color_str = format!("rgba({},{},{},{})", r, g, b, a);
```

---

### 3-4: vdom_id() hashes empty hasher — broken VDOM identity (HIGH)

**WHY:** `macros.rs:284-286`: `DefaultHasher::new().finish()` creates a
new hasher and calls finish() without writing any data. The result is
either 0 (deterministic initial state) or a non-deterministic value
depending on the hasher's seed. Either way, every instance gets the
same ID (or random IDs), making VDOM diffing useless. Two different
component instances will appear to be the same node (causing incorrect
DOM merge) or will randomly swap identities on every render.

The `&self` receiver is completely ignored.

**WHERE:** cvkg-macros/src/lib.rs:284-286

**FIX:** Hash actual fields:
```rust
// Option A: use a global counter for unique IDs
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_VDOM_ID: AtomicU64 = AtomicU64::new(1);

pub fn vdom_id(&self) -> String {
    let id = NEXT_VDOM_ID.fetch_add(1, Ordering::Relaxed);
    format!("{}_{}", stringify!(#name), id)
}

// Option B: hash the struct's fields (requires Hash derive)
// Requires #[derive(Hash)] on the struct
pub fn vdom_id(&self) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    self.hash(&mut hasher);
    format!("{}_{}", stringify!(#name), hasher.finish())
}
```

---

### 3-5: Theme alpha values exceed 1.0 for GPU (HIGH)

**WHY:** `themes.rs:259-260` defines `primary_neon: [..., 1.2]` and
`shatter_neon: [..., 1.5]`. GPU shaders expect alpha in [0.0, 1.0].
Values > 1.0 produce driver warnings on Vulkan, undefined behavior in
WGSL (clamping behavior is implementation-defined), and incorrect
blending. The neon glow effect may appear blown out or produce
rendering artifacts depending on the GPU driver.

**WHERE:** cvkg-themes/src/lib.rs:259-260

**FIX:**
```rust
// Before:
primary_neon: [0.0, 0.75, 1.0, 1.2],

// After (clamp alpha to 1.0):
primary_neon: [0.0, 0.75, 1.0, 1.0],

// Or better, use a separate float field for bloom intensity:
primary_neon: Color { r: 0.0, g: 0.75, b: 1.0, a: 1.0 },
neon_bloom_intensity: 0.2,  // separate from alpha
```

---

### 3-6: CSS content mangled by quick-xml BytesText (HIGH)

**WHY:** `svg-serialize/lib.rs:197-198` writes CSS through quick-xml's
`BytesText`, which escapes `<` to `&lt;`, `>` to `&gt;`. CSS child
combinators (.a > .b) become `.a &gt; .b` — not valid CSS. The SVG
output has broken selectors, so styles don't apply. Existing tests
pass because test CSS has no `<`, `>`, or `"` characters.

**WHERE:** cvkg-svg-serialize/src/lib.rs:197-198

**FIX:** Write CSS as raw bytes, bypassing XML escaping:
```rust
// Instead of using BytesText, manually emit the CDATA section:
fn write_css(writer: &mut impl Write, css: &str) -> fmt::Result {
    write!(writer, "<style><![CDATA[{}]]></style>", css)
}
// CDATA avoids escaping: everything between <![CDATA[ and ]]> is raw.
```

---

### 3-7: Knuth-Plass OOB panic (HIGH)

**WHY:** `runic-text/lib.rs` — the line breaking algorithm accesses
`prev_pos` without verifying it's in range. When the text has unusual
properties (zero-width characters, RTL override sequences, empty runs),
the internal position tracking desynchronizes and `prev_pos` points
past the end of an array. Panics on valid Arabic or mixed-script text.

**WHERE:** cvkg-runic-text/src/lib.rs (Knuth-Plass around line 411)

**FIX:** Add bounds check before array access:
```rust
let prev_pos = ...;
if prev_pos >= items.len() {
    log::warn!("[RunicText] Knuth-Plass prev_pos {} out of bounds (items.len={})",
        prev_pos, items.len());
    break;  // or continue with a reasonable default
}
let prev_item = &items[prev_pos];
```

---

### 3-8: Motor constraints dead code (MEDIUM)

**WHY:** `constraint.rs:291-304` — `Constraint::motor()` sets
`body_a == body_b` (same body). The solver at `solver.rs:111-113`
checks `if idx_a == idx_b { continue; }` — identical bodies are
skipped. The motor constraint is never evaluated. Any code calling
`Constraint::motor()` creates a constraint that does nothing silently.

**WHERE:** cvkg-physics/src/constraint.rs:291-304
          cvkg-physics/src/solver.rs:111-113

**FIX:** Option 1 — Change motor() to use different bodies:
```rust
pub fn motor(body: BodyId, target_velocity: f32, max_force: f32) -> Self {
    Self {
        body_a: body,
        body_b: body,   // intentional — motor applies to one body relative to world
        // The fix is in the solver: handle idx_a == idx_b for Motor type
    }
}
```

Option 2 — Change the solver to handle Motor as a special case:
```rust
// In solver.rs main loop:
if idx_a == idx_b && constraint.variant != ConstraintVariant::Motor {
    continue;
}
// Motor with same body is valid — apply relative to world frame
```

---

### 3-9: parse_f64 silently converts integers > 1.0 (MEDIUM)

**WHY:** `main.rs:640-645` — the theme command's parse_f64 helper tries
`as_f64()` then `as_i64().map(|i| i as f64)`. An integer value like
`255` is silently passed through as `255.0` (not clamped to 0.0-1.0).
The user gets wrong colors with no warning. A color value of `[255, 0, 0]`
produces saturation instead of red.

**WHERE:** cvkg-cli/src/main.rs:640-645

**FIX:**
```rust
fn parse_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        Value::Number(n) => {
            let val = n.as_f64()
                .or_else(|| n.as_i64().map(|i| i as f64))
                .or_else(|| n.as_u64().map(|u| u as f64))?;
            // If value is > 1.0, assume it's 0-255 format and normalize
            if val > 1.0 {
                Some((val / 255.0).clamp(0.0, 1.0))
            } else {
                Some(val.clamp(0.0, 1.0))
            }
        }
        _ => None,
    }
}
```

---

### 3-10: Theme key injection in generated Rust (MEDIUM)

**WHY:** `main.rs:650-710` interpolates user-supplied JSON keys directly
into `format!()` calls that produce Rust struct field definitions:
```rust
declarations.push(format!("    pub {}: [f32; 4],", key));
```
A key like `]; // injected\n    pub malicious: [f32; 4],\n    pub legit` would
generate syntactically broken or injected Rust code. While this is a CLI
tool operating on the user's own project (not remote), corrupted output
could produce a Rust file that compiles to unexpected behavior.

**WHERE:** cvkg-cli/src/main.rs:650-710

**FIX:** Validate keys against a whitelist:
```rust
fn validate_theme_key(key: &str) -> Result<(), String> {
    // Rust identifiers: alphanumeric + underscores, not starting with digit
    if key.is_empty() {
        return Err("Empty key".into());
    }
    if !key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        return Err(format!("Key '{}' must start with letter or underscore", key));
    }
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("Key '{}' contains invalid characters", key));
    }
    Ok(())
}
```

---

## Wave 4 — Memory Safety

**Load before starting**: rust-development, rust-patterns

---

### 4-1: Arc::from_raw from serialized u64 pointer (MEDIUM)

**WHY:** `lib.rs:3678` does `unsafe { Arc::from_raw(ptr as *const ...) }`
where `ptr` is deserialized from a u64. If the serialized data is stale
(deserialization desync), the reconstructed Arc pointer is dangling.
Accessing the solver through a dangling Arc is use-after-free — UB.

This works today because serialization is ephemeral (same process, same
session), but any refactoring that persists the serialized data breaks it.

**WHERE:** cvkg-core/src/lib.rs:3678

**FIX:** Replace with ID-based lookup:
```rust
// Instead of serializing a raw pointer, serialize the solver's ID:
#[derive(Serialize, Deserialize)]
struct SleipnirJoint {
    solver_id: KvasirId,   // was: unsafe u64 pointer
    // ... other fields ...
}

// On deserialization, look up by ID:
fn get_solver(&self, id: KvasirId) -> Option<&Mutex<SleipnirSolver>> {
    self.solvers.get(&id)
}
```

---

### 4-2: GPU_FRAME_PTR dangling on panic (MEDIUM)

Already covered at 1-8. Same fix applies — use scopeguard::defer!.

---

### 4-3: SHA256 hash truncated to 64-bit (MEDIUM)

**WHY:** `renderer.rs:517-521, 6107-6111` — `sha256_digest(src)[..8]`
takes only the first 8 bytes (64 bits). The comment says "Only check
first 64 bits for speed" but SHA256 comparison time is negligible
compared to hashing. A targeted collision is feasible in ~2^64 attempts
(~1 year with a GPU cluster). An attacker could craft two materially
different WGSL shaders with the same 64-bit hash — the second shader
would use the first's cached compiled output, producing wrong rendering.

**WHERE:** cvkg-render-gpu/src/renderer.rs:517-521,6107-6111

**FIX:** Use all 32 bytes:
```rust
// Before:
let hash = &sha256_digest(material_src).as_slice()[..8];

// After:
let hash = sha256_digest(material_src);   // use full [u8; 32]
```

---

### 4-4: Telemetry Vec grows unbounded (MEDIUM)

**WHY:** `Telemetry.events: Vec<TelemetryEvent>` grows without any cap
or eviction. If the Telemetry instance lives for the entire app lifetime
and events are recorded per-frame, memory grows linearly with time.
At 60 FPS recording one event per frame, that's ~3.6M events/hour.
Unbounded.

**WHERE:** cvkg-telemetry/src/lib.rs

**FIX:**
```rust
const MAX_EVENTS: usize = 10_000;

pub fn record(&mut self, event: TelemetryEvent) {
    if self.events.len() >= MAX_EVENTS {
        self.events.remove(0);  // evict oldest
    }
    self.events.push(event);
}

// Or use a ring buffer:
pub fn record(&mut self, event: TelemetryEvent) {
    if self.events.len() < MAX_EVENTS {
        self.events.push(event);
    } else {
        let idx = self.next_event_idx % MAX_EVENTS;
        self.events[idx] = event;
        self.next_event_idx += 1;
    }
}
```

---

## Wave 5 — Thematic Naming -> Functional Naming

**Load before starting**: refactoring

Use `execute_code` with Python `re.sub` for bulk replacement across the
workspace. Do NOT rename manually — too many call sites.

**Example bulk rename script:**
```python
import re, subprocess
from pathlib import Path

RENAMES = {
    r'\bKvasirGraph\b':       'RenderGraph',
    r'\bKvasirNode\b':        'RenderGraphNode',
    r'\bSurtrConfig\b':       'RendererConfig',
    r'\bSleipnirParams\b':    'SpringParams',
    r'\bSleipnirSolver\b':    'SpringSolver',
}

for src in Path('.').rglob('*.rs'):
    text = src.read_text(errors='replace')
    new = text
    for pattern, replacement in RENAMES.items():
        new = re.sub(pattern, replacement, new)
    if new != text:
        src.write_text(new)
        print(f'Renamed in {src}')
```

Then sweep for false positives with `git diff --stat` and run
`cargo check --workspace`.

Full rename table:

| Old Name | New Name | Kind | Crate |
|----------|----------|------|-------|
| KvasirGraph | RenderGraph | struct | cvkg-render-gpu |
| KvasirNode | RenderGraphNode | trait | cvkg-render-gpu |
| SurtrConfig | RendererConfig | struct | cvkg-render-gpu |
| BifrostModifier | FrostedGlassModifier | struct | cvkg-core |
| BifrostBridgeModifier | SharedElementModifier | struct | cvkg-core |
| GungnirModifier | NeonGlowModifier | struct | cvkg-core |
| GungnirPulseModifier | PulsingGlowModifier | struct | cvkg-core |
| MjolnirSliceModifier | SliceTransitionModifier | struct | cvkg-core |
| MjolnirShatterModifier | ShatterTransitionModifier | struct | cvkg-core |
| SleipnirParams | SpringParams | struct | cvkg-core |
| SleipnirSolver | SpringSolver | struct | cvkg-core |
| SleipnirModifier | SpringModifier | struct | cvkg-core |
| FafnirModifier | UsageGrowthModifier | struct | cvkg-core |
| MimirIntentModifier | PointerAnticipationModifier | struct | cvkg-core |
| OdinsEyeModifier | ObservabilityModifier | struct | cvkg-core |
| ManiGlowModifier | ProximityGlowModifier | struct | cvkg-core |
| YggdrasilTokens | TokenManager | struct | cvkg-core |
| KvasirId | ViewId | struct | cvkg-core |
| forge() | new() | method | cvkg-render-gpu |
| kvasir/ | render_graph/ | module dir | cvkg-render-gpu |

---

## Wave 6 — File Decomposition

**Load before starting**: refactoring, rust-development (P1-13 pattern),
subagent-driven-development

### 6-1: cvkg-render-gpu/src/renderer.rs (6,636 lines)

The largest file in the workspace. Mixes: renderer init, frame
submission, pipeline cache, shader cache, capture, swapchain handling,
error formatting.

**Proposed split:**
| New file | Contents |
|----------|----------|
| renderer/init.rs | Context creation, adapter selection, surface config |
| renderer/frame.rs | begin_frame/end_frame/present, render pass submission |
| renderer/pipelines.rs | Pipeline lookup, cache management |
| renderer/cache.rs | Shader cache filesystem I/O |
| renderer/capture.rs | Frame capture (screenshot) |

Keep `renderer.rs` as a thin re-export module.

**Risks:** Cross-module state (RendererContext) passed through all
subsystems. Extract the data types first, then the subsystems.

### 6-2: cvkg-render-native/src/lib.rs (4,276 lines)

**Proposed split:**
| New file | Contents |
|----------|----------|
| src/window.rs | Window management, event loop |
| src/asset.rs | Asset loading, image I/O |
| src/audio.rs | Rodio audio engine |
| src/render.rs | Rendering loop, frame submission |
| src/clipboard.rs | Clipboard integration |

### 6-3: cvkg-svg-filters/src/lib.rs (4,020 lines)

**Proposed split:**
| New file | Contents |
|----------|----------|
| src/filter.rs | Filter struct and variants |
| src/input.rs | Input handling, region calculation |
| src/output.rs | Output handling, bounds |
| src/primitive.rs | Per-primitive implementations |

---

## Cross-Cutting Concerns

### Test discipline for all fixes

Every P0 fix must have a regression test:
1. Write test that demonstrates the bug (fails against current code)
2. Apply the fix
3. Test passes
4. `cargo test --workspace --no-fail-fast` — no new failures

### Commit discipline

One commit per P-number. Tag with P-number for traceability:
```
fix(cvkg-anim): P0-14 guard spawn_rate=0 div-by-zero
fix(cvkg-layout): P0-2 aspect-ratio y-center *0.0 -> *0.5
```

### Verification before push

```bash
cargo check --workspace --tests --examples
cargo test --workspace --no-fail-fast
# Check no NEW test failures (baseline may have pre-existing env failures)
```

## Execution Order

```
Wave 1 Panic/Safety (P0 first, 12 items)
   ↓
Wave 2 Security (1 CRITICAL, 6 HIGH/MED)
   ↓
Wave 3 Logic/Correctness (2 CRITICAL, 10 HIGH/MED)
   ↓
Wave 4 Memory Safety (4 MED)
   ↓
Wave 5 Thematic Naming (20 renames)
   ↓
Wave 6 File Decomposition (3 large files)
```

Within each wave, fix items by severity (CRITICAL > HIGH > MED > LOW).
One commit per item.

# cvkg-stl Implementation Plan

## 1. Purpose

Replace the `stl_io` crate dependency with a purpose-built `cvkg-stl` crate. The `stl_io` crate is unmaintained, adds unnecessary deps, and only supports binary STL. Our crate will:

- Parse **both ASCII and binary STL** formats (auto-detect)
- Generate vertex/normal/index data compatible with `cvkg_core::Mesh`
- Support normals from the file (not default them to [0,0,1])
- Zero external deps (just `std`)
- Minimal allocations, zero-copy where possible

## 2. Format Specification (from LOC/IANA/synalysis grammar)

### Binary STL
```
Offset  Size    Type       Field
0       80      bytes      Header (arbitrary, often "solid" or program name)
80      4       u32 LE     Number of triangles (N)
84      50*N    bytes      Triangle data

Each triangle:
  0   4   f32 LE   Normal vector X
  4   4   f32 LE   Normal vector Y
  8   4   f32 LE   Normal vector Z
  12  4   f32 LE   Vertex 1 X
  16  4   f32 LE   Vertex 1 Y
  20  4   f32 LE   Vertex 1 Z
  24  4   f32 LE   Vertex 2 X
  28  4   f32 LE   Vertex 2 Y
  32  4   f32 LE   Vertex 2 Z
  36  4   f32 LE   Vertex 3 X
  40  4   f32 LE   Vertex 3 Y
  44  4   f32 LE   Vertex 3 Z
  48  2   u16      Attribute byte count (usually 0)
```

### ASCII STL
```
solid <name>
  facet normal <nx> <ny> <nz>
    outer loop
      vertex <x> <y> <z>
      vertex <x> <y> <z>
      vertex <x> <y> <z>
    endloop
  endfacet
endsolid <name>
```

### Auto-detection
Read first 80 bytes. If `solid ` starts at byte 0 AND no non-ASCII bytes in first 80 AND ASCII `endsolid` exists — try ASCII parse. Fall back to binary.

## 3. Architecture

### Module Layout
```
cvkg-stl/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public StlMesh type + from_reader + parse
│   ├── ascii.rs            # ASCII STL parser
│   ├── binary.rs           # Binary STL parser
│   ├── detect.rs           # Format auto-detection
│   ├── normal.rs           # Vertex deduplication + normal computation
│   └── error.rs            # StlError enum
├── tests/
│   ├── roundtrip_tests.rs  # Known-geometry assertions
│   ├── fuzz_tests.rs       # Random bytes don't panic
│   └── ascii_tests.rs      # ASCII format edge cases
├── reference/              # Test STL files (generated, not committed)
│   ├── cube_binary.stl
│   └── cube_ascii.txt      # ASCII "cube" for parser testing
└── README.md
```

### Core Types

```rust
/// A parsed STL mesh with shared vertices and computed normals.
#[derive(Debug, Clone, PartialEq)]
pub struct StlMesh {
    /// Deduplicated vertices (unique positions).
    pub vertices: Vec<[f32; 3]>,
    /// Per-vertex normals (either from file or computed from faces).
    pub normals: Vec<[f32; 3]>,
    /// Triangle vertex indices (every 3 = one face).
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StlFormat {
    Ascii,
    Binary,
}

pub enum StlError {
    Io(String),
    InvalidHeader,
    InvalidAscii(String),       // line number + message
    Truncated,                  // unexpected EOF
    NonTriangleFace,            // vertex count != 3
}
```

### Public API

```rust
impl StlMesh {
    /// Parse STL from any `Read` impl. Auto-detects ASCII/binary.
    pub fn from_reader(r: impl Read) -> Result<Self, StlError>;

    /// Parse STL from byte slice (convenience).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StlError>;

    /// Parse with a specific format hint (skip auto-detection).
    pub fn from_reader_with_hint(r: impl Read, hint: StlFormat) -> Result<Self, StlError>;

    /// Convert to a `cvkg_core::Mesh`.
    pub fn to_mesh(&self) -> cvkg_core::Mesh;

    /// Compute normals from face geometry (replaces file normals).
    pub fn compute_normals(&mut self);
}

/// Detect STL format without full parse.
pub fn detect_format(header: &[u8]) -> Option<StlFormat>;
```

### Integration with cvkg_core

In `cvkg-core/src/mesh.rs`, replace `stl_io::read_stl()`:
```rust
pub fn from_stl(data: &[u8]) -> anyhow::Result<Self> {
    let stl = cvkg_stl::StlMesh::from_bytes(data)
        .map_err(|e| anyhow::anyhow!("STL parse failed: {e}"))?;
    Ok(stl.to_mesh())
}
```

Remove `stl_io` from workspace dependencies entirely.

## 4. Implementation Phases (Red-Green TDD)

**Skills applied across all phases:**
- `software-development/rust-patterns` — float NaN handling, private module visibility, zero-copy parsing, f32 byte-keyed hashmap dedup, total_cmp for normal sorting
- `rust-tdd` — red-green-refactor cycle, assert_eq! for concrete values, proptest for parser panic-freedom, no unwrap/expect in tests, test functions return Result

### Phase P0 — Binary Parser (RED → GREEN)

**Skills**: `rust-tdd` (RED: write failing tests first), `software-development/rust-patterns` (NaN passthrough, zero-copy byte parsing)

| Red test | What it verifies |
|----------|-----------------|
| `test_binary_triangle_count_matches_header` | Header count == parsed triangles |
| `test_binary_vertex_count_is_triangles_times_3` | Index array length correct |
| `test_binary_known_cube` | Parse a known 12-triangle cube, verify vertex positions |
| `test_binary_truncated_header_errors` | < 84 bytes returns `StlError::Truncated` |
| `test_binary_zero_triangles_ok` | Empty mesh (0 triangles) parses successfully |
| `test_binary_normal_preserved` | Facet normals from file appear in output |

**Green impl**: `src/binary.rs` — read 80-byte header, u32 LE triangle count, then 50 bytes per triangle. Build `StlMesh` with non-duplicated vertices and per-vertex normals.

**Key patterns from `rust-patterns`**:
- Use `<[f32; 3]>::from_le_bytes()` for safe float parsing (NaN passthrough is fine — preserve file normals)
- Pre-allocate `Vec::with_capacity(num_triangles * 3)` to avoid reallocations
- Flush denormals before cross-product in normal computation: `if mag < 1e-15 { return [0.0, 0.0, 1.0]; }`

### Phase P1 — ASCII Parser (RED → GREEN)

**Skills**: `rust-tdd` (RED phase), `software-development/rust-patterns` (Peekable iterator clone for speculative float parsing, case-insensitive keyword matching)

| Red test | What it verifies |
|----------|-----------------|
| `test_ascii_simple_cube` | Parse minimal ASCII STL with 2 triangles |
| `test_ascii_name_preserved` | `solid <name>` extracted correctly |
| `test_ascii_extra_whitespace` | Tabs, multiple spaces handled |
| `test_ascii_scientific_notation` | `1.5e-3` parsed correctly |
| `test_ascii_missing_endsolid_errors` | Unterminated solid returns error |
| `test_ascii_non_triangle_errors` | Facet with != 3 vertices errors |
| `test_ascii_case_insensitive` | `FACET NORMAL` mixed case works |

**Green impl**: `src/ascii.rs` — line-by-line scan. Use `str::trim()` + `split_whitespace()` for tokenization. Match keywords case-insensitively with `eq_ignore_ascii_case`.

**Key patterns from `rust-patterns`**:
- For speculative line parsing (e.g. distinguishing `facet` from `endsolid`), clone the iterator/line buffer before attempting parse
- Use `f32::from_str()` for float parsing — handles scientific notation automatically

### Phase P2 — Auto-Detection + Error Handling

**Skills**: `rust-tdd` (proptest for fuzz testing), `software-development/rust-patterns` (graceful error types, no panic paths)

| Red test | What it verifies |
|----------|-----------------|
| `test_detect_ascii_from_solid_header` | First 5 bytes == "solid " → ASCII hint |
| `test_detect_binary_from_no_header` | Binary data without "solid " → Binary |
| `test_detect_malformed_ascii_falls_back` | "solid " prefix but invalid chars → try Binary |
| `test_parse_ascii_when_hint_is_binary` | Wrong hint doesn't crash, produces best effort |
| `test_io_error_propagation` | `Read` errors bubble up correctly |

**Key patterns from `rust-patterns`**:
- `StlError` enum uses `String` for messages, not `&'static str` (allows dynamic context)
- All error paths return `Result`, never `unwrap()` or `expect()` — consistent with rust-patterns "graceful GPU resource lookup pattern"

### Phase P3 — Vertex Deduplication + Normal Computation

**Skills**: `software-development/rust-patterns` (HashMap keyed by raw f32 bytes `[u8; 12]`, total_cmp for float sorting, cross-product normal computation with denormal flush)

| Red test | What it verifies |
|----------|-----------------|
| `test_shared_vertices_deduplicated` | Cube corners: 8 vertices, not 24 |
| `test_compute_normals_flat_shaded` | Each face gets correct flat normal |
| `test_file_normals_preserved_when_present` | StlMesh uses file normals unless `compute_normals` called |
| `test_dense_mesh_performance` | 100k triangles parses in < 100ms |

**Key patterns from `rust-patterns`**:
- Vertex dedup: `HashMap<[u8; 12], u32>` where key is `x.to_le_bytes() ++ y.to_le_bytes() ++ z.to_le_bytes()`. O(1) lookup, no float comparison issues.
- Normal computation: `edge1.cross(edge2).normalize()` with denormal flush: `if mag.norm() < 1e-15 { NAN }`

### Phase P4 — Integration + cvkg_core::Mesh Bridge

**Skills**: `software-development/rust-patterns` (private module visibility — re-export from lib.rs), `rust-tdd` (integration tests access only pub API)

| Red test | What it verifies |
|----------|-----------------|
| `test_stl_to_mesh_roundtrip` | `cvkg_stl::StlMesh` → `cvkg_core::Mesh` preserves vertices/indices |
| `test_cube_volume_positive` | Parsed unit cube has correct bounding box |

**Key patterns from `rust-patterns`**:
- Re-export all public types from `lib.rs` with `pub use` — integration tests can only access crate-root paths
- Test functions return `Result<(), Box<dyn Error>>` with `?` for error propagation (no unwrap in tests)

## 5. Format Conversion Details

### Binary Parser Algorithm
1. Read 80 bytes → `header: [u8; 80]`
2. Read 4 bytes → `num_triangles: u32` (LE)
3. Pre-allocate `vertices: Vec::with_capacity(num_triangles * 3)`
4. Per triangle (50 bytes):
   - Read normal (3 × f32 LE) → if zero-vector, flag for later compute
   - Read 3 vertices (9 × f32 LE)
   - Read 2 bytes attribute → discard
   - Deduplicate vertex via HashMap
   - Push indices
5. Return `StlMesh`

### ASCII Parser Algorithm
1. Read first line → `solid <name>` (optional capture)
2. Loop:
   - Match `facet normal <nx> <ny> <nz>`
   - Match `outer loop`
   - Match 3× `vertex <x> <y> <z>` with tolerance for extra whitespace
   - Match `endloop` / `endfacet`
3. Match `endsolid <name>` (optional)
4. Build `StlMesh`

### Flat Normal Computation
For flagged triangles with zero normals in file:
```rust
let v0 = self.vertices[idx0];
let v1 = self.vertices[idx1];
let v2 = self.vertices[idx2];
let edge1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
let edge2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
let nx = edge1[1]*edge2[2] - edge1[2]*edge2[1];
let ny = edge1[2]*edge2[0] - edge1[0]*edge2[2];
let nz = edge1[0]*edge2[1] - edge1[1]*edge2[0];
let mag = (nx*nx + ny*ny + nz*nz).sqrt();
if mag < 1e-15 { [0.0, 0.0, 1.0] } else { [nx/mag, ny/mag, nz/mag] }
```

## 6. Removing `stl_io`

1. Remove `stl_io = "0.11"` from root `Cargo.toml` workspace deps
2. Remove `stl_io = { workspace = true }` from `cvkg-core/Cargo.toml`
3. Change `cvkg-core/src/mesh.rs:from_stl()` to use `cvkg_stl::StlMesh::from_bytes()`
4. Run `cargo update -p stl_io` to clear lockfile entries
5. Verify `grep -c 'stl_io' Cargo.lock` → 0

## 7. Verification

| Check | Command |
|-------|---------|
| Crate compiles | `cargo check -p cvkg-stl` |
| All tests pass | `cargo test -p cvkg-stl` |
| Workspace clean | `cargo check --workspace` (0 warnings) |
| stl_io removed | `grep stl_io Cargo.lock` → no matches |
| cvkg-core still builds | `cargo check -p cvkg-core` |
| cvkg-core tests pass | `cargo test -p cvkg-core --lib` |

## 8. Risk Mitigation

| Risk | Mitigation | Skill Source |
|------|-----------|--------------|
| ASCII STL has many dialect variants | Accept any whitespace, optional name, case-insensitive keywords | `rust-tdd` (protest with arbitrary inputs) |
| Binary STL header contains "solid " tricking auto-detect | If ASCII parse fails mid-way, fall back to binary | `software-development/rust-patterns` (graceful fallback) |
| Vertex dedup is O(n²) with naive approach | Use `HashMap<[u8; 12], u32>` keyed by raw f32 bytes for O(1) | `software-development/rust-patterns` (dependency graph dedup pattern) |
| Denormalized floats from bad STL files | Flush-to-zero before normal computation | `software-development/rust-patterns` (MSDF spread validation, NaN clamping) |
| Non-manifold meshes | Not our concern — STL is lowest common denominator | — |
| proptest in integration tests panics on config | Don't use `#![proptest_config]` in `tests/` dir; use env var or default | `software-development/rust-patterns` (proptest in integration tests section) |
| Integration tests can't access private modules | Re-export all pub API from `lib.rs` with `pub use` | `software-development/rust-patterns` (private module visibility) |
| Test functions use `unwrap()` | Return `Result` + `?` per `rust-tdd` discipline | `rust-tdd` (assertions section) |

## 9. Test Fixtures (generated at test time)

No committed binary files needed. Each test generates its own STL data:
```rust
fn make_unit_cube_binary() -> Vec<u8> { ... }  // 12 triangles, 8 unique corners
fn make_unit_cube_ascii() -> Vec<u8> { ... }
fn make_tetrahedron_binary() -> Vec<u8> { ... }
```

This avoids binary blob commits and tests the parser against known geometry.

## 10. Skill Quick Reference (per plan issue)

| When | Load Skill |
|------|-----------|
| Before writing any test | `rust-tdd` (red-green-refactor, no unwrap, Result return) |
| Designing float parsing / normal math | `software-development/rust-patterns` (NaN handling, cross-product, denormal flush) |
| Designing vertex dedup | `software-development/rust-patterns` (HashMap key patterns, f32 byte keys) |
| Writing integration tests | `software-development/rust-patterns` (private module visibility, pub use re-export) |
| Workspace integration | `cvkg-project` (lockstep versioning, autotests=false, defers to TDD) |
| After every change | `cargo check --workspace` + `cargo test --workspace` |

## 11. Estimated Scope

| Item | Count |
|------|-------|
| Source files | 6 |
| Test files | 3 |
| Lines of code | ~600 |
| Tests | ~28 |
| Dependencies | 0 (std only) |

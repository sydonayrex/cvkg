# cvkg-inputs Implementation Plan (with Skill Mapping)

## 0. Plan Meta

| Field | Value |
|-------|-------|
| Crate name | `cvkg-inputs` |
| Workspace version | 0.2.15 (lockstep) |
| Edition | 2024 |
| Estimated crates touched | 4 (inputs, core, render-native, components) |
| Estimated new files | ~25 |
| Estimated tests | 111+ |
| Red-green phases | 5 (P0–P4) |

---

## 1. Purpose

`cvkg-inputs` is the HID (Human Interface Device) interconnect crate for CVKG. It provides:

- **Gamepad/controller** detection, polling, and event dispatch (USB, Bluetooth)
- **Keyboard/mouse** abstraction beyond winit (raw input, low-latency)
- **Touch/gesture** multi-touch tracking
- **HID device enumeration** (hot-plug, device info)
- **Input mapping/remapping** layer (action maps, axis deadzones, sensitivity)
- **Cross-platform** backend (gilrs cross-platform, evdev on Linux for raw HID)

---

## 2. Architecture

### 2.1 Module Layout

```
cvkg-inputs/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public re-exports, InputState, InputBackend trait
│   ├── error.rs                # InputError enum (thiserror)
│   ├── backend/
│   │   ├── mod.rs              # Backend trait + dispatch
│   │   ├── gilrs_backend.rs    # gilrs implementation
│   │   ├── evdev_backend.rs    # Linux raw HID (cfg linux)
│   │   └── noop_backend.rs     # Fallback
│   ├── gamepad/
│   │   ├── mod.rs              # GamepadId, GamepadState, GamepadEvent
│   │   ├── mapping.rs          # Standard layout (Xbox/PS/Switch)
│   │   └── rumble.rs           # Force-feedback
│   ├── keyboard/
│   │   ├── mod.rs              # KeyboardState, RawKeyEvent
│   │   └── mapping.rs          # Scancode → Key
│   ├── mouse/
│   │   ├── mod.rs              # MouseState, relative motion
│   │   └── gesture.rs          # Gesture detection
│   ├── touch/
│   │   ├── mod.rs              # TouchPoint, TouchState
│   │   └── gesture.rs          # Multi-touch gestures
│   ├── hid/
│   │   ├── mod.rs              # HidDeviceInfo, enumeration
│   │   └── report.rs           # HID report descriptor parsing
│   ├── action/
│   │   ├── mod.rs              # ActionMap, Action, Axis, Binding
│   │   └── deadzone.rs         # Radial/stick deadzone math
│   └── platform/
│       ├── mod.rs              # Platform detection
│       ├── linux.rs            # Linux-specific
│       ├── macos.rs            # macOS-specific
│       └── windows.rs          # Windows-specific
├── tests/
│   ├── gamepad_mapping_tests.rs
│   ├── deadzone_tests.rs
│   ├── action_map_tests.rs
│   └── evdev_backend_tests.rs  # Linux-only
└── examples/
    └── input_viewer.rs
```

---

## 3. Implementation Phases

### Phase P0 — Skeleton

**Goal**: Crate compiles with stub types. All tests FAIL (red).

| Requirement | Skills |
|-------------|--------|
| Create `cvkg-inputs/Cargo.toml` with workspace lockstep, feature flags | `rust-workspace-documentation` |
| Define `InputBackend` trait, `InputEvent` enum, `InputState` struct | `clean-code`, `ponytail`, `software-design-philosophy` |
| Define `InputError` with `thiserror` | `error-handling`, `rust-error-propagation` |
| Red tests: `test_backend_name`, `test_poll_empty`, `test_event_clone` | `tdd`, `red-green-refactor`, `rust-tdd` |
| Add to workspace `Cargo.toml` members | `rust-workspace-audit` |

**Verification**: `cargo check -p cvkg-inputs` passes. `cargo test -p cvkg-inputs` fails (red).

---

### Phase P1 — Core Types + Deadzone Math

**Goal**: Type system + deadzone math tested via red-green TDD.

| Requirement | Skills |
|-------------|--------|
| `deadzone::apply(value: f32, threshold: f32) → f32` | `clean-code`, `ponytail` |
| `deadzone::radial(x: f32, y: f32, threshold: f32) → (f32, f32)` | `pragmatic-programmer` |
| `ActionMap::bind(action, binding)` | `clean-code`, `software-design-philosophy` |
| `ActionMap::evaluate(&InputEvent) → Vec<String>` | `tdd`, `rust-patterns` |
| PBT: deadzone with 1000 random f32 inputs (NaN, ±∞, subnormal, ±0.0) | `strong-tests`, `test-driven-development` |
| PBT: action map binding round-trip | `strong-tests`, `test-patterns` |
| Negative tests: threshold= 0.0, threshold=1.0, NaN passthrough | `strong-tests` |

**Red tests first**:
- `deadzone_clamps_small_values_to_zero`
- `deadzone_preserves_full_deflection`
- `linear_deadzone_scales_midrange`
- `radial_deadzone_handles_diagonal`
- `action_map_binding_to_button`
- `action_map_axis_range_triggers_in_threshold`

**Green impl**: minimum code to pass each test, one at a time.

**Verification**: `cargo test -p cvkg-inputs` all green.

---

### Phase P2 — Gilrs Backend

**Goal**: Gamepad connection/axis/button events via gilrs crate.

| Requirement | Skills |
|-------------|--------|
| `GilrsBackend::new()` initializes gilrs context | `backend-patterns`, `rust-patterns` |
| `GilrsBackend::poll() → Vec<InputEvent>` | `clean-code`, `ponytail` |
| `GilrsBackend::set_rumble(device, weak, strong)` | `error-handling` |
| `mapping::gilrs_button_to_standard(Button) → GamepadButton` | `software-design-philosophy` |
| `mapping::gilrs_axis_to_standard(Axis) → GamepadAxis` | `clean-code` |
| Mock backend trait injection for testing (no real hardware needed) | `rust-testing`, `test-patterns` |
| PBT: axis mapping round-trip | `strong-tests` |

**Red tests first**:
- `gilrs_detects_connected_gamepad`
- `gilrs_emits_axis_event`
- `gilrs_emits_button_press_release`
- `gilrs_handles_disconnect`
- `mapping_xbox_layout_correct`

**Green impl**: inject `MockGilrsContext` via trait object or generic, avoid real hardware in CI.

**Verification**: `cargo test -p cvkg-inputs --features gilrs` all green. CI runs mock tests on all platforms.

---

### Phase P3 — Evdev Backend (Linux-only)

**Goal**: Raw evdev keyboard/mouse/gamepad on Linux.

| Requirement | Skills |
|-------------|--------|
| `EvdevBackend::new() → Result<Self>` | `error-handling`, `rust-error-propagation` |
| `EvdevBackend::enumerate() → Vec<HidDeviceInfo>` | `backend-patterns` |
| `EvdevBackend::poll() → Vec<InputEvent>` | `clean-code` |
| `keyboard::mapping::linux_scancode_to_key(u16) → Key` | `software-design-philosophy` |
| `hid::HidDevice::from_path(Path) → Result<HidDevice>` | `rust-patterns` |
| Graceful fallback when `/dev/input/*` permission denied | `error-handling`, `ponytail` |
| `#[cfg(test)]` mock evdev device from byte vec | `rust-testing` |

**Red tests** (cfg(linux)):
- `evdev_enumerates_input_devices`
- `evdev_parses_keyboard_event`
- `evdev_parses_mouse_relative_axes`
- `evdev_parses_gamepad_abs_axis`

**Verification**: `cargo test -p cvkg-inputs --features evdev` on Linux. Skipped on other platforms via `#[cfg(target_os = "linux")]`.

---

### Phase P4 — Action System Integration

**Goal**: Full action map with chords, remapping, sensitivity. Integrate into CVKG event loop.

| Requirement | Skills |
|-------------|--------|
| `InputSystem` owns backends + `Arc<RwLock<InputState>>` | `rust-patterns`, `software-design-philosophy` |
| Action chord: multi-input combo (`Ctrl+JUMP`) | `clean-code` |
| Axis remapping: invert, sensitivity multiplier | `ponytail` |
| `InputState: Clone` (for VDOM hit testing) | `rust-patterns` |
| Extend `cvkg_core::Event` with `Gamepad*` variants | `cvkg-core-patterns`, `cvkg-implementation-patterns` |
| `cvkg-render-native` converts `InputEvent → Event::Gamepad*` behind `inputs` feature | `cvkg-project`, `dependency-updater` |
| `verification-before-completion`: run `cargo check --workspace` | `verification-before-completion` |
| `ponytail-audit`: scan for over-engineering | `ponytail`, `ponytail-audit` |

**Red tests first**:
- `action_triggered_on_button_press`
- `action_axis_value_with_deadzone`
- `chord_requires_all_inputs_pressed`
- `remap_axis_inverts_direction`
- `state_clone_preserves_current_values`

**Verification**: `cargo check --workspace` passes. `cargo test -p cvkg-inputs` all green.

---

## 4. Dependency + Feature Flag Matrix

| Feature | Deps | Platform |
|---------|------|----------|
| `default = ["gilrs"]` | `gilrs 0.11` | All |
| `evdev` | `evdev 0.12`, `input 0.8` | Linux only |
| `rumble` | (extra gilrs features) | All |
| `serde` | `serde 1` + `serde/derive` | All (action map serialization) |

**Skills**: `dependency-updater` (audits for bloat), `ponytail` (only add deps that are directly used).

---

## 5. Testing Strategy

| Layer | Skill | Count | Pattern |
|-------|-------|-------|---------|
| Unit tests | `rust-testing`, `tdd` | ~60 | `#[test]` per function |
| Property tests | `strong-tests`, `test-driven-development` | ~20 | `proptest` for deadzone/axis serialize |
| Mock backend | `rust-testing`, `test-patterns` | ~30 | `MockBackend` trait impl |
| Integration | `strong-tests` | ~10 | `tests/` dir, serial_test for global state |
| Mutation gutcheck | `strong-tests` | manual | flip `>` to `>=`, verify test fails |

**Strong test rules** (from `strong-tests` skill):
- No trivial inputs: deadzone tests use `f32::NAN`, `f32::INFINITY`, subnormals, `±0.0`
- Mutation-resistant: assert exact values, not `is_finite()`
- Negative: disconnected backend, permission denied, rumble on device without FF
- PBT for all math: `proptest` generates 1000 random inputs

**TDD discipline** (from `tdd`, `red-green-refactor` skills):
1. Write failing test
2. `cargo test` — RED
3. Write minimum code
4. `cargo test` — GREEN
5. Refactor (clean-code, ponytail)

---

## 6. Risk Mitigation

| Risk | Mitigation | Skills |
|------|-----------|--------|
| gilrs doesn't support macOS IOKit | Stub macOS, defer to Phase 2 | `specification-writing` |
| evdev needs root | Graceful fallback to gilrs + `tracing::warn` | `error-handling` |
| Platform CI gaps | Mock backend on all platforms; evdev cfg(linux) | `rust-testing` |
| Input latency | Backend polls in dedicated thread, channel to main | `rust-patterns` |
| Deadlock on `InputState` lock | `parking_lot::Mutex` or `tokio::sync::RwLock` | `rust-patterns`, `debugging` |
| Test flakiness from global state | `serial_test` for shared state | `strong-tests` |

---

## 7. Skill Quick Reference (per plan action)

| When | Load Skill |
|------|-----------|
| Before writing any test | `strong-tests`, `tdd`, `red-green-refactor` |
| Before writing production code | `ponytail`, `clean-code`, `pragmatic-programmer` |
| Before touching workspace `Cargo.toml` | `rust-workspace-audit`, `dependency-updater` |
| Before extending `cvkg_core::Event` | `cvkg-core-patterns`, `cvkg-implementation-patterns` |
| Before final integration | `verification-before-completion`, `ponytail-audit` |
| When stuck on architecture | `software-design-philosophy`, `system-design` |
| Before writing docs/README | `writing-plans`, `writing-guidelines` |
| After every change | `cargo check --workspace` (mandatory) |

---

## 8. Deliverables Checklist

- [ ] `cvkg-inputs/` in workspace `Cargo.toml` members
- [ ] Feature flags: `gilrs` (default), `evdev`, `rumble`, `serde`
- [ ] `InputBackend` trait with 3 implementations
- [ ] `InputState: Clone + Send + Sync`
- [ ] `ActionMap` with chords, deadzone, sensitivity
- [ ] `GamepadButton`/`GamepadAxis` standard mappings
- [ ] 111+ tests passing (red-green verified)
- [ ] `examples/input_viewer.rs` runs
- [ ] `cvkg-core` has `Gamepad*` event variants
- [ ] `cvkg-render-native` has `inputs` feature flag
- [ ] `cargo check --workspace` — zero warnings
- [ ] `cargo test --workspace` — all green
- [ ] `docs/architecture.md` updated with inputs crate

---

## 9. First Action (P0 Start)

The immediate next step if you say "proceed":

1. Add `"cvkg-inputs"` to workspace `Cargo.toml` members array
2. Create `cvkg-inputs/Cargo.toml` with deps + feature flags
3. Create `src/lib.rs` with stub `InputBackend` trait and `InputEvent` enum
4. Write 3 red tests
5. `cargo check -p cvkg-inputs` (should pass with stubs)
6. `cargo test -p cvkg-inputs` (should FAIL — red phase complete)

# CVKG Animation Vocabulary

**Version:** 1.0
**Status:** Active
**Scope:** All CVKG component `render()` methods, animation callbacks, and spring-based transitions.

---

## Physics Parameters

All CVKG animations use spring physics defined by `SleipnirParams` (in `cvkg-anim`):

- `stiffness` — how strongly the spring pulls toward target (N/m equivalent)
- `damping` — how much oscillation is absorbed (0 = no damping, 1 = critical)
- `mass` — virtual weight of the animated property (higher = slower, heavier feel)

Duration is emergent from these parameters — no hardcoded durations.

---

## Standard Presets

| Preset | Stiffness | Damping | Mass | Effective Duration | Use For |
|--------|-----------|---------|------|-------------------|---------|
| `snappy` | 800 | 30 | 0.4 | ~150ms | Button press, toggle, tooltip, chevron rotate |
| `fluid` | 400 | 22 | 0.5 | ~200ms | Hover enter, tab switch, dropdown open, focus ring |
| `heavy` | 200 | 18 | 0.8 | ~300ms | Modal open, drawer slide, panel expand, toast enter |
| `bouncy` | 300 | 12 | 0.6 | ~400ms | Error shake, achievement unlock, playful transitions |

Defined in `Theme::motion` field as `MotionScale { snappy, fluid, heavy, bouncy }`.

---

## Interaction-to-Animation Mapping

### Button / Interactive Controls
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Press (mouse down) | snappy | Immediate tactile feedback |
| Release (mouse up) | snappy | Quick return to rest |
| Hover enter | fluid | Subtle awareness, not jarring |
| Hover exit | fluid | Gentle fade |
| Focus ring appear | fluid | Draw attention without startling |
| Disabled state transition | snappy | Instant clarity |

### Overlay / Sheet / Modal
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Open / appear | heavy | Authoritative entrance, sense of weight |
| Close / dismiss | snappy | Fast removal, user expects quick exit |
| Backdrop fade in | fluid | Smooth context shift |
| Backdrop fade out | snappy | Quick cleanup |

### Tab Bar / Navigation
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Indicator slide | fluid | Spatial continuity, track user's eye |
| Tab content crossfade | fluid | Smooth content replacement |
| Navigate back | snappy | Quick dismissal |

### List / Table
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Item entrance (stagger) | snappy | Fast list population, 30-50ms stagger |
| Item hover | fluid | Subtle highlight |
| Item selection | snappy | Instant confirmation |
| Delete / remove | snappy | Fast cleanup |
| Reorder / drag | heavy | Sense of weight while dragging |
| Drop into place | bouncy | Satisfying settle |

### Toast / Notification
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Enter | heavy | Attention-grabbing, must be noticed |
| Auto-dismiss countdown | — | No animation, just timer |
| Manual dismiss | snappy | Quick removal |
| Stack reordering | fluid | Smooth repositioning |

### Form / Input
| Interaction | Preset | Rationale |
|-------------|-------|-----------|
| Focus enter | fluid | Smooth transition to active state |
| Focus exit | snappy | Quick deactivation |
| Validation error appear | bouncy | Draw attention to error |
| Validation success appear | snappy | Quick confirmation |
| Character counter update | — | No animation, just text change |

---

## Implementation Rules

1. **Use theme presets.** Access via `ctx.theme().motion.{snappy|fluid|heavy|bouncy}`.
2. **Never hardcode spring params** in component render methods.
3. **Match the table above**. If your interaction isn't listed, use `fluid` as default.
4. **Stagger list items** at 30-50ms intervals using `theme::motion::snappy`.
5. **Exit animations** should be shorter than enter (~60-70% of enter duration).
6. **When in doubt**, use `fluid`. It's the safest default.

---

## Reduced Motion

When `AccessibilityOverrides.reduce_motion` is true:
- Replace all spring animations with instant transitions (value = target, no interpolation)
- Skip stagger delays
- Disable parallax, bounce, and decorative motion
- Keep essential state changes (focus rings, error indicators) but make them instant

---

## How to Add a New Preset

1. Add variant to `MotionScale` struct in `cvkg-themes/src/lib.rs`
2. Add constructor to `SleipnirParams` in `cvkg-anim/src/lib.rs`
3. Document it in this file
4. Add test in `cvkg-components/tests/animation_test.rs`
5. Get design steward approval (see `doc/DESIGN_STEWARD.md`)

---

*Last updated: 2026-06-14 by OWL*
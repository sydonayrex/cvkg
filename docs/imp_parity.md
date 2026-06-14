# CVKG Tahoe Parity Implementation Plan

**Current Parity: ~55-65%**
**Target Parity: 85-90%**

---

## Priority 1: Text Rendering (Current: C+ → Target: A-)

### Problem
Text rendering has been a recurring issue across multiple reviews. The architecture (MSDF, subpixel, Knuth-Plass, emoji) is sound, but the execution has gaps in baseline positioning, kerning at small sizes, and integration with the GPU renderer.

### Issues to Fix
1. **Baseline-relative positioning** -- `draw_text` y-parameter is text origin, but glyphs are baseline-relative. All callers must compute `y = desired_baseline - ascent`. The `measure_text_baseline` trait method was added but the vtable dispatch issue means it falls through to the default (returns 0). This needs a proper fix.
2. **Kerning at small sizes** -- Jupiteroid font at 12-13px shows overly tight glyph advances. Need to verify MSDF metrics and potentially add letter-spacing.
3. **Variable font support** -- cvkg-runic-text needs variable font axis handling for SF Pro and similar.
4. **Color fonts / emoji** -- Ensure emoji rendering works correctly with the subpixel pipeline.
5. **SF Symbols alignment** -- Symbol fonts need baseline alignment matching system text.

### Steps
1. Fix `measure_text_baseline` vtable dispatch (investigate why default impl is called)
2. Audit all `draw_text` callers for correct baseline positioning
3. Add letter-spacing support to `TextStyle`
4. Test with system fonts (SF Pro, Helvetica Neue) at 11-17px
5. Verify emoji rendering in agent chat and other text-heavy components

**Estimated effort: 2-3 days**

---

## Priority 2: SVG Rendering (Current: C → Target: B+)

### Problem
SVG pipeline has the foundation (usvg, filters, serialization) but lacks browser-level fidelity. Recent fixes addressed material routing, per-path transforms, and animation offsets. Remaining gaps are significant.

### Issues to Fix
1. **Gradient support** -- Currently only solid colors are supported. SVG gradients (linear, radial) need tessellation-time color interpolation.
2. **Masks and clip paths** -- No support for `<mask>`, `<clipPath>` elements. These require stencil buffer or alpha masking.
3. **Filter correctness** -- SVG filters (blur, drop-shadow, color matrix) need GPU-side implementation matching browser output.
4. **Path intersection/fill rules** -- Non-zero and even-odd fill rules need correct tessellation.
5. **CSS styling** -- Inline styles and class-based styling from SVG need proper cascade resolution.
6. **`<use>` element** -- Symbol reuse and referencing needs implementation.
7. **Text in SVG** -- SVG `<text>` elements need to route through the text shaping engine.

### Steps
1. Add gradient color interpolation to tessellation (linear first, then radial)
2. Implement clip paths via stencil buffer in the geometry pass
3. Implement masks via alpha texture in the compositor
4. Add SVG filter GPU kernels (gaussian blur, color matrix, composite)
5. Fix fill rule handling in lyon tessellation
6. Add CSS style cascade resolution for SVG attributes
7. Implement `<use>` element resolution
8. Route SVG `<text>` through cvkg-runic-text

**Estimated effort: 5-7 days**

---

## Priority 3: Tahoe Material System (Current: B → Target: A-)

### Problem
Glass/backdrop infrastructure exists but lacks the full Tahoe material stack: dynamic luminance adaptation, vibrancy, content-aware tinting, layered translucency, depth-reactive materials, specular highlights, and edge refraction.

### Issues to Fix
1. **Dynamic luminance adaptation** -- Glass materials should adapt their tint based on the brightness of content behind them. Requires sampling the backdrop texture luminance.
2. **Vibrancy** -- macOS vibrancy effect needs integration with the compositor. This is more than blur -- it's a color saturation/brightness shift.
3. **Content-aware tinting** -- The glass tint should shift based on the dominant color of the backdrop content.
4. **Layered translucency** -- Multiple glass layers should composite correctly with depth-aware blending.
5. **Specular highlights** -- Glass surfaces should have subtle specular highlights based on light direction.
6. **Edge refraction** -- Light bending at glass edges for realistic depth perception.
7. **Design tokens** -- A comprehensive design token system for colors, spacing, typography, and animation curves matching Tahoe.

### Steps
1. Add luminance sampling pass to the render graph (compute average luminance of backdrop region)
2. Implement vibrancy as a post-process on the glass blur output
3. Add dominant color extraction from backdrop for content-aware tinting
4. Implement layered glass compositing with depth-aware blending
5. Add specular highlight calculation to the glass fragment shader
6. Add edge refraction effect using derivative-based distortion
7. Create a design token system (colors, spacing, typography scales, animation curves)
8. Match Tahoe's specific animation curves (spring, ease-in-out timing)

**Estimated effort: 7-10 days**

---

## Priority 4: Accessibility (Current: C+ → Target: B+)

### Problem
Accessibility scaffolding exists (passes, components, roles, focus state) but lacks the complete implementation needed for VoiceOver and other assistive technologies.

### Issues to Fix
1. **VoiceOver integration** -- On macOS, need to export the accessibility tree via `NSAccessibility` protocol or equivalent.
2. **Native accessibility tree export** -- The component tree needs to be serializable to a format consumable by platform accessibility APIs.
3. **Keyboard navigation correctness** -- Tab order, arrow key navigation, and escape handling need to work correctly across all components.
4. **Rotor support** -- VoiceOver rotor needs custom actions and element grouping.
5. **Semantic announcements** -- Dynamic content changes need to trigger accessibility announcements.
6. **Accessibility actions** -- Custom actions (activate, increment, decrement, scroll) need to be exposed.
7. **Reduced motion** -- The `prefers-reduced-motion` media query needs to actually disable/reduce animations.

### Steps
1. Implement accessibility tree serialization (role, label, value, actions, children)
2. Add platform-specific export for macOS (NSAccessibility), Windows (UIA), Linux (AT-SPI)
3. Implement correct tab order traversal across all components
4. Add arrow key navigation for composite components (menus, lists, trees)
5. Implement VoiceOver rotor with custom action groups
6. Add semantic announcement queue for dynamic content changes
7. Implement accessibility action protocol (activate, scroll, value change)
8. Wire `prefers-reduced-motion` to the animation system

**Estimated effort: 5-7 days**

---

## Priority 5: Native macOS Fidelity (Current: D+ → Target: B)

### Problem
CVKG feels like a custom GPU UI framework inspired by Apple design, not a framework capable of reproducing Tahoe exactly. Missing native behaviors are the biggest gap.

### Issues to Fix
1. **NSVisualEffectView parity** -- The glass/liquid glass implementation needs to match the exact visual behavior of NSVisualEffectView.
2. **Native window chrome** -- Traffic lights (close/minimize/maximize), title bar, toolbar need to match macOS exactly.
3. **Menu bar integration** -- Global menu bar with correct styling and behavior.
4. **Stage Manager awareness** -- Window should respond to Stage Manager state changes.
5. **Spotlight-style search** -- Command palette should match Spotlight's behavior and visual style.
6. **Native focus rings** -- Focus indicators should match macOS focus ring appearance.
7. **macOS animation curves** -- All animations should use the exact same timing curves as macOS (spring parameters, ease durations).
8. **Native drag-and-drop** -- Drag and drop should use native macOS semantics (pasteboard, drag images, drop animations).

### Steps
1. Study NSVisualEffectView behavior and match blur/vibrancy parameters exactly
2. Implement native window chrome with traffic light controls
3. Add global menu bar with standard macOS menu structure
4. Implement Stage Manager window state observation
5. Recreate Spotlight-style search with matching visual design
6. Add native focus ring rendering matching macOS appearance
7. Extract and match macOS spring animation parameters
8. Implement native drag-and-drop with pasteboard integration

**Estimated effort: 10-14 days**

---

## Priority 6: Design Polish (Current: C+ → Target: B+)

### Problem
Component coverage is large but visual and behavioral fidelity is inconsistent. Components need to match Tahoe's exact spacing, typography, animation, and interaction patterns.

### Issues to Fix
1. **Consistent spacing system** -- All components should use the same spacing tokens (4px grid, 8px increments).
2. **Typography consistency** -- Font sizes, weights, and line heights should match Tahoe exactly.
3. **Color consistency** -- All colors should come from the design token system, not hardcoded.
4. **Animation consistency** -- All transitions should use the same timing curves and durations.
5. **Interaction consistency** -- Hover, pressed, focused, disabled states should be consistent across all components.
6. **Icon consistency** -- All icons should use SF Symbols or matching custom icons at consistent sizes.

### Steps
1. Audit all components for spacing consistency against Tahoe
2. Create a comprehensive spacing token system
3. Audit typography across all components
4. Replace all hardcoded colors with design token references
5. Standardize animation timing across all components
6. Audit and standardize interaction states
7. Replace custom icons with SF Symbols where appropriate

**Estimated effort: 5-7 days**

---

## Implementation Order

### Phase 1: Foundation (Week 1)
1. Fix text rendering baseline positioning
2. Add gradient support to SVG pipeline
3. Implement design token system
4. Fix spacing consistency across components

### Phase 2: Rendering (Week 2)
1. Implement SVG clip paths and masks
2. Add SVG filter GPU kernels
3. Implement luminance adaptation for glass materials
4. Add vibrancy and content-aware tinting

### Phase 3: Platform (Week 3)
1. Implement accessibility tree export
2. Add keyboard navigation correctness
3. Implement native window chrome
4. Add macOS animation curve matching

### Phase 4: Polish (Week 4)
1. SVG `<use>` element and text support
2. Layered glass compositing
3. Specular highlights and edge refraction
4. Design consistency audit and fixes
5. Native drag-and-drop

### Phase 5: Integration (Week 5)
1. VoiceOver integration testing
2. Stage Manager awareness
3. Spotlight-style search
4. Full regression testing across all components

---

## Success Metrics

- [ ] Text renders identically to NSTextField at all sizes 11-17px
- [ ] SVG test suite passes at 95%+ fidelity vs Safari rendering
- [ ] Glass materials are visually indistinguishable from NSVisualEffectView
- [ ] VoiceOver can navigate the entire UI tree
- [ ] All animations match macOS timing curves within 5ms
- [ ] Window chrome is visually identical to native macOS windows
- [ ] All components use design tokens (zero hardcoded colors/spacing)

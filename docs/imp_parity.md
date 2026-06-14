# CVKG Tahoe Parity Implementation Plan -- A+ Target

**Current Parity: ~38-65%** (two independent audits)
**Target Parity: A+ in every area (>95%)**

The rendering architecture is solid. The remaining work is fidelity -- making every pixel, every interaction, and every accessibility behavior indistinguishable from native macOS Tahoe.

---

## Audit Findings Summary

Two independent audits identified the following critical gaps:

| Audit | Overall Parity | Critical Gaps | Already Aligned |
|-------|---------------|----------------|-----------------|
| Code review | 55-65% | Text, SVG, Accessibility, Native macOS | Render graph, GPU pipeline |
| HTML audit | 38% | Liquid glass, chrome, icons, hover states | Text shaping, physics animation |

**Combined grade per area (lowest score wins):**

| Area | Grade | Key Gap |
|------|-------|---------|
| Render Graph | A | Already strong |
| GPU Pipeline | A- | Missing displacement/refraction |
| Text Shaping | A- | Baseline positioning at small sizes |
| Physics Animation | A | Already ahead (RK4 vs spring) |
| Glass Effects | D | Frost ≠ liquid glass, no displacement |
| Component Coverage | B+ | Inconsistent radii, no squircle icons |
| Typography | C+ | Baseline offset, kerning at small sizes |
| SVG Fidelity | D | No filters, masks, clip paths |
| Accessibility | C+ | No reduce-transparency mode |
| Native macOS | D | No NSVisualEffectView bridge, no chrome |
| Design Polish | C+ | No radius tokens, no icon silhouette contract |

---

## 1. Liquid Glass / Material System: A+ (Current: D)

### Definition of A+
Glass materials are **visually indistinguishable** from NSVisualEffectView at every vibrancy setting. Real-time edge refraction, specular highlights, and displacement mapping match Tahoe's "wet glass" physics.

### Concrete Requirements
1. **Edge refraction**: Snell's law light bending at glass edges using screen-space derivatives. Chromatic aberration at extreme angles.
2. **Specular highlights**: PBR GGX specular model with IOR 1.52 matching glass. Light position awareness.
3. **Displacement mapping**: `feDisplacementMap` analog in WGSL -- backdrop distortion at glass edges creating the "wet glass" look.
4. **feTurbulence analog**: Procedural noise for organic glass surface variation.
5. **Dynamic luminance adaptation**: Glass tint/opacity adapts in real-time based on backdrop brightness. 100ms response time.
6. **18 vibrancy modes**: `.underWindowBackground`, `.fullScreenUI`, `.hudWindow`, `.menu`, `.popover`, `.sidebar`, `.titlebar`, `.selection`, `.headerView`, `.sheet`, `.windowBackground`, `.toolTip`, `.contentBackground`, `.underPageBackground` -- all matched exactly.
7. **Content-aware tinting**: Dominant color extraction from backdrop subtly tints the glass surface.
8. **Layered translucency**: N glass layers composite correctly with depth-aware blending. No artifacts at 5+ layers.
9. **Depth reaction**: Materials respond to z-distance (parallax, blur intensity variation).
10. **Subsurface scattering**: Light diffusion through translucent materials for soft glow.

### Steps
1. Replace Bifrost frost shader with true liquid glass shader (displacement + refraction + specular)
2. Add `feDisplacementMap` WGSL kernel using screen-space derivatives for edge distortion
3. Add `feSpecularLighting` WGSL kernel with point light and GGX distribution
4. Add `feTurbulence` WGSL kernel (Perlin noise) for organic surface variation
5. Add compute pass for backdrop luminance sampling (per-region average)
6. Implement 18 vibrancy modes as WGSL shader variants
7. Add dominant color extraction compute pass (k-means on downsampled backdrop)
8. Implement layered glass compositing with depth peeling
9. Add depth reaction by passing scene depth to material shader
10. Implement subsurface scattering approximation (wrap diffusion profile)
11. Validate against NSVisualEffectView side-by-side at all 18 modes

**Estimated effort: 14-21 days**

---

## 2. Native macOS Fidelity: A+ (Current: D)

### Definition of A+
The window, chrome, menus, and interactions are **pixel-identical and behaviorally identical** to native macOS. A user cannot tell the difference between a CVKG app and a native AppKit app.

### Concrete Requirements
1. **NSVisualEffectView bridge**: Direct integration with NSVisualEffectView for system glass effects. Fallback to custom shader on non-macOS.
2. **Transparent menubar**: Menubar with correct transparency, blur, and content reflection.
3. **Sidebar reflect-and-refract**: Sidebar that reflects and refracts content behind it, matching Tahoe's sidebar idiom.
4. **Window chrome**: Traffic lights with correct hover/pressed states, positioning, and behavior. Fullscreen transition matches macOS exactly.
5. **Menu bar integration**: Global menu bar with correct font, spacing, hover states, submenu animation, keyboard shortcut display.
6. **Stage Manager awareness**: Window resizes/repositions correctly when Stage Manager is activated.
7. **Spotlight-style search**: Command palette matching Spotlight's visual design, fuzzy search, keyboard navigation, animation timing.
8. **Native focus rings**: Rendered with exact macOS focus ring appearance (color, radius, animation, spacing).
9. **Animation curves**: Spring animations match `UIView.animate` parameters exactly. Cubic bezier curves match `CAMediaTimingFunction`.
10. **Drag and drop**: Native NSPasteboard integration, drag images, drop animations, spring-loaded folders.
11. **Services menu**: App services appear in Services submenu and work correctly.
12. **Trackpad gestures**: Pinch to zoom, rotate, swipe between pages -- all with correct physics.
13. **Window tabbing**: Tabbed window support with correct tab bar appearance.
14. **State restoration**: App lifecycle matches macOS expectations for sudden termination and state restoration.

### Steps
1. Implement NSVisualEffectView bridge via cvkg-render-native macOS backend
2. Add transparent menubar mode with content reflection
3. Implement sidebar reflect-and-refract using displacement mapping
4. Build native window chrome with traffic light controls
5. Add global menu bar with NSMenu integration
6. Implement Stage Manager observation via NSWindow notifications
7. Recreate Spotlight-style search with fuzzy matching
8. Implement native focus ring rendering
9. Extract and match iOS/macOS spring animation parameters
10. Implement NSPasteboard integration for drag and drop
11. Add Services menu integration via NSApplication services API
12. Add trackpad gesture recognizers with correct physics
13. Add window tabbing support via NSWindow tabbing API
14. Implement state restoration using NSCoding/NSKeyedArchiver

**Estimated effort: 21-28 days**

---

## 3. Text Rendering: A+ (Current: C+)

### Definition of A+
Text rendered by CVKG is **pixel-identical** to NSTextField/NSTextView at all sizes from 9px to 72px. The text shaping system (rustybuzz + swash) is already feature-equivalent to CoreText. Remaining work is integration fidelity.

### Concrete Requirements
1. **Baseline positioning**: Every `draw_text` call computes `y = target_baseline - ascent` correctly. Zero vertical offset at any font size.
2. **Kerning at small sizes**: Glyph spacing matches CoreText output within 0.1px at 9-17px.
3. **SF-style optical sizing**: Fonts automatically adjust stroke thickness based on point size.
4. **Variable font support**: Full axis variation support (weight, width, optical size, slant).
5. **Color fonts**: SBIX/COLR/CPAL table rendering for emoji and color glyphs.
6. **SF Symbols alignment**: Symbols align to text baseline and respond to font weight/scale.
7. **Ligatures**: Contextual alternates, discretionary ligatures, required ligatures all render correctly.
8. **Bi-directional text**: Full Unicode bidi algorithm with correct glyph reordering.
9. **Subpixel positioning**: Fractional pixel positioning for crisp text at non-integer coordinates.

### Steps
1. Fix `measure_text_baseline` vtable dispatch (currently falls through to default returning 0)
2. Audit all `draw_text` callers for correct baseline positioning
3. Add letter-spacing support to `TextStyle` for small-size kerning adjustment
4. Add optical size axis handling for SF-style rendering
5. Add variable font axis API to `TextStyle`
6. Implement color font rendering in the MSDF pipeline
7. Add SF Symbols baseline alignment metadata
8. Implement full OpenType feature support (calt, liga, dlig, clig)
9. Verify bidi algorithm output against CoreText
10. Enable subpixel position snapping in the vertex shader
11. Pixel-comparison test at 9, 11, 12, 13, 15, 17, 20, 22, 28, 34, 40, 48, 72px

**Estimated effort: 5-7 days**

---

## 4. SVG Rendering: A+ (Current: D)

### Definition of A+
SVG rendering matches Safari's output within 1px RMS error for the entire SVG 1.1 test suite. All filter primitives, gradients, masks, clip paths, and animations work correctly.

### Concrete Requirements
1. **Linear gradients**: Correct color interpolation in objectBoundingBox and userSpaceOnUse coordinate systems.
2. **Radial gradients**: Correct focal point and radius interpolation.
3. **Mesh gradients**: Coons patch interpolation (SVG 2 / CSS Images Level 4).
4. **Clip paths**: Arbitrary path clipping with correct fill rule handling (non-zero, even-odd).
5. **Masks**: Luminance and alpha masking with correct compositing.
6. **Filter primitives**: feGaussianBlur, feColorMatrix, feComposite, feMerge, feOffset, feDropShadow, feBlend, feTurbulence, feDisplacementMap, feSpecularLighting -- all matching browser output.
7. **Pattern fills**: Tile patterns with correct transform and overflow handling.
8. **Stroke styles**: Dashes, line caps, line joins, miter limits -- all matching SVG spec.
9. **Text in SVG**: `<text>`, `<tspan>`, `<textPath>` routing through the text shaping engine.
10. **`<use>` element**: Symbol definition, reference resolution, and transform inheritance.
11. **CSS cascade**: Inline styles, class selectors, inheritance, and specificity matching.
12. **SMIL animations**: `<animate>`, `<animateTransform>`, `<animateMotion>` with correct timing.
13. **Coordinate transforms**: Nested `<svg>`, `<g>`, `viewBox`, `preserveAspectRatio` all compose correctly.

### Steps
1. Implement linear gradient tessellation (interpolate colors across triangles)
2. Implement radial gradient tessellation with focal point support
4. Add stencil buffer usage for clip path rendering
5. Implement alpha/luminance mask compositing in the compositor
6. Create GPU kernel library for SVG filter primitives (15+ kernels)
7. Implement pattern fill tiling in the vertex shader
8. Add dash pattern support to stroke tessellation
9. Route SVG `<text>` elements through cvkg-runic-text
10. Implement `<use>` element resolution with transform composition
11. Build CSS cascade resolver for SVG styling
12. Implement SMIL animation timing and interpolation
13. Implement nested coordinate system composition
14. Create SVG conformance test suite (render comparison vs Safari)

**Estimated effort: 14-21 days**

---

## 5. Accessibility: A+ (Current: C+)

### Definition of A+
VoiceOver, Switch Control, and all assistive technologies work identically to native AppKit. CVKG can **leapfrog Apple** by shipping a working reduce-transparency mode from day one (Tahoe's is reportedly broken in early releases).

### Concrete Requirements
1. **VoiceOver**: Full NSAccessibility protocol. Every element announces correctly. Custom actions exposed. Rotor navigation works.
2. **Native accessibility tree export**: Tree serialized to platform format (NSAccessibility, UIA, AT-SPI) in real-time.
3. **Keyboard navigation**: Tab order matches visual order exactly. Arrow keys navigate within composites. Escape dismisses. Enter/Space activate.
4. **Rotor support**: Custom rotor modes (headings, links, form controls, tables, landmarks).
5. **Semantic announcements**: `accessibilityAnnouncement` fires for dynamic content changes with correct priority.
6. **Accessibility actions**: activate, increment, decrement, scrollToVisible, custom actions -- all functional.
7. **Reduce transparency mode**: Working from day one (leapfrog Tahoe's broken implementation). Replaces blur with solid backgrounds.
8. **Reduce motion mode**: Disables all non-essential animations. Reduced particle counts.
9. **Dynamic type**: All text scales with system dynamic type setting. Layout reflows correctly at all sizes.
10. **Color contrast**: All color combinations meet WCAG AAA (7:1 ratio).
11. **Focus rings**: Native macOS focus ring appearance with correct color, radius, and animation.
12. **Live regions**: ARIA live region equivalents for dynamic content updates.

### Steps
1. Implement NSAccessibility protocol wrapper for the accessibility tree
2. Add real-time tree diffing and serialization to platform format
3. Create keyboard navigation system with focus chain management
4. Implement composite widget navigation (menus, lists, trees, tabs)
5. Build rotor system with custom action registration
6. Add announcement queue with priority and timeout handling
7. Implement accessibility action protocol with all standard actions
8. **Leapfrog**: Implement reduce-transparency mode (solid backgrounds replacing blur)
9. Wire `prefers-reduced-motion` to the animation engine
10. Implement dynamic type scaling through the layout engine
11. Add color contrast verification to the design token system
12. Implement native focus ring rendering
13. Add live region support to the accessibility announcement system
14. Test with VoiceOver, Switch Control, and Accessibility Inspector

**Estimated effort: 14-21 days**

---

## 6. Design System + Polish: A+ (Current: C+)

### Definition of A+
Every component, spacing value, animation, color, and interaction matches Apple's Human Interface Guidelines pixel-for-pixel. Includes corner radius tokens, icon silhouette contracts, hover→shader feedback, and a light Tahoe-compatible theme.

### Concrete Requirements
1. **12px-anchored corner radius system**: 4, 6, 8, 10, 12, 16, 20px radii matching Tahoe's 12px standard.
2. **Squircle icon mask**: Uniform squircle icon shape for Dock-style icons. Squircle clip layer and manifest-level shape contract.
3. **Light Tahoe theme**: Neutral-translucent color token set matching Tahoe's light mode (not just dark/neon Berserker theme).
4. **Transparent menubar mode**: Theme support for transparent menubar with content reflection.
5. **Spacing system**: 4px grid with 4, 8, 12, 16, 20, 24, 32, 40, 48px increments. Zero measurements outside this system.
6. **Typography**: Font sizes 11, 12, 13, 15, 17, 20, 22, 28, 34, 40, 48px with matching weights and tracking.
7. **Animation timing**: 200ms quick, 350ms standard, 500ms complex. Spring with damping 0.8, frequency 2.5Hz.
8. **Hover→shader feedback loop**: Wire vdom hover events to GPU uniform uploads per-component for depth shifts and specular shimmers.
9. **Interaction states**: hover (+2% brightness), pressed (-10% brightness, 0.97 scale), focused (focus ring), disabled (0.4 alpha), selected (accent color).
10. **SF Symbols icons**: At 16, 20, 24, 32, 40px with correct weight and scale.
11. **Shadows**: 3 levels (small, medium, large) matching macOS window and popover shadows.
12. **Borders**: 1px at 0.1 alpha for separators, 0.3 for focus.
13. **Scroll bars**: Overlay scroll bars with correct width, corner radius, auto-hide behavior.
14. **Cursor changes**: `pointingHand` for clickable, `IBeam` for text, resize handles for edges.
15. **Sound effects**: System beep for errors, subtle click sounds for button presses.
16. **Haptic feedback**: NSHapticFeedbackManager integration for supported interactions.

### Steps
1. Define 12px-anchored corner radius token system and audit all components
2. Implement squircle mask primitive in the GPU renderer
3. Create light Tahoe-compatible theme token set (neutral-translucent)
4. Add transparent menubar mode to the theme system
5. Define spacing token system and audit every component
6. Create typography scale matching Apple's text styles
7. Audit all animations against HIG timing specifications
8. Wire vdom hover events to GPU uniform uploads (hover→shader feedback loop)
9. Define interaction state tokens for all components
10. Integrate SF Symbols as the icon system
11. Implement 3-level shadow system matching macOS
12. Add border token system
13. Implement overlay scroll bar rendering
14. Add cursor management system
15. Integrate system sound effects
16. Add haptic feedback support
17. Full pixel-level audit against native macOS controls at all sizes
18. Create visual regression test suite

**Estimated effort: 14-21 days**

---

## 7. Micro-interactions + Hover States: A+ (New from HTML audit)

### Definition of A+
Every interactive element responds to hover/focus with subtle depth shifts and specular shimmers matching Tahoe's glass element behavior. Hover state is communicated through both visual and haptic feedback.

### Concrete Requirements
1. **Hover→shader feedback loop**: vdom hover events trigger GPU uniform uploads per-component
2. **Depth shift on hover**: Glass elements shift 1-2px in z-depth on hover
3. **Specular shimmer**: Subtle specular highlight sweeps across glass on hover
4. **Haptic feedback**: Light tap on hover enter, medium tap on activation
5. **Cursor changes**: Appropriate cursor for each element type
6. **Sound feedback**: Subtle click on activation, error beep on invalid action

### Steps
1. Create GPU uniform upload path from vdom event system to material shader
2. Add depth offset uniform to glass material (triggered by hover state)
3. Add specular sweep animation to glass material (triggered by hover enter)
4. Wire NSHapticFeedbackManager to interaction events
5. Implement cursor management system
6. Add system sound effect integration

**Estimated effort: 3-5 days**

---

## Implementation Order

### Phase 1: Foundation (Weeks 1-2)
1. Text baseline fix and kerning audit
2. Design token system (spacing, radii, colors, typography)
3. Light Tahoe theme creation
4. Corner radius audit across all components

### Phase 2: Glass + Materials (Weeks 2-4)
1. Replace Bifrost frost with true liquid glass shader
2. Implement displacement mapping and edge refraction
3. Add specular highlights (GGX, IOR 1.52)
4. Implement 18 vibrancy modes
5. Add luminance adaptation and content-aware tinting
6. Layered glass compositing

### Phase 3: SVG + Filters (Weeks 4-6)
1. SVG gradient tessellation
2. SVG stencil-based clip paths
3. SVG render graph mask compositing
4. SVG filter GPU kernels (15+ primitives)
5. SVG `<use>`, CSS cascade, SMIL animations

### Phase 4: Platform Integration (Weeks 6-9)
1. NSVisualEffectView bridge
2. Native window chrome and traffic lights
3. Menu bar integration
4. Accessibility tree export
5. Reduce transparency mode (leapfrog)
6. Keyboard navigation system
7. VoiceOver rotor and announcements
8. Drag and drop with NSPasteboard

### Phase 5: Micro-interactions + Polish (Weeks 9-11)
1. Hover→shader feedback loop
2. Squircle icon mask
3. SF Symbols integration
4. Shadow and border token system
5. Scroll bar rendering
6. Cursor management
7. Sound and haptic feedback
8. Animation curve matching

### Phase 6: Validation (Week 11-12)
1. SVG conformance test suite vs Safari
2. Accessibility audit with VoiceOver
3. Pixel comparison audit against native macOS
4. Performance profiling and optimization
5. Full regression test across all components

---

## Success Metrics (All Must Pass)

### Liquid Glass
- [ ] Visually indistinguishable from NSVisualEffectView at all 18 vibrancy modes
- [ ] Edge refraction matches Tahoe's "wet glass" within 1px RMS
- [ ] Specular highlights match PBR IOR 1.52
- [ ] Luminance adaptation responds within 100ms
- [ ] 5+ glass layers composite correctly

### Native macOS
- [ ] Window chrome pixel-identical to native
- [ ] Traffic light behavior matches exactly
- [ ] Menu bar integration works correctly
- [ ] Spring animations match UIView parameters within 5ms
- [ ] Drag and drop uses native pasteboard

### Text
- [ ] Pixel-identical to NSTextField at 9, 11, 12, 13, 15, 17, 20, 22, 28, 34, 40, 48, 72px
- [ ] Kerning matches CoreText within 0.1px
- [ ] Variable fonts interpolate correctly
- [ ] SF Symbols align to text baseline

### SVG
- [ ] SVG 1.1 test suite: >95% pass rate vs Safari
- [ ] All 15+ filter primitives produce correct output
- [ ] Gradients, masks, clip paths all render correctly
- [ ] SMIL animations match browser timing

### Accessibility
- [ ] VoiceOver navigates entire tree correctly
- [ ] Reduce transparency mode works (leapfrog Tahoe)
- [ ] Keyboard navigation matches macOS tab order
- [ ] Dynamic type scaling works at all sizes

### Design
- [ ] Zero spacing values outside 4px grid
- [ ] Zero hardcoded colors (all from tokens)
- [ ] All corner radii from 12px-anchored token system
- [ ] Squircle icon mask applied to all Dock-style icons
- [ ] Hover→shader feedback loop active on all interactive elements
- [ ] Visual regression suite passes at 99%+ match

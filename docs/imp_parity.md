# CVKG Tahoe Parity Implementation Plan -- A+ Target

**Current Parity: ~55-65%**
**Target Parity: A+ in every area (>95%)**

The rendering architecture is solid. The remaining work is fidelity -- making every pixel, every interaction, and every accessibility behavior indistinguishable from native macOS Tahoe.

---

## 1. Text Rendering: A+ (Current: C+)

### Definition of A+
Text rendered by CVKG is **pixel-identical** to NSTextField/NSTextView at all sizes from 9px to 72px. Kerning, baseline, ligatures, variable fonts, color fonts, and emoji are indistinguishable from CoreText output.

### Concrete Requirements
1. **Baseline positioning**: Every `draw_text` call computes `y = target_baseline - ascent` correctly. Zero vertical offset at any font size.
2. **Kerning**: Glyph spacing matches CoreText output within 0.1px at all sizes.
3. **Variable fonts**: Full support for axis variations (weight, width, optical size, slant) with real-time interpolation.
4. **Color fonts**: SBIX/COLR/CPAL table rendering for emoji and color glyphs.
5. **SF Symbols**: Symbols align to text baseline and respond to font weight/scale exactly like Apple's SF Symbols.
6. **Ligatures**: Contextual alternates, discretionary ligatures, and required ligatures all render correctly.
7. **Optical sizing**: Fonts automatically adjust stroke thickness based on point size.
8. **Bi-directional text**: Full Unicode bidi algorithm with correct glyph reordering.
9. **Subpixel positioning**: Fractional pixel positioning for crisp text at non-integer coordinates.

### Steps
1. Audit `cvkg-runic-text` shaping output against CoreText for the same input strings at sizes 9, 11, 13, 16, 24, 48, 72px
2. Fix `measure_text_baseline` -- the vtable dispatch issue means the default impl (always returns 0) is called. Investigate root cause: likely a trait object / cross-crate vtable issue. Fix by either making the method non-default or using a different dispatch mechanism.
3. Add variable font axis API to `TextStyle`
4. Implement color font rendering in the MSDF pipeline
5. Add SF Symbols baseline alignment metadata
6. Implement full OpenType feature support (calt, liga, dlig, clig)
7. Add optical size axis handling
8. Implement Unicode bidi algorithm (or integrate ICU)
9. Enable subpixel position snapping in the vertex shader

**Estimated effort: 5-7 days**

---

## 2. SVG Rendering: A+ (Current: C)

### Definition of A+
SVG rendering matches Safari's output within 1px RMS error for the entire SVG 1.1 test suite. All filter primitives, gradients, masks, clip paths, and animations work correctly.

### Concrete Requirements
1. **Linear gradients**: Correct color interpolation in objectBoundingBox and userSpaceOnUse coordinate systems.
2. **Radial gradients**: Correct focal point and radius interpolation.
3. **Mesh gradients**: Coons patch interpolation (SVG 2 / CSS Images Level 4).
4. **Clip paths**: Arbitrary path clipping with correct fill rule handling (non-zero, even-odd).
5. **Masks**: Luminance and alpha masking with correct compositing.
6. **Filter primitives**: feGaussianBlur, feColorMatrix, feComposite, feMerge, feOffset, feDropShadow, feBlend, feTurbulence, feDisplacementMap -- all matching browser output.
7. **Pattern fills**: Tile patterns with correct transform and overflow handling.
8. **Stroke styles**: Dashes, line caps, line joins, miter limits -- all matching SVG spec.
9. **Text in SVG**: `<text>`, `<tspan>`, `<textPath>` routing through the text shaping engine.
10. **`<use>` element**: Symbol definition, reference resolution, and transform inheritance.
11. **CSS cascade**: Inline styles, class selectors, inheritance, and specificity matching.
12. **SMIL animations**: `<animate>`, `<animateTransform>`, `<animateMotion>` with correct timing.
13. **Foreign object**: HTML content inside SVG (for mixed rendering).
14. **Coordinate transforms**: Nested `<svg>`, `<g>`, `viewBox`, `preserveAspectRatio` all compose correctly.
15. **Intersection testing**: SVG-aware hit testing for pointer events.

### Steps
1. Implement linear gradient tessellation (interpolate colors across triangles at tessellation time)
2. Implement radial gradient tessellation with focal point support
3. Add stencil buffer usage for clip path rendering
4. Implement alpha/luminance mask compositing in the compositor
5. Create GPU kernel library for SVG filter primitives (15 kernels)
6. Implement pattern fill tiling in the vertex shader
7. Add dash pattern support to stroke tessellation
8. Route SVG `<text>` elements through cvkg-runic-text
9. Implement `<use>` element resolution with transform composition
10. Build CSS cascade resolver for SVG styling
11. Implement SMIL animation timing and interpolation
12. Add foreign object placeholder support
13. Implement nested coordinate system composition
14. Add SVG-aware hit testing using the tessellated geometry
15. Create SVG conformance test suite (render comparison vs Safari)

**Estimated effort: 14-21 days**

---

## 3. Tahoe Material System: A+ (Current: B)

### Definition of A+
Glass materials are **visually indistinguishable** from NSVisualEffectView at every vibrancy setting. The material system includes specular highlights, edge refraction, depth reaction, and dynamic adaptation.

### Concrete Requirements
1. **Dynamic luminance adaptation**: Glass tint/opacity adapts in real-time based on backdrop brightness. 100ms response time.
2. **Vibrancy**: Color saturation and brightness shift matching NSVisualEffectView's `.underWindowBackground`, `.fullScreenUI`, `.hudWindow`, `.menu`, `.popover`, `.sidebar`, `.titlebar`, `.selection`, `.headerView`, `.sheet`, `.windowBackground`, `.hudWindow`, `.fullScreenUI`, `.toolTip`, `.contentBackground`, `.underPageBackground` modes.
3. **Content-aware tinting**: Dominant color extraction from backdrop subtly tints the glass surface.
4. **Layered translucency**: N glass layers composite correctly with depth-aware blending. No visual artifacts at 5+ layers.
5. **Specular highlights**: Physically-based specular response matching glass IOR (1.52). Light position awareness.
6. **Edge refraction**: Snell's law refraction at glass edges. Chromatic aberration at extreme angles.
7. **Depth reaction**: Materials respond to z-distance from camera (parallax, blur intensity variation).
8. **Subsurface scattering**: Light diffusion through translucent materials for soft glow effects.
9. **Design tokens**: Complete Tahoe design token set (colors, spacing, typography, animation curves, shadow definitions, corner radii, border widths).
10. **Animation curves**: Spring (damping ratio, frequency), cubic bezier, and timing functions matching CAMediaTimingFunction and UIView spring animations.

### Steps
1. Add compute pass for backdrop luminance sampling (per-region average and histogram)
2. Implement 18 vibrancy modes as WGSL shader variants with correct saturation/brightness curves
3. Add dominant color extraction compute pass (k-means on downsampled backdrop)
4. Implement layered translucency with depth peeling in the compositor
5. Add PBR glass shader with GGX specular model and IOR 1.52
6. Implement edge refraction using screen-space derivatives (dFdx/dFdy)
7. Add depth reaction by pass`ing scene depth to the material shader
8. Implement subsurface scattering approximation (wrap diffusion profile)
9. Create design token JSON/YAML definitions matching Apple's HIG specifications
10. Implement spring physics solver matching UISpringTimingParameters
11. Create cubic bezier timing function matching CAMediaTimingFunction control points
12. Validate against NSVisualEffectView side-by-side comparison at all modes

**Estimated effort: 14-21 days**

---

## 4. Accessibility: A+ (Current: C+)

### Definition of A+
VoiceOver, Switch Control, and all assistive technologies work identically to native AppKit. The accessibility tree is fully exported, keyboard navigation is perfect, and all semantic announcements are correct.

### Concrete Requirements
1. **VoiceOver**: Full NSAccessibility protocol implementation. Every element announces correctly. Custom actions are exposed. Rotor navigation works for headings, links, form controls, etc.
2. **Native accessibility tree export**: Tree is serialized to the platform format (NSAccessibility on macOS, UIA on Windows, AT-SPI on Linux) in real-time.
3. **Keyboard navigation**: Tab order matches visual order exactly. Arrow keys navigate within composite widgets. Escape dismisses. Enter/Space activate. All macOS keyboard shortcuts work.
4. **Rotor support**: Custom rotor modes (headings, links, form controls, tables, landmarks, visited links).
5. **Semantic announcements**: `accessibilityAnnouncement` fires for dynamic content changes with correct priority (low, medium, high).
6. **Accessibility actions**: activate, increment, decrement, scrollToVisible, custom actions -- all exposed and functional.
7. **Reduced motion**: `prefers-reduced-motion` disables all non-essential animations. Reduced transparency mode replaces blur with solid backgrounds.
8. **Dynamic type**: All text scales with the system dynamic type setting. Layout reflows correctly at all sizes.
9. **Color contrast**: All color combinations meet WCAG AAA (7:1 ratio).
10. **Focus rings**: Native macOS focus ring appearance with correct color, radius, and animation.
11. **Live regions**: ARIA live region equivalents for dynamic content updates.

### Steps
1. Implement NSAccessibility protocol wrapper for the accessibility tree
2. Add real-time tree diffing and serialization to platform format
3. Create keyboard navigation system with focus chain management
4. Implement composite widget navigation (menus, lists, trees, tabs)
5. Build rotor system with custom action registration
6. Add announcement queue with priority and timeout handling
7. Implement accessibility action protocol with all standard actions
8. Wire `prefers-reduced-motion` to the animation engine (disable spring animations, reduce particle counts)
9. Wire `prefers-reduced-transparency` to the material system (replace blur with solid fill)
10. Implement dynamic type scaling through the layout engine
11. Add color contrast verification to the design token system
12. Implement native focus ring rendering in the compositor
13. Add live region support to the accessibility announcement system
14. Test with VoiceOver, Switch Control, and Accessibility Inspector
15. Create conformance test suite against macOS accessibility expectations

**Estimated effort: 14-21 days**

---

## 5. Native macOS Fidelity: A+ (Current: D+)

### Definition of A+
The window, chrome, menus, and interactions are **pixel-identical and behaviorally identical** to native macOS. A user cannot tell the difference between a CVKG app and a native AppKit app.

### Concrete Requirements
1. **NSVisualEffectView parity**: Blur radius, vibrancy mode, material appearance matching exactly. Side-by-side comparison shows zero visual difference.
2. **Window chrome**: Traffic lights (close/minimize/maximize) with correct hover/pressed states, positioning, and behavior. Fullscreen transition matches macOS exactly.
3. **Menu bar**: Global menu bar with correct font, spacing, hover states, submenu animation, keyboard shortcut display, and dimmed items.
4. **Stage Manager**: Window resizes and repositions correctly when Stage Manager is activated/deactivated. Thumbnail preview is correct.
5. **Spotlight-style search**: Command palette with matching visual design, fuzzy search scoring, keyboard navigation, and animation timing.
6. **Focus rings**: Rendered with the exact macOS focus ring appearance (color, radius, animation, spacing).
7. **Animation curves**: Spring animations match `-[UIView animateWithDuration:delay:usingSpringWithDamping:initialSpringVelocity:options:animations:completion:]` parameters exactly. Cubic bezier curves match `+[CAMediaTimingFunction functionWithControlPoints:]`.
8. **Drag and drop**: Native `NSPasteboard` integration, drag images, drop animations, spring-loaded folders.
9. **Services menu**: App services appear in the Services submenu and work correctly.
10. **Force Touch**: Peek and pop behavior, pressure-sensitive interactions.
11. **Trackpad gestures**: Pinch to zoom, rotate, swipe between pages, smart zoom -- all with correct physics.
12. **Dictation and input**: Input methods, inline(dictation), and character picker work correctly.
13. **Window tabbing**: Tabbed window support with correct tab bar appearance and behavior.
14. **Automatic termination**: App lifecycle matches macOS expectations for sudden termination and state restoration.

### Steps
1. Capture NSVisualEffectView output at all vibrancy modes and match blur/vibrancy parameters exactly
2. Implement native window chrome with traffic light controls using `NSWindow` standardWindowButton API
3. Build global menu bar with `NSMenu` integration
4. Implement Stage Manager observation via `NSWindow` notifications
5. Recreate Spotlight-style search with fuzzy matching (fuzzywuzzy-rs or similar) and matching animations
6. Implement native focus ring using `NSTableView`/`NSButton` focus ring rendering
7. Extract and match iOS/macOS spring animation parameters (damping ratio 0.7-1.0, frequency 2-4Hz)
8. Implement `NSPasteboard` integration for drag and drop with native drag images
9. Add Services menu integration via `NSApplication` services API
10. Implement Force Touch handling via pressure-sensitive trackpad events (if hardware available)
11. Add trackpad gesture recognizers with correct physics simulation
12. Implement input method support for CJK and other complex scripts
13. Add window tabbing support via `NSWindow` tabbing API
14. Implement state restoration using `NSCoding`/`NSKeyedArchiver`

**Estimated effort: 21-28 days**

---

## 6. Design Polish: A+ (Current: C+)

### Definition of A+
Every component, spacing value, animation, color, and interaction matches Apple's Human Interface Guidelines pixel-for-pixel. The entire UI feels like it was built by Apple.

### Concrete Requirements
1. **Spacing system**: 4px grid with 4, 8, 12, 16, 20, 24, 32, 40, 48px increments. Zero measurements outside this system.
2. **Typography**: Font sizes 11, 12, 13, 15, 17, 20, 22, 28, 34, 40, 48px with matching weights (regular, medium, semibold, bold) and tracking values.
3. **Colors**: All colors from the design token system. Dark mode variants for every color. No hardcoded RGBA values anywhere.
4. **Animation timing**: 200ms for quick transitions, 350ms for standard, 500ms for complex. Spring animations with damping 0.8, frequency 2.5Hz.
5. **Interaction states**: hover (+2% brightness), pressed (-10% brightness, 0.97 scale), focused (focus ring), disabled (0.4 alpha, no interaction), selected (accent color).
6. **Icons**: SF Symbols at 16, 20, 24, 32, 40px with correct weight and scale.
7. **Corner radii**: 4, 6, 8, 10, 12, 16, 20px matching macOS controls.
8. **Shadows**: 3 shadow levels (small, medium, large) matching macOS window and popover shadows.
9. **Borders**: 1px borders at 0.1 alpha for separators, 0.3 for focus.
10. **Scroll bars**: Overlay scroll bars with correct width, corner radius, and auto-hide behavior.
11. **Cursor changes**: `pointingHand` for clickable, `IBeam` for text, `resizeLeftRight`/`resizeUpDown` for resize handles.
12. **Sound effects**: `NSBeep` for errors, subtle click sounds for button presses.
13. **Haptic feedback**: `NSHapticFeedbackManager` integration for supported interactions.

### Steps
1. Define spacing token system and audit every component
2. Create typography scale matching Apple's text styles (largeTitle, title1, title2, title3, headline, body, callout, subheadline, footnote, caption1, caption2)
3. Build complete color token system with light/dark mode variants
4. Audit all animations against HIG timing specifications
5. Define interaction state tokens for all components
6. Integrate SF Symbols as the icon system
7. Define corner radius token system
8. Implement 3-level shadow system matching macOS
9. Add border token system
10. Implement overlay scroll bar rendering
11. Add cursor management system
12. Integrate system sound effects
13. Add haptic feedback support
14. Full pixel-level audit against native macOS controls at all sizes
15. Create visual regression test suite

**Estimated effort: 14-21 days**

---

## Implementation Order

### Phase 1: Text + SVG Foundation (Weeks 1-3)
- Text baseline fix and kerning audit
- SVG gradient tessellation
- SVG stencil-based clip paths
- SVG render graph mask compositing
- Design token system creation

### Phase 2: Materials + Filters (Weeks 3-5)
- SVG filter GPU kernels (15 primitives)
- Dynamic luminance adaptation compute pass
- Vibrancy material modes (18 variants)
- Content-aware tinting
- Layered glass compositing
- Specular highlights and edge refraction

### Phase 3: Platform Integration (Weeks 5-8)
- Accessibility tree export (NSAccessibility)
- Keyboard navigation system
- VoiceOver rotor and announcements
- Native window chrome
- Menu bar integration
- Drag and drop with NSPasteboard
- Spring animation curve matching

### Phase 4: Design System + Polish (Weeks 8-10)
- Full component audit against HIG
- Typography scale matching
- Color token system completion
- Animation timing standardization
- SF Symbols integration
- Visual regression test suite
- Pixel-level comparison testing

### Phase 5: Validation (Week 10)
- SVG conformance test suite vs Safari
- Accessibility audit with VoiceOver
- Pixel comparison audit against native macOS
- Performance profiling and optimization
- Full regression test across all components

---

## Success Metrics (All Must Pass)

### Text
- [ ] Pixel-identical to NSTextField at 11, 12, 13, 15, 17, 20, 22, 28, 34, 40, 48px
- [ ] Kerning matches CoreText within 0.1px
- [ ] Variable fonts interpolate correctly across all axes
- [ ] Color fonts render correctly at all sizes
- [ ] SF Symbols align to text baseline

### SVG
- [ ] SVG 1.1 test suite: >95% pass rate vs Safari
- [ ] All 15 filter primitives produce correct output
- [ ] Gradients, masks, clip paths all render correctly
- [ ] SMIL animations match browser timing
- [ ] `<use>` element resolution works correctly

### Materials
- [ ] Glass visually indistinguishable from NSVisualEffectView at all 18 vibrancy modes
- [ ] Luminance adaptation responds within 100ms
- [ ] Layered glass (5+ layers) composites correctly
- [ ] Specular highlights match PBR IOR 1.52
- [ ] Design tokens cover 100% of UI values

### Accessibility
- [ ] VoiceOver navigates entire tree correctly
- [ ] All semantic roles and actions are exposed
- [ ] Keyboard navigation matches macOS tab order exactly
- [ ] Rotor navigation works for all element types
- [ ] Dynamic type scaling works at all system sizes
- [ ] Reduced motion and reduced transparency modes work

### Native macOS
- [ ] Window chrome is pixel-identical to native
- [ ] Traffic light behavior matches exactly
- [ ] Menu bar integration works correctly
- [ ] Spring animations match UIView parameters within 5ms
- [ ] Drag and drop uses native pasteboard
- [ ] Trackpad gestures have correct physics

### Design
- [ ] Zero spacing values outside the 4px grid
- [ ] Zero hardcoded colors (all from tokens)
- [ ] All animations within 10ms of HIG timing
- [ ] All components match HIG appearance specifications
- [ ] Visual regression suite passes at 99%+ match

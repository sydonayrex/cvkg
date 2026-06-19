# User Research Pipeline

## Purpose

Define repeatable user research procedures for evaluating CVKG's glass/liquid
rendering, accessibility, and motion behaviour. All procedures are derived from
the gaps identified in the Liquid Glass article review.

## Heuristic Evaluation Checklist

### Glass Legibility (Critical)

- [ ] Text on glass has APCA Lc >= 60 (dark theme)
- [ ] Text on glass has APCA Lc >= 60 (light theme)
- [ ] Glass tint does not reduce contrast below 4.5:1 (WCAG AA minimum)
- [ ] High-contrast mode disables glass entirely
- [ ] Glass is disabled when `prefers-reduced-transparency` is set

### Motion Sensitivity

- [ ] All spring animations respect `prefers-reduced-motion`
- [ ] Reduced motion uses snap-to-target (no gradual interpolation)
- [ ] No animation exceeds 300ms duration in reduced-motion mode
- [ ] Parallax and scroll-linked effects are disabled in reduced motion

### Touch Targets

- [ ] All interactive targets are >= 44x44 logical pixels (Apple HIG)
- [ ] All interactive targets are >= 48x48dp (Material 3)
- [ ] Spacing between adjacent targets >= 8px

### Accessibility

- [ ] All icons have `aria-label` or are marked decorative
- [ ] Focus rings are visible (APCA Lc >= 30 against adjacent colours)
- [ ] Tab order follows visual order
- [ ] Screen reader announces state changes (expanded, checked, etc.)

## Contrast Testing Procedure

### Synthetic Test (Automated)

The test `heuristic_glass_legibility` in `cvkg-tests/` creates a glass rect
over three synthetic backgrounds and asserts APCA contrast:

1. **Busy background**: 8x8 checkerboard (black/white) at 50% scale
2. **Gradient background**: linear gradient white-to-black
3. **Photo-like background**: generated Perlin noise alpha-mapped to a warm palette

For each background:
1. Render the background
2. Overlay a glass rect (blur radius 16px, tint opacity 0.3)
3. Render white text (16px, medium weight) on the glass
4. Read back pixel values in the text region
5. Compute APCA Lc between text and local backdrop
6. Assert Lc >= 60

### Manual Test Protocol

1. Set desktop wallpaper to a high-contrast photo (portrait, landscape)
2. Open CVKG demo with glass sidebar/panel active
3. Check all text labels, secondary text, and icon labels for legibility
4. Record: pass/fail per element, with screenshot
5. Repeat under bright ambient light (>500 lux) and dim (<50 lux)

## Motion Sensitivity Testing

1. Enable `prefers-reduced-motion` (OS-level setting)
2. Open CVKG demo with animations enabled (springs, transitions)
3. Verify all animations complete in <= 50ms (effectively instant)
4. Disable `prefers-reduced-motion`
5. Verify springs animate with correct physics (overshoot, settle)

## Touch Target Audit

1. Render CVKG component showcase at 1x scale
2. For each interactive element, measure bounding rect from `Rect` output
3. Assert `width >= 44.0 && height >= 44.0`
4. Compile report of failing elements

## Beta Testing

### Recruitment

Target 3-5 beta testers from CVKG Discord/GitHub with diverse setups:
- At least 1 macOS user (latest Tahoe/Sonoma)
- At least 1 Linux user (Wayland + NVidia/AMD)
- At least 1 user with accessibility needs (reduced motion, high contrast)

### Quarterly Survey Template

Adapted from Tech Edvocate metrics (65%/72%/80% thresholds):

1. "I can read text over glass panels without strain" (agree %)
2. "Animations feel natural and don't cause discomfort" (agree %)
3. "I can consistently tap the correct button/link" (agree %)
4. "Glass effects improve the visual hierarchy" (agree %)
5. "I would recommend this UI to a colleague" (NPS)

Success threshold: >= 65% agree on all items, >= 80% on legibility (item 1).

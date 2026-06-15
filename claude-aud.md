**Visual language**  
**Gap**  
**Liquid Glass / backdrop refraction**  
Tahoe's signature effect is real-time edge-refraction and specular highlights on translucent surfaces (feDisplacementMap + feSpecularLighting). CVKG has "Bifrost frosting" — a blur-based frost shader. Frost ≠ liquid glass. Bifrost has no displacement map or edge-distortion pass; it approximates glassmorphism but misses the wet-glass physics Apple ships.  
cvkg-render-gpu → bifrost shader · WGSL pipeline missing feTurbulence analog  
**Gap**  
**Corner radii**  
Tahoe uses 12px arcs uniformly across windows, dialogs, and panels. CVKG's Mjolnir clipping system is capable of arbitrary rounded rects, but the default theme tokens are undocumented and the component showcase doesn't commit to a specific radius scale. No 12px-anchored token visible in cvkg-themes.  
cvkg-themes · cvkg-components → needs radius token audit  
**Partial**  
**Transparency & translucency system**  
CVKG supports alpha-composited layers via WGPU. The "Berserker" theme leans dark/neon rather than neutral-translucent. There is no equivalent of Tahoe's transparent menubar mode or the sidebar reflect-and-refract idiom. A light Tahoe-compatible theme would require a separate color token set.  
cvkg-themes · cvkg-render-gpu compositing passes  
**Gap**  
**Icon silhouette uniformity**  
Tahoe mandates a uniform squircle icon shape across the Dock. CVKG uses GPU-accelerated SVG tessellation (lyon) for icons but has no squircle-mask primitive or icon silhouette enforcement. The vector iconography subsystem would need a squircle clip layer and a manifest-level shape contract.  
cvkg-components iconography · cvkg-render-gpu svg tessellation  
**Motion & animation**  
**Ahead**  
**Physics-based animation**  
CVKG's Sleipnir solver uses RK4 integration — more accurate than the spring-interpolation Apple uses under the hood in SwiftUI. Spring constants and damping are already first-class parameters. This is a strength; CVKG can model Tahoe's "fluid" motion idioms without new architecture.  
cvkg-anim → Sleipnir RK4 · map to NSSpringAnimation equivalents  
**Partial**  
**Micro-interaction & hover states**  
Tahoe adds subtle depth shifts and specular shimmers on hover/focus for glass elements. CVKG has event dispatch and state management via cvkg-vdom but no built-in hover→shader feedback loop. This requires wiring vdom hover events to GPU uniform uploads per-component.  
cvkg-vdom event system → cvkg-render-gpu uniform pipeline  
**Typography**  
**Ahead**  
**Text shaping & font system**  
cvkg-runic-text uses rustybuzz + swash with BiDi, global font fallback, and IME. This is feature-equivalent to or stronger than Core Text for cross-platform text. San Francisco (SF) is not available outside Apple platforms, but with correct font metrics the rendering quality can match.  
cvkg-runic-text → confirm SF-style optical sizing fallback  
**Accessibility**  
**Partial**  
**Reduce-transparency / contrast mode**  
Tahoe's "Reduce Transparency" toggle is widely reported as broken in early releases — a known regression. CVKG has AccessKit/Section 508 screen-reader support but no documented theme override for transparency reduction. Given Tahoe's regression, CVKG could leapfrog Apple here by shipping a working reduce-transparency mode from day one.  
cvkg-themes → add REDUCED_MOTION + REDUCE_TRANSPARENCY token overrides  
**Platform integration (native)**  
**Gap**  
**System chrome: menubar, sidebar, Dock**  
Tahoe's transparent menubar, Liquid Glass sidebar reflections, and Dock effects are OS-native surfaces. CVKG targets native windowing via cvkg-render-native but there is no evidence of a macOS Tahoe menubar integration or a system sidebar abstraction that reflects-and-refracts content. These require NSVisualEffectView equivalents via the native backend.  
cvkg-render-native → macOS Metal path · needs NSVisualEffectView bridge  
   

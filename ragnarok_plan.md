# RAGNAROK PLAN: CVKG Cosmic UI Integration

## Executive Summary

**Objective**: Integrate cosmic-ui inspired sci-fi components and patterns into CVKG's Cyberpunk Viking aesthetic.

**Status**: Phase 1 Planning

---

## Current State Analysis

### CVKG Strengths
- **Norse Mythology Visual Effects**: Bifrost (glass), Gungnir (neon), Mjolnir (lightning)
- **Cyberpunk Color Palette**: Cyan (#00FFFF), Magenta (#FF00FF), Amber (#FFAA00)
- **Modern Rust Architecture**: 13 specialized crates, GPU-accelerated rendering
- **Agentic Framework**: Built for AI-assisted development

### Cosmic UI Inspirations
- SVG-based irregular shapes (hexagons, trapezoids)
- Zero-dependency component philosophy
- Sci-fi dashboard/game UI patterns
- Zag.js state machine component patterns

---

## Phase 1: Irregular Shape Primitives

### 1.1 Hexagonal Shape Component

```rust
// cvkg-components/src/shapes.rs
use cvkg_core::{View, Rect, Renderer, Never};

/// A hexagonal shape primitive for sci-fi interfaces
pub struct Hexagon {
    pub size: f32,
    pub color: [f32; 4],
    pub stroke_width: f32,
}

impl Hexagon {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            color: [0.0, 0.8, 1.0, 1.0], // Cyan
            stroke_width: 2.0,
        }
    }
    
    pub fn color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

impl View for Hexagon {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (self.size / 2.0).min(rect.width / 2.0).min(rect.height / 2.0);
        
        // Calculate hexagon vertices
        let vertices: Vec<[f32; 2]> = (0..6)
            .map(|i| {
                let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
                [
                    center_x + radius * angle.cos(),
                    center_y + radius * angle.sin(),
                ]
            })
            .collect();       
        renderer.fill_polygon(&vertices, self.color);
        renderer.stroke_polygon(&vertices, [1.0, 1.0, 1.0, 0.8], self.stroke_width);
    }
    
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        let size = proposal.width.unwrap_or(self.size);
        cvkg_core::Size { width: size, height: size }
    }
}
```

### 1.2 Trapezoidal Panel

```rust
/// A trapezoidal panel for holographic displays
pub struct Trapezoid {
    pub top_width: f32,
    pub bottom_width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

impl View for Trapezoid {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let vertices = [
            [rect.x, rect.y + rect.height / 2.0], // top left
            [rect.x + rect.width, rect.y + rect.height / 2.0], // top right
            [rect.x + rect.width * 0.85, rect.y + rect.height], // bottom right
            [rect.x + rect.width * 0.15, rect.y + rect.height], // bottom left
        ];
        
        renderer.fill_polygon(&vertices, self.color);
    }
}
```

---

## Phase 2: Holographic Effects

### 2.1 Enhanced Bifrost with Dynamic Opacity

```rust
// cvkg-components/src/effects.rs
use cvkg_core::{View, Rect, Renderer, Never};

/// Holographic projection effect with scanline animation
pub struct Hologram {
    pub base_color: [f32; 4],
    pub scanline_speed: f32,
    pub flicker_intensity: f32,
}

impl Default for Hologram {
    fn default() -> Self {
        Self {
            base_color: [0.0, 0.8, 1.0, 0.3],
            scanline_speed: 2.0,
            flicker_intensity: 0.1,
        }
    }
}

impl View for Hologram {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        // Base hologram color with flicker
        let flicker = 1.0 + (t * 13.0).sin() * self.flicker_intensity;
        let color = [
            self.base_color[0] * flicker,
            self.base_color[1] * flicker,
            self.base_color[2] * flicker,
            self.base_color[3],
        ];
        
        renderer.fill_rounded_rect(rect, 8.0, color);
        
        // Animated scanlines
        let scan_y = (t * self.scanline_speed).fract() * rect.height;
        for i in 0..5 {
            let y = rect.y + (scan_y + i as f32 * 20.0) % rect.height;
            renderer.draw_line(
                rect.x, y,
                rect.x + rect.width, y,
                [0.5, 1.0, 0.8, 0.4],// Teal scanline
                1.0,
            );
        }
    }
}
```

---

## Phase 3: Radial Menu Component

### 3.1 Tactical Radial Menu

```rust
use cvkg_core::{View, Rect, Renderer, Never, SizeProposal, Size};

/// A radial menu for tactical interfaces
pub struct RadialMenu {
    items: Vec<RadialMenuItem>,
    is_open: bool,
    on_select: std::sync::Arc<dyn Fn(usize) + Send + Sync>,
}

pub struct RadialMenuItem {
    icon: String,
    label: String,
}

impl RadialMenu {
    pub fn new(on_select: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            items: Vec::new(),
            is_open: false,
            on_select: std::sync::Arc::new(on_select),
        }
    }
    
    pub fn add_item(mut self, icon: &str, label: &str) -> Self {
        self.items.push(RadialMenuItem { icon: icon.to_string(), label: label.to_string() });
        self
    }
} 

impl View for RadialMenu {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.is_open || self.items.is_empty() {
            return;
        }
        
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = (rect.width / 2.0).min(rect.height / 2.0) * 0.6;
        
        // Draw segments
        let segment_angle = 2.0 * std::f32::consts::PI / self.items.len() as f32;
        
        for (i, item) in self.items.iter().enumerate() {
            let angle = segment_angle * i as f32 - std::f32::consts::PI / 2.0;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            
            // Draw segment button
            renderer.fill_rounded_rect(
                Rect { x: x - 30.0, y: y - 30.0, width: 60.0, height: 60.0 },
                30.0,
                [0.0, 0.5, 0.8, 0.8],
            );
            
            // Draw label
            renderer.draw_text(&item.label, x - 20.0, y + 5.0, 10.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}
```

---

## Phase 4: Glitch Text Effect

### 4.1 Digital Distortion Text

```rust
/// Glitch-text effect for cyberpunk error states
pub struct GlitchText {
    pub content: String,
    pub font_size: f32,
    pub base_color: [f32; 4],
    pub glitch_intensity: f32,
}

impl View for GlitchText {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }
    
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        
        // Base text
        renderer.draw_text(&self.content, rect.x, rect.y, self.font_size, self.base_color);
        
        // Red glitch offset
        if (t * 10.0).sin().abs() > 0.8 {
            renderer.draw_text(
                &self.content,
                rect.x + (t * 15.0).sin() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [1.0, 0.0, 0.3, 0.8],
            );
        }
        
        // Blue glitch offset
        if (t * 7.0).cos().abs() > 0.85 {
            renderer.draw_text(
                &self.content,
                rect.x - (t * 12.0).cos() * self.glitch_intensity,
                rect.y,
                self.font_size,
                [0.3, 0.7, 1.0, 0.8],
            );
        }
    }
}
```

---

## Implementation Timeline

| Phase | Feature | Estimated Effort | Priority |
|-------|---------|------------------|----------|
| 1 | Irregular Shape Primitives (Hexagon, Trapezoid) | 3 days | High |
| 2 | Hologram Effect Component | 2 days | Medium |
| 3 | Radial Tactical Menu | 4 days | High |
| 4 | Glitch Text Effect | 2 days | Medium |
| 5 | Scanline Overlay Component | 1 day | Low |

---

## Integration Points

### With Existing CVKG Systems
- **Bifrost**: Combine with Hologram for frosted glass + scanlines
- **Gungnir**: Apply neon glow to irregular shapes
- **Mjolnir**: Add lightning effects to glitch transitions
- **Sleipnir**: Animate radial menu segments

### File Locations
- `cvkg-components/src/shapes.rs` - New shape primitives
- `cvkg-components/src/effects.rs` - Enhanced effects
- `cvkg-components/src/hud.rs` - Radial menu and tactical components

---

## Success Criteria

- [ ] Hexagon component renders correctly with Gungnir glow
- [ ] Hologram effect animates scanlines smoothly
- [ ] Radial menu supports 6+ segments with selection callback
- [ ] Glitch text performs well under 60fps
- [ ] All components work with existing Bifrost/Gungnir effects
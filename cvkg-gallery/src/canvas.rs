//! Canvas — live component preview area with real showcase implementations.

use crate::state::{GalleryState, Registry};
use cvkg_components::{Button, Divider, HStack, Progress, Select, Text, Toggle, VStack};
use cvkg_components::interactive::hrungnirsegmented::{HrungnirSegmented as SegmentedControl, SegmentedStyle};
use cvkg_core::{Rect, Renderer, View, Size, SizeProposal, AnyView};
use std::sync::{Arc, Mutex};

/// Canvas for rendering the selected component.
#[derive(Clone)]
pub struct GalleryCanvas {
    state: Arc<Mutex<GalleryState>>,
}

impl GalleryCanvas {
    pub fn new(state: Arc<Mutex<GalleryState>>) -> Self {
        Self { state }
    }

    fn get_current_showcase(&self) -> Option<AnyView> {
        let state = self.state.lock().unwrap();
        let name = state.selected_component.as_ref()?;
        Registry::find(name).map(|meta| (meta.factory)(&*state))
    }
}

impl View for GalleryCanvas {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let showcase = self.get_current_showcase();
        let scale = {
            let state = self.state.lock().unwrap();
            state.scale.factor()
        };
        let bg = {
            let state = self.state.lock().unwrap();
            state.canvas_bg
        };

        // Apply theme background
        let bg_color = match bg {
            crate::state::CanvasBackground::Light => [0.98, 0.98, 0.98, 1.0],
            crate::state::CanvasBackground::Dark => [0.15, 0.15, 0.15, 1.0],
            crate::state::CanvasBackground::Checkered => [0.9, 0.9, 0.9, 1.0],
            crate::state::CanvasBackground::Transparent => [0.0, 0.0, 0.0, 0.0],
        };
        renderer.fill_rect(rect, bg_color);

        // Draw checkered pattern if selected
        if matches!(bg, crate::state::CanvasBackground::Checkered) {
            let square = 20.0;
            for x in (rect.x as i32..(rect.x + rect.width) as i32).step_by(square as usize * 2) {
                for y in (rect.y as i32..(rect.y + rect.height) as i32).step_by(square as usize * 2) {
                    let alt = ((x / square as i32) + (y / square as i32)) % 2 == 0;
                    let color = if alt { [0.95, 0.95, 0.95, 1.0] } else { [0.85, 0.85, 0.85, 1.0] };
                    let sq_rect = Rect {
                        x: x as f32,
                        y: y as f32,
                        width: square,
                        height: square,
                    };
                    renderer.fill_rect(sq_rect, color);
                }
            }
        }

        // Render the showcase component if available
        if let Some(showcase) = showcase {
            // Apply scale transform using push_transform
            if scale != 1.0 {
                let cx = rect.x + rect.width / 2.0;
                let cy = rect.y + rect.height / 2.0;
                renderer.push_transform([cx, cy], [scale, scale], 0.0);
            }

            // Center the showcase in the canvas
            let showcase_size = showcase.intrinsic_size(renderer, cvkg_core::SizeProposal {
                width: Some(rect.width * 0.8),
                height: Some(rect.height * 0.8),
            });
            let showcase_rect = Rect {
                x: rect.x + (rect.width - showcase_size.width) / 2.0,
                y: rect.y + (rect.height - showcase_size.height) / 2.0,
                width: showcase_size.width,
                height: showcase_size.height,
            };
            showcase.render(renderer, showcase_rect);

            if scale != 1.0 {
                renderer.pop_transform();
            }
        } else {
            // Empty state - no component selected
            renderer.push_vnode(rect, "GalleryCanvas");
            let msg = "Select a component from the sidebar";
            let msg_w = renderer.measure_text(msg, 16.0).0;
            renderer.draw_text_raw(
                msg,
                rect.x + (rect.width - msg_w) / 2.0,
                rect.y + rect.height / 2.0,
                16.0,
                [0.5, 0.5, 0.5, 1.0],
            );
            renderer.pop_vnode();
        }
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(800.0), height: proposal.height.unwrap_or(600.0) }
    }
}

impl cvkg_core::layout::LayoutView for GalleryCanvas {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size { width: proposal.width.unwrap_or(800.0), height: proposal.height.unwrap_or(600.0) }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        // Leaf view
    }
}

/// Toolbar at the top of the canvas.
#[derive(Clone)]
pub struct CanvasToolbar {
    state: Arc<Mutex<GalleryState>>,
    on_theme_change: Arc<dyn Fn(crate::state::GalleryTheme) + Send + Sync>,
    on_scale_change: Arc<dyn Fn(crate::state::ViewportScale) + Send + Sync>,
    on_bg_change: Arc<dyn Fn(crate::state::CanvasBackground) + Send + Sync>,
    on_viewport_change: Arc<dyn Fn(crate::state::ViewportPreset) + Send + Sync>,
}

impl CanvasToolbar {
    pub fn new(
        state: Arc<Mutex<GalleryState>>,
        on_theme_change: impl Fn(crate::state::GalleryTheme) + Send + Sync + 'static,
        on_scale_change: impl Fn(crate::state::ViewportScale) + Send + Sync + 'static,
        on_bg_change: impl Fn(crate::state::CanvasBackground) + Send + Sync + 'static,
        on_viewport_change: impl Fn(crate::state::ViewportPreset) + Send + Sync + 'static,
    ) -> Self {
        Self {
            state,
            on_theme_change: Arc::new(on_theme_change),
            on_scale_change: Arc::new(on_scale_change),
            on_bg_change: Arc::new(on_bg_change),
            on_viewport_change: Arc::new(on_viewport_change),
        }
    }
}

impl View for CanvasToolbar {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let state = self.state.lock().unwrap();
        let theme = state.theme;
        let scale = state.scale;
        let bg = state.canvas_bg;
        let viewport = state.viewport;
        drop(state);

        // Toolbar background
        renderer.fill_rect(rect, [0.95, 0.95, 0.95, 0.8]);
        renderer.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            [0.7, 0.7, 0.7, 1.0],
            1.0,
        );

        let y = rect.y + 6.0;
        let segment_height = 32.0;
        let mut x = rect.x + 16.0;
        let spacing = 12.0;
        let right_reserved = 160.0 + 32.0; // Cmd+K hint + screenshot btn

        // Calculate available width for 4 segments dynamically
        let total_padding_and_spacing = 32.0 + (3.0 * spacing) + right_reserved;
        let available = rect.width - total_padding_and_spacing;
        let segment_width = (available / 4.0).clamp(120.0, 220.0);

        // Theme selector - using SegmentedControl
        let theme_labels = ["Light", "Dark", "High Contrast"];
        let theme_options = theme_labels.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let theme_idx = match theme {
            crate::state::GalleryTheme::Light => 0,
            crate::state::GalleryTheme::Dark => 1,
            crate::state::GalleryTheme::HighContrast => 2,
        };
        let theme_rect = Rect {
            x,
            y,
            width: segment_width,
            height: segment_height,
        };
        let state_clone = self.state.clone();
        let theme_segmented = cvkg_components::SegmentedControl::new(theme_options, theme_idx)
            .style(cvkg_components::interactive::hrungnirsegmented::SegmentedStyle::Capsule)
            .on_select(move |idx| {
                let mut state = state_clone.lock().unwrap();
                state.theme = match idx {
                    0 => crate::state::GalleryTheme::Light,
                    1 => crate::state::GalleryTheme::Dark,
                    _ => crate::state::GalleryTheme::HighContrast,
                };
            });
        theme_segmented.render(renderer, theme_rect);
        x += theme_rect.width + spacing;

        // Scale selector - using SegmentedControl
        let scale_labels: Vec<String> = crate::state::ViewportScale::ALL
            .iter()
            .map(|s| format!("{}%", *s as u32))
            .collect();
        let scale_idx = crate::state::ViewportScale::ALL.iter().position(|s| *s == scale).unwrap_or(2);
        let scale_rect = Rect {
            x,
            y,
            width: segment_width,
            height: segment_height,
        };
        let state_clone = self.state.clone();
        let scale_segmented = cvkg_components::SegmentedControl::new(scale_labels, scale_idx)
            .style(cvkg_components::interactive::hrungnirsegmented::SegmentedStyle::Capsule)
            .on_select(move |idx| {
                if let Some(new_scale) = crate::state::ViewportScale::ALL.get(idx) {
                    let mut state = state_clone.lock().unwrap();
                    state.scale = *new_scale;
                }
            });
        scale_segmented.render(renderer, scale_rect);
        x += scale_rect.width + spacing;

        // Background selector - using SegmentedControl
        let bg_labels = ["Light", "Dark", "Checkered", "Transparent"];
        let bg_idx = match bg {
            crate::state::CanvasBackground::Light => 0,
            crate::state::CanvasBackground::Dark => 1,
            crate::state::CanvasBackground::Checkered => 2,
            crate::state::CanvasBackground::Transparent => 3,
        };
        let bg_rect = Rect {
            x,
            y,
            width: segment_width,
            height: segment_height,
        };
        let bg_options = bg_labels.iter().map(|s| s.to_string()).collect();
        let state_clone = self.state.clone();
        let bg_segmented = cvkg_components::SegmentedControl::new(bg_options, bg_idx)
            .style(cvkg_components::interactive::hrungnirsegmented::SegmentedStyle::Capsule)
            .on_select(move |idx| {
                let mut state = state_clone.lock().unwrap();
                state.canvas_bg = match idx {
                    0 => crate::state::CanvasBackground::Light,
                    1 => crate::state::CanvasBackground::Dark,
                    2 => crate::state::CanvasBackground::Checkered,
                    _ => crate::state::CanvasBackground::Transparent,
                };
            });
        bg_segmented.render(renderer, bg_rect);
        x += bg_rect.width + spacing;

        // Viewport preset selector - using SegmentedControl
        let viewport_labels: Vec<String> = crate::state::ViewportPreset::ALL
            .iter()
            .map(|p| p.label().to_string())
            .collect();
        let viewport_idx = crate::state::ViewportPreset::ALL.iter().position(|p| *p == viewport).unwrap_or(0);
        let viewport_rect = Rect {
            x,
            y,
            width: segment_width,
            height: segment_height,
        };
        let viewport_options = viewport_labels.iter().map(|s| s.to_string()).collect();
        let state_clone = self.state.clone();
        let viewport_segmented = cvkg_components::SegmentedControl::new(viewport_options, viewport_idx)
            .style(cvkg_components::interactive::hrungnirsegmented::SegmentedStyle::Capsule)
            .on_select(move |idx| {
                if let Some(new_viewport) = crate::state::ViewportPreset::ALL.get(idx) {
                    let mut state = state_clone.lock().unwrap();
                    state.viewport = *new_viewport;
                }
            });
        viewport_segmented.render(renderer, viewport_rect);
        x += viewport_rect.width + spacing;

        // Spotlight hint
        renderer.draw_text_raw(
            "Cmd+K to search",
            rect.x + rect.width - 160.0,
            rect.y + 10.0,
            12.0,
            [0.5, 0.5, 0.5, 1.0],
        );

        // Screenshot export button
        let screenshot_rect = Rect {
            x: rect.x + rect.width - 40.0,
            y: rect.y + 6.0,
            width: 32.0,
            height: 32.0,
        };
        let screenshot_btn = cvkg_components::Button::new("📷", || {
            eprintln!("Screenshot capture requested");
        })
        .variant(cvkg_components::ButtonVariant::Ghost)
        .frame(Some(32.0), Some(32.0));
        screenshot_btn.render(renderer, screenshot_rect);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(800.0), height: 44.0 }
    }
}

impl cvkg_core::layout::LayoutView for CanvasToolbar {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size { width: proposal.width.unwrap_or(800.0), height: 44.0 }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        // Leaf
    }
}

/// Button showcase implementation
#[derive(Clone)]
pub struct ButtonShowcase;

impl View for ButtonShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ButtonShowcase");
        
        // Title
        renderer.draw_text_raw("Button Variants", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let variants = [
            ("Primary", cvkg_components::Button::new("Primary Button", || {})),
            ("Secondary", cvkg_components::Button::new("Secondary Button", || {}).variant(cvkg_components::ButtonVariant::Secondary)),
            ("Destructive", cvkg_components::Button::new("Delete", || {}).variant(cvkg_components::ButtonVariant::Destructive)),
            ("Ghost", cvkg_components::Button::new("Ghost Button", || {}).variant(cvkg_components::ButtonVariant::Ghost)),
            ("Disabled", cvkg_components::Button::new("Disabled", || {}).disabled(true)),
        ];
        
        let mut y = rect.y + 60.0;
        for (label, btn) in variants {
            renderer.draw_text_raw(label, rect.x + 16.0, y - 24.0, 12.0, [0.5, 0.5, 0.5, 1.0]);
            let btn_rect = Rect { x: rect.x + 16.0, y, width: 180.0, height: 38.0 };
            btn.render(renderer, btn_rect);
            y += 50.0;
        }
        
        // Button sizes section
        y += 10.0;
        renderer.draw_text_raw("Button Sizes", rect.x + 16.0, y, 16.0, [0.2, 0.2, 0.2, 1.0]);
        y += 30.0;
        
        let sizes = [
            ("Small", cvkg_components::Button::new("Small", || {}).size(cvkg_components::ButtonSize::Small)),
            ("Default", cvkg_components::Button::new("Default", || {}).size(cvkg_components::ButtonSize::Default)),
            ("Large", cvkg_components::Button::new("Large", || {}).size(cvkg_components::ButtonSize::Large)),
        ];
        
        for (label, btn) in sizes {
            renderer.draw_text_raw(label, rect.x + 16.0, y - 24.0, 12.0, [0.5, 0.5, 0.5, 1.0]);
            let btn_rect = Rect { x: rect.x + 16.0, y, width: 180.0, height: 38.0 };
            btn.render(renderer, btn_rect);
            y += 50.0;
        }
        
        // Loading state
        y += 10.0;
        renderer.draw_text_raw("Loading State", rect.x + 16.0, y, 16.0, [0.2, 0.2, 0.2, 1.0]);
        y += 30.0;
        
        let loading_btn = cvkg_components::Button::new("Loading...", || {}).loading(true);
        let btn_rect = Rect { x: rect.x + 16.0, y, width: 180.0, height: 38.0 };
        loading_btn.render(renderer, btn_rect);
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 350.0 }
    }
}

impl cvkg_core::layout::LayoutView for ButtonShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 350.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Checkbox showcase
#[derive(Clone)]
pub struct CheckboxShowcase;

impl View for CheckboxShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "CheckboxShowcase");
        renderer.draw_text_raw("Checkbox States", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let checks = [
            ("Unchecked", false, false),
            ("Checked", true, false),
            ("Indeterminate", false, true),
        ];
        
        let mut y = rect.y + 50.0;
        for (label, checked, indeterminate) in checks {
            renderer.draw_text_raw(label, rect.x + 16.0, y + 4.0, 14.0, [0.3, 0.3, 0.3, 1.0]);
            let chk_rect = Rect { x: rect.x + 16.0, y, width: 22.0, height: 22.0 };
            let chk = cvkg_components::Checkbox::new(checked, |_| {}).label(label);
            if indeterminate { /* would need indeterminate variant */ }
            chk.render(renderer, chk_rect);
            y += 35.0;
        }
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
}

impl cvkg_core::layout::LayoutView for CheckboxShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Input showcase
#[derive(Clone)]
pub struct InputShowcase;

impl View for InputShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "InputShowcase");
        renderer.draw_text_raw("Text Input Variants", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let variants = [
            ("Default", cvkg_components::Input::new("Placeholder")),
            ("With Value", cvkg_components::Input::new("Hello World")),
            ("Disabled", cvkg_components::Input::new("Disabled input").focused(false)),
        ];
        
        let mut y = rect.y + 50.0;
        for (label, input) in variants {
            renderer.draw_text_raw(label, rect.x + 16.0, y - 24.0, 12.0, [0.5, 0.5, 0.5, 1.0]);
            let input_rect = Rect { x: rect.x + 16.0, y, width: 280.0, height: 38.0 };
            input.render(renderer, input_rect);
            y += 50.0;
        }
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
}

impl cvkg_core::layout::LayoutView for InputShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Select showcase
#[derive(Clone)]
pub struct SelectShowcase;

impl View for SelectShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SelectShowcase");
        renderer.draw_text_raw("Dropdown Select", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let options = vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()];
        let select = cvkg_components::Select::new("Choose an option")
            .option("Option 1", 0)
            .option("Option 2", 1)
            .option("Option 3", 2)
            .selected(0)
            .frame(Some(200.0), Some(38.0));
        let select_rect = Rect { x: rect.x + 16.0, y: rect.y + 50.0, width: 200.0, height: 38.0 };
        select.render(renderer, select_rect);
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 300.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for SelectShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 300.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Toggle showcase
#[derive(Clone)]
pub struct ToggleShowcase;

impl View for ToggleShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ToggleShowcase");
        renderer.draw_text_raw("Toggle Switches", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let toggles = [
            ("Off", false),
            ("On", true),
            ("Disabled Off", false),
            ("Disabled On", true),
        ];
        
        let mut y = rect.y + 50.0;
        for (label, state) in toggles {
            renderer.draw_text_raw(label, rect.x + 16.0, y + 4.0, 14.0, [0.3, 0.3, 0.3, 1.0]);
            let toggle = cvkg_components::Toggle::new(label, state, |_| {});
            let toggle_rect = Rect { x: rect.x + 16.0, y, width: 200.0, height: 30.0 };
            toggle.render(renderer, toggle_rect);
            y += 40.0;
        }
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
}

impl cvkg_core::layout::LayoutView for ToggleShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Slider showcase
#[derive(Clone)]
pub struct SliderShowcase;

impl View for SliderShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SliderShowcase");
        renderer.draw_text_raw("Range Sliders", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        
        let sliders = [
            ("Default (0-1)", 0.0..=1.0, 0.5),
            ("Percentage (0-100)", 0.0..=100.0, 50.0),
            ("Disabled", 0.0..=100.0, 75.0),
        ];
        
        let mut y = rect.y + 50.0;
        for (label, range, value) in sliders {
            renderer.draw_text_raw(label, rect.x + 16.0, y - 24.0, 12.0, [0.5, 0.5, 0.5, 1.0]);
            let slider = cvkg_components::Slider::new(value, range, |_| {}).frame(Some(280.0), Some(38.0));
            let slider_rect = Rect { x: rect.x + 16.0, y, width: 280.0, height: 38.0 };
            slider.render(renderer, slider_rect);
            y += 50.0;
        }
        
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
}

impl cvkg_core::layout::LayoutView for SliderShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 250.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// DatePicker showcase
#[derive(Clone)]
pub struct DatePickerShowcase;

impl View for DatePickerShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DatePickerShowcase");
        renderer.draw_text_raw("Date Picker", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Calendar-based date selection", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
}

impl cvkg_core::layout::LayoutView for DatePickerShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// SearchField showcase
#[derive(Clone)]
pub struct SearchFieldShowcase;

impl View for SearchFieldShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SearchFieldShowcase");
        renderer.draw_text_raw("Search Field with Suggestions", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Auto-complete search input", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for SearchFieldShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Dialog showcase
#[derive(Clone)]
pub struct DialogShowcase;

impl View for DialogShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DialogShowcase");
        renderer.draw_text_raw("Modal Dialog", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Dialog with title, content, and actions", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for DialogShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Popover showcase
#[derive(Clone)]
pub struct PopoverShowcase;

impl View for PopoverShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "PopoverShowcase");
        renderer.draw_text_raw("Popover", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Floating anchored content", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for PopoverShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Tooltip showcase
#[derive(Clone)]
pub struct TooltipShowcase;

impl View for TooltipShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TooltipShowcase");
        renderer.draw_text_raw("Tooltip", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Hover-activated helper text", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for TooltipShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// ContextMenu showcase
#[derive(Clone)]
pub struct ContextMenuShowcase;

impl View for ContextMenuShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ContextMenuShowcase");
        renderer.draw_text_raw("Context Menu", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Right-click contextual actions", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for ContextMenuShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// VStack showcase
#[derive(Clone)]
pub struct VStackShowcase;

impl View for VStackShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "VStackShowcase");
        renderer.draw_text_raw("Vertical Stack", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Vertical layout with alignment", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for VStackShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// HStack showcase
#[derive(Clone)]
pub struct HStackShowcase;

impl View for HStackShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "HStackShowcase");
        renderer.draw_text_raw("Horizontal Stack", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Horizontal layout with distribution", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for HStackShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Grid showcase
#[derive(Clone)]
pub struct GridShowcase;

impl View for GridShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "GridShowcase");
        renderer.draw_text_raw("Responsive Grid", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Auto-fit responsive grid layout", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for GridShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Divider showcase
#[derive(Clone)]
pub struct DividerShowcase;

impl View for DividerShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DividerShowcase");
        renderer.draw_text_raw("Divider", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Visual separator lines", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for DividerShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Table showcase
#[derive(Clone)]
pub struct TableShowcase;

impl View for TableShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TableShowcase");
        renderer.draw_text_raw("Data Table", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Sortable, selectable data grid", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 500.0, height: 300.0 }
    }
}

impl cvkg_core::layout::LayoutView for TableShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 500.0, height: 300.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Badge showcase
#[derive(Clone)]
pub struct BadgeShowcase;

impl View for BadgeShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BadgeShowcase");
        renderer.draw_text_raw("Status Badges", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Variants: default, secondary, destructive, outline", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for BadgeShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Avatar showcase
#[derive(Clone)]
pub struct AvatarShowcase;

impl View for AvatarShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AvatarShowcase");
        renderer.draw_text_raw("User Avatars", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Image, initials, status indicator", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for AvatarShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Progress showcase
#[derive(Clone)]
pub struct ProgressShowcase;

impl View for ProgressShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ProgressShowcase");
        renderer.draw_text_raw("Progress Bars", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Determinate & indeterminate", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for ProgressShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Toast showcase
#[derive(Clone)]
pub struct ToastShowcase;

impl View for ToastShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ToastShowcase");
        renderer.draw_text_raw("Toast Notifications", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Transient feedback messages", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for ToastShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Alert showcase
#[derive(Clone)]
pub struct AlertShowcase;

impl View for AlertShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AlertShowcase");
        renderer.draw_text_raw("Alert Messages", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Inline contextual alerts", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for AlertShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Skeleton showcase
#[derive(Clone)]
pub struct SkeletonShowcase;

impl View for SkeletonShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SkeletonShowcase");
        renderer.draw_text_raw("Loading Skeletons", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Loading placeholder components", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for SkeletonShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Tabs showcase
#[derive(Clone)]
pub struct TabsShowcase;

impl View for TabsShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TabsShowcase");
        renderer.draw_text_raw("Tab Navigation", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Tab panels with content", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
}

impl cvkg_core::layout::LayoutView for TabsShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 200.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Breadcrumb showcase
#[derive(Clone)]
pub struct BreadcrumbShowcase;

impl View for BreadcrumbShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "BreadcrumbShowcase");
        renderer.draw_text_raw("Breadcrumb Trail", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Hierarchical navigation", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for BreadcrumbShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Pagination showcase
#[derive(Clone)]
pub struct PaginationShowcase;

impl View for PaginationShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "PaginationShowcase");
        renderer.draw_text_raw("Pagination Controls", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Page navigation", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
}

impl cvkg_core::layout::LayoutView for PaginationShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 150.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// ColorPicker showcase
#[derive(Clone)]
pub struct ColorPickerShowcase;

impl View for ColorPickerShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ColorPickerShowcase");
        renderer.draw_text_raw("Color Picker", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Multi-format color selection", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 400.0, height: 300.0 }
    }
}

impl cvkg_core::layout::LayoutView for ColorPickerShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 400.0, height: 300.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// CommandPalette showcase
#[derive(Clone)]
pub struct CommandPaletteShowcase;

impl View for CommandPaletteShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "CommandPaletteShowcase");
        renderer.draw_text_raw("Command Palette", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("MimirSpotlight integration", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 500.0, height: 300.0 }
    }
}

impl cvkg_core::layout::LayoutView for CommandPaletteShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 500.0, height: 300.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}

/// Sidebar showcase
#[derive(Clone)]
pub struct SidebarShowcase;

impl View for SidebarShowcase {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SidebarShowcase");
        renderer.draw_text_raw("NiflheimSidebar", rect.x + 16.0, rect.y + 24.0, 18.0, [0.2, 0.2, 0.2, 1.0]);
        renderer.draw_text_raw("Collapsible navigation sidebar", rect.x + 16.0, rect.y + 50.0, 14.0, [0.5, 0.5, 0.5, 1.0]);
        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size { width: 240.0, height: 500.0 }
    }
}

impl cvkg_core::layout::LayoutView for SidebarShowcase {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) -> Size {
        Size { width: 240.0, height: 500.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView], _cache: &mut cvkg_core::layout::LayoutCache) {}
}
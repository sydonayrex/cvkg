use crate::theme;
use crate::{RADIUS_LG, RADIUS_MD};
use cvkg_core::{AriaProperties, AriaRole, Event, Never, Rect, Renderer, View};
use std::sync::Arc;

/// Liquid glass tabs with chromatic aberration.
/// Section 4.7: "Tactile realm-switching navigation with fluid feedback."
#[derive(Clone)]
#[doc(alias = "Tabs")]
pub struct BifrostTabs {
    pub options: Vec<String>,
    pub selected_index: usize,
    pub on_select: Arc<dyn Fn(usize) + Send + Sync>,
    /// Optional callback invoked with the index of the tab to close.
    pub on_close: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    /// Whether tabs show a close button (default: false).
    pub closable: bool,
}

impl BifrostTabs {
    /// Create a new BifrostTabs component with the given tab labels and selection callback.
    ///
    /// # Examples
    /// ```
    /// use cvkg_components::BifrostTabs;
    /// let tabs = BifrostTabs::new(
    ///     vec!["Tab 1".to_string(), "Tab 2".to_string()],
    ///     0,
    ///     |idx| println!("Selected tab: {}", idx),
    /// );
    /// ```
    pub fn new(
        options: Vec<String>,
        selected: usize,
        on_select: impl Fn(usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            options,
            selected_index: selected,
            on_select: Arc::new(on_select),
            on_close: None,
            closable: false,
        }
    }

    /// Set whether tabs display a close button.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set the callback invoked when a tab's close button is clicked.
    pub fn on_close(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }
}

impl View for BifrostTabs {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn layout(&self) -> Option<&dyn cvkg_core::LayoutView> {
        Some(self)
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: cvkg_core::SizeProposal) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: 44.0,
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let t = renderer.elapsed_time();
        let tab_width = rect.width / self.options.len() as f32;

        // 1. Background: solid dark background. Glassmorphism sits on top.
        renderer.fill_rounded_rect(
            rect,
            RADIUS_LG,
            [0.10, 0.05, 0.15, 0.95], // ceramic black / dark metal
        );
        renderer.stroke_rounded_rect(
            rect,
            RADIUS_LG,
            [0.70, 0.50, 0.90, 0.60], // bright purple border
            1.5,
        );

        // 2. Liquid Selection Indicator (The Bifrost bridge)
        let target_x = rect.x + (self.selected_index as f32 * tab_width);

        // Animated indicator with "jelly" physics (sinusoidal wobble)
        let wobble = (t * 4.0).sin() * 2.0;
        let indicator_rect = Rect {
            x: target_x + 4.0,
            y: rect.y + 4.0 + wobble,
            width: tab_width - 8.0,
            height: rect.height - 8.0,
        };

        // Draw indicator background first
        renderer.fill_rounded_rect(
            indicator_rect,
            RADIUS_MD,
            theme::surface_high_contrast(),
        );

        // 3. Tab Labels - draw BEFORE glass effect so text sits on top
        for (i, option) in self.options.iter().enumerate() {
            let x = rect.x + (i as f32 * tab_width);
            let alpha = if i == self.selected_index { 1.0 } else { 0.6 };

            // Account for close button width when positioning label
            let label_offset = if self.closable { 12.0 } else { 0.0 };
            let font_size = 14.0;

            let text_color = if i == self.selected_index {
                [0.0, 0.0, 0.0, alpha] // black — visible against bloom-lit tab
            } else {
                [1.0, 1.0, 1.0, alpha]
            };

            let (tw, _th) = renderer.measure_text(option, font_size);
            renderer.draw_text(
                option,
                x + label_offset + (tab_width - tw) / 2.0,
                rect.y + rect.height / 2.0 + 5.0,
                font_size,
                text_color,
            );
        }

        // Close button (×) -- min 24x24px hit target
        if self.closable {
            for (i, _option) in self.options.iter().enumerate() {
                let x = rect.x + (i as f32 * tab_width);
                let alpha = if i == self.selected_index { 1.0 } else { 0.6 };
                let close_size = 24.0_f32;
                let close_x = x + tab_width - close_size - 4.0;
                let close_y = rect.y + (rect.height - close_size) / 2.0;

                renderer.draw_text(
                    "×",
                    close_x + 6.0,
                    close_y + 4.0,
                    14.0,
                    [0.8, 0.4, 0.4, alpha],
                );

                // Close button hit target
                if let Some(on_close) = self.on_close.as_ref() {
                    let on_close = on_close.clone();
                    let close_sz = close_size;
                    let idx = i;
                    renderer.register_handler(
                        &format!("pointerdown_close_{}", i),
                        Arc::new(move |ev| {
                            if let Event::PointerDown { x, y, .. } = ev {
                                if x >= close_x
                                && x <= close_x + close_sz
                                && y >= close_y
                                && y <= close_y + close_sz
                            {
                                on_close(idx);
                            }
                            }
                        }),
                    );
                }
            }
        }

        // 4. Glass effect drawn LAST so it doesn't wash out text
        if crate::theme::glassmorphism_enabled() {
            renderer.gungnir(indicator_rect, [0.8, 0.4, 1.0, 0.9], 15.0, 0.9);
        }

        // Single Interaction Handler for all tabs
        let on_select = self.on_select.clone();
        let tab_w = tab_width;
        let start_x = rect.x;
        let num_tabs = self.options.len();
        let rect_y = rect.y;
        let rect_h = rect.height;
        renderer.register_handler(
            "pointerdown",
            Arc::new(move |ev| {
                if let cvkg_core::Event::PointerDown { x, y, .. } = ev {
                    if y >= rect_y && y <= rect_y + rect_h && x >= start_x && x <= start_x + (num_tabs as f32 * tab_w) {
                        let idx = ((x - start_x) / tab_w) as usize;
                        if idx < num_tabs {
                            on_select(idx);
                        }
                    }
                }
            }),
        );

        // 4. Keyboard navigation: Arrow Left/Right/Tab to switch, W to close
        let tab_count = self.options.len();
        let selected = self.selected_index;
        let on_select = self.on_select.clone();
        let on_close = self.on_close.clone();
        let closable = self.closable;
        renderer.register_handler(
            "keydown",
            Arc::new(move |event| {
                if let Event::KeyDown { key, .. } = event {
                    match key.as_str() {
                        "ArrowRight" if tab_count > 0 => {
                            let next = (selected + 1) % tab_count;
                            on_select(next);
                        }
                        "ArrowLeft" if tab_count > 0 => {
                            let prev = if selected == 0 {
                                tab_count - 1
                            } else {
                                selected - 1
                            };
                            on_select(prev);
                        }
                        "Tab" if tab_count > 0 => {
                            let next = (selected + 1) % tab_count;
                            on_select(next);
                        }
                        "w" | "W" => {
                            if closable && let Some(ref cb) = on_close {
                                cb(selected);
                            }
                        }
                        _ => {}
                    }
                }
            }),
        );
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        Some(AriaProperties::new(AriaRole::Tablist, "Tabs"))
    }
}

impl cvkg_core::LayoutView for BifrostTabs {
    fn size_that_fits(
        &self,
        proposal: cvkg_core::SizeProposal,
        _subviews: &[&dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) -> cvkg_core::Size {
        cvkg_core::Size {
            width: proposal.width.unwrap_or(300.0),
            height: 44.0,
        }
    }

    fn place_subviews(
        &self,
        _bounds: cvkg_core::Rect,
        _subviews: &mut [&mut dyn cvkg_core::LayoutView],
        _cache: &mut cvkg_core::LayoutCache,
    ) {}
}

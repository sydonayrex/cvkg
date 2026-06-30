//! Text animation components: TextAnimate, TypewriterEffect, NumberTicker.
//! Card effect components: CardStack, CardHoverEffect, ExpandableCard, DraggableCard.
//! Button components: ShimmerButton, RippleButton, StatefulButton.
//! All components use the cvkg theme system (theme::* helpers) for full themability.

use crate::RADIUS_XL;
use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// =============================================================================
// TEXT ANIMATIONS
// =============================================================================

/// Animation effect variant for TextAnimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAnimEffect {
    Fade,
    Slide,
    Scale,
    Blur,
}

/// TextAnimate -- animate text with various effects (fade, slide, scale, blur).
#[derive(Clone)]
pub struct TextAnimate {
    /// Text content.
    pub text: String,
    /// Font size.
    pub font_size: f32,
    /// Animation effect type.
    pub effect: TextAnimEffect,
    /// Animation progress 0.0..1.0.
    pub progress: f32,
    /// Text color.
    pub color: [f32; 4],
}

impl TextAnimate {
    /// Create a new TextAnimate.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            font_size: 24.0,
            effect: TextAnimEffect::Fade,
            progress: 1.0,
            color: theme::text(),
        }
    }

    /// Set the font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set the animation effect.
    pub fn effect(mut self, e: TextAnimEffect) -> Self {
        self.effect = e;
        self
    }

    /// Set the animation progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set the text color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }
}

impl View for TextAnimate {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TextAnimate");
        let p = self.progress;
        let alpha = match self.effect {
            TextAnimEffect::Fade => p,
            TextAnimEffect::Slide => {
                let offset_y = (1.0 - p) * 20.0;
                renderer.draw_text_raw(
                    &self.text,
                    rect.x,
                    rect.y + offset_y,
                    self.font_size,
                    theme::with_alpha(self.color, self.color[3] * p),
                );
                renderer.pop_vnode();
                return;
            }
            TextAnimEffect::Scale => p,
            TextAnimEffect::Blur => p,
        };
        renderer.draw_text_raw(
            &self.text,
            rect.x,
            rect.y,
            self.font_size,
            theme::with_alpha(self.color, self.color[3] * alpha),
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        Size {
            width: tw,
            height: th,
        }
    }
}

// ----------------------------------------------------------------------------
// TypewriterEffect -- typewriter-style character-by-character reveal
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct TypewriterEffect {
    /// Full text content.
    pub text: String,
    /// Number of characters revealed.
    pub revealed: usize,
    /// Font size.
    pub font_size: f32,
    /// Text color.
    pub color: [f32; 4],
    /// Whether to show a blinking cursor.
    pub show_cursor: bool,
    /// Cursor blink time.
    pub cursor_time: f32,
}

impl TypewriterEffect {
    /// Create a new TypewriterEffect.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            revealed: 0,
            font_size: 16.0,
            color: theme::text(),
            show_cursor: true,
            cursor_time: 0.0,
        }
    }

    /// Set the number of revealed characters.
    pub fn revealed(mut self, n: usize) -> Self {
        self.revealed = n;
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set the text color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set whether to show the cursor.
    pub fn cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }

    /// Set cursor blink time.
    pub fn cursor_time(mut self, t: f32) -> Self {
        self.cursor_time = t;
        self
    }
}

impl View for TypewriterEffect {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TypewriterEffect");
        let visible = &self.text[..self.revealed.min(self.text.len())];
        renderer.draw_text_raw(
            visible,
            rect.x,
            rect.y + self.font_size,
            self.font_size,
            self.color,
        );
        // Blinking cursor
        if self.show_cursor {
            let cursor_visible = ((self.cursor_time * 2.0) as u32).is_multiple_of(2);
            if cursor_visible {
                let (tw, _) = renderer.measure_text(visible, self.font_size);
                renderer.fill_rect(
                    Rect {
                        x: rect.x + tw + 2.0,
                        y: rect.y,
                        width: 2.0,
                        height: self.font_size + 4.0,
                    },
                    theme::accent(),
                );
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        Size {
            width: tw + 4.0,
            height: th + 4.0,
        }
    }
}

// ----------------------------------------------------------------------------
// NumberTicker -- animated number counter with rolling digits
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct NumberTicker {
    /// Current displayed value.
    pub value: f64,
    /// Font size.
    pub font_size: f32,
    /// Number of decimal places.
    pub decimals: usize,
    /// Text color.
    pub color: [f32; 4],
    /// Animation progress 0.0..1.0.
    pub progress: f32,
    /// Prefix string (e.g. "$").
    pub prefix: String,
    /// Suffix string (e.g. "%").
    pub suffix: String,
}

impl NumberTicker {
    /// Create a new NumberTicker.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            font_size: 32.0,
            decimals: 0,
            color: theme::text(),
            progress: 1.0,
            prefix: String::new(),
            suffix: String::new(),
        }
    }

    /// Set the font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set the number of decimal places.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }

    /// Set the text color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set the animation progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set the prefix.
    pub fn prefix(mut self, p: &str) -> Self {
        self.prefix = p.to_string();
        self
    }

    /// Set the suffix.
    pub fn suffix(mut self, s: &str) -> Self {
        self.suffix = s.to_string();
        self
    }
}

impl View for NumberTicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "NumberTicker");
        let display_val = self.value * self.progress as f64;
        let formatted = if self.decimals == 0 {
            format!("{}{:.0}{}", self.prefix, display_val, self.suffix)
        } else {
            format!(
                "{}{:.prec$}{}",
                self.prefix,
                display_val,
                self.suffix,
                prec = self.decimals
            )
        };
        renderer.draw_text_raw(
            &formatted,
            rect.x,
            rect.y + self.font_size,
            self.font_size,
            self.color,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let text = format!("{}{:.0}{}", self.prefix, self.value, self.suffix);
        let (tw, th) = renderer.measure_text(&text, self.font_size);
        Size {
            width: tw,
            height: th,
        }
    }
}

// =============================================================================
// CARD EFFECTS
// =============================================================================

/// CardStack -- stacked cards with depth and parallax.
#[derive(Clone)]
pub struct CardStack {
    /// Number of cards to stack.
    pub count: usize,
    /// Offset per card in pixels.
    pub offset_per_card: f32,
    /// Card width.
    pub card_width: f32,
    /// Card height.
    pub card_height: f32,
    /// Base card color.
    pub base_color: [f32; 4],
}

impl CardStack {
    /// Create a new CardStack.
    pub fn new() -> Self {
        Self {
            count: 3,
            offset_per_card: 12.0,
            card_width: 200.0,
            card_height: 120.0,
            base_color: theme::surface_elevated(),
        }
    }

    /// Set the number of cards.
    pub fn count(mut self, n: usize) -> Self {
        self.count = n;
        self
    }

    /// Set the offset per card.
    pub fn offset(mut self, o: f32) -> Self {
        self.offset_per_card = o;
        self
    }

    /// Set the card size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.card_width = w;
        self.card_height = h;
        self
    }

    /// Set the base color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.base_color = c;
        self
    }
}

impl Default for CardStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for CardStack {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "CardStack");
        for i in (0..self.count).rev() {
            let offset = i as f32 * self.offset_per_card;
            let card_rect = Rect {
                x: rect.x + offset,
                y: rect.y + offset,
                width: self.card_width,
                height: self.card_height,
            };
            let alpha = 1.0 - (i as f32 * 0.15);
            renderer.fill_rounded_rect(
                card_rect,
                RADIUS_XL,
                theme::with_alpha(self.base_color, self.base_color[3] * alpha),
            );
            renderer.stroke_rounded_rect(card_rect, RADIUS_XL, theme::border(), 1.0);
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.card_width + self.count as f32 * self.offset_per_card,
            height: self.card_height + self.count as f32 * self.offset_per_card,
        }
    }
}

// ----------------------------------------------------------------------------
// CardHoverEffect -- card with 3D tilt on hover
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct CardHoverEffect {
    /// Card width.
    pub width: f32,
    /// Card height.
    pub height: f32,
    /// Hover progress 0.0..1.0.
    pub hover: f32,
    /// Card background color.
    pub bg_color: [f32; 4],
    /// Card label text.
    pub label: String,
}

impl CardHoverEffect {
    /// Create a new CardHoverEffect.
    pub fn new() -> Self {
        Self {
            width: 220.0,
            height: 140.0,
            hover: 0.0,
            bg_color: theme::surface_elevated(),
            label: String::new(),
        }
    }

    /// Set the card size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set hover progress.
    pub fn hover(mut self, h: f32) -> Self {
        self.hover = h.clamp(0.0, 1.0);
        self
    }

    /// Set the background color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the label text.
    pub fn label(mut self, l: &str) -> Self {
        self.label = l.to_string();
        self
    }
}

impl Default for CardHoverEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl View for CardHoverEffect {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "CardHoverEffect");
        let tilt = self.hover * 8.0;
        let card_rect = Rect {
            x: rect.x + tilt,
            y: rect.y - tilt * 0.5,
            width: self.width,
            height: self.height,
        };
        // Shadow
        if self.hover > 0.0 {
            renderer.push_shadow(12.0 * self.hover, theme::shadow(), [tilt * 0.5, tilt * 0.5]);
        }
        renderer.fill_rounded_rect(card_rect, RADIUS_XL, self.bg_color);
        renderer.stroke_rounded_rect(card_rect, RADIUS_XL, theme::border(), 1.0);
        if self.hover > 0.0 {
            renderer.pop_shadow();
        }
        // Spotlight effect
        if self.hover > 0.0 {
            let spot_rect = Rect {
                x: card_rect.x + self.hover * 40.0,
                y: card_rect.y,
                width: self.width * 0.6,
                height: self.height,
            };
            let spot_color = theme::surface_overlay();
            renderer.fill_rounded_rect(
                spot_rect,
                RADIUS_XL,
                theme::with_alpha(spot_color, 0.03 * self.hover),
            );
        }
        // Label
        if !self.label.is_empty() {
            renderer.draw_text_raw(
                &self.label,
                card_rect.x + 16.0,
                card_rect.y + 28.0,
                16.0,
                theme::text(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width + 16.0,
            height: self.height + 16.0,
        }
    }
}

// ----------------------------------------------------------------------------
// ExpandableCard -- card that expands to reveal more content
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ExpandableCard {
    /// Card title.
    pub title: String,
    /// Summary text (always visible).
    pub summary: String,
    /// Expanded detail text.
    pub detail: String,
    /// Whether the card is expanded.
    pub expanded: bool,
    /// Animation progress 0.0..1.0.
    pub progress: f32,
    /// Card width.
    pub width: f32,
}

impl ExpandableCard {
    /// Create a new ExpandableCard.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            summary: String::new(),
            detail: String::new(),
            expanded: false,
            progress: 0.0,
            width: 300.0,
        }
    }

    /// Set the summary text.
    pub fn summary(mut self, s: &str) -> Self {
        self.summary = s.to_string();
        self
    }

    /// Set the detail text.
    pub fn detail(mut self, d: &str) -> Self {
        self.detail = d.to_string();
        self
    }

    /// Set expanded state.
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = e;
        self
    }

    /// Set animation progress.
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    /// Set the card width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl View for ExpandableCard {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ExpandableCard");
        let base_h = 64.0;
        let expanded_h = if self.expanded {
            let lines = (self.detail.len() as f32 * 8.0 / (self.width - 32.0))
                .ceil()
                .max(1.0);
            base_h + lines * 20.0 + 16.0
        } else {
            base_h
        };
        let h = base_h + (expanded_h - base_h) * self.progress;
        let card_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: h,
        };
        renderer.fill_rounded_rect(card_rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(card_rect, RADIUS_XL, theme::border(), 1.0);
        // Title
        renderer.draw_text_raw(
            &self.title,
            rect.x + 16.0,
            rect.y + 24.0,
            16.0,
            theme::text(),
        );
        // Chevron
        let ch_x = rect.x + self.width - 24.0;
        let ch_y = rect.y + 20.0;
        let chev_col = theme::text_muted();
        let rot = self.progress;
        if rot > 0.5 {
            renderer.draw_line(ch_x, ch_y + 4.0, ch_x + 5.0, ch_y, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y, ch_x + 10.0, ch_y + 4.0, chev_col, 2.0);
        } else {
            renderer.draw_line(ch_x, ch_y, ch_x + 5.0, ch_y + 4.0, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y + 4.0, ch_x + 10.0, ch_y, chev_col, 2.0);
        }
        // Summary
        if !self.summary.is_empty() {
            renderer.draw_text_raw(
                &self.summary,
                rect.x + 16.0,
                rect.y + 44.0,
                13.0,
                theme::text_muted(),
            );
        }
        // Detail (expanded)
        if self.progress > 0.0 && !self.detail.is_empty() {
            renderer.draw_text_raw(
                &self.detail,
                rect.x + 16.0,
                rect.y + 60.0,
                13.0,
                theme::with_alpha(theme::text(), self.progress),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let expanded_h = if self.expanded {
            let lines = (self.detail.len() as f32 * 8.0 / (self.width - 32.0))
                .ceil()
                .max(1.0);
            64.0 + lines * 20.0 + 16.0
        } else {
            64.0
        };
        Size {
            width: proposal.width.unwrap_or(self.width),
            height: 64.0 + (expanded_h - 64.0) * self.progress,
        }
    }
}

// ----------------------------------------------------------------------------
// DraggableCard -- card that can be dragged and dropped
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DraggableCard {
    /// Card title.
    pub title: String,
    /// Card description.
    pub description: String,
    /// Whether the card is currently being dragged.
    pub is_dragging: bool,
    /// Drag offset X.
    pub drag_x: f32,
    /// Drag offset Y.
    pub drag_y: f32,
    /// Card width.
    pub width: f32,
    /// Card height.
    pub height: f32,
}

impl DraggableCard {
    /// Create a new DraggableCard.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            is_dragging: false,
            drag_x: 0.0,
            drag_y: 0.0,
            width: 200.0,
            height: 100.0,
        }
    }

    /// Set the description.
    pub fn description(mut self, d: &str) -> Self {
        self.description = d.to_string();
        self
    }

    /// Set dragging state.
    pub fn dragging(mut self, d: bool) -> Self {
        self.is_dragging = d;
        self
    }

    /// Set drag offset.
    pub fn drag_offset(mut self, dx: f32, dy: f32) -> Self {
        self.drag_x = dx;
        self.drag_y = dy;
        self
    }

    /// Set the card size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
}

impl View for DraggableCard {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DraggableCard");
        let card_rect = Rect {
            x: rect.x + self.drag_x,
            y: rect.y + self.drag_y,
            width: self.width,
            height: self.height,
        };
        // Shadow when dragging
        if self.is_dragging {
            renderer.push_shadow(16.0, theme::shadow(), [0.0, 4.0]);
        }
        renderer.fill_rounded_rect(card_rect, RADIUS_XL, theme::surface_elevated());
        renderer.stroke_rounded_rect(card_rect, RADIUS_XL, theme::border(), 1.0);
        if self.is_dragging {
            renderer.pop_shadow();
        }
        // Drag handle indicator
        let handle_x = card_rect.x + 12.0;
        let handle_y = card_rect.y + 16.0;
        for row in 0..3 {
            renderer.fill_rounded_rect(
                Rect {
                    x: handle_x,
                    y: handle_y + row as f32 * 6.0,
                    width: 12.0,
                    height: 3.0,
                },
                1.5,
                theme::text_dim(),
            );
        }
        // Title
        renderer.draw_text_raw(
            &self.title,
            card_rect.x + 16.0,
            card_rect.y + 40.0,
            14.0,
            theme::text(),
        );
        // Description
        if !self.description.is_empty() {
            renderer.draw_text_raw(
                &self.description,
                card_rect.x + 16.0,
                card_rect.y + 58.0,
                12.0,
                theme::text_muted(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// =============================================================================
// BUTTON COMPONENTS
// =============================================================================

// ----------------------------------------------------------------------------
// ShimmerButton -- button with shimmer/sweep animation
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ShimmerButton {
    /// Button label.
    pub label: String,
    /// Button width.
    pub width: f32,
    /// Button height.
    pub height: f32,
    /// Shimmer animation time.
    pub time: f32,
    /// Whether the button is disabled.
    pub disabled: bool,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Text color.
    pub text_color: [f32; 4],
}

impl ShimmerButton {
    /// Create a new ShimmerButton.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            width: 160.0,
            height: 44.0,
            time: 0.0,
            disabled: false,
            bg_color: theme::accent(),
            text_color: theme::bg(),
        }
    }

    /// Set the button size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set the shimmer time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, c: [f32; 4]) -> Self {
        self.text_color = c;
        self
    }
}

impl View for ShimmerButton {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ShimmerButton");
        let bg = if self.disabled {
            theme::disabled()
        } else {
            self.bg_color
        };
        let tc = if self.disabled {
            theme::disabled_text()
        } else {
            self.text_color
        };
        renderer.fill_rounded_rect(rect, RADIUS_XL, bg);
        // Shimmer sweep
        if !self.disabled {
            let sweep_x = rect.x + ((self.time * 100.0) % (rect.width + 80.0)) - 40.0;
            let shimmer_rect = Rect {
                x: sweep_x,
                y: rect.y,
                width: 40.0,
                height: rect.height,
            };
            let shimmer_color = theme::accent();
            renderer.fill_rounded_rect(
                shimmer_rect,
                RADIUS_XL,
                theme::with_alpha(shimmer_color, 0.15),
            );
        }
        // Label
        let (tw, th) = renderer.measure_text(&self.label, 15.0);
        renderer.draw_text_raw(
            &self.label,
            rect.x + (self.width - tw) / 2.0,
            rect.y + (self.height - th) / 2.0,
            15.0,
            tc,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// RippleButton -- button with material ripple effect
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct RippleButton {
    /// Button label.
    pub label: String,
    /// Button width.
    pub width: f32,
    /// Button height.
    pub height: f32,
    /// Ripple progress 0.0..1.0 (0 = no ripple).
    pub ripple: f32,
    /// Ripple center X (relative to button).
    pub ripple_cx: f32,
    /// Ripple center Y (relative to button).
    pub ripple_cy: f32,
    /// Whether the button is disabled.
    pub disabled: bool,
    /// Background color.
    pub bg_color: [f32; 4],
    /// Text color.
    pub text_color: [f32; 4],
}

impl RippleButton {
    /// Create a new RippleButton.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            width: 160.0,
            height: 44.0,
            ripple: 0.0,
            ripple_cx: 0.5,
            ripple_cy: 0.5,
            disabled: false,
            bg_color: theme::accent(),
            text_color: theme::bg(),
        }
    }

    /// Set the button size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set ripple progress.
    pub fn ripple(mut self, r: f32) -> Self {
        self.ripple = r.clamp(0.0, 1.0);
        self
    }

    /// Set ripple center (0.0..1.0).
    pub fn ripple_center(mut self, cx: f32, cy: f32) -> Self {
        self.ripple_cx = cx;
        self.ripple_cy = cy;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    /// Set the background color.
    pub fn bg_color(mut self, c: [f32; 4]) -> Self {
        self.bg_color = c;
        self
    }

    /// Set the text color.
    pub fn text_color(mut self, c: [f32; 4]) -> Self {
        self.text_color = c;
        self
    }
}

impl View for RippleButton {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "RippleButton");
        let bg = if self.disabled {
            theme::disabled()
        } else {
            self.bg_color
        };
        let tc = if self.disabled {
            theme::disabled_text()
        } else {
            self.text_color
        };
        renderer.fill_rounded_rect(rect, RADIUS_XL, bg);
        // Ripple effect
        if self.ripple > 0.0 && !self.disabled {
            let max_r = (self.width * self.width + self.height * self.height).sqrt();
            let ripple_r = self.ripple * max_r;
            let cx = rect.x + self.ripple_cx * self.width;
            let cy = rect.y + self.ripple_cy * self.height;
            let ripple_alpha = (1.0 - self.ripple) * 0.3;
            let ripple_color = theme::accent();
            renderer.fill_ellipse(
                Rect {
                    x: cx - ripple_r,
                    y: cy - ripple_r,
                    width: ripple_r * 2.0,
                    height: ripple_r * 2.0,
                },
                theme::with_alpha(ripple_color, ripple_alpha),
            );
        }
        // Label
        let (tw, th) = renderer.measure_text(&self.label, 15.0);
        renderer.draw_text_raw(
            &self.label,
            rect.x + (self.width - tw) / 2.0,
            rect.y + (self.height - th) / 2.0,
            15.0,
            tc,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// StatefulButton -- button with loading/success/error states
// ----------------------------------------------------------------------------

/// Button state for StatefulButton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Clone)]
pub struct StatefulButton {
    /// Button label.
    pub label: String,
    /// Current state.
    pub state: ButtonState,
    /// Button width.
    pub width: f32,
    /// Button height.
    pub height: f32,
    /// Loading spinner rotation angle.
    pub spinner_angle: f32,
}

impl StatefulButton {
    /// Create a new StatefulButton.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            state: ButtonState::Idle,
            width: 160.0,
            height: 44.0,
            spinner_angle: 0.0,
        }
    }

    /// Set the button state.
    pub fn state(mut self, s: ButtonState) -> Self {
        self.state = s;
        self
    }

    /// Set the button size.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set the spinner angle.
    pub fn spinner_angle(mut self, a: f32) -> Self {
        self.spinner_angle = a;
        self
    }
}

impl View for StatefulButton {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "StatefulButton");
        let bg = match self.state {
            ButtonState::Idle => theme::accent(),
            ButtonState::Loading => theme::hover(),
            ButtonState::Success => theme::success(),
            ButtonState::Error => theme::error_color(),
        };
        renderer.fill_rounded_rect(rect, RADIUS_XL, bg);
        let tc = match self.state {
            ButtonState::Idle | ButtonState::Loading => theme::bg(),
            ButtonState::Success => theme::bg(),
            ButtonState::Error => theme::bg(),
        };
        match self.state {
            ButtonState::Loading => {
                // Spinner
                let cx = rect.x + self.width / 2.0;
                let cy = rect.y + self.height / 2.0;
                let r = 10.0;
                for i in 0..8 {
                    let angle = self.spinner_angle + i as f32 * 0.785;
                    let px = cx + angle.cos() * r;
                    let py = cy + angle.sin() * r;
                    let alpha = 0.2 + (i as f32 * 0.1);
                    renderer.fill_ellipse(
                        Rect {
                            x: px - 2.0,
                            y: py - 2.0,
                            width: 4.0,
                            height: 4.0,
                        },
                        theme::with_alpha(tc, alpha),
                    );
                }
            }
            ButtonState::Success => {
                // Checkmark
                let cx = rect.x + self.width / 2.0;
                let cy = rect.y + self.height / 2.0;
                renderer.draw_line(cx - 6.0, cy, cx - 2.0, cy + 5.0, tc, 2.5);
                renderer.draw_line(cx - 2.0, cy + 5.0, cx + 7.0, cy - 5.0, tc, 2.5);
            }
            ButtonState::Error => {
                // X mark
                let cx = rect.x + self.width / 2.0;
                let cy = rect.y + self.height / 2.0;
                renderer.draw_line(cx - 5.0, cy - 5.0, cx + 5.0, cy + 5.0, tc, 2.5);
                renderer.draw_line(cx + 5.0, cy - 5.0, cx - 5.0, cy + 5.0, tc, 2.5);
            }
            ButtonState::Idle => {
                let (tw, th) = renderer.measure_text(&self.label, 15.0);
                renderer.draw_text_raw(
                    &self.label,
                    rect.x + (self.width - tw) / 2.0,
                    rect.y + (self.height - th) / 2.0,
                    15.0,
                    tc,
                );
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

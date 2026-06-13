//! Form control components.
//!
//! Label — form label with required indicator.
//! DateTimePicker — combined date and time picker.
//! Link — tappable link with underline.
//! SearchField — search input with icon.
//! SearchSuggestions — search suggestion dropdown.
//! Tag — small dismissible tag/label.
//!
//! All components use cvkg theme system (theme::*) for full themability.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// ----------------------------------------------------------------------------
// Label — form label with required indicator
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Label {
    /// Label text.
    pub text: String,
    /// Font size.
    pub font_size: f32,
    /// Text color.
    pub color: [f32; 4],
    /// Whether the label is required (shows asterisk).
    pub required: bool,
}

impl Label {
    /// Create a new Label.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            font_size: 14.0,
            color: theme::text(),
            required: false,
        }
    }

    /// Set font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set text color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set required.
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
}

impl View for Label {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Label");
        let display = if self.required {
            format!("{} *", self.text)
        } else {
            self.text.clone()
        };
        renderer.draw_text(
            &display,
            rect.x,
            rect.y + self.font_size,
            self.font_size,
            self.color,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        Size {
            width: tw + if self.required { 12.0 } else { 0.0 },
            height: th,
        }
    }
}

// ----------------------------------------------------------------------------
// DateTimePicker — combined date and time picker
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct DateTimePicker {
    /// Selected hour (0-23).
    pub hour: u8,
    /// Selected minute (0-59).
    pub minute: u8,
    /// Selected day (1-31).
    pub day: u8,
    /// Selected month (1-12).
    pub month: u8,
    /// Selected year.
    pub year: u32,
    /// Whether the picker is open.
    pub open: bool,
    /// Picker width.
    pub width: f32,
}

impl DateTimePicker {
    /// Create a new DateTimePicker.
    pub fn new() -> Self {
        Self {
            hour: 12,
            minute: 0,
            day: 1,
            month: 1,
            year: 2024,
            open: false,
            width: 300.0,
        }
    }

    /// Set the time.
    pub fn time(mut self, hour: u8, minute: u8) -> Self {
        self.hour = hour % 24;
        self.minute = minute % 60;
        self
    }

    /// Set the date.
    pub fn date(mut self, day: u8, month: u8, year: u32) -> Self {
        self.day = day;
        self.month = month;
        self.year = year;
        self
    }

    /// Set open state.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Default for DateTimePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl View for DateTimePicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "DateTimePicker");
        renderer.fill_rounded_rect(rect, 8.0, theme::input_bg());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        let dt_str = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        );
        renderer.draw_text(&dt_str, rect.x + 12.0, rect.y + 20.0, 14.0, theme::text());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: 36.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Link — tappable link with underline
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Link {
    /// Link text.
    pub text: String,
    /// Font size.
    pub font_size: f32,
    /// Whether visited.
    pub visited: bool,
}

impl Link {
    /// Create a new Link.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            font_size: 14.0,
            visited: false,
        }
    }

    /// Set font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }

    /// Set visited state.
    pub fn visited(mut self, v: bool) -> Self {
        self.visited = v;
        self
    }
}

impl View for Link {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Link");
        let color = if self.visited {
            [0.5, 0.3, 0.8, 1.0]
        } else {
            theme::accent()
        };
        renderer.draw_text(
            &self.text,
            rect.x,
            rect.y + self.font_size,
            self.font_size,
            color,
        );
        let (tw, _) = renderer.measure_text(&self.text, self.font_size);
        renderer.draw_line(
            rect.x,
            rect.y + self.font_size + 2.0,
            rect.x + tw,
            rect.y + self.font_size + 2.0,
            color,
            1.0,
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        Size {
            width: tw,
            height: th + 4.0,
        }
    }
}

// ----------------------------------------------------------------------------
// SearchField — search input with icon
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SearchField {
    /// Current search text.
    pub text: String,
    /// Placeholder.
    pub placeholder: String,
    /// Whether focused.
    pub focused: bool,
    /// Field width.
    pub width: f32,
}

impl SearchField {
    /// Create a new SearchField.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: "Search...".to_string(),
            focused: false,
            width: 240.0,
        }
    }

    /// Set the text.
    pub fn text(mut self, t: &str) -> Self {
        self.text = t.to_string();
        self
    }

    /// Set the placeholder.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_string();
        self
    }

    /// Set focused.
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Default for SearchField {
    fn default() -> Self {
        Self::new()
    }
}

impl View for SearchField {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SearchField");
        renderer.fill_rounded_rect(rect, 8.0, theme::input_bg());
        let border = if self.focused {
            theme::accent()
        } else {
            theme::border()
        };
        renderer.stroke_rounded_rect(rect, 8.0, border, if self.focused { 2.0 } else { 1.0 });
        let ix = rect.x + 10.0;
        let iy = rect.y + 10.0;
        renderer.stroke_ellipse(
            Rect {
                x: ix,
                y: iy,
                width: 12.0,
                height: 12.0,
            },
            theme::text_muted(),
            1.5,
        );
        renderer.draw_line(
            ix + 9.0,
            iy + 9.0,
            ix + 14.0,
            iy + 14.0,
            theme::text_muted(),
            1.5,
        );
        let display = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };
        let color = if self.text.is_empty() {
            theme::text_muted()
        } else {
            theme::text()
        };
        renderer.draw_text(display, rect.x + 32.0, rect.y + 18.0, 14.0, color);
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: 32.0,
        }
    }
}

// ----------------------------------------------------------------------------
// SearchSuggestions — search suggestion dropdown
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SearchSuggestions {
    /// Suggestion items.
    pub items: Vec<String>,
    /// Selected index.
    pub selected: Option<usize>,
    /// Whether visible.
    pub visible: bool,
    /// Width.
    pub width: f32,
}

impl SearchSuggestions {
    /// Create a new SearchSuggestions.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            visible: true,
            width: 240.0,
        }
    }

    /// Add a suggestion.
    pub fn suggestion(mut self, text: &str) -> Self {
        self.items.push(text.to_string());
        self
    }

    /// Set selected index.
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Set visibility.
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }

    /// Set width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Default for SearchSuggestions {
    fn default() -> Self {
        Self::new()
    }
}

impl View for SearchSuggestions {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.visible || self.items.is_empty() {
            return;
        }
        renderer.push_vnode(rect, "SearchSuggestions");
        let item_h = 32.0;
        let total_h = self.items.len() as f32 * item_h;
        let bg_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: self.width,
            height: total_h,
        };
        renderer.fill_rounded_rect(bg_rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(bg_rect, 8.0, theme::border(), 1.0);
        for (i, item) in self.items.iter().enumerate() {
            let iy = rect.y + i as f32 * item_h;
            if self.selected == Some(i) {
                renderer.fill_rounded_rect(
                    Rect {
                        x: rect.x + 4.0,
                        y: iy + 2.0,
                        width: self.width - 8.0,
                        height: item_h - 4.0,
                    },
                    4.0,
                    theme::hover(),
                );
            }
            renderer.draw_text(item, rect.x + 12.0, iy + item_h * 0.6, 13.0, theme::text());
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        Size {
            width: self.width,
            height: self.items.len() as f32 * 32.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Tag — small dismissible tag/label
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Tag {
    /// Tag text.
    pub text: String,
    /// Tag color.
    pub color: [f32; 4],
    /// Whether dismissible.
    pub dismissible: bool,
    /// Font size.
    pub font_size: f32,
}

impl Tag {
    /// Create a new Tag.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            color: theme::accent(),
            dismissible: false,
            font_size: 12.0,
        }
    }

    /// Set the color.
    pub fn color(mut self, c: [f32; 4]) -> Self {
        self.color = c;
        self
    }

    /// Set dismissible.
    pub fn dismissible(mut self, d: bool) -> Self {
        self.dismissible = d;
        self
    }

    /// Set font size.
    pub fn font_size(mut self, s: f32) -> Self {
        self.font_size = s;
        self
    }
}

impl View for Tag {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Tag");
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        let pad_x = 10.0;
        let pad_y = 4.0;
        let tag_w = tw + pad_x * 2.0 + if self.dismissible { 16.0 } else { 0.0 };
        let tag_h = th + pad_y * 2.0;
        let tag_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: tag_w,
            height: tag_h,
        };
        renderer.fill_rounded_rect(
            tag_rect,
            tag_h / 2.0,
            [self.color[0], self.color[1], self.color[2], 0.15],
        );
        renderer.stroke_rounded_rect(tag_rect, tag_h / 2.0, self.color, 1.0);
        renderer.draw_text(
            &self.text,
            rect.x + pad_x,
            rect.y + pad_y + th * 0.2,
            self.font_size,
            theme::text(),
        );
        if self.dismissible {
            let cx = rect.x + tag_w - 12.0;
            let cy = rect.y + tag_h / 2.0;
            renderer.draw_line(
                cx - 3.0,
                cy - 3.0,
                cx + 3.0,
                cy + 3.0,
                theme::text_muted(),
                1.5,
            );
            renderer.draw_line(
                cx + 3.0,
                cy - 3.0,
                cx - 3.0,
                cy + 3.0,
                theme::text_muted(),
                1.5,
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, renderer: &mut dyn Renderer, _proposal: SizeProposal) -> Size {
        let (tw, th) = renderer.measure_text(&self.text, self.font_size);
        Size {
            width: tw + 20.0 + if self.dismissible { 16.0 } else { 0.0 },
            height: th + 8.0,
        }
    }
}

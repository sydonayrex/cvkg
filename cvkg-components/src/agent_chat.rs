//! Agent chat components: AgentChat, MessageList, InputBar, UserMessage,
//! AssistantMessage, Markdown, ToolCard, SuggestionChips, ModelPicker,
//! CopyToolbar, TextShimmer.
//!
//! All components use the cvkg theme system (theme::* helpers) for full
//! themability via cvkg-themes.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, Size, SizeProposal, View};

// ----------------------------------------------------------------------------
// TextShimmer -- animated shimmer loading placeholder for streaming text
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct TextShimmer {
    /// Width of the shimmer area.
    pub width: f32,
    /// Height of the shimmer area.
    pub height: f32,
    /// Animation time offset for the shimmer sweep.
    pub time: f32,
}

impl TextShimmer {
    /// Create a new TextShimmer with default 200x16 size.
    pub fn new() -> Self {
        Self {
            width: 200.0,
            height: 16.0,
            time: 0.0,
        }
    }

    /// Set the width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set the height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Set the animation time.
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }
}

impl Default for TextShimmer {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TextShimmer {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TextShimmer");
        // Base skeleton bar
        renderer.fill_rounded_rect(rect, 4.0, theme::skeleton_base());
        // Shimmer highlight sweeping across
        let sweep_x = rect.x + ((self.time * 120.0) % (rect.width + 60.0)) - 30.0;
        let shimmer_rect = Rect {
            x: sweep_x,
            y: rect.y,
            width: 60.0,
            height: rect.height,
        };
        renderer.fill_rounded_rect(shimmer_rect, 4.0, theme::skeleton_highlight());
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(self.width),
            height: self.height,
        }
    }
}

// ----------------------------------------------------------------------------
// UserMessage -- styled user message bubble with avatar
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct UserMessage {
    /// Message text content.
    pub text: String,
    /// Optional avatar label (single char or initials).
    pub avatar: String,
}

impl UserMessage {
    /// Create a new UserMessage.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            avatar: "U".to_string(),
        }
    }

    /// Set the avatar label.
    pub fn avatar(mut self, label: &str) -> Self {
        self.avatar = label.to_string();
        self
    }
}

impl View for UserMessage {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "UserMessage");
        // Bubble background
        renderer.fill_rounded_rect(rect, 12.0, theme::chat_bubble_user());
        // Avatar circle
        let avatar_size = 28.0;
        let avatar_rect = Rect {
            x: rect.x + 12.0,
            y: rect.y + 12.0,
            width: avatar_size,
            height: avatar_size,
        };
        renderer.fill_ellipse(avatar_rect, theme::accent());
        renderer.draw_text(
            &self.avatar,
            avatar_rect.x + 8.0,
            avatar_rect.y + 20.0,
            12.0,
            theme::chat_text_user(),
        );
        // Message text
        renderer.draw_text(
            &self.text,
            rect.x + 52.0,
            rect.y + 20.0,
            15.0,
            theme::chat_text_user(),
        );
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(400.0);
        let lines = (self.text.len() as f32 * 8.5 / (width - 70.0))
            .ceil()
            .max(1.0);
        Size {
            width,
            height: 44.0 + lines * 20.0 + 12.0,
        }
    }
}

// ----------------------------------------------------------------------------
// AssistantMessage -- styled assistant message with avatar and actions
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct AssistantMessage {
    /// Message text content.
    pub text: String,
    /// Optional avatar label.
    pub avatar: String,
    /// Whether to show action buttons (copy, retry).
    pub show_actions: bool,
}

impl AssistantMessage {
    /// Create a new AssistantMessage.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            avatar: "AI".to_string(),
            show_actions: true,
        }
    }

    /// Set the avatar label.
    pub fn avatar(mut self, label: &str) -> Self {
        self.avatar = label.to_string();
        self
    }

    /// Enable or disable action buttons.
    pub fn actions(mut self, show: bool) -> Self {
        self.show_actions = show;
        self
    }
}

impl View for AssistantMessage {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AssistantMessage");
        // Bubble background
        renderer.fill_rounded_rect(rect, 12.0, theme::chat_bubble_assistant());
        renderer.stroke_rounded_rect(rect, 12.0, theme::border(), 1.0);
        // Avatar circle
        let avatar_size = 28.0;
        let avatar_rect = Rect {
            x: rect.x + 12.0,
            y: rect.y + 12.0,
            width: avatar_size,
            height: avatar_size,
        };
        renderer.fill_ellipse(avatar_rect, theme::surface_elevated());
        renderer.draw_text(
            &self.avatar,
            avatar_rect.x + 6.0,
            avatar_rect.y + 20.0,
            10.0,
            theme::text(),
        );
        // Message text
        renderer.draw_text(
            &self.text,
            rect.x + 52.0,
            rect.y + 20.0,
            15.0,
            theme::chat_text_assistant(),
        );
        // Action buttons row
        if self.show_actions {
            let actions_y = rect.y + rect.height - 28.0;
            renderer.draw_text(
                "Copy",
                rect.x + 16.0,
                actions_y + 14.0,
                12.0,
                theme::text_muted(),
            );
            renderer.draw_text(
                "Retry",
                rect.x + 60.0,
                actions_y + 14.0,
                12.0,
                theme::text_muted(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let width = proposal.width.unwrap_or(400.0);
        let lines = (self.text.len() as f32 * 8.5 / (width - 70.0))
            .ceil()
            .max(1.0);
        let actions_h = if self.show_actions { 32.0 } else { 0.0 };
        Size {
            width,
            height: 44.0 + lines * 20.0 + 12.0 + actions_h,
        }
    }
}

// ----------------------------------------------------------------------------
// MessageList -- scrollable list of chat messages
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct MessageList {
    /// List of message texts with role tags: ("user"|"assistant", text).
    pub messages: Vec<(String, String)>,
    /// Scroll offset from the bottom (0 = fully scrolled down).
    pub scroll_offset: f32,
}

impl MessageList {
    /// Create a new empty MessageList.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0.0,
        }
    }

    /// Add a message.
    pub fn message(mut self, role: &str, text: &str) -> Self {
        self.messages.push((role.to_string(), text.to_string()));
        self
    }

    /// Set the scroll offset.
    pub fn scroll(mut self, offset: f32) -> Self {
        self.scroll_offset = offset;
        self
    }
}

impl Default for MessageList {
    fn default() -> Self {
        Self::new()
    }
}

impl View for MessageList {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "MessageList");
        // Background
        renderer.fill_rect(rect, theme::bg());
        let mut y = rect.y + rect.height - 8.0;
        for (role, text) in self.messages.iter().rev() {
            let msg_rect = Rect {
                x: rect.x + 8.0,
                y: y - 60.0,
                width: rect.width - 16.0,
                height: 60.0,
            };
            if role == "user" {
                UserMessage::new(text).render(renderer, msg_rect);
            } else {
                AssistantMessage::new(text).render(renderer, msg_rect);
            }
            y -= 68.0;
            if y < rect.y {
                break;
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: (self.messages.len() as f32 * 68.0).max(200.0),
        }
    }
}

// ----------------------------------------------------------------------------
// InputBar -- text input with send/stop button and character count
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct InputBar {
    /// Current input text.
    pub text: String,
    /// Placeholder text when empty.
    pub placeholder: String,
    /// Whether the input is in "streaming" state (shows stop button).
    pub is_streaming: bool,
    /// Maximum character count (0 = unlimited).
    pub max_chars: usize,
}

impl InputBar {
    /// Create a new InputBar.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: "Send a message...".to_string(),
            is_streaming: false,
            max_chars: 0,
        }
    }

    /// Set the input text.
    pub fn text(mut self, t: &str) -> Self {
        self.text = t.to_string();
        self
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_string();
        self
    }

    /// Set streaming state.
    pub fn streaming(mut self, s: bool) -> Self {
        self.is_streaming = s;
        self
    }

    /// Set max character count.
    pub fn max_chars(mut self, m: usize) -> Self {
        self.max_chars = m;
        self
    }
}

impl Default for InputBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for InputBar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "InputBar");
        // Input background
        renderer.fill_rounded_rect(rect, 12.0, theme::input_bg());
        renderer.stroke_rounded_rect(rect, 12.0, theme::border(), 1.0);
        // Text or placeholder
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };
        let text_color = if self.text.is_empty() {
            theme::text_muted()
        } else {
            theme::text()
        };
        renderer.draw_text(display_text, rect.x + 16.0, rect.y + 20.0, 15.0, text_color);
        // Character count
        if self.max_chars > 0 {
            let count_str = format!("{}/{}", self.text.len(), self.max_chars);
            let (cw, _) = renderer.measure_text(&count_str, 11.0);
            renderer.draw_text(
                &count_str,
                rect.x + rect.width - cw - 56.0,
                rect.y + 22.0,
                11.0,
                theme::text_dim(),
            );
        }
        // Send / Stop button
        let btn_size = 32.0;
        let btn_rect = Rect {
            x: rect.x + rect.width - btn_size - 12.0,
            y: rect.y + (rect.height - btn_size) / 2.0,
            width: btn_size,
            height: btn_size,
        };
        if self.is_streaming {
            // Stop button (red square)
            renderer.fill_rounded_rect(btn_rect, 4.0, theme::error_color());
        } else {
            // Send button (accent circle)
            renderer.fill_rounded_rect(btn_rect, 8.0, theme::accent());
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(500.0),
            height: 56.0,
        }
    }
}

// ----------------------------------------------------------------------------
// SuggestionChips -- quick-action suggestion chips below input
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct SuggestionChips {
    /// Chip labels.
    pub chips: Vec<String>,
    /// Currently selected chip index (None = none selected).
    pub selected: Option<usize>,
}

impl SuggestionChips {
    /// Create a new SuggestionChips.
    pub fn new() -> Self {
        Self {
            chips: Vec::new(),
            selected: None,
        }
    }

    /// Add a chip.
    pub fn chip(mut self, label: &str) -> Self {
        self.chips.push(label.to_string());
        self
    }

    /// Set the selected index.
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }
}

impl Default for SuggestionChips {
    fn default() -> Self {
        Self::new()
    }
}

impl View for SuggestionChips {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "SuggestionChips");
        let mut x = rect.x;
        let chip_h = 28.0;
        let chip_gap = 8.0;
        for (i, label) in self.chips.iter().enumerate() {
            let (tw, _) = renderer.measure_text(label, 13.0);
            let chip_w = tw + 24.0;
            let chip_rect = Rect {
                x,
                y: rect.y,
                width: chip_w,
                height: chip_h,
            };
            let bg = if self.selected == Some(i) {
                theme::accent()
            } else {
                theme::surface_elevated()
            };
            let border = if self.selected == Some(i) {
                theme::accent()
            } else {
                theme::border()
            };
            let text_col = if self.selected == Some(i) {
                theme::bg()
            } else {
                theme::text()
            };
            renderer.fill_rounded_rect(chip_rect, 14.0, bg);
            renderer.stroke_rounded_rect(chip_rect, 14.0, border, 1.0);
            renderer.draw_text(label, x + 12.0, rect.y + 19.0, 13.0, text_col);
            x += chip_w + chip_gap;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let total_w: f32 = self
            .chips
            .iter()
            .map(|c| c.len() as f32 * 8.0 + 24.0 + 8.0)
            .sum();
        Size {
            width: proposal.width.unwrap_or(total_w.max(200.0)),
            height: 28.0,
        }
    }
}

// ----------------------------------------------------------------------------
// ModelPicker -- dropdown to select AI model
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ModelPicker {
    /// Available model names.
    pub models: Vec<String>,
    /// Currently selected model index.
    pub selected: usize,
    /// Whether the dropdown is open.
    pub is_open: bool,
}

impl ModelPicker {
    /// Create a new ModelPicker with default models.
    pub fn new() -> Self {
        Self {
            models: vec![
                "GPT-4o".to_string(),
                "Claude 3.5".to_string(),
                "Gemini Pro".to_string(),
            ],
            selected: 0,
            is_open: false,
        }
    }

    /// Set the available models.
    pub fn models(mut self, m: Vec<String>) -> Self {
        self.models = m;
        self
    }

    /// Set the selected index.
    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx;
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }
}

impl Default for ModelPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ModelPicker {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ModelPicker");
        // Trigger button
        renderer.fill_rounded_rect(rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        let label = self
            .models
            .get(self.selected)
            .map(|s| s.as_str())
            .unwrap_or("Select Model");
        renderer.draw_text(label, rect.x + 12.0, rect.y + 18.0, 14.0, theme::text());
        // Chevron
        let ch_x = rect.x + rect.width - 20.0;
        let ch_y = rect.y + 14.0;
        let chev_col = theme::text_muted();
        if self.is_open {
            renderer.draw_line(ch_x, ch_y + 4.0, ch_x + 5.0, ch_y, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y, ch_x + 10.0, ch_y + 4.0, chev_col, 2.0);
        } else {
            renderer.draw_line(ch_x, ch_y, ch_x + 5.0, ch_y + 4.0, chev_col, 2.0);
            renderer.draw_line(ch_x + 5.0, ch_y + 4.0, ch_x + 10.0, ch_y, chev_col, 2.0);
        }
        // Dropdown list
        if self.is_open {
            let item_h = 32.0;
            for (i, model) in self.models.iter().enumerate() {
                let item_rect = Rect {
                    x: rect.x,
                    y: rect.y + rect.height + i as f32 * item_h,
                    width: rect.width,
                    height: item_h,
                };
                let bg = if i == self.selected {
                    theme::hover()
                } else {
                    theme::surface_elevated()
                };
                renderer.fill_rect(item_rect, bg);
                renderer.draw_text(
                    model,
                    item_rect.x + 12.0,
                    item_rect.y + 20.0,
                    14.0,
                    theme::text(),
                );
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let extra_h = if self.is_open {
            self.models.len() as f32 * 32.0
        } else {
            0.0
        };
        Size {
            width: proposal.width.unwrap_or(200.0),
            height: 36.0 + extra_h,
        }
    }
}

// ----------------------------------------------------------------------------
// CopyToolbar -- floating toolbar with copy, share, export actions
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct CopyToolbar {
    /// Whether the toolbar is visible.
    pub visible: bool,
    /// Action labels to show.
    pub actions: Vec<String>,
}

impl CopyToolbar {
    /// Create a new CopyToolbar with default actions.
    pub fn new() -> Self {
        Self {
            visible: true,
            actions: vec![
                "Copy".to_string(),
                "Share".to_string(),
                "Export".to_string(),
            ],
        }
    }

    /// Set visibility.
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }

    /// Set action labels.
    pub fn actions(mut self, a: Vec<String>) -> Self {
        self.actions = a;
        self
    }
}

impl Default for CopyToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for CopyToolbar {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if !self.visible {
            return;
        }
        renderer.push_vnode(rect, "CopyToolbar");
        // Toolbar background
        renderer.fill_rounded_rect(rect, 8.0, theme::surface_overlay());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        renderer.push_shadow(8.0, theme::shadow(), [0.0, 2.0]);
        renderer.pop_shadow();
        let mut x = rect.x + 8.0;
        for action in &self.actions {
            let (tw, _) = renderer.measure_text(action, 13.0);
            renderer.draw_text(action, x, rect.y + 14.0, 13.0, theme::text());
            x += tw + 20.0;
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let total_w: f32 = self
            .actions
            .iter()
            .map(|a| a.len() as f32 * 8.0 + 20.0)
            .sum();
        Size {
            width: proposal.width.unwrap_or(total_w.max(120.0)),
            height: 28.0,
        }
    }
}

// ----------------------------------------------------------------------------
// ToolCard -- display tool call with name, args, status, result
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct ToolCard {
    /// Tool name.
    pub name: String,
    /// Short description of args.
    pub args_summary: String,
    /// Status: "running" | "done" | "error".
    pub status: String,
    /// Result summary text.
    pub result: String,
    /// Whether the card is expanded.
    pub expanded: bool,
}

impl ToolCard {
    /// Create a new ToolCard.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            args_summary: String::new(),
            status: "running".to_string(),
            result: String::new(),
            expanded: false,
        }
    }

    /// Set the args summary.
    pub fn args(mut self, a: &str) -> Self {
        self.args_summary = a.to_string();
        self
    }

    /// Set the status.
    pub fn status(mut self, s: &str) -> Self {
        self.status = s.to_string();
        self
    }

    /// Set the result text.
    pub fn result(mut self, r: &str) -> Self {
        self.result = r.to_string();
        self
    }

    /// Set expanded state.
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = e;
        self
    }
}

impl View for ToolCard {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "ToolCard");
        // Card background
        renderer.fill_rounded_rect(rect, 8.0, theme::surface_elevated());
        renderer.stroke_rounded_rect(rect, 8.0, theme::border(), 1.0);
        // Status indicator dot
        let dot_color = match self.status.as_str() {
            "done" => theme::success(),
            "error" => theme::error_color(),
            _ => theme::warning(),
        };
        renderer.fill_ellipse(
            Rect {
                x: rect.x + 12.0,
                y: rect.y + 14.0,
                width: 8.0,
                height: 8.0,
            },
            dot_color,
        );
        // Tool name
        renderer.draw_text(
            &self.name,
            rect.x + 28.0,
            rect.y + 20.0,
            14.0,
            theme::text(),
        );
        // Args summary
        if !self.args_summary.is_empty() {
            renderer.draw_text(
                &self.args_summary,
                rect.x + 16.0,
                rect.y + 40.0,
                12.0,
                theme::text_muted(),
            );
        }
        // Result (when expanded)
        if self.expanded && !self.result.is_empty() {
            renderer.draw_text(
                &self.result,
                rect.x + 16.0,
                rect.y + 58.0,
                12.0,
                theme::text_dim(),
            );
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let h = if self.expanded && !self.result.is_empty() {
            76.0
        } else if !self.args_summary.is_empty() {
            56.0
        } else {
            40.0
        };
        Size {
            width: proposal.width.unwrap_or(300.0),
            height: h,
        }
    }
}

// ----------------------------------------------------------------------------
// Markdown -- render markdown with code highlighting, tables, links
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Markdown {
    /// Markdown source text.
    pub source: String,
}

impl Markdown {
    /// Create a new Markdown component.
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }
}

impl View for Markdown {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Markdown");
        let mut y = rect.y + 4.0;
        for line in self.source.lines() {
            if let Some(rest) = line.strip_prefix("# ") {
                renderer.draw_text(rest, rect.x + 8.0, y + 24.0, 24.0, theme::text());
                y += 32.0;
            } else if let Some(rest) = line.strip_prefix("## ") {
                renderer.draw_text(rest, rect.x + 8.0, y + 20.0, 20.0, theme::text());
                y += 28.0;
            } else if line.starts_with("```") {
                continue;
            } else if let Some(rest) = line.strip_prefix("- ") {
                renderer.draw_text(
                    &format!("  {}", rest),
                    rect.x + 8.0,
                    y + 16.0,
                    14.0,
                    theme::text(),
                );
                y += 22.0;
            } else if line.starts_with("[") && line.contains("](") {
                // Link
                if let Some(bracket_end) = line.find(']') {
                    let link_text = &line[1..bracket_end];
                    renderer.draw_text(link_text, rect.x + 8.0, y + 16.0, 14.0, theme::accent());
                    y += 22.0;
                } else {
                    renderer.draw_text(line, rect.x + 8.0, y + 16.0, 14.0, theme::text());
                    y += 22.0;
                }
            } else if !line.trim().is_empty() {
                renderer.draw_text(line, rect.x + 8.0, y + 16.0, 14.0, theme::text());
                y += 22.0;
            } else {
                y += 8.0;
            }
            if y > rect.y + rect.height {
                break;
            }
        }
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let line_count = self.source.lines().count().max(1);
        Size {
            width: proposal.width.unwrap_or(400.0),
            height: line_count as f32 * 22.0 + 16.0,
        }
    }
}

// ----------------------------------------------------------------------------
// AgentChat -- full chat surface composing MessageList + InputBar + Suggestions
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct AgentChat {
    /// Messages as (role, text) pairs.
    pub messages: Vec<(String, String)>,
    /// Input text.
    pub input_text: String,
    /// Whether streaming.
    pub is_streaming: bool,
    /// Suggestion chips.
    pub suggestions: Vec<String>,
    /// Selected model index.
    pub model_index: usize,
}

impl AgentChat {
    /// Create a new AgentChat.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            is_streaming: false,
            suggestions: vec![
                "Summarize".to_string(),
                "Explain".to_string(),
                "Refactor".to_string(),
            ],
            model_index: 0,
        }
    }

    /// Add a message.
    pub fn message(mut self, role: &str, text: &str) -> Self {
        self.messages.push((role.to_string(), text.to_string()));
        self
    }

    /// Set input text.
    pub fn input(mut self, text: &str) -> Self {
        self.input_text = text.to_string();
        self
    }

    /// Set streaming state.
    pub fn streaming(mut self, s: bool) -> Self {
        self.is_streaming = s;
        self
    }

    /// Set suggestion chips.
    pub fn suggestions(mut self, s: Vec<String>) -> Self {
        self.suggestions = s;
        self
    }

    /// Set model index.
    pub fn model(mut self, idx: usize) -> Self {
        self.model_index = idx;
        self
    }
}

impl Default for AgentChat {
    fn default() -> Self {
        Self::new()
    }
}

impl View for AgentChat {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "AgentChat");
        // Background
        renderer.fill_rect(rect, theme::bg());
        // Layout: model picker at top, messages in middle, input at bottom
        let picker_h = 36.0;
        let input_h = 56.0;
        let suggestions_h = if self.suggestions.is_empty() {
            0.0
        } else {
            36.0
        };
        let msg_area = Rect {
            x: rect.x,
            y: rect.y + picker_h,
            width: rect.width,
            height: rect.height - picker_h - input_h - suggestions_h,
        };
        // Model picker
        let picker_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: picker_h,
        };
        ModelPicker::new()
            .selected(self.model_index)
            .render(renderer, picker_rect);
        // Message list
        let mut msg_list = MessageList::new();
        for (role, text) in &self.messages {
            msg_list = msg_list.message(role, text);
        }
        msg_list.render(renderer, msg_area);
        // Suggestions
        if !self.suggestions.is_empty() {
            let sug_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height - input_h - suggestions_h,
                width: rect.width,
                height: suggestions_h,
            };
            let mut chips = SuggestionChips::new();
            for s in &self.suggestions {
                chips = chips.chip(s);
            }
            chips.render(renderer, sug_rect);
        }
        // Input bar
        let input_rect = Rect {
            x: rect.x,
            y: rect.y + rect.height - input_h,
            width: rect.width,
            height: input_h,
        };
        InputBar::new()
            .text(&self.input_text)
            .streaming(self.is_streaming)
            .render(renderer, input_rect);
        renderer.pop_vnode();
    }
    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(500.0),
            height: 400.0,
        }
    }
}

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};

/// A single chat message with role, content, timestamp, and optional code blocks.
#[derive(Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: String,
    pub is_streaming: bool,
}

#[derive(Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: String::new(),
            is_streaming: false,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            timestamp: String::new(),
            is_streaming: false,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            timestamp: String::new(),
            is_streaming: false,
        }
    }

    pub fn timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = ts.into();
        self
    }

    pub fn streaming(mut self, streaming: bool) -> Self {
        self.is_streaming = streaming;
        self
    }
}

/// A chat block component with message history, streaming support, code blocks, and timestamps.
#[doc(alias = "Messenger")]
pub struct RavenMessenger {
    pub messages: Vec<ChatMessage>,
    pub is_huginn: bool, // Top position flag for backward compat
    pub duration: f32,
}

impl RavenMessenger {
    /// Create a new chat block with messages.
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            is_huginn: true,
            duration: 3.0,
        }
    }

    /// Push a new message (supports mid-stream appends).
    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }

    /// Backward compat: top notification style.
    pub fn top(msg: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::system(msg)],
            is_huginn: true,
            duration: 3.0,
        }
    }

    /// Backward compat: bottom notification style.
    pub fn bottom(msg: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::system(msg)],
            is_huginn: false,
            duration: 3.0,
        }
    }
}

impl View for RavenMessenger {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        if self.messages.is_empty() {
            return;
        }

        renderer.push_vnode(rect, "RavenMessenger");

        let t = renderer.elapsed_time();
        let padding = 12.0;
        let msg_spacing = 8.0;
        let header_h = 20.0;
        let _footer_h = 16.0;
        let code_block_padding = 8.0;
        let bubble_radius = 8.0;

        let input_area_h: f32 = 40.0;
        let mut current_y = rect.y + padding;

        // ── Render each message ──
        // Collect non-streaming messages for up-front rendering, then streaming ones at bottom
        let mut rendered_count = 0;
        for msg in &self.messages {
            if msg.content.is_empty() && !msg.is_streaming {
                continue;
            }

            let is_user = msg.role == ChatRole::User;
            let is_system = msg.role == ChatRole::System;

            // Role icon and label
            let (icon, role_label, bubble_color, text_color, align_right) = if is_user {
                (
                    'ᚢ',
                    "You",
                    theme::chat_bubble_user(),
                    [1.0, 1.0, 1.0, 0.95],
                    true,
                )
            } else if is_system {
                (
                    'ᛊ',
                    "System",
                    [0.1, 0.1, 0.15, 0.7],
                    [0.7, 0.7, 0.8, 0.9],
                    false,
                )
            } else {
                (
                    'ᚻ',
                    "Assistant",
                    [0.05, 0.08, 0.15, 0.85],
                    [0.9, 0.95, 1.0, 0.95],
                    false,
                )
            };

            let max_bubble_w = rect.width * 0.75;
            let bubble_x = if align_right {
                rect.x + rect.width - max_bubble_w - padding
            } else {
                rect.x + padding
            };

            // ── Role header (icon + label + timestamp) ──
            if !is_system {
                let header_color = theme::accent();
                let header_text = format!("{} {}", icon, role_label);
                renderer.draw_text_raw(&header_text, bubble_x, current_y, 11.0, header_color);

                if !msg.timestamp.is_empty() {
                    renderer.draw_text_raw(
                        &msg.timestamp,
                        bubble_x + max_bubble_w - 60.0,
                        current_y,
                        9.0,
                        [0.4, 0.4, 0.5, 0.7],
                    );
                }
                current_y += header_h;
            }

            // ── Parse content for code blocks ──
            let segments = parse_code_blocks(&msg.content);
            let mut seg_y = current_y;

            for segment in &segments {
                match segment {
                    ContentSegment::Text(text) => {
                        // Estimate text height (approx: 14px per line, ~60 chars per line)
                        let chars_per_line = ((max_bubble_w - 16.0) / 7.0) as usize;
                        let line_count = text.len().max(1) / chars_per_line.max(1) + 1;
                        let text_h = line_count as f32 * 16.0 + 8.0;
                        let text_rect = Rect {
                            x: bubble_x,
                            y: seg_y,
                            width: max_bubble_w,
                            height: text_h,
                        };
                        renderer.fill_rounded_rect(text_rect, bubble_radius, bubble_color);
                        if msg.is_streaming {
                            let pulse = (t * 4.0).sin() * 0.15 + 0.85;
                            renderer.stroke_rounded_rect(
                                text_rect,
                                bubble_radius,
                                [0.0, 1.0, 1.0, pulse],
                                1.0,
                            );
                        }
                        renderer.draw_text_raw(
                            text,
                            text_rect.x + 8.0,
                            text_rect.y + 4.0,
                            13.0,
                            text_color,
                        );
                        seg_y += text_h + 2.0;
                    }
                    ContentSegment::CodeBlock(code, lang) => {
                        let lines: Vec<&str> = code.lines().collect();
                        let code_h = lines.len() as f32 * 15.0 + code_block_padding * 2.0;
                        let code_w = max_bubble_w;
                        let code_rect = Rect {
                            x: bubble_x,
                            y: seg_y,
                            width: code_w,
                            height: code_h,
                        };
                        // Dark code background
                        renderer.fill_rounded_rect(
                            code_rect,
                            4.0,
                            theme::with_alpha(theme::surface_elevated(), 0.95),
                        );
                        renderer.stroke_rounded_rect(code_rect, 4.0, theme::border(), 1.0);

                        // Language label
                        if let Some(l) = lang {
                            renderer.draw_text_raw(
                                l,
                                code_rect.x + 8.0,
                                code_rect.y + 2.0,
                                9.0,
                                [0.3, 0.5, 0.8, 0.6],
                            );
                        }

                        // Code lines with line numbers
                        for (line_idx, line) in lines.iter().enumerate() {
                            let line_y =
                                code_rect.y + code_block_padding + 4.0 + line_idx as f32 * 15.0;
                            // Line number (dim)
                            let line_num = format!("{:>3} ", line_idx + 1);
                            renderer.draw_text_raw(
                                &line_num,
                                code_rect.x + 4.0,
                                line_y,
                                10.0,
                                [0.25, 0.25, 0.35, 0.6],
                            );
                            // Code text
                            renderer.draw_text_raw(
                                line,
                                code_rect.x + 34.0,
                                line_y,
                                11.0,
                                [0.8, 0.9, 0.7, 0.9],
                            );
                        }
                        seg_y += code_h + 2.0;
                    }
                }
            }

            // ── Streaming cursor indicator ──
            if msg.is_streaming {
                let cursor_alpha = (t * 3.0).sin() * 0.5 + 0.5;
                renderer.fill_rounded_rect(
                    Rect {
                        x: bubble_x + 4.0,
                        y: seg_y + 2.0,
                        width: 8.0,
                        height: 14.0,
                    },
                    2.0,
                    [0.0, 1.0, 1.0, cursor_alpha],
                );
                seg_y += 20.0;
            }

            current_y = seg_y + msg_spacing;
            rendered_count += 1;

            // Don't overflow the available rect
            if current_y > rect.y + rect.height - input_area_h {
                break;
            }
            let _ = rendered_count;
        }

        renderer.pop_vnode();
    }
}

/// Content segment for chat message rendering.
enum ContentSegment {
    Text(String),
    CodeBlock(String, Option<String>),
}

/// Parse markdown-style code blocks from message content.
/// Supports ```language ... ``` fenced code blocks.
fn parse_code_blocks(content: &str) -> Vec<ContentSegment> {
    let mut segments = Vec::new();
    let mut remaining = content;

    while let Some(start_idx) = remaining.find("```") {
        // Text before the code block
        if start_idx > 0 {
            let text = &remaining[..start_idx];
            if !text.trim().is_empty() {
                segments.push(ContentSegment::Text(text.to_string()));
            }
        }

        // Find the closing ```
        let after_fence = &remaining[start_idx + 3..];
        let mut lang = None;

        // Extract language from the first line after ```
        if let Some(newline_pos) = after_fence.find('\n') {
            let first_line = &after_fence[..newline_pos];
            let trimmed = first_line.trim();
            if !trimmed.is_empty() {
                lang = Some(trimmed.to_string());
            }
            let code_start = start_idx + 3 + newline_pos + 1;

            // Find closing ```
            if let Some(end_pos) = remaining[code_start..].find("```") {
                let code = &remaining[code_start..code_start + end_pos];
                segments.push(ContentSegment::CodeBlock(code.trim_end().to_string(), lang));
                remaining = &remaining[code_start + end_pos + 3..];
            } else {
                // Unclosed code block - render rest as code
                segments.push(ContentSegment::CodeBlock(
                    after_fence[newline_pos + 1..].to_string(),
                    lang,
                ));
                break;
            }
        } else {
            // Single-line ```...```
            if let Some(end_pos) = after_fence.find("```") {
                let code = &after_fence[..end_pos];
                segments.push(ContentSegment::CodeBlock(code.to_string(), lang));
                remaining = &remaining[start_idx + 3 + end_pos + 3..];
            } else {
                segments.push(ContentSegment::Text(remaining.to_string()));
                break;
            }
        }
    }

    // Remaining text after last code block
    if !remaining.trim().is_empty() {
        segments.push(ContentSegment::Text(remaining.to_string()));
    }

    if segments.is_empty() {
        segments.push(ContentSegment::Text(content.to_string()));
    }

    segments
}

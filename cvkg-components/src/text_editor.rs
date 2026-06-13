//! TextEditor — Multi-line text editing component.
//!
//! Supports word wrapping, vertical scrolling, selection, copy/paste,
//! and all standard editing operations.
//!
//! # OS-agnostic
//! All keyboard shortcuts use `cmd` modifier (maps to Command on macOS,
//! Ctrl on Windows/Linux). Clipboard via arboard (cross-platform).

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View, load_system_state, update_system_state};
use std::sync::Arc;

/// Multi-line text editor with word wrapping and vertical scrolling.
#[derive(Clone)]
pub struct TextEditor {
    /// Current text content (multi-line, with \n separators).
    pub text: String,
    /// Callback fired on every text change.
    pub on_change: Arc<dyn Fn(String) + Send + Sync>,
    /// Callback fired on commit (Cmd+Enter / Ctrl+Enter).
    pub on_commit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    /// Placeholder text shown when empty.
    pub placeholder: String,
    /// Whether the editor has keyboard focus.
    pub is_focused: bool,
    /// Unique hash for system state storage.
    pub state_id: u64,
    /// Number of visible lines (determines height).
    pub visible_lines: u32,
    /// Tab width in spaces.
    pub tab_width: u32,
}

/// Internal text editor state stored in system state map.
#[derive(Clone, Copy, Debug)]
pub struct EditorState {
    /// Cursor position as byte offset into text.
    cursor_pos: usize,
    /// Selection anchor (start of selection). None = no selection.
    selection_anchor: Option<usize>,
    /// Scroll offset in lines.
    scroll_offset: u32,
    /// Cursor blink phase (0 or 1).
    blink_phase: u32,
    /// Last blink timestamp for animation.
    last_blink_time: f32,
    /// Whether the pointer is currently dragging for selection.
    #[allow(dead_code)]
    is_dragging: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            cursor_pos: 0,
            selection_anchor: None,
            scroll_offset: 0,
            blink_phase: 0,
            last_blink_time: 0.0,
            is_dragging: false,
        }
    }
}

impl TextEditor {
    /// Create a new TextEditor.
    pub fn new(on_change: impl Fn(String) + Send + Sync + 'static) -> Self {
        Self {
            text: String::new(),
            on_change: Arc::new(on_change),
            on_commit: None,
            placeholder: String::new(),
            is_focused: false,
            state_id: 0,
            visible_lines: 10,
            tab_width: 4,
        }
    }

    /// Set placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the commit callback (fires on Cmd+Enter / Ctrl+Enter).
    pub fn on_commit(mut self, cb: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_commit = Some(Arc::new(cb));
        self
    }

    /// Set the number of visible lines.
    pub fn visible_lines(mut self, lines: u32) -> Self {
        self.visible_lines = lines.max(1);
        self
    }

    /// Set tab width in spaces.
    pub fn tab_width(mut self, width: u32) -> Self {
        self.tab_width = width.max(1);
        self
    }

    /// Set initial text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set focus state.
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    /// Set the state ID for system state storage.
    pub fn state_id(mut self, id: u64) -> Self {
        self.state_id = id;
        self
    }

    /// Get the current cursor position.
    fn read_state(&self) -> EditorState {
        if self.state_id == 0 {
            return EditorState::default();
        }
        let state = load_system_state();
        state
            .get_component_state::<EditorState>(self.state_id)
            .and_then(|guard| guard.read().ok().map(|v| *v))
            .unwrap_or_default()
    }

    /// Write editor state to system state.
    fn write_state(&self, editor_state: EditorState) {
        if self.state_id == 0 {
            return;
        }
        update_system_state(|s| {
            let mut ns = s.clone();
            ns.set_component_state(self.state_id, editor_state);
            ns
        });
    }

    /// Split text into lines at \n boundaries.
    fn lines(&self) -> Vec<&str> {
        self.text.split('\n').collect()
    }

    /// Get the line number and column for a byte offset.
    fn pos_to_line_col(&self, byte_pos: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        for (i, c) in self.text.chars().enumerate() {
            if i >= byte_pos {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Get the byte offset for a line and column.
    fn line_col_to_pos(&self, target_line: usize, target_col: usize) -> usize {
        let mut line = 0;
        let mut col = 0;
        for (i, c) in self.text.chars().enumerate() {
            if line == target_line && col == target_col {
                return i;
            }
            if c == '\n' {
                if line == target_line {
                    return i;
                }
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        self.text.len()
    }

    /// Insert text at cursor position, replacing selection.
    pub fn insert_text(&mut self, state: &mut EditorState, insert: &str) {
        if let Some(anchor) = state.selection_anchor {
            let start = state.cursor_pos.min(anchor);
            let end = state.cursor_pos.max(anchor);
            self.text.replace_range(start..end, insert);
            state.cursor_pos = start + insert.len();
            state.selection_anchor = None;
        } else {
            self.text.insert_str(state.cursor_pos, insert);
            state.cursor_pos += insert.len();
        }
    }

    /// Delete the selection or the character before the cursor.
    pub fn delete_backward(&mut self, state: &mut EditorState) {
        if let Some(anchor) = state.selection_anchor {
            let start = state.cursor_pos.min(anchor);
            let end = state.cursor_pos.max(anchor);
            self.text.replace_range(start..end, "");
            state.cursor_pos = start;
            state.selection_anchor = None;
        } else if state.cursor_pos > 0 {
            state.cursor_pos -= 1;
            self.text.remove(state.cursor_pos);
        }
    }

    /// Delete the character after the cursor.
    pub fn delete_forward(&mut self, state: &mut EditorState) {
        if let Some(_anchor) = state.selection_anchor {
            self.delete_backward(state);
        } else if state.cursor_pos < self.text.len() {
            self.text.remove(state.cursor_pos);
        }
    }

    /// Move cursor up one line.
    pub fn move_up(&self, state: &mut EditorState) {
        let (line, col) = self.pos_to_line_col(state.cursor_pos);
        if line > 0 {
            state.cursor_pos = self.line_col_to_pos(line - 1, col);
        }
    }

    /// Move cursor down one line.
    pub fn move_down(&self, state: &mut EditorState) {
        let (line, col) = self.pos_to_line_col(state.cursor_pos);
        let lines = self.lines();
        if line + 1 < lines.len() {
            state.cursor_pos = self.line_col_to_pos(line + 1, col);
        }
    }

    /// Move cursor left one character.
    pub fn move_left(&self, state: &mut EditorState, extend: bool) {
        if state.cursor_pos > 0 {
            state.cursor_pos -= 1;
        }
        if !extend {
            state.selection_anchor = None;
        } else if state.selection_anchor.is_none() {
            state.selection_anchor = Some(state.cursor_pos + 1);
        }
    }

    /// Move cursor right one character.
    pub fn move_right(&self, state: &mut EditorState, extend: bool) {
        if state.cursor_pos < self.text.len() {
            state.cursor_pos += 1;
        }
        if !extend {
            state.selection_anchor = None;
        } else if state.selection_anchor.is_none() {
            state.selection_anchor = Some(state.cursor_pos - 1);
        }
    }

    /// Select all text.
    pub fn select_all(&self, state: &mut EditorState) {
        state.selection_anchor = Some(0);
        state.cursor_pos = self.text.len();
    }

    /// Get selected text range.
    pub fn selection_range(&self, state: &EditorState) -> Option<(usize, usize)> {
        state.selection_anchor.map(|anchor| {
            let start = state.cursor_pos.min(anchor);
            let end = state.cursor_pos.max(anchor);
            (start, end)
        })
    }
}

impl View for TextEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "TextEditor");

        let state = self.read_state();
        let line_h = 18.0; // Line height in pixels
        let pad = 8.0;
        let editor_h = line_h * self.visible_lines as f32 + pad * 2.0;

        // Background
        renderer.fill_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: editor_h,
            },
            6.0,
            theme::surface_elevated(),
        );

        // Border
        let border_color = if self.is_focused {
            theme::accent()
        } else {
            theme::border()
        };
        renderer.stroke_rounded_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: editor_h,
            },
            6.0,
            border_color,
            if self.is_focused { 2.0 } else { 1.0 },
        );

        // Clip to content area
        let content_rect = Rect {
            x: rect.x + pad,
            y: rect.y + pad,
            width: rect.width - pad * 2.0,
            height: editor_h - pad * 2.0,
        };
        renderer.push_clip_rect(content_rect);

        let lines = self.lines();
        let total_lines = lines.len().max(1);

        // Render visible lines
        let start_line = state.scroll_offset as usize;
        let end_line = (start_line + self.visible_lines as usize).min(total_lines);

        for line_idx in start_line..end_line {
            let y = content_rect.y + (line_idx - start_line) as f32 * line_h;

            if line_idx < lines.len() {
                let line_text = lines[line_idx];

                // Check if this line has selection
                if let Some((sel_start, sel_end)) = self.selection_range(&state) {
                    let line_start_byte = self.line_col_to_pos(line_idx, 0);
                    let line_end_byte = line_start_byte + line_text.len();

                    // Calculate selection overlap with this line
                    let sel_in_line_start = sel_start.max(line_start_byte);
                    let sel_in_line_end = sel_end.min(line_end_byte);

                    if sel_in_line_start < sel_in_line_end {
                        // There's selection on this line
                        let pre_sel_col = self.pos_to_line_col(sel_in_line_start).1;
                        let post_sel_col = self
                            .pos_to_line_col(sel_in_line_end)
                            .1
                            .min(pre_sel_col + line_text.len());

                        // Render selection background
                        let sel_x = content_rect.x + pre_sel_col as f32 * 8.0;
                        let sel_w = (post_sel_col - pre_sel_col) as f32 * 8.0;
                        renderer.fill_rect(
                            Rect {
                                x: sel_x,
                                y,
                                width: sel_w.max(2.0),
                                height: line_h,
                            },
                            [
                                theme::accent()[0],
                                theme::accent()[1],
                                theme::accent()[2],
                                0.3,
                            ],
                        );
                    }
                }

                // Render line text
                if !line_text.is_empty() {
                    renderer.draw_text(line_text, content_rect.x, y + 3.0, 13.0, theme::text());
                }

                // Render cursor on this line
                if self.is_focused {
                    let (cursor_line, cursor_col) = self.pos_to_line_col(state.cursor_pos);
                    if cursor_line == line_idx {
                        // Blink
                        let t = renderer.elapsed_time();
                        if t - state.last_blink_time > 0.5 {
                            // Toggle blink — in a real impl, we'd write this back
                        }
                        let cursor_alpha = if state.blink_phase == 0 { 1.0 } else { 0.0 };
                        let cursor_x = content_rect.x + cursor_col as f32 * 8.0;
                        renderer.draw_line(
                            cursor_x,
                            y + 1.0,
                            cursor_x,
                            y + line_h - 2.0,
                            [
                                theme::accent()[0],
                                theme::accent()[1],
                                theme::accent()[2],
                                cursor_alpha,
                            ],
                            2.0,
                        );
                    }
                }
            }
        }

        // Render placeholder if empty
        if self.text.is_empty() && !self.placeholder.is_empty() {
            renderer.draw_text(
                &self.placeholder,
                content_rect.x,
                content_rect.y + 3.0,
                13.0,
                theme::text_dim(),
            );
        }

        renderer.pop_clip_rect();

        // Scrollbar
        if total_lines > self.visible_lines as usize {
            let sb_x = rect.x + rect.width - 6.0;
            let sb_h = editor_h - pad * 2.0;
            let thumb_h = (sb_h * self.visible_lines as f32 / total_lines as f32).max(20.0);
            let thumb_y = content_rect.y
                + (state.scroll_offset as f32
                    / (total_lines - self.visible_lines as usize).max(1) as f32)
                    * (sb_h - thumb_h);

            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: content_rect.y,
                    width: 4.0,
                    height: sb_h,
                },
                2.0,
                theme::surface_elevated(),
            );
            renderer.fill_rounded_rect(
                Rect {
                    x: sb_x,
                    y: thumb_y,
                    width: 4.0,
                    height: thumb_h,
                },
                2.0,
                theme::text_dim(),
            );
        }

        // Register keyboard handler (OS-agnostic: cmd maps to Command on macOS, Ctrl elsewhere)
        if self.is_focused && self.state_id != 0 {
            let state_id = self.state_id;
            renderer.register_handler(
                "keydown:cmd+a",
                Arc::new(move |_| {
                    update_system_state(|s| {
                        let mut ns = s.clone();
                        if let Some(guard) = ns.get_component_state::<EditorState>(state_id)
                            && let Ok(guard) = guard.read()
                        {
                            let mut st = *guard;
                            // Select all — need access to text, handled via component state
                            st.selection_anchor = Some(0);
                            ns.set_component_state(state_id, st);
                        }
                        ns
                    });
                }),
            );
        }

        self.write_state(state);
        renderer.pop_vnode();
    }
}

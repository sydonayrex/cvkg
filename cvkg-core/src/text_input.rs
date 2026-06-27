/// Direction for cursor movement in text input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Forward,
    Backward,
    Up,
    Down,
    LineStart,
    LineEnd,
    WordForward,
    WordBackward,
}

/// Text input state managed by the renderer.
///
/// Components don't store this directly -- the renderer maintains it
/// and components query/modify it through the Renderer trait methods.
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    /// The full text content.
    pub text: String,
    /// Cursor position as byte offset into the text.
    pub cursor_pos: usize,
    /// Selection anchor. If Some, the selection is from anchor to cursor.
    /// If None, there is no selection.
    pub selection_anchor: Option<usize>,
    /// Whether the input is focused (shows cursor, accepts keyboard).
    pub focused: bool,
    /// Whether the caret is currently visible (for blinking).
    pub caret_visible: bool,
    /// Last edit timestamp for undo coalescing.
    pub last_edit_time: f32,
}

impl TextInputState {
    /// Create a new TextInputState with the given initial text.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor_pos = text.len();
        Self {
            text,
            cursor_pos,
            selection_anchor: None,
            focused: false,
            caret_visible: true,
            last_edit_time: 0.0,
        }
    }

    /// Get the selection range as (start, end) byte offsets.
    /// Returns None if there is no selection.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            if anchor <= self.cursor_pos {
                (anchor, self.cursor_pos)
            } else {
                (self.cursor_pos, anchor)
            }
        })
    }

    /// Get the selected text, or empty string if no selection.
    pub fn selected_text(&self) -> String {
        self.selection_range()
            .map(|(start, end)| self.text[start..end].to_string())
            .unwrap_or_default()
    }

    /// Insert text at the current cursor position, replacing any selection.
    pub fn insert(&mut self, new_text: &str) {
        if let Some((start, end)) = self.selection_range() {
            self.text.replace_range(start..end, new_text);
            self.cursor_pos = start + new_text.len();
        } else {
            self.text.insert_str(self.cursor_pos, new_text);
            self.cursor_pos += new_text.len();
        }
        self.selection_anchor = None;
    }

    /// Delete characters. If there's a selection, delete it.
    /// Otherwise delete `count` characters backward (backspace) or forward (delete).
    pub fn delete(&mut self, backward: bool, count: usize) -> String {
        if let Some((start, end)) = self.selection_range() {
            let deleted = self.text[start..end].to_string();
            self.text.replace_range(start..end, "");
            self.cursor_pos = start;
            self.selection_anchor = None;
            return deleted;
        }

        if backward && self.cursor_pos > 0 {
            let start = self.cursor_pos.saturating_sub(count);
            let deleted = self.text[start..self.cursor_pos].to_string();
            self.text.replace_range(start..self.cursor_pos, "");
            self.cursor_pos = start;
            deleted
        } else if !backward && self.cursor_pos < self.text.len() {
            let end = (self.cursor_pos + count).min(self.text.len());
            let deleted = self.text[self.cursor_pos..end].to_string();
            self.text.replace_range(self.cursor_pos..end, "");
            deleted
        } else {
            String::new()
        }
    }

    /// Move the cursor in the given direction.
    pub fn move_cursor(&mut self, direction: TextDirection, extend_selection: bool) {
        if !extend_selection {
            self.selection_anchor = None;
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_pos);
        }

        match direction {
            TextDirection::Forward if self.cursor_pos < self.text.len() => {
                // Move to next character boundary (UTF-8 safe)
                let next = self.text[self.cursor_pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| self.cursor_pos + i)
                    .unwrap_or(self.text.len());
                self.cursor_pos = next;
            }
            TextDirection::Backward if self.cursor_pos > 0 => {
                let prev = self.text[..self.cursor_pos]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.cursor_pos = prev;
            }
            TextDirection::LineStart => {
                self.cursor_pos = 0;
            }
            TextDirection::LineEnd => {
                self.cursor_pos = self.text.len();
            }
            TextDirection::WordForward => {
                // Find next word boundary
                let rest = &self.text[self.cursor_pos..];
                // Skip current word chars
                let after_word = rest
                    .char_indices()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(i, _)| i)
                    .unwrap_or(rest.len());
                // Skip whitespace
                let after_space = rest[after_word..]
                    .char_indices()
                    .find(|(_, c)| !c.is_whitespace())
                    .map(|(i, _)| after_word + i)
                    .unwrap_or(rest.len());
                self.cursor_pos = (self.cursor_pos + after_space).min(self.text.len());
            }
            TextDirection::WordBackward => {
                let before = &self.text[..self.cursor_pos];
                // Skip whitespace going backward
                let before_word = before
                    .char_indices()
                    .rev()
                    .find(|(_, c)| !c.is_whitespace())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                // Skip word chars going backward
                let word_start = before[..before_word]
                    .char_indices()
                    .rev()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.cursor_pos = word_start;
            }
            _ => {} // Up/Down handled by multi-line components
        }

        if !extend_selection {
            self.selection_anchor = None;
        }
    }

    /// Select all text.
    pub fn select_all(&mut self) {
        self.cursor_pos = self.text.len();
        self.selection_anchor = Some(0);
    }

    /// Get the byte offset of the cursor.
    pub fn cursor_byte_pos(&self) -> usize {
        self.cursor_pos
    }
}

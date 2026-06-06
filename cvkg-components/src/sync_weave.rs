//! SyncWeave — Real-Time Collaboration Primitives.
//!
//! Provides a CRDT-backed collaborative text buffer with remote cursor
//! visualization. Uses a simplified LogootSplit CRDT for deterministic
//! merge of concurrent edits.

use crate::theme;
use cvkg_core::{Event, Never, Rect, Renderer, View};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A single character in the CRDT buffer with a unique position identifier.
#[derive(Debug, Clone)]
struct WeaveChar {
    position: Vec<u64>,
    value: char,
    site_id: u64,
    timestamp: u64,
    deleted: bool,
}

/// A CRDT-backed collaborative text buffer.
///
/// Uses LogootSplit (simplified) for deterministic merge of concurrent edits.
/// Each character has a unique position ID that is totally ordered, so
/// concurrent inserts at the same position are ordered by ID without conflicts.
#[derive(Debug, Clone)]
pub struct SyncWeave {
    chars: Vec<WeaveChar>,
    site_id: u64,
    clock: u64,
}

impl SyncWeave {
    pub fn new(site_id: u64) -> Self {
        Self {
            chars: Vec::new(),
            site_id,
            clock: 0,
        }
    }

    /// Insert a character at the given visible position.
    pub fn local_insert(&mut self, visible_pos: usize, ch: char) -> WeaveOp {
        self.clock += 1;
        let position = self.allocate_position(visible_pos);
        let weave_char = WeaveChar {
            position,
            value: ch,
            site_id: self.site_id,
            timestamp: self.clock,
            deleted: false,
        };
        self.chars.insert(visible_pos, weave_char.clone());
        WeaveOp::Insert { ch: weave_char }
    }

    /// Delete the character at the given visible position.
    pub fn local_delete(&mut self, visible_pos: usize) -> Option<WeaveOp> {
        let actual = self.visible_to_actual(visible_pos)?;
        self.clock += 1;
        self.chars[actual].deleted = true;
        Some(WeaveOp::Delete {
            position: self.chars[actual].position.clone(),
        })
    }

    /// Apply a remote operation from another collaborator.
    pub fn apply_remote(&mut self, op: &WeaveOp) {
        match op {
            WeaveOp::Insert { ch } => {
                let idx = self
                    .chars
                    .partition_point(|c| c.position < ch.position);
                self.chars.insert(idx, ch.clone());
            }
            WeaveOp::Delete { position } => {
                if let Some(c) = self.chars.iter_mut().find(|c| &c.position == position) {
                    c.deleted = true;
                }
            }
        }
    }

    /// Get the visible text (excluding deleted characters).
    pub fn text(&self) -> String {
        self.chars
            .iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }

    /// Get the number of visible characters.
    pub fn len(&self) -> usize {
        self.chars.iter().filter(|c| !c.deleted).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn allocate_position(&self, visible_pos: usize) -> Vec<u64> {
        let prev = if visible_pos > 0 {
            self.chars.get(visible_pos - 1).map(|c| &c.position)
        } else {
            None
        };
        let next = self.chars.get(visible_pos).map(|c| &c.position);

        match (prev, next) {
            (Some(p), Some(n)) => midpoint(p, n),
            (Some(p), None) => increment_last(p),
            (None, Some(n)) => vec![n[0] / 2],
            (None, None) => vec![u64::MAX / 2],
        }
    }

    fn visible_to_actual(&self, visible_pos: usize) -> Option<usize> {
        let mut visible = 0;
        for (i, c) in self.chars.iter().enumerate() {
            if !c.deleted {
                if visible == visible_pos {
                    return Some(i);
                }
                visible += 1;
            }
        }
        None
    }
}

/// A collaborative operation that can be broadcast to other editors.
#[derive(Debug, Clone)]
pub enum WeaveOp {
    Insert { ch: WeaveChar },
    Delete { position: Vec<u64> },
}

/// A cursor position from a remote collaborator.
#[derive(Debug, Clone)]
pub struct PeerCursor {
    pub site_id: u64,
    pub name: String,
    pub color: [f32; 4],
    pub position: usize,
}

/// The collaborative text editor component.
pub struct SyncEditor {
    buffer: Arc<Mutex<SyncWeave>>,
    remote_cursors: Arc<Mutex<HashMap<u64, PeerCursor>>>,
    cursor_pos: Arc<Mutex<usize>>,
    site_id: u64,
}

impl SyncEditor {
    pub fn new(site_id: u64) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(SyncWeave::new(site_id))),
            remote_cursors: Arc::new(Mutex::new(HashMap::new())),
            cursor_pos: Arc::new(Mutex::new(0)),
            site_id,
        }
    }

    /// Get access to the shared buffer.
    pub fn buffer(&self) -> Arc<Mutex<SyncWeave>> {
        self.buffer.clone()
    }

    /// Update a remote cursor position.
    pub fn update_peer_cursor(&self, cursor: PeerCursor) {
        if let Ok(mut cursors) = self.remote_cursors.lock() {
            cursors.insert(cursor.site_id, cursor);
        }
    }

    /// Remove a remote cursor.
    pub fn remove_peer(&self, site_id: u64) {
        if let Ok(mut cursors) = self.remote_cursors.lock() {
            cursors.remove(&site_id);
        }
    }

    /// Get the current local cursor position.
    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos.lock().map(|p| *p).unwrap_or(0)
    }
}

impl View for SyncEditor {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let buffer = match self.buffer.lock() {
            Ok(b) => b.clone(),
            Err(_) => {
                renderer.draw_text("[sync error]", rect.x, rect.y, 14.0, [1.0, 0.0, 0.0, 1.0]);
                return;
            }
        };

        let text = buffer.text();
        let cursor_pos = self.cursor_pos();
        let cursors = match self.remote_cursors.lock() {
            Ok(c) => c.clone(),
            Err(_) => HashMap::new(),
        };

        // Render background
        renderer.fill_rect(rect, theme::surface());

        // Render text
        let mut y = rect.y + 4.0;
        let line_height = 20.0;
        for (_line_idx, line) in text.lines().enumerate() {
            renderer.draw_text(line, rect.x + 4.0, y, 14.0, theme::text());
            y += line_height;
        }

        // Render local cursor
        let text_before = text.chars().take(cursor_pos).collect::<String>();
        let last_nl = text_before.rfind('\n').map(|p| p + 1).unwrap_or(0);
        let chars_on_line = text_before[last_nl..].chars().count();
        let lines_before = text_before.chars().filter(|c| *c == '\n').count();

        let local_cx = rect.x + 4.0 + (chars_on_line as f32) * 8.0;
        let local_cy = rect.y + 4.0 + (lines_before as f32) * line_height;

        renderer.draw_line(
            local_cx,
            local_cy,
            local_cx,
            local_cy + line_height,
            [0.0, 0.8, 1.0, 1.0],
            2.0,
        );

        // Render remote cursors
        for (_site_id, cursor) in cursors.iter() {
            let text_before = text.chars().take(cursor.position).collect::<String>();
            let last_nl = text_before.rfind('\n').map(|p| p + 1).unwrap_or(0);
            let chars_on_line = text_before[last_nl..].chars().count();
            let lines_before = text_before.chars().filter(|c| *c == '\n').count();

            let cx = rect.x + 4.0 + (chars_on_line as f32) * 8.0;
            let cy = rect.y + 4.0 + (lines_before as f32) * line_height;

            renderer.draw_line(cx, cy, cx, cy + line_height, cursor.color, 2.0);

            // Name label above cursor
            let label_rect = Rect::new(cx, cy - 14.0, cursor.name.len() as f32 * 7.0 + 4.0, 14.0);
            renderer.fill_rounded_rect(label_rect, 2.0, cursor.color);
            renderer.draw_text(&cursor.name, cx + 2.0, cy - 2.0, 10.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}

fn midpoint(a: &[u64], b: &[u64]) -> Vec<u64> {
    let max_len = a.len().max(b.len());
    let mut a_pad = a.to_vec();
    a_pad.resize(max_len, 0);
    let mut b_pad = b.to_vec();
    b_pad.resize(max_len, 0);

    for i in 0..max_len {
        if a_pad[i] != b_pad[i] {
            let mid = a_pad[i] + (b_pad[i] - a_pad[i]) / 2;
            let mut result = Vec::with_capacity(i + 1);
            for j in 0..i {
                result.push(a_pad[j]);
            }
            result.push(mid);
            return result;
        }
    }
    let mut result = a.to_vec();
    result.push(u64::MAX / 2);
    result
}

fn increment_last(a: &[u64]) -> Vec<u64> {
    let mut result = a.to_vec();
    if let Some(last) = result.last_mut() {
        *last += 1;
    }
    result
}

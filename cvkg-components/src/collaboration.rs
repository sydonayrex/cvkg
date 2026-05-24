use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};

/// Participant in a collaboration session
pub struct Participant {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub status: ParticipantStatus,
    pub cursor: Option<(f32, f32)>,
}

pub enum ParticipantStatus {
    Online,
    Away,
    Offline,
}

/// Collaboration engine for real-time multi-user editing
pub struct CollaborationEngine {
    pub(crate) participants: Vec<Participant>,
    pub(crate) session_name: String,
}

impl CollaborationEngine {
    pub fn new(name: &str) -> Self {
        Self {
            participants: Vec::new(),
            session_name: name.to_string(),
        }
    }

    pub fn participant(mut self, id: &str, name: &str, avatar: &str) -> Self {
        self.participants.push(Participant {
            id: id.to_string(),
            name: name.to_string(),
            avatar: avatar.to_string(),
            status: ParticipantStatus::Online,
            cursor: None,
        });
        self
    }

    pub fn cursor(mut self, id: &str, x: f32, y: f32) -> Self {
        if let Some(p) = self.participants.iter_mut().find(|p| p.id == id) {
            p.cursor = Some((x, y));
        }
        self
    }
}

impl View for CollaborationEngine {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Session header
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 32.0,
            },
            theme::surface_elevated(),
        );
        renderer.draw_text(
            &self.session_name,
            rect.x + 8.0,
            rect.y + 12.0,
            14.0,
            [0.8, 0.9, 1.0, 1.0],
        );

        // Participants list
        let item_h = 40.0;
        let mut current_y = rect.y + 40.0;
        for p in &self.participants {
            let part_rect = Rect {
                x: rect.x,
                y: current_y,
                width: rect.width,
                height: item_h,
            };

            let status_color = match p.status {
                ParticipantStatus::Online => theme::success(),
                ParticipantStatus::Away => [0.8, 0.6, 0.0, 1.0],
                ParticipantStatus::Offline => [0.4, 0.4, 0.4, 1.0],
            };

            let status_rect = Rect {
                x: part_rect.x + 10.0,
                y: part_rect.y + 14.0,
                width: 12.0,
                height: 12.0,
            };
            renderer.fill_ellipse(status_rect, status_color);
            renderer.stroke_ellipse(status_rect, theme::shadow(), 1.0);
            renderer.draw_text(
                &p.name,
                part_rect.x + 28.0,
                part_rect.y + 14.0,
                12.0,
                theme::text(),
            );

            // Draw cursor position if present
            if let Some((cx, cy)) = p.cursor {
                renderer.draw_text(
                    &format!("📍 ({}, {})", cx as i32, cy as i32),
                    part_rect.x + 28.0,
                    part_rect.y + 28.0,
                    10.0,
                    [0.5, 0.6, 0.7, 1.0],
                );
            }
            current_y += item_h;
        }
    }
}

impl LayoutView for CollaborationEngine {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 200.0,
            height: 40.0 + self.participants.len() as f32 * 40.0,
        }
    }
    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

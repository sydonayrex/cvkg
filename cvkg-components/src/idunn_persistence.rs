//! Idunn Persistence - Workspace persistence and session management
//!
//! Idunn the Vanir goddess preserves eternal youth and golden apples of renewal -
//! this persistence system preserves workspace state and enables session restoration.

use crate::theme;
use cvkg_core::{
    Never, Rect, Renderer, Size, View,
    layout::{LayoutCache, LayoutView, SizeProposal},
};
use std::collections::HashMap;

/// Workspace state snapshot
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub name: String,
    pub timestamp: f64,
    pub component_states: HashMap<String, String>,
    pub layout: String,
}

/// Session restore point
#[derive(Debug, Clone)]
pub struct SessionRestore {
    pub session_id: String,
    pub workspace_id: String,
    pub restore_time: f64,
}

/// Idunn Persistence for workspace state management
pub struct IdunnPersistence {
    pub snapshots: Vec<WorkspaceSnapshot>,
    pub active_session: Option<SessionRestore>,
    pub auto_restore: bool,
}

impl Default for IdunnPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl IdunnPersistence {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            active_session: None,
            auto_restore: true,
        }
    }

    /// Create a snapshot of current workspace
    pub fn snapshot(mut self, id: &str, name: &str, layout: &str) -> Self {
        self.snapshots.push(WorkspaceSnapshot {
            id: id.to_string(),
            name: name.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            component_states: HashMap::new(),
            layout: layout.to_string(),
        });
        self
    }

    /// Add component state to snapshot
    pub fn component_state(mut self, snapshot_id: &str, key: &str, state: &str) -> Self {
        if let Some(snap) = self.snapshots.iter_mut().find(|s| s.id == snapshot_id) {
            snap.component_states
                .insert(key.to_string(), state.to_string());
        }
        self
    }

    /// Start a session
    pub fn session(mut self, session_id: &str, workspace_id: &str) -> Self {
        self.active_session = Some(SessionRestore {
            session_id: session_id.to_string(),
            workspace_id: workspace_id.to_string(),
            restore_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        });
        self
    }

    /// Enable/disable auto restore
    pub fn auto_restore(mut self, enabled: bool) -> Self {
        self.auto_restore = enabled;
        self
    }
}

impl View for IdunnPersistence {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Header
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: 28.0,
            },
            [0.06, 0.08, 0.06, 1.0],
        );
        renderer.draw_text(
            "Idunn Persistence",
            rect.x + 10.0,
            rect.y + 9.0,
            13.0,
            theme::success(),
        );

        // Auto-restore indicator
        let auto_text = if self.auto_restore {
            "Auto-Restore: ON"
        } else {
            "Auto-Restore: OFF"
        };
        renderer.draw_text(
            auto_text,
            rect.x + 120.0,
            rect.y + 9.0,
            10.0,
            [0.5, 0.8, 0.5, 1.0],
        );

        // Active session
        if let Some(session) = &self.active_session {
            renderer.draw_text(
                &format!("Session: {}", session.session_id),
                rect.x + 10.0,
                rect.y + 45.0,
                11.0,
                theme::success(),
            );
            renderer.draw_text(
                &format!("Workspace: {}", session.workspace_id),
                rect.x + 10.0,
                rect.y + 62.0,
                10.0,
                [0.5, 0.7, 0.5, 1.0],
            );
        }

        // Snapshots list
        let mut y = rect.y + 85.0;
        renderer.draw_text("Snapshots:", rect.x + 10.0, y, 11.0, [0.8, 0.9, 0.8, 1.0]);
        y += 20.0;

        for snap in &self.snapshots {
            let age = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
                - snap.timestamp;
            let age_str = if age < 60.0 {
                format!("{}s ago", age as u32)
            } else if age < 3600.0 {
                format!("{}m ago", (age / 60.0) as u32)
            } else {
                format!("{}h ago", (age / 3600.0) as u32)
            };

            renderer.fill_rect(
                Rect {
                    x: rect.x + 15.0,
                    y,
                    width: rect.width - 30.0,
                    height: 22.0,
                },
                [0.05, 0.07, 0.05, 1.0],
            );
            renderer.draw_text(&snap.name, rect.x + 20.0, y + 6.0, 10.0, theme::success());
            renderer.draw_text(
                &age_str,
                rect.x + rect.width - 60.0,
                y + 6.0,
                9.0,
                [0.5, 0.7, 0.5, 1.0],
            );
            y += 26.0;
        }
    }
}

impl LayoutView for IdunnPersistence {
    fn size_that_fits(
        &self,
        _proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        Size {
            width: 260.0,
            height: 100.0 + self.snapshots.len() as f32 * 26.0,
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

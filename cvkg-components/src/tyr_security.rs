//! Tyr Security - Security and enterprise features
//!
//! Tyr the Aesir god of law and justice protects order - this security system
//! provides permission management, role-based rendering, and audit capabilities.

use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never};

/// User permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Guest,
    User,
    Admin,
    SuperAdmin,
}

/// Security role definition
#[derive(Debug, Clone)]
pub struct SecurityRole {
    pub name: String,
    pub level: PermissionLevel,
    pub permissions: Vec<String>,
}

/// Audit entry for security events
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: f64,
    pub user: String,
    pub action: String,
    pub resource: String,
    pub success: bool,
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub user: String,
    pub created_at: f64,
    pub expires_at: f64,
    pub ip_address: String,
}

/// Tyr Security system for enterprise protection
pub struct TyrSecurity {
    pub(crate) roles: Vec<SecurityRole>,
    pub(crate) audit_log: Vec<AuditEntry>,
    pub(crate) current_session: Option<SessionInfo>,
}

impl Default for TyrSecurity {
    fn default() -> Self {
        Self::new()
    }
}

impl TyrSecurity {
    pub fn new() -> Self {
        Self {
            roles: Vec::new(),
            audit_log: Vec::new(),
            current_session: None,
        }
    }

    /// Add a security role
    pub fn role(mut self, name: &str, level: PermissionLevel, permissions: Vec<&str>) -> Self {
        self.roles.push(SecurityRole {
            name: name.to_string(),
            level,
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Log an audit event
    pub fn audit(mut self, user: &str, action: &str, resource: &str, success: bool) -> Self {
        self.audit_log.push(AuditEntry {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
            user: user.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            success,
        });
        self
    }

    /// Start a session
    pub fn session(mut self, id: &str, user: &str, expires_in_hours: f64) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
        self.current_session = Some(SessionInfo {
            id: id.to_string(),
            user: user.to_string(),
            created_at: now,
            expires_at: now + expires_in_hours * 3600.0,
            ip_address: "127.0.0.1".to_string(),
        });
        self
    }

    /// Check if action is permitted for role
    pub fn can(&self, role_name: &str, permission: &str) -> bool {
        self.roles.iter()
            .find(|r| r.name == role_name)
            .map(|r| r.permissions.contains(&permission.to_string()))
            .unwrap_or(false)
    }
}

impl View for TyrSecurity {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.08, 0.04, 0.06, 1.0]);
        renderer.draw_text("Tyr Security", rect.x + 10.0, rect.y + 20.0, 14.0, [0.9, 0.6, 0.6, 1.0]);

        // Session info
        if let Some(session) = &self.current_session {
            renderer.draw_text(&format!("Session: {}", session.id), rect.x + 110.0, rect.y + 20.0, 10.0, [0.8, 0.7, 0.9, 1.0]);

            let remaining = session.expires_at - std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
            let hours = (remaining / 3600.0).max(0.0) as u32;
            renderer.draw_text(&format!("Expires: {}h", hours), rect.x + 110.0, rect.y + 35.0, 9.0, [0.6, 0.8, 0.9, 1.0]);
        }

        // Roles
        let mut y = rect.y + 60.0;
        renderer.draw_text("Roles:", rect.x + 10.0, y, 11.0, [0.9, 0.7, 0.7, 1.0]);
        y += 20.0;

        for role in &self.roles {
            let level_str = match role.level {
                PermissionLevel::Guest => "Guest",
                PermissionLevel::User => "User",
                PermissionLevel::Admin => "Admin",
                PermissionLevel::SuperAdmin => "Super",
            };

            renderer.fill_rect(Rect { x: rect.x + 15.0, y, width: 60.0, height: 18.0 }, [0.4, 0.2, 0.2, 1.0]);
            renderer.draw_text(level_str, rect.x + 20.0, y + 4.0, 9.0, [0.9, 0.8, 0.8, 1.0]);
            renderer.draw_text(&role.name, rect.x + 80.0, y + 4.0, 10.0, [0.8, 0.9, 1.0, 1.0]);
            y += 22.0;
        }

        // Recent audit log
        let audit_y = rect.y + rect.height - 80.0;
        renderer.draw_text("Recent Activity:", rect.x + 10.0, audit_y, 10.0, [0.7, 0.8, 1.0, 1.0]);

        for (i, entry) in self.audit_log.iter().rev().take(3).enumerate() {
            let status = if entry.success { "✓" } else { "✗" };
            let color = if entry.success { [0.4, 0.9, 0.4, 1.0] } else { [0.9, 0.4, 0.4, 1.0] };

            renderer.draw_text(&format!("{} {} {}", status, entry.user, entry.action), rect.x + 15.0, audit_y + 15.0 + i as f32 * 15.0, 9.0, color);
        }
    }
}

impl LayoutView for TyrSecurity {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        Size { width: 280.0, height: 150.0 + self.roles.len() as f32 * 22.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}
use cvkg_core::{layout::{LayoutCache, LayoutView, SizeProposal}, Rect, Renderer, Size, View, Never};

/// Agent task in an orchestration workflow
pub struct AgentTask {
    pub id: String,
    pub agent_name: String,
    pub status: TaskStatus,
    pub progress: f32,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Multi-agent orchestrator for managing agent workflows
pub struct MultiAgentOrchestrator {
    pub(crate) tasks: Vec<AgentTask>,
    pub(crate) workflow_name: String,
}

impl MultiAgentOrchestrator {
    pub fn new(name: &str) -> Self {
        Self {
            tasks: Vec::new(),
            workflow_name: name.to_string(),
        }
    }

    pub fn task(mut self, id: &str, agent: &str) -> Self {
        self.tasks.push(AgentTask {
            id: id.to_string(),
            agent_name: agent.to_string(),
            status: TaskStatus::Pending,
            progress: 0.0,
        });
        self
    }

    pub fn progress(mut self, id: &str, progress: f32) -> Self {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.progress = progress;
            if progress >= 1.0 { t.status = TaskStatus::Completed; }
        }
        self
    }
}

impl View for MultiAgentOrchestrator {
    type Body = Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Workflow header
        renderer.fill_rect(
            Rect { x: rect.x, y: rect.y, width: rect.width, height: 30.0 },
            [0.08, 0.06, 0.12, 1.0]
        );
        renderer.draw_text(&self.workflow_name, rect.x + 8.0, rect.y + 9.0, 14.0, [0.8, 0.7, 1.0, 1.0]);

        // Task list
        let row_h = 36.0;
        let mut current_y = rect.y + 42.0;
        for task in &self.tasks {
            let task_rect = Rect { x: rect.x + 10.0, y: current_y, width: rect.width - 20.0, height: row_h };

            // Background
            let bg = match task.status {
                TaskStatus::Pending => [0.04, 0.04, 0.08, 1.0],
                TaskStatus::Running => [0.06, 0.08, 0.12, 1.0],
                TaskStatus::Completed => [0.04, 0.12, 0.08, 1.0],
                TaskStatus::Failed => [0.12, 0.04, 0.06, 1.0],
            };
            renderer.fill_rounded_rect(task_rect, 4.0, bg);

            // Agent name
            renderer.draw_text(&task.agent_name, task_rect.x + 8.0, task_rect.y + 12.0, 12.0, [0.7, 0.8, 0.9, 1.0]);

            // Progress bar
            let progress_w = (task_rect.width - 100.0) * task.progress;
            renderer.fill_rect(
                Rect { x: task_rect.x + 90.0, y: task_rect.y + 24.0, width: progress_w, height: 6.0 },
                [0.0, 0.8, 1.0, 1.0]
            );
            renderer.stroke_rounded_rect(
                Rect { x: task_rect.x + 90.0, y: task_rect.y + 24.0, width: task_rect.width - 100.0, height: 6.0 },
                3.0, [0.3, 0.4, 0.5, 1.0], 1.0
            );

            // Status indicator
            let status_char = match task.status {
                TaskStatus::Pending => "○",
                TaskStatus::Running => "◐",
                TaskStatus::Completed => "✓",
                TaskStatus::Failed => "✗",
            };
            renderer.draw_text(status_char, task_rect.x + task_rect.width - 20.0, task_rect.y + 12.0, 14.0, [0.6, 0.8, 1.0, 1.0]);

            current_y += row_h + 4.0;
        }
    }
}

impl LayoutView for MultiAgentOrchestrator {
    fn size_that_fits(&self, _proposal: SizeProposal, _subviews: &[&dyn LayoutView], _cache: &mut LayoutCache) -> Size {
        Size { width: 300.0, height: 40.0 + self.tasks.len() as f32 * 40.0 }
    }
    fn place_subviews(&self, _bounds: Rect, _subviews: &mut [&mut dyn LayoutView], _cache: &mut LayoutCache) {}
}

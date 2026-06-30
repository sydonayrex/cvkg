use super::MultiAgentOrchestrator;
use super::types::{MessageType, NodeExecutionStatus, OrchestratorNode};
use crate::theme;
use crate::{RADIUS_LG, RADIUS_MD, RADIUS_SM, RADIUS_XS};
use cvkg_core::{Rect, Renderer};

// ============================================================
// Helper: make a Rect
// ============================================================
fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Rendering Helpers
// ═══════════════════════════════════════════════════════════════════════════

impl MultiAgentOrchestrator {
    /// Render the main graph canvas with nodes and edges.
    pub(crate) fn render_graph_canvas(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, theme::editor_bg());

        // Grid
        self.render_grid(renderer, rect);

        // Clip to graph area
        renderer.push_clip_rect(rect);

        // Render edges (behind nodes)
        self.render_edges(renderer, rect);

        // Render nodes
        for node in &self.state.nodes {
            self.render_node(renderer, node, rect);
        }

        // Render pending edge (if any)
        if let Some((ref node_id, ref port)) = self.pending_edge {
            self.render_pending_edge(renderer, node_id, port, rect);
        }

        renderer.pop_clip_rect();

        // ── Toolbar ─────────────────────────────────────────────────────
        self.render_toolbar(renderer, rect);
    }

    /// Render the background grid.
    fn render_grid(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let grid_spacing = 40.0 * self.state.viewport_zoom;
        let offset_x = self.state.viewport_offset.0 % grid_spacing;
        let offset_y = self.state.viewport_offset.1 % grid_spacing;
        let grid_color = theme::editor_grid();

        let mut x = rect.x + offset_x;
        while x < rect.x + rect.width {
            renderer.draw_line(x, rect.y, x, rect.y + rect.height, grid_color, 0.5);
            x += grid_spacing;
        }

        let mut y = rect.y + offset_y;
        while y < rect.y + rect.height {
            renderer.draw_line(rect.x, y, rect.x + rect.width, y, grid_color, 0.5);
            y += grid_spacing;
        }
    }

    /// Render all edges as Bézier curves.
    fn render_edges(&self, renderer: &mut dyn Renderer, _rect: Rect) {
        for edge in &self.state.edges {
            let source = match self.state.nodes.iter().find(|n| n.id == edge.source_node) {
                Some(n) => n,
                None => continue,
            };
            let target = match self.state.nodes.iter().find(|n| n.id == edge.target_node) {
                Some(n) => n,
                None => continue,
            };

            let sx = source.position.0 + source.size.0;
            let sy = source.position.1 + source.size.1 / 2.0;
            let tx = target.position.0;
            let ty = target.position.1 + target.size.1 / 2.0;

            let dx = (tx - sx).abs();
            let cp_offset = dx * 0.4;

            // Glow (wider, translucent)
            let glow_color = if edge.is_active {
                theme::with_alpha(theme::accent(), 0.15)
            } else {
                theme::with_alpha(theme::border(), 0.1)
            };
            self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, glow_color, 6.0);

            // Main edge
            let edge_color = if edge.is_active {
                theme::with_alpha(theme::accent(), 0.9)
            } else {
                theme::with_alpha(theme::border(), 0.7)
            };
            self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, edge_color, 2.0);

            // Arrow head
            self.draw_arrow_head(renderer, tx, ty, tx - cp_offset, ty, edge_color);

            // Condition label
            if let Some(ref cond) = edge.condition {
                let mid_x = (sx + tx) / 2.0;
                let mid_y = (sy + ty) / 2.0 - 12.0;
                let tw = renderer.measure_text(cond, 9.0);
                renderer.fill_rounded_rect(
                    Rect {
                        x: mid_x - tw.0 / 2.0 - 4.0,
                        y: mid_y - 6.0,
                        width: tw.0 + 8.0,
                        height: 14.0,
                    },
                    3.0,
                    theme::surface_elevated(),
                );
                renderer.draw_text_raw(cond, mid_x - tw.0 / 2.0, mid_y + 3.0, 9.0, theme::text());
            }
        }
    }

    /// Draw a cubic Bézier curve as a series of line segments.
    fn draw_bezier_edge(
        &self,
        renderer: &mut dyn Renderer,
        x0: f32,
        y0: f32,
        x3: f32,
        y3: f32,
        cp_offset: f32,
        color: [f32; 4],
        width: f32,
    ) {
        let x1 = x0 + cp_offset;
        let y1 = y0;
        let x2 = x3 - cp_offset;
        let y2 = y3;

        let segments = 24;
        let mut prev_x = x0;
        let mut prev_y = y0;

        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            let x = mt3 * x0 + 3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3 * x3;
            let y = mt3 * y0 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3 * y3;

            renderer.draw_line(prev_x, prev_y, x, y, color, width);
            prev_x = x;
            prev_y = y;
        }
    }

    /// Draw an arrow head at the end of an edge.
    fn draw_arrow_head(
        &self,
        renderer: &mut dyn Renderer,
        tip_x: f32,
        tip_y: f32,
        from_x: f32,
        from_y: f32,
        color: [f32; 4],
    ) {
        let dx = tip_x - from_x;
        let dy = tip_y - from_y;
        let len = (dx * dx + dy * dy).sqrt().max(0.01);
        let nx = dx / len;
        let ny = dy / len;

        let arrow_len = 10.0;
        let arrow_width = 5.0;

        let left_x = tip_x - nx * arrow_len - ny * arrow_width;
        let left_y = tip_y - ny * arrow_len + nx * arrow_width;
        let right_x = tip_x - nx * arrow_len + ny * arrow_width;
        let right_y = tip_y - ny * arrow_len - nx * arrow_width;

        renderer.draw_line(tip_x, tip_y, left_x, left_y, color, 2.0);
        renderer.draw_line(tip_x, tip_y, right_x, right_y, color, 2.0);
        renderer.draw_line(left_x, left_y, right_x, right_y, color, 2.0);
    }

    /// Render a single node.
    fn render_node(&self, renderer: &mut dyn Renderer, node: &OrchestratorNode, _rect: Rect) {
        let nx = node.position.0 + self.state.viewport_offset.0;
        let ny = node.position.1 + self.state.viewport_offset.1;
        let nw = node.size.0;
        let nh = node.size.1;

        let node_rect = Rect {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
        };

        let is_selected = self.state.selected_node.as_ref() == Some(&node.id);
        let node_color = node.node_type.color();

        // Drop shadow
        renderer.fill_rounded_rect(
            Rect {
                x: nx + 3.0,
                y: ny + 3.0,
                width: nw,
                height: nh,
            },
            6.0,
            theme::shadow(),
        );

        // Node body
        renderer.fill_rounded_rect(node_rect, RADIUS_MD, theme::surface_elevated());

        // Selection highlight
        if is_selected {
            renderer.stroke_rounded_rect(node_rect, RADIUS_MD, theme::accent(), 2.0);
        }

        // Title bar
        let title_h = 28.0;
        let title_rect = Rect {
            x: nx,
            y: ny,
            width: nw,
            height: title_h,
        };
        renderer.fill_rounded_rect(title_rect, RADIUS_MD, node_color);
        // Cover bottom corners of title bar
        renderer.fill_rect(
            Rect {
                x: nx,
                y: ny + title_h - 8.0,
                width: nw,
                height: 8.0,
            },
            node_color,
        );

        // Icon
        renderer.draw_text_raw(
            node.node_type.icon(),
            nx + 8.0,
            ny + 7.0,
            14.0,
            theme::with_alpha(theme::text(), 0.9),
        );

        // Name
        renderer.draw_text_raw(&node.name, nx + 28.0, ny + 7.0, 12.0, theme::surface());

        // Status indicator (if running)
        if let Some(ref run) = self.state.current_run
            && let Some(node_state) = run.node_states.get(&node.id)
        {
            let status_color = node_state.status.color();
            let status_label = node_state.status.label();
            let tw = renderer.measure_text(status_label, 9.0);
            renderer.draw_text_raw(
                status_label,
                nx + nw - tw.0 - 8.0,
                ny + 9.0,
                9.0,
                status_color,
            );
        }

        // Separator line
        renderer.draw_line(
            nx + 6.0,
            ny + title_h,
            nx + nw - 6.0,
            ny + title_h,
            theme::with_alpha(theme::border(), 0.5),
            1.0,
        );

        // Ports
        self.render_ports(renderer, node, nx, ny, nw, nh);
    }

    /// Render input and output ports for a node.
    fn render_ports(
        &self,
        renderer: &mut dyn Renderer,
        node: &OrchestratorNode,
        nx: f32,
        ny: f32,
        nw: f32,
        nh: f32,
    ) {
        let port_radius = 5.0;
        let title_h = 28.0;
        let content_h = nh - title_h;
        let input_count = node.inputs.len().max(1);
        let output_count = node.outputs.len().max(1);
        let spacing_in = content_h / (input_count as f32 + 1.0);
        let spacing_out = content_h / (output_count as f32 + 1.0);

        // Input ports (left side)
        for (i, port_name) in node.inputs.iter().enumerate() {
            let py = ny + title_h + spacing_in * (i as f32 + 1.0);
            let px = nx;

            renderer.fill_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                theme::accent(),
            );
            renderer.stroke_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                theme::with_alpha(theme::accent(), 0.6),
                1.0,
            );
            renderer.draw_text_raw(port_name, px + 10.0, py - 4.0, 9.0, theme::text_muted());
        }

        // Output ports (right side)
        for (i, port_name) in node.outputs.iter().enumerate() {
            let py = ny + title_h + spacing_out * (i as f32 + 1.0);
            let px = nx + nw;

            renderer.fill_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                theme::success(),
            );
            renderer.stroke_ellipse(
                Rect {
                    x: px - port_radius,
                    y: py - port_radius,
                    width: port_radius * 2.0,
                    height: port_radius * 2.0,
                },
                theme::with_alpha(theme::success(), 0.6),
                1.0,
            );
            let tw = renderer.measure_text(port_name, 9.0);
            renderer.draw_text_raw(
                port_name,
                px - 10.0 - tw.0,
                py - 4.0,
                9.0,
                theme::text_muted(),
            );
        }
    }

    /// Render a pending edge being created.
    fn render_pending_edge(
        &self,
        renderer: &mut dyn Renderer,
        node_id: &str,
        _port: &str,
        _rect: Rect,
    ) {
        let source = match self.state.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => return,
        };

        let sx = source.position.0 + source.size.0;
        let sy = source.position.1 + source.size.1 / 2.0;
        let tx = self.pointer_pos.0;
        let ty = self.pointer_pos.1;

        let dx = (tx - sx).abs();
        let cp_offset = dx * 0.4;

        // Dashed preview
        let preview_color = theme::with_alpha(theme::accent(), 0.5);
        self.draw_bezier_edge(renderer, sx, sy, tx, ty, cp_offset, preview_color, 1.5);

        // Source dot
        renderer.fill_ellipse(
            Rect {
                x: sx - 6.0,
                y: sy - 6.0,
                width: 12.0,
                height: 12.0,
            },
            theme::accent(),
        );
    }

    /// Render the toolbar at the top of the graph canvas.
    fn render_toolbar(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let toolbar_h = 36.0;
        let toolbar_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: toolbar_h,
        };

        renderer.fill_rect(toolbar_rect, theme::surface_elevated());
        renderer.draw_line(
            rect.x,
            rect.y + toolbar_h,
            rect.x + rect.width,
            rect.y + toolbar_h,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        // Title
        renderer.draw_text_raw(
            "Orchestrator",
            rect.x + 12.0,
            rect.y + 10.0,
            14.0,
            theme::text(),
        );

        // Run/Stop button
        let btn_x = rect.x + rect.width - 100.0;
        let btn_y = rect.y + 6.0;
        let btn_w = 88.0;
        let btn_h = 24.0;

        if self.state.is_executing {
            renderer.fill_rounded_rect(
                Rect {
                    x: btn_x,
                    y: btn_y,
                    width: btn_w,
                    height: btn_h,
                },
                4.0,
                theme::error_color(),
            );
            renderer.draw_text_raw("■ Stop", btn_x + 12.0, btn_y + 6.0, 11.0, theme::text());
        } else {
            renderer.fill_rounded_rect(
                Rect {
                    x: btn_x,
                    y: btn_y,
                    width: btn_w,
                    height: btn_h,
                },
                4.0,
                theme::success(),
            );
            renderer.draw_text_raw("▶ Run", btn_x + 16.0, btn_y + 6.0, 11.0, theme::text());
        }

        // Execution progress
        if self.state.is_executing {
            let total = self.engine.total_steps().max(1);
            let current = self.engine.current_step_index();
            let progress = current as f32 / total as f32;
            let bar_x = rect.x + 140.0;
            let bar_y = rect.y + 14.0;
            let bar_w = 120.0;
            let bar_h = 8.0;

            renderer.fill_rounded_rect(
                Rect {
                    x: bar_x,
                    y: bar_y,
                    width: bar_w,
                    height: bar_h,
                },
                3.0,
                theme::surface(),
            );
            renderer.fill_rounded_rect(
                Rect {
                    x: bar_x,
                    y: bar_y,
                    width: bar_w * progress,
                    height: bar_h,
                },
                3.0,
                theme::accent(),
            );
            let pct_text = format!("{}/{}", current, total);
            let tw = renderer.measure_text(&pct_text, 9.0);
            renderer.draw_text_raw(
                &pct_text,
                bar_x + bar_w / 2.0 - tw.0 / 2.0,
                bar_y - 1.0,
                9.0,
                theme::text_muted(),
            );
        }
    }

    /// Render the log panel on the right side.
    pub(crate) fn render_log_panel(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, theme::surface_elevated());

        // Left border
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        // Header
        let header_h = 32.0;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: header_h,
            },
            theme::surface_elevated(),
        );
        renderer.draw_text_raw(
            "Execution Log",
            rect.x + 10.0,
            rect.y + 9.0,
            13.0,
            theme::text(),
        );
        renderer.draw_line(
            rect.x,
            rect.y + header_h,
            rect.x + rect.width,
            rect.y + header_h,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        // Log entries
        let logs: Vec<&crate::multi_agent_orchestrator::types::OrchestratorLog> =
            match &self.state.current_run {
                Some(run) => run.logs.iter().rev().take(50).collect(),
                None => Vec::new(),
            };

        if logs.is_empty() {
            renderer.draw_text_raw(
                "No logs yet. Click ▶ Run to start.",
                rect.x + 10.0,
                rect.y + header_h + 20.0,
                11.0,
                theme::text_dim(),
            );
            return;
        }

        let mut y = rect.y + header_h + 6.0;
        let line_h = 16.0;
        let max_y = rect.y + rect.height;

        for log in logs.into_iter().rev() {
            if y + line_h > max_y {
                break;
            }

            let level_color = log.level.color();
            let level_label = log.level.label();

            // Level badge
            renderer.draw_text_raw(level_label, rect.x + 8.0, y, 9.0, level_color);

            // Message
            renderer.draw_text_raw(&log.message, rect.x + 36.0, y, 10.0, theme::text_muted());

            y += line_h;
        }
    }

    /// Render the metrics panel.
    pub(crate) fn render_metrics_panel(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Background
        renderer.fill_rect(rect, theme::surface_elevated());

        // Left border
        renderer.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        // Header
        let header_h = 32.0;
        renderer.fill_rect(
            Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: header_h,
            },
            theme::surface_elevated(),
        );
        renderer.draw_text_raw("Metrics", rect.x + 10.0, rect.y + 9.0, 13.0, theme::text());
        renderer.draw_line(
            rect.x,
            rect.y + header_h,
            rect.x + rect.width,
            rect.y + header_h,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        let mut y = rect.y + header_h + 12.0;
        let line_h = 20.0;

        if let Some(ref run) = self.state.current_run {
            // Run status
            let status_color = run.status.color();
            renderer.draw_text_raw("Status:", rect.x + 10.0, y, 11.0, theme::text_muted());
            y += line_h;
            renderer.draw_text_raw(run.status.label(), rect.x + 16.0, y, 11.0, status_color);
            y += line_h + 6.0;

            // Node count
            let total_nodes = run.node_states.len();
            let completed = run
                .node_states
                .values()
                .filter(|s| s.status == NodeExecutionStatus::Completed)
                .count();
            let failed = run
                .node_states
                .values()
                .filter(|s| s.status == NodeExecutionStatus::Failed)
                .count();
            let running = run
                .node_states
                .values()
                .filter(|s| s.status.is_active())
                .count();

            renderer.draw_text_raw("Nodes:", rect.x + 10.0, y, 11.0, theme::text_muted());
            y += line_h;
            let node_summary = format!(
                "{} total, {} done, {} running, {} failed",
                total_nodes, completed, running, failed
            );
            renderer.draw_text_raw(&node_summary, rect.x + 16.0, y, 10.0, theme::text_muted());
            y += line_h + 6.0;

            // Token usage
            renderer.draw_text_raw("Tokens:", rect.x + 10.0, y, 11.0, theme::text_muted());
            y += line_h;
            let token_text = format!(
                "In: {}  Out: {}  Total: {}",
                run.total_usage.input_tokens,
                run.total_usage.output_tokens,
                run.total_usage.total_tokens
            );
            renderer.draw_text_raw(&token_text, rect.x + 16.0, y, 10.0, theme::text_muted());
            y += line_h;

            // Cost
            renderer.draw_text_raw("Cost:", rect.x + 10.0, y, 11.0, theme::text_muted());
            y += line_h;
            let cost_text = format!("${:.4}", run.total_usage.estimated_cost);
            renderer.draw_text_raw(&cost_text, rect.x + 16.0, y, 10.0, theme::success());
            y += line_h + 6.0;

            // Per-node breakdown
            renderer.draw_text_raw("Per-Node:", rect.x + 10.0, y, 11.0, theme::text_muted());
            y += line_h;

            for node in &self.state.nodes {
                if let Some(node_state) = run.node_states.get(&node.id) {
                    if y + line_h > rect.y + rect.height {
                        break;
                    }
                    let status_color = node_state.status.color();
                    let line = format!(
                        "{}: {} ({}toks)",
                        node.name,
                        node_state.status.label(),
                        node_state.token_usage.total_tokens
                    );
                    renderer.draw_text_raw(&line, rect.x + 16.0, y, 9.0, status_color);
                    y += line_h;
                }
            }
        } else {
            renderer.draw_text_raw(
                "No run data yet.",
                rect.x + 10.0,
                y,
                11.0,
                theme::text_dim(),
            );
        }
    }

    /// Render the template library overlay panel.
    pub(crate) fn render_template_library(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let panel_w = 380.0;
        let panel_h = rect.height - 80.0;
        let panel_x = rect.x + (rect.width - panel_w) / 2.0;
        let panel_y = rect.y + 40.0;

        // Background overlay
        renderer.fill_rect(rect, theme::shadow());

        // Panel
        let panel_rect = Rect {
            x: panel_x,
            y: panel_y,
            width: panel_w,
            height: panel_h,
        };
        renderer.fill_rounded_rect(panel_rect, RADIUS_LG, theme::surface_elevated());
        renderer.stroke_rounded_rect(panel_rect, RADIUS_LG, theme::border(), 1.0);

        // Title
        renderer.draw_text_raw(
            "Template Library",
            panel_x + 16.0,
            panel_y + 14.0,
            14.0,
            theme::text(),
        );
        renderer.draw_line(
            panel_x,
            panel_y + 32.0,
            panel_x + panel_w,
            panel_y + 32.0,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        // Template list
        let templates = &self.state.templates;
        if templates.is_empty() {
            renderer.draw_text_raw(
                "No templates saved yet.",
                panel_x + 16.0,
                panel_y + 50.0,
                11.0,
                theme::text_dim(),
            );
            renderer.draw_text_raw(
                "Save your current workflow as a template.",
                panel_x + 16.0,
                panel_y + 66.0,
                10.0,
                theme::text_dim(),
            );
        } else {
            let mut y = panel_y + 42.0;
            for template in templates.iter().take(20) {
                // Template card
                let card_rect = Rect {
                    x: panel_x + 10.0,
                    y,
                    width: panel_w - 20.0,
                    height: 52.0,
                };
                renderer.fill_rounded_rect(card_rect, RADIUS_SM, theme::surface_elevated());
                renderer.draw_text_raw(
                    &template.name,
                    card_rect.x + 8.0,
                    card_rect.y + 10.0,
                    11.0,
                    theme::text(),
                );
                let desc = if template.description.len() > 40 {
                    &template.description[..40]
                } else {
                    &template.description
                };
                renderer.draw_text_raw(
                    desc,
                    card_rect.x + 8.0,
                    card_rect.y + 26.0,
                    9.0,
                    theme::text_dim(),
                );
                let meta = format!("v{} · {} nodes", template.version, template.nodes.len());
                renderer.draw_text_raw(
                    &meta,
                    card_rect.x + 8.0,
                    card_rect.y + 38.0,
                    8.0,
                    theme::text_dim(),
                );
                y += 58.0;
            }
        }

        // Close hint
        renderer.draw_text_raw(
            "Click outside to close",
            panel_x + 16.0,
            panel_y + panel_h - 16.0,
            9.0,
            theme::text_dim(),
        );
    }

    /// Render the run comparison overlay panel.
    pub(crate) fn render_run_comparison(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let panel_w = 500.0;
        let panel_h = rect.height - 80.0;
        let panel_x = rect.x + (rect.width - panel_w) / 2.0;
        let panel_y = rect.y + 40.0;

        // Background overlay
        renderer.fill_rect(rect, theme::shadow());

        // Panel
        let panel_rect = Rect {
            x: panel_x,
            y: panel_y,
            width: panel_w,
            height: panel_h,
        };
        renderer.fill_rounded_rect(panel_rect, RADIUS_LG, theme::surface_elevated());
        renderer.stroke_rounded_rect(panel_rect, RADIUS_LG, theme::border(), 1.0);

        // Title
        renderer.draw_text_raw(
            "Run Comparison",
            panel_x + 16.0,
            panel_y + 14.0,
            14.0,
            theme::text(),
        );
        renderer.draw_line(
            panel_x,
            panel_y + 32.0,
            panel_x + panel_w,
            panel_y + 32.0,
            theme::with_alpha(theme::border(), 0.6),
            1.0,
        );

        let history = &self.state.run_history;
        if history.len() < 2 {
            renderer.draw_text_raw(
                "Need at least 2 runs to compare.",
                panel_x + 16.0,
                panel_y + 50.0,
                11.0,
                theme::text_dim(),
            );
            renderer.draw_text_raw(
                "Run the workflow multiple times.",
                panel_x + 16.0,
                panel_y + 66.0,
                10.0,
                theme::text_dim(),
            );
        } else {
            let mut y = panel_y + 42.0;
            // Compare last two runs
            let run_a = &history[history.len() - 2];
            let run_b = &history[history.len() - 1];

            renderer.draw_text_raw(
                "Run A (previous)",
                panel_x + 16.0,
                y,
                11.0,
                theme::success(),
            );
            renderer.draw_text_raw("Run B (latest)", panel_x + 260.0, y, 11.0, theme::accent());
            y += 20.0;

            // Duration comparison
            renderer.draw_text_raw("Duration:", panel_x + 16.0, y, 10.0, theme::text_muted());
            let dur_a = format!("{:?}", run_a.duration);
            let dur_b = format!("{:?}", run_b.duration);
            renderer.draw_text_raw(&dur_a, panel_x + 100.0, y, 10.0, theme::text_muted());
            renderer.draw_text_raw(&dur_b, panel_x + 260.0, y, 10.0, theme::text_muted());
            y += 16.0;

            // Token comparison
            renderer.draw_text_raw("Tokens:", panel_x + 16.0, y, 10.0, theme::text_muted());
            let tok_a = format!("{}", run_a.total_usage.total_tokens);
            let tok_b = format!("{}", run_b.total_usage.total_tokens);
            renderer.draw_text_raw(&tok_a, panel_x + 100.0, y, 10.0, theme::text_muted());
            renderer.draw_text_raw(&tok_b, panel_x + 260.0, y, 10.0, theme::text_muted());
            y += 16.0;

            // Cost comparison
            renderer.draw_text_raw("Cost:", panel_x + 16.0, y, 10.0, theme::text_muted());
            let cost_a = format!("${:.4}", run_a.total_cost);
            let cost_b = format!("${:.4}", run_b.total_cost);
            renderer.draw_text_raw(&cost_a, panel_x + 100.0, y, 10.0, theme::text_muted());
            renderer.draw_text_raw(&cost_b, panel_x + 260.0, y, 10.0, theme::text_muted());
            y += 24.0;

            // Per-node comparison
            renderer.draw_text_raw(
                "Per-node token usage:",
                panel_x + 16.0,
                y,
                10.0,
                theme::text_muted(),
            );
            y += 16.0;

            for (node_id, state_a) in &run_a.node_states {
                if let Some(state_b) = run_b.node_states.get(node_id) {
                    let tok_a = state_a.token_usage.total_tokens;
                    let tok_b = state_b.token_usage.total_tokens;
                    let diff = if tok_b > tok_a {
                        format!("+{}", tok_b - tok_a)
                    } else {
                        format!("{}", tok_b - tok_a)
                    };
                    let line = format!("  {}: {} → {} ({})", node_id, tok_a, tok_b, diff);
                    let color = if tok_b > tok_a {
                        theme::warning()
                    } else {
                        theme::success()
                    };
                    renderer.draw_text_raw(&line, panel_x + 16.0, y, 9.0, color);
                    y += 14.0;
                }
            }
        }

        // Close hint
        renderer.draw_text_raw(
            "Click outside to close",
            panel_x + 16.0,
            panel_y + panel_h - 16.0,
            9.0,
            theme::text_dim(),
        );
    }

    pub(crate) fn render_output_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw("Node Output", x + 12.0, y + 8.0, 14.0, theme::text());
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                let output_text = if node.outputs.is_empty() {
                    "No output yet."
                } else {
                    &node.outputs[0]
                };
                renderer.draw_text_raw(output_text, x + 12.0, y + 30.0, 12.0, theme::text_muted());
            }
        } else {
            renderer.draw_text_raw(
                "Select a node to inspect output.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    pub(crate) fn render_message_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw("Agent Messages", x + 12.0, y + 8.0, 14.0, theme::text());
        let mut cy = y + 30.0;
        for msg in state.message_log.iter().rev().take(20) {
            let color = match msg.message_type {
                MessageType::Request => theme::accent(),
                MessageType::Response => theme::success(),
                MessageType::Error => theme::error_color(),
                MessageType::Info => theme::text_muted(),
            };
            let label = format!(
                "[{}] {} -> {}",
                msg.message_type, msg.from_node, msg.to_node
            );
            renderer.draw_text_raw(&label, x + 12.0, cy, 11.0, color);
            cy += 16.0;
            renderer.draw_text_raw(&msg.content, x + 20.0, cy, 10.0, theme::text_muted());
            cy += 20.0;
            if cy > y + h - 20.0 {
                break;
            }
        }
    }

    pub(crate) fn render_validation_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw("Validation", x + 12.0, y + 8.0, 14.0, theme::text());
        if state.validation_errors.is_empty() {
            renderer.draw_text_raw(
                "No issues found.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::success(),
            );
        } else {
            let mut cy = y + 30.0;
            for err in &state.validation_errors {
                let color = if err.is_error {
                    theme::error_color()
                } else {
                    theme::warning()
                };
                renderer.draw_text_raw(&err.message, x + 12.0, cy, 11.0, color);
                cy += 18.0;
            }
        }
    }

    pub(crate) fn render_skills_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw(
            "Skills Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        renderer.draw_text_raw(
            "Available skills:",
            x + 12.0,
            y + 30.0,
            12.0,
            theme::text_muted(),
        );
        let mut cy = y + 50.0;
        for skill in state.skill_registry.list_skills() {
            renderer.draw_text_raw(
                &format!("• {}", skill.name),
                x + 20.0,
                cy,
                11.0,
                theme::info(),
            );
            cy += 16.0;
            if cy > y + h - 20.0 {
                break;
            }
        }
    }

    pub(crate) fn render_webhook_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw(
            "Webhook Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                renderer.draw_text_raw(
                    &format!("URL: {}", node.webhook_config.url),
                    x + 12.0,
                    y + 30.0,
                    11.0,
                    theme::text_muted(),
                );
                renderer.draw_text_raw(
                    &format!("Method: {}", node.webhook_config.method),
                    x + 12.0,
                    y + 48.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        } else {
            renderer.draw_text_raw(
                "Select a webhook node.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    pub(crate) fn render_schedule_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw(
            "Schedule Configuration",
            x + 12.0,
            y + 8.0,
            14.0,
            theme::text(),
        );
        if let Some(sel) = &state.selected_node {
            if let Some(node) = state.nodes.iter().find(|n| &n.id == sel) {
                renderer.draw_text_raw(
                    &format!("Cron: {}", node.schedule_config.cron_expression),
                    x + 12.0,
                    y + 30.0,
                    11.0,
                    theme::text_muted(),
                );
                renderer.draw_text_raw(
                    &format!("Interval: {}s", node.schedule_config.interval_seconds),
                    x + 12.0,
                    y + 48.0,
                    11.0,
                    theme::text_muted(),
                );
            }
        } else {
            renderer.draw_text_raw(
                "Select a schedule node.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        }
    }

    pub(crate) fn render_recurring_panel(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_MD, theme::border_strong(), 1.0);
        renderer.draw_text_raw("Recurring Runs", x + 12.0, y + 8.0, 14.0, theme::text());
        if state.recurring_runs.is_empty() {
            renderer.draw_text_raw(
                "No recurring runs configured.",
                x + 12.0,
                y + 30.0,
                12.0,
                theme::text_muted(),
            );
        } else {
            let mut cy = y + 30.0;
            for run in &state.recurring_runs {
                renderer.draw_text_raw(
                    &format!("Every {}s ({} runs)", run.interval_seconds, run.run_count),
                    x + 12.0,
                    cy,
                    11.0,
                    theme::text_muted(),
                );
                cy += 18.0;
            }
        }
    }

    pub(crate) fn render_minimap(
        &self,
        renderer: &mut dyn Renderer,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let state = &self.state;
        renderer.fill_rounded_rect(r(x, y, w, h), RADIUS_SM, theme::surface_elevated());
        renderer.stroke_rounded_rect(r(x, y, w, h), RADIUS_SM, theme::border(), 1.0);
        let scale = 0.1;
        for node in &state.nodes {
            let nx = x + node.position.0 * scale;
            let ny = y + node.position.1 * scale;
            let nw = node.size.0 * scale;
            let nh = node.size.1 * scale;
            renderer.fill_rounded_rect(r(nx, ny, nw.max(2.0), nh.max(2.0)), 1.0, theme::text_dim());
        }
        let vp_x = x + state.viewport_offset.0 * scale;
        let vp_y = y + state.viewport_offset.1 * scale;
        let vp_w = 200.0 * state.viewport_zoom * scale;
        let vp_h = 150.0 * state.viewport_zoom * scale;
        renderer.stroke_rounded_rect(
            r(vp_x, vp_y, vp_w, vp_h),
            RADIUS_XS,
            theme::with_alpha(theme::success(), 0.5),
            1.0,
        );
    }
}

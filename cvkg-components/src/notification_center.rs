use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A slide-out panel that displays notifications grouped by app.
///
/// It visually emulates the macOS Notification Center, sliding in from the right edge
/// of the screen when `notification_center_visible` is true.
#[derive(Clone, Default)]
pub struct NotificationCenterPanel;

impl NotificationCenterPanel {
    /// Creates a new `NotificationCenterPanel`.
    pub fn new() -> Self {
        Self
    }
}

/// Constant width of the Notification Center panel in logical pixels.
const PANEL_WIDTH: f32 = 320.0;
/// Margin from the edges of the panel.
const MARGIN: f32 = 16.0;
/// Spacing between notification items.
const ITEM_SPACING: f32 = 10.0;
/// Corner radius for notification cards.
const CARD_RADIUS: f32 = 8.0;

impl View for NotificationCenterPanel {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    /// Renders the slide-out panel using dynamic animation and layout calculations.
    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let state = cvkg_core::load_system_state();
        let anim_hash = 99999;
        let target = if state.notification_center_visible {
            1.0
        } else {
            0.0
        };
        let mut t_val = 0.0;

        // Initialize SpringSolver if it doesn't exist yet
        {
            if state
                .get_component_state::<cvkg_anim::SpringSolver>(anim_hash)
                .is_none()
            {
                cvkg_core::update_system_state(|st| {
                    let mut new_st = st.clone();
                    new_st.set_component_state(
                        anim_hash,
                        cvkg_anim::SpringSolver::new(
                            cvkg_anim::SpringParams::snappy(),
                            target,
                            0.0,
                        ),
                    );
                    new_st
                });
            }
        }

        // Tick solver and retrieve current interpolation value
        {
            let s = cvkg_core::load_system_state();
            if let Some(solver_arc) = s.get_component_state::<cvkg_anim::SpringSolver>(anim_hash)
            {
                let mut solver = solver_arc.write().unwrap_or_else(|e| {
                    log::warn!("Lock poisoned, recovering...");
                    e.into_inner()
                });
                solver.set_target(target);
                t_val = solver.tick(renderer.delta_time());
            }
        }

        // Do not render anything if completely closed
        if t_val <= 0.001 {
            return;
        }

        let slide_offset = (1.0 - t_val) * PANEL_WIDTH;
        let panel_rect = Rect {
            x: rect.x + rect.width - PANEL_WIDTH + slide_offset,
            y: rect.y,
            width: PANEL_WIDTH,
            height: rect.height,
        };

        renderer.push_opacity(t_val);
        renderer.push_transform([slide_offset, 0.0], [1.0, 1.0], 0.0);
        renderer.push_vnode(panel_rect, "NotificationCenterPanel");

        // 1. Bifrost glass background
        renderer.bifrost(panel_rect, 20.0, 1.5, 0.96);

        // 2. Translucent panel fill
        renderer.fill_rounded_rect(panel_rect, 0.0, theme::with_alpha(theme::surface_elevated(), 0.85));

        // 3. Leading border/separator line
        renderer.draw_line(
            panel_rect.x,
            panel_rect.y,
            panel_rect.x,
            panel_rect.y + panel_rect.height,
            theme::border(),
            1.0,
        );

        // 4. Header title: "Notification Center"
        renderer.draw_text(
            "Notification Center",
            panel_rect.x + MARGIN,
            panel_rect.y + MARGIN + 4.0,
            16.0,
            theme::text(),
        );

        // 5. Render Clear All / Close control buttons
        let mut curr_y = panel_rect.y + MARGIN + 32.0;

        let active_notifs: Vec<_> = state
            .notifications
            .iter()
            .filter(|n| !n.dismissed)
            .cloned()
            .collect();

        if !active_notifs.is_empty() {
            // Draw "Clear All" button
            let clear_text = "Clear All";
            let clear_x = panel_rect.x + panel_rect.width - MARGIN - 60.0;
            let clear_y = panel_rect.y + MARGIN + 4.0;
            renderer.draw_text(clear_text, clear_x, clear_y, 12.0, theme::text_muted());

            renderer.register_handler(
                "pointerclick",
                Arc::new(move |event| {
                    if let cvkg_core::Event::PointerClick { x, y, .. } = event
                        && x >= clear_x
                        && x <= clear_x + 60.0
                        && y >= clear_y
                        && y <= clear_y + 16.0
                    {
                        cvkg_core::update_system_state(|st| {
                            let mut new_st = st.clone();
                            for n in &mut new_st.notifications {
                                n.dismissed = true;
                            }
                            new_st
                        });
                    }
                }),
            );

            // Group notifications by app_name
            let mut groups: BTreeMap<String, Vec<cvkg_core::Notification>> = BTreeMap::new();
            for notif in active_notifs {
                let app = notif
                    .app_name
                    .clone()
                    .unwrap_or_else(|| "System".to_string());
                groups.entry(app).or_default().push(notif);
            }

            // Draw grouped items
            for (app, notifs) in groups {
                // Group Header
                renderer.draw_text(
                    &app.to_uppercase(),
                    panel_rect.x + MARGIN,
                    curr_y,
                    10.0,
                    theme::text_dim(),
                );
                curr_y += 18.0;

                for notif in notifs {
                    let card_height = if notif.actions.is_empty() { 68.0 } else { 94.0 };
                    let card_rect = Rect {
                        x: panel_rect.x + MARGIN,
                        y: curr_y,
                        width: PANEL_WIDTH - MARGIN * 2.0,
                        height: card_height,
                    };

                    // Draw notification card
                    renderer.fill_rounded_rect(card_rect, CARD_RADIUS, [1.0, 1.0, 1.0, 0.05]);
                    renderer.stroke_rounded_rect(card_rect, CARD_RADIUS, theme::border(), 0.5);

                    // Title
                    renderer.draw_text(
                        &notif.title,
                        card_rect.x + 8.0,
                        card_rect.y + 8.0,
                        13.0,
                        theme::text(),
                    );

                    // Body description text
                    let display_body = if notif.body.len() > 60 {
                        format!("{}...", &notif.body[..57])
                    } else {
                        notif.body.clone()
                    };
                    renderer.draw_text(
                        &display_body,
                        card_rect.x + 8.0,
                        card_rect.y + 24.0,
                        11.0,
                        theme::text_muted(),
                    );

                    // Individual dismissal close button (X) in top right of card
                    let close_size = 14.0;
                    let close_x = card_rect.x + card_rect.width - 18.0;
                    let close_y = card_rect.y + 8.0;
                    renderer.draw_line(
                        close_x,
                        close_y,
                        close_x + 8.0,
                        close_y + 8.0,
                        theme::text_dim(),
                        1.0,
                    );
                    renderer.draw_line(
                        close_x + 8.0,
                        close_y,
                        close_x,
                        close_y + 8.0,
                        theme::text_dim(),
                        1.0,
                    );

                    let notif_id_dismiss = notif.id.clone();
                    renderer.register_handler(
                        "pointerclick",
                        Arc::new(move |event| {
                            if let cvkg_core::Event::PointerClick { x, y, .. } = event
                                && x >= close_x - 4.0
                                && x <= close_x + close_size
                                && y >= close_y - 4.0
                                && y <= close_y + close_size
                            {
                                let _ = cvkg_core::get_notification_handler()
                                    .dismiss(&notif_id_dismiss);
                            }
                        }),
                    );

                    // Render notification action buttons if available
                    if !notif.actions.is_empty() {
                        let mut btn_x = card_rect.x + 8.0;
                        let btn_y = card_rect.y + 44.0;
                        let btn_h = 20.0;

                        for action in &notif.actions {
                            let btn_w = 70.0;
                            let btn_rect = Rect {
                                x: btn_x,
                                y: btn_y,
                                width: btn_w,
                                height: btn_h,
                            };

                            let fill_col = if action.is_destructive {
                                [0.8, 0.2, 0.2, 0.3]
                            } else {
                                [1.0, 1.0, 1.0, 0.08]
                            };
                            renderer.fill_rounded_rect(btn_rect, 4.0, fill_col);
                            renderer.stroke_rounded_rect(btn_rect, 4.0, theme::border(), 0.5);

                            // Action button text
                            let text_col = if action.is_destructive {
                                [0.95, 0.3, 0.3, 1.0]
                            } else {
                                theme::text()
                            };
                            renderer.draw_text(
                                &action.title,
                                btn_x + 6.0,
                                btn_y + 4.0,
                                10.0,
                                text_col,
                            );

                            let notif_id_act = notif.id.clone();
                            let action_id = action.id.clone();
                            renderer.register_handler(
                                "pointerclick",
                                Arc::new(move |event| {
                                    if let cvkg_core::Event::PointerClick { x, y, .. } = event
                                        && x >= btn_rect.x
                                        && x <= btn_rect.x + btn_rect.width
                                        && y >= btn_rect.y
                                        && y <= btn_rect.y + btn_rect.height
                                    {
                                        log::info!(
                                            "Notification action triggered: ID={}, Action={}",
                                            notif_id_act,
                                            action_id
                                        );
                                        let _ = cvkg_core::get_notification_handler()
                                            .dismiss(&notif_id_act);
                                    }
                                }),
                            );

                            btn_x += btn_w + 8.0;
                        }
                    }

                    curr_y += card_height + ITEM_SPACING;
                }
                curr_y += 8.0;
            }
        } else {
            // Draw empty state
            renderer.draw_text(
                "No Notifications",
                panel_rect.x + MARGIN,
                curr_y + 16.0,
                13.0,
                theme::text_dim(),
            );
        }

        renderer.pop_vnode();
        renderer.pop_transform();
        renderer.pop_opacity();
    }
}

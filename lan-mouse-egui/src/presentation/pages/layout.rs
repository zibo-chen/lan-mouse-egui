use eframe::egui::{
    self, Color32, Context, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, Vec2, vec2,
};
use lan_mouse_ipc::FrontendRequest;

use crate::{
    application::LanMouseDesktopApp,
    domain::ActiveTheme,
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, help_text, section_heading,
        tinted_frame, toggle_switch,
    },
};

/// Render the Screen Layout Manager page.
///
/// Layout: main canvas (left) + screen list sidebar (right) + bottom controls bar.
pub fn render(app: &mut LanMouseDesktopApp, ui: &mut Ui, ctx: &Context) {
    let theme = app.theme().clone();
    let text = app.text();

    // Header
    section_heading(
        ui,
        text.label_screen_layout,
        text.nav_layout_subtitle,
        &theme,
    );
    ui.add_space(6.0);

    // Status line
    let n_devices = app.layout.device_count();
    let n_screens = app.layout.screen_count();
    help_text(
        ui,
        &format!(
            "{} {} · {} {}",
            n_devices, text.layout_status_devices, n_screens, text.layout_status_screens,
        ),
        &theme,
    );
    ui.add_space(6.0);

    // Main area: Canvas + detected-screens sidebar
    let avail = ui.available_size();
    let item_spacing = ui.spacing().item_spacing.x;
    // elevated_frame inner_margin(10)*2 + tinted_frame inner_margin(8)*2 + gap + item_spacing
    let frame_overhead = 20.0 + 16.0 + 4.0 + item_spacing;
    let usable_w = (avail.x - frame_overhead).max(200.0);
    let sidebar_w = (usable_w * 0.22).clamp(140.0, 200.0);
    let canvas_w = usable_w - sidebar_w;
    let canvas_h = (avail.y - 44.0).max(200.0); // reserve 44 for bottom controls

    ui.horizontal(|ui| {
        // --- Canvas ---
        render_canvas(app, ui, ctx, vec2(canvas_w, canvas_h));

        ui.add_space(4.0);

        // --- Detected Screens sidebar ---
        render_screen_list(app, ui, vec2(sidebar_w, canvas_h));
    });

    ui.add_space(6.0);

    // Bottom controls bar
    render_controls(app, ui);
}

// ─── Canvas ───

fn render_canvas(app: &mut LanMouseDesktopApp, ui: &mut Ui, _ctx: &Context, size: Vec2) {
    let theme = app.theme().clone();

    elevated_frame(&theme).show(ui, |ui| {
        let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
        let canvas_origin = resp.rect.min;
        let canvas_rect = resp.rect;

        // Draw grid
        if app.layout.show_grid {
            draw_grid(&painter, canvas_rect, &theme);
        }

        let scale = app.layout.scale;

        // Determine if we're starting a new drag
        if resp.drag_started() {
            if let Some(pointer) = resp.interact_pointer_pos() {
                // Find which screen was clicked (iterate in reverse for z-order)
                let mut hit = None;
                for (i, screen) in app.layout.screens.iter().enumerate().rev() {
                    let r = screen.canvas_rect(scale).translate(canvas_origin.to_vec2());
                    if r.contains(pointer) {
                        hit = Some(i);
                        break;
                    }
                }
                if let Some(idx) = hit {
                    let r = app.layout.screens[idx]
                        .canvas_rect(scale)
                        .translate(canvas_origin.to_vec2());
                    app.layout.dragging = Some(idx);
                    app.layout.drag_offset = pointer - r.min;
                }
            }
        }

        // Drag in progress
        if let Some(idx) = app.layout.dragging {
            if resp.dragged() {
                if let Some(pointer) = resp.interact_pointer_pos() {
                    let raw_x = pointer.x - canvas_origin.x - app.layout.drag_offset.x;
                    let raw_y = pointer.y - canvas_origin.y - app.layout.drag_offset.y;
                    let next_x = app.layout.snap(raw_x);
                    let next_y = app.layout.snap(raw_y);
                    let delta_x = next_x - app.layout.screens[idx].x;
                    let delta_y = next_y - app.layout.screens[idx].y;
                    let dragged_client = app.layout.screens[idx].id.client;
                    for screen in app
                        .layout
                        .screens
                        .iter_mut()
                        .filter(|screen| screen.id.client == dragged_client)
                    {
                        screen.x += delta_x;
                        screen.y += delta_y;
                    }
                    app.layout.dirty = true;
                }
            }
            if resp.drag_stopped() {
                app.layout.dragging = None;
                // Auto-save layout on drag stop
                apply_layout(app);
            }
        }

        // Draw each screen rectangle
        for (i, screen) in app.layout.screens.iter().enumerate() {
            let r = screen.canvas_rect(scale).translate(canvas_origin.to_vec2());
            let is_local = screen.id.client.is_none();
            let is_dragging = app.layout.dragging == Some(i);

            let fill = if is_local {
                theme.palette.accent_soft
            } else {
                theme.palette.surface_tint
            };

            let stroke_color = if is_dragging {
                theme.palette.accent
            } else if is_local {
                theme.palette.accent_secondary
            } else {
                theme.palette.border_strong
            };

            let stroke_w = if is_dragging { 2.0 } else { 1.0 };

            painter.rect_filled(r, CornerRadius::same(4), fill);
            painter.rect_stroke(
                r,
                CornerRadius::same(4),
                Stroke::new(stroke_w, stroke_color),
                StrokeKind::Inside,
            );

            // Screen label (device name + resolution)
            let label = &screen.label;
            let res = screen.info.resolution_label();

            let center = r.center();
            let label_galley = painter.layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(11.0),
                theme.palette.text_primary,
            );
            let res_galley = painter.layout_no_wrap(
                res.clone(),
                egui::FontId::proportional(9.0),
                theme.palette.text_secondary,
            );

            let total_h = label_galley.size().y + res_galley.size().y + 2.0;
            let y_start = center.y - total_h * 0.5;

            // Only draw text if rectangle is big enough
            if r.width() > 40.0 && r.height() > 30.0 {
                let label_h = label_galley.size().y;
                let label_w = label_galley.size().x;
                let res_w = res_galley.size().x;
                painter.galley(
                    Pos2::new((center.x - label_w * 0.5).max(r.left() + 2.0), y_start),
                    label_galley,
                    Color32::TRANSPARENT,
                );
                painter.galley(
                    Pos2::new(
                        (center.x - res_w * 0.5).max(r.left() + 2.0),
                        y_start + label_h + 2.0,
                    ),
                    res_galley,
                    Color32::TRANSPARENT,
                );
            }

            // Primary badge
            if screen.info.primary && r.width() > 60.0 {
                let badge = painter.layout_no_wrap(
                    "★".to_string(),
                    egui::FontId::proportional(10.0),
                    theme.palette.accent,
                );
                painter.galley(
                    Pos2::new(r.left() + 4.0, r.top() + 3.0),
                    badge,
                    Color32::TRANSPARENT,
                );
            }
        }
    });
}

fn draw_grid(painter: &egui::Painter, rect: Rect, theme: &ActiveTheme) {
    let grid = 16.0;
    let color = theme.palette.border.gamma_multiply(0.3);
    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(0.5, color),
        );
        x += grid;
    }
    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(0.5, color),
        );
        y += grid;
    }
}

// ─── Detected screens sidebar ───

fn render_screen_list(app: &mut LanMouseDesktopApp, ui: &mut Ui, size: Vec2) {
    let theme = app.theme().clone();
    let text = app.text();

    tinted_frame(&theme).show(ui, |ui| {
        ui.set_min_size(size - vec2(16.0, 16.0)); // account for frame margins
        card_title(ui, text.label_detected_screens, &theme);
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for screen in &app.layout.screens {
                    let is_local = screen.id.client.is_none();

                    Frame::new()
                        .fill(if is_local {
                            theme.palette.accent_soft.gamma_multiply(0.3)
                        } else {
                            theme.palette.surface_raised
                        })
                        .stroke(Stroke::new(1.0, theme.palette.border))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&screen.info.name)
                                    .size(12.0)
                                    .strong()
                                    .color(theme.palette.text_primary),
                            );

                            ui.label(
                                RichText::new(&screen.device_label)
                                    .size(10.0)
                                    .color(theme.palette.text_secondary),
                            );

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(screen.info.resolution_label())
                                        .size(10.0)
                                        .color(theme.palette.text_secondary),
                                );
                                if screen.info.primary {
                                    ui.label(
                                        RichText::new(format!("★ {}", text.label_primary))
                                            .size(10.0)
                                            .color(theme.palette.accent),
                                    );
                                }
                            });
                        });

                    ui.add_space(4.0);
                }
            });
    });
}

// ─── Bottom controls ───

fn render_controls(app: &mut LanMouseDesktopApp, ui: &mut Ui) {
    let theme = app.theme().clone();
    let text = app.text();

    ui.horizontal(|ui| {
        // Snap to grid
        let mut snap = app.layout.snap_to_grid;
        if toggle_switch(ui, &mut snap, text.label_snap_to_grid, &theme) {
            app.layout.snap_to_grid = snap;
        }
        ui.label(
            RichText::new(text.label_snap_to_grid)
                .size(12.0)
                .color(theme.palette.text_primary),
        );

        ui.add_space(12.0);

        // Show grid
        let mut grid = app.layout.show_grid;
        if toggle_switch(ui, &mut grid, text.label_show_grid, &theme) {
            app.layout.show_grid = grid;
        }
        ui.label(
            RichText::new(text.label_show_grid)
                .size(12.0)
                .color(theme.palette.text_primary),
        );

        // Right-justify the action buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(action_button(text.action_cancel, ButtonKind::Ghost, &theme))
                .clicked()
            {
                // Mark for rebuild on next frame
                app.layout.dirty = false;
                app.layout.screens.clear(); // will trigger rebuild in poll_events
            }

            if ui
                .add_enabled(
                    app.layout.dirty,
                    action_button(text.action_apply, ButtonKind::Primary, &theme),
                )
                .clicked()
            {
                apply_layout(app);
            }

            if ui
                .add(action_button(
                    text.action_auto_arrange,
                    ButtonKind::Secondary,
                    &theme,
                ))
                .clicked()
            {
                app.layout.auto_arrange();
                apply_layout(app);
            }
        });
    });
}

/// Send current layout to the backend: positions, layout rects, and save.
fn apply_layout(app: &mut LanMouseDesktopApp) {
    // Derive and send adjacent edge positions for capture barrier management
    let positions = app.layout.derive_client_positions();
    for (handle, pos_list) in positions {
        if let Some(client) = app.workspace.clients.get_mut(&handle) {
            client.position = pos_list.first().copied().unwrap_or_default();
        }
        app.send_request(FrontendRequest::UpdatePositions(handle, pos_list));
    }

    // Send layout rects for each client (2D coordinate mapping)
    let client_rects = app.layout.derive_client_layout_rects();
    for (handle, rects) in client_rects {
        app.send_request(FrontendRequest::UpdateLayout(handle, rects));
    }

    // Send local layout rects
    let local_rects = app.layout.derive_local_layout_rects();
    app.send_request(FrontendRequest::UpdateLocalLayout(local_rects));

    // Persist to disk
    app.send_request(FrontendRequest::SaveConfiguration);

    app.layout.dirty = false;
}

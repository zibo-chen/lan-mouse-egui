use eframe::egui::{self, ComboBox, Grid, Layout, ScrollArea, Slider, TextEdit, Vec2};
use lan_mouse_ipc::FrontendRequest;

use crate::{
    application::LanMouseDesktopApp,
    domain::{ClientConnectivity, parse_port_input},
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, empty_state, field_label, help_text,
        ip_chip, status_pill, tinted_frame,
    },
};

pub fn render(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    if app.workspace.clients.is_empty() {
        empty_state(ui, text.empty_clients, text.empty_clients_hint, theme);
        ui.add_space(8.0);
        if ui
            .add(action_button(
                text.action_create_first_client,
                ButtonKind::Primary,
                theme,
            ))
            .clicked()
        {
            app.send_request(FrontendRequest::Create);
        }
        return;
    }

    let total_w = ui.available_width();
    let list_w = (total_w * 0.32).clamp(200.0, 300.0);

    ui.horizontal_top(|ui| {
        ui.allocate_ui(Vec2::new(list_w, ui.available_height()), |ui| {
            render_roster(app, ui);
        });
        ui.add_space(8.0);
        ui.vertical(|ui| {
            render_editor(app, ui);
        });
    });
}

fn render_roster(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    elevated_frame(theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            card_title(ui, text.label_clients, theme);
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(action_button(
                        text.action_add_client,
                        ButtonKind::Primary,
                        theme,
                    ))
                    .clicked()
                {
                    app.send_request(FrontendRequest::Create);
                }
            });
        });

        ui.add_space(4.0);

        ScrollArea::vertical()
            .max_height(500.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for handle in app.workspace.clients.keys().copied().collect::<Vec<_>>() {
                    let Some(client) = app.workspace.clients.get(&handle) else {
                        continue;
                    };
                    let selected = app.current_client() == Some(handle);
                    let fill = if selected {
                        theme.palette.nav_active_fill
                    } else {
                        theme.palette.surface_tint
                    };
                    let stroke_color = if selected {
                        theme.palette.nav_active_stroke
                    } else {
                        theme.palette.border
                    };

                    let response = egui::Frame::new()
                        .fill(fill)
                        .stroke(egui::Stroke::new(1.0, stroke_color))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(8)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(client.title())
                                    .strong()
                                    .color(theme.palette.text_primary),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}:{} · {}",
                                    client.hostname.as_deref().unwrap_or("-"),
                                    client.port_label(),
                                    text.position(client.position)
                                ))
                                .size(11.0)
                                .color(theme.palette.text_secondary),
                            );
                            let (status_label, color) = match client.status() {
                                ClientConnectivity::Reachable => {
                                    (text.status_reachable, theme.palette.success)
                                }
                                ClientConnectivity::Unresolved => {
                                    (text.status_unresolved, theme.palette.danger)
                                }
                                ClientConnectivity::Resolving => {
                                    (text.status_resolving, theme.palette.warning)
                                }
                            };
                            ui.horizontal(|ui| {
                                status_pill(ui, status_label, color, theme);
                                if client.active {
                                    status_pill(
                                        ui,
                                        text.status_active,
                                        theme.palette.accent,
                                        theme,
                                    );
                                }
                            });
                        })
                        .response;

                    if response.clicked() {
                        app.selected_client = Some(handle);
                    }
                    ui.add_space(4.0);
                }
            });
    });
}

fn render_editor(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();
    let Some(handle) = app.current_client() else {
        empty_state(
            ui,
            text.empty_selected_client,
            text.empty_selected_client_hint,
            theme,
        );
        return;
    };

    let mut requests = Vec::new();
    let mut delete_client = false;

    elevated_frame(theme).show(ui, |ui| {
        let Some(client) = app.workspace.clients.get_mut(&handle) else {
            empty_state(
                ui,
                text.empty_selected_client,
                text.empty_selected_client_hint,
                theme,
            );
            return;
        };

        // Header
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(client.title())
                    .size(18.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let response = ui.checkbox(&mut client.active, text.label_route_input);
                if response.changed() {
                    requests.push(FrontendRequest::Activate(handle, client.active));
                }
            });
        });

        ui.add_space(6.0);

        // Network section
        tinted_frame(theme).show(ui, |ui| {
            card_title(ui, text.label_network, theme);
            ui.add_space(4.0);
            Grid::new(("client-net", handle))
                .num_columns(2)
                .spacing(Vec2::new(12.0, 4.0))
                .show(ui, |ui| {
                    field_label(ui, text.label_hostname, theme);
                    let resp = ui.add(
                        TextEdit::singleline(&mut client.hostname_input)
                            .desired_width(200.0)
                            .hint_text("host.local"),
                    );
                    client.hostname_dirty = resp.has_focus();
                    if resp.changed() {
                        let hostname = Some(client.hostname_input.trim().to_string())
                            .filter(|v| !v.is_empty());
                        requests.push(FrontendRequest::UpdateHostname(handle, hostname));
                    }
                    ui.end_row();

                    field_label(ui, text.label_port, theme);
                    let resp = ui.add(
                        TextEdit::singleline(&mut client.port_input)
                            .desired_width(100.0)
                            .hint_text(app.workspace.port.to_string()),
                    );
                    client.port_dirty = resp.has_focus();
                    if resp.changed() {
                        requests.push(FrontendRequest::UpdatePort(
                            handle,
                            parse_port_input(&client.port_input),
                        ));
                    }
                    ui.end_row();

                    field_label(ui, text.label_known_ips, theme);
                    if client.ips.is_empty() {
                        help_text(ui, text.empty_addresses, theme);
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            for ip in &client.ips {
                                ip_chip(ui, &ip.to_string(), theme);
                            }
                        });
                    }
                    ui.end_row();
                });
        });

        ui.add_space(6.0);

        // Input profile section
        tinted_frame(theme).show(ui, |ui| {
            card_title(ui, text.label_input_profile, theme);
            ui.add_space(4.0);
            Grid::new(("client-profile", handle))
                .num_columns(2)
                .spacing(Vec2::new(12.0, 4.0))
                .show(ui, |ui| {
                    field_label(ui, text.label_remote_platform, theme);
                    let prev = client.input_profile.source_platform;
                    ComboBox::from_id_salt(("platform", handle))
                        .selected_text(text.platform(client.input_profile.source_platform))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for p in [
                                lan_mouse_ipc::ClientPlatform::Unknown,
                                lan_mouse_ipc::ClientPlatform::Windows,
                                lan_mouse_ipc::ClientPlatform::Macos,
                                lan_mouse_ipc::ClientPlatform::Linux,
                            ] {
                                ui.selectable_value(
                                    &mut client.input_profile.source_platform,
                                    p,
                                    text.platform(p),
                                );
                            }
                        });
                    if prev != client.input_profile.source_platform {
                        requests.push(FrontendRequest::UpdateInputProfile(
                            handle,
                            client.input_profile.clone(),
                        ));
                    }
                    ui.end_row();

                    field_label(ui, text.label_shortcut_style, theme);
                    let prev = client.input_profile.shortcut_mode;
                    ComboBox::from_id_salt(("shortcut", handle))
                        .selected_text(text.shortcut_mode(client.input_profile.shortcut_mode))
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for m in [
                                lan_mouse_ipc::ShortcutMode::Physical,
                                lan_mouse_ipc::ShortcutMode::SourceNative,
                            ] {
                                ui.selectable_value(
                                    &mut client.input_profile.shortcut_mode,
                                    m,
                                    text.shortcut_mode(m),
                                );
                            }
                        });
                    if prev != client.input_profile.shortcut_mode {
                        requests.push(FrontendRequest::UpdateInputProfile(
                            handle,
                            client.input_profile.clone(),
                        ));
                    }
                    ui.end_row();

                    field_label(ui, text.label_scroll_style, theme);
                    let prev = client.input_profile.scroll_mode;
                    ComboBox::from_id_salt(("scroll", handle))
                        .selected_text(text.scroll_mode(client.input_profile.scroll_mode))
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for m in [
                                lan_mouse_ipc::ScrollMode::Physical,
                                lan_mouse_ipc::ScrollMode::TargetNative,
                            ] {
                                ui.selectable_value(
                                    &mut client.input_profile.scroll_mode,
                                    m,
                                    text.scroll_mode(m),
                                );
                            }
                        });
                    if prev != client.input_profile.scroll_mode {
                        requests.push(FrontendRequest::UpdateInputProfile(
                            handle,
                            client.input_profile.clone(),
                        ));
                    }
                    ui.end_row();

                    field_label(ui, text.label_horizontal_scroll, theme);
                    if ui
                        .add(
                            Slider::new(&mut client.input_profile.scroll_scale_x, 0.25..=3.0)
                                .step_by(0.05),
                        )
                        .changed()
                    {
                        requests.push(FrontendRequest::UpdateInputProfile(
                            handle,
                            client.input_profile.clone(),
                        ));
                    }
                    ui.end_row();

                    field_label(ui, text.label_vertical_scroll, theme);
                    if ui
                        .add(
                            Slider::new(&mut client.input_profile.scroll_scale_y, 0.25..=3.0)
                                .step_by(0.05),
                        )
                        .changed()
                    {
                        requests.push(FrontendRequest::UpdateInputProfile(
                            handle,
                            client.input_profile.clone(),
                        ));
                    }
                    ui.end_row();
                });

            ui.add_space(4.0);
            help_text(ui, text.input_profile_hint, theme);
        });

        ui.add_space(6.0);

        // Actions
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(action_button(
                    text.action_resolve_dns,
                    ButtonKind::Primary,
                    theme,
                ))
                .clicked()
            {
                requests.push(FrontendRequest::ResolveDns(handle));
            }
            if ui
                .add(action_button(
                    text.action_delete_client,
                    ButtonKind::Danger,
                    theme,
                ))
                .clicked()
            {
                delete_client = true;
            }
        });
    });

    if delete_client {
        app.send_request(FrontendRequest::Delete(handle));
    }
    for request in requests {
        app.send_request(request);
    }
}

use eframe::egui::{self, Context, Grid, Layout, Vec2};
use lan_mouse_ipc::{FrontendRequest, Status};

use crate::{
    application::LanMouseDesktopApp,
    domain::{ClientConnectivity, NavigationPage, OverviewTab},
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, help_text, metric_card,
        section_heading, tab_button, table_cell, table_header,
    },
};

pub fn render(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, _ctx: &Context) {
    let theme = &app.theme().clone();
    let text = app.text();

    section_heading(
        ui,
        text.nav_label(NavigationPage::Overview),
        text.nav_subtitle(NavigationPage::Overview),
        theme,
    );
    ui.add_space(6.0);

    // ── Metrics row ──
    ui.horizontal_wrapped(|ui| {
        metric_card(
            ui,
            "",
            text.label_connected_count,
            &app.workspace.client_count().to_string(),
            theme.palette.accent,
            theme,
        );
        metric_card(
            ui,
            "",
            text.label_routes_count,
            &app.workspace.active_routes().to_string(),
            theme.palette.success,
            theme,
        );
        metric_card(
            ui,
            "",
            text.label_trusted_devices,
            &app.workspace.trusted_devices().to_string(),
            theme.palette.warning,
            theme,
        );
        metric_card(
            ui,
            "",
            text.label_listen_port,
            &app.workspace.port.to_string(),
            theme.palette.accent_secondary,
            theme,
        );
    });

    ui.add_space(6.0);

    // ── Service status ──
    elevated_frame(theme).show(ui, |ui| {
        card_title(ui, text.label_service_controls, theme);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let capture_on = app.workspace.capture_status == Status::Enabled;
            let capture_color = if capture_on {
                theme.palette.success
            } else {
                theme.palette.danger
            };
            ui.label(egui::RichText::new("●").color(capture_color));
            ui.label(
                egui::RichText::new(text.label_capture)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.label(if capture_on {
                text.status_enabled
            } else {
                text.status_disabled
            });

            ui.add_space(16.0);

            let emu_on = app.workspace.emulation_status == Status::Enabled;
            let emu_color = if emu_on {
                theme.palette.success
            } else {
                theme.palette.danger
            };
            ui.label(egui::RichText::new("●").color(emu_color));
            ui.label(
                egui::RichText::new(text.label_emulation)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.label(if emu_on {
                text.status_enabled
            } else {
                text.status_disabled
            });
        });
    });

    ui.add_space(6.0);

    // ── Tabbed detail section ──
    elevated_frame(theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            if tab_button(
                ui,
                text.label_device_list,
                app.preferences.overview_tab == OverviewTab::DeviceList,
                theme,
            )
            .clicked()
            {
                app.preferences.overview_tab = OverviewTab::DeviceList;
            }
            if tab_button(
                ui,
                text.label_security_events,
                app.preferences.overview_tab == OverviewTab::SecurityEvents,
                theme,
            )
            .clicked()
            {
                app.preferences.overview_tab = OverviewTab::SecurityEvents;
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(action_button(
                        text.action_scan,
                        ButtonKind::Secondary,
                        theme,
                    ))
                    .clicked()
                {
                    for handle in app.workspace.clients.keys().copied().collect::<Vec<_>>() {
                        app.send_request(FrontendRequest::ResolveDns(handle));
                    }
                }
            });
        });
        ui.add_space(4.0);

        match app.preferences.overview_tab {
            OverviewTab::DeviceList => render_device_list(app, ui),
            OverviewTab::SecurityEvents => render_events_list(app, ui),
        }
    });
}

fn render_device_list(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    if app.workspace.clients.is_empty() {
        help_text(ui, text.empty_clients, theme);
        return;
    }

    Grid::new("overview-devices")
        .num_columns(4)
        .spacing(Vec2::new(12.0, 4.0))
        .striped(true)
        .show(ui, |ui| {
            table_header(ui, text.col_name, theme);
            table_header(ui, text.col_ip_port, theme);
            table_header(ui, text.label_screen_edge, theme);
            table_header(ui, text.label_status, theme);
            ui.end_row();

            for client in app.workspace.clients.values() {
                let status_color = match client.status() {
                    ClientConnectivity::Reachable => theme.palette.success,
                    ClientConnectivity::Resolving => theme.palette.warning,
                    ClientConnectivity::Unresolved => theme.palette.danger,
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").size(10.0).color(status_color));
                    table_cell(ui, &client.title(), theme);
                });

                let ip_text = if client.ips.is_empty() {
                    "-".to_string()
                } else {
                    format!("{}:{}", client.ips[0], client.port)
                };
                table_cell(ui, &ip_text, theme);
                table_cell(ui, text.position(client.position), theme);

                let status_label = match client.status() {
                    ClientConnectivity::Reachable => text.status_reachable,
                    ClientConnectivity::Resolving => text.status_resolving,
                    ClientConnectivity::Unresolved => text.status_unresolved,
                };
                table_cell(ui, status_label, theme);
                ui.end_row();
            }
        });
}

fn render_events_list(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    if app.workspace.network_events.is_empty() {
        help_text(ui, text.recent_security_hint, theme);
        return;
    }

    Grid::new("overview-events")
        .num_columns(3)
        .spacing(Vec2::new(12.0, 4.0))
        .striped(true)
        .show(ui, |ui| {
            table_header(ui, text.col_timestamp, theme);
            table_header(ui, text.col_event_desc, theme);
            table_header(ui, text.col_name, theme);
            ui.end_row();

            for event in app.workspace.network_events.iter().rev().take(50) {
                table_cell(ui, &event.timestamp_label, theme);
                table_cell(ui, &event.description, theme);
                table_cell(ui, &event.source, theme);
                ui.end_row();
            }
        });
}

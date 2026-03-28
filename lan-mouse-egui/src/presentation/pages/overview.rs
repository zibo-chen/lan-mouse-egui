use eframe::egui::{self, Context, Grid, Layout, Vec2};
use lan_mouse_ipc::{FrontendRequest, Status};

use crate::{
    application::LanMouseDesktopApp,
    domain::{ClientConnectivity, OverviewTab},
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, help_text, metric_card,
        section_heading, tab_button, table_cell, table_header, tinted_frame, toggle_switch,
    },
};

pub fn render(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, ctx: &Context) {
    let theme = &app.theme().clone();
    let text = app.text();

    // ══════════ Section 1: Network Topology ══════════
    section_heading(ui, text.label_overview_topology, "", theme);
    ui.add_space(8.0);

    elevated_frame(theme).show(ui, |ui| {
        render_topology(app, ui);
    });

    ui.add_space(14.0);

    // ══════════ Section 2: Quick Actions ══════════
    card_title(ui, text.label_quick_actions_bar, theme);
    ui.add_space(8.0);

    elevated_frame(theme).show(ui, |ui| {
        render_quick_actions(app, ui);
    });

    ui.add_space(14.0);

    // ══════════ Section 3: Real-time Details ══════════
    card_title(ui, text.label_realtime_details, theme);
    ui.add_space(8.0);

    elevated_frame(theme).show(ui, |ui| {
        render_realtime_details(app, ui, ctx);
    });
}

// ────────────── Topology Tree ──────────────

fn render_topology(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    // Root node: local host
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("▼")
                .size(12.0)
                .color(theme.palette.accent),
        );
        ui.label(
            egui::RichText::new(&app.workspace.local_hostname)
                .size(16.0)
                .strong()
                .color(theme.palette.text_primary),
        );
    });

    // Indented children - use vertical layout with left padding
    ui.vertical(|ui| {
        ui.add_space(6.0);

        // Local machine entry
        ui.horizontal(|ui| {
            ui.add_space(16.0); // Manual indent
            ui.label(
                egui::RichText::new("├─ 🖥")
                    .size(13.0)
                    .color(theme.palette.text_secondary),
            );
            ui.label(egui::RichText::new(text.topology_local).color(theme.palette.text_primary));
        });

        // Remote clients
        for client in app.workspace.clients.values() {
            let icon = match client.status() {
                ClientConnectivity::Reachable => "🟢",
                ClientConnectivity::Resolving => "🟡",
                ClientConnectivity::Unresolved => "🔴",
            };

            ui.horizontal(|ui| {
                ui.add_space(16.0); // Manual indent
                ui.label(
                    egui::RichText::new("├─ 🖥")
                        .size(13.0)
                        .color(theme.palette.text_secondary),
                );
                ui.label(egui::RichText::new(icon).size(10.0));
                ui.label(
                    egui::RichText::new(format!("{}  {}", client.title(), text.topology_remote,))
                        .color(theme.palette.text_primary),
                );
                if !client.ips.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("{}", client.ips[0]))
                            .size(12.0)
                            .color(theme.palette.text_secondary),
                    );
                }
            });
        }
    });
}

// ────────────── Quick Actions ──────────────

fn render_quick_actions(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    // Metrics row
    ui.horizontal_wrapped(|ui| {
        metric_card(
            ui,
            "🔗",
            text.label_connected_count,
            &app.workspace.client_count().to_string(),
            theme.palette.accent,
            theme,
        );
        metric_card(
            ui,
            "⇄",
            text.label_routes_count,
            &app.workspace.active_routes().to_string(),
            theme.palette.success,
            theme,
        );
        metric_card(
            ui,
            "🛡",
            text.label_trusted_devices,
            &app.workspace.trusted_devices().to_string(),
            theme.palette.warning,
            theme,
        );
        metric_card(
            ui,
            "📡",
            text.label_listen_port,
            &app.workspace.port.to_string(),
            theme.palette.accent_secondary,
            theme,
        );
    });

    ui.add_space(12.0);

    // Toggle switches row
    ui.horizontal(|ui| {
        let mut capture_on = app.workspace.capture_status == Status::Enabled;
        let capture_label = text.label_capture_control;

        // Capture toggle
        tinted_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("● ").color(if capture_on {
                    theme.palette.success
                } else {
                    theme.palette.danger
                }));
                ui.label(
                    egui::RichText::new(capture_label)
                        .strong()
                        .color(theme.palette.text_primary),
                );
                ui.add_space(8.0);
                if toggle_switch(ui, &mut capture_on, "", theme) {
                    app.send_request(FrontendRequest::EnableCapture);
                }
            });
        });

        ui.add_space(16.0);

        // Emulation toggle
        let mut emulation_on = app.workspace.emulation_status == Status::Enabled;
        let emulation_label = text.label_emulation_control;

        tinted_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("● ").color(if emulation_on {
                    theme.palette.success
                } else {
                    theme.palette.danger
                }));
                ui.label(
                    egui::RichText::new(emulation_label)
                        .strong()
                        .color(theme.palette.text_primary),
                );
                ui.add_space(8.0);
                if toggle_switch(ui, &mut emulation_on, "", theme) {
                    app.send_request(FrontendRequest::EnableEmulation);
                }
            });
        });
    });
}

// ────────────── Real-time Details (Tabbed) ──────────────

fn render_realtime_details(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, _ctx: &Context) {
    let theme = &app.theme().clone();
    let text = app.text();

    // Tab bar
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

        // Scan button (right-aligned)
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(action_button(
                    text.action_scan,
                    ButtonKind::Secondary,
                    theme,
                ))
                .clicked()
            {
                // Trigger DNS resolution for all clients
                for handle in app.workspace.clients.keys().copied().collect::<Vec<_>>() {
                    app.send_request(FrontendRequest::ResolveDns(handle));
                }
            }
        });
    });

    ui.add_space(10.0);

    match app.preferences.overview_tab {
        OverviewTab::DeviceList => render_device_table(app, ui),
        OverviewTab::SecurityEvents => render_events_table(app, ui),
    }
}

fn render_device_table(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    if app.workspace.clients.is_empty() {
        help_text(ui, text.empty_clients, theme);
        return;
    }

    // Table header
    Grid::new("device-table")
        .num_columns(6)
        .spacing(Vec2::new(16.0, 8.0))
        .striped(true)
        .min_col_width(60.0)
        .show(ui, |ui| {
            table_header(ui, text.col_status_led, theme);
            table_header(ui, text.col_name, theme);
            table_header(ui, text.col_ip_port, theme);
            table_header(ui, text.col_timestamp, theme);
            table_header(ui, text.col_event_desc, theme);
            table_header(ui, text.col_association, theme);
            ui.end_row();

            for client in app.workspace.clients.values() {
                // Status LED
                let status_color = match client.status() {
                    ClientConnectivity::Reachable => theme.palette.success,
                    ClientConnectivity::Resolving => theme.palette.warning,
                    ClientConnectivity::Unresolved => theme.palette.danger,
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("●").size(14.0).color(status_color));
                });

                // Name
                table_cell(ui, &client.title(), theme);

                // IP/Port
                let ip_text = if client.ips.is_empty() {
                    "-".to_string()
                } else {
                    format!("{}:{}", client.ips[0], client.port)
                };
                table_cell(ui, &ip_text, theme);

                // Timestamp (use empty for static devices)
                table_cell(ui, "-", theme);

                // Event description
                let status_desc = match client.status() {
                    ClientConnectivity::Reachable => text.status_reachable,
                    ClientConnectivity::Resolving => text.status_resolving,
                    ClientConnectivity::Unresolved => text.status_unresolved,
                };
                table_cell(ui, status_desc, theme);

                // Association
                let assoc = text.position(client.position);
                table_cell(ui, assoc, theme);

                ui.end_row();
            }
        });
}

fn render_events_table(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    if app.workspace.network_events.is_empty() {
        help_text(ui, text.recent_security_hint, theme);
        return;
    }

    Grid::new("events-table")
        .num_columns(5)
        .spacing(Vec2::new(16.0, 8.0))
        .striped(true)
        .min_col_width(60.0)
        .show(ui, |ui| {
            table_header(ui, text.col_status_led, theme);
            table_header(ui, text.col_name, theme);
            table_header(ui, text.col_ip_port, theme);
            table_header(ui, text.col_timestamp, theme);
            table_header(ui, text.col_event_desc, theme);
            ui.end_row();

            // Show events in reverse chronological order
            for event in app.workspace.network_events.iter().rev().take(50) {
                // LED
                ui.label(
                    egui::RichText::new("●")
                        .size(14.0)
                        .color(theme.palette.accent),
                );

                // Source
                table_cell(ui, &event.source, theme);

                // IP (same as source for events)
                table_cell(ui, &event.source, theme);

                // Timestamp
                table_cell(ui, &event.timestamp_label, theme);

                // Description
                table_cell(ui, &event.description, theme);

                ui.end_row();
            }
        });
}

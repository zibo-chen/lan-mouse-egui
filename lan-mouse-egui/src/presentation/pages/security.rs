use eframe::egui::{self, Layout, Vec2};
use lan_mouse_ipc::FrontendRequest;

use crate::{
    application::LanMouseDesktopApp,
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, empty_state, fingerprint_block,
        help_text,
    },
};

pub fn render(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let theme = &app.theme().clone();
    let text = app.text();

    ui.horizontal_top(|ui| {
        let half = (ui.available_width() - 14.0) / 2.0;
        ui.allocate_ui(Vec2::new(half, 0.0), |ui| {
            elevated_frame(theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    card_title(ui, text.label_identity, theme);
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_copy = !app.workspace.public_key_fingerprint.is_empty();
                        if ui
                            .add_enabled(
                                can_copy,
                                action_button(
                                    text.action_copy_fingerprint,
                                    ButtonKind::Secondary,
                                    theme,
                                ),
                            )
                            .clicked()
                        {
                            app.copy_fingerprint(ctx);
                        }
                    });
                });
                ui.add_space(10.0);
                if app.workspace.public_key_fingerprint.is_empty() {
                    help_text(ui, text.fingerprint_pending, theme);
                } else {
                    fingerprint_block(ui, &app.workspace.public_key_fingerprint, theme);
                }
            });
        });
        ui.add_space(14.0);
        ui.allocate_ui(Vec2::new(half, 0.0), |ui| {
            elevated_frame(theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    card_title(ui, text.label_allowlist, theme);
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(action_button(
                                text.action_add_fingerprint,
                                ButtonKind::Primary,
                                theme,
                            ))
                            .clicked()
                        {
                            app.open_fingerprint_dialog(None);
                        }
                    });
                });
                ui.add_space(10.0);
                help_text(ui, text.recent_security_hint, theme);
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} {}",
                        text.label_trusted_devices,
                        app.workspace.trusted_devices()
                    ))
                    .strong()
                    .color(theme.palette.text_primary),
                );
            });
        });
    });

    ui.add_space(14.0);
    if app.workspace.authorized_keys.is_empty() {
        empty_state(ui, text.empty_security, text.empty_security_hint, theme);
        return;
    }

    let keys = app
        .workspace
        .authorized_keys
        .iter()
        .map(|(fingerprint, description)| (fingerprint.clone(), description.clone()))
        .collect::<Vec<_>>();

    for (fingerprint, description) in keys {
        elevated_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new((ui.available_width() - 120.0).max(200.0), 0.0),
                    Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(
                            egui::RichText::new(description)
                                .strong()
                                .color(theme.palette.text_primary),
                        );
                        fingerprint_block(ui, &fingerprint, theme);
                    },
                );
                ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                    if ui
                        .add(action_button(text.action_remove, ButtonKind::Danger, theme))
                        .clicked()
                    {
                        app.send_request(FrontendRequest::RemoveAuthorizedKey(fingerprint.clone()));
                    }
                });
            });
        });
        ui.add_space(10.0);
    }
}

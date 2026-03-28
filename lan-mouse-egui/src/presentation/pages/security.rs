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

    let total_w = ui.available_width();
    let half = (total_w - 8.0) / 2.0;

    ui.horizontal_top(|ui| {
        // Identity card
        ui.allocate_ui_with_layout(
            Vec2::new(half, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_max_width(half);
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
                    ui.add_space(4.0);
                    if app.workspace.public_key_fingerprint.is_empty() {
                        help_text(ui, text.fingerprint_pending, theme);
                    } else {
                        fingerprint_block(ui, &app.workspace.public_key_fingerprint, theme);
                    }
                });
            },
        );

        ui.add_space(8.0);

        // Allowlist card
        ui.allocate_ui_with_layout(
            Vec2::new(half, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_max_width(half);
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
                    ui.add_space(4.0);
                    help_text(ui, text.recent_security_hint, theme);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {}",
                            text.label_trusted_devices,
                            app.workspace.trusted_devices()
                        ))
                        .strong()
                        .color(theme.palette.text_primary),
                    );
                });
            },
        );
    });

    ui.add_space(8.0);

    if app.workspace.authorized_keys.is_empty() {
        empty_state(ui, text.empty_security, text.empty_security_hint, theme);
        return;
    }

    let keys: Vec<_> = app
        .workspace
        .authorized_keys
        .iter()
        .map(|(f, d)| (f.clone(), d.clone()))
        .collect();

    for (fingerprint, description) in keys {
        elevated_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&description)
                        .strong()
                        .color(theme.palette.text_primary),
                );
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(action_button(text.action_remove, ButtonKind::Danger, theme))
                        .clicked()
                    {
                        app.send_request(FrontendRequest::RemoveAuthorizedKey(fingerprint.clone()));
                    }
                });
            });
            fingerprint_block(ui, &fingerprint, theme);
        });
        ui.add_space(4.0);
    }
}

use eframe::egui::{self, Context, TextEdit};
use lan_mouse_ipc::FrontendRequest;

use crate::{
    application::LanMouseDesktopApp,
    presentation::components::{
        ButtonKind, action_button, elevated_frame, fingerprint_block, help_text,
    },
};

pub fn render_dialogs(app: &mut LanMouseDesktopApp, ctx: &Context) {
    let text = app.text();
    let theme = &app.theme().clone();

    if let Some(fingerprint) = app.dialogs.authorization_request.clone() {
        egui::Window::new(text.auth_request_title)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                elevated_frame(theme).show(ui, |ui| {
                    help_text(ui, text.auth_request_body, theme);
                    ui.add_space(10.0);
                    fingerprint_block(ui, &fingerprint, theme);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(action_button(
                                text.action_authorize,
                                ButtonKind::Primary,
                                theme,
                            ))
                            .clicked()
                        {
                            app.open_fingerprint_dialog(Some(fingerprint.clone()));
                            app.dialogs.authorization_request = None;
                        }
                        if ui
                            .add(action_button(
                                text.action_dismiss,
                                ButtonKind::Secondary,
                                theme,
                            ))
                            .clicked()
                        {
                            app.dialogs.authorization_request = None;
                        }
                    });
                });
            });
    }

    if let Some(mut form) = app.dialogs.fingerprint_form.take() {
        let mut keep_open = true;
        egui::Window::new(text.action_add_fingerprint)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                elevated_frame(theme).show(ui, |ui| {
                    ui.label(text.label_description);
                    ui.add(TextEdit::singleline(&mut form.description).desired_width(360.0));
                    ui.add_space(6.0);
                    ui.label(text.label_fingerprint);
                    ui.add(TextEdit::singleline(&mut form.fingerprint).desired_width(360.0));
                    ui.add_space(12.0);

                    let ready =
                        !form.description.trim().is_empty() && !form.fingerprint.trim().is_empty();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                ready,
                                action_button(text.action_save, ButtonKind::Primary, theme),
                            )
                            .clicked()
                        {
                            app.send_request(FrontendRequest::AuthorizeKey(
                                form.description.trim().to_string(),
                                form.fingerprint.trim().to_string(),
                            ));
                            keep_open = false;
                        }
                        if ui
                            .add(action_button(
                                text.action_cancel,
                                ButtonKind::Secondary,
                                theme,
                            ))
                            .clicked()
                        {
                            keep_open = false;
                        }
                    });
                });
            });

        if keep_open {
            app.dialogs.fingerprint_form = Some(form);
        }
    }
}

pub fn render_toasts(app: &mut LanMouseDesktopApp, ctx: &Context) {
    let theme = &app.theme().clone();
    let now = std::time::Instant::now();
    app.toasts.retain(|toast| toast.expires_at > now);
    if app.toasts.is_empty() {
        return;
    }

    egui::Area::new("toasts".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, [-18.0, -18.0])
        .show(ctx, |ui| {
            ui.set_width(320.0);
            for toast in &app.toasts {
                egui::Frame::new()
                    .fill(theme.palette.surface_raised)
                    .stroke(egui::Stroke::new(1.0, theme.palette.border))
                    .corner_radius(egui::CornerRadius::same(16))
                    .inner_margin(12)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&toast.text).color(theme.palette.text_primary),
                        );
                    });
                ui.add_space(8.0);
            }
        });
}

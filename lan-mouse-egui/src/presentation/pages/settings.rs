use eframe::egui::{self, ComboBox, Layout, TextEdit, Vec2};
use lan_mouse_ipc::{DEFAULT_PORT, FrontendRequest, Status};

use crate::{
    application::LanMouseDesktopApp,
    domain::{LanguageChoice, ThemeFamily, ThemeModeChoice, parse_port_input, port_to_input},
    presentation::components::{
        ButtonKind, action_button, card_title, elevated_frame, field_label, help_text, status_pill,
        tinted_frame,
    },
};

pub fn render(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    ui.horizontal_top(|ui| {
        let half = (ui.available_width() - 14.0) / 2.0;
        ui.allocate_ui(Vec2::new(half, 0.0), |ui| {
            elevated_frame(theme).show(ui, |ui| {
                card_title(ui, text.label_desktop, theme);
                ui.add_space(10.0);

                field_label(ui, text.label_theme_family, theme);
                ComboBox::from_id_salt("theme-family")
                    .selected_text(text.theme_family(app.preferences.theme_family))
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for family in ThemeFamily::ALL {
                            ui.selectable_value(
                                &mut app.preferences.theme_family,
                                family,
                                text.theme_family(family),
                            );
                        }
                    });
                ui.add_space(8.0);

                field_label(ui, text.label_appearance, theme);
                ComboBox::from_id_salt("theme-mode")
                    .selected_text(text.theme_mode(app.preferences.theme_mode))
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for mode in ThemeModeChoice::ALL {
                            ui.selectable_value(
                                &mut app.preferences.theme_mode,
                                mode,
                                text.theme_mode(mode),
                            );
                        }
                    });
                ui.add_space(8.0);

                field_label(ui, text.label_language, theme);
                ComboBox::from_id_salt("language-choice")
                    .selected_text(text.language_choice(app.preferences.language))
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for choice in LanguageChoice::ALL {
                            ui.selectable_value(
                                &mut app.preferences.language,
                                choice,
                                text.language_choice(choice),
                            );
                        }
                    });
                ui.add_space(10.0);
                help_text(ui, text.app_tagline, theme);
            });
        });
        ui.add_space(14.0);
        ui.allocate_ui(Vec2::new(half, 0.0), |ui| {
            elevated_frame(theme).show(ui, |ui| {
                card_title(ui, text.label_network, theme);
                ui.add_space(10.0);

                field_label(ui, text.label_port, theme);
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut app.workspace.port_input)
                            .desired_width(120.0)
                            .hint_text(DEFAULT_PORT.to_string()),
                    );
                    let parsed_port = parse_port_input(&app.workspace.port_input);
                    let changed = parsed_port != app.workspace.port;
                    if ui
                        .add_enabled(
                            changed,
                            action_button(text.action_apply, ButtonKind::Primary, theme),
                        )
                        .clicked()
                    {
                        app.send_request(FrontendRequest::ChangePort(parsed_port));
                    }
                    if ui
                        .add_enabled(
                            changed,
                            action_button(text.action_reset, ButtonKind::Secondary, theme),
                        )
                        .clicked()
                    {
                        app.workspace.port_input = port_to_input(app.workspace.port);
                    }
                });
                ui.add_space(8.0);
                help_text(
                    ui,
                    &format!("{} {}", text.default_port_hint, DEFAULT_PORT),
                    theme,
                );
                ui.add_space(12.0);
                tinted_frame(theme).show(ui, |ui| {
                    card_title(ui, text.label_background_behavior, theme);
                    ui.add_space(6.0);
                    help_text(
                        ui,
                        if app.has_tray() {
                            text.background_hint_tray
                        } else {
                            text.background_hint_minimize
                        },
                        theme,
                    );
                });
            });
        });
    });

    ui.add_space(14.0);
    elevated_frame(theme).show(ui, |ui| {
        card_title(ui, text.label_service_controls, theme);
        ui.add_space(10.0);
        service_row(
            app,
            ui,
            text.label_capture,
            app.workspace.capture_status == Status::Enabled,
            FrontendRequest::EnableCapture,
        );
        ui.add_space(10.0);
        service_row(
            app,
            ui,
            text.label_emulation,
            app.workspace.emulation_status == Status::Enabled,
            FrontendRequest::EnableEmulation,
        );
    });
}

fn service_row(
    app: &mut LanMouseDesktopApp,
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    retry_request: FrontendRequest,
) {
    let theme = &app.theme().clone();
    let text = app.text();
    tinted_frame(theme).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if active {
                    status_pill(ui, text.status_enabled, theme.palette.success, theme);
                } else {
                    if ui
                        .add(action_button(text.action_retry, ButtonKind::Primary, theme))
                        .clicked()
                    {
                        app.send_request(retry_request);
                    }
                    status_pill(ui, text.status_disabled, theme.palette.danger, theme);
                }
            });
        });
    });
}

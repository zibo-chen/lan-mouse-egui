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

    let total_w = ui.available_width();
    let half = (total_w - 8.0) / 2.0;
    let combo_w = (half - 40.0).clamp(100.0, 180.0);

    ui.horizontal_top(|ui| {
        // Appearance
        ui.allocate_ui_with_layout(Vec2::new(half, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_max_width(half);
            elevated_frame(theme).show(ui, |ui| {
                card_title(ui, text.label_desktop, theme);
                ui.add_space(4.0);

                field_label(ui, text.label_theme_family, theme);
                ComboBox::from_id_salt("theme-family")
                    .selected_text(text.theme_family(app.preferences.theme_family))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for family in ThemeFamily::ALL {
                            ui.selectable_value(
                                &mut app.preferences.theme_family,
                                family,
                                text.theme_family(family),
                            );
                        }
                    });
                ui.add_space(4.0);

                field_label(ui, text.label_appearance, theme);
                ComboBox::from_id_salt("theme-mode")
                    .selected_text(text.theme_mode(app.preferences.theme_mode))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for mode in ThemeModeChoice::ALL {
                            ui.selectable_value(
                                &mut app.preferences.theme_mode,
                                mode,
                                text.theme_mode(mode),
                            );
                        }
                    });
                ui.add_space(4.0);

                field_label(ui, text.label_language, theme);
                ComboBox::from_id_salt("lang")
                    .selected_text(text.language_choice(app.preferences.language))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for choice in LanguageChoice::ALL {
                            ui.selectable_value(
                                &mut app.preferences.language,
                                choice,
                                text.language_choice(choice),
                            );
                        }
                    });
            });
        });

        ui.add_space(8.0);

        // Network
        ui.allocate_ui_with_layout(Vec2::new(half, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_max_width(half);
            elevated_frame(theme).show(ui, |ui| {
                card_title(ui, text.label_network, theme);
                ui.add_space(4.0);

                field_label(ui, text.label_port, theme);
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut app.workspace.port_input)
                            .desired_width(100.0)
                            .hint_text(DEFAULT_PORT.to_string()),
                    );
                    let parsed = parse_port_input(&app.workspace.port_input);
                    let changed = parsed != app.workspace.port;
                    if ui
                        .add_enabled(
                            changed,
                            action_button(text.action_apply, ButtonKind::Primary, theme),
                        )
                        .clicked()
                    {
                        app.send_request(FrontendRequest::ChangePort(parsed));
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
                ui.add_space(4.0);
                help_text(
                    ui,
                    &format!("{} {}", text.default_port_hint, DEFAULT_PORT),
                    theme,
                );
                ui.add_space(6.0);
                tinted_frame(theme).show(ui, |ui| {
                    card_title(ui, text.label_background_behavior, theme);
                    ui.add_space(2.0);
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

    ui.add_space(8.0);

    // Service controls
    elevated_frame(theme).show(ui, |ui| {
        card_title(ui, text.label_service_controls, theme);
        ui.add_space(4.0);
        service_row(
            app,
            ui,
            text.label_capture,
            app.workspace.capture_status == Status::Enabled,
            FrontendRequest::EnableCapture,
        );
        ui.add_space(4.0);
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

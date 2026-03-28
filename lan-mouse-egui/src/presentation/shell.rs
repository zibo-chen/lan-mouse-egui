use eframe::{
    Frame,
    egui::{self, CentralPanel, Context, Frame as EguiFrame, ScrollArea, Vec2},
};
use egui_desktop::{TitleBar, TitleBarOptions, apply_rounded_corners, render_resize_handles};

use crate::{
    application::LanMouseDesktopApp,
    domain::NavigationPage,
    presentation::{
        components::{nav_icon_button, paint_background, panel_frame},
        dialogs, pages,
    },
};

const APP_ICON: &[u8] = include_bytes!("../../../lan-mouse-gtk/resources/de.feschber.LanMouse.svg");

/// Sidebar width for the narrow icon-only mode
const SIDEBAR_W: f32 = 72.0;

pub fn render(app: &mut LanMouseDesktopApp, ctx: &Context, frame: &mut Frame) {
    apply_rounded_corners(frame);

    let title_text = format!(
        "局域网鼠标  Lan Mouse        {}: {}",
        app.text().label_host,
        app.workspace.local_hostname,
    );

    TitleBar::new(
        TitleBarOptions::new()
            .with_title(&title_text)
            .with_theme_mode(app.preferences.theme_mode.into())
            .with_show_bottom_border(false)
            .with_app_icon(APP_ICON, "lan-mouse.svg"),
    )
    .with_theme(app.theme().title_bar_theme())
    .show(ctx);

    render_resize_handles(ctx);
    dialogs::render_dialogs(app, ctx);

    CentralPanel::default()
        .frame(EguiFrame::new().fill(app.theme().palette.canvas))
        .show(ctx, |ui| {
            paint_background(ui, app.theme());
            let available_h = ui.available_height();
            EguiFrame::new().inner_margin(8).show(ui, |ui| {
                let inner_h = (available_h - 16.0).max(0.0);
                ui.horizontal_top(|ui| {
                    // — Narrow icon sidebar —
                    ui.allocate_ui(Vec2::new(SIDEBAR_W, inner_h), |ui| {
                        render_sidebar(app, ui, ctx);
                    });
                    ui.add_space(8.0);
                    // — Main content —
                    ui.allocate_ui(Vec2::new(ui.available_width(), inner_h), |ui| {
                        render_main(app, ui, ctx);
                    });
                });
            });
        });

    dialogs::render_toasts(app, ctx);
}

/// Icons for nav pages (simple Unicode symbols)
const NAV_ICONS: &[(NavigationPage, &str)] = &[
    (NavigationPage::Overview, "⊞"),
    (NavigationPage::Clients, "👤"),
    (NavigationPage::Security, "🔒"),
    (NavigationPage::Settings, "⚙"),
];

fn render_sidebar(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, _ctx: &Context) {
    let theme = &app.theme().clone();
    let text = app.text();

    panel_frame(theme).show(ui, |ui| {
        ui.set_min_height(ui.available_height());
        ui.set_min_width(SIDEBAR_W - 36.0); // account for margins
        ui.vertical_centered(|ui| {
            // App icon area
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("🖱")
                    .size(28.0)
                    .color(theme.palette.accent),
            );
            ui.add_space(4.0);

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(12.0);

            // Navigation buttons
            for &(page, icon) in NAV_ICONS {
                let label = text.nav_label(page);
                if nav_icon_button(ui, icon, label, app.preferences.navigation == page, theme)
                    .clicked()
                {
                    app.preferences.navigation = page;
                }
                ui.add_space(6.0);
            }
        });
    });
}

fn render_main(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui, ctx: &Context) {
    let theme = &app.theme().clone();

    panel_frame(theme).show(ui, |ui| {
        ui.set_min_height(ui.available_height());
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match app.preferences.navigation {
                NavigationPage::Overview => pages::overview::render(app, ui, ctx),
                NavigationPage::Clients => pages::clients::render(app, ui),
                NavigationPage::Security => pages::security::render(app, ui, ctx),
                NavigationPage::Settings => pages::settings::render(app, ui),
            });
    });
}

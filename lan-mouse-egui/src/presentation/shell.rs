use eframe::{
    Frame,
    egui::{self, CentralPanel, Context, Frame as EguiFrame, ScrollArea, SidePanel},
};
use egui_desktop::{TitleBar, TitleBarOptions, apply_rounded_corners, render_resize_handles};

use crate::{
    application::LanMouseDesktopApp,
    domain::NavigationPage,
    presentation::{components::nav_icon_button, dialogs, pages},
};

const APP_ICON: &[u8] = include_bytes!("../../../lan-mouse-gtk/resources/de.feschber.LanMouse.svg");

const SIDEBAR_W: f32 = 52.0;

pub fn render(app: &mut LanMouseDesktopApp, ctx: &Context, frame: &mut Frame) {
    apply_rounded_corners(frame);

    let title_text = format!(
        "局域网鼠标  Lan Mouse    {}: {}",
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

    let sidebar_fill = app.theme().palette.sidebar_fill;
    SidePanel::left("nav_sidebar")
        .exact_width(SIDEBAR_W)
        .resizable(false)
        .frame(EguiFrame::new().fill(sidebar_fill).inner_margin(4))
        .show(ctx, |ui| {
            render_sidebar(app, ui);
        });

    let canvas = app.theme().palette.canvas;
    CentralPanel::default()
        .frame(EguiFrame::new().fill(canvas).inner_margin(12))
        .show(ctx, |ui| {
            // Layout page needs the real viewport height — don't wrap it in ScrollArea
            if matches!(app.preferences.navigation, NavigationPage::Layout) {
                pages::layout::render(app, ui, ctx);
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match app.preferences.navigation {
                            NavigationPage::Overview => pages::overview::render(app, ui, ctx),
                            NavigationPage::Clients => pages::clients::render(app, ui),
                            NavigationPage::Layout => unreachable!(),
                            NavigationPage::Security => pages::security::render(app, ui, ctx),
                            NavigationPage::Settings => pages::settings::render(app, ui),
                        }
                    });
            }
        });

    dialogs::render_toasts(app, ctx);
}

const NAV_ICONS: &[(NavigationPage, &str)] = &[
    (NavigationPage::Overview, "⊞"),
    (NavigationPage::Clients, "👤"),
    (NavigationPage::Layout, "🖥"),
    (NavigationPage::Security, "🔒"),
    (NavigationPage::Settings, "⚙"),
];

fn render_sidebar(app: &mut LanMouseDesktopApp, ui: &mut egui::Ui) {
    let theme = &app.theme().clone();
    let text = app.text();

    ui.vertical_centered(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("🖱")
                .size(20.0)
                .color(theme.palette.accent),
        );
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        for &(page, icon) in NAV_ICONS {
            let label = text.nav_label(page);
            if nav_icon_button(ui, icon, label, app.preferences.navigation == page, theme).clicked()
            {
                app.preferences.navigation = page;
            }
            ui.add_space(2.0);
        }
    });
}

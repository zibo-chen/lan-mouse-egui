use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use eframe::egui::Context;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

use crate::domain::Catalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Hide,
    Quit,
}

pub struct SystemTray {
    _tray_icon: TrayIcon,
    receiver: Receiver<MenuId>,
    show_id: MenuId,
    hide_id: MenuId,
    quit_id: MenuId,
    show_item: MenuItem,
    hide_item: MenuItem,
    quit_item: MenuItem,
}

impl SystemTray {
    pub fn new(
        ctx: &Context,
        catalog: &'static Catalog,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();
        let show_item = MenuItem::new(catalog.tray_open, true, None);
        let hide_item = MenuItem::new(catalog.tray_hide, true, None);
        let quit_item = MenuItem::new(catalog.tray_quit, true, None);

        menu.append(&show_item)?;
        menu.append(&hide_item)?;
        menu.append(&quit_item)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Lan Mouse")
            .with_icon(app_icon())
            .build()?;

        let (tx, rx) = mpsc::channel();
        let repaint_ctx = ctx.clone();
        thread::Builder::new()
            .name("lan-mouse-egui-tray-events".into())
            .spawn(move || {
                loop {
                    let mut drained = false;
                    while let Ok(event) = MenuEvent::receiver().try_recv() {
                        if tx.send(event.id).is_err() {
                            return;
                        }
                        drained = true;
                    }

                    if drained {
                        repaint_ctx.request_repaint();
                    }

                    thread::sleep(Duration::from_millis(100));
                }
            })?;

        Ok(Self {
            _tray_icon: tray_icon,
            receiver: rx,
            show_id: show_item.id().clone(),
            hide_id: hide_item.id().clone(),
            quit_id: quit_item.id().clone(),
            show_item,
            hide_item,
            quit_item,
        })
    }

    pub fn poll(&self) -> Vec<TrayEvent> {
        let mut events = Vec::new();
        while let Ok(id) = self.receiver.try_recv() {
            if id == self.show_id {
                events.push(TrayEvent::Show);
            } else if id == self.hide_id {
                events.push(TrayEvent::Hide);
            } else if id == self.quit_id {
                events.push(TrayEvent::Quit);
            }
        }
        events
    }

    pub fn refresh_labels(&mut self, catalog: &'static Catalog) {
        self.show_item.set_text(catalog.tray_open);
        self.hide_item.set_text(catalog.tray_hide);
        self.quit_item.set_text(catalog.tray_quit);
    }
}

fn app_icon() -> Icon {
    let size = 32;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let distance = (dx * dx + dy * dy).sqrt();
            let (r, g, b, a) = if distance < 14.5 {
                let highlight = ((31 - y) as f32 / 31.0).clamp(0.0, 1.0);
                (
                    (28.0 + highlight * 24.0) as u8,
                    (84.0 + highlight * 44.0) as u8,
                    (96.0 + highlight * 34.0) as u8,
                    255,
                )
            } else {
                (0, 0, 0, 0)
            };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }

    for y in 9..23 {
        for x in 7..12 {
            let offset = (y * size + x) * 4;
            rgba[offset] = 232;
            rgba[offset + 1] = 241;
            rgba[offset + 2] = 242;
            rgba[offset + 3] = 255;
        }
    }
    for y in 11..16 {
        for x in 12..25 {
            let offset = (y * size + x) * 4;
            rgba[offset] = 232;
            rgba[offset + 1] = 241;
            rgba[offset + 2] = 242;
            rgba[offset + 3] = 255;
        }
    }
    for y in 18..23 {
        for x in 12..25 {
            let offset = (y * size + x) * 4;
            rgba[offset] = 153;
            rgba[offset + 1] = 221;
            rgba[offset + 2] = 210;
            rgba[offset + 3] = 255;
        }
    }

    Icon::from_rgba(rgba, size as u32, size as u32).expect("valid tray icon")
}

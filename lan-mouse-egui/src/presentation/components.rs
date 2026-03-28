use eframe::egui::{
    self, Align, Button, Color32, CornerRadius, Frame, Layout, Margin, Painter, Pos2, RichText,
    Stroke, Ui, Vec2, vec2,
};

use crate::domain::ActiveTheme;

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

pub fn paint_background(ui: &mut Ui, theme: &ActiveTheme) {
    let rect = ui.max_rect();
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, theme.palette.canvas);
    painter.circle_filled(
        rect.left_top() + vec2(rect.width() * 0.18, rect.height() * 0.22),
        rect.width() * 0.18,
        theme.palette.accent_soft.gamma_multiply(0.55),
    );
    painter.circle_filled(
        rect.right_top() + vec2(-rect.width() * 0.15, rect.height() * 0.10),
        rect.width() * 0.22,
        theme.palette.accent_secondary.gamma_multiply(0.08),
    );
    painter.circle_filled(
        rect.center_bottom() + vec2(rect.width() * 0.10, -rect.height() * 0.20),
        rect.width() * 0.20,
        theme.palette.accent.gamma_multiply(0.08),
    );
}

pub fn panel_frame(theme: &ActiveTheme) -> Frame {
    Frame::new()
        .fill(theme.palette.surface)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(26))
        .inner_margin(18)
}

pub fn elevated_frame(theme: &ActiveTheme) -> Frame {
    Frame::new()
        .fill(theme.palette.surface_raised)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(22))
        .inner_margin(16)
}

pub fn tinted_frame(theme: &ActiveTheme) -> Frame {
    Frame::new()
        .fill(theme.palette.surface_tint)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(14)
}

pub fn action_button(label: &str, kind: ButtonKind, theme: &ActiveTheme) -> Button<'static> {
    let (fill, stroke, text) = match kind {
        ButtonKind::Primary => (
            theme.palette.accent,
            theme.palette.accent_secondary,
            Color32::WHITE,
        ),
        ButtonKind::Secondary => (
            theme.palette.surface_tint,
            theme.palette.border_strong,
            theme.palette.text_primary,
        ),
        ButtonKind::Danger => (theme.palette.danger, theme.palette.danger, Color32::WHITE),
        ButtonKind::Ghost => (
            theme.palette.nav_fill,
            theme.palette.border,
            theme.palette.text_primary,
        ),
    };

    Button::new(RichText::new(label.to_owned()).strong().color(text))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(14))
        .min_size(vec2(0.0, 38.0))
}

pub fn nav_button(ui: &mut Ui, label: &str, selected: bool, theme: &ActiveTheme) -> egui::Response {
    let (fill, stroke) = if selected {
        (
            theme.palette.nav_active_fill,
            theme.palette.nav_active_stroke,
        )
    } else {
        (theme.palette.nav_fill, theme.palette.border)
    };

    ui.add_sized(
        [ui.available_width(), 42.0],
        Button::new(
            RichText::new(label.to_owned())
                .strong()
                .color(theme.palette.text_primary),
        )
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(14)),
    )
}

pub fn section_heading(ui: &mut Ui, title: &str, subtitle: &str, theme: &ActiveTheme) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(title)
                .size(22.0)
                .strong()
                .color(theme.palette.text_primary),
        );
        ui.label(
            RichText::new(subtitle)
                .size(13.0)
                .color(theme.palette.text_secondary),
        );
    });
}

pub fn metric_tile(ui: &mut Ui, label: &str, value: &str, accent: Color32, theme: &ActiveTheme) {
    Frame::new()
        .fill(theme.palette.surface_tint)
        .stroke(Stroke::new(1.0, accent))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_min_width(90.0);
            ui.label(
                RichText::new(label)
                    .size(12.0)
                    .color(theme.palette.text_secondary),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new(value)
                    .size(22.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
        });
}

pub fn status_pill(ui: &mut Ui, label: &str, color: Color32, theme: &ActiveTheme) {
    Frame::new()
        .fill(color.gamma_multiply(0.16))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .size(12.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
        });
}

pub fn field_label(ui: &mut Ui, label: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(label)
            .size(12.5)
            .strong()
            .color(theme.palette.text_secondary),
    );
}

pub fn empty_state(ui: &mut Ui, title: &str, detail: &str, theme: &ActiveTheme) {
    tinted_frame(theme).show(ui, |ui| {
        ui.set_min_height(160.0);
        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            ui.add_space(14.0);
            ui.label(
                RichText::new(title)
                    .size(18.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.label(
                RichText::new(detail)
                    .size(13.0)
                    .color(theme.palette.text_secondary),
            );
        });
    });
}

pub fn fingerprint_block(ui: &mut Ui, fingerprint: &str, theme: &ActiveTheme) {
    Frame::new()
        .fill(theme.palette.nav_fill)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.monospace(fingerprint);
        });
}

pub fn ip_chip(ui: &mut Ui, ip: &str, theme: &ActiveTheme) {
    Frame::new()
        .fill(theme.palette.nav_fill)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.monospace(ip);
        });
}

pub fn card_title(ui: &mut Ui, title: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(title)
            .size(17.0)
            .strong()
            .color(theme.palette.text_primary),
    );
}

pub fn help_text(ui: &mut Ui, message: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(message)
            .size(12.5)
            .color(theme.palette.text_secondary),
    );
}

pub fn action_row(ui: &mut Ui, theme: &ActiveTheme, mut items: impl FnMut(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
        items(ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |_| {});
    });
    let _ = theme;
}

// ─── New components for redesigned UI ───

/// Narrow sidebar icon button with tooltip label.
pub fn nav_icon_button(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    selected: bool,
    theme: &ActiveTheme,
) -> egui::Response {
    let (fill, stroke) = if selected {
        (
            theme.palette.nav_active_fill,
            theme.palette.nav_active_stroke,
        )
    } else {
        (theme.palette.nav_fill, theme.palette.border)
    };

    let text_color = if selected {
        theme.palette.accent
    } else {
        theme.palette.text_secondary
    };

    let btn = Button::new(RichText::new(icon.to_owned()).size(20.0).color(text_color))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(14))
        .min_size(vec2(44.0, 44.0));

    let response = ui.add(btn);
    // Show label as tooltip
    response.clone().on_hover_text(label);
    response
}

/// Small status LED circle (green/yellow/red).
pub fn status_led(painter: &Painter, center: Pos2, color: Color32) {
    painter.circle_filled(center, 5.0, color);
    painter.circle_stroke(center, 5.0, Stroke::new(1.0, color.gamma_multiply(0.6)));
}

/// Toggle switch widget — returns true if toggled.
pub fn toggle_switch(ui: &mut Ui, on: &mut bool, label: &str, theme: &ActiveTheme) -> bool {
    let desired_size = vec2(40.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
    }

    let anim_t = ui.ctx().animate_bool_with_time(response.id, *on, 0.15);
    let radius = rect.height() / 2.0 - 2.0;
    let bg = if *on {
        theme.palette.success
    } else {
        theme.palette.border_strong
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same((rect.height() / 2.0) as u8), bg);

    let circle_x = egui::lerp(
        rect.left() + radius + 3.0..=rect.right() - radius - 3.0,
        anim_t,
    );
    ui.painter().circle_filled(
        egui::pos2(circle_x, rect.center().y),
        radius,
        Color32::WHITE,
    );

    // Draw label next to the toggle if provided
    if !label.is_empty() {
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                egui::pos2(rect.right() + 8.0, rect.top()),
                vec2(200.0, rect.height()),
            )),
            |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new(label)
                            .color(theme.palette.text_primary)
                            .strong(),
                    );
                });
            },
        );
    }

    response.changed()
}

/// Metric card used in the quick-actions bar – compact horizontal style.
pub fn metric_card(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    value: &str,
    accent: Color32,
    theme: &ActiveTheme,
) {
    Frame::new()
        .fill(theme.palette.surface_tint)
        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.5)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(20.0).color(accent));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(label)
                            .size(11.0)
                            .color(theme.palette.text_secondary),
                    );
                    ui.label(
                        RichText::new(value)
                            .size(20.0)
                            .strong()
                            .color(theme.palette.text_primary),
                    );
                });
            });
        });
}

/// Tab button used for switching between device list and security events.
pub fn tab_button(ui: &mut Ui, label: &str, active: bool, theme: &ActiveTheme) -> egui::Response {
    let (fill, text_color) = if active {
        (theme.palette.nav_active_fill, theme.palette.accent)
    } else {
        (Color32::TRANSPARENT, theme.palette.text_secondary)
    };

    let stroke = if active {
        Stroke::new(2.0, theme.palette.accent)
    } else {
        Stroke::NONE
    };

    ui.add(
        Button::new(RichText::new(label.to_owned()).strong().color(text_color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(10))
            .min_size(vec2(0.0, 32.0)),
    )
}

/// Table header cell.
pub fn table_header(ui: &mut Ui, label: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(label)
            .size(11.5)
            .strong()
            .color(theme.palette.text_secondary),
    );
}

/// Table body cell.
pub fn table_cell(ui: &mut Ui, text: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(text)
            .size(12.5)
            .color(theme.palette.text_primary),
    );
}

use eframe::egui::{
    self, Button, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, vec2,
};

use crate::domain::ActiveTheme;

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

// ─── Frame helpers ───

pub fn elevated_frame(theme: &ActiveTheme) -> Frame {
    Frame::new()
        .fill(theme.palette.surface_raised)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(10)
}

pub fn tinted_frame(theme: &ActiveTheme) -> Frame {
    Frame::new()
        .fill(theme.palette.surface_tint)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(8)
}

// ─── Buttons ───

pub fn action_button(label: &str, kind: ButtonKind, theme: &ActiveTheme) -> Button<'static> {
    let (fill, stroke, text_color) = match kind {
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

    Button::new(RichText::new(label.to_owned()).strong().color(text_color))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(8))
        .min_size(vec2(0.0, 28.0))
}

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

    let btn = Button::new(RichText::new(icon.to_owned()).size(18.0).color(text_color))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(10))
        .min_size(vec2(40.0, 40.0));

    let response = ui.add(btn);
    response.clone().on_hover_text(label);
    response
}

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
            .corner_radius(CornerRadius::same(6))
            .min_size(vec2(0.0, 26.0)),
    )
}

// ─── Text helpers ───

pub fn section_heading(ui: &mut Ui, title: &str, subtitle: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(title)
            .size(18.0)
            .strong()
            .color(theme.palette.text_primary),
    );
    if !subtitle.is_empty() {
        ui.label(
            RichText::new(subtitle)
                .size(12.0)
                .color(theme.palette.text_secondary),
        );
    }
}

pub fn card_title(ui: &mut Ui, title: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(title)
            .size(14.0)
            .strong()
            .color(theme.palette.text_primary),
    );
}

pub fn field_label(ui: &mut Ui, label: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(label)
            .size(12.0)
            .strong()
            .color(theme.palette.text_secondary),
    );
}

pub fn help_text(ui: &mut Ui, message: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(message)
            .size(11.5)
            .color(theme.palette.text_secondary),
    );
}

// ─── Display components ───

pub fn status_pill(ui: &mut Ui, label: &str, color: Color32, theme: &ActiveTheme) {
    Frame::new()
        .fill(color.gamma_multiply(0.15))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .size(11.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
        });
}

pub fn empty_state(ui: &mut Ui, title: &str, detail: &str, theme: &ActiveTheme) {
    tinted_frame(theme).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new(title)
                    .size(15.0)
                    .strong()
                    .color(theme.palette.text_primary),
            );
            ui.label(
                RichText::new(detail)
                    .size(12.0)
                    .color(theme.palette.text_secondary),
            );
            ui.add_space(8.0);
        });
    });
}

pub fn fingerprint_block(ui: &mut Ui, fingerprint: &str, theme: &ActiveTheme) {
    Frame::new()
        .fill(theme.palette.nav_fill)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(8)
        .show(ui, |ui| {
            ui.monospace(fingerprint);
        });
}

pub fn ip_chip(ui: &mut Ui, ip: &str, theme: &ActiveTheme) {
    Frame::new()
        .fill(theme.palette.nav_fill)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.monospace(ip);
        });
}

pub fn metric_card(
    ui: &mut Ui,
    _icon: &str,
    label: &str,
    value: &str,
    accent: Color32,
    theme: &ActiveTheme,
) {
    Frame::new()
        .fill(theme.palette.surface_tint)
        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.4)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(label)
                        .size(11.0)
                        .color(theme.palette.text_secondary),
                );
                ui.label(
                    RichText::new(value)
                        .size(18.0)
                        .strong()
                        .color(theme.palette.text_primary),
                );
            });
        });
}

// ─── Toggle switch ───

pub fn toggle_switch(ui: &mut Ui, on: &mut bool, _label: &str, theme: &ActiveTheme) -> bool {
    let desired_size = vec2(32.0, 18.0);
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

    response.changed()
}

// ─── Table helpers ───

pub fn table_header(ui: &mut Ui, label: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(label)
            .size(11.0)
            .strong()
            .color(theme.palette.text_secondary),
    );
}

pub fn table_cell(ui: &mut Ui, text: &str, theme: &ActiveTheme) {
    ui.label(
        RichText::new(text)
            .size(12.0)
            .color(theme.palette.text_primary),
    );
}

use eframe::egui::{Color32, Context, CornerRadius, Shadow, Stroke, Style, Visuals, vec2};
use egui_desktop::{ThemeMode, TitleBarTheme, detect_system_dark_mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFamily {
    Graphite,
    Sand,
    Ocean,
}

impl ThemeFamily {
    pub const ALL: [Self; 3] = [Self::Graphite, Self::Sand, Self::Ocean];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeModeChoice {
    System,
    Light,
    Dark,
}

impl ThemeModeChoice {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn is_dark(self) -> bool {
        match self {
            Self::System => detect_system_dark_mode(),
            Self::Light => false,
            Self::Dark => true,
        }
    }
}

impl From<ThemeModeChoice> for ThemeMode {
    fn from(value: ThemeModeChoice) -> Self {
        match value {
            ThemeModeChoice::System => ThemeMode::System,
            ThemeModeChoice::Light => ThemeMode::Light,
            ThemeModeChoice::Dark => ThemeMode::Dark,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub canvas: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub surface_tint: Color32,
    pub sidebar_fill: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent_secondary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub nav_fill: Color32,
    pub nav_active_fill: Color32,
    pub nav_active_stroke: Color32,
}

#[derive(Debug, Clone)]
pub struct ActiveTheme {
    pub palette: Palette,
    pub visuals: Visuals,
    pub dark: bool,
}

impl ActiveTheme {
    pub fn title_bar_theme(&self) -> TitleBarTheme {
        TitleBarTheme {
            background_color: self.palette.sidebar_fill,
            hover_color: self.palette.surface_tint,
            close_hover_color: self.palette.danger,
            close_icon_color: self.palette.text_secondary,
            maximize_icon_color: self.palette.text_secondary,
            restore_icon_color: self.palette.text_secondary,
            minimize_icon_color: self.palette.text_secondary,
            title_color: self.palette.text_primary,
            menu_text_color: self.palette.text_secondary,
            menu_text_size: 12.0,
            menu_hover_color: self.palette.surface_tint,
            keyboard_selection_color: self.palette.accent,
            submenu_background_color: self.palette.surface_raised,
            submenu_text_color: self.palette.text_primary,
            submenu_text_size: 11.0,
            submenu_hover_color: self.palette.surface_tint,
            submenu_disabled_color: self.palette.text_secondary,
            submenu_shortcut_color: self.palette.text_secondary,
            submenu_border_color: self.palette.border,
            submenu_keyboard_selection_color: self.palette.accent,
        }
    }
}

pub fn resolve_theme(family: ThemeFamily, mode: ThemeModeChoice) -> ActiveTheme {
    let dark = mode.is_dark();
    let palette = match (family, dark) {
        (ThemeFamily::Graphite, true) => Palette {
            canvas: Color32::from_rgb(10, 14, 20),
            surface: Color32::from_rgb(18, 24, 32),
            surface_raised: Color32::from_rgb(24, 32, 43),
            surface_tint: Color32::from_rgb(33, 44, 58),
            sidebar_fill: Color32::from_rgb(15, 21, 29),
            border: Color32::from_rgb(45, 59, 76),
            border_strong: Color32::from_rgb(78, 111, 142),
            text_primary: Color32::from_rgb(237, 242, 247),
            text_secondary: Color32::from_rgb(145, 161, 181),
            accent: Color32::from_rgb(96, 165, 250),
            accent_soft: Color32::from_rgb(32, 58, 101),
            accent_secondary: Color32::from_rgb(56, 189, 248),
            success: Color32::from_rgb(52, 211, 153),
            warning: Color32::from_rgb(251, 191, 36),
            danger: Color32::from_rgb(248, 113, 113),
            nav_fill: Color32::from_rgb(23, 31, 41),
            nav_active_fill: Color32::from_rgb(29, 52, 84),
            nav_active_stroke: Color32::from_rgb(96, 165, 250),
        },
        (ThemeFamily::Graphite, false) => Palette {
            canvas: Color32::from_rgb(243, 247, 252),
            surface: Color32::from_rgb(255, 255, 255),
            surface_raised: Color32::from_rgb(249, 251, 255),
            surface_tint: Color32::from_rgb(236, 242, 250),
            sidebar_fill: Color32::from_rgb(247, 250, 255),
            border: Color32::from_rgb(203, 213, 225),
            border_strong: Color32::from_rgb(96, 165, 250),
            text_primary: Color32::from_rgb(15, 23, 42),
            text_secondary: Color32::from_rgb(90, 104, 127),
            accent: Color32::from_rgb(37, 99, 235),
            accent_soft: Color32::from_rgb(219, 234, 254),
            accent_secondary: Color32::from_rgb(2, 132, 199),
            success: Color32::from_rgb(5, 150, 105),
            warning: Color32::from_rgb(217, 119, 6),
            danger: Color32::from_rgb(220, 38, 38),
            nav_fill: Color32::from_rgb(243, 247, 252),
            nav_active_fill: Color32::from_rgb(219, 234, 254),
            nav_active_stroke: Color32::from_rgb(37, 99, 235),
        },
        (ThemeFamily::Sand, true) => Palette {
            canvas: Color32::from_rgb(23, 16, 13),
            surface: Color32::from_rgb(35, 24, 19),
            surface_raised: Color32::from_rgb(45, 31, 24),
            surface_tint: Color32::from_rgb(62, 43, 34),
            sidebar_fill: Color32::from_rgb(29, 20, 16),
            border: Color32::from_rgb(90, 63, 50),
            border_strong: Color32::from_rgb(249, 115, 22),
            text_primary: Color32::from_rgb(250, 241, 232),
            text_secondary: Color32::from_rgb(197, 169, 148),
            accent: Color32::from_rgb(249, 115, 22),
            accent_soft: Color32::from_rgb(108, 52, 18),
            accent_secondary: Color32::from_rgb(251, 191, 36),
            success: Color32::from_rgb(74, 222, 128),
            warning: Color32::from_rgb(245, 158, 11),
            danger: Color32::from_rgb(248, 113, 113),
            nav_fill: Color32::from_rgb(40, 28, 22),
            nav_active_fill: Color32::from_rgb(82, 46, 18),
            nav_active_stroke: Color32::from_rgb(249, 115, 22),
        },
        (ThemeFamily::Sand, false) => Palette {
            canvas: Color32::from_rgb(252, 246, 239),
            surface: Color32::from_rgb(255, 252, 248),
            surface_raised: Color32::from_rgb(255, 248, 241),
            surface_tint: Color32::from_rgb(251, 236, 223),
            sidebar_fill: Color32::from_rgb(255, 248, 241),
            border: Color32::from_rgb(226, 196, 173),
            border_strong: Color32::from_rgb(234, 88, 12),
            text_primary: Color32::from_rgb(67, 32, 17),
            text_secondary: Color32::from_rgb(136, 93, 72),
            accent: Color32::from_rgb(234, 88, 12),
            accent_soft: Color32::from_rgb(255, 221, 194),
            accent_secondary: Color32::from_rgb(217, 119, 6),
            success: Color32::from_rgb(22, 163, 74),
            warning: Color32::from_rgb(202, 138, 4),
            danger: Color32::from_rgb(220, 38, 38),
            nav_fill: Color32::from_rgb(252, 246, 239),
            nav_active_fill: Color32::from_rgb(255, 221, 194),
            nav_active_stroke: Color32::from_rgb(234, 88, 12),
        },
        (ThemeFamily::Ocean, true) => Palette {
            canvas: Color32::from_rgb(8, 18, 24),
            surface: Color32::from_rgb(16, 28, 36),
            surface_raised: Color32::from_rgb(22, 37, 47),
            surface_tint: Color32::from_rgb(34, 54, 66),
            sidebar_fill: Color32::from_rgb(12, 23, 30),
            border: Color32::from_rgb(43, 75, 91),
            border_strong: Color32::from_rgb(45, 212, 191),
            text_primary: Color32::from_rgb(233, 247, 246),
            text_secondary: Color32::from_rgb(144, 178, 176),
            accent: Color32::from_rgb(45, 212, 191),
            accent_soft: Color32::from_rgb(22, 84, 78),
            accent_secondary: Color32::from_rgb(56, 189, 248),
            success: Color32::from_rgb(74, 222, 128),
            warning: Color32::from_rgb(251, 191, 36),
            danger: Color32::from_rgb(248, 113, 113),
            nav_fill: Color32::from_rgb(20, 32, 41),
            nav_active_fill: Color32::from_rgb(17, 94, 89),
            nav_active_stroke: Color32::from_rgb(45, 212, 191),
        },
        (ThemeFamily::Ocean, false) => Palette {
            canvas: Color32::from_rgb(240, 249, 249),
            surface: Color32::from_rgb(255, 255, 255),
            surface_raised: Color32::from_rgb(245, 252, 252),
            surface_tint: Color32::from_rgb(224, 243, 241),
            sidebar_fill: Color32::from_rgb(245, 252, 252),
            border: Color32::from_rgb(180, 214, 210),
            border_strong: Color32::from_rgb(13, 148, 136),
            text_primary: Color32::from_rgb(8, 47, 45),
            text_secondary: Color32::from_rgb(76, 111, 109),
            accent: Color32::from_rgb(13, 148, 136),
            accent_soft: Color32::from_rgb(204, 251, 241),
            accent_secondary: Color32::from_rgb(2, 132, 199),
            success: Color32::from_rgb(22, 163, 74),
            warning: Color32::from_rgb(202, 138, 4),
            danger: Color32::from_rgb(220, 38, 38),
            nav_fill: Color32::from_rgb(240, 249, 249),
            nav_active_fill: Color32::from_rgb(204, 251, 241),
            nav_active_stroke: Color32::from_rgb(13, 148, 136),
        },
    };

    ActiveTheme {
        visuals: build_visuals(&palette, dark),
        palette,
        dark,
    }
}

pub fn apply_theme(ctx: &Context, theme: &ActiveTheme) {
    let mut style: Style = (*ctx.style()).clone();
    style.spacing.item_spacing = vec2(12.0, 12.0);
    style.spacing.button_padding = vec2(14.0, 10.0);
    style.spacing.interact_size = vec2(44.0, 36.0);
    style.spacing.indent = 18.0;
    style.visuals = theme.visuals.clone();
    style.visuals.window_corner_radius = CornerRadius::same(24);
    style.visuals.menu_corner_radius = CornerRadius::same(18);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(16);
    style.visuals.window_shadow = Shadow::NONE;
    ctx.set_style(style);
}

fn build_visuals(palette: &Palette, dark: bool) -> Visuals {
    let mut visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.override_text_color = Some(palette.text_primary);
    visuals.panel_fill = palette.canvas;
    visuals.window_fill = palette.surface_raised;
    visuals.faint_bg_color = palette.surface_tint;
    visuals.extreme_bg_color = palette.surface;
    visuals.code_bg_color = palette.surface_tint;
    visuals.selection.bg_fill = palette.accent;
    visuals.selection.stroke = Stroke::new(1.0, palette.accent_secondary);
    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text_secondary);
    visuals.widgets.inactive.bg_fill = palette.surface;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.hovered.bg_fill = palette.surface_tint;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.border_strong);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.widgets.active.bg_fill = palette.nav_active_fill;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.nav_active_stroke);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text_primary);
    visuals.hyperlink_color = palette.accent;
    visuals
}

#[cfg(test)]
mod tests {
    use super::{ThemeFamily, ThemeModeChoice, resolve_theme};

    #[test]
    fn theme_resolution_works_for_all_presets() {
        for family in ThemeFamily::ALL {
            for mode in ThemeModeChoice::ALL {
                let theme = resolve_theme(family, mode);
                let _ = theme.title_bar_theme();
            }
        }
    }
}

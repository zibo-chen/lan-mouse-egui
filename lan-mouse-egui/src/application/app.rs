use eframe::{
    App, CreationContext, Frame,
    egui::{Context, FontData, FontDefinitions, FontFamily},
};
use egui_extras::install_image_loaders;
use hostname::get as get_hostname;
use lan_mouse_ipc::{ClientHandle, ConnectionError, FrontendEvent, FrontendRequest, Status};

use crate::{
    domain::{
        ActiveTheme, Catalog, ClientViewModel, DialogState, FingerprintForm, Language, LayoutState,
        Toast, UiPreferences, WorkspaceState, apply_theme, catalog, port_to_input, resolve_theme,
    },
    infrastructure::{BackendConnection, SystemTray, TrayEvent},
    presentation::shell,
};

pub struct LanMouseDesktopApp {
    pub(crate) backend: BackendConnection,
    pub(crate) tray: Option<SystemTray>,
    pub(crate) workspace: WorkspaceState,
    pub(crate) preferences: UiPreferences,
    pub(crate) dialogs: DialogState,
    pub(crate) layout: LayoutState,
    pub(crate) selected_client: Option<ClientHandle>,
    pub(crate) toasts: Vec<Toast>,
    pub(crate) window_visible: bool,
    pub(crate) exit_requested: bool,
    close_notice_shown: bool,
    theme: ActiveTheme,
    resolved_language: Language,
}

impl LanMouseDesktopApp {
    pub fn new(cc: &CreationContext<'_>) -> Result<Self, ConnectionError> {
        install_image_loaders(&cc.egui_ctx);
        configure_cjk_fonts(&cc.egui_ctx);

        let preferences = UiPreferences::default();
        let resolved_language = preferences.language.resolve();
        let theme = resolve_theme(preferences.theme_family, preferences.theme_mode);
        apply_theme(&cc.egui_ctx, &theme);

        let backend = BackendConnection::connect(&cc.egui_ctx)?;
        let local_hostname = get_hostname()
            .ok()
            .and_then(|hostname| hostname.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());
        let workspace = WorkspaceState::new(local_hostname);
        let tray = match SystemTray::new(&cc.egui_ctx, catalog(resolved_language)) {
            Ok(tray) => Some(tray),
            Err(err) => {
                log::warn!("failed to initialize tray icon: {err}");
                None
            }
        };

        let mut layout = LayoutState::default();
        layout.rebuild_from_workspace(&workspace);

        Ok(Self {
            backend,
            tray,
            workspace,
            preferences,
            dialogs: DialogState::default(),
            layout,
            selected_client: None,
            toasts: Vec::new(),
            window_visible: true,
            exit_requested: false,
            close_notice_shown: false,
            theme,
            resolved_language,
        })
    }

    pub(crate) fn theme(&self) -> &ActiveTheme {
        &self.theme
    }

    pub(crate) fn text(&self) -> &'static Catalog {
        catalog(self.resolved_language)
    }

    pub(crate) fn has_tray(&self) -> bool {
        self.tray.is_some()
    }

    pub(crate) fn current_client(&self) -> Option<ClientHandle> {
        self.selected_client
    }

    pub(crate) fn push_toast(&mut self, text: impl Into<String>) {
        self.toasts.push(Toast::new(text));
    }

    pub(crate) fn send_request(&mut self, request: FrontendRequest) {
        if let Err(err) = self.backend.request(request) {
            let prefix = self.text().toast_failed_request;
            self.push_toast(format!("{prefix}: {err}"));
        }
    }

    pub(crate) fn open_fingerprint_dialog(&mut self, fingerprint: Option<String>) {
        self.dialogs.fingerprint_form = Some(FingerprintForm {
            description: String::new(),
            fingerprint: fingerprint.unwrap_or_default(),
        });
    }

    pub(crate) fn copy_hostname(&mut self, ctx: &Context) {
        ctx.copy_text(self.workspace.local_hostname.clone());
        self.push_toast(self.text().toast_hostname_copied);
    }

    pub(crate) fn copy_fingerprint(&mut self, ctx: &Context) {
        if self.workspace.public_key_fingerprint.is_empty() {
            return;
        }
        ctx.copy_text(self.workspace.public_key_fingerprint.clone());
        self.push_toast(self.text().toast_fingerprint_copied);
    }

    pub(crate) fn hide_window(&mut self, ctx: &Context) {
        self.window_visible = false;
        if self.tray.is_some() && !cfg!(target_os = "windows") {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Visible(false));
        } else {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Minimized(true));
        }
    }

    pub(crate) fn show_window(&mut self, ctx: &Context) {
        self.window_visible = true;
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Focus);
    }

    pub(crate) fn request_quit(&mut self, ctx: &Context) {
        self.exit_requested = true;
        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
    }

    fn prepare_frame(&mut self, ctx: &Context) {
        let resolved_language = self.preferences.language.resolve();
        if resolved_language != self.resolved_language {
            self.resolved_language = resolved_language;
            let text = self.text();
            if let Some(tray) = self.tray.as_mut() {
                tray.refresh_labels(text);
            }
        }

        self.theme = resolve_theme(self.preferences.theme_family, self.preferences.theme_mode);
        apply_theme(ctx, &self.theme);
    }

    fn ensure_selected_client(&mut self) {
        if let Some(handle) = self.selected_client {
            if self.workspace.clients.contains_key(&handle) {
                return;
            }
        }
        self.selected_client = self.workspace.clients.keys().copied().next();
    }

    fn poll_events(&mut self) {
        let mut layout_needs_rebuild = false;
        for event in self.backend.drain_events() {
            match event {
                FrontendEvent::Created(handle, config, state) => {
                    self.workspace
                        .clients
                        .insert(handle, ClientViewModel::new(handle, config, state));
                    self.selected_client = Some(handle);
                    layout_needs_rebuild = true;
                }
                FrontendEvent::Deleted(handle) => {
                    self.workspace.clients.remove(&handle);
                    layout_needs_rebuild = true;
                }
                FrontendEvent::State(handle, config, state) => {
                    if let Some(client) = self.workspace.clients.get_mut(&handle) {
                        client.apply_config(config);
                        client.apply_state(state);
                    } else {
                        self.workspace
                            .clients
                            .insert(handle, ClientViewModel::new(handle, config, state));
                    }
                    layout_needs_rebuild = true;
                }
                FrontendEvent::Enumerate(clients) => {
                    self.workspace.clients.clear();
                    for (handle, config, state) in clients {
                        self.workspace
                            .clients
                            .insert(handle, ClientViewModel::new(handle, config, state));
                    }
                    layout_needs_rebuild = true;
                }
                FrontendEvent::LocalDisplaysChanged(displays) => {
                    self.workspace.local_screens = displays;
                    layout_needs_rebuild = true;
                }
                FrontendEvent::PortChanged(port, message) => {
                    self.workspace.port = port;
                    self.workspace.port_input = port_to_input(port);
                    if let Some(message) = message {
                        self.push_toast(message);
                    }
                }
                FrontendEvent::CaptureStatus(status) => {
                    self.workspace.capture_status = status;
                }
                FrontendEvent::EmulationStatus(status) => {
                    self.workspace.emulation_status = status;
                }
                FrontendEvent::AuthorizedUpdated(keys) => {
                    self.workspace.authorized_keys = keys.into_iter().collect();
                }
                FrontendEvent::PublicKeyFingerprint(fingerprint) => {
                    self.workspace.public_key_fingerprint = fingerprint;
                }
                FrontendEvent::ConnectionAttempt { fingerprint } => {
                    self.dialogs.authorization_request = Some(fingerprint);
                }
                FrontendEvent::DeviceConnected { addr, .. } => {
                    self.push_toast(format!("{}: {addr}", self.text().toast_device_connected));
                    self.workspace.push_event(
                        self.text().toast_device_connected,
                        addr.to_string(),
                        "".to_string(),
                    );
                }
                FrontendEvent::DeviceEntered { addr, pos, .. } => {
                    self.push_toast(format!(
                        "{}: {addr} ({})",
                        self.text().toast_device_entered,
                        self.text().position(pos)
                    ));
                    self.workspace.push_event(
                        self.text().toast_device_entered,
                        addr.to_string(),
                        self.text().position(pos).to_string(),
                    );
                }
                FrontendEvent::IncomingDisconnected(addr) => {
                    self.push_toast(format!("{}: {addr}", self.text().toast_disconnected));
                    self.workspace.push_event(
                        self.text().toast_disconnected,
                        addr.to_string(),
                        "".to_string(),
                    );
                }
                FrontendEvent::Error(message) => {
                    self.push_toast(message);
                }
                FrontendEvent::NoSuchClient(handle) => {
                    self.push_toast(format!("{}: {handle}", self.text().toast_no_such_client));
                }
                FrontendEvent::LayoutSynced {
                    handle,
                    sender_rects,
                    receiver_rects,
                } => {
                    self.layout
                        .apply_synced_rects(handle, &receiver_rects, &sender_rects);
                    layout_needs_rebuild = false; // already applied inline
                }
            }
        }

        self.ensure_selected_client();

        // Keep layout screen list in sync with clients
        let expected_count = self.workspace.local_screens.len()
            + self
                .workspace
                .clients
                .values()
                .map(|client| client.screens.len())
                .sum::<usize>();
        if layout_needs_rebuild || self.layout.screen_count() != expected_count {
            self.layout.rebuild_from_workspace(&self.workspace);
        }
    }

    fn poll_tray_events(&mut self, ctx: &Context) {
        let Some(tray) = self.tray.as_ref() else {
            return;
        };

        for event in tray.poll() {
            match event {
                TrayEvent::Show => self.show_window(ctx),
                TrayEvent::Hide => self.hide_window(ctx),
                TrayEvent::Quit => self.request_quit(ctx),
            }
        }
    }

    fn process_window_events(&mut self, ctx: &Context) {
        if self.exit_requested {
            return;
        }

        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::CancelClose);
            self.hide_window(ctx);
            if !self.close_notice_shown {
                let message = if self.has_tray() {
                    self.text().close_notice_tray
                } else {
                    self.text().close_notice_minimize
                };
                self.push_toast(message);
                self.close_notice_shown = true;
            }
        }
    }

    pub(crate) fn page_title(&self) -> &'static str {
        self.text().nav_label(self.preferences.navigation)
    }

    pub(crate) fn page_subtitle(&self) -> &'static str {
        self.text().nav_subtitle(self.preferences.navigation)
    }

    pub(crate) fn health_badge(&self) -> (&'static str, eframe::egui::Color32) {
        if self.workspace.capture_status == Status::Enabled
            && self.workspace.emulation_status == Status::Enabled
        {
            (self.text().status_enabled, self.theme.palette.success)
        } else {
            (self.text().status_disabled, self.theme.palette.warning)
        }
    }

    pub(crate) fn current_client_title(&self) -> Option<String> {
        self.selected_client
            .and_then(|handle| self.workspace.clients.get(&handle))
            .map(ClientViewModel::title)
    }
}

impl App for LanMouseDesktopApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.poll_events();
        self.poll_tray_events(ctx);
        self.process_window_events(ctx);
        self.prepare_frame(ctx);
        shell::render(self, ctx, frame);
    }
}

/// Try to load a system CJK font and register it as a fallback so that
/// Chinese / Japanese / Korean text renders instead of showing tofu (□).
fn configure_cjk_fonts(ctx: &Context) {
    let Some(bytes) = load_system_cjk_font() else {
        log::warn!("no system CJK font found – Chinese text will show as □");
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("cjk".into(), FontData::from_owned(bytes).into());

    // Append as fallback to both proportional and monospace families.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push("cjk".into());
        }
    }

    ctx.set_fonts(fonts);
}

fn load_system_cjk_font() -> Option<Vec<u8>> {
    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &[
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
    ];

    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
    ];

    #[cfg(all(unix, not(target_os = "macos")))]
    let candidates: &[&str] = &[
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            log::info!("loaded CJK font from {path}");
            return Some(bytes);
        }
    }
    None
}

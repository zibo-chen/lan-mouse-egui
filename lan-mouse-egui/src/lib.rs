use std::{
    collections::BTreeMap,
    net::IpAddr,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use eframe::{
    App, CreationContext, Frame, NativeOptions,
    egui::{
        self, Align, Color32, ComboBox, Context, CornerRadius, Layout, RichText, ScrollArea,
        Stroke, TextEdit, Vec2, vec2,
    },
};
use hostname::get as get_hostname;
use lan_mouse_ipc::{
    ClientConfig, ClientHandle, ClientState, ConnectionError, DEFAULT_PORT, FrontendEvent,
    FrontendRequest, FrontendRequestWriter, Position, Status, connect,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EguiError {
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error(transparent)]
    Native(#[from] eframe::Error),
}

pub fn run() -> Result<(), EguiError> {
    log::debug!("running egui frontend");
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lan Mouse")
            .with_inner_size([1040.0, 760.0])
            .with_min_inner_size([860.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lan Mouse",
        options,
        Box::new(|cc| Ok(Box::new(LanMouseApp::new(cc)?))),
    )?;

    Ok(())
}

#[derive(Clone)]
struct ClientView {
    handle: ClientHandle,
    hostname: Option<String>,
    hostname_input: String,
    hostname_dirty: bool,
    port: u16,
    port_input: String,
    port_dirty: bool,
    position: Position,
    active: bool,
    resolving: bool,
    has_ips: bool,
    ips: Vec<IpAddr>,
}

impl ClientView {
    fn new(handle: ClientHandle, config: ClientConfig, state: ClientState) -> Self {
        let hostname_input = config.hostname.clone().unwrap_or_default();
        let port_input = port_to_input(config.port);
        let mut client = Self {
            handle,
            hostname: config.hostname,
            hostname_input,
            hostname_dirty: false,
            port: config.port,
            port_input,
            port_dirty: false,
            position: config.pos,
            active: false,
            resolving: false,
            has_ips: false,
            ips: Vec::new(),
        };
        client.apply_state(state);
        client
    }

    fn apply_config(&mut self, config: ClientConfig) {
        self.hostname = config.hostname;
        let hostname_input = self.hostname.clone().unwrap_or_default();
        if !self.hostname_dirty || self.hostname_input == hostname_input {
            self.hostname_input = hostname_input;
            self.hostname_dirty = false;
        }
        self.port = config.port;
        let port_input = port_to_input(config.port);
        if !self.port_dirty || self.port_input == port_input {
            self.port_input = port_input;
            self.port_dirty = false;
        }
        self.position = config.pos;
    }

    fn apply_state(&mut self, state: ClientState) {
        self.active = state.active;
        self.resolving = state.resolving;
        self.has_ips = !state.ips.is_empty();
        self.ips = sort_ips(state.ips.into_iter().collect());
    }

    fn title(&self) -> String {
        self.hostname
            .clone()
            .unwrap_or_else(|| format!("client-{}", self.handle))
    }
}

struct Toast {
    text: String,
    expires_at: Instant,
}

struct FingerprintDialog {
    description: String,
    fingerprint: String,
}

struct LanMouseApp {
    requester: FrontendRequestWriter,
    events: Receiver<FrontendEvent>,
    clients: BTreeMap<ClientHandle, ClientView>,
    authorized_keys: BTreeMap<String, String>,
    local_hostname: String,
    public_key_fingerprint: String,
    port: u16,
    port_input: String,
    capture_active: bool,
    emulation_active: bool,
    authorization_request: Option<String>,
    fingerprint_dialog: Option<FingerprintDialog>,
    toasts: Vec<Toast>,
}

impl LanMouseApp {
    fn new(cc: &CreationContext<'_>) -> Result<Self, ConnectionError> {
        configure_theme(&cc.egui_ctx);

        let (mut event_reader, requester) = connect()?;
        let (event_tx, event_rx) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();

        thread::Builder::new()
            .name("lan-mouse-egui-events".into())
            .spawn(move || {
                while let Some(event) = event_reader.next_event() {
                    match event {
                        Ok(event) => {
                            if event_tx.send(event).is_err() {
                                break;
                            }
                            ctx.request_repaint();
                        }
                        Err(e) => {
                            log::error!("frontend event stream failed: {e}");
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn egui event thread");

        Ok(Self {
            requester,
            events: event_rx,
            clients: BTreeMap::new(),
            authorized_keys: BTreeMap::new(),
            local_hostname: get_hostname()
                .ok()
                .and_then(|hostname| hostname.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            public_key_fingerprint: String::new(),
            port: DEFAULT_PORT,
            port_input: String::new(),
            capture_active: true,
            emulation_active: true,
            authorization_request: None,
            fingerprint_dialog: None,
            toasts: Vec::new(),
        })
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                FrontendEvent::Created(handle, config, state) => {
                    self.clients
                        .insert(handle, ClientView::new(handle, config, state));
                }
                FrontendEvent::Deleted(handle) => {
                    self.clients.remove(&handle);
                }
                FrontendEvent::State(handle, config, state) => {
                    if let Some(client) = self.clients.get_mut(&handle) {
                        client.apply_config(config);
                        client.apply_state(state);
                    } else {
                        self.clients
                            .insert(handle, ClientView::new(handle, config, state));
                    }
                }
                FrontendEvent::Enumerate(clients) => {
                    for (handle, config, state) in clients {
                        self.clients
                            .insert(handle, ClientView::new(handle, config, state));
                    }
                }
                FrontendEvent::PortChanged(port, message) => {
                    self.port = port;
                    self.port_input = port_to_input(port);
                    if let Some(message) = message {
                        self.push_toast(message);
                    }
                }
                FrontendEvent::CaptureStatus(status) => {
                    self.capture_active = status == Status::Enabled;
                }
                FrontendEvent::EmulationStatus(status) => {
                    self.emulation_active = status == Status::Enabled;
                }
                FrontendEvent::AuthorizedUpdated(keys) => {
                    self.authorized_keys = keys.into_iter().collect();
                }
                FrontendEvent::PublicKeyFingerprint(fingerprint) => {
                    self.public_key_fingerprint = fingerprint;
                }
                FrontendEvent::ConnectionAttempt { fingerprint } => {
                    self.authorization_request = Some(fingerprint);
                }
                FrontendEvent::DeviceConnected { addr, .. } => {
                    self.push_toast(format!("device connected: {addr}"));
                }
                FrontendEvent::DeviceEntered { addr, pos, .. } => {
                    self.push_toast(format!("device entered: {addr} ({pos})"));
                }
                FrontendEvent::IncomingDisconnected(addr) => {
                    self.push_toast(format!("{addr} disconnected"));
                }
                FrontendEvent::Error(message) => {
                    self.push_toast(message);
                }
                FrontendEvent::NoSuchClient(handle) => {
                    self.push_toast(format!("no such client: {handle}"));
                }
            }
        }
    }

    fn send_request(&mut self, request: FrontendRequest) {
        if let Err(e) = self.requester.request(request) {
            self.push_toast(format!("failed to send request: {e}"));
        }
    }

    fn push_toast(&mut self, text: impl Into<String>) {
        self.toasts.push(Toast {
            text: text.into(),
            expires_at: Instant::now() + Duration::from_secs(4),
        });
    }

    fn draw_hero(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        card_frame(Color32::from_rgb(18, 28, 34), Color32::from_rgb(46, 78, 86)).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    vec2((ui.available_width() - 180.0).max(360.0), 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.label(
                            RichText::new("Lan Mouse")
                                .size(30.0)
                                .strong()
                                .color(Color32::from_rgb(239, 246, 240)),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Cross-device input routing with a cleaner desktop control surface.",
                            )
                            .size(15.0)
                            .color(Color32::from_rgb(160, 183, 176)),
                        );
                        ui.add_space(16.0);
                        ui.horizontal_wrapped(|ui| {
                            metric_tile(
                                ui,
                                "Host",
                                &self.local_hostname,
                                Color32::from_rgb(43, 114, 122),
                            );
                            metric_tile(
                                ui,
                                "Clients",
                                &self.clients.len().to_string(),
                                Color32::from_rgb(108, 83, 46),
                            );
                            metric_tile(
                                ui,
                                "Authorized",
                                &self.authorized_keys.len().to_string(),
                                Color32::from_rgb(52, 92, 61),
                            );
                            metric_tile(
                                ui,
                                "Port",
                                &self.port.to_string(),
                                Color32::from_rgb(106, 62, 50),
                            );
                        });
                    },
                );

                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    if ui
                        .add(accent_button("Copy Hostname", false))
                        .clicked()
                    {
                        ctx.copy_text(self.local_hostname.clone());
                        self.push_toast("hostname copied");
                    }
                    let can_copy_fp = !self.public_key_fingerprint.is_empty();
                    if ui
                        .add_enabled(can_copy_fp, accent_button("Copy Fingerprint", true))
                        .clicked()
                    {
                        ctx.copy_text(self.public_key_fingerprint.clone());
                        self.push_toast("fingerprint copied");
                    }
                });
            });
        });
    }

    fn draw_system_panel(&mut self, ui: &mut egui::Ui) {
        card_frame(Color32::from_rgb(24, 31, 37), Color32::from_rgb(48, 62, 73)).show(ui, |ui| {
            section_header(ui, "System Health", "Capture and emulation readiness");
            ui.add_space(10.0);
            self.draw_status_row(
                ui,
                "Input Capture",
                self.capture_active,
                FrontendRequest::EnableCapture,
            );
            ui.add_space(8.0);
            self.draw_status_row(
                ui,
                "Input Emulation",
                self.emulation_active,
                FrontendRequest::EnableEmulation,
            );
        });
    }

    fn draw_status_row(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        active: bool,
        retry_request: FrontendRequest,
    ) {
        tinted_frame(
            if active {
                Color32::from_rgb(20, 42, 32)
            } else {
                Color32::from_rgb(52, 31, 28)
            },
            if active {
                Color32::from_rgb(49, 121, 83)
            } else {
                Color32::from_rgb(150, 79, 63)
            },
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if active {
                        badge(ui, "Enabled", Color32::from_rgb(53, 163, 97));
                    } else {
                        if ui.add(accent_button("Retry", true)).clicked() {
                            self.send_request(retry_request);
                        }
                        badge(ui, "Disabled", Color32::from_rgb(206, 102, 78));
                    }
                });
            });
        });
    }

    fn draw_network_panel(&mut self, ui: &mut egui::Ui) {
        card_frame(Color32::from_rgb(23, 29, 35), Color32::from_rgb(46, 61, 75)).show(ui, |ui| {
            section_header(ui, "Network", "Local listener configuration");
            ui.add_space(10.0);
            ui.label(
                RichText::new("Listen Port")
                    .strong()
                    .color(Color32::from_rgb(225, 231, 235)),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.port_input)
                        .desired_width(120.0)
                        .hint_text(DEFAULT_PORT.to_string()),
                );
                let parsed_port = parse_port_input(&self.port_input);
                let changed = parsed_port != self.port;
                if ui
                    .add_enabled(changed, accent_button("Apply", true))
                    .clicked()
                {
                    self.send_request(FrontendRequest::ChangePort(parsed_port));
                }
                if ui
                    .add_enabled(changed, accent_button("Reset", false))
                    .clicked()
                {
                    self.port_input = port_to_input(self.port);
                }
            });
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!(
                    "Default transport port is {}. Empty input falls back to the default.",
                    DEFAULT_PORT
                ))
                .size(13.0)
                .color(Color32::from_rgb(138, 154, 163)),
            );
        });
    }

    fn draw_clients(&mut self, ui: &mut egui::Ui) {
        card_frame(Color32::from_rgb(24, 29, 34), Color32::from_rgb(44, 58, 68)).show(ui, |ui| {
            ui.horizontal(|ui| {
                section_header(ui, "Clients", "Remote machines and edge routing");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(accent_button("Add Client", true)).clicked() {
                        self.send_request(FrontendRequest::Create);
                    }
                });
            });
            ui.add_space(12.0);

            if self.clients.is_empty() {
                empty_state(ui, "No configured clients yet.");
                return;
            }

            let handles = self.clients.keys().copied().collect::<Vec<_>>();
            for handle in handles {
                let mut queued_requests = Vec::new();
                let mut delete_client = false;

                if let Some(client) = self.clients.get_mut(&handle) {
                    client_card_frame(client.active).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(
                                vec2((ui.available_width() - 160.0).max(260.0), 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.label(
                                        RichText::new(client.title())
                                            .size(20.0)
                                            .strong()
                                            .color(Color32::from_rgb(239, 242, 244)),
                                    );
                                    ui.add_space(2.0);
                                    ui.horizontal_wrapped(|ui| {
                                        subtle_label(ui, &format!("Handle {}", client.handle));
                                        let status = if client.resolving {
                                            ("Resolving DNS", Color32::from_rgb(201, 154, 66))
                                        } else if client.has_ips {
                                            ("Reachable", Color32::from_rgb(56, 167, 112))
                                        } else {
                                            ("Unresolved", Color32::from_rgb(200, 97, 74))
                                        };
                                        badge(ui, status.0, status.1);
                                        if client.active {
                                            badge(ui, "Active", Color32::from_rgb(44, 133, 181));
                                        } else {
                                            badge(ui, "Idle", Color32::from_rgb(108, 119, 127));
                                        }
                                    });
                                },
                            );

                            let active_response = ui.checkbox(&mut client.active, "Route input");
                            if active_response.changed() {
                                queued_requests
                                    .push(FrontendRequest::Activate(handle, client.active));
                            }
                        });

                        ui.add_space(12.0);
                        ui.columns(2, |columns| {
                            columns[0].label(field_label("Hostname"));
                            let response = columns[0].add(
                                TextEdit::singleline(&mut client.hostname_input)
                                    .desired_width(f32::INFINITY),
                            );
                            client.hostname_dirty = response.has_focus();
                            if response.changed() {
                                let hostname = Some(client.hostname_input.trim().to_string())
                                    .filter(|value| !value.is_empty());
                                queued_requests
                                    .push(FrontendRequest::UpdateHostname(handle, hostname));
                            }

                            columns[1].label(field_label("Port"));
                            let response = columns[1].add(
                                TextEdit::singleline(&mut client.port_input)
                                    .desired_width(120.0)
                                    .hint_text(DEFAULT_PORT.to_string()),
                            );
                            client.port_dirty = response.has_focus();
                            if response.changed() {
                                queued_requests.push(FrontendRequest::UpdatePort(
                                    handle,
                                    parse_port_input(&client.port_input),
                                ));
                            }

                            columns[0].add_space(8.0);
                            columns[0].label(field_label("Screen Edge"));
                            let previous_position = client.position;
                            ComboBox::from_id_salt(("position", handle))
                                .selected_text(position_label(client.position))
                                .width(180.0)
                                .show_ui(&mut columns[0], |ui| {
                                    for position in [
                                        Position::Left,
                                        Position::Right,
                                        Position::Top,
                                        Position::Bottom,
                                    ] {
                                        ui.selectable_value(
                                            &mut client.position,
                                            position,
                                            position_label(position),
                                        );
                                    }
                                });
                            if client.position != previous_position {
                                queued_requests
                                    .push(FrontendRequest::UpdatePosition(handle, client.position));
                            }

                            columns[1].add_space(8.0);
                            columns[1].label(field_label("Known IPs"));
                            if client.ips.is_empty() {
                                columns[1].label(
                                    RichText::new("No addresses discovered yet.")
                                        .color(Color32::from_rgb(132, 146, 154)),
                                );
                            } else {
                                columns[1].horizontal_wrapped(|ui| {
                                    for ip in &client.ips {
                                        ip_chip(ui, &ip.to_string());
                                    }
                                });
                            }
                        });

                        ui.add_space(12.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.add(accent_button("Resolve DNS", true)).clicked() {
                                queued_requests.push(FrontendRequest::ResolveDns(handle));
                            }
                            if ui.add(accent_button("Delete Client", false)).clicked() {
                                delete_client = true;
                            }
                        });
                    });
                }

                if delete_client {
                    self.send_request(FrontendRequest::Delete(handle));
                }
                for request in queued_requests {
                    self.send_request(request);
                }
                ui.add_space(10.0);
            }
        });
    }

    fn draw_security_panel(&mut self, ui: &mut egui::Ui) {
        card_frame(Color32::from_rgb(24, 30, 35), Color32::from_rgb(48, 64, 72)).show(ui, |ui| {
            ui.horizontal(|ui| {
                section_header(ui, "Authorized Fingerprints", "Trusted incoming devices");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(accent_button("Add", true)).clicked() {
                        self.fingerprint_dialog = Some(FingerprintDialog {
                            description: String::new(),
                            fingerprint: String::new(),
                        });
                    }
                });
            });
            ui.add_space(10.0);

            ui.label(field_label("Local Device Fingerprint"));
            if self.public_key_fingerprint.is_empty() {
                ui.label(
                    RichText::new("Fingerprint pending from service.")
                        .color(Color32::from_rgb(132, 146, 154)),
                );
            } else {
                fingerprint_block(ui, &self.public_key_fingerprint);
            }

            ui.add_space(12.0);

            if self.authorized_keys.is_empty() {
                empty_state(ui, "No authorized fingerprints.");
                return;
            }

            let keys = self
                .authorized_keys
                .iter()
                .map(|(fingerprint, description)| (fingerprint.clone(), description.clone()))
                .collect::<Vec<_>>();

            for (fingerprint, description) in keys {
                tinted_frame(Color32::from_rgb(28, 34, 40), Color32::from_rgb(51, 67, 76)).show(
                    ui,
                    |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(description)
                                    .strong()
                                    .color(Color32::from_rgb(232, 238, 241)),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.add(accent_button("Remove", false)).clicked() {
                                    self.send_request(FrontendRequest::RemoveAuthorizedKey(
                                        fingerprint.clone(),
                                    ));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        fingerprint_block(ui, &fingerprint);
                    },
                );
                ui.add_space(8.0);
            }
        });
    }

    fn draw_pending_dialogs(&mut self, ctx: &Context) {
        if let Some(fingerprint) = self.authorization_request.clone() {
            egui::Window::new("Authorization Required")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("A device is requesting access.");
                    ui.label("Fingerprint:");
                    ui.monospace(&fingerprint);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Authorize").clicked() {
                            self.fingerprint_dialog = Some(FingerprintDialog {
                                description: String::new(),
                                fingerprint: fingerprint.clone(),
                            });
                            self.authorization_request = None;
                        }
                        if ui.button("Dismiss").clicked() {
                            self.authorization_request = None;
                        }
                    });
                });
        }

        if let Some(mut dialog) = self.fingerprint_dialog.take() {
            let mut keep_open = true;
            egui::Window::new("Add Authorized Fingerprint")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Description");
                    ui.add(TextEdit::singleline(&mut dialog.description).desired_width(340.0));
                    ui.label("SHA256 Fingerprint");
                    ui.add(TextEdit::singleline(&mut dialog.fingerprint).desired_width(340.0));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let ready = !dialog.description.trim().is_empty()
                            && !dialog.fingerprint.trim().is_empty();
                        if ui.add_enabled(ready, egui::Button::new("Save")).clicked() {
                            self.send_request(FrontendRequest::AuthorizeKey(
                                dialog.description.trim().to_string(),
                                dialog.fingerprint.trim().to_string(),
                            ));
                            keep_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            keep_open = false;
                        }
                    });
                });

            if keep_open {
                self.fingerprint_dialog = Some(dialog);
            }
        }
    }

    fn draw_toasts(&mut self, ctx: &Context) {
        let now = Instant::now();
        self.toasts.retain(|toast| toast.expires_at > now);
        if self.toasts.is_empty() {
            return;
        }

        egui::Area::new("toasts".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
            .show(ctx, |ui| {
                ui.set_width(320.0);
                for toast in &self.toasts {
                    egui::Frame::window(ui.style())
                        .fill(Color32::from_rgba_unmultiplied(30, 34, 42, 235))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&toast.text).color(Color32::WHITE));
                        });
                    ui.add_space(6.0);
                }
            });
    }
}

impl App for LanMouseApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        self.poll_events();
        self.draw_pending_dialogs(&ctx);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.draw_hero(ui, &ctx);
                ui.add_space(16.0);

                let stacked = ui.available_width() < 980.0;
                if stacked {
                    self.draw_system_panel(ui);
                    ui.add_space(12.0);
                    self.draw_network_panel(ui);
                    ui.add_space(12.0);
                    self.draw_clients(ui);
                    ui.add_space(12.0);
                    self.draw_security_panel(ui);
                } else {
                    ui.horizontal_top(|ui| {
                        let total_width = ui.available_width();
                        let left_width = (total_width * 0.62).max(520.0);
                        let right_width = (total_width - left_width - 16.0).max(280.0);

                        ui.allocate_ui_with_layout(
                            Vec2::new(left_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                self.draw_clients(ui);
                            },
                        );
                        ui.add_space(16.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(right_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                self.draw_system_panel(ui);
                                ui.add_space(12.0);
                                self.draw_network_panel(ui);
                                ui.add_space(12.0);
                                self.draw_security_panel(ui);
                            },
                        );
                    });
                }
            });

        self.draw_toasts(&ctx);
    }
}

fn parse_port_input(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(DEFAULT_PORT)
}

fn port_to_input(port: u16) -> String {
    if port == DEFAULT_PORT {
        String::new()
    } else {
        port.to_string()
    }
}

fn position_label(position: Position) -> &'static str {
    match position {
        Position::Left => "Left",
        Position::Right => "Right",
        Position::Top => "Top",
        Position::Bottom => "Bottom",
    }
}

fn sort_ips(mut ips: Vec<IpAddr>) -> Vec<IpAddr> {
    ips.sort_by_key(IpAddr::to_string);
    ips
}

fn configure_theme(ctx: &Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(14.0, 9.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size = vec2(44.0, 34.0);
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(Color32::from_rgb(224, 231, 235));
    style.visuals.panel_fill = Color32::from_rgb(13, 18, 22);
    style.visuals.window_fill = Color32::from_rgb(17, 24, 28);
    style.visuals.faint_bg_color = Color32::from_rgb(20, 26, 31);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 14, 17);
    style.visuals.code_bg_color = Color32::from_rgb(14, 19, 23);
    style.visuals.window_corner_radius = CornerRadius::same(20);
    style.visuals.menu_corner_radius = CornerRadius::same(16);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(24, 30, 36);
    style.visuals.widgets.noninteractive.bg_stroke =
        Stroke::new(1.0, Color32::from_rgb(47, 63, 73));
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(31, 40, 47);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(60, 77, 88));
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(39, 52, 60);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(83, 112, 121));
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(46, 87, 96);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(128, 174, 181));
    style.visuals.widgets.active.corner_radius = CornerRadius::same(14);
    style.visuals.selection.bg_fill = Color32::from_rgb(58, 108, 118);
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(189, 216, 220));
    ctx.set_global_style(style);
}

fn card_frame(fill: Color32, stroke: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(18)
}

fn tinted_frame(fill: Color32, stroke: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(12)
}

fn client_card_frame(active: bool) -> egui::Frame {
    let (fill, stroke) = if active {
        (
            Color32::from_rgb(23, 41, 42),
            Color32::from_rgb(75, 122, 120),
        )
    } else {
        (Color32::from_rgb(28, 33, 38), Color32::from_rgb(52, 66, 74))
    };
    tinted_frame(fill, stroke)
}

fn accent_button(label: &str, primary: bool) -> egui::Button<'static> {
    let (fill, stroke, text) = if primary {
        (
            Color32::from_rgb(45, 113, 120),
            Color32::from_rgb(131, 192, 197),
            Color32::from_rgb(245, 250, 250),
        )
    } else {
        (
            Color32::from_rgb(34, 41, 47),
            Color32::from_rgb(73, 89, 99),
            Color32::from_rgb(225, 231, 235),
        )
    };
    egui::Button::new(RichText::new(label).color(text))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(12))
        .min_size(vec2(0.0, 36.0))
}

fn badge(ui: &mut egui::Ui, label: &str, color: Color32) {
    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36);
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(12.5).strong().color(color));
        });
}

fn subtle_label(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .size(12.0)
            .color(Color32::from_rgb(138, 154, 163)),
    );
}

fn section_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(title)
                .size(21.0)
                .strong()
                .color(Color32::from_rgb(237, 242, 244)),
        );
        ui.label(
            RichText::new(subtitle)
                .size(13.5)
                .color(Color32::from_rgb(143, 159, 168)),
        );
    });
}

fn metric_tile(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            accent.r(),
            accent.g(),
            accent.b(),
            28,
        ))
        .stroke(Stroke::new(1.0, accent))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_min_width(145.0);
            ui.label(
                RichText::new(label)
                    .size(12.5)
                    .color(Color32::from_rgb(166, 186, 189)),
            );
            ui.add_space(3.0);
            ui.label(
                RichText::new(value)
                    .size(20.0)
                    .strong()
                    .color(Color32::WHITE),
            );
        });
}

fn empty_state(ui: &mut egui::Ui, message: &str) {
    tinted_frame(Color32::from_rgb(22, 27, 31), Color32::from_rgb(44, 54, 61)).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            ui.label(
                RichText::new(message)
                    .size(15.0)
                    .color(Color32::from_rgb(141, 156, 165)),
            );
            ui.add_space(6.0);
        });
    });
}

fn field_label(label: &str) -> RichText {
    RichText::new(label)
        .size(12.5)
        .strong()
        .color(Color32::from_rgb(155, 171, 178))
}

fn ip_chip(ui: &mut egui::Ui, ip: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(32, 41, 48))
        .stroke(Stroke::new(1.0, Color32::from_rgb(58, 74, 84)))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.monospace(ip);
        });
}

fn fingerprint_block(ui: &mut egui::Ui, fingerprint: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(18, 23, 27))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 58, 67)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.monospace(fingerprint);
        });
}

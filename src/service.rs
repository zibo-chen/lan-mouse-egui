use crate::{
    capture::{Capture, CaptureType, ICaptureEvent},
    client::ClientManager,
    config::{Config, ConfigClient},
    connect::LanMouseConnection,
    crypto,
    dns::{DnsEvent, DnsResolver},
    emulation::{Emulation, EmulationEvent},
    listen::{LanMouseListener, ListenerCreationError},
};
use futures::StreamExt;
use hickory_resolver::ResolveError;
use input_capture::CaptureHandle;
use lan_mouse_ipc::{
    AsyncFrontendListener, ClientConfig, ClientHandle, ClientState, DisplayInfo, FrontendEvent,
    FrontendRequest, IpcError, IpcListenerCreationError, LayoutRect, Position, Status,
};
use lan_mouse_proto::LayoutRectProto;
use log;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    net::{IpAddr, SocketAddr},
    sync::{Arc, RwLock},
};
use thiserror::Error;
use tokio::{process::Command, signal, sync::Notify};

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Dns(#[from] ResolveError),
    #[error(transparent)]
    IpcListen(#[from] IpcListenerCreationError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    ListenError(#[from] ListenerCreationError),
    #[error("failed to load certificate: `{0}`")]
    Certificate(#[from] crypto::Error),
}

pub struct Service {
    /// configuration
    config: Config,
    /// input capture
    capture: Capture,
    /// input emulation
    emulation: Emulation,
    /// dns resolver
    resolver: DnsResolver,
    /// frontend listener
    frontend_listener: AsyncFrontendListener,
    /// authorized public key sha256 fingerprints
    authorized_keys: Arc<RwLock<HashMap<String, String>>>,
    /// (outgoing) client information
    client_manager: ClientManager,
    /// current port
    port: u16,
    /// local display topology for the host device
    local_screens: Vec<DisplayInfo>,
    /// layout rects for local displays in the shared 2D layout space
    local_layout_rects: Vec<LayoutRect>,
    /// the public key fingerprint for (D)TLS
    public_key_fingerprint: String,
    /// notify for pending frontend events
    frontend_event_pending: Notify,
    /// frontend events queued for sending
    pending_frontend_events: VecDeque<FrontendEvent>,
    /// status of input capture (enabled / disabled)
    capture_status: Status,
    /// status of input emulation (enabled / disabled)
    emulation_status: Status,
    /// maps capture sub-handle → (client_handle, position) for multi-edge barriers
    capture_handle_map: HashMap<CaptureHandle, (ClientHandle, Position)>,
    /// maps client_handle → list of capture sub-handles created for it
    client_capture_handles: HashMap<ClientHandle, Vec<CaptureHandle>>,
    /// next unique capture sub-handle
    next_capture_sub_handle: CaptureHandle,
    /// multiple barrier positions per client (from layout adjacency)
    client_positions: HashMap<ClientHandle, Vec<Position>>,
    /// keep track of registered connections to avoid duplicate barriers
    incoming_conns: HashSet<SocketAddr>,
    /// map from capture handle to connection info
    incoming_conn_info: HashMap<ClientHandle, Incoming>,
    next_trigger_handle: u64,
}

#[derive(Debug)]
struct Incoming {
    fingerprint: String,
    addr: SocketAddr,
    pos: Position,
}

impl Service {
    pub async fn new(config: Config) -> Result<Self, ServiceError> {
        let client_manager = ClientManager::default();
        for client in config.clients() {
            let config = ClientConfig {
                hostname: client.hostname,
                fix_ips: client.ips.into_iter().collect(),
                port: client.port,
                pos: client.pos,
                layout_rects: client.layout_rects,
                cmd: client.enter_hook,
                input_profile: client.input_profile,
            };
            let state = ClientState {
                active: client.active,
                ips: HashSet::from_iter(config.fix_ips.iter().cloned()),
                ..Default::default()
            };
            let handle = client_manager.add_client();
            client_manager.set_config(handle, config);
            client_manager.set_state(handle, state);
        }

        // load certificate
        let cert = crypto::load_or_generate_key_and_cert(config.cert_path())?;
        let public_key_fingerprint = crypto::certificate_fingerprint(&cert);

        // create frontend communication adapter, exit if already running
        let frontend_listener = AsyncFrontendListener::new().await?;
        let local_screens = input_capture::current_displays()
            .into_iter()
            .map(to_ipc_display)
            .collect();

        let authorized_keys = Arc::new(RwLock::new(config.authorized_fingerprints()));
        // listener + connection
        let listener =
            LanMouseListener::new(config.port(), cert.clone(), authorized_keys.clone()).await?;
        let conn = LanMouseConnection::new(cert.clone(), client_manager.clone());

        // input capture + emulation
        let capture_backend = config.capture_backend().map(|b| b.into());
        let capture = Capture::new(capture_backend, conn, config.release_bind());
        let emulation_backend = config.emulation_backend().map(|b| b.into());
        let emulation = Emulation::new(emulation_backend, listener, client_manager.clone());

        // create dns resolver
        let resolver = DnsResolver::new()?;

        let port = config.port();
        let service = Self {
            config,
            capture,
            emulation,
            frontend_listener,
            resolver,
            authorized_keys,
            public_key_fingerprint,
            client_manager,
            frontend_event_pending: Default::default(),
            port,
            local_screens,
            local_layout_rects: Vec::new(),
            pending_frontend_events: Default::default(),
            capture_status: Default::default(),
            emulation_status: Default::default(),
            capture_handle_map: Default::default(),
            client_capture_handles: Default::default(),
            next_capture_sub_handle: 0,
            client_positions: Default::default(),
            incoming_conn_info: Default::default(),
            incoming_conns: Default::default(),
            next_trigger_handle: 0,
        };
        Ok(service)
    }

    pub async fn run(&mut self) -> Result<(), ServiceError> {
        let active = self.client_manager.active_clients();
        for handle in active.iter() {
            // small hack: `activate_client()` checks, if the client
            // is already active in client_manager and does not create a
            // capture barrier in that case so we have to deactivate it first
            self.client_manager.deactivate_client(*handle);
        }

        for handle in active {
            self.activate_client(handle);
        }

        loop {
            tokio::select! {
                request = self.frontend_listener.next() => self.handle_frontend_request(request),
                _ = self.frontend_event_pending.notified() => self.handle_frontend_pending().await,
                event = self.emulation.event() => self.handle_emulation_event(event),
                event = self.capture.event() => self.handle_capture_event(event),
                event = self.resolver.event() => self.handle_resolver_event(event),
                r = signal::ctrl_c() => break r.expect("failed to wait for CTRL+C"),
            }
        }

        log::info!("terminating service ...");
        log::debug!("terminating capture ...");
        self.capture.terminate().await;
        log::debug!("terminating emulation ...");
        self.emulation.terminate().await;
        log::debug!("terminating dns resolver ...");
        self.resolver.terminate().await;

        Ok(())
    }

    fn handle_frontend_request(&mut self, request: Option<Result<FrontendRequest, IpcError>>) {
        let request = match request.expect("frontend listener closed") {
            Ok(r) => r,
            Err(e) => return log::error!("error receiving request: {e}"),
        };
        match request {
            FrontendRequest::Activate(handle, active) => {
                self.set_client_active(handle, active);
                self.save_config();
            }
            FrontendRequest::AuthorizeKey(desc, fp) => {
                self.add_authorized_key(desc, fp);
                self.save_config();
            }
            FrontendRequest::ChangePort(port) => self.change_port(port),
            FrontendRequest::Create => {
                self.add_client();
                self.save_config();
            }
            FrontendRequest::Delete(handle) => {
                self.remove_client(handle);
                self.save_config();
            }
            FrontendRequest::EnableCapture => self.capture.reenable(),
            FrontendRequest::EnableEmulation => self.emulation.reenable(),
            FrontendRequest::Enumerate() => self.enumerate(),
            FrontendRequest::UpdateFixIps(handle, fix_ips) => {
                self.update_fix_ips(handle, fix_ips);
                self.save_config();
            }
            FrontendRequest::UpdateHostname(handle, host) => {
                self.update_hostname(handle, host);
                self.save_config();
            }
            FrontendRequest::UpdatePort(handle, port) => {
                self.update_port(handle, port);
                self.save_config();
            }
            FrontendRequest::UpdatePosition(handle, pos) => {
                self.update_pos(handle, pos);
                self.save_config();
            }
            FrontendRequest::UpdatePositions(handle, positions) => {
                self.update_positions(handle, positions);
                self.save_config();
            }
            FrontendRequest::UpdateLayout(handle, layout_rects) => {
                self.update_layout(handle, layout_rects);
                self.save_config();
            }
            FrontendRequest::UpdateLocalLayout(layout_rects) => {
                self.local_layout_rects = layout_rects;
                self.rebuild_captures();
                self.save_config();
            }
            FrontendRequest::ResolveDns(handle) => self.resolve(handle),
            FrontendRequest::Sync => self.sync_frontend(),
            FrontendRequest::RemoveAuthorizedKey(key) => {
                self.remove_authorized_key(key);
                self.save_config();
            }
            FrontendRequest::UpdateEnterHook(handle, enter_hook) => {
                self.update_enter_hook(handle, enter_hook)
            }
            FrontendRequest::UpdateInputProfile(handle, input_profile) => {
                self.update_input_profile(handle, input_profile)
            }
            FrontendRequest::SaveConfiguration => self.save_config(),
            FrontendRequest::SyncLayout => self.sync_layout_to_peers(),
        }
    }

    fn save_config(&mut self) {
        let clients = self.client_manager.clients();
        let clients = clients
            .into_iter()
            .map(|(c, _s)| ConfigClient {
                ips: HashSet::from_iter(c.fix_ips),
                hostname: c.hostname,
                port: c.port,
                pos: c.pos,
                layout_rects: c.layout_rects,
                active: _s.active,
                enter_hook: c.cmd,
                input_profile: c.input_profile,
            })
            .collect();
        self.config.set_clients(clients);
        let authorized_keys = self.authorized_keys.read().expect("lock").clone();
        self.config.set_authorized_keys(authorized_keys);
        if let Err(e) = self.config.write_back() {
            log::warn!("failed to write config: {e}");
        }
    }

    async fn handle_frontend_pending(&mut self) {
        while let Some(event) = self.pending_frontend_events.pop_front() {
            self.frontend_listener.broadcast(event).await;
        }
    }

    fn handle_emulation_event(&mut self, event: EmulationEvent) {
        match event {
            EmulationEvent::ConnectionAttempt { fingerprint } => {
                self.notify_frontend(FrontendEvent::ConnectionAttempt { fingerprint });
            }
            EmulationEvent::Entered {
                addr,
                pos,
                fingerprint,
            } => {
                // check if already registered
                if !self.incoming_conns.contains(&addr) {
                    self.add_incoming(addr, pos, fingerprint.clone());
                    self.notify_frontend(FrontendEvent::DeviceEntered {
                        fingerprint,
                        addr,
                        pos,
                        entry_x: None,
                        entry_y: None,
                    });
                } else {
                    self.update_incoming(addr, pos, fingerprint);
                }
            }
            EmulationEvent::Disconnected { addr } => {
                if let Some(addr) = self.remove_incoming(addr) {
                    self.notify_frontend(FrontendEvent::IncomingDisconnected(addr));
                }
            }
            EmulationEvent::PortChanged(port) => match port {
                Ok(port) => {
                    self.port = port;
                    self.notify_frontend(FrontendEvent::PortChanged(port, None));
                }
                Err(e) => self
                    .notify_frontend(FrontendEvent::PortChanged(self.port, Some(format!("{e}")))),
            },
            EmulationEvent::EmulationDisabled => {
                self.emulation_status = Status::Disabled;
                self.notify_frontend(FrontendEvent::EmulationStatus(self.emulation_status));
            }
            EmulationEvent::EmulationEnabled => {
                self.emulation_status = Status::Enabled;
                self.notify_frontend(FrontendEvent::EmulationStatus(self.emulation_status));
            }
            EmulationEvent::ReleaseNotify => self.capture.release(),
            EmulationEvent::Connected { addr, fingerprint } => {
                self.notify_frontend(FrontendEvent::DeviceConnected { addr, fingerprint });
            }
            EmulationEvent::LayoutSync {
                addr,
                sender_rects,
                receiver_rects,
            } => {
                self.handle_layout_sync(addr, sender_rects, receiver_rects);
            }
        }
    }

    fn handle_capture_event(&mut self, event: ICaptureEvent) {
        match event {
            ICaptureEvent::CaptureBegin(sub_handle, _x, _y) => {
                // Check if this is an incoming connection handle
                if let Some(incoming) = self.incoming_conn_info.get(&sub_handle) {
                    self.emulation.send_leave_event(incoming.addr);
                }
                // (sub-handle resolution for outgoing is done via ClientEntered)
            }
            ICaptureEvent::CaptureDisabled => {
                self.capture_status = Status::Disabled;
                self.notify_frontend(FrontendEvent::CaptureStatus(self.capture_status));
            }
            ICaptureEvent::CaptureEnabled => {
                self.capture_status = Status::Enabled;
                self.notify_frontend(FrontendEvent::CaptureStatus(self.capture_status));
            }
            ICaptureEvent::ClientEntered(sub_handle) => {
                // Resolve sub-handle to real client handle
                let client_handle = self
                    .resolve_capture_handle(sub_handle)
                    .map(|(h, _)| h)
                    .unwrap_or(sub_handle);
                log::info!("entering client {client_handle} (sub_handle={sub_handle}) ...");
                self.spawn_hook_command(client_handle);
            }
            ICaptureEvent::RemoteStateChanged(sub_handle) => {
                let client_handle = self
                    .resolve_capture_handle(sub_handle)
                    .map(|(h, _)| h)
                    .unwrap_or(sub_handle);
                self.broadcast_client(client_handle);
            }
        }
    }

    fn handle_resolver_event(&mut self, event: DnsEvent) {
        let handle = match event {
            DnsEvent::Resolving(handle) => {
                self.client_manager.set_resolving(handle, true);
                handle
            }
            DnsEvent::Resolved(handle, hostname, ips) => {
                self.client_manager.set_resolving(handle, false);
                if let Err(e) = &ips {
                    log::warn!("could not resolve {hostname}: {e}");
                }
                let ips = ips.unwrap_or_default();
                self.client_manager.set_dns_ips(handle, ips);
                handle
            }
        };
        self.broadcast_client(handle);
    }

    fn resolve(&self, handle: ClientHandle) {
        if let Some(hostname) = self.client_manager.get_hostname(handle) {
            self.resolver.resolve(handle, hostname);
        }
    }

    fn sync_frontend(&mut self) {
        self.enumerate();
        self.notify_frontend(FrontendEvent::LocalDisplaysChanged(
            self.local_screens.clone(),
        ));
        self.notify_frontend(FrontendEvent::EmulationStatus(self.emulation_status));
        self.notify_frontend(FrontendEvent::CaptureStatus(self.capture_status));
        self.notify_frontend(FrontendEvent::PortChanged(self.port, None));
        self.notify_frontend(FrontendEvent::PublicKeyFingerprint(
            self.public_key_fingerprint.clone(),
        ));
        let keys = self.authorized_keys.read().expect("lock").clone();
        self.notify_frontend(FrontendEvent::AuthorizedUpdated(keys));
    }

    const ENTER_HANDLE_BEGIN: u64 = u64::MAX / 2 + 1;

    fn add_incoming(&mut self, addr: SocketAddr, pos: Position, fingerprint: String) {
        let handle = Self::ENTER_HANDLE_BEGIN + self.next_trigger_handle;
        self.next_trigger_handle += 1;
        self.capture
            .create(handle, pos, CaptureType::EnterOnly, None);
        self.incoming_conns.insert(addr);
        self.incoming_conn_info.insert(
            handle,
            Incoming {
                fingerprint,
                addr,
                pos,
            },
        );
    }

    fn update_incoming(&mut self, addr: SocketAddr, pos: Position, fingerprint: String) {
        let incoming = self
            .incoming_conn_info
            .iter_mut()
            .find(|(_, i)| i.addr == addr)
            .map(|(_, i)| i)
            .expect("no such client");
        let mut changed = false;
        if incoming.fingerprint != fingerprint {
            incoming.fingerprint = fingerprint.clone();
            changed = true;
        }
        if incoming.pos != pos {
            incoming.pos = pos;
            changed = true;
        }
        if changed {
            self.remove_incoming(addr);
            self.add_incoming(addr, pos, fingerprint.clone());
            self.notify_frontend(FrontendEvent::IncomingDisconnected(addr));
            self.notify_frontend(FrontendEvent::DeviceEntered {
                fingerprint,
                addr,
                pos,
                entry_x: None,
                entry_y: None,
            });
        }
    }

    fn remove_incoming(&mut self, addr: SocketAddr) -> Option<SocketAddr> {
        let handle = self
            .incoming_conn_info
            .iter()
            .find(|(_, incoming)| incoming.addr == addr)
            .map(|(k, _)| *k)?;
        self.capture.destroy(handle);
        self.incoming_conns.remove(&addr);
        self.incoming_conn_info
            .remove(&handle)
            .map(|incoming| incoming.addr)
    }

    fn notify_frontend(&mut self, event: FrontendEvent) {
        self.pending_frontend_events.push_back(event);
        self.frontend_event_pending.notify_one();
    }

    fn add_authorized_key(&mut self, desc: String, fp: String) {
        self.authorized_keys.write().expect("lock").insert(fp, desc);
        let keys = self.authorized_keys.read().expect("lock").clone();
        self.notify_frontend(FrontendEvent::AuthorizedUpdated(keys));
    }

    fn remove_authorized_key(&mut self, fp: String) {
        self.authorized_keys.write().expect("lock").remove(&fp);
        let keys = self.authorized_keys.read().expect("lock").clone();
        self.notify_frontend(FrontendEvent::AuthorizedUpdated(keys));
    }

    fn enumerate(&mut self) {
        let clients = self.client_manager.get_client_states();
        self.notify_frontend(FrontendEvent::Enumerate(clients));
    }

    fn add_client(&mut self) {
        let handle = self.client_manager.add_client();
        log::info!("added client {handle}");
        let (c, s) = self.client_manager.get_state(handle).unwrap();
        self.notify_frontend(FrontendEvent::Created(handle, c, s));
    }

    fn set_client_active(&mut self, handle: ClientHandle, active: bool) {
        if active {
            self.activate_client(handle);
        } else {
            self.deactivate_client(handle);
        }
    }

    fn deactivate_client(&mut self, handle: ClientHandle) {
        log::debug!("deactivating client {handle}");
        if self.client_manager.deactivate_client(handle) {
            self.destroy_client_captures(handle);
            self.broadcast_client(handle);
            log::info!("deactivated client {handle}");
        }
    }

    fn activate_client(&mut self, handle: ClientHandle) {
        log::debug!("activating client");

        /* resolve dns on activate */
        self.resolve(handle);

        /* activate the client */
        if self.client_manager.activate_client(handle) {
            self.create_client_captures(handle);
            self.broadcast_client(handle);
        }
    }

    /// Create capture barriers for a client at all its configured positions.
    /// Uses sub-handles so each (client, position) pair gets its own capture.
    fn create_client_captures(&mut self, handle: ClientHandle) {
        let positions = self
            .client_positions
            .get(&handle)
            .cloned()
            .or_else(|| self.client_manager.get_pos(handle).map(|p| vec![p]))
            .unwrap_or_default();

        let mut sub_handles = Vec::new();
        for pos in &positions {
            let sub_handle = self.next_capture_sub_handle;
            self.next_capture_sub_handle += 1;
            self.capture_handle_map.insert(sub_handle, (handle, *pos));
            self.capture
                .create(sub_handle, *pos, CaptureType::Default, Some(handle));
            sub_handles.push(sub_handle);
            log::info!("activated client {handle} barrier @ {pos} (sub_handle={sub_handle})");
        }
        self.client_capture_handles.insert(handle, sub_handles);
    }

    /// Destroy all capture barriers for a client.
    fn destroy_client_captures(&mut self, handle: ClientHandle) {
        if let Some(sub_handles) = self.client_capture_handles.remove(&handle) {
            for sub_handle in sub_handles {
                self.capture_handle_map.remove(&sub_handle);
                self.capture.destroy(sub_handle);
            }
        }
    }

    /// Resolve a capture sub-handle to the original (client_handle, position).
    fn resolve_capture_handle(
        &self,
        sub_handle: CaptureHandle,
    ) -> Option<(ClientHandle, Position)> {
        self.capture_handle_map.get(&sub_handle).copied()
    }

    fn change_port(&mut self, port: u16) {
        if self.port != port {
            self.emulation.request_port_change(port);
        } else {
            self.notify_frontend(FrontendEvent::PortChanged(self.port, None));
        }
    }

    fn remove_client(&mut self, handle: ClientHandle) {
        if self
            .client_manager
            .remove_client(handle)
            .map(|(_, s)| s.active)
            .unwrap_or(false)
        {
            self.destroy_client_captures(handle);
        }
        self.client_positions.remove(&handle);
        self.notify_frontend(FrontendEvent::Deleted(handle));
    }

    fn update_fix_ips(&mut self, handle: ClientHandle, fix_ips: Vec<IpAddr>) {
        self.client_manager.set_fix_ips(handle, fix_ips);
        self.broadcast_client(handle);
    }

    fn update_hostname(&mut self, handle: ClientHandle, hostname: Option<String>) {
        log::info!("hostname changed: {hostname:?}");
        if self.client_manager.set_hostname(handle, hostname.clone()) {
            self.resolve(handle);
        }
        self.broadcast_client(handle);
    }

    fn update_port(&mut self, handle: ClientHandle, port: u16) {
        self.client_manager.set_port(handle, port);
        self.broadcast_client(handle);
    }

    fn update_pos(&mut self, handle: ClientHandle, pos: Position) {
        // update state in event input emulator & input capture
        if self.client_manager.set_pos(handle, pos) {
            // Also update client_positions to use this single position
            self.client_positions.insert(handle, vec![pos]);
            self.destroy_client_captures(handle);
            self.create_client_captures(handle);
        }
        self.broadcast_client(handle);
    }

    fn update_positions(&mut self, handle: ClientHandle, positions: Vec<Position>) {
        // Store the primary position for config
        if let Some(primary) = positions.first() {
            self.client_manager.set_pos(handle, *primary);
        }
        // Store all positions for barrier management
        let changed = self.client_positions.get(&handle) != Some(&positions);
        self.client_positions.insert(handle, positions);
        if changed {
            // Rebuild barriers if the client is active
            self.destroy_client_captures(handle);
            if self.client_manager.active_clients().contains(&handle) {
                self.create_client_captures(handle);
            }
        }
        self.broadcast_client(handle);
    }

    fn update_layout(&mut self, handle: ClientHandle, layout_rects: Vec<LayoutRect>) {
        if self.client_manager.set_layout_rects(handle, layout_rects) {
            self.rebuild_captures();
        }
        self.broadcast_client(handle);
    }

    /// Send the current layout to all connected peers via LayoutSync.
    fn sync_layout_to_peers(&mut self) {
        let local_rects: Vec<LayoutRectProto> = self
            .local_layout_rects
            .iter()
            .map(|r| LayoutRectProto {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            })
            .collect();

        for handle in self.client_manager.active_clients() {
            let client_rects: Vec<LayoutRectProto> = self
                .client_manager
                .get_layout_rects(handle)
                .into_iter()
                .map(|r| LayoutRectProto {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                })
                .collect();
            self.capture
                .send_layout_sync(handle, local_rects.clone(), client_rects);
        }
    }

    /// Process a LayoutSync received from a remote peer.
    fn handle_layout_sync(
        &mut self,
        addr: SocketAddr,
        sender_rects: Vec<LayoutRectProto>,
        receiver_rects: Vec<LayoutRectProto>,
    ) {
        // Find which client handle corresponds to this address (try active addr then IP)
        let handle = self
            .client_manager
            .find_handle_by_addr(addr)
            .or_else(|| self.client_manager.find_handle_by_ip(addr.ip()));
        let handle = match handle {
            Some(h) => h,
            None => {
                log::warn!("LayoutSync from unknown addr {addr}");
                return;
            }
        };

        log::info!(
            "received LayoutSync from client {handle} @ {addr}: {} sender rects, {} receiver rects",
            sender_rects.len(),
            receiver_rects.len()
        );

        // sender_rects = remote peer's local display positions (from our perspective, the client's rects)
        let client_layout_rects: Vec<LayoutRect> = sender_rects
            .iter()
            .map(|r| LayoutRect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            })
            .collect();
        // receiver_rects = our local display positions as set by the remote peer
        let local_layout_rects: Vec<LayoutRect> = receiver_rects
            .iter()
            .map(|r| LayoutRect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            })
            .collect();

        // Update stored layout rects for the client
        self.client_manager
            .set_layout_rects(handle, client_layout_rects.clone());
        // Update local layout rects
        self.local_layout_rects = local_layout_rects.clone();

        self.rebuild_captures();

        // Notify frontend about the layout sync
        self.notify_frontend(FrontendEvent::LayoutSynced {
            handle,
            sender_rects: client_layout_rects,
            receiver_rects: local_layout_rects,
        });
    }

    /// rebuild all capture barriers based on current layout geometry
    fn rebuild_captures(&mut self) {
        let active = self.client_manager.active_clients();
        for &handle in &active {
            self.destroy_client_captures(handle);
        }
        for handle in active {
            self.create_client_captures(handle);
        }
    }

    /// compute entry point on the target client's display space
    /// given a crossing point in local display coordinates and the layout geometry
    pub(crate) fn compute_entry_point(
        &self,
        crossing_x: f64,
        crossing_y: f64,
        handle: ClientHandle,
    ) -> (f64, f64) {
        let local_rects = &self.local_layout_rects;
        let client_rects = self.client_manager.get_layout_rects(handle);
        let client_screens: Vec<DisplayInfo> = self
            .client_manager
            .get_state(handle)
            .map(|(_, s)| s.screens)
            .unwrap_or_default();

        // Map crossing point from local display coords to layout space
        let layout_point =
            local_display_to_layout(crossing_x, crossing_y, &self.local_screens, local_rects);

        // Find adjacent client rect and compute entry point in layout space
        let (entry_lx, entry_ly, rect_idx) =
            find_entry_on_client_rect(layout_point.0, layout_point.1, &client_rects);

        // Convert from layout space to client's local display coords
        if rect_idx < client_rects.len() && rect_idx < client_screens.len() {
            let lr = &client_rects[rect_idx];
            let ds = &client_screens[rect_idx];
            // layout-space offset within this rect → fraction → display-space
            let frac_x = if lr.w > 0.0 {
                (entry_lx - lr.x) / lr.w
            } else {
                0.5
            };
            let frac_y = if lr.h > 0.0 {
                (entry_ly - lr.y) / lr.h
            } else {
                0.5
            };
            let display_x = ds.x as f64 + frac_x * ds.width as f64;
            let display_y = ds.y as f64 + frac_y * ds.height as f64;
            (display_x, display_y)
        } else {
            // fallback: center of first client display
            if let Some(ds) = client_screens.first() {
                (
                    ds.x as f64 + ds.width as f64 * 0.5,
                    ds.y as f64 + ds.height as f64 * 0.5,
                )
            } else {
                (0.0, 0.0)
            }
        }
    }

    fn update_enter_hook(&mut self, handle: ClientHandle, enter_hook: Option<String>) {
        self.client_manager.set_enter_hook(handle, enter_hook);
        self.broadcast_client(handle);
    }

    fn update_input_profile(
        &mut self,
        handle: ClientHandle,
        input_profile: lan_mouse_ipc::InputProfile,
    ) {
        self.client_manager.set_input_profile(handle, input_profile);
        self.broadcast_client(handle);
    }

    fn broadcast_client(&mut self, handle: ClientHandle) {
        let event = self
            .client_manager
            .get_state(handle)
            .map(|(c, s)| FrontendEvent::State(handle, c, s))
            .unwrap_or(FrontendEvent::NoSuchClient(handle));
        self.notify_frontend(event);
    }

    fn spawn_hook_command(&self, handle: ClientHandle) {
        let Some(cmd) = self.client_manager.get_enter_cmd(handle) else {
            return;
        };
        tokio::task::spawn_local(async move {
            log::info!("spawning command!");
            let mut child = match Command::new("sh").arg("-c").arg(cmd.as_str()).spawn() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("could not execute cmd: {e}");
                    return;
                }
            };
            match child.wait().await {
                Ok(s) => {
                    if s.success() {
                        log::info!("{cmd} exited successfully");
                    } else {
                        log::warn!("{cmd} exited with {s}");
                    }
                }
                Err(e) => log::warn!("{cmd}: {e}"),
            }
        });
    }
}

fn to_ipc_display(display: input_capture::DisplayInfo) -> DisplayInfo {
    DisplayInfo {
        name: display.name,
        x: display.x,
        y: display.y,
        width: display.width,
        height: display.height,
        primary: display.primary,
    }
}

/// Map a point in local display coordinates to layout space.
/// Finds which local display contains the point, then maps via the layout rect.
fn local_display_to_layout(
    display_x: f64,
    display_y: f64,
    local_screens: &[DisplayInfo],
    local_layout_rects: &[LayoutRect],
) -> (f64, f64) {
    for (i, screen) in local_screens.iter().enumerate() {
        let sx = screen.x as f64;
        let sy = screen.y as f64;
        let sw = screen.width as f64;
        let sh = screen.height as f64;
        // Check if point is within (or at the edge of) this display
        if display_x >= sx - 1.0
            && display_x <= sx + sw + 1.0
            && display_y >= sy - 1.0
            && display_y <= sy + sh + 1.0
        {
            if let Some(lr) = local_layout_rects.get(i) {
                let frac_x = if sw > 0.0 { (display_x - sx) / sw } else { 0.5 };
                let frac_y = if sh > 0.0 { (display_y - sy) / sh } else { 0.5 };
                return (lr.x + frac_x * lr.w, lr.y + frac_y * lr.h);
            }
        }
    }
    // fallback: use first screen or return as-is
    if let (Some(screen), Some(lr)) = (local_screens.first(), local_layout_rects.first()) {
        let frac_x = if screen.width > 0 {
            (display_x - screen.x as f64) / screen.width as f64
        } else {
            0.5
        };
        let frac_y = if screen.height > 0 {
            (display_y - screen.y as f64) / screen.height as f64
        } else {
            0.5
        };
        (lr.x + frac_x * lr.w, lr.y + frac_y * lr.h)
    } else {
        (display_x, display_y)
    }
}

/// Find the entry point on the nearest client rect edge given a layout-space point.
/// Returns (entry_x, entry_y, rect_index).
fn find_entry_on_client_rect(lx: f64, ly: f64, client_rects: &[LayoutRect]) -> (f64, f64, usize) {
    if client_rects.is_empty() {
        return (lx, ly, 0);
    }

    // Find the nearest client rect to the layout point
    let mut best_idx = 0;
    let mut best_dist = f64::MAX;
    for (i, r) in client_rects.iter().enumerate() {
        // distance from point to nearest point on rect
        let nearest_x = lx.clamp(r.x, r.x + r.w);
        let nearest_y = ly.clamp(r.y, r.y + r.h);
        let dx = lx - nearest_x;
        let dy = ly - nearest_y;
        let dist = dx * dx + dy * dy;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }

    let r = &client_rects[best_idx];
    // Clamp the layout point to just inside the client rect
    let entry_x = lx.clamp(r.x, r.x + r.w - 1.0);
    let entry_y = ly.clamp(r.y, r.y + r.h - 1.0);

    (entry_x, entry_y, best_idx)
}

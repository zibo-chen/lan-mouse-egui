use std::{
    cmp::Ordering,
    collections::BTreeMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use eframe::egui;

use lan_mouse_ipc::{
    ClientConfig, ClientHandle, ClientState, DEFAULT_PORT, DisplayInfo as DeviceDisplay,
    InputProfile, LayoutRect, Position, Status,
};

use super::{LanguageChoice, ThemeFamily, ThemeModeChoice};

/// Which sub-tab is active inside the overview's "real-time details" section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewTab {
    DeviceList,
    SecurityEvents,
}

/// A timestamped network / security event shown in the overview table.
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub timestamp: Instant,
    pub timestamp_label: String,
    pub description: String,
    pub source: String,
    pub association: String,
}

#[derive(Clone)]
pub struct ClientViewModel {
    pub handle: ClientHandle,
    pub hostname: Option<String>,
    pub hostname_input: String,
    pub hostname_dirty: bool,
    pub port: u16,
    pub port_input: String,
    pub port_dirty: bool,
    pub position: Position,
    pub input_profile: InputProfile,
    pub layout_rects: Vec<LayoutRect>,
    pub active: bool,
    pub resolving: bool,
    pub has_ips: bool,
    pub ips: Vec<IpAddr>,
    pub screens: Vec<DeviceDisplay>,
}

impl ClientViewModel {
    pub fn new(handle: ClientHandle, config: ClientConfig, state: ClientState) -> Self {
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
            input_profile: config.input_profile,
            layout_rects: config.layout_rects,
            active: false,
            resolving: false,
            has_ips: false,
            ips: Vec::new(),
            screens: Vec::new(),
        };
        client.apply_state(state);
        client
    }

    pub fn apply_config(&mut self, config: ClientConfig) {
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
        self.input_profile = config.input_profile;
        self.layout_rects = config.layout_rects;
    }

    pub fn apply_state(&mut self, state: ClientState) {
        self.active = state.active;
        self.resolving = state.resolving;
        self.has_ips = !state.ips.is_empty();
        self.ips = sort_ips(state.ips.into_iter().collect());
        self.screens = state.screens;
    }

    pub fn title(&self) -> String {
        self.hostname
            .clone()
            .unwrap_or_else(|| format!("client-{}", self.handle))
    }

    pub fn port_label(&self) -> String {
        if self.port == DEFAULT_PORT {
            DEFAULT_PORT.to_string()
        } else {
            self.port.to_string()
        }
    }

    pub fn status(&self) -> ClientConnectivity {
        if self.resolving {
            ClientConnectivity::Resolving
        } else if self.has_ips {
            ClientConnectivity::Reachable
        } else {
            ClientConnectivity::Unresolved
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientConnectivity {
    Reachable,
    Unresolved,
    Resolving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationPage {
    Overview,
    Clients,
    Layout,
    Security,
    Settings,
}

impl NavigationPage {
    pub const ALL: [Self; 5] = [
        Self::Overview,
        Self::Clients,
        Self::Layout,
        Self::Security,
        Self::Settings,
    ];
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub expires_at: Instant,
}

impl Toast {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            expires_at: Instant::now() + Duration::from_secs(4),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FingerprintForm {
    pub description: String,
    pub fingerprint: String,
}

#[derive(Debug, Default)]
pub struct DialogState {
    pub authorization_request: Option<String>,
    pub fingerprint_form: Option<FingerprintForm>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiPreferences {
    pub theme_family: ThemeFamily,
    pub theme_mode: ThemeModeChoice,
    pub language: LanguageChoice,
    pub navigation: NavigationPage,
    pub overview_tab: OverviewTab,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme_family: ThemeFamily::Graphite,
            theme_mode: ThemeModeChoice::System,
            language: LanguageChoice::System,
            navigation: NavigationPage::Overview,
            overview_tab: OverviewTab::DeviceList,
        }
    }
}

pub struct WorkspaceState {
    pub clients: BTreeMap<ClientHandle, ClientViewModel>,
    pub authorized_keys: BTreeMap<String, String>,
    pub local_hostname: String,
    pub local_screens: Vec<DeviceDisplay>,
    pub public_key_fingerprint: String,
    pub port: u16,
    pub port_input: String,
    pub capture_status: Status,
    pub emulation_status: Status,
    pub network_events: Vec<NetworkEvent>,
}

impl WorkspaceState {
    pub fn new(local_hostname: String) -> Self {
        Self {
            clients: BTreeMap::new(),
            authorized_keys: BTreeMap::new(),
            local_hostname,
            local_screens: Vec::new(),
            public_key_fingerprint: String::new(),
            port: DEFAULT_PORT,
            port_input: String::new(),
            capture_status: Status::Enabled,
            emulation_status: Status::Enabled,
            network_events: Vec::new(),
        }
    }

    pub fn push_event(
        &mut self,
        description: impl Into<String>,
        source: impl Into<String>,
        association: impl Into<String>,
    ) {
        let now = chrono::Local::now();
        self.network_events.push(NetworkEvent {
            timestamp: Instant::now(),
            timestamp_label: now.format("%Y-%m-%d %H:%M").to_string(),
            description: description.into(),
            source: source.into(),
            association: association.into(),
        });
        // Keep at most 200 events
        if self.network_events.len() > 200 {
            self.network_events.remove(0);
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn active_routes(&self) -> usize {
        self.clients.values().filter(|client| client.active).count()
    }

    pub fn reachable_clients(&self) -> usize {
        self.clients
            .values()
            .filter(|client| matches!(client.status(), ClientConnectivity::Reachable))
            .count()
    }

    pub fn pending_resolution(&self) -> usize {
        self.clients
            .values()
            .filter(|client| client.resolving)
            .count()
    }

    pub fn trusted_devices(&self) -> usize {
        self.authorized_keys.len()
    }
}

// ─── Screen layout model ───

/// Unique identifier for a screen within the layout canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenId {
    /// `None` = local host, `Some(handle)` = remote client.
    pub client: Option<ClientHandle>,
    /// Monitor index within that device (0-based).
    pub monitor: u32,
}

/// A single physical monitor announced by a device.
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
    pub name: String,
}

impl ScreenInfo {
    pub fn resolution_label(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

/// A screen rectangle placed on the layout canvas.  
/// Coordinates are in *logical* pixels (the canvas has its own coordinate system).
#[derive(Debug, Clone)]
pub struct LayoutScreen {
    pub id: ScreenId,
    /// Display label (e.g. "主机 (DESKTOP-XXX) - 屏幕 1 (主)").
    pub label: String,
    /// Short device label for the screen list panel.
    pub device_label: String,
    pub info: ScreenInfo,
    /// Top-left position on the layout canvas (logical coords).
    pub x: f32,
    pub y: f32,
}

impl LayoutScreen {
    /// Visual width on the canvas (scaled from real resolution).
    pub fn canvas_w(&self, scale: f32) -> f32 {
        self.info.width as f32 * scale
    }
    /// Visual height on the canvas.
    pub fn canvas_h(&self, scale: f32) -> f32 {
        self.info.height as f32 * scale
    }
    /// Canvas rect.
    pub fn canvas_rect(&self, scale: f32) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.x, self.y),
            egui::vec2(self.canvas_w(scale), self.canvas_h(scale)),
        )
    }
}

/// Persistent state for the layout editor page.
#[derive(Debug)]
pub struct LayoutState {
    pub screens: Vec<LayoutScreen>,
    pub dragging: Option<usize>,
    pub drag_offset: egui::Vec2,
    /// Dirty flag: layout has been modified since last apply.
    pub dirty: bool,
    /// Scale factor that maps real pixels → canvas pixels.
    pub scale: f32,
    /// Viewer zoom level (1.0 = 100%).
    pub zoom: f32,
    /// Viewer pan offset (canvas-space pixels).
    pub pan_offset: egui::Vec2,
    /// Whether the canvas background is being panned.
    pub panning: bool,
    /// Snap distance threshold in canvas pixels for edge snapping.
    pub snap_threshold: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            screens: Vec::new(),
            dragging: None,
            drag_offset: egui::Vec2::ZERO,
            dirty: false,
            scale: 0.15,
            zoom: 0.7,
            pan_offset: egui::Vec2::ZERO,
            panning: false,
            snap_threshold: 12.0,
        }
    }
}

impl LayoutState {
    /// Rebuild the screen list from current workspace state.
    pub fn rebuild_from_workspace(&mut self, ws: &WorkspaceState) {
        // Collect existing ids so we can preserve positions
        let old: BTreeMap<(Option<ClientHandle>, u32), (f32, f32)> = self
            .screens
            .iter()
            .map(|s| ((s.id.client, s.id.monitor), (s.x, s.y)))
            .collect();

        self.screens.clear();

        self.push_device_screens(
            &old,
            None,
            format!("主机 ({})", ws.local_hostname),
            &ws.local_screens,
            &[], // local screens use existing canvas positions or fallback
            100.0,
            150.0,
        );

        for (index, (handle, client)) in ws.clients.iter().enumerate() {
            let ip_label = client
                .ips
                .first()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| client.title());
            self.push_device_screens(
                &old,
                Some(*handle),
                format!("客户端 {} ({})", handle, ip_label),
                &client.screens,
                &client.layout_rects,
                140.0 + index as f32 * 280.0,
                340.0,
            );
        }
    }

    fn push_device_screens(
        &mut self,
        old: &BTreeMap<(Option<ClientHandle>, u32), (f32, f32)>,
        client: Option<ClientHandle>,
        device_label: String,
        displays: &[DeviceDisplay],
        layout_rects: &[LayoutRect],
        fallback_x: f32,
        fallback_y: f32,
    ) {
        if displays.is_empty() {
            return;
        }

        let min_x = displays.iter().map(|display| display.x).min().unwrap_or(0);
        let min_y = displays.iter().map(|display| display.y).min().unwrap_or(0);

        for (monitor, display) in displays.iter().enumerate() {
            let screen_id = ScreenId {
                client,
                monitor: monitor as u32,
            };

            // Priority: 1) existing canvas position, 2) saved layout rect, 3) fallback
            let (x, y) = if let Some(&(ox, oy)) =
                old.get(&(screen_id.client, screen_id.monitor))
            {
                (ox, oy)
            } else if let Some(lr) = layout_rects.get(monitor) {
                // layout_rects are in real-pixel layout space → convert to canvas coords
                (lr.x as f32 * self.scale, lr.y as f32 * self.scale)
            } else {
                let default_x = fallback_x + (display.x - min_x) as f32 * self.scale;
                let default_y = fallback_y + (display.y - min_y) as f32 * self.scale;
                (default_x, default_y)
            };

            self.screens.push(LayoutScreen {
                id: screen_id,
                label: format!("{} - {}", device_label, display.name),
                device_label: device_label.clone(),
                info: ScreenInfo {
                    x: display.x,
                    y: display.y,
                    width: display.width,
                    height: display.height,
                    primary: display.primary,
                    name: display.name.clone(),
                },
                x,
                y,
            });
        }
    }

    /// Snap a screen position to nearby edges of other screens.
    /// Returns the snapped (x, y) for the screen at `drag_idx`
    /// given proposed position `(px, py)` in canvas coords.
    pub fn snap_to_edges(&self, drag_idx: usize, px: f32, py: f32) -> (f32, f32) {
        let scale = self.scale;
        let dragged = &self.screens[drag_idx];
        let dw = dragged.canvas_w(scale);
        let dh = dragged.canvas_h(scale);
        let dragged_client = dragged.id.client;
        let thresh = self.snap_threshold;

        let mut sx = px;
        let mut sy = py;
        let mut best_dx = thresh + 1.0;
        let mut best_dy = thresh + 1.0;

        for (i, other) in self.screens.iter().enumerate() {
            // Skip screens belonging to the same device (they move together)
            if other.id.client == dragged_client {
                continue;
            }
            // Also skip if this screen is being dragged as a group member
            if Some(i) == self.dragging {
                continue;
            }
            let or = other.canvas_rect(scale);

            // Horizontal edge snapping
            // Right edge of dragged → left edge of other
            let d = (px + dw - or.left()).abs();
            if d < best_dx {
                best_dx = d;
                sx = or.left() - dw;
            }
            // Left edge of dragged → right edge of other
            let d = (px - or.right()).abs();
            if d < best_dx {
                best_dx = d;
                sx = or.right();
            }
            // Left edge of dragged → left edge of other
            let d = (px - or.left()).abs();
            if d < best_dx {
                best_dx = d;
                sx = or.left();
            }
            // Right edge of dragged → right edge of other
            let d = (px + dw - or.right()).abs();
            if d < best_dx {
                best_dx = d;
                sx = or.right() - dw;
            }

            // Vertical edge snapping
            // Bottom of dragged → top of other
            let d = (py + dh - or.top()).abs();
            if d < best_dy {
                best_dy = d;
                sy = or.top() - dh;
            }
            // Top of dragged → bottom of other
            let d = (py - or.bottom()).abs();
            if d < best_dy {
                best_dy = d;
                sy = or.bottom();
            }
            // Top of dragged → top of other
            let d = (py - or.top()).abs();
            if d < best_dy {
                best_dy = d;
                sy = or.top();
            }
            // Bottom of dragged → bottom of other
            let d = (py + dh - or.bottom()).abs();
            if d < best_dy {
                best_dy = d;
                sy = or.bottom() - dh;
            }
        }

        // Only snap if within threshold
        if best_dx > thresh {
            sx = px;
        }
        if best_dy > thresh {
            sy = py;
        }
        (sx, sy)
    }

    /// Auto-arrange screens in a tidy row.
    pub fn auto_arrange(&mut self) {
        let gap = 12.0;
        let mut x = 50.0;
        let y = 120.0;
        for screen in &mut self.screens {
            screen.x = x;
            screen.y = y;
            x += screen.canvas_w(self.scale) + gap;
        }
        self.dirty = true;
    }

    /// Apply layout rects synced from a remote peer.
    /// `local_rects` = rects for local host screens (from remote's perspective),
    /// `remote_rects` = rects for the peer's screens.
    pub fn apply_synced_rects(
        &mut self,
        peer_handle: ClientHandle,
        local_rects: &[LayoutRect],
        peer_rects: &[LayoutRect],
    ) {
        // Update local host screen positions
        let local_screens: Vec<usize> = self
            .screens
            .iter()
            .enumerate()
            .filter(|(_, s)| s.id.client.is_none())
            .map(|(i, _)| i)
            .collect();
        for (i, &idx) in local_screens.iter().enumerate() {
            if let Some(lr) = local_rects.get(i) {
                self.screens[idx].x = lr.x as f32 * self.scale;
                self.screens[idx].y = lr.y as f32 * self.scale;
            }
        }

        // Update the peer's screen positions
        let peer_screens: Vec<usize> = self
            .screens
            .iter()
            .enumerate()
            .filter(|(_, s)| s.id.client == Some(peer_handle))
            .map(|(i, _)| i)
            .collect();
        for (i, &idx) in peer_screens.iter().enumerate() {
            if let Some(lr) = peer_rects.get(i) {
                self.screens[idx].x = lr.x as f32 * self.scale;
                self.screens[idx].y = lr.y as f32 * self.scale;
            }
        }
    }

    /// Compute a zoom level that fits all screens into the given canvas size.
    pub fn fit_zoom(&self, canvas_size: egui::Vec2) -> f32 {
        if self.screens.is_empty() {
            return 1.0;
        }
        let scale = self.scale;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for screen in &self.screens {
            let r = screen.canvas_rect(scale);
            min_x = min_x.min(r.left());
            min_y = min_y.min(r.top());
            max_x = max_x.max(r.right());
            max_y = max_y.max(r.bottom());
        }
        let content_w = (max_x - min_x).max(1.0);
        let content_h = (max_y - min_y).max(1.0);
        let margin = 40.0;
        let zx = (canvas_size.x - margin * 2.0) / content_w;
        let zy = (canvas_size.y - margin * 2.0) / content_h;
        zx.min(zy).clamp(0.1, 3.0)
    }

    /// Compute a pan offset that centers all screens in the canvas.
    pub fn center_pan(&self, canvas_size: egui::Vec2) -> egui::Vec2 {
        if self.screens.is_empty() {
            return egui::Vec2::ZERO;
        }
        let scale = self.scale;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for screen in &self.screens {
            let r = screen.canvas_rect(scale);
            min_x = min_x.min(r.left());
            min_y = min_y.min(r.top());
            max_x = max_x.max(r.right());
            max_y = max_y.max(r.bottom());
        }
        let content_center = egui::vec2((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
        let canvas_center = egui::vec2(canvas_size.x * 0.5, canvas_size.y * 0.5);
        canvas_center - content_center * self.zoom
    }

    pub fn device_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for s in &self.screens {
            seen.insert(s.id.client);
        }
        seen.len()
    }

    pub fn screen_count(&self) -> usize {
        self.screens.len()
    }

    /// Derive the set of adjacent edge positions for each client by examining
    /// per-screen-pair adjacency. Returns multiple positions when a client's
    /// screens touch different edges of different local screens.
    pub fn derive_client_positions(&self) -> BTreeMap<ClientHandle, Vec<Position>> {
        let mut result: BTreeMap<ClientHandle, Vec<Position>> = BTreeMap::new();
        let scale = self.scale;

        let local_screens: Vec<_> = self
            .screens
            .iter()
            .filter(|s| s.id.client.is_none())
            .collect();

        if local_screens.is_empty() {
            return result;
        }

        for client_handle in self
            .screens
            .iter()
            .filter_map(|s| s.id.client)
            .collect::<std::collections::BTreeSet<_>>()
        {
            let client_screens: Vec<_> = self
                .screens
                .iter()
                .filter(|s| s.id.client == Some(client_handle))
                .collect();

            let mut positions = Vec::new();

            // For each pair of (local screen, client screen), check adjacency.
            // An adjacency requires: small gap on one axis AND overlap on the other.
            const GAP_THRESHOLD: f32 = 32.0; // max pixel gap to consider "adjacent"

            for local in &local_screens {
                let lr = local.canvas_rect(scale);
                for client in &client_screens {
                    let cr = client.canvas_rect(scale);

                    let v_overlap = axis_overlap(lr.top(), lr.bottom(), cr.top(), cr.bottom());
                    let h_overlap = axis_overlap(lr.left(), lr.right(), cr.left(), cr.right());

                    // Client is to the RIGHT of local screen
                    if v_overlap > 0.0 && (cr.left() - lr.right()).abs() < GAP_THRESHOLD {
                        if !positions.contains(&Position::Right) {
                            positions.push(Position::Right);
                        }
                    }
                    // Client is to the LEFT of local screen
                    if v_overlap > 0.0 && (lr.left() - cr.right()).abs() < GAP_THRESHOLD {
                        if !positions.contains(&Position::Left) {
                            positions.push(Position::Left);
                        }
                    }
                    // Client is ABOVE local screen
                    if h_overlap > 0.0 && (lr.top() - cr.bottom()).abs() < GAP_THRESHOLD {
                        if !positions.contains(&Position::Top) {
                            positions.push(Position::Top);
                        }
                    }
                    // Client is BELOW local screen
                    if h_overlap > 0.0 && (cr.top() - lr.bottom()).abs() < GAP_THRESHOLD {
                        if !positions.contains(&Position::Bottom) {
                            positions.push(Position::Bottom);
                        }
                    }
                }
            }

            // Fallback: if no per-screen adjacency found, use bounding-box method
            if positions.is_empty() {
                if let (Some(local_rect), Some(device_rect)) = (
                    self.device_rect(None),
                    self.device_rect(Some(client_handle)),
                ) {
                    positions.push(derive_position_from_rects(local_rect, device_rect));
                }
            }

            if !positions.is_empty() {
                result.insert(client_handle, positions);
            }
        }

        result
    }

    /// Convert each client's screen canvas positions into `LayoutRect`s (in real pixels).
    pub fn derive_client_layout_rects(&self) -> BTreeMap<ClientHandle, Vec<LayoutRect>> {
        let mut result: BTreeMap<ClientHandle, Vec<LayoutRect>> = BTreeMap::new();
        for screen in &self.screens {
            if let Some(handle) = screen.id.client {
                result.entry(handle).or_default().push(LayoutRect {
                    x: (screen.x / self.scale) as f64,
                    y: (screen.y / self.scale) as f64,
                    w: screen.info.width as f64,
                    h: screen.info.height as f64,
                });
            }
        }
        result
    }

    /// Convert local (host) screen canvas positions into `LayoutRect`s (in real pixels).
    pub fn derive_local_layout_rects(&self) -> Vec<LayoutRect> {
        self.screens
            .iter()
            .filter(|s| s.id.client.is_none())
            .map(|screen| LayoutRect {
                x: (screen.x / self.scale) as f64,
                y: (screen.y / self.scale) as f64,
                w: screen.info.width as f64,
                h: screen.info.height as f64,
            })
            .collect()
    }

    fn device_rect(&self, client: Option<ClientHandle>) -> Option<egui::Rect> {
        let mut iter = self
            .screens
            .iter()
            .filter(|screen| screen.id.client == client);
        let first = iter.next()?;
        let mut rect = first.canvas_rect(self.scale);
        for screen in iter {
            rect = rect.union(screen.canvas_rect(self.scale));
        }
        Some(rect)
    }
}

fn derive_position_from_rects(local_rect: egui::Rect, device_rect: egui::Rect) -> Position {
    let fallback = fallback_position_from_centers(local_rect, device_rect);
    let vertical_overlap = axis_overlap(
        local_rect.top(),
        local_rect.bottom(),
        device_rect.top(),
        device_rect.bottom(),
    );
    let horizontal_overlap = axis_overlap(
        local_rect.left(),
        local_rect.right(),
        device_rect.left(),
        device_rect.right(),
    );

    [
        (
            Position::Left,
            (local_rect.left() - device_rect.right()).abs(),
            vertical_overlap,
        ),
        (
            Position::Right,
            (device_rect.left() - local_rect.right()).abs(),
            vertical_overlap,
        ),
        (
            Position::Top,
            (local_rect.top() - device_rect.bottom()).abs(),
            horizontal_overlap,
        ),
        (
            Position::Bottom,
            (device_rect.top() - local_rect.bottom()).abs(),
            horizontal_overlap,
        ),
    ]
    .into_iter()
    .min_by(|a, b| compare_position_candidates(*a, *b, fallback))
    .map(|candidate| candidate.0)
    .unwrap_or(fallback)
}

fn compare_position_candidates(
    a: (Position, f32, f32),
    b: (Position, f32, f32),
    fallback: Position,
) -> Ordering {
    a.1.partial_cmp(&b.1)
        .unwrap_or(Ordering::Equal)
        .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal))
        .then_with(|| {
            position_preference_rank(a.0, fallback).cmp(&position_preference_rank(b.0, fallback))
        })
}

fn position_preference_rank(position: Position, fallback: Position) -> u8 {
    u8::from(position != fallback)
}

fn fallback_position_from_centers(local_rect: egui::Rect, device_rect: egui::Rect) -> Position {
    let local_center = local_rect.center();
    let device_center = device_rect.center();
    let dx = device_center.x - local_center.x;
    let dy = device_center.y - local_center.y;

    if dx.abs() >= dy.abs() {
        if dx >= 0.0 {
            Position::Right
        } else {
            Position::Left
        }
    } else if dy >= 0.0 {
        Position::Bottom
    } else {
        Position::Top
    }
}

fn axis_overlap(start_a: f32, end_a: f32, start_b: f32, end_b: f32) -> f32 {
    (end_a.min(end_b) - start_a.max(start_b)).max(0.0)
}

pub fn parse_port_input(value: &str) -> u16 {
    value.trim().parse::<u16>().unwrap_or(DEFAULT_PORT)
}

pub fn port_to_input(port: u16) -> String {
    if port == DEFAULT_PORT {
        String::new()
    } else {
        port.to_string()
    }
}

fn sort_ips(mut ips: Vec<IpAddr>) -> Vec<IpAddr> {
    ips.sort_by_key(IpAddr::to_string);
    ips
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_screen(
        client: Option<ClientHandle>,
        monitor: u32,
        width: u32,
        height: u32,
        x: f32,
        y: f32,
    ) -> LayoutScreen {
        LayoutScreen {
            id: ScreenId { client, monitor },
            label: format!("screen-{monitor}"),
            device_label: format!("device-{monitor}"),
            info: ScreenInfo {
                x: 0,
                y: 0,
                width,
                height,
                primary: monitor == 0,
                name: format!("display-{monitor}"),
            },
            x,
            y,
        }
    }

    #[test]
    fn derive_client_positions_prefers_nearest_edge_over_center_quadrant() {
        let layout = LayoutState {
            scale: 1.0,
            screens: vec![
                make_screen(None, 0, 100, 200, 0.0, 0.0),
                make_screen(None, 1, 100, 200, 0.0, 200.0),
                make_screen(Some(1), 0, 100, 100, 100.0, 300.0),
            ],
            ..Default::default()
        };

        assert_eq!(
            layout.derive_client_positions().get(&1),
            Some(&vec![Position::Right])
        );
    }

    #[test]
    fn derive_client_positions_uses_overlap_to_resolve_corner_touching() {
        let layout = LayoutState {
            scale: 1.0,
            screens: vec![
                make_screen(None, 0, 400, 100, 0.0, 0.0),
                make_screen(Some(1), 0, 100, 100, 300.0, -100.0),
            ],
            ..Default::default()
        };

        assert_eq!(
            layout.derive_client_positions().get(&1),
            Some(&vec![Position::Top])
        );
    }

    #[test]
    fn derive_client_positions_falls_back_to_center_for_diagonal_layouts() {
        let layout = LayoutState {
            scale: 1.0,
            screens: vec![
                make_screen(None, 0, 100, 100, 0.0, 0.0),
                make_screen(Some(1), 0, 100, 100, 150.0, -150.0),
            ],
            ..Default::default()
        };

        assert_eq!(
            layout.derive_client_positions().get(&1),
            Some(&vec![Position::Right])
        );
    }
}

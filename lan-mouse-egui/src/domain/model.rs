use std::{
    collections::BTreeMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use lan_mouse_ipc::{
    ClientConfig, ClientHandle, ClientState, DEFAULT_PORT, InputProfile, Position, Status,
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
    pub active: bool,
    pub resolving: bool,
    pub has_ips: bool,
    pub ips: Vec<IpAddr>,
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
            active: false,
            resolving: false,
            has_ips: false,
            ips: Vec::new(),
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
    }

    pub fn apply_state(&mut self, state: ClientState) {
        self.active = state.active;
        self.resolving = state.resolving;
        self.has_ips = !state.ips.is_empty();
        self.ips = sort_ips(state.ips.into_iter().collect());
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
    Security,
    Settings,
}

impl NavigationPage {
    pub const ALL: [Self; 4] = [
        Self::Overview,
        Self::Clients,
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

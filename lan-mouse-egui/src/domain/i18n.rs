use std::env;

use lan_mouse_ipc::{ClientPlatform, Position, ScrollMode, ShortcutMode};

use super::{NavigationPage, ThemeFamily, ThemeModeChoice};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageChoice {
    System,
    English,
    SimplifiedChinese,
}

impl LanguageChoice {
    pub const ALL: [Self; 3] = [Self::System, Self::English, Self::SimplifiedChinese];

    pub fn resolve(self) -> Language {
        match self {
            Self::System => detect_language(),
            Self::English => Language::English,
            Self::SimplifiedChinese => Language::SimplifiedChinese,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    SimplifiedChinese,
}

pub struct Catalog {
    pub app_title: &'static str,
    pub app_tagline: &'static str,
    pub hero_heading: &'static str,
    pub hero_body: &'static str,
    pub tray_open: &'static str,
    pub tray_hide: &'static str,
    pub tray_quit: &'static str,
    pub label_host: &'static str,
    pub label_clients: &'static str,
    pub label_active_routes: &'static str,
    pub label_trusted_devices: &'static str,
    pub label_listen_port: &'static str,
    pub label_capture: &'static str,
    pub label_emulation: &'static str,
    pub label_network: &'static str,
    pub label_desktop: &'static str,
    pub label_service_controls: &'static str,
    pub label_identity: &'static str,
    pub label_allowlist: &'static str,
    pub label_client_workspace: &'static str,
    pub label_input_profile: &'static str,
    pub label_theme_family: &'static str,
    pub label_appearance: &'static str,
    pub label_language: &'static str,
    pub label_background_behavior: &'static str,
    pub label_quick_actions: &'static str,
    pub label_connection_model: &'static str,
    pub label_recent_security: &'static str,
    pub label_hostname: &'static str,
    pub label_port: &'static str,
    pub label_screen_edge: &'static str,
    pub label_known_ips: &'static str,
    pub label_remote_platform: &'static str,
    pub label_shortcut_style: &'static str,
    pub label_scroll_style: &'static str,
    pub label_horizontal_scroll: &'static str,
    pub label_vertical_scroll: &'static str,
    pub label_local_fingerprint: &'static str,
    pub label_description: &'static str,
    pub label_fingerprint: &'static str,
    pub label_route_input: &'static str,
    pub label_selected_client: &'static str,
    pub label_status: &'static str,
    pub status_capture_ready: &'static str,
    pub status_capture_offline: &'static str,
    pub status_emulation_ready: &'static str,
    pub status_emulation_offline: &'static str,
    pub status_tray_enabled: &'static str,
    pub status_window_visible: &'static str,
    pub status_background_mode: &'static str,
    pub status_enabled: &'static str,
    pub status_disabled: &'static str,
    pub status_active: &'static str,
    pub status_idle: &'static str,
    pub status_reachable: &'static str,
    pub status_unresolved: &'static str,
    pub status_resolving: &'static str,
    pub action_hide_to_tray: &'static str,
    pub action_minimize: &'static str,
    pub action_copy_hostname: &'static str,
    pub action_copy_fingerprint: &'static str,
    pub action_quit: &'static str,
    pub action_add_client: &'static str,
    pub action_add_fingerprint: &'static str,
    pub action_resolve_dns: &'static str,
    pub action_delete_client: &'static str,
    pub action_remove: &'static str,
    pub action_authorize: &'static str,
    pub action_dismiss: &'static str,
    pub action_save: &'static str,
    pub action_cancel: &'static str,
    pub action_apply: &'static str,
    pub action_reset: &'static str,
    pub action_retry: &'static str,
    pub action_create_first_client: &'static str,
    pub empty_clients: &'static str,
    pub empty_clients_hint: &'static str,
    pub empty_security: &'static str,
    pub empty_security_hint: &'static str,
    pub empty_addresses: &'static str,
    pub empty_selected_client: &'static str,
    pub empty_selected_client_hint: &'static str,
    pub fingerprint_pending: &'static str,
    pub auth_request_title: &'static str,
    pub auth_request_body: &'static str,
    pub input_profile_hint: &'static str,
    pub connection_model_hint: &'static str,
    pub recent_security_hint: &'static str,
    pub default_port_hint: &'static str,
    pub background_hint_tray: &'static str,
    pub background_hint_minimize: &'static str,
    pub toast_hostname_copied: &'static str,
    pub toast_fingerprint_copied: &'static str,
    pub toast_failed_request: &'static str,
    pub toast_device_connected: &'static str,
    pub toast_device_entered: &'static str,
    pub toast_disconnected: &'static str,
    pub toast_no_such_client: &'static str,
    pub close_notice_tray: &'static str,
    pub close_notice_minimize: &'static str,
    // New labels for redesigned overview
    pub label_overview_topology: &'static str,
    pub label_quick_actions_bar: &'static str,
    pub label_realtime_details: &'static str,
    pub label_device_list: &'static str,
    pub label_security_events: &'static str,
    pub label_connected_count: &'static str,
    pub label_routes_count: &'static str,
    pub label_capture_control: &'static str,
    pub label_emulation_control: &'static str,
    pub action_scan: &'static str,
    pub col_status_led: &'static str,
    pub col_name: &'static str,
    pub col_ip_port: &'static str,
    pub col_timestamp: &'static str,
    pub col_event_desc: &'static str,
    pub col_association: &'static str,
    pub topology_local: &'static str,
    pub topology_remote: &'static str,
    pub topology_storage: &'static str,
    // Navigation icon labels
    pub nav_connection_mode: &'static str,
    // Layout page
    pub nav_layout: &'static str,
    pub nav_layout_subtitle: &'static str,
    pub label_screen_layout: &'static str,
    pub label_detected_screens: &'static str,
    #[allow(dead_code)]
    pub label_snap_to_grid: &'static str,
    #[allow(dead_code)]
    pub label_show_grid: &'static str,
    pub action_auto_arrange: &'static str,
    pub label_primary: &'static str,
    pub label_local_host: &'static str,
    pub layout_status_devices: &'static str,
    pub layout_status_screens: &'static str,
}

impl Catalog {
    pub fn nav_label(&self, page: NavigationPage) -> &'static str {
        match page {
            NavigationPage::Overview => self.nav_connection_mode,
            NavigationPage::Clients => match self.app_title {
                "Lan Mouse" => "Clients",
                _ => "客户端",
            },
            NavigationPage::Layout => self.nav_layout,
            NavigationPage::Security => match self.app_title {
                "Lan Mouse" => "Security",
                _ => "安全",
            },
            NavigationPage::Settings => match self.app_title {
                "Lan Mouse" => "Settings",
                _ => "设置",
            },
        }
    }

    pub fn nav_subtitle(&self, page: NavigationPage) -> &'static str {
        match page {
            NavigationPage::Overview => match self.app_title {
                "Lan Mouse" => "Service health, identity, and desktop controls",
                _ => "服务状态、设备身份和桌面控制中心",
            },
            NavigationPage::Clients => match self.app_title {
                "Lan Mouse" => "Route remote machines and tune their behavior",
                _ => "管理远程机器并调整它们的行为",
            },
            NavigationPage::Layout => self.nav_layout_subtitle,
            NavigationPage::Security => match self.app_title {
                "Lan Mouse" => "Trusted fingerprints and incoming authorization",
                _ => "可信指纹与传入授权管理",
            },
            NavigationPage::Settings => match self.app_title {
                "Lan Mouse" => "Theme, language, network, and background behavior",
                _ => "主题、语言、网络和后台行为",
            },
        }
    }

    pub fn language_choice(&self, choice: LanguageChoice) -> &'static str {
        match choice {
            LanguageChoice::System => match self.app_title {
                "Lan Mouse" => "Follow system",
                _ => "跟随系统",
            },
            LanguageChoice::English => "English",
            LanguageChoice::SimplifiedChinese => "简体中文",
        }
    }

    pub fn theme_family(&self, family: ThemeFamily) -> &'static str {
        match family {
            ThemeFamily::Graphite => match self.app_title {
                "Lan Mouse" => "Graphite",
                _ => "石墨",
            },
            ThemeFamily::Sand => match self.app_title {
                "Lan Mouse" => "Sand",
                _ => "砂岩",
            },
            ThemeFamily::Ocean => match self.app_title {
                "Lan Mouse" => "Ocean",
                _ => "潮汐",
            },
        }
    }

    pub fn theme_mode(&self, mode: ThemeModeChoice) -> &'static str {
        match mode {
            ThemeModeChoice::System => match self.app_title {
                "Lan Mouse" => "System",
                _ => "系统",
            },
            ThemeModeChoice::Light => match self.app_title {
                "Lan Mouse" => "Light",
                _ => "浅色",
            },
            ThemeModeChoice::Dark => match self.app_title {
                "Lan Mouse" => "Dark",
                _ => "深色",
            },
        }
    }

    pub fn position(&self, position: Position) -> &'static str {
        match position {
            Position::Left => match self.app_title {
                "Lan Mouse" => "Left",
                _ => "左侧",
            },
            Position::Right => match self.app_title {
                "Lan Mouse" => "Right",
                _ => "右侧",
            },
            Position::Top => match self.app_title {
                "Lan Mouse" => "Top",
                _ => "上方",
            },
            Position::Bottom => match self.app_title {
                "Lan Mouse" => "Bottom",
                _ => "下方",
            },
        }
    }

    pub fn platform(&self, platform: ClientPlatform) -> &'static str {
        match platform {
            ClientPlatform::Unknown => match self.app_title {
                "Lan Mouse" => "Unknown",
                _ => "未知",
            },
            ClientPlatform::Windows => "Windows",
            ClientPlatform::Macos => "macOS",
            ClientPlatform::Linux => "Linux",
        }
    }

    pub fn shortcut_mode(&self, mode: ShortcutMode) -> &'static str {
        match mode {
            ShortcutMode::Physical => match self.app_title {
                "Lan Mouse" => "Physical passthrough",
                _ => "物理透传",
            },
            ShortcutMode::SourceNative => match self.app_title {
                "Lan Mouse" => "Source-native shortcuts",
                _ => "源端原生快捷键",
            },
        }
    }

    pub fn scroll_mode(&self, mode: ScrollMode) -> &'static str {
        match mode {
            ScrollMode::Physical => match self.app_title {
                "Lan Mouse" => "Physical direction",
                _ => "物理方向",
            },
            ScrollMode::TargetNative => match self.app_title {
                "Lan Mouse" => "Target-native direction",
                _ => "目标端原生方向",
            },
        }
    }
}

pub fn catalog(language: Language) -> &'static Catalog {
    match language {
        Language::English => &ENGLISH,
        Language::SimplifiedChinese => &SIMPLIFIED_CHINESE,
    }
}

fn detect_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(key) {
            let lower = value.to_ascii_lowercase();
            if lower.contains("zh") {
                return Language::SimplifiedChinese;
            }
        }
    }
    Language::English
}

static ENGLISH: Catalog = Catalog {
    app_title: "Lan Mouse",
    app_tagline: "A desktop command center for keyboard and mouse sharing.",
    hero_heading: "One desktop console for every edge handoff.",
    hero_body: "Switch clients, trust devices, and tune cross-platform behavior without leaving a single desktop workspace.",
    tray_open: "Open Lan Mouse",
    tray_hide: "Hide Window",
    tray_quit: "Quit",
    label_host: "Host",
    label_clients: "Clients",
    label_active_routes: "Active routes",
    label_trusted_devices: "Trusted devices",
    label_listen_port: "Listen port",
    label_capture: "Capture",
    label_emulation: "Emulation",
    label_network: "Network",
    label_desktop: "Desktop",
    label_service_controls: "Service controls",
    label_identity: "Device identity",
    label_allowlist: "Trusted fingerprints",
    label_client_workspace: "Client workspace",
    label_input_profile: "Input profile",
    label_theme_family: "Theme family",
    label_appearance: "Appearance",
    label_language: "Language",
    label_background_behavior: "Background behavior",
    label_quick_actions: "Quick actions",
    label_connection_model: "Connection model",
    label_recent_security: "Security snapshot",
    label_hostname: "Hostname",
    label_port: "Port",
    label_screen_edge: "Screen edge",
    label_known_ips: "Known IPs",
    label_remote_platform: "Remote platform",
    label_shortcut_style: "Shortcut style",
    label_scroll_style: "Scroll style",
    label_horizontal_scroll: "Horizontal scroll",
    label_vertical_scroll: "Vertical scroll",
    label_local_fingerprint: "Local fingerprint",
    label_description: "Description",
    label_fingerprint: "Fingerprint",
    label_route_input: "Route input",
    label_selected_client: "Selected client",
    label_status: "Status",
    status_capture_ready: "Capture ready",
    status_capture_offline: "Capture offline",
    status_emulation_ready: "Emulation ready",
    status_emulation_offline: "Emulation offline",
    status_tray_enabled: "Tray enabled",
    status_window_visible: "Window visible",
    status_background_mode: "Background mode",
    status_enabled: "Enabled",
    status_disabled: "Disabled",
    status_active: "Active",
    status_idle: "Idle",
    status_reachable: "Reachable",
    status_unresolved: "Unresolved",
    status_resolving: "Resolving DNS",
    action_hide_to_tray: "Hide to tray",
    action_minimize: "Minimize",
    action_copy_hostname: "Copy hostname",
    action_copy_fingerprint: "Copy fingerprint",
    action_quit: "Quit",
    action_add_client: "Add client",
    action_add_fingerprint: "Add fingerprint",
    action_resolve_dns: "Resolve DNS",
    action_delete_client: "Delete client",
    action_remove: "Remove",
    action_authorize: "Authorize",
    action_dismiss: "Dismiss",
    action_save: "Save",
    action_cancel: "Cancel",
    action_apply: "Apply",
    action_reset: "Reset",
    action_retry: "Retry",
    action_create_first_client: "Create first client",
    empty_clients: "No configured clients yet.",
    empty_clients_hint: "Create a client and map it to a screen edge to start routing input across devices.",
    empty_security: "No authorized fingerprints yet.",
    empty_security_hint: "Authorized devices appear here after you approve them or add fingerprints manually.",
    empty_addresses: "No addresses discovered yet.",
    empty_selected_client: "No client selected.",
    empty_selected_client_hint: "Choose a client from the roster to edit its endpoint and input profile.",
    fingerprint_pending: "Waiting for the service to publish the local fingerprint.",
    auth_request_title: "Authorization required",
    auth_request_body: "An incoming device is asking for access. Review the fingerprint before allowing it.",
    input_profile_hint: "Set this to the remote machine you are controlling from so Lan Mouse can remap scroll direction and primary shortcuts cleanly.",
    connection_model_hint: "Only one side is active at a time. An active client receives events, while inactive clients stay ready to hand control back.",
    recent_security_hint: "Trusted fingerprints prevent unexpected devices from taking control. Keep the allowlist tight.",
    default_port_hint: "Leave the field empty to use the default port.",
    background_hint_tray: "Closing the window keeps Lan Mouse running in the background. Reopen or quit it from the tray menu.",
    background_hint_minimize: "Closing the window keeps Lan Mouse running and minimizes it to the taskbar or dock.",
    toast_hostname_copied: "Hostname copied",
    toast_fingerprint_copied: "Fingerprint copied",
    toast_failed_request: "Failed to send request",
    toast_device_connected: "Device connected",
    toast_device_entered: "Device entered",
    toast_disconnected: "Disconnected",
    toast_no_such_client: "No such client",
    close_notice_tray: "Lan Mouse is still running in the background. Reopen or quit it from the tray menu.",
    close_notice_minimize: "Lan Mouse is still running in the background. Restore it from the taskbar or dock.",
    label_overview_topology: "Overview — Local Network Topology",
    label_quick_actions_bar: "Quick Actions",
    label_realtime_details: "Real-time Details",
    label_device_list: "Device List",
    label_security_events: "Network & Security Events",
    label_connected_count: "Connected",
    label_routes_count: "Routes",
    label_capture_control: "Capture Control",
    label_emulation_control: "Emulation Control",
    action_scan: "Scan",
    col_status_led: "Status",
    col_name: "Name",
    col_ip_port: "IP/Port",
    col_timestamp: "Timestamp",
    col_event_desc: "Event",
    col_association: "Association",
    topology_local: "Local",
    topology_remote: "Remote Desktop",
    topology_storage: "Storage",
    nav_connection_mode: "Connection Mode",
    nav_layout: "Layout",
    nav_layout_subtitle: "Drag screens to arrange mouse handoff between devices",
    label_screen_layout: "Screen Layout Manager",
    label_detected_screens: "Detected Screens",
    label_snap_to_grid: "Snap to Grid",
    label_show_grid: "Show Grid",
    action_auto_arrange: "Auto Arrange",
    label_primary: "Primary",
    label_local_host: "Local Host",
    layout_status_devices: "devices",
    layout_status_screens: "screens",
};

static SIMPLIFIED_CHINESE: Catalog = Catalog {
    app_title: "局域网鼠标",
    app_tagline: "一个面向桌面的键鼠共享控制中心。",
    hero_heading: "把所有屏幕边缘切换收进一个桌面工作台。",
    hero_body: "在同一个桌面界面里管理客户端、信任设备，并微调跨平台输入行为。",
    tray_open: "打开局域网鼠标",
    tray_hide: "隐藏窗口",
    tray_quit: "退出",
    label_host: "主机",
    label_clients: "客户端",
    label_active_routes: "活动路由",
    label_trusted_devices: "可信设备",
    label_listen_port: "监听端口",
    label_capture: "输入捕获",
    label_emulation: "输入仿真",
    label_network: "网络",
    label_desktop: "桌面",
    label_service_controls: "服务控制",
    label_identity: "设备身份",
    label_allowlist: "可信指纹",
    label_client_workspace: "客户端工作区",
    label_input_profile: "输入画像",
    label_theme_family: "主题风格",
    label_appearance: "外观模式",
    label_language: "语言",
    label_background_behavior: "后台行为",
    label_quick_actions: "快捷操作",
    label_connection_model: "连接模型",
    label_recent_security: "安全概览",
    label_hostname: "主机名",
    label_port: "端口",
    label_screen_edge: "屏幕边缘",
    label_known_ips: "已知 IP",
    label_remote_platform: "远端平台",
    label_shortcut_style: "快捷键风格",
    label_scroll_style: "滚动风格",
    label_horizontal_scroll: "水平滚动",
    label_vertical_scroll: "垂直滚动",
    label_local_fingerprint: "本机指纹",
    label_description: "描述",
    label_fingerprint: "指纹",
    label_route_input: "路由输入",
    label_selected_client: "当前客户端",
    label_status: "状态",
    status_capture_ready: "捕获已就绪",
    status_capture_offline: "捕获离线",
    status_emulation_ready: "仿真已就绪",
    status_emulation_offline: "仿真离线",
    status_tray_enabled: "托盘已启用",
    status_window_visible: "窗口可见",
    status_background_mode: "后台模式",
    status_enabled: "已启用",
    status_disabled: "已禁用",
    status_active: "活动中",
    status_idle: "空闲",
    status_reachable: "可达",
    status_unresolved: "未解析",
    status_resolving: "DNS 解析中",
    action_hide_to_tray: "隐藏到托盘",
    action_minimize: "最小化",
    action_copy_hostname: "复制主机名",
    action_copy_fingerprint: "复制指纹",
    action_quit: "退出",
    action_add_client: "添加客户端",
    action_add_fingerprint: "添加指纹",
    action_resolve_dns: "解析 DNS",
    action_delete_client: "删除客户端",
    action_remove: "移除",
    action_authorize: "授权",
    action_dismiss: "忽略",
    action_save: "保存",
    action_cancel: "取消",
    action_apply: "应用",
    action_reset: "重置",
    action_retry: "重试",
    action_create_first_client: "创建第一个客户端",
    empty_clients: "还没有配置任何客户端。",
    empty_clients_hint: "先创建一个客户端并绑定到屏幕边缘，然后就可以开始在设备间路由输入。",
    empty_security: "还没有任何已授权指纹。",
    empty_security_hint: "批准设备或手动添加指纹后，它们会出现在这里。",
    empty_addresses: "暂未发现地址。",
    empty_selected_client: "尚未选择客户端。",
    empty_selected_client_hint: "从左侧列表选择一个客户端，即可编辑它的端点与输入画像。",
    fingerprint_pending: "正在等待服务发布本机指纹。",
    auth_request_title: "需要授权",
    auth_request_body: "有设备正在请求访问。请先核对指纹，再决定是否允许。",
    input_profile_hint: "把这里设置成你当前正在控制的远端机器类型，Lan Mouse 会据此更自然地处理滚动方向和主快捷键。",
    connection_model_hint: "任意时刻只有一端处于活动状态。活动客户端接收事件，其余客户端保持待命以便把控制权切回。",
    recent_security_hint: "可信指纹用于阻止意外设备接管控制。建议始终保持授权列表精简。",
    default_port_hint: "留空则使用默认端口。",
    background_hint_tray: "关闭窗口不会停止 Lan Mouse，它会继续在后台运行。你可以从托盘菜单重新打开或退出。",
    background_hint_minimize: "关闭窗口不会停止 Lan Mouse，它会继续在后台运行，并最小化到任务栏或 Dock。",
    toast_hostname_copied: "主机名已复制",
    toast_fingerprint_copied: "指纹已复制",
    toast_failed_request: "发送请求失败",
    toast_device_connected: "设备已连接",
    toast_device_entered: "设备已进入",
    toast_disconnected: "已断开连接",
    toast_no_such_client: "客户端不存在",
    close_notice_tray: "Lan Mouse 仍在后台运行。你可以从托盘菜单重新打开或退出。",
    close_notice_minimize: "Lan Mouse 仍在后台运行。你可以从任务栏或 Dock 恢复窗口。",
    label_overview_topology: "概览 - 本地网络拓扑",
    label_quick_actions_bar: "快速操作",
    label_realtime_details: "实时详细信息",
    label_device_list: "设备列表",
    label_security_events: "网络与安全事件",
    label_connected_count: "连接端",
    label_routes_count: "路由数",
    label_capture_control: "捕获控制",
    label_emulation_control: "仿真控制",
    action_scan: "Scan",
    col_status_led: "状态LED",
    col_name: "名称",
    col_ip_port: "IP/端口",
    col_timestamp: "时间戳",
    col_event_desc: "事件描述",
    col_association: "关联",
    topology_local: "主操",
    topology_remote: "远程台式机",
    topology_storage: "服务存储",
    nav_connection_mode: "连接模式",
    nav_layout: "布局",
    nav_layout_subtitle: "拖动屏幕安排设备之间的鼠标切换方式",
    label_screen_layout: "屏幕布局管理",
    label_detected_screens: "已检测屏幕",
    label_snap_to_grid: "对齐网格",
    label_show_grid: "显示网格",
    action_auto_arrange: "自动排列",
    label_primary: "主屏",
    label_local_host: "本机",
    layout_status_devices: "台设备",
    layout_status_screens: "个屏幕",
};

#[cfg(test)]
mod tests {
    use super::{Language, LanguageChoice};

    #[test]
    fn explicit_language_choice_is_stable() {
        assert_eq!(LanguageChoice::English.resolve(), Language::English);
        assert_eq!(
            LanguageChoice::SimplifiedChinese.resolve(),
            Language::SimplifiedChinese
        );
    }
}

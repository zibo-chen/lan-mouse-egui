pub mod ipc;
pub mod tray;

pub use ipc::BackendConnection;
pub use tray::{SystemTray, TrayEvent};

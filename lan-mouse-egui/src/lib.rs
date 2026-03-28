mod application;
mod domain;
mod infrastructure;
mod presentation;

use eframe::{NativeOptions, egui::ViewportBuilder};
use thiserror::Error;

pub use application::LanMouseDesktopApp;

#[derive(Debug, Error)]
pub enum EguiError {
    #[error(transparent)]
    Connection(#[from] lan_mouse_ipc::ConnectionError),
    #[error(transparent)]
    Native(#[from] eframe::Error),
}

pub fn run() -> Result<(), EguiError> {
    log::debug!("running egui frontend");

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Lan Mouse")
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([720.0, 480.0])
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Lan Mouse",
        options,
        Box::new(|cc| Ok(Box::new(LanMouseDesktopApp::new(cc)?))),
    )?;

    Ok(())
}

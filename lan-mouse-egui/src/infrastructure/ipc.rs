use std::{
    io,
    sync::mpsc::{self, Receiver},
    thread,
};

use eframe::egui::Context;
use lan_mouse_ipc::{
    ConnectionError, FrontendEvent, FrontendRequest, FrontendRequestWriter, connect,
};

pub struct BackendConnection {
    requester: FrontendRequestWriter,
    events: Receiver<FrontendEvent>,
}

impl BackendConnection {
    pub fn connect(ctx: &Context) -> Result<Self, ConnectionError> {
        let (mut event_reader, requester) = connect()?;
        let (event_tx, event_rx) = mpsc::channel();
        let repaint_ctx = ctx.clone();

        thread::Builder::new()
            .name("lan-mouse-egui-events".into())
            .spawn(move || {
                while let Some(event) = event_reader.next_event() {
                    match event {
                        Ok(event) => {
                            if event_tx.send(event).is_err() {
                                break;
                            }
                            repaint_ctx.request_repaint();
                        }
                        Err(err) => {
                            log::error!("frontend event stream failed: {err}");
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn egui event thread");

        Ok(Self {
            requester,
            events: event_rx,
        })
    }

    pub fn drain_events(&self) -> Vec<FrontendEvent> {
        let mut drained = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            drained.push(event);
        }
        drained
    }

    pub fn request(&mut self, request: FrontendRequest) -> Result<(), io::Error> {
        self.requester.request(request)
    }
}

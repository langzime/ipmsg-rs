use slint::Weak;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::AppWindow;
use crate::models::event::{ModelEvent, UiEvent};
use slint::ComponentHandle;

pub struct UiWorker {
    pub channel: UnboundedSender<UiEvent>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl UiWorker {
    pub fn new(app_window: &AppWindow) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = app_window.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(ui_worker_loop(r, handle_weak))
                    .unwrap()
            }
        });
        Self {
            channel,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(UiEvent::Quit);
        self.worker_thread.join()
    }
}

async fn ui_worker_loop(
    mut r: UnboundedReceiver<UiEvent>,
    handle: slint::Weak<AppWindow>,
) -> tokio::io::Result<()> {
    loop {
        if let Some(msg) = r.recv().await{

        }
    }
    Ok(())
}


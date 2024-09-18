use slint::{Model, Weak};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::{IpmsgUI, ListViewPageAdapter};
use crate::models::event::{ModelEvent, UiEvent};
use slint::ComponentHandle;

pub struct UiWorker {
    pub channel: UnboundedSender<UiEvent>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl UiWorker {
    pub fn new(app_window: &IpmsgUI) -> Self {
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
    handle: Weak<IpmsgUI>,
) -> tokio::io::Result<()> {
    loop {
        if let Some(msg) = r.recv().await{
            println!("{msg:?}");
            match msg {
                UiEvent::Quit => {
                    break;
                },
                UiEvent::UpdateUserListFooterStatus(_) => {

                }
                UiEvent::OpenOrReOpenChatWindow { .. } => {

                }
                UiEvent::UserListRemoveOne(_) => {

                }
                UiEvent::UserListAddOne(user) => {
                    handle
                        .clone()
                        .upgrade_in_event_loop(move |ipmsg_ui| {
                            let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
                            for i in 0..users.row_count() {
                                if let Some(mut c) = users.row_data(i) {

                                }else{

                                }
                            }
                            //h.global::<DependencyData>().set_model(ModelRc::new(model))
                        });
                }
                UiEvent::CloseChatWindow(_) => {

                }
                UiEvent::OpenOrReOpenChatWindow1 { .. } => {

                }
                UiEvent::DisplaySelfSendMsgInHis { .. } => {

                }
                UiEvent::DisplayReceivedMsgInHis { .. } => {

                }
                UiEvent::RemoveInReceivedList { .. } => {

                }
            }
        }
    }
    Ok(())
}


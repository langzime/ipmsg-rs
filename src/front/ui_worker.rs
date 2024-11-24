use crate::models::event::{ModelEvent, UiEvent};
use crate::models::model::Packet;
use crate::{IpmsgUI, ListViewPageAdapter, Msg, User};
use anyhow::{anyhow, Result};
use log::debug;
use slint::ComponentHandle;
use slint::{Model, VecModel, Weak};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::interval;

pub struct UiWorker {
    pub channel: UnboundedSender<UiEvent>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl UiWorker {
    pub fn new(app_window: &IpmsgUI) -> Self {
        let (channel, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = app_window.as_weak();
            move || tokio::runtime::Runtime::new().unwrap().block_on(ui_worker_loop(r, handle_weak)).unwrap()
        });
        Self { channel, worker_thread }
    }

    pub fn join(self) -> Result<()> {
        let _ = self.channel.send(UiEvent::Quit);
        self.worker_thread.join().unwrap();
        Ok(())
    }
}

async fn ui_worker_loop(mut r: UnboundedReceiver<UiEvent>, handle: Weak<IpmsgUI>) -> tokio::io::Result<()> {
    loop {
        let mut interval = interval(Duration::from_secs(60 * 5));
        interval.tick().await;
        tokio::select! {
            res = r.recv() => {
                if let Some(msg) = res {
                    match msg {
                        UiEvent::Quit => {
                            debug!("ui_worker_loop quit!");
                            break;
                        }
                        UiEvent::UserListRemoveOne(user_ip) => {
                            let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
                                let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
                                let the_model = users.as_any().downcast_ref::<VecModel<User>>().expect("downcast_ref VecModel<User> fail!");
                                for i in 0..the_model.row_count() {
                                    if let Some(mut c) = the_model.row_data(i) {
                                        if c.userId == user_ip {
                                            the_model.remove(i);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        UiEvent::UserListAddOne(user) => {
                            let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
                                let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
                                let the_model = users.as_any().downcast_ref::<VecModel<User>>().expect("downcast_ref VecModel<User> fail!");
                                let u_opt = the_model.iter().find(|u| u.userId == user.ip);
                                let timestamp_now = OffsetDateTime::now_utc().unix_timestamp();
                                if u_opt.is_none() {
                                    println!("新上线用户：{}", user.ip);
                                    let user = User {
                                        name: user.name.into(),
                                        on_line: true,
                                        unread_count: 0,
                                        userId: user.ip.into(),
                                        active: false,
                                        last_beat_time: format!("{}", timestamp_now).into(),
                                    };
                                    the_model.push(user);
                                }
                            });
                        }
                        // UiEvent::CloseChatWindow(_) => {}
                        UiEvent::OpenOrReOpenChatWindow1{ name, ip, packet} => {
                            println!("OpenOrReOpenChatWindow1: {:?} {:?} {:?}", name, ip, packet);
                            /*if let Some(Packet{additional_section: Some(additional_section), ..}) = packet {
                                let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
                                    let msgs = ipmsg_ui.global::<ListViewPageAdapter>().get_msgs();
                                    let the_model = msgs.as_any().downcast_ref::<VecModel<Msg>>().expect("downcast_ref VecModel<User> fail!");
                                    let additions = additional_section.split("\0").collect::<Vec<_>>();
                                    let msg = Msg {
                                        image_url: Default::default(),
                                        name: Default::default(),
                                        text: additions[0].into(),
                                        userId: Default::default()
                                    };
                                    the_model.push(msg);
                                });
                            }*/
                        }
                        UiEvent::DisplaySelfSendMsgInHis { .. } => {}
                        UiEvent::DisplayReceivedMsgInHis { .. } => {}
                        UiEvent::RemoveInReceivedList { .. } => {}
                        UiEvent::AppendingMessages(messages) => {
                            let msg = messages.get(0).unwrap();
                            let msg_user_id = msg.sender_id.clone();
                            println!("AppendingMessages->{}, {:?}", msg_user_id, messages.clone());
                            let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
                                let user_id = ipmsg_ui.global::<ListViewPageAdapter>().get_user_id();
                                let selected_user = user_id.as_str();
                                println!("AppendingMessages-> sender:{} selected_user:{}", msg_user_id, selected_user);
                                    let msgs = ipmsg_ui.global::<ListViewPageAdapter>().get_msgs();
                                    let the_model = msgs.as_any().downcast_ref::<VecModel<Msg>>().expect("downcast_ref VecModel<User> fail!");
                                    for m in messages {
                                        let msg = Msg {
                                            image_url: Default::default(),
                                            name: m.sender_name.into(),
                                            text: m.content.into(),
                                            userId: m.sender_id.into(),
                                            is_self: m.is_self
                                        };
                                        the_model.push(msg);
                                    }
                            });
                        }
                        _ => {
                            println!("unknown event: {:?}", msg);
                        }
                    }
                }
            }
            _ = interval.tick() => {
                println!("tick");
            }
        }
    }
    Ok(())
}

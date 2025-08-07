// #![windows_subsystem = "windows"]

mod constants;
mod core;
mod front;
mod models;
mod store;
mod utils;

const APP_ID: &'static str = "com.github.ipmsg-rs";
slint::include_modules!();
use crate::constants::protocol::msg_type::MSG_TYPE_TEXT;
use crate::constants::protocol::{HOST_NAME, IPMSG_VERSION, LOCAL_IP};
use crate::core::net_worker::UdpWorker;
use crate::front::ui_worker::UiWorker;
use crate::models::event::{UdpEvent, UiEvent};
use crate::models::message::create_sendmsg;
use crate::store::logic::{db_init, insert_message, list_latest_messages};
use crate::store::models::NewMessage;
use crate::utils::logs;
use crate::utils::util::{get_config_dir, utf8_to_gb18030};
use anyhow::Result;
use slint::{Model, VecModel};
use tracing::debug;

fn main() -> Result<()> {
    let _g = logs::init(get_config_dir().to_str().unwrap());
    db_init()?;
    let ui = IpmsgUI::new()?;
    let handle = ui.as_weak();
    ui.global::<UserListAdapter>()
        .on_change_selected_user(move |selected_user_id, selected_user_name| {
            let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
                let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
                let msgs = ipmsg_ui.global::<ListViewPageAdapter>().get_msgs();
                ipmsg_ui.global::<ListViewPageAdapter>().set_user_id(selected_user_id.clone());
                ipmsg_ui.global::<ListViewPageAdapter>().set_user_name(selected_user_name.clone());
                let the_model = users.as_any().downcast_ref::<VecModel<User>>().expect("downcast_ref VecModel<User> fail!");
                for i in 0..the_model.row_count() {
                    if let Some(mut u) = the_model.row_data(i) {
                        if u.userId == selected_user_id {
                            u.active = true;
                        } else {
                            u.active = false;
                        }
                        the_model.set_row_data(i, u);
                    }
                }
                let the_model = msgs.as_any().downcast_ref::<VecModel<Msg>>().expect("downcast_ref VecModel<Msg> fail!");
                the_model.clear();
                //显示历史消息
                if !selected_user_id.is_empty() {
                    let db_messages = list_latest_messages(selected_user_id.to_string(), 20).expect("查询数据库失败！");
                    let ui_msgs = db_messages
                        .iter()
                        .map(|x| Msg {
                            image_url: Default::default(),
                            name: x.sender_name.to_string().into(),
                            text: x.content.to_string().into(),
                            userId: x.sender_name.to_string().into(),
                            is_self: x.is_self,
                        })
                        .collect::<Vec<_>>();
                    the_model.set_vec(ui_msgs);
                }
            });
        });

    let ui_worker = UiWorker::new(&ui);
    let udp_worker = UdpWorker::new(ui_worker.channel.clone());
    let udp_worker_sender = udp_worker.channel.clone();
    let ui_worker_sender = ui_worker.channel.clone();
    ui.global::<Logic>().on_send_msg(move |ip, name, msg_type, text| {
        let mut messages = Vec::new();
        let (packet, _) = create_sendmsg(text.to_string().clone(), None, ip.to_string());
        let text_message = NewMessage {
            ver: IPMSG_VERSION.to_string(),
            message_id: packet.packet_no.clone(),
            msg_type: MSG_TYPE_TEXT as i32,
            sender_id: LOCAL_IP.clone(),
            sender_name: HOST_NAME.clone(),
            receiver_id: ip.to_string(),
            receiver_name: name.to_string(), //对方的昵称
            group_id: "".to_string(),
            is_self: true,
            content: text.to_string().clone(),
            is_read: false,
        };
        debug!("on_send_msg-{text_message:?}");
        messages.push(text_message.clone());
        insert_message(text_message).expect("insert insert_message fail!");
        udp_worker_sender
            .send(UdpEvent::Bytes((utf8_to_gb18030(packet.clone().to_string().as_ref()), ip.to_string())))
            .expect("send failed!");
        ui_worker_sender.send(UiEvent::AppendingMessages(messages)).expect("send message fail!");
    });
    udp_worker.send_ipmsg_br_entry()?;
    ui.run()?;
    ui_worker.join()?;
    udp_worker.join()?;
    Ok(())
}

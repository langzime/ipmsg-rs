// #![windows_subsystem = "windows"]

mod constants;
mod core;
mod events;
mod front;
mod models;
mod store;
mod util;

const APP_ID: &'static str = "com.github.ipmsg-rs";
slint::include_modules!();
use crate::core::net_worker::UdpWorker;
use crate::core::GLOBLE_SENDER;
use crate::front::ui_worker::UiWorker;
use crate::models::event::ModelEvent::SendTextMsg;
use crate::store::logic::{db_init, list_latest_messages};
use anyhow::Result;
use diesel::prelude::*;
use slint::{Model, VecModel};

fn main() -> Result<()> {
    let config_str = include_str!("../config/log4rs.yaml");
    let config = serde_yaml::from_str(config_str)?;
    log4rs::init_raw_config(config)?;
    db_init()?;
    let ui = IpmsgUI::new()?;
    let handle = ui.as_weak();
    ui.global::<UserListAdapter>().on_change_selected_user(move |selected_user_id| {
        let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
            let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
            let msgs = ipmsg_ui.global::<ListViewPageAdapter>().get_msgs();
            ipmsg_ui.global::<ListViewPageAdapter>().set_user_id(selected_user_id.clone());
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
                    })
                    .collect::<Vec<_>>();
                the_model.set_vec(ui_msgs);
            }
        });
    });

    let ui_worker = UiWorker::new(&ui);
    ui.global::<Logic>().on_send_msg(|ip, msg_type, text| {
        GLOBLE_SENDER
            .send(SendTextMsg {
                to_ip: ip.to_string(),
                context: text.to_string(),
            })
            .unwrap();
    });
    let udp_worker = UdpWorker::new(ui_worker.channel.clone());
    ui.run()?;
    ui_worker.join();
    udp_worker.join();
    Ok(())
}

// #![windows_subsystem = "windows"]

mod constants;
mod core;
mod events;
mod front;
mod models;
mod store;
mod util;
use std::net::UdpSocket;
// use human_panic::setup_panic;

const APP_ID: &'static str = "com.github.ipmsg-rs";
slint::include_modules!();
use crate::events::model::model_run;
use crate::front::ui_worker::UiWorker;
use crate::store::establish_connection;
use crate::store::logic::init;
use crate::store::models::Messages;
use anyhow::Result;
use diesel::prelude::*;
use log::{debug, info, warn};
use slint::format;
use slint::{Color, Model, ModelRc, StandardListViewItem, VecModel, Weak};
use std::rc::Rc;

fn main() -> Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default())?;
    init()?;
    let ui = IpmsgUI::new()?;
    let handle = ui.as_weak();
    ui.global::<UserListAdapter>().on_change_selected_user(move |selected_user_id| {
        let _ = handle.clone().upgrade_in_event_loop(move |ipmsg_ui| {
            let users = ipmsg_ui.global::<ListViewPageAdapter>().get_users();
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
        });
    });

    let ui_worker = UiWorker::new(&ui);

    let socket: UdpSocket = match UdpSocket::bind(constants::protocol::ADDR.as_str()) {
        Ok(s) => {
            info!("udp server start listening! {:?}", constants::protocol::ADDR.as_str());
            s
        }
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    model_run(socket, ui_worker.channel.clone());
    ui.run()?;
    Ok(())
}

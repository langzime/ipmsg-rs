// #![windows_subsystem = "windows"]

mod front;
mod models;

mod constants;

mod events;

mod core;

mod util;

use std::net::UdpSocket;
// use human_panic::setup_panic;

const APP_ID: &'static str = "com.github.ipmsg-rs";
slint::include_modules!();
use slint::{Color, Model, ModelRc, StandardListViewItem, VecModel, Weak};
use std::rc::Rc;
use log::{debug, info, warn};
use slint::format;
use crate::events::model::model_run;
use crate::front::ui_worker::UiWorker;
use anyhow::Result;

fn main() -> Result<()> {

    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let todo_model = Rc::new(slint::VecModel::<User>::from(vec![
        User { name: "王艳青".into(), userId: "xxxxx".into(), active: true },
        User { name: "王小满".into(), userId: "xxxxx".into(), active: false },
    ]));

    let ui = AppWindow::new()?;
    ui.global::<ListViewPageAdapter>().set_users(todo_model.into());

    let ui_worker = UiWorker::new(&ui);

    let socket: UdpSocket = match UdpSocket::bind(constants::protocol::ADDR.as_str()) {
        Ok(s) => {
            info!("udp server start listening! {:?}", constants::protocol::ADDR.as_str());
            s
        }
        Err(e) => panic!("couldn't bind socket: {}", e)
    };
    model_run(socket, ui_worker.channel.clone());

    let ui_handle: Weak<AppWindow> = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });
    ui.run()?;
    Ok(())
}
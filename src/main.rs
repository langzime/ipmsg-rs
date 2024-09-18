// #![windows_subsystem = "windows"]

mod constants;
mod core;
mod events;
mod front;
mod models;
mod util;
use std::net::UdpSocket;
// use human_panic::setup_panic;

const APP_ID: &'static str = "com.github.ipmsg-rs";
slint::include_modules!();
use crate::events::model::model_run;
use crate::front::ui_worker::UiWorker;
use anyhow::Result;
use log::{debug, info, warn};
use slint::format;
use slint::{Color, Model, ModelRc, StandardListViewItem, VecModel, Weak};
use std::rc::Rc;

fn main() -> Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default())?;
    let todo_model = Rc::new(slint::VecModel::<User>::from(vec![
        User {
            name: "浪子".into(),
            userId: "langzi".into(),
            active: true,
        },
        User {
            name: "爱心💌".into(),
            userId: "heart".into(),
            active: false,
        },
    ]));

    let todo_model1 = Rc::new(slint::VecModel::<Msg>::from(vec![
        Msg {
            image_url: Default::default(),
            name: "浪子1".into(),
            text: Default::default(),
            userId: "xxxxx".into(),
        },
        Msg {
            image_url: Default::default(),
            name: "浪子2".into(),
            text: Default::default(),
            userId: "xxxxx".into(),
        },
        Msg {
            image_url: Default::default(),
            name: "浪子3".into(),
            text: Default::default(),
            userId: "xxxxx".into(),
        },
        Msg {
            image_url: Default::default(),
            name: "浪子4".into(),
            text: Default::default(),
            userId: "xxxxx".into(),
        },
        Msg {
            image_url: Default::default(),
            name: "浪子5".into(),
            text: Default::default(),
            userId: "xxxxx".into(),
        },
    ]));

    let ui = IpmsgUI::new()?;
    // ui.global::<ListViewPageAdapter>().set_users(todo_model.into());
    ui.global::<ListViewPageAdapter>().set_msgs(todo_model1.into());

    ui.global::<UserListAdapter>().on_change_selected_user(move |index| {
        println!("selected user index: {}", index);
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

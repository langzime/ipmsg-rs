#![windows_subsystem = "windows"]

use human_panic::setup_panic;
use gio::prelude::*;
use log::info;
use crate::ui::main_win::MainWindow;

mod models;
mod util;
mod events;
mod ui;
mod core;
mod constants;

const APP_ID: &'static str = "com.github.ipmsg-rs";

fn main() -> glib::ExitCode {
    setup_panic!();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let application = adw::Application::builder().application_id(APP_ID).build();
    application.connect_startup(move |app| {
        info!("starting up");
        MainWindow::new(app);
    });
    application.connect_activate(|_| {
        info!("connect_activate");
    });

    application.connect_shutdown(move |_| {
        info!("shutdown!");
    });

    application.run()
}
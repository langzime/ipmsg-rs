
use human_panic::setup_panic;
use gio::ApplicationFlags;
use gio::prelude::*;
use log::info;
use adw::prelude::*;
use crate::ui::main_win::MainWindow;

mod models;
mod util;
mod events;
mod ui;
mod core;
mod constants;

fn main() -> glib::ExitCode {
    setup_panic!();
    ::std::env::set_var("RUST_LOG", "info");
    drop(env_logger::init());
    /*let application = adw::Application::new(
        Some("com.github.ipmsg-rs"),
        ApplicationFlags::FLAGS_NONE);*/
    let application = adw::Application::builder().application_id("com.github.ipmsg-rs").build();
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
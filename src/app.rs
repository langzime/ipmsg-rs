use chrono::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::env::args;
use human_panic::setup_panic;
use gio::ApplicationFlags;
use gio::prelude::*;
use log::info;
use crate::main_win::MainWindow;

pub fn run(){
    setup_panic!();
    ::std::env::set_var("RUST_LOG", "info");
    drop(env_logger::init());
    let application = gtk::Application::new(
        Some("com.github.raudient"),
                    ApplicationFlags::FLAGS_NONE);
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

    application.run();
}
use ::gtk::prelude::*;
use ::gtk::{
    self, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};
use std::net::UdpSocket;
use std::thread;
use std::cell::{RefCell, Ref};

pub fn spwan() {
    let addr: String = format!("{}{}", "0.0.0.0:", 9000);

    let socket: UdpSocket = match UdpSocket::bind(addr.as_str()) {
        Ok(s) => {
            println!("{:?} 开启端口监听", s);
            s
        },
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    ::demons::GLOBAL_UDPSOCKET.with(move |global| {
        *global.borrow_mut() = Some(socket.try_clone().unwrap());
    });

    ::demons::GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let soc = socket.try_clone().unwrap();
            thread::spawn(move||{
                soc.send_to("aaa".to_string().as_bytes(),"192.168.0.133").expect("couldn't send message");
            });

        }
    });
}
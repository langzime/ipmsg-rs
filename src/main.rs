
extern crate gtk;
extern crate glib;
extern crate chrono;
extern crate hostname;
extern crate local_ip;
extern crate encoding;
#[macro_use]
extern crate lazy_static;


mod constant;
mod model;
mod demons;
mod message;
//mod app;


use gtk::prelude::*;
use gtk::{
    CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};

use chrono::prelude::*;

use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::sync::mpsc;
use std::net::UdpSocket;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use model::Packet;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(gtk::WindowType::Toplevel);
    window.set_title("飞鸽传书");
    //window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(200, 500);
    window.set_resizable(false);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    //纵向
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let menu_bar = MenuBar::new();
    let sytem_item = MenuItem::new_with_label("系统");
    menu_bar.append(&sytem_item);
    let window_item = MenuItem::new_with_label("窗口");
    menu_bar.append(&window_item);
    let config_item = MenuItem::new_with_label("配置");
    menu_bar.append(&config_item);

    let label = Label::new("");

    let scrolled = ScrolledWindow::new(None, None);
    let tree = create_and_setup_view();
    let model = create_and_fill_model();
    tree.set_model(Some(&model));
    scrolled.add(&tree);
    scrolled.set_min_content_height(450);
    v_box.add(&menu_bar);
    v_box.add(&scrolled);
    v_box.add(&label);

    tree.connect_cursor_changed(move |tree_view| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            &label.set_text(&format!("-- {} --", model.get_value(&iter, 0).get::<String>().unwrap()));
        }
    });

    tree.connect_row_activated(move |tree_view, tree_path, tree_view_column| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            let ip_str = model.get_value(&iter, 3).get::<String>().unwrap();
            let name = model.get_value(&iter, 0).get::<String>().unwrap();
            let chat_title = &format!("和{}({})聊天窗口", name, ip_str);
            let chat_window = Window::new(gtk::WindowType::Toplevel);
            chat_window.set_title(chat_title);
            chat_window.set_position(gtk::WindowPosition::Center);
            chat_window.set_default_size(450, 500);
            let v_chat_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let text_view = gtk::TextView::new();
            let scroll = gtk::ScrolledWindow::new(None, None);
            scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            scroll.add(&text_view);
            v_chat_box.add(&scroll);
            chat_window.add(&v_chat_box);
            chat_window.show_all();
        }
    });

    let (tx1, rx1) = mpsc::channel();
    let new_user_sender = tx1.clone();
    // put ListStore and receiver in thread local storage
    demons::GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((model, rx1))
    });

    let addr: String = format!("{}{}", "0.0.0.0:", constant::IPMSG_DEFAULT_PORT);

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

    ///待处理消息队列
    let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel();

    let tx_demon = tx.clone();
    //接收消息守护线程
    demons::start_daemon(tx_demon);
    //消息处理守护线程
    demons::start_message_processer(rx, new_user_sender);
    //启动发送上线消息
    message::send_ipmsg_br_entry();

    window.add(&v_box);
    window.show_all();
    //app::spwan();
    gtk::main();
}

fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    // Creating the two columns inside the view.
    append_column(&tree, 0, "用户名");
    //append_column(&tree, 1, "工作组");
    //append_column(&tree, 2, "主机名");
    tree.set_headers_visible(false);
    tree
}

fn append_column(tree: &TreeView, id: i32, title: &str) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.set_title(title);
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
    tree.set_headers_visible(true);
}

fn create_and_fill_model() -> ListStore {
    // Creation of a model with two rows.
    let model = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type()]);

    // Filling up the tree view.
    /*let entries = &[("啦啦啦", "11"), ("啦啦啦", "22"), ("啦啦啦", "33"), ("啦啦啦", "44"), ("啦啦啦", "55"), ("啦啦啦", "66"), ("啦啦啦", "77"), ("啦啦啦", "88")];
    for (i, entry) in entries.iter().enumerate() {
        model.insert_with_values(None, &[0, 1, 2], &[&(i as u32 + 1), &entry.0, &entry.1]);
    }*/
    model
}
use gtk::prelude::*;
use gtk::{
    self, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};

use chrono::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use model::Packet;

pub fn run(){
    ::env_logger::init().unwrap();
    info!("starting up");
    if gtk::init().is_err() {
        info!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(gtk::WindowType::Toplevel);
    window.set_title("飞鸽传书");
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
    let menu_sys = Menu::new();
    let about = MenuItem::new_with_label("关于");
    let quit = MenuItem::new_with_label("退出");
    menu_sys.append(&about);
    menu_sys.append(&quit);
    sytem_item.set_submenu(Some(&menu_sys));
    menu_bar.append(&sytem_item);

    let window_item = MenuItem::new_with_label("窗口");
    let menu_window = Menu::new();
    let send_file = MenuItem::new_with_label("发送文件");
    let manage_file = MenuItem::new_with_label("管理文件");
    menu_window.append(&send_file);
    menu_window.append(&manage_file);
    window_item.set_submenu(Some(&menu_window));
    menu_bar.append(&window_item);

    let window_about = window.clone();
    about.connect_activate(move |_| {
        let p = AboutDialog::new();
        p.set_website_label(Some("ipmsg"));
        p.set_website(Some("https://www.langzi.me"));
        p.set_authors(&["langzi"]);
        p.set_title("关于");
        p.set_transient_for(Some(&window_about));
        p.run();
        p.destroy();
    });

    let window_quit = window.clone();
    quit.connect_activate(move |_| {
        gtk::main_quit();
    });

    let label = Label::new("");
    let scrolled = ScrolledWindow::new(None, None);
    scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
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

    let (remained_sender, remained_receiver): (mpsc::Sender<((String, String), Option<Packet>)>, mpsc::Receiver<((String, String), Option<Packet>)>) = mpsc::channel();

    let remained_sender1 = remained_sender.clone();
    tree.connect_row_activated(move |tree_view, tree_path, tree_view_column| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            let ip_str = model.get_value(&iter, 3).get::<String>().unwrap();
            let name = model.get_value(&iter, 0).get::<String>().unwrap();
            remained_sender1.send(((name, ip_str), None));
            ::glib::idle_add(::demons::create_or_open_chat);

        }
    });

    let (user_add_sender, user_list_receiver) = mpsc::channel();
    let new_user_sender_clone = user_add_sender.clone();
    // put ListStore and receiver in thread local storage
    ::demons::GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((model, user_list_receiver))
    });

    //let addr: String = format!("{}{}", "0.0.0.0:", constant::IPMSG_DEFAULT_PORT);

    let socket: UdpSocket = match UdpSocket::bind(::constant::addr.as_str()) {
        Ok(s) => {
            info!("udp server start listening! {:?}", ::constant::addr.as_str());
            s
        },
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    ::demons::GLOBAL_UDPSOCKET.with(move |global| {
        *global.borrow_mut() = Some(socket.try_clone().unwrap());
    });

    ///待处理消息队列
    let (packet_sender, packet_receiver): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel();

    ::demons::GLOBAL_WINDOWS.with(move |global| {
        *global.borrow_mut() = Some((HashMap::new(), remained_receiver));
    });

    let packet_sender_clone = packet_sender.clone();
    //接收消息守护线程
    ::demons::start_daemon(packet_sender_clone);
    ::demons::start_file_processer();
    //消息处理守护线程
    ::demons::start_message_processer(packet_receiver, new_user_sender_clone, remained_sender.clone());
    //启动发送上线消息
    ::message::send_ipmsg_br_entry();

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
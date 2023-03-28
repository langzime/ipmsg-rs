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
use gio::{ApplicationFlags, Menu, MenuItem};
use gtk::prelude::*;
use gtk::{
    self, CellRendererText, CellRendererProgress, AboutDialog, IconSize, Image, Label, Window,
    ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    Widget, TextView, Fixed, ScrolledWindow, ListBox, ListBoxRow, Application
};
use gdk_pixbuf::Pixbuf;
use glib::{Receiver, MainContext};
use crossbeam_channel::unbounded;
use log::{info, trace, warn, debug};
use glib::clone;
use crate::model::{self, User, OperUser, Operate, ShareInfo, Packet, FileInfo, ReceivedSimpleFileInfo, ReceivedPacketInner, ErrMsg};
use crate::chat_window::ChatWindow;
use crate::events::{ui::UiEvent, model::ModelEvent, model::model_run};

pub struct MainWindow {

}

impl MainWindow {

    pub fn new(application: &Application) -> MainWindow {
        let (tx, rx): (glib::Sender<UiEvent>, glib::Receiver<UiEvent>) = MainContext::channel::<UiEvent>(glib::PRIORITY_HIGH);
        let (model_sender, model_receiver): (crossbeam_channel::Sender<ModelEvent>, crossbeam_channel::Receiver<ModelEvent>) = unbounded();

        let window = gtk::ApplicationWindow::new(application);
        window.set_title(Some("飞鸽传书"));
        // window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(200, 500);
        window.set_resizable(false);
        /*window.connect_delete_event(clone!(@weak window => @default-return Inhibit(false), move |_, _| {
            unsafe {
                &window.destroy();
            }
            return Inhibit(false);
        }));*/
        window.connect_close_request(clone!(@strong tx => @default-return glib::signal::Inhibit(false), move |win| {
            if let Some(application) = win.application() {
                application.remove_window(win);
            }
            glib::signal::Inhibit(false)
        }));

        //纵向
        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        /*let menu_bar = Menu::new();
        let sytem_item = MenuItem::with_label("系统");
        let menu_sys = Menu::new();
        let about = MenuItem::with_label("关于");
        let quit = MenuItem::with_label("退出");
        MenuItem::new(Some("xxx"), Some("yyyyy"));
        menu_sys.append(&about);
        menu_sys.append(&quit);
        sytem_item.set_submenu(Some(&menu_sys));
        menu_bar.append(&sytem_item);

        about.connect_activate(clone!(@weak  window => move |_| {
            let p = AboutDialog::new();
            p.set_website_label(Some("ipmsg"));
            p.set_website(Some("https://www.langzi.me"));
            p.set_authors(&["langzi"]);
            p.set_logo(Some(&Pixbuf::from_file("./resources/eye.png").unwrap()));
            p.set_title("关于");
            p.set_transient_for(Some(&window));
            p.run();
            unsafe {
             p.destroy();
            }
        }));

        quit.connect_activate(clone!(@weak window => move |_| {
            unsafe {
                &window.destroy();
            }
        }));*/

        let label = Label::new(Option::from(""));
        let scrolled = ScrolledWindow::new();//None::<&gtk::Adjustment>, None::<&gtk::Adjustment>
        scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        let tree = create_and_setup_view();
        let model = create_and_fill_model();
        tree.set_model(Some(&model));
        scrolled.set_child(Some(&tree));
        scrolled.set_min_content_height(450);
        // v_box.add(&menu_bar);
        v_box.append(&scrolled);
        v_box.append(&label);
        model_sender.clone().send(ModelEvent::UserListSelected(String::from("未选择"))).unwrap();

        tree.connect_cursor_changed(clone!(@strong model_sender => move |tree_view| {
        let selection = tree_view.selection();
        if let Some((model, iter)) = selection.selected() {
            let str1 = model.get_value(&iter, 0).get::<String>().unwrap();
            model_sender.send(ModelEvent::UserListSelected(str1)).unwrap();
        }
    }));

        let mut chat_windows: HashMap<String, ChatWindow> = HashMap::new();

        tree.connect_row_activated(clone!(@strong model_sender => move |tree_view, tree_path, tree_view_column| {
        let selection = tree_view.selection();
        if let Some((model, iter)) = selection.selected() {
            let ip_str = model.get_value(&iter, 3).get::<String>().unwrap();
            let name = model.get_value(&iter, 0).get::<String>().unwrap();
            model_sender.send(ModelEvent::UserListDoubleClicked{name, ip: ip_str }).unwrap();
        }
    }));

        let socket: UdpSocket = match UdpSocket::bind(crate::constant::ADDR.as_str()) {
            Ok(s) => {
                info!("udp server start listening! {:?}", crate::constant::ADDR.as_str());
                s
            },
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        model_run(socket.try_clone().unwrap(), model_receiver, model_sender.clone(),tx);

        let main_context = MainContext::default();
        main_context.acquire();
        rx.attach(Some(&main_context), move |event| {
            match event {
                UiEvent::OpenOrReOpenChatWindow {name, ip} => {
                    match &chat_windows.get(&ip) {
                        Some(win) => {
                        }
                        None => {
                            let chat_win = crate::chat_window::create_chat_window(model_sender.clone(), name, ip.clone());
                            chat_windows.insert(ip.clone(), chat_win);
                        }
                    }
                }
                UiEvent::UpdateUserListFooterStatus(text) => {
                    label.set_text(&format!("-- {} --", text));
                }
                UiEvent::UserListRemoveOne(ip) => {
                    if let Some(first) = model.iter_first(){//拿出来第一条
                        let mut num :u32 = model.string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                        let ip1 = model.get_value(&first, 3).get::<String>().unwrap();//获取ip
                        if ip == ip1 {
                            model.remove(&first);
                        }else {
                            loop {
                                num = num + 1;
                                if let Some(next_iter) = model.iter_from_string(&num.to_string()){
                                    let next_ip = model.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
                                    if next_ip == ip1 {
                                        model.remove(&next_iter);
                                        break;
                                    }
                                }else{
                                    break;
                                }
                            }
                        }
                    }
                }
                UiEvent::UserListAddOne(income_user) => {
                    let mut in_flag = false;
                    if let Some(first) = model.iter_first(){//拿出来第一条
                        let mut num :u32 = model.string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                        let ip = model.get_value(&first, 3).get::<String>().unwrap();//获取ip
                        if ip == income_user.ip {
                            in_flag = true;
                        }else {
                            loop {
                                num = num + 1;
                                if let Some(next_iter) = model.iter_from_string(&num.to_string()){
                                    let next_ip = model.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
                                    if next_ip == income_user.ip {
                                        in_flag = true;
                                        break;
                                    }
                                }else{
                                    break;
                                }
                            }
                        }
                    }
                    if !in_flag {
                        //model.insert_with_values(None, &[0, 1, 2, 3], &[&&income_user.name, &&income_user.group, &&income_user.host, &&income_user.ip]);
                        model.insert_with_values(None, &[(0, &&income_user.name), (1, &&income_user.group), (2, &&income_user.host), (3, &&income_user.ip)]);
                    }
                }
                UiEvent::CloseChatWindow(ip) => {
                    match &chat_windows.get(&ip) {
                        Some(win) => {
                            chat_windows.remove(&ip);
                        }
                        None => {
                        }
                    }
                }
                UiEvent::OpenOrReOpenChatWindow1 { name, ip, packet} => {
                    match &chat_windows.get(&ip) {
                        Some(win) => {
                            //&window.set_focus(Some(v_box));
                            //win.win.show();
                        }
                        None => {
                            let chat_win = crate::chat_window::create_chat_window(model_sender.clone(), name, ip.clone());
                            chat_windows.insert(ip.clone(), chat_win);
                        }
                    }
                }
                UiEvent::DisplaySelfSendMsgInHis {to_ip, context, files} => {
                    match &chat_windows.get(&to_ip) {
                        Some(win) => {
                            let (his_start_iter, mut his_end_iter) = win.his_view.buffer().bounds();
                            win.his_view.buffer().insert(&mut his_end_iter, format!("{}:{}\n", "我", context).as_str());

                        }
                        None => {}
                    }
                }
                UiEvent::DisplayReceivedMsgInHis{ from_ip, name, context, files } => {
                    match &chat_windows.get(&from_ip) {
                        Some(win) => {
                            let (his_start_iter, mut his_end_iter) = win.his_view.buffer().bounds();
                            win.his_view.buffer().insert(&mut his_end_iter, format!("{}:{}\n", name, context).as_str());

                            for file in &files {
                                //win.received_store.insert_with_values(None, &[0, 1, 2, 3, 4, 5], &[&&file.name, &&file.file_id, &&file.packet_id, &&file.attr, &&file.size, &&file.mtime]);
                                win.received_store.insert_with_values(None, &[(0, &&file.name), (1, &&file.file_id), (2, &&file.packet_id), (3, &&file.attr), (4, &&file.size), (5, &&file.mtime)]);
                            }
                        }
                        None => {}
                    }
                }
                UiEvent::RemoveInReceivedList{packet_id, file_id, download_ip } => {
                    match &chat_windows.get(&download_ip) {
                        Some(win) => {
                            let pre_receive_file_store = &win.received_store;
                            if let Some(first) = pre_receive_file_store.iter_first(){
                                let mut num :u32 = pre_receive_file_store.string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                                let received_file_id = pre_receive_file_store.get_value(&first, 1).get::<u32>().unwrap();
                                let received_packet_id = pre_receive_file_store.get_value(&first, 2).get::<u32>().unwrap();
                                if file_id == received_file_id&&packet_id == received_packet_id {
                                    pre_receive_file_store.remove(&first);
                                }else {
                                    loop {
                                        num = num + 1;
                                        if let Some(next_iter) = pre_receive_file_store.iter_from_string(&num.to_string()){
                                            let next_file_id = pre_receive_file_store.get_value(&next_iter, 1).get::<u32>().unwrap();
                                            let next_packet_id = pre_receive_file_store.get_value(&next_iter, 2).get::<u32>().unwrap();
                                            if next_file_id == file_id&&next_packet_id == packet_id {
                                                pre_receive_file_store.remove(&next_iter);
                                                break;
                                            }
                                        }else{
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                _ => {
                    println!("{}", "aaa");
                }
            };
            glib::Continue(true)
        });

        window.set_child(Some(&v_box));
        window.present();
        MainWindow{}
    }
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
    model
}
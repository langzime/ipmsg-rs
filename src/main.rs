
extern crate gtk;
extern crate gtk_sys;
extern crate chrono;
extern crate hostname;
extern crate local_ip;


mod constant;
mod model;
mod demons;
mod message;


use gtk::prelude::*;
use gtk::{
    CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};
use gtk_sys::*;

use chrono::prelude::*;


use std::thread;
use std::net::UdpSocket;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr, ToSocketAddrs};



fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("飞鸽传书");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 300);
    //window.set_resizable(false);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    /*
    let grid = Grid::new();

    //横向
    let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    //横向
    let h_box_list = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let h_align_button = Alignment::new(0f32, 0f32, 0f32, 0f32);

    //纵向
    let v_box = gtk::Box::new(gtk::Orientation::Vertical, 30);

    v_box.add(&grid);

    let scrolled = ScrolledWindow::new(None, None);

    let button = gtk::Button::new_with_label("刷新");

    let text_view = gtk::TextView::new();
    text_view.get_buffer().unwrap().set_text("啦啦啦啦\n啦啦啦啦\n啦啦啦啦");
    text_view.set_editable(false);

    let tree = create_and_setup_view();
    let model = create_and_fill_model();

    // Setting the model into the view.
    tree.set_model(Some(&model));

//    scrolled.set_min_content_height(250);
//    scrolled.set_min_content_width(250);
    scrolled.add(&tree);

    let button1 = gtk::Button::new_with_label("刷新11");
    let button2 = gtk::Button::new_with_label("刷新22");
    let button3 = gtk::Button::new_with_label("刷新33");
    grid.attach(&scrolled, 0, 0, 4, 5);
    grid.attach(&button1, 5, 3, 1, 1);
    grid.set_cell_width(&scrolled, 300);

    //h_box_list.add(&scrolled);

    //h_align_button.add(&button);
    //h_align_button.set_margin_left(60);
    //h_align_button.set_margin_top(80);
    //h_box.pack_start(&h_box_list, false, false, 0);
    //h_box.pack_start(&h_align_button, false, false, 0);

    v_box.add(&h_box);
    v_box.add(&text_view);
    window.add(&v_box);*/

    //启动守护线程
    //demons::start_demon();

    //启动发送上线消息
    //demons::send_ipmsg_br_entry();
    //window.show_all();
    //gtk::main();

    /*println!("Major: {}, Minor: {}", gtk::get_major_version(), gtk::get_minor_version());
    let glade_src = include_str!("gtktest.glade");
    let builder = Builder::new_from_string(glade_src);
    let window: Window = builder.get_object("window1").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.show_all();
    gtk::main();*/

    let grid = Grid::new();
    let scrolled = ScrolledWindow::new(None, None);
    let tree = create_and_setup_view();
    let model = create_and_fill_model();
    // Setting the model into the view.
    tree.set_model(Some(&model));
    let button1 = gtk::Button::new_with_label("刷新11");
    //grid.set_cell_width(100);
    grid.attach(&tree, 0, 0, 4, 5);
    grid.attach(&button1, 5, 2, 1, 1);
    let text_view = gtk::TextView::new();
    text_view.get_buffer().unwrap().set_text("啦啦啦啦\n啦啦啦啦\n啦啦啦啦");
    //text_view.set_editable(false);
    text_view.set_margin_top(10);
    grid.attach(&text_view, 0, 6, 6, 1);


    let addr: String = format!("{}{}", "0.0.0.0:", constant::IPMSG_DEFAULT_PORT);


    //let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), constant::IPMSG_DEFAULT_PORT as u16));

    let socket: UdpSocket = match UdpSocket::bind(addr.as_str()) {
        Ok(s) => {
            println!("{:?} 开启端口监听", s);
            s
        },
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    let sock_clone0 = socket.try_clone().unwrap();

    //启动守护线程
    demons::start_demon(sock_clone0);

    let sock_clone1 = socket.try_clone().unwrap();

    //启动发送上线消息
    //demons::send_ipmsg_br_entry(sock_clone1);
    window.show_all();

    window.add(&grid);
    window.show_all();
    gtk::main();

}

fn create_and_setup_view() -> TreeView {
    // Creating the tree view.
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    // Creating the two columns inside the view.
    append_column(&tree, 0, "用户名");
    append_column(&tree, 1, "工作组");
    append_column(&tree, 2, "主机名");
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
    let model = ListStore::new(&[u32::static_type(), String::static_type(), String::static_type()]);

    // Filling up the tree view.
    let entries = &[("啦啦啦", "11"), ("啦啦啦", "22"), ("啦啦啦", "33"), ("啦啦啦", "44"), ("啦啦啦", "55"), ("啦啦啦", "66"), ("啦啦啦", "77"), ("啦啦啦", "88")];
    for (i, entry) in entries.iter().enumerate() {
        model.insert_with_values(None, &[0, 1, 2], &[&(i as u32 + 1), &entry.0, &entry.1]);
    }
    model
}
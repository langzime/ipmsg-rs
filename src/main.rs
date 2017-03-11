extern crate gtk;
extern crate chrono;
extern crate hostname;
extern crate local_ip;

use gtk::prelude::*;
use gtk::{IconSize, Orientation, ReliefStyle, Widget};
use std::thread;

mod constant;
mod model;
mod demons;
mod message;



fn main() {

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("First GTK+ Program");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let button = gtk::Button::new_with_label("点击我!");

    window.add(&button);
    //启动守护线程
    demons::start_demon();

    //启动发送上线消息
    demons::send_ipmsg_br_entry();
    window.show_all();
    gtk::main();
}
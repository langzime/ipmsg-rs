
use gtk;
use gtk::prelude::*;
use gtk::{
    CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};
use message;
use demons;

/*
pub fn create_chatui(map :&mut HashMap<String, Window>, name: String, ip_str: String) {
    let chat_title = &format!("和{}({})聊天窗口", name, ip_str);
    let chat_window = Window::new(gtk::WindowType::Toplevel);
    chat_window.set_title(chat_title);
    chat_window.set_border_width(5);
    chat_window.set_position(gtk::WindowPosition::Center);
    chat_window.set_default_size(450, 500);
    let v_chat_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let h_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let button1 = gtk::Button::new_with_label("清空");
    let button2 = gtk::Button::new_with_label("发送");
    h_button_box.add(&button1);
    h_button_box.add(&button2);
    let text_view = gtk::TextView::new();
    let scroll = gtk::ScrolledWindow::new(None, None);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.set_min_content_height(350);
    text_view.set_cursor_visible(false);
    text_view.set_editable(false);
    scroll.add(&text_view);

    let text_view_presend = gtk::TextView::new();
    let scroll1 = gtk::ScrolledWindow::new(None, None);
    scroll1.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll1.set_margin_top(10);
    scroll1.set_min_content_height(80);
    scroll1.add(&text_view_presend);
    v_chat_box.add(&scroll);
    v_chat_box.add(&scroll1);
    v_chat_box.add(&h_button_box);
    chat_window.add(&v_chat_box);
    let ip_str_1 = ip_str.clone();
    let ip_str_2 = ip_str.clone();
    let ip_str_3 = ip_str.clone();
    button2.connect_clicked(move|_|{
        let start_iter = &text_view_presend.get_buffer().unwrap().get_start_iter();
        let end_iter = &text_view_presend.get_buffer().unwrap().get_end_iter();
        let context :&str = &text_view_presend.get_buffer().unwrap().get_text(&start_iter, &end_iter, false).unwrap();
        message::send_ipmsg(context.to_owned(), ip_str_1.clone());
        &text_view.get_buffer().unwrap().set_text(context);
        &text_view_presend.get_buffer().unwrap().set_text("");
    });
    chat_window.show_all();
    chat_window.connect_delete_event(move|_, _| {
        //demons::GLOBAL_WINDOWS.with(|global| {
            //if let Some((ref mut map, _)) = *global.borrow_mut() {
                map.remove(&ip_str_3);
          //  }
        //});
        Inhibit(false)
    });
    let clone_chat = chat_window.clone();
   // demons::GLOBAL_WINDOWS.with(move |global| {
     //   if let Some((ref mut map, _)) = *global.borrow_mut() {
            map.insert(ip_str_2, clone_chat);
       // }
    //});
}*/

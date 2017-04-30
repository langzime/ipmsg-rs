use gtk::prelude::*;
use gtk::{
    self, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::path::{PathBuf, Path};
use std::fs::{self, File, Metadata, ReadDir};
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};
use chrono::prelude::*;
use model::{self, Packet};
use message;
use demons::GLOBAL_WINDOWS;

#[derive(Clone)]
pub struct ChatWindow {
    pub win :Window,
    pub his_view :TextView,
    pub ip :String,
}

pub fn create_chat_window<S: Into<String>>(name :S, host_ip :S, packet: Option<Packet>) -> ChatWindow {
    let name: String = name.into();
    let host_ip: String = host_ip.into();
    let chat_title = &format!("和{}({})聊天窗口", name, host_ip);
    let chat_window = Window::new(::gtk::WindowType::Toplevel);
    chat_window.set_title(chat_title);
    chat_window.set_border_width(5);
    chat_window.set_position(::gtk::WindowPosition::Center);
    chat_window.set_default_size(450, 500);
    let v_chat_box = gtk::Box::new(::gtk::Orientation::Vertical, 0);
    let h_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let button1 = gtk::Button::new_with_label("清空");
    let button2 = gtk::Button::new_with_label("发送");
    let button3 = gtk::Button::new_with_label("选择文件");
    let button4 = gtk::Button::new_with_label("选择文件夹");
    h_button_box.add(&button1);
    h_button_box.add(&button2);
    h_button_box.add(&button3);
    h_button_box.add(&button4);
    let text_view = gtk::TextView::new();
    let scroll = gtk::ScrolledWindow::new(None, None);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.set_min_content_height(350);
    text_view.set_cursor_visible(false);
    text_view.set_editable(false);
    scroll.add(&text_view);
    if let Some(pac) = packet {
        let additional_section =  pac.additional_section.unwrap();
        let v: Vec<&str> = additional_section.split('\0').into_iter().collect();
        &text_view.get_buffer().unwrap().set_text(format!("{}:{}\n", name, v[0]).as_str());
    }
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
    let ip_str_1 = host_ip.clone();
    let ip_str_2 = host_ip.clone();
    let ip_str_3 = host_ip.clone();
    let ip_str_4 = host_ip.clone();
    let clone_hist_view_event = text_view.clone();
    let pre_send_files: Arc<RefCell<Vec<model::FileInfo>>> = Arc::new(RefCell::new(Vec::new()));//待发送文件列表
    let files_send_clone = pre_send_files.clone();
    button2.connect_clicked(move|_|{
        let (start_iter, mut end_iter) = text_view_presend.get_buffer().unwrap().get_bounds();
        let context :&str = &text_view_presend.get_buffer().unwrap().get_text(&start_iter, &end_iter, false).unwrap();
        println!("{}", context);
        message::send_ipmsg(context.to_owned(), files_send_clone.borrow().to_vec(), ip_str_1.clone());
        (*files_send_clone.borrow_mut()).clear();
        let (his_start_iter, mut his_end_iter) = clone_hist_view_event.get_buffer().unwrap().get_bounds();
        &clone_hist_view_event.get_buffer().unwrap().insert(&mut his_end_iter, format!("{}:{}\n", "我", context).as_str());
        &text_view_presend.get_buffer().unwrap().set_text("");
    });
    let chat_window1 = chat_window.clone();
    let files_add_clone = pre_send_files.clone();
    button3.connect_clicked(move|_|{
        let file_chooser = gtk::FileChooserDialog::new(
            Some("打开文件"), Some(&chat_window1), gtk::FileChooserAction::Open);
        file_chooser.add_buttons(&[
            ("选择文件", gtk::ResponseType::Ok.into()),
            ("取消", gtk::ResponseType::Cancel.into()),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok.into() {
            let filename: PathBuf = file_chooser.get_filename().unwrap();
            let metadata: Metadata = fs::metadata(&filename).unwrap();
            let size = metadata.len();
            let attr = if metadata.is_file() {
                ::constant::IPMSG_FILE_REGULAR
            }else if metadata.is_dir() {
                ::constant::IPMSG_FILE_DIR
            }else {
                panic!("oh no!");
            };
            let modify_time: time::SystemTime = metadata.modified().unwrap();
            let chrono_time = ::util::system_time_to_date_time(modify_time);
            let local_time = chrono_time.with_timezone(&::chrono::Local);
            let file_info = model::FileInfo {
                file_id: Local::now().timestamp() as u32,
                file_name: filename.clone(),
                attr: attr as u8,
                size: size,
                mtime: Local::now().time(),
                atime: Local::now().time(),
                crtime: Local::now().time(),
                is_selected: false,
            };
            let ref mut files_add = *files_add_clone.borrow_mut();
            files_add.push(file_info);//添加待发送文件
        }
        file_chooser.destroy();
    });
    let chat_window2 = chat_window.clone();
    let files_add_folder_clone = pre_send_files.clone();
    button4.connect_clicked(move|_|{
        let file_chooser = gtk::FileChooserDialog::new(
            Some("打开文件夹"), Some(&chat_window2), gtk::FileChooserAction::SelectFolder);
        file_chooser.add_buttons(&[
            ("选择文件夹", gtk::ResponseType::Ok.into()),
            ("取消", gtk::ResponseType::Cancel.into()),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok.into() {
            let filename: PathBuf = file_chooser.get_filename().unwrap();
            let metadata: Metadata = fs::metadata(&filename).unwrap();
            let size = metadata.len();
            let attr = if metadata.is_file() {
                ::constant::IPMSG_FILE_REGULAR
            }else if metadata.is_dir() {
                ::constant::IPMSG_FILE_DIR
            }else {
                panic!("oh no!");
            };
            let modify_time: time::SystemTime = metadata.modified().unwrap();
            let chrono_time = ::util::system_time_to_date_time(modify_time);
            let local_time = chrono_time.with_timezone(&::chrono::Local);
            let file_info = model::FileInfo {
                file_id: Local::now().timestamp() as u32,
                file_name: filename.clone(),
                attr: attr as u8,
                size: size,
                mtime: Local::now().time(),
                atime: Local::now().time(),
                crtime: Local::now().time(),
                is_selected: false,
            };
            let ref mut files_add = *files_add_folder_clone.borrow_mut();
            files_add.push(file_info);//添加待发送文件
        }
        file_chooser.destroy();
    });
    chat_window.show_all();
    chat_window.connect_delete_event(move|_, _| {
        GLOBAL_WINDOWS.with(|global| {
            if let Some((ref mut map1, _)) = *global.borrow_mut() {
                map1.remove(&ip_str_3);
            }
        });
        Inhibit(false)
    });
    let clone_chat = chat_window.clone();
    let clone_hist_view = text_view.clone();
    ChatWindow{ win: clone_chat, his_view:  clone_hist_view, ip: ip_str_2}
}
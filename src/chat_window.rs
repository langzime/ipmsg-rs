use gtk::prelude::*;
use gtk::{
    self, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment, ButtonBox,
};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;
use std::path::{PathBuf, Path};
use std::fs::{self, File, Metadata, ReadDir};
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};
use chrono::prelude::*;
use model::{self, Packet, ShareInfo, ReceivedSimpleFileInfo};
use message;
use constant;
use app::GLOBAL_WINDOWS;

#[derive(Clone)]
pub struct ChatWindow {
    pub win :Window,
    pub his_view :TextView,
    pub ip :String,
    pub pre_send_files :Arc<RefCell<Vec<model::FileInfo>>>,
    pub received_store :Option<ListStore>,
    pub received_files: Arc<RefCell<Vec<ReceivedSimpleFileInfo>>>
}

pub fn create_chat_window<S: Into<String>>(name :S, host_ip :S, packet: Option<Packet>, received_files: Option<Vec<ReceivedSimpleFileInfo>>) -> ChatWindow {
    let name: String = name.into();
    let host_ip: String = host_ip.into();
    let ip_str = host_ip.clone();
    let ip_str1 = host_ip.clone();
    let ip_str2 = host_ip.clone();
    let ip_str3 = host_ip.clone();
    let ip_str4 = host_ip.clone();
    let ip_str5 = host_ip.clone();
    let chat_title = &format!("和{}({})聊天窗口", name, host_ip);

    let glade_src = include_str!("chat_window.glade");
    let builder = Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let chat_window: Window = builder.get_object("chat_window").unwrap();
    chat_window.set_title(chat_title);
    chat_window.set_border_width(5);
    let text_view_history: TextView = builder.get_object("text_view_history").unwrap();
    let text_view_presend: TextView = builder.get_object("text_view_presend").unwrap();
    let tree_view_presend: TreeView = builder.get_object("tree_view_presend").unwrap();//tree_view_received
    let tree_view_received: TreeView = builder.get_object("tree_view_received").unwrap();
    append_column(&tree_view_presend, 0, "待发送文件");
    append_column(&tree_view_received, 0, "收到的文件");


    if let Some(pac) = packet {
        let additional_section =  pac.additional_section.unwrap();
        let v: Vec<&str> = additional_section.split('\0').into_iter().collect();
        &text_view_history.get_buffer().unwrap().set_text(format!("{}:{}\n", name, v[0]).as_str());
    }
    let btn_clear: Button = builder.get_object("btn_clear").unwrap();
    let btn_send: Button = builder.get_object("btn_send").unwrap();//btn_file
    let btn_file: Button = builder.get_object("btn_file").unwrap();
    let btn_dir: Button = builder.get_object("btn_dir").unwrap();

    let text_view_presend_clone = text_view_presend.clone();
    let text_view_history_clone = text_view_history.clone();
    let arc_received_files: Arc<RefCell<Vec<ReceivedSimpleFileInfo>>> = Arc::new(RefCell::new(Vec::new()));
    let pre_send_files: Arc<RefCell<Vec<model::FileInfo>>> = Arc::new(RefCell::new(Vec::new()));//待发送文件列表
    let pre_send_files_model = create_and_fill_model();
    let pre_send_files_model_send = pre_send_files_model.clone();
    tree_view_presend.set_model(Some(&pre_send_files_model_send));
    let pre_received_files_model = create_and_fill_model1();
    let pre_send_files_model_clone = pre_received_files_model.clone();
    tree_view_received.set_model(Some(&pre_send_files_model_clone));
    let files_send_clone = pre_send_files.clone();
    btn_send.connect_clicked(move|_|{
        let (start_iter, mut end_iter) = text_view_presend_clone.get_buffer().unwrap().get_bounds();
        let context :&str = &text_view_presend_clone.get_buffer().unwrap().get_text(&start_iter, &end_iter, false).unwrap();
        message::send_ipmsg(context.to_owned(), files_send_clone.clone(), ip_str2.clone());
        (*files_send_clone.borrow_mut()).clear();
        pre_send_files_model_send.clear();
        let (his_start_iter, mut his_end_iter) = text_view_history_clone.get_buffer().unwrap().get_bounds();
        &text_view_history_clone.get_buffer().unwrap().insert(&mut his_end_iter, format!("{}:{}\n", "我", context).as_str());
        &text_view_presend_clone.get_buffer().unwrap().set_text("");
    });

    let chat_window_open_save = chat_window.clone();
    //let pre_send_files_model_clone1 = pre_received_files_model.clone();
    tree_view_received.connect_row_activated(move |tree_view, tree_path, tree_view_column| {
        let selection = tree_view.get_selection();
        if let Some((model, iter)) = selection.get_selected() {
            let name = model.get_value(&iter, 0).get::<String>().unwrap();
            let fid = model.get_value(&iter, 1).get::<u32>().unwrap();
            let pid = model.get_value(&iter, 2).get::<u32>().unwrap();
            let file_type = model.get_value(&iter, 3).get::<u8>().unwrap();
            let file_chooser = gtk::FileChooserDialog::new(
                Some("保存文件"), Some(&chat_window_open_save), gtk::FileChooserAction::CreateFolder);
            file_chooser.add_buttons(&[
                ("保存", gtk::ResponseType::Ok.into()),
                ("取消", gtk::ResponseType::Cancel.into()),
            ]);
            if file_chooser.run() == gtk::ResponseType::Ok.into() {
                let base_filename: PathBuf = file_chooser.get_filename().unwrap();
                let base_filename_clone = base_filename.clone();
                let target_ip = format!("{}:{}", ip_str4, constant::IPMSG_DEFAULT_PORT);
                info!("choosed {:?} {:?} {:?} {:?} {:?}", name, fid, pid, base_filename.clone(), file_type);
                let p1 = pid.clone();
                let f1 = fid.clone();
                let p2 = pid.clone();
                let f2 = fid.clone();
                let in_ip = ip_str5.clone();
                thread::spawn(move|| {
                    if let Ok(_) = ::download::download(target_ip, base_filename_clone, p1, f1, name, file_type as u32) {
                        //::成功删除
                        ::glib::idle_add(move || ::demons::remove_downloaded_file(&in_ip, p2, f2));
                    }else {
                        error!("download error!!");
                    }
                });
            }
            file_chooser.destroy();
        }
    });

    let text_view_presend_clone = text_view_presend.clone();
    btn_clear.connect_clicked(move|_|{
        &text_view_presend_clone.get_buffer().unwrap().set_text("");
    });

    let chat_window_open_file = chat_window.clone();
    let pre_send_files_open_file = pre_send_files.clone();
    let pre_send_files_model_file = pre_send_files_model.clone();

    btn_file.connect_clicked(move|_|{
        let file_chooser = gtk::FileChooserDialog::new(
            Some("打开文件"), Some(&chat_window_open_file), gtk::FileChooserAction::Open);
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
            let name = filename.file_name().unwrap().to_str().unwrap();
            let file_info = model::FileInfo {
                file_id: Local::now().timestamp() as u32,
                file_name: filename.clone(),
                name: name.to_owned(),
                attr: attr as u8,
                size: size,
                mtime: Local::now().time(),
                atime: Local::now().time(),
                crtime: Local::now().time(),
                is_selected: false,
            };
            let ref mut files_add = *pre_send_files_open_file.borrow_mut();
            files_add.push(file_info.clone());//添加待发送文件
            pre_send_files_model_file.insert_with_values(None, &[0, 1], &[&&name, &format!("{}", &file_info.file_id)]);
        }
        file_chooser.destroy();
    });

    let chat_window_open_dir = chat_window.clone();
    let pre_send_files_open_dir = pre_send_files.clone();
    let pre_send_files_model_dir = pre_send_files_model.clone();

    btn_dir.connect_clicked(move|_|{
        let file_chooser = gtk::FileChooserDialog::new(
            Some("打开文件夹"), Some(&chat_window_open_dir), gtk::FileChooserAction::SelectFolder);
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
            let name = filename.file_name().unwrap().to_str().unwrap();
            let file_info = model::FileInfo {
                file_id: Local::now().timestamp() as u32,
                file_name: filename.clone(),
                name: name.to_owned(),
                attr: attr as u8,
                size: size,
                mtime: Local::now().time(),
                atime: Local::now().time(),
                crtime: Local::now().time(),
                is_selected: false,
            };
            let ref mut files_add = *pre_send_files_open_dir.borrow_mut();
            files_add.push(file_info.clone());//添加待发送文件
            pre_send_files_model_dir.insert_with_values(None, &[0, 1], &[&name, &format!("{}", &file_info.file_id)]);
        }
        file_chooser.destroy();
    });

    chat_window.connect_delete_event(move|_, _| {
        GLOBAL_WINDOWS.with(|global| {
            if let Some((ref mut map1, _)) = *global.borrow_mut() {
                map1.remove(&ip_str1);
            }
        });
        Inhibit(false)
    });

    let arc_received_files_clone = arc_received_files.clone();
    if let Some(received_files) = received_files.clone() {
        for file in &received_files {
            info!("init {}  {}", file.packet_id, file.file_id);
            pre_received_files_model.insert_with_values(None, &[0, 1, 2, 3], &[&&file.name, &&file.file_id, &&file.packet_id, &&file.attr]);
        }
        *arc_received_files_clone.borrow_mut() = received_files;
    }

    chat_window.show_all();
    let clone_chat = chat_window.clone();
    let clone_hist_view = text_view_history.clone();
    ChatWindow{ win: clone_chat, his_view:  clone_hist_view, ip: ip_str, pre_send_files: pre_send_files, received_store: Some(pre_send_files_model_clone), received_files: arc_received_files}
}

fn append_column(tree: &TreeView, id: i32, title: &str) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();
    column.pack_start(&cell, true);
    column.set_title(title);
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
    tree.set_headers_visible(true);
}

fn create_and_fill_model() -> ListStore {
    let model = ListStore::new(&[String::static_type(), String::static_type()]);
    model
}

fn create_and_fill_model1() -> ListStore {
    let model = ListStore::new(&[String::static_type(), u32::static_type(), u32::static_type(), u8::static_type()]);
    model
}

/// ip
fn modify_received_list(received_store :Option<ListStore>, received_files: Arc<RefCell<Vec<ReceivedSimpleFileInfo>>>) -> ::glib::Continue {

    ::glib::Continue(false)
}
use gtk::prelude::*;
use gtk::{
    self, CellRendererText, AboutDialog, IconSize, Image, Label, Window,
    ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    Widget, TextView, Fixed, ScrolledWindow, WrapMode
};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;
use std::path::{PathBuf, Path};
use std::fs::{self, File, Metadata, ReadDir};
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};
use chrono::prelude::*;
use log::{info, trace, warn};
use glib::clone;
use crate::core::GLOBLE_SENDER;
use crate::models::model::{self, Packet, ShareInfo, ReceivedSimpleFileInfo};
use crate::models::event::ModelEvent;
use crate::models::message;
use crate::constants::protocol;
//use crate::app::GLOBAL_CHATWINDOWS;

// make moving clones into closures more convenient


#[derive(Clone)]
pub struct ChatWindow {
    pub win :Window,
    pub his_view :TextView,
    pub ip :String,
    pub pre_send_files :Arc<RefCell<Vec<model::FileInfo>>>,
    pub received_store :ListStore,
}

pub fn create_chat_window<S: Into<String>>(name :S, host_ip :S) -> ChatWindow {
    let name: String = name.into();
    let host_ip: String = host_ip.into();
    let chat_title = &format!("和{}({})聊天窗口", name, host_ip);

    let glade_src = include_str!("chat_window.ui");
    let builder = Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let chat_window: Window = builder.object("chat_window").unwrap();
    chat_window.set_title(Some(chat_title));
    // chat_window.set_border_width(5);
    //历史
    let text_view_history: TextView = builder.object("text_view_history").unwrap();
    //待发送的
    let text_view_presend: TextView = builder.object("text_view_presend").unwrap();
    //待发送文件
    let tree_view_presend: TreeView = builder.object("tree_view_presend").unwrap();
    //接受的文件
    let tree_view_received: TreeView = builder.object("tree_view_received").unwrap();

    text_view_history.set_wrap_mode(WrapMode::WordChar);
    text_view_presend.set_wrap_mode(WrapMode::WordChar);
    append_column(&tree_view_presend, 0, "待发送文件");
    append_column(&tree_view_received, 0, "收到的文件");

    let btn_clear: Button = builder.object("btn_clear").unwrap();
    let btn_send: Button = builder.object("btn_send").unwrap();//btn_file
    let btn_file: Button = builder.object("btn_file").unwrap();
    let btn_dir: Button = builder.object("btn_dir").unwrap();

    //let text_view_presend_clone = text_view_presend.clone();
    let text_view_history_clone = text_view_history.clone();
    //let arc_received_files: Arc<RefCell<Vec<ReceivedSimpleFileInfo>>> = Arc::new(RefCell::new(Vec::new()));
    let pre_send_files: Arc<RefCell<Vec<model::FileInfo>>> = Arc::new(RefCell::new(Vec::new()));//待发送文件列表
    let pre_send_files_model = create_and_fill_model();
    tree_view_presend.set_model(Some(&pre_send_files_model));
    //let pre_send_files_model_send = pre_send_files_model.clone();
    let pre_received_files_model = create_and_fill_model1();
    tree_view_received.set_model(Some(&pre_received_files_model));
    let files_send_clone = pre_send_files.clone();
    btn_send.connect_clicked(clone!(@strong pre_send_files_model, @strong host_ip, @strong text_view_presend => move|_|{
        let (start_iter, mut end_iter) = text_view_presend.buffer().bounds();
        let context :&str = &text_view_presend.buffer().text(&start_iter, &end_iter, false);
        let (packet, share_file) = message::create_sendmsg(context.to_owned(), files_send_clone.clone().borrow().to_vec(), host_ip.clone());
        GLOBLE_SENDER.send(ModelEvent::SendOneMsg {to_ip: host_ip.clone(), packet, context: context.to_owned(), files: share_file}).unwrap();
        files_send_clone.borrow_mut().clear();
        pre_send_files_model.clear();
        text_view_presend.buffer().set_text("");
    }));

    let chat_window_open_save = chat_window.clone();
    tree_view_received.connect_row_activated(clone!(@weak chat_window_open_save, @strong host_ip => move |tree_view, tree_path, tree_view_column| {
        let selection = tree_view.selection();
        if let Some((model, iter)) = selection.selected() {
            let name: String = model.get_value(&iter, 0).get().unwrap();
            let fid: u32 = model.get_value(&iter, 1).get().unwrap();
            let pid: u32 = model.get_value(&iter, 2).get().unwrap();
            let file_type: u8 = model.get_value(&iter, 3).get().unwrap();
            let size: u64 = model.get_value(&iter, 4).get().unwrap();
            let mtime: i64 = model.get_value(&iter, 5).get().unwrap();

            let file_chooser = gtk::FileChooserDialog::new(
                Some("保存文件"),
                Some(&chat_window_open_save),
                gtk::FileChooserAction::SelectFolder,
                &[("保存", gtk::ResponseType::Ok), ("取消", gtk::ResponseType::Cancel)],
            );

            let host_ip = host_ip.clone();
            file_chooser.connect_response(move |d: &gtk::FileChooserDialog, response: gtk::ResponseType| {
                if response == gtk::ResponseType::Ok {
                    let file = d.file().expect("Couldn't get file");
                    let save_base_path: PathBuf = file.path().expect("Couldn't get file path");
                    info!("choosed {:?} {:?} {:?} {:?} {:?}", name, fid, pid, save_base_path, file_type);
                    GLOBLE_SENDER.send(ModelEvent::PutDownloadTaskInPool{ file: ReceivedSimpleFileInfo{
                        file_id: fid,
                        packet_id: pid,
                        name: name.clone(),
                        attr: file_type,
                        size,
                        mtime
                    }, save_base_path, download_ip: host_ip.clone() });
                }
                d.close();
            });
            file_chooser.show();
        }
    }));

    btn_clear.connect_clicked(clone!(@strong text_view_presend => move|_|{
        text_view_presend.buffer().set_text("");
    }));

    let chat_window_open_file = chat_window.clone();
    let pre_send_files_open_file = pre_send_files.clone();
    let chat_window_open_save = chat_window_open_save.clone();
    btn_file.connect_clicked(clone!(@strong pre_send_files_model, @weak chat_window_open_save => move|_|{

        let file_chooser = gtk::FileChooserDialog::new(
                Some("打开文件"),
                Some(&chat_window_open_save),
                gtk::FileChooserAction::Open,
                &[("选择文件", gtk::ResponseType::Ok), ("取消", gtk::ResponseType::Cancel)],
            );

        let pre_send_files_open_file = pre_send_files_open_file.clone();
        let pre_send_files_model = pre_send_files_model.clone();
        let file_chooser_tmp = file_chooser.clone();
        file_chooser.connect_response(move |d: &gtk::FileChooserDialog, response: gtk::ResponseType| {
            if response == gtk::ResponseType::Ok {
                let filename: PathBuf = file_chooser_tmp.clone().file().unwrap().path().unwrap();
                let metadata: Metadata = fs::metadata(&filename).unwrap();
                let size = metadata.len();
                let attr = if metadata.is_file() {
                    crate::constants::protocol::IPMSG_FILE_REGULAR
                }else if metadata.is_dir() {
                    crate::constants::protocol::IPMSG_FILE_DIR
                }else {
                    panic!("oh no!");
                };
                let modify_time: time::SystemTime = metadata.modified().unwrap();
                let chrono_time = crate::util::system_time_to_date_time(modify_time);
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
                    crtime: Local::now().time()
                };
                let ref mut files_add = *pre_send_files_open_file.borrow_mut();
                files_add.push(file_info.clone());//添加待发送文件
                //pre_send_files_model.insert_with_values(None, &[0, 1], &[&&name, &format!("{}", &file_info.file_id)]);
                pre_send_files_model.insert_with_values(None, &[(0, &&name), (1, &format!("{}", &file_info.file_id))]);
            }
            d.close();
        });
        file_chooser.show();
    }));

    let chat_window_open_dir = chat_window.clone();
    let pre_send_files_open_dir = pre_send_files.clone();
    let chat_window_open_save = chat_window_open_save.clone();
    btn_dir.connect_clicked(clone!(@strong pre_send_files_model, @strong pre_send_files, @weak chat_window_open_save => move|_|{
        let file_chooser = gtk::FileChooserDialog::new(
                Some("打开文件夹"),
                Some(&chat_window_open_save),
                gtk::FileChooserAction::SelectFolder,
                &[("选择文件夹", gtk::ResponseType::Ok), ("取消", gtk::ResponseType::Cancel)],
            );
        let pre_send_files_open_dir = pre_send_files.clone();
        let pre_send_files_model = pre_send_files_model.clone();
        let file_chooser_tmp = file_chooser.clone();
        file_chooser.connect_response(move |d: &gtk::FileChooserDialog, response: gtk::ResponseType| {
                if response == gtk::ResponseType::Ok {
                    let filename: PathBuf = file_chooser_tmp.file().unwrap().path().unwrap();
                    let metadata: Metadata = fs::metadata(&filename).unwrap();
                    let size = metadata.len();
                    let attr = if metadata.is_file() {
                        crate::constants::protocol::IPMSG_FILE_REGULAR
                    }else if metadata.is_dir() {
                        crate::constants::protocol::IPMSG_FILE_DIR
                    }else {
                        panic!("oh no!");
                    };
                    let modify_time: time::SystemTime = metadata.modified().unwrap();
                    let chrono_time = crate::util::system_time_to_date_time(modify_time);
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
                    };
                    let ref mut files_add = *pre_send_files_open_dir.borrow_mut();
                    files_add.push(file_info.clone());//添加待发送文件
                    pre_send_files_model.insert_with_values(None, &[(0, &name), (1, &format!("{}", &file_info.file_id))]);
                }
                d.close();
        });
        file_chooser.show();
    }));


    /*chat_window.connect_close_request(clone!(@strong model_sender, @strong host_ip, @weak chat_window => @default-return Inhibit(false),  move|_, _| {
        model_sender.send(ModelEvent::ClickChatWindowCloseBtn{from_ip: host_ip.clone()}).unwrap();
        unsafe {
                chat_window.destroy();
            }
        return Inhibit(false);
    }));*/

    chat_window.connect_close_request(clone!(@strong host_ip => @default-return glib::signal::Inhibit(false), move |window| {
        GLOBLE_SENDER.send(ModelEvent::ClickChatWindowCloseBtn{from_ip: host_ip.clone()}).unwrap();
        if let Some(application) = window.application() {
            application.remove_window(window);
        }
        glib::signal::Inhibit(false)
    }));


    // chat_window.show_all();
    chat_window.present();
    let clone_chat = chat_window.clone();
    let clone_hist_view = text_view_history.clone();
    ChatWindow{ win: clone_chat, his_view:  clone_hist_view, ip: host_ip, pre_send_files, received_store: pre_received_files_model}
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
    let model = ListStore::new(&[String::static_type(), u32::static_type(), u32::static_type(), u8::static_type(), u64::static_type(), i64::static_type()]);
    model
}

/// ip
fn modify_received_list(received_store :Option<ListStore>, received_files: Arc<RefCell<Vec<ReceivedSimpleFileInfo>>>) -> Continue {

    Continue(false)
}
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
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use constant::{self, IPMSG_SENDMSG, IPMSG_FILEATTACHOPT, IPMSG_DEFAULT_PORT, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT};
use model::{self, User, OperUser, Operate, ShareInfo, Packet, FileInfo, ReceivedSimpleFileInfo, ReceivedPacketInner};
use chat_box::{self, ChatBox};

/*thread_local!(
    pub static GLOBAL_USERLIST: RefCell<Option<(::gtk::ListStore, mpsc::Receiver<OperUser>)>> = RefCell::new(None);//用户列表
    pub static GLOBAL_UDPSOCKET: RefCell<Option<UdpSocket>> = RefCell::new(None);//udp全局变量
    pub static GLOBAL_CHATWINDOWS: RefCell<Option<(HashMap<String, ChatBox>, mpsc::Receiver<ReceivedPacketInner>)>> = RefCell::new(None);//聊天窗口列表
    pub static GLOBAL_SHARELIST: RefCell<Option<Arc<Mutex<Vec<ShareInfo>>>>> = RefCell::new(Some(Arc::new(Mutex::new(Vec::new()))));//发送文件列表
    pub static GLOBAL_RECEIVELIST: RefCell<Option<(::gtk::ListStore, mpsc::Receiver<ShareInfo>)>> = RefCell::new(None);//接收文件列表
);*/

thread_local!(
    //用户列表
    pub static GLOBAL_USERLIST: RefCell<Option<(::gtk::ListStore)>> = RefCell::new(None);
    //接收文件列表
    pub static GLOBAL_RECEIVELIST: RefCell<Option<(::gtk::ListStore)>> = RefCell::new(None);
);

struct App {
    pub menubar: MenuBar,
    pub window: Window,
    pub chat_windows: HashMap<String, ChatBox>,
    pub udpsocket: UdpSocket,
}

impl App {
    fn new() -> App {
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

        tree.connect_row_activated(move |tree_view, tree_path, tree_view_column| {
            let selection = tree_view.get_selection();
            if let Some((model, iter)) = selection.get_selected() {
                let ip_str = model.get_value(&iter, 3).get::<String>().unwrap();
                let name = model.get_value(&iter, 0).get::<String>().unwrap();
                ::glib::idle_add(move || create_or_open_chat(ip_str.clone(), name.clone(), None));
            }
        });

        window.add(&v_box);
        window.show_all();

        let socket: UdpSocket = match UdpSocket::bind(::constant::addr.as_str()) {
            Ok(s) => {
                info!("udp server start listening! {:?}", ::constant::addr.as_str());
                s
            },
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        GLOBAL_USERLIST.with(move |global| {
            *global.borrow_mut() = Some(model);
        });

        App {
            menubar: menu_bar,
            window: window,
            chat_windows: HashMap::new(),
            udpsocket: socket,
        }
    }

    fn start_listening(&self, sender: mpsc::Sender<Packet>) {
        let socket_clone = self.udpsocket.try_clone().unwrap();
        thread::spawn(move||{
            loop {
                let mut buf = [0; 2048];
                match socket_clone.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        //let receive_str = unsafe { str::from_utf8_unchecked(&buf[0..amt])};
                        //todo 默认是用中文编码 还没想到怎么做兼容
                        let receive_str = GB18030.decode(&buf[0..amt], DecoderTrap::Strict).unwrap();
                        info!("receive raw message -> {:?} from ip -> {:?}", receive_str, src.ip());
                        let v: Vec<&str> = receive_str.splitn(6, |c| c == ':').collect();
                        if v.len() > 4 {
                            let mut packet = Packet::from(String::from(v[0]),
                                                          String::from(v[1]),
                                                          String::from(v[2]),
                                                          String::from(v[3]),
                                                          v[4].parse::<u32>().unwrap(),
                                                          None
                            );
                            if v.len() > 5 {
                                packet.additional_section = Some(String::from(v[5]));
                            }
                            packet.ip = src.ip().to_string();
                            sender.send(packet);
                        }else {
                            println!("Invalid packet {} !", receive_str);
                        }
                    },
                    Err(e) => {
                        info!("couldn't recieve a datagram: {}", e);
                    }
                }
            }
        });
    }

    fn boot_broadcast(&self) {
        let socket_clone = self.udpsocket.try_clone().unwrap();
        thread::spawn(move||{
            let packet = Packet::new(IPMSG_BR_ENTRY|IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::hostname, *constant::hostname)));
            socket_clone.set_broadcast(true).unwrap();
            let addr:String = format!("{}:{}", ::constant::IPMSG_LIMITED_BROADCAST, ::constant::IPMSG_DEFAULT_PORT);
            socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str());
        });
    }

    fn start_message_processor(&self, receiver: Arc<Mutex<mpsc::Receiver<Packet>>>) {
        let socket_clone = self.udpsocket.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                let packet: Packet = receiver.lock().unwrap().recv().unwrap();
                let mut extstr = String::new();
                if let Some(ref additional_section) = (&packet).additional_section {
                    extstr = additional_section.to_owned();
                }
                let opt = constant::get_opt((&packet).command_no);
                let cmd = constant::get_mode((&packet).command_no);
                info!("{:?}", &packet);
                info!("{:x}", &packet.packet_no.parse::<i32>().unwrap());
                info!("cmd {:x} opt {:x} opt section {:?}", cmd, opt, extstr);
                let addr:String = format!("{}:{}", &packet.ip, constant::IPMSG_DEFAULT_PORT);
                if opt&constant::IPMSG_SENDCHECKOPT != 0 {
                    let recvmsg = Packet::new(constant::IPMSG_RECVMSG, Some((&packet).packet_no.to_string()));
                    socket_clone.send_to(recvmsg.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                }
                if cmd == constant::IPMSG_BR_EXIT {//收到下线通知消息
                    let user = User::new((&packet).sender_name.to_owned(), (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), "".to_owned());
                    ::glib::idle_add(move || del_user(user.clone()));
                } else if cmd == constant::IPMSG_BR_ENTRY {//收到上线通知消息
                    ///扩展段 用户名|用户组
                    let ext_vec = extstr.splitn(2, |c| c == ':').collect::<Vec<&str>>();
                    let ansentry_packet = Packet::new(constant::IPMSG_ANSENTRY, None);
                    socket_clone.set_broadcast(false).unwrap();
                    socket_clone.send_to(ansentry_packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                    let group_name = if ext_vec.len() > 2 {
                        ext_vec[1].to_owned()
                    }else {
                        "".to_owned()
                    };
                    let user_name = if ext_vec.len() > 1&& !ext_vec[0].is_empty() {
                        ext_vec[0].to_owned()
                    }else {
                        (&packet).sender_name.to_owned()
                    };
                    let user = User::new(user_name, (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), group_name);
                    ::glib::idle_add(move || add_user(user.clone()));
                }else if cmd == constant::IPMSG_ANSENTRY {//通报新上线
                    let user = User::new((&packet).sender_name.to_owned(), (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), "".to_owned());
                    ::glib::idle_add(move || add_user(user.clone()));
                }else if cmd == constant::IPMSG_SENDMSG {//收到发送的消息
                    //文字消息|文件扩展段
                    let ext_vec = extstr.split('\0').collect::<Vec<&str>>();
                    if opt&constant::IPMSG_SECRETOPT != 0 {//是否是密封消息
                        info!("i am secret message !");
                    }
                    let msg_str = if ext_vec.len() > 0 { ext_vec[0] } else { "" };
                    //文字消息内容|文件扩展
                    let mut files_opt: Option<Vec<ReceivedSimpleFileInfo>> = None;
                    if opt&constant::IPMSG_FILEATTACHOPT != 0 {
                        if ext_vec.len() > 1 {
                            let files_str: &str = ext_vec[1];
                            info!("i have file attachment {:?}", files_str);
                            let files = files_str.split(constant::FILELIST_SEPARATOR).into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                            let mut simple_file_infos = Vec::new();
                            for file_str in files {
                                let file_attr = file_str.splitn(6, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                                if file_attr.len() >= 5 {
                                    let file_id = file_attr[0].parse::<u32>().unwrap();
                                    let file_name = file_attr[1];
                                    let size = file_attr[2];//大小
                                    let mmtime = file_attr[3];//修改时间
                                    let mmtime_num = i64::from_str_radix(mmtime, 16).unwrap();//时间戳
                                    let file_attr = file_attr[4].parse::<u32>().unwrap();//文件属性
                                    let ntime = NaiveDateTime::from_timestamp(mmtime_num as i64, 0);
                                    info!("{}", ntime.format("%Y-%m-%d %H:%M:%S").to_string());
                                    if file_attr == constant::IPMSG_FILE_REGULAR {
                                        info!("i am ipmsg_file_regular");
                                    }else if file_attr == constant::IPMSG_FILE_DIR {
                                        info!("i am ipmsg_file_dir");
                                    }else {
                                        panic!("no no type")
                                    }
                                    let simple_file_info = ReceivedSimpleFileInfo {
                                        file_id: file_id,
                                        packet_id: (&packet).packet_no.parse::<u32>().unwrap(),
                                        name: file_name.to_owned(),
                                        attr: file_attr as u8,
                                        is_active: 0,
                                    };
                                    simple_file_infos.push(simple_file_info);
                                }
                            }
                            if simple_file_infos.len() > 0 {
                                files_opt = Some(simple_file_infos);
                            }
                        };
                    }
                    let packet_clone = packet.clone();
                    let received_packet_inner = ReceivedPacketInner::new((&packet).ip.to_owned()).packet(packet_clone).option_opt_files(files_opt);
                    //remained_sender.send(received_packet_inner);
                    //::glib::idle_add(create_or_open_chat);
                }else {

                }
            }
        });
    }

}

pub fn start(){
    drop(::env_logger::init().unwrap());
    info!("starting up");
    if gtk::init().is_err() {
        info!("Failed to initialize GTK.");
        return;
    }
    let (sender, receiver): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let app = App::new();
    //app::spwan();
    app.start_listening(sender.clone());
    app.boot_broadcast();
    app.start_message_processor(receiver);
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
    let model = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type()]);
    model
}

fn add_user(user: User)-> ::glib::Continue {
    GLOBAL_USERLIST.with(|global| {
        if let Some(ref store) = *global.borrow() {
            let mut in_flag = false;
            if let Some(first) = store.get_iter_first(){//拿出来第一条
                let mut num :u32 = store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                let ip = store.get_value(&first, 3).get::<String>().unwrap();//获取ip
                if ip == user.ip {
                    in_flag = true;
                }else {
                    loop {
                        num = num + 1;
                        if let Some(next_iter) = store.get_iter_from_string(&num.to_string()){
                            let next_ip = store.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
                            if next_ip == user.ip {
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
                store.insert_with_values(None, &[0, 1, 2, 3], &[&&user.name, &&user.group, &&user.host, &&user.ip]);
            }
        }
    });
    ::glib::Continue(false)
}

fn del_user(user: User)-> ::glib::Continue {
    GLOBAL_USERLIST.with(|global| {
        if let Some(ref store) = *global.borrow() {
            if let Some(first) = store.get_iter_first(){//拿出来第一条
                let mut num :u32 = store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                let ip = store.get_value(&first, 3).get::<String>().unwrap();//获取ip
                if ip == user.ip {
                    store.remove(&first);
                }else {
                    loop {
                        num = num + 1;
                        if let Some(next_iter) = store.get_iter_from_string(&num.to_string()){
                            let next_ip = store.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
                            if next_ip == user.ip {
                                store.remove(&next_iter);
                                break;
                            }
                        }else{
                            break;
                        }
                    }
                }
            }
        }
    });
    ::glib::Continue(false)
}

fn create_or_open_chat<S: Into<String>>(name :S, host_ip :S, packet: Option<Packet>) -> ::glib::Continue {
    chat_box::create_chat_window(name, host_ip, packet);
    ::glib::Continue(false)
}
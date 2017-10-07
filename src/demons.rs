
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;
use std::net::{TcpStream, TcpListener};
use std::sync::mpsc;
use std::collections::HashMap;
use std::time::{self, Duration, SystemTime, UNIX_EPOCH};
use std::io::Read;
use std::io::Write;
use std::fs::{self, File, Metadata, ReadDir};
use std::io::{BufReader, BufWriter};
use std::path::{PathBuf, Path};

use constant;
use model::{self, User, OperUser, Operate, ShareInfo, FileInfo, ReceivedSimpleFileInfo, ReceivedPacketInner};
use chrono::prelude::*;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use gtk::prelude::*;
use gtk::{
    self, TreeModelExt, CellRendererText, AboutDialog, CheckMenuItem, IconSize, Image, Label, Menu, MenuBar, MenuItem, Window,
    WindowPosition, WindowType, StatusIcon, ListStore, TreeView, TreeViewColumn, Builder, Grid, Button, Orientation,
    ReliefStyle, Widget, TextView, Fixed, ScrolledWindow, Alignment,
};
use message;
use util;
use chat_window::{self, ChatWindow};
use app::{self, GLOBAL_UDPSOCKET, GLOBAL_SHARELIST, GLOBAL_CHATWINDOWS, GLOBAL_USERLIST};

///启动消息监听线程
pub fn start_daemon(sender: mpsc::Sender<Packet>){
    ::demons::GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
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
    });
}

///信息处理
pub fn start_message_processer(receiver :mpsc::Receiver<Packet>, sender :mpsc::Sender<OperUser>, remained_sender :mpsc::Sender<ReceivedPacketInner>){
    ::demons::GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
            thread::spawn(move || {
                loop {
                    let packet: Packet = receiver.recv().unwrap();
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
                        sender.send(OperUser::new(user, Operate::REMOVE));
                        ::glib::idle_add(receive);
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
                        sender.send(OperUser::new(user, Operate::ADD));
                        ::glib::idle_add(move || receive());
                    }else if cmd == constant::IPMSG_ANSENTRY {//通报新上线
                        let user = User::new((&packet).sender_name.to_owned(), (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), "".to_owned());
                        sender.send(OperUser::new(user, Operate::ADD));
                        ::glib::idle_add(receive);
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
                        remained_sender.send(received_packet_inner);
                        ::glib::idle_add(create_or_open_chat);
                    }else {

                    }
                }
            });
        }
    });
}

pub fn start_file_processer() {
    GLOBAL_SHARELIST.with(|global_shares| {
        if let Some(ref share_infos_arc) = *global_shares.borrow() {
            let search_arc_tmp = share_infos_arc.clone();
            thread::spawn(move || {
                let tcp_listener: TcpListener = TcpListener::bind(constant::addr.as_str()).unwrap();
                info!("tcp server start listening! {:?}", constant::addr.as_str());
                for stream in tcp_listener.incoming() {
                    let base_stream = stream.unwrap().try_clone().unwrap();
                    let search_arc = search_arc_tmp.clone();
                    thread::spawn(move || {
                        let mut stream_echo = base_stream;
                        let mut buf = [0; 2048];
                        stream_echo.read(&mut buf[..]).unwrap();
                        let tmp_str = GB18030.decode(&buf, DecoderTrap::Strict).unwrap();
                        let receive_str = tmp_str.trim_right_matches('\u{0}');
                        info!("file_processer receive raw str {:?}", receive_str);
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
                                let cmd = constant::get_mode(packet.command_no);
                                if cmd == constant::IPMSG_GETFILEDATA {
                                    //文件请求
                                    let file_attr = v[5].splitn(4, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                                    info!("file packet parse {:?}", file_attr);
                                    if file_attr.len() >= 3 {
                                        let packet_id = i64::from_str_radix(file_attr[0], 16).unwrap() as u32;
                                        let file_id = i64::from_str_radix(file_attr[1], 16).unwrap();
                                        let offset = file_attr[2].parse::<u32>().unwrap();
                                        let mut search_result: Option<ShareInfo> = Option::None;
                                        {
                                            let search = search_arc.lock().unwrap();
                                            let ref vec: Vec<ShareInfo> = *search;
                                            let result = vec.iter().find(|ref s| s.packet_no == packet_id);
                                            search_result = result.cloned();
                                        }
                                        if let Some(result_share_file) = search_result {
                                            let file_info = result_share_file.file_info.iter().find(|ref f| f.file_id == file_id as u32);
                                            if let Some(file_info) = file_info {
                                                let mut f: File = File::open(&file_info.file_name).unwrap();
                                                let mut buf = [0; 1024];
                                                let mut buffer = BufWriter::new(stream_echo);
                                                while let Ok(bytes_read) = f.read(&mut buf) {
                                                    if bytes_read == 0 { break; }
                                                    buffer.write(&buf[..bytes_read]).unwrap();
                                                }
                                                buffer.flush().unwrap();
                                            }
                                        }

                                    }
                                } else if cmd == constant::IPMSG_GETDIRFILES {
                                    //dir todo send dir has problem
                                    let file_attr = v[5].splitn(3, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                                    info!("file dir packet parse {:?}", file_attr);
                                    if file_attr.len() >= 2 {
                                        let packet_id = i64::from_str_radix(file_attr[0], 16).unwrap() as u32;
                                        let file_id = i64::from_str_radix(file_attr[1], 16).unwrap();
                                        let mut search_result: Option<ShareInfo> = Option::None;
                                        {
                                            let search = search_arc.lock().unwrap();
                                            let ref vec: Vec<ShareInfo> = *search;
                                            let result = vec.iter().find(|ref s| s.packet_no == packet_id);
                                            search_result = result.cloned();
                                        }
                                        if let Some(result_share_file) = search_result {
                                            let file_info = result_share_file.file_info.iter().find(|ref f| f.file_id == file_id as u32);
                                            if let Some(file_info) = file_info {
                                                let ref root_path: PathBuf = file_info.file_name;
                                                let mut buffer = BufWriter::new(stream_echo.try_clone().unwrap());
                                                send_dir(root_path, &mut buffer);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            info!("Invalid packet {:?} !", receive_str);
                        }
                    });
                }
            });
        }
    });
}

//send dir
pub fn send_dir(root_path: &PathBuf, mut buffer : & mut BufWriter<TcpStream>) {
    buffer.write(util::utf8_to_gb18030(&make_header(&root_path)).as_slice());//root dir
    info!("{:?}", &make_header(&root_path));
    if root_path.is_dir() {
        for sub_path in fs::read_dir(root_path).unwrap() {
            let sub = &sub_path.unwrap().path();
            if sub.is_file() {
                let header = make_header(sub);
                buffer.write(util::utf8_to_gb18030(&header).as_slice());
                info!("{:?}", header);
                let mut buf = [0; 1024 * 4];
                let mut f: File = File::open(sub).unwrap();
                while let Ok(bytes_read) = f.read(&mut buf) {
                    if bytes_read == 0 { break; }
                    buffer.write(&buf[..bytes_read]);
                }
            }else {
                send_dir(sub, &mut buffer);
            }
        }
    }

    buffer.write("000D:.:0:3:0:".as_bytes());
    info!("{:?}", "000D:.:0:3:0:");
}

pub fn make_header(path: &PathBuf) -> String {
    let path_metadata: Metadata = fs::metadata(&path).unwrap();
    let file_attr = if path_metadata.is_dir() {
        constant::IPMSG_FILE_DIR
    } else {
        constant::IPMSG_FILE_REGULAR
    };
    let file_name: &str = &path.file_name().unwrap().to_str().unwrap();
    let mut header = String::new();
    header.push_str(":");
    header.push_str(file_name);//filename
    header.push_str(":");
    header.push_str(format!("{:x}", path_metadata.len()).as_str());//filesize
    header.push_str(":");
    header.push_str(format!("{:x}", file_attr).as_str());//fileattr
    let timestamp_now = Local::now().timestamp();
    header.push_str(format!(":{:x}={:x}:{:x}={:x}:", constant::IPMSG_FILE_CREATETIME, timestamp_now, constant::IPMSG_FILE_MTIME, timestamp_now).as_str());//
    let mut length = util::utf8_to_gb18030(&header).len();
    length = length + format!("{:0>4x}", length).len();
    header.insert_str(0, format!("{:0>4x}", length).as_str());
    header
}

pub fn create_or_open_chat() -> ::glib::Continue {
    GLOBAL_CHATWINDOWS.with(|global| {
        if let Some((ref mut map, ref rx)) = *global.borrow_mut() {
            if let Ok(receivedPacketInner) = rx.try_recv() {
                let host_ip = receivedPacketInner.clone().ip;
                let name = receivedPacketInner.clone().packet.map(|pac| pac.sender_name).unwrap_or("".to_owned());
                let received_files = receivedPacketInner.clone().opt_files;
                let packet = receivedPacketInner.clone().packet;
                let select_map = map.clone();
                if !host_ip.is_empty(){
                    if let Some(chat_win) = select_map.get(&host_ip) {
                        if let Some(pac) = packet {
                            let additional_section =  pac.additional_section.unwrap();
                            let v = additional_section.split('\0').into_iter().collect::<Vec<&str>>();
                            let (start, mut end) = chat_win.his_view.get_buffer().unwrap().get_bounds();
                            chat_win.his_view.get_buffer().unwrap().insert(&mut end, format!("{}:{}\n", pac.sender_name, v[0]).as_str());
                            if let Some(received_files) = received_files {
                                if let Some(ref pre_receive_file_store) = chat_win.received_store {
                                    for file in received_files {
                                        let received_file_id = file.file_id;
                                        let received_packet_id = file.packet_id;
                                        info!(" {}  {}", received_packet_id, received_file_id);
                                        let mut in_flag = false;
                                        if let Some(first) = pre_receive_file_store.get_iter_first(){
                                            let mut num :u32 = pre_receive_file_store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                                            let file_id = pre_receive_file_store.get_value(&first, 1).get::<u32>().unwrap();
                                            let packet_id = pre_receive_file_store.get_value(&first, 2).get::<u32>().unwrap();
                                            if file_id == received_file_id&&packet_id == received_packet_id {
                                                in_flag = true;
                                            }else {
                                                loop {
                                                    num = num + 1;
                                                    if let Some(next_iter) = pre_receive_file_store.get_iter_from_string(&num.to_string()){
                                                        let next_file_id = pre_receive_file_store.get_value(&next_iter, 1).get::<u32>().unwrap();
                                                        let next_packet_id = pre_receive_file_store.get_value(&next_iter, 2).get::<u32>().unwrap();
                                                        if next_file_id == received_file_id&&next_packet_id == received_packet_id {
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
                                            pre_receive_file_store.insert_with_values(None, &[0, 1, 2, 3], &[&&file.name, &&file.file_id, &&file.packet_id, &&file.attr]);
                                        }
                                    }
                                }
                            }
                        }
                    }else {
                        let ip_str1 = host_ip.clone();
                        let ip_str2 = host_ip.clone();
                        let chat_win = chat_window::create_chat_window(name, ip_str1, packet, received_files);
                        map.insert(ip_str2, chat_win);
                    }
                }
            }
        }
    });
    ::glib::Continue(false)
}

///
///
pub fn remove_downloaded_file(host_ip: &str, pid: u32, fid: u32) -> ::glib::Continue {
    GLOBAL_CHATWINDOWS.with(|global| {
        if let Some((ref mut map, _)) = *global.borrow_mut() {
            let select_map = map.clone();
            if !host_ip.is_empty() {
                if let Some(chat_win) = select_map.get(host_ip) {
                    if let Some(ref pre_receive_file_store) = chat_win.received_store {
                        let received_file_id = fid;
                        let received_packet_id = pid;
                        info!(" {}  {}", received_packet_id, received_file_id);
                        let mut in_flag = false;
                        if let Some(first) = pre_receive_file_store.get_iter_first(){
                            let mut num :u32 = pre_receive_file_store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                            let file_id = pre_receive_file_store.get_value(&first, 1).get::<u32>().unwrap();
                            let packet_id = pre_receive_file_store.get_value(&first, 2).get::<u32>().unwrap();
                            if file_id == received_file_id&&packet_id == received_packet_id {
                                pre_receive_file_store.remove(&first);
                            }else {
                                loop {
                                    num = num + 1;
                                    if let Some(next_iter) = pre_receive_file_store.get_iter_from_string(&num.to_string()){
                                        let next_file_id = pre_receive_file_store.get_value(&next_iter, 1).get::<u32>().unwrap();
                                        let next_packet_id = pre_receive_file_store.get_value(&next_iter, 2).get::<u32>().unwrap();
                                        if next_file_id == received_file_id&&next_packet_id == received_packet_id {
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
                }
            }
        }
    });
    ::glib::Continue(false)
}

fn receive() -> ::glib::Continue {
    GLOBAL_USERLIST.with(|global| {
        if let Some((ref store, ref rx)) = *global.borrow() {
            if let Ok(op_user) = rx.try_recv() {
                let income_user = op_user.user;
                let oper = op_user.oper;
                if oper == Operate::ADD {
                    let mut in_flag = false;
                    if let Some(first) = store.get_iter_first(){//拿出来第一条
                        let mut num :u32 = store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                        let ip = store.get_value(&first, 3).get::<String>().unwrap();//获取ip
                        if ip == income_user.ip {
                            in_flag = true;
                        }else {
                            loop {
                                num = num + 1;
                                if let Some(next_iter) = store.get_iter_from_string(&num.to_string()){
                                    let next_ip = store.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
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
                        store.insert_with_values(None, &[0, 1, 2, 3], &[&&income_user.name, &&income_user.group, &&income_user.host, &&income_user.ip]);
                    }
                }
                if oper == Operate::REMOVE {
                    if let Some(first) = store.get_iter_first(){//拿出来第一条
                        let mut num :u32 = store.get_string_from_iter(&first).unwrap().parse::<u32>().unwrap();//序号 会改变
                        let ip = store.get_value(&first, 3).get::<String>().unwrap();//获取ip
                        if ip == income_user.ip {
                            store.remove(&first);
                        }else {
                            loop {
                                num = num + 1;
                                if let Some(next_iter) = store.get_iter_from_string(&num.to_string()){
                                    let next_ip = store.get_value(&next_iter, 3).get::<String>().unwrap();//获取ip
                                    if next_ip == income_user.ip {
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
            }
        }
    });
    ::glib::Continue(false)
}
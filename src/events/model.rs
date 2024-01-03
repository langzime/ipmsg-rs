use std::thread;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::all::GB18030;
use chrono::prelude::*;
use std::collections::HashMap;
use async_channel::Sender;
use log::{debug, error, info, trace, warn};
use combine::parser::Parser;
use crate::models::model::{FileInfo, Packet, ReceivedPacketInner, ReceivedSimpleFileInfo, ShareInfo, User};
use crate::core::download::{ManagerPool, PoolFile};
use crate::core::fileserver::FileServer;
use crate::models::event::{ModelEvent, UiEvent};
use crate::constants::protocol::{self, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT, IPMSG_DEFAULT_PORT, IPMSG_FILEATTACHOPT, IPMSG_LIMITED_BROADCAST, IPMSG_SENDMSG};
use crate::core::{GLOBLE_RECEIVER, GLOBLE_SENDER};
use crate::util::packet_parser;

pub fn model_run(socket: UdpSocket, ui_event_sender: Sender<UiEvent>) {

    let file_pool: Arc<Mutex<Vec<ShareInfo>>> = Arc::new(Mutex::new(Vec::new()));

    let file_server = FileServer::new(file_pool.clone());

    file_server.run();

    let download_pool: Arc<Mutex<HashMap<u32, PoolFile>>> = Arc::new(Mutex::new(HashMap::new()));

    let manager_pool = ManagerPool::new(download_pool);

    model_event_loop(socket.try_clone().unwrap(), ui_event_sender, file_server, manager_pool);

    start_daemon(socket.try_clone().unwrap());

    send_ipmsg_br_entry();
}

pub fn send_ipmsg_br_entry() {
    let packet = Packet::new(IPMSG_BR_ENTRY|IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *protocol::HOST_NAME, *protocol::HOST_NAME)));
    GLOBLE_SENDER.send(ModelEvent::BroadcastEntry(packet)).unwrap();
}

pub fn start_daemon(socket: UdpSocket){
    let socket_clone = socket.try_clone().unwrap();
    thread::spawn(move||{
        loop {
            let mut buf = [0; 2048];
            match socket_clone.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    //todo 默认是用中文编码 可配置化
                    let receive_str = GB18030.decode(&buf[0..amt], DecoderTrap::Strict).unwrap();
                    info!("receive raw message -> {:?} from ip -> {:?}", receive_str, src.ip());
                    let result = packet_parser().parse(receive_str.as_str());
                    match result {
                        Ok((mut packet, _)) => {
                            packet.ip = src.ip().to_string();
                            GLOBLE_SENDER.send(ModelEvent::ReceivedPacket {packet}).unwrap();
                        }
                        Err(_) => {
                            error!("Invalid packet {} !", receive_str);
                        }
                    }
                },
                Err(e) => {
                    error!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    });
}

fn model_event_loop(socket: UdpSocket, ui_event_sender: Sender<UiEvent>, file_server: FileServer, manager_pool: ManagerPool) {
    let socket_clone = socket.try_clone().unwrap();
    thread::spawn(move || {
        while let Ok(ev) = GLOBLE_RECEIVER.recv() {
            match ev {
                ModelEvent::ReceivedPacket {packet} => {
                    model_packet_dispatcher(packet);
                }
                ModelEvent::UserListSelected(text) => {
                    ui_event_sender.try_send(UiEvent::UpdateUserListFooterStatus(text)).unwrap();
                }
                ModelEvent::UserListDoubleClicked {name, ip} => {
                    ui_event_sender.try_send(UiEvent::OpenOrReOpenChatWindow {name, ip}).unwrap();
                }
                ModelEvent::BroadcastEntry(packet) => {
                    socket_clone.set_broadcast(true).unwrap();
                    let addr:String = format!("{}:{}", IPMSG_LIMITED_BROADCAST, IPMSG_DEFAULT_PORT);
                    socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                    info!("send BroadcastEntry !");
                }
                ModelEvent::RecMsgReply{packet, from_ip} => {
                    let addr:String = format!("{}:{}", from_ip, protocol::IPMSG_DEFAULT_PORT);
                    socket_clone.set_broadcast(false).unwrap();
                    socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                    info!("send RecMsgReply !");
                }
                ModelEvent::BroadcastExit(ip) => {
                    ui_event_sender.try_send(UiEvent::UserListRemoveOne(ip)).unwrap();
                }
                ModelEvent::RecOnlineMsgReply {packet, from_user} => {
                    {
                        let addr:String = format!("{}:{}", from_user.ip, protocol::IPMSG_DEFAULT_PORT);
                        socket_clone.set_broadcast(false).unwrap();
                        socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                        info!("send RecOnlineMsgReply ! {packet:?}");
                    }
                    ui_event_sender.try_send(UiEvent::UserListAddOne(from_user)).unwrap();
                }
                ModelEvent::ClickChatWindowCloseBtn{from_ip} => {
                    ui_event_sender.try_send(UiEvent::CloseChatWindow(from_ip)).unwrap();
                }
                ModelEvent::NotifyOnline {user} => {
                    ui_event_sender.try_send(UiEvent::UserListAddOne(user)).unwrap();
                }
                ModelEvent::ReceivedMsg {msg} => {
                    let name = msg.clone().packet.unwrap().sender_name;
                    let ip = msg.clone().ip.clone();
                    ui_event_sender.try_send(UiEvent::OpenOrReOpenChatWindow1 { name: name.clone(), ip: ip.clone(), packet: msg.clone().packet}).unwrap();
                    let additional_section =  msg.clone().packet.unwrap().additional_section.unwrap();
                    let v: Vec<&str> = additional_section.split('\0').into_iter().collect();
                    ui_event_sender.try_send(UiEvent::DisplayReceivedMsgInHis{
                        from_ip: ip.clone(),
                        name: name.clone(),
                        context: v[0].to_owned(),
                        files: msg.opt_files.unwrap_or(vec![])
                    }).unwrap();
                }
                ModelEvent::SendOneMsg {to_ip, packet, context, files} => {
                    let addr:String = format!("{}:{}", to_ip, protocol::IPMSG_DEFAULT_PORT);
                    socket_clone.set_broadcast(false).unwrap();
                    socket_clone.send_to(crate::util::utf8_to_gb18030(packet.to_string().as_ref()).as_slice(), addr.as_str()).expect("couldn't send message");
                    info!("send SendOneMsg !");
                    ui_event_sender.try_send(UiEvent::DisplaySelfSendMsgInHis {to_ip, context, files: files.clone()}).unwrap();
                    {
                        let mut file_pool = file_server.file_pool.lock().unwrap();
                        if let Some(file) = files {
                            file_pool.push(file);
                        }

                    }
                }
                ModelEvent::DownloadIsBusy{ file } => {
                    info!("{} is downloading!!!", file.name);
                }
                ModelEvent::PutDownloadTaskInPool {file, save_base_path, download_ip} => {
                    manager_pool.clone().run(file, save_base_path, download_ip);
                }
                ModelEvent::RemoveDownloadTaskInPool {packet_id, file_id, download_ip } => {
                    ui_event_sender.try_send(UiEvent::RemoveInReceivedList{
                        packet_id,
                        file_id,
                        download_ip
                    }).unwrap();
                }
                _ => {
                    println!("{}", "aa");
                }
            }
        }
    });

}

fn model_packet_dispatcher(packet: Packet) {
    let mut extstr = String::new();
    if let Some(ref additional_section) = (&packet).additional_section {
        extstr = additional_section.to_owned();
    }
    let opt = protocol::get_opt((&packet).command_no);
    let cmd = protocol::get_mode((&packet).command_no);
    if opt& protocol::IPMSG_SENDCHECKOPT != 0 {
        let recvmsg = Packet::new(protocol::IPMSG_RECVMSG, Some((&packet).packet_no.to_string()));
        GLOBLE_SENDER.send(ModelEvent::RecMsgReply{packet: recvmsg, from_ip: (&packet).ip.to_owned()});
    }
    if cmd == protocol::IPMSG_BR_EXIT {//收到下线通知消息
        GLOBLE_SENDER.send(ModelEvent::BroadcastExit((&packet).sender_host.to_owned()));
    }else if cmd == protocol::IPMSG_BR_ENTRY {//收到上线通知消息
        ///扩展段 用户名|用户组
        let ext_vec = extstr.splitn(2, |c| c == ':').collect::<Vec<&str>>();
        let ansentry_packet = Packet::new(protocol::IPMSG_ANSENTRY, None);

        let group_name = if ext_vec.len() > 2 {
            ext_vec[1].to_owned()
        }else {
            "".to_owned()
        };
        let user_name = if ext_vec.len() > 1&& !ext_vec[0].is_empty() {
            ext_vec[0].to_owned()
        }else {
            packet.sender_name.clone()
        };

        let user = User::new(user_name, (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), group_name);
        info!("{user:?}");
        GLOBLE_SENDER.send(ModelEvent::RecOnlineMsgReply{packet: ansentry_packet, from_user: user});
    }else if cmd == protocol::IPMSG_ANSENTRY {//通报新上线
        let user = User::new((&packet).sender_name.to_owned(), (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), "".to_owned());
        GLOBLE_SENDER.send(ModelEvent::NotifyOnline{user});
    }else if cmd == protocol::IPMSG_SENDMSG {//收到发送的消息
        //文字消息|文件扩展段
        let ext_vec = extstr.split('\0').collect::<Vec<&str>>();
        if opt& protocol::IPMSG_SECRETOPT != 0 {//是否是密封消息
            info!("i am secret message !");
        }
        let msg_str = if ext_vec.len() > 0 { ext_vec[0] } else { "" };
        //文字消息内容|文件扩展
        let mut files_opt: Option<Vec<ReceivedSimpleFileInfo>> = None;
        if opt& protocol::IPMSG_FILEATTACHOPT != 0 {
            if ext_vec.len() > 1 {
                let files_str: &str = ext_vec[1];
                info!("i have file attachment {:?}", files_str);
                let files = files_str.split(protocol::FILELIST_SEPARATOR).into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                let mut simple_file_infos = Vec::new();
                for file_str in files {
                    let file_attr = file_str.splitn(6, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
                    if file_attr.len() >= 5 {
                        let file_id = file_attr[0].parse::<u32>().unwrap();
                        let file_name = file_attr[1];
                        let size = u64::from_str_radix(file_attr[2], 16).unwrap();//大小
                        let mmtime = file_attr[3];//修改时间
                        let mut mmtime_num = i64::from_str_radix(mmtime, 16).unwrap();//时间戳
                        if mmtime_num >= 10000000000 {
                            mmtime_num = (mmtime_num as i64)/1000;
                        }
                        let file_attr = file_attr[4].parse::<u32>().unwrap();//文件属性
                        let ntime = NaiveDateTime::from_timestamp(mmtime_num, 0);
                        if file_attr == protocol::IPMSG_FILE_REGULAR {
                            info!("i am ipmsg_file_regular");
                        }else if file_attr == protocol::IPMSG_FILE_DIR {
                            info!("i am ipmsg_file_dir");
                        }else {
                            panic!("no no type")
                        }
                        let simple_file_info = ReceivedSimpleFileInfo {
                            file_id,
                            packet_id: (&packet).packet_no.parse::<u32>().unwrap(),
                            name: file_name.to_owned(),
                            attr: file_attr as u8,
                            size,
                            mtime: mmtime_num
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
        GLOBLE_SENDER.send(ModelEvent::ReceivedMsg {msg: received_packet_inner}).unwrap();
    }else if cmd == protocol::IPMSG_NOOPERATION {
        info!("i am IPMSG_NOOPERATION");
    }else if cmd == protocol::IPMSG_BR_ABSENCE {
        info!("i am IPMSG_BR_ABSENCE");
    }else {

    }
}
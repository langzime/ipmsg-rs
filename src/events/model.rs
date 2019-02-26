use std::thread;
use std::net::UdpSocket;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::prelude::*;
use crate::model::{Packet, User, ReceivedSimpleFileInfo, ReceivedPacketInner, FileInfo};
use crate::events::ui::UiEvent;
use crate::constant::{self, IPMSG_SENDMSG, IPMSG_FILEATTACHOPT, IPMSG_DEFAULT_PORT, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT, IPMSG_LIMITED_BROADCAST};

pub enum ModelEvent {
    //Quit,
    //SearchFor(String),
    UserListSelected(String),
    UserListDoubleClicked{
        name: String,
        ip:String
    },
    ReceivedPacket{
        packet: Packet
    },
    BroadcastEntry(Packet),
    RecMsgReply{ packet: Packet, from_ip: String},
    BroadcastExit(String),
    RecOnlineMsgReply{ packet: Packet, from_user: User},
    ClickChatWindowCloseBtn{from_ip: String},
    NotifyOnline{ user: User},
    ReceivedMsg{msg: ReceivedPacketInner},
    SendOneMsg {to_ip: String, packet: Packet, context: String, files: Vec<FileInfo>},
    PutInTcpFilePool(),
}

pub fn model_run(socket: UdpSocket, receiver: crossbeam_channel::Receiver<ModelEvent>, model_event_sender: crossbeam_channel::Sender<ModelEvent>, ui_event_sender: glib::Sender<UiEvent>) {

    start_daemon(socket.try_clone().unwrap(), model_event_sender.clone());

    model_event_loop(socket.try_clone().unwrap(), receiver, model_event_sender.clone(), ui_event_sender);

    send_ipmsg_br_entry(model_event_sender.clone());
}

pub fn send_ipmsg_br_entry(model_event_sender: crossbeam_channel::Sender<ModelEvent>) {
    let packet = Packet::new(IPMSG_BR_ENTRY|IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::hostname, *constant::hostname)));
    model_event_sender.send(ModelEvent::BroadcastEntry(packet)).unwrap();
}

pub fn start_daemon(socket: UdpSocket, sender: crossbeam_channel::Sender<ModelEvent>){
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
                        //sender.send(packet);
                        sender.send(ModelEvent::ReceivedPacket {packet}).unwrap();
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

fn model_event_loop(socket: UdpSocket, receiver: crossbeam_channel::Receiver<ModelEvent>, model_event_sender: crossbeam_channel::Sender<ModelEvent>, ui_event_sender: glib::Sender<UiEvent>) {
    let socket_clone = socket.try_clone().unwrap();
    thread::spawn(move || {
        while let Ok(ev) = receiver.recv() {
            match ev {
                ModelEvent::UserListSelected(text) => {
                    ui_event_sender.send(UiEvent::UpdateUserListFooterStatus(text)).unwrap();
                }
                ModelEvent::UserListDoubleClicked {name, ip} => {
                    ui_event_sender.send(UiEvent::OpenOrReOpenChatWindow {name, ip}).unwrap();
                }
                ModelEvent::ReceivedPacket {packet} => {
                    model_packet_dispatcher(packet, model_event_sender.clone());
                }
                ModelEvent::BroadcastEntry(packet) => {
                    socket_clone.set_broadcast(true).unwrap();
                    let addr:String = format!("{}:{}", IPMSG_LIMITED_BROADCAST, IPMSG_DEFAULT_PORT);
                    socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str());
                }
                ModelEvent::RecMsgReply{packet, from_ip} => {
                    let addr:String = format!("{}:{}", from_ip, constant::IPMSG_DEFAULT_PORT);
                    socket_clone.set_broadcast(false).unwrap();
                    socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                }
                ModelEvent::BroadcastExit(ip) => {
                    ui_event_sender.send(UiEvent::UserListRemoveOne(ip)).unwrap();
                }
                ModelEvent::RecOnlineMsgReply {packet, from_user} => {
                    {
                        let addr:String = format!("{}:{}", from_user.ip, constant::IPMSG_DEFAULT_PORT);
                        socket_clone.set_broadcast(false).unwrap();
                        socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                    }
                    ui_event_sender.send(UiEvent::UserListAddOne(from_user)).unwrap();
                }
                ModelEvent::ClickChatWindowCloseBtn{from_ip} => {
                    ui_event_sender.send(UiEvent::CloseChatWindow(from_ip)).unwrap();
                }
                ModelEvent::NotifyOnline {user} => {
                    ui_event_sender.send(UiEvent::UserListAddOne(user)).unwrap();
                }
                ModelEvent::ReceivedMsg {msg} => {
                    let name = msg.clone().packet.unwrap().sender_name;
                    let ip = msg.clone().ip.clone();
                    ui_event_sender.send(UiEvent::OpenOrReOpenChatWindow1 { name: name.clone(), ip: ip.clone(), packet: msg.clone().packet}).unwrap();
                    let additional_section =  msg.clone().packet.unwrap().additional_section.unwrap();
                    let v: Vec<&str> = additional_section.split('\0').into_iter().collect();
                    ui_event_sender.send(UiEvent::DisplayReceivedMsgInHis{
                        from_ip: ip.clone(),
                        name: name.clone(),
                        context: v[0].to_owned(),
                        files: msg.opt_files.unwrap_or(vec![])
                    }).unwrap();
                }
                ModelEvent::SendOneMsg {to_ip, packet, context, files} => {
                    let addr:String = format!("{}:{}", to_ip, constant::IPMSG_DEFAULT_PORT);
                    socket_clone.set_broadcast(false).unwrap();
                    socket_clone.send_to(crate::util::utf8_to_gb18030(packet.to_string().as_ref()).as_slice(), addr.as_str()).expect("couldn't send message");
                    ui_event_sender.send(UiEvent::DisplaySelfSendMsgInHis {to_ip, context, files}).unwrap();
                }
                _ => {
                    println!("{}", "aa");
                }
            }
        }
    });

}

fn model_packet_dispatcher(packet: Packet, model_event_sender: crossbeam_channel::Sender<ModelEvent>) {
    let mut extstr = String::new();
    if let Some(ref additional_section) = (&packet).additional_section {
        extstr = additional_section.to_owned();
    }
    let opt = constant::get_opt((&packet).command_no);
    let cmd = constant::get_mode((&packet).command_no);
    info!("{:?}", &packet);
    info!("{:x}", &packet.packet_no.parse::<i32>().unwrap());
    info!("cmd {:x} opt {:x} opt section {:?}", cmd, opt, extstr);
    if opt&constant::IPMSG_SENDCHECKOPT != 0 {
        let recvmsg = Packet::new(constant::IPMSG_RECVMSG, Some((&packet).packet_no.to_string()));
        model_event_sender.send(ModelEvent::RecMsgReply{packet: recvmsg, from_ip: (&packet).ip.to_owned()});
    }
    if cmd == constant::IPMSG_BR_EXIT {//收到下线通知消息
        model_event_sender.send(ModelEvent::BroadcastExit((&packet).sender_host.to_owned()));
    }else if cmd == constant::IPMSG_BR_ENTRY {//收到上线通知消息
        ///扩展段 用户名|用户组
        let ext_vec = extstr.splitn(2, |c| c == ':').collect::<Vec<&str>>();
        let ansentry_packet = Packet::new(constant::IPMSG_ANSENTRY, None);

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

        model_event_sender.send(ModelEvent::RecOnlineMsgReply{packet: ansentry_packet, from_user: user});
    }else if cmd == constant::IPMSG_ANSENTRY {//通报新上线
        let user = User::new((&packet).sender_name.to_owned(), (&packet).sender_host.to_owned(), (&packet).ip.to_owned(), "".to_owned());
        model_event_sender.send(ModelEvent::NotifyOnline{user});
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
        model_event_sender.send(ModelEvent::ReceivedMsg {msg: received_packet_inner}).unwrap();
        //remained_sender.send(received_packet_inner);
        //::glib::idle_add(create_or_open_chat);
    }else if cmd == constant::IPMSG_NOOPERATION {
        info!("i am IPMSG_NOOPERATION");
    }else if cmd == constant::IPMSG_BR_ABSENCE {
        info!("i am IPMSG_BR_ABSENCE");
    }else {

    }
}
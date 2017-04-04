
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;
use std::sync::mpsc;

use gtk::ListStore;
use constant;
use model;
use model::{User, OperUser, Operate};
use chrono::prelude::*;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use std::cell::RefCell;
use gtk::TreeModelExt;

///启动发送上线消息
pub fn send_ipmsg_br_entry(socket: UdpSocket){
    thread::spawn(move||{
        let packet = Packet::new(constant::IPMSG_BR_ENTRY|constant::IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", "dujiajiyi", "user")));
        socket.set_broadcast(true).unwrap();
        let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
        socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
    });
}

///启动消息监听线程
pub fn start_daemon(socket: UdpSocket, sender: mpsc::Sender<Packet>){
    thread::spawn(move||{
        loop {
            let mut buf = [0; 2048];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    //let receive_str = unsafe { str::from_utf8_unchecked(&buf[0..amt])};
                    //todo 默认是用中文编码 还没想到怎么做兼容
                    let receive_str = GB18030.decode(&buf[0..amt], DecoderTrap::Strict).unwrap();
                    println!("收到原始信息 -> {} 来自 ip -> {}", receive_str, src.ip());
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
                    println!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    });
}

///信息处理
pub fn start_message_processer(socket: UdpSocket, receiver :mpsc::Receiver<Packet>, sender :mpsc::Sender<OperUser>){
    thread::spawn(move || {
        loop {
            let packet = receiver.recv().unwrap();
            let opt = constant::get_opt(packet.command_no);
            let cmd = constant::get_mode(packet.command_no);
            let extstr: String = packet.additional_section.unwrap();
            let ext_vec: Vec<&str> = extstr.split('\0').into_iter().filter(|x: &&str| !x.is_empty()).collect();
            println!("我是扩展消息 {:?}", ext_vec);
            println!("命令位 {:x} 扩展位{:x}", cmd, opt);
            let addr:String = format!("{}:{}", packet.ip, constant::IPMSG_DEFAULT_PORT);
            if opt&constant::IPMSG_SENDCHECKOPT != 0 {
                let recvmsg = Packet::new(constant::IPMSG_RECVMSG, Some(packet.packet_no.to_string()));
                socket.send_to(recvmsg.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
            }
            if cmd == constant::IPMSG_BR_ENTRY {//收到上线通知消息
                let ansentry_packet = Packet::new(constant::IPMSG_ANSENTRY, None);
                socket.set_broadcast(false).unwrap();
                socket.send_to(ansentry_packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
                let user = User::new(packet.sender_name, packet.sender_host, packet.ip, "".to_owned());
                sender.send(OperUser::new(user, Operate::ADD));
                ::glib::idle_add(receive);
            }else if cmd == constant::IPMSG_SENDMSG {//收到发送的消息
                if opt&constant::IPMSG_SECRETOPT != 0 {//是否是密封消息
                    println!("我是密封消息");
                } else {
                    //let extstr: String = packet.additional_section.unwrap();
                    //let v: Vec<&str> = extstr.split('\0').into_iter().filter(|x: &&str| !x.is_empty()).collect();
                    //println!("我是明文消息 {:?}", v);println!("我是明文消息 {:?}", v);
                }
            }else {

            }
        }
    });
}

fn receive() -> ::glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref store, ref rx)) = *global.borrow() {
            if let Ok(op_user) = rx.try_recv() {
//                let new_iter = &store.append();
//                store.set(new_iter, &[0, 1, 2], &[&&op_user.user.name, &&op_user.user.group, &&op_user.user.host]);
//                println!("{:?}", store.get_string_from_iter(new_iter));
//                let iter = store.insert_with_values(None, &[0, 1, 2, 3], &[&&op_user.user.name, &&op_user.user.group, &&op_user.user.host, &&op_user.user.ip]);
//                println!("{:?}", store.get_string_from_iter(&iter));
//                store.remove(&new_iter);
//                println!("{:?}", store.get_value(&store.get_iter_from_string("0").unwrap(), 3).get::<String>().unwrap());

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

thread_local!(
    pub static GLOBAL: RefCell<Option<(::gtk::ListStore, mpsc::Receiver<OperUser>)>> = RefCell::new(None)
);
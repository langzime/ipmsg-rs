
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;
use std::sync::mpsc;

use gtk::ListStore;
use constant;
use chrono::prelude::*;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use std::cell::RefCell;

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
pub fn start_message_processer(socket: UdpSocket, receiver :mpsc::Receiver<Packet>, sender :mpsc::Sender<String>){
    thread::spawn(move || {
        loop {
            let packet = receiver.recv().unwrap();
            let opt = constant::get_opt(packet.command_no);
            let cmd = constant::get_mode(packet.command_no);
            let extstr: String = packet.additional_section.unwrap();
            let ext_vec: Vec<&str> = extstr.split('\0').into_iter().filter(|x: &&str| !x.is_empty()).collect();
            println!("我是明文消息 {:?}", ext_vec);
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
                sender.send("111".to_owned());
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
            if let Ok(str) = rx.try_recv() {
                store.insert_with_values(None, &[0, 1, 2], &[&9, &"111", &"2222"]);
            }
        }
    });
    ::glib::Continue(false)
}

thread_local!(
    pub static GLOBAL: RefCell<Option<(::gtk::ListStore, mpsc::Receiver<String>)>> = RefCell::new(None)
);
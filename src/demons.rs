
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;
use std::sync::mpsc;

use constant;
use chrono::prelude::*;

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
                    let receive_str = unsafe { str::from_utf8_unchecked(&buf[0..amt])};
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
pub fn start_message_processer(socket: UdpSocket, receiver :mpsc::Receiver<Packet>){
    thread::spawn(move || {
        loop {
            let packet = receiver.recv().unwrap();
            let opt = constant::get_opt(packet.command_no);
            let cmd = constant::get_mode(packet.command_no);
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
            }else if cmd == constant::IPMSG_SENDMSG {//收到发送的消息

            }else {

            }
        }
    });
}
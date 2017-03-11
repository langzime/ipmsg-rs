use std::str;
use model::Packet;
use std::net::UdpSocket;

use constant;
use chrono::prelude::*;

///发送消息
pub fn send(tar_ip: &str, packet: Packet) {
    let socket:UdpSocket = UdpSocket::bind("0.0.0.0:34301").unwrap();
    let cmd = constant::get_mode(packet.command_no);
    let local_ip = constant::get_local_ip().unwrap().to_string();
    println!("{} {}", local_ip, tar_ip);
    if local_ip != tar_ip {
        if cmd == constant::IPMSG_BR_ENTRY {
            println!("发送上线通知消息");
            socket.set_broadcast(true).unwrap();
            socket.broadcast().unwrap();
            let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
            socket.connect(addr.as_str()).unwrap();
            socket.send(packet.to_string().as_bytes()).expect("couldn't send message");
        }else if cmd == constant::IPMSG_ANSENTRY {
            println!("发送上线应答信息");
            let addr:String = format!("{}:{}", tar_ip, constant::IPMSG_DEFAULT_PORT);
            socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
        }else {

        }
    }
}

///接收消息
pub fn recevice(src_ip: &str, packet: Packet) {
    println!("{:?}", packet);
    println!("命令位 {:x} 扩展位{:x}", constant::get_mode(packet.command_no), constant::get_opt(packet.command_no));
    if constant::get_mode(packet.command_no) == constant::IPMSG_BR_ENTRY {
        println!("收到上线通知消息{}", src_ip);
        let ansentry_packet = Packet::new(constant::IPMSG_ANSENTRY, None);
        send(src_ip, ansentry_packet);
    }
}
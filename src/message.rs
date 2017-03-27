use std::str;
use model::Packet;
use std::net::UdpSocket;

use constant;
use chrono::prelude::*;

///发送消息
pub fn send(socket: UdpSocket, tar_ip: &str, packet: Packet) {
    let cmd = constant::get_mode(packet.command_no);
    let local_ip = constant::get_local_ip().unwrap().to_string();
    println!("{} {}", local_ip, tar_ip);
    if local_ip != tar_ip {
        if cmd == constant::IPMSG_BR_ENTRY {
            println!("{:x}发送上线通知消息", cmd);
            socket.set_broadcast(true).unwrap();
            let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
            socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
        }else if cmd == constant::IPMSG_ANSENTRY {
            println!("{:x}发送上线应答信息", cmd);
            socket.set_broadcast(false).unwrap();
            let addr:String = format!("{}:{}", tar_ip, constant::IPMSG_DEFAULT_PORT);
            socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
        }else {

        }
    }
}

///接收消息
pub fn recevice(socket: UdpSocket, src_ip: &str, packet: Packet) {
    println!("命令位 {:x} 扩展位{:x}", constant::get_mode(packet.command_no), constant::get_opt(packet.command_no));
    if constant::get_mode(packet.command_no) == constant::IPMSG_BR_ENTRY {
        println!("收到上线通知消息{}", src_ip);
        let ansentry_packet = Packet::new(constant::IPMSG_ANSENTRY, None);
        let sock_clone = socket.try_clone().unwrap();
        send(sock_clone, src_ip, ansentry_packet);
    }
}
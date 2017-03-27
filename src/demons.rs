
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;

use constant;
use message;
use chrono::prelude::*;

///启动发送上线消息
pub fn send_ipmsg_br_entry(socket: UdpSocket){
    thread::spawn(move||{
        let packet = Packet::new(constant::IPMSG_BR_ENTRY|constant::IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", "dujiajiyi", "user")));
        message::send(socket, "", packet);
        println!("send_ipmsg_br_entry！！");
    });
}

///启动消息监听线程
pub fn start_demon(socket: UdpSocket){
    thread::spawn(move||{
        loop {
            let mut buf = [0; 2048];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("-------收到消息----------start------------");
                    let receive_str = unsafe { str::from_utf8_unchecked(&buf[0..amt])};
                    println!("{}", receive_str);
                    println!("req ip {}", src.ip());

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
                        let sock_clone = socket.try_clone().unwrap();
                        message::recevice(sock_clone, &src.ip().to_string(), packet);
                    }else {
                        println!("Invalid packet {} !", receive_str);
                    }
                    println!("-------收到消息-----------end-----------\n");
                },
                Err(e) => {
                    println!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    });
}
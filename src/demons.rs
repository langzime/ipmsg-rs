
use std::str;
use std::thread;
use model::Packet;
use std::net::UdpSocket;

use constant;
use message;
use chrono::prelude::*;

///启动发送上线消息
pub fn send_ipmsg_br_entry(){
    let packet = Packet::new(constant::IPMSG_BR_ENTRY|constant::IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", "dujiajiyi", "user")));
    message::send("", packet);
}

///启动消息监听线程
pub fn start_demon(){
    thread::spawn(move||{
        loop {
            let addr: String = format!("{}{}", "0.0.0.0:", constant::IPMSG_DEFAULT_PORT);

            let socket: UdpSocket = match UdpSocket::bind(addr.as_str()) {
                Ok(s) => {
                    println!("{:?} 开启端口监听", s);
                    s
                },
                Err(e) => panic!("couldn't bind socket: {}", e)
            };

            let mut buf = [0; 2048];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        let receive_str = str::from_utf8(&buf[0..amt]);
                        println!("{}", receive_str.unwrap());
                        println!("req ip {}", src.ip());

                        let v: Vec<&str> = receive_str.unwrap().splitn(6, |c| c == ':').collect();
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

                            message::recevice(&src.ip().to_string(), packet);
                        }else {
                            println!("Invalid packet {} !", receive_str.unwrap());
                        }
                    },
                    Err(e) => {
                        println!("couldn't recieve a datagram: {}", e);
                    }
                }
            }
        }
    });
}
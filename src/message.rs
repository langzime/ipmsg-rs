use std::thread;
use std::net::UdpSocket;
use model::Packet;
use constant;

///启动发送上线消息
pub fn send_ipmsg_br_entry(socket: UdpSocket){
    thread::spawn(move||{
        let packet = Packet::new(constant::IPMSG_BR_ENTRY|constant::IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::homename, *constant::homename)));
        socket.set_broadcast(true).unwrap();
        let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
        socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
    });
}

///启动发送上线消息
pub fn send_ipmsg(socket: UdpSocket){
    thread::spawn(move||{
        let packet = Packet::new(constant::IPMSG_BR_ENTRY|constant::IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::homename, *constant::homename)));
        socket.set_broadcast(true).unwrap();
        let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
        socket.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
    });
}


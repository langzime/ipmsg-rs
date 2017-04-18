use std::thread;
use std::net::UdpSocket;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use model::{self, Packet};
use constant::{self, IPMSG_SENDMSG, IPMSG_FILEATTACHOPT, IPMSG_DEFAULT_PORT, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT};

///启动发送上线消息
pub fn send_ipmsg_br_entry(){
    ::demons::GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
            thread::spawn(move||{
                let packet = Packet::new(IPMSG_BR_ENTRY|IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::homename, *constant::homename)));
                socket_clone.set_broadcast(true).unwrap();
                let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
                socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str()).expect("couldn't send message");
            });
        }
    });
}

///发送消息
pub fn send_ipmsg(context :String, files: Vec<model::FileInfo>, tar_ip: String){
    //IPMSG_SENDMSG|IPMSG_FILEATTACHOPT
    ::demons::GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
            thread::spawn(move||{
                let packet = Packet::new(IPMSG_SENDMSG|IPMSG_FILEATTACHOPT, Some("\u{0}248235534:chrome.VisualElementsManifest.xml:191:58eb5e02:1:\u{7}".to_owned()));
                let addr:String = format!("{}:{}", tar_ip, IPMSG_DEFAULT_PORT);
                socket_clone.send_to(::util::utf8_to_gb18030(packet.to_string().as_ref()).as_slice(), addr.as_str()).expect("couldn't send message");
            });
        }
    });
}


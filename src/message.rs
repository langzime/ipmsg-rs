use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::net::UdpSocket;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::prelude::*;
use model::{self, Packet};
use constant::{self, IPMSG_SENDMSG, IPMSG_FILEATTACHOPT, IPMSG_DEFAULT_PORT, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT};
use app::{self, GLOBAL_UDPSOCKET, GLOBAL_SHARELIST, GLOBAL_WINDOWS, GLOBAL_USERLIST};

///启动发送上线消息
pub fn send_ipmsg_br_entry() {
    GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
            thread::spawn(move||{
                let packet = Packet::new(IPMSG_BR_ENTRY|IPMSG_BROADCASTOPT, Some(format!("{}\0\n{}", *constant::hostname, *constant::hostname)));
                socket_clone.set_broadcast(true).unwrap();
                let addr:String = format!("{}:{}", constant::IPMSG_LIMITED_BROADCAST, constant::IPMSG_DEFAULT_PORT);
                socket_clone.send_to(packet.to_string().as_bytes(), addr.as_str());
            });
        }
    });
}

///发送消息
pub fn send_ipmsg(context :String, files: Arc<RefCell<Vec<model::FileInfo>>>, tar_ip: String){
    let files_clone = files.clone();
    let files = files_clone.borrow().to_vec();
    GLOBAL_UDPSOCKET.with(|global| {
        if let Some(ref socket) = *global.borrow() {
            let socket_clone = socket.try_clone().unwrap();
            let commond = if (&files).len() > 0 { IPMSG_SENDMSG|IPMSG_FILEATTACHOPT } else { IPMSG_SENDMSG };//如果有文件，需要扩展文件

            let share_info = if (&files).len() > 0 {
                Some(model::ShareInfo {
                    packet_no: Local::now().timestamp() as u32,
                    host: tar_ip.clone(),
                    host_cnt: 1,
                    file_info: files.clone(),
                    file_cnt: 1,
                    attach_time: Local::now().time(),
                })
            }else {
                None
            };

            let mut additional = String::new();
            for (i, file) in (&files).iter().enumerate() {
                additional.push_str(file.to_fileinfo_msg().as_str());
                additional.push('\u{7}');
            }
            if let Some(share_info) = share_info {
                GLOBAL_SHARELIST.with(|global_shares| {
                    if let Some(ref share_infos_arc) = *global_shares.borrow() {
                        let mut share_infos = share_infos_arc.lock().unwrap();
                        (*share_infos).push(share_info);
                    }
                });
            }
            let mut context1: String = context.to_owned();
            context1.push_str(context.as_str());
            context1.push('\u{0}');
            context1.push_str(additional.as_str());
            context1.push('\u{0}');
            let packet = Packet::new(commond, Some(context1));
            info!("send message {:?}", packet);
            thread::spawn(move||{
                let addr:String = format!("{}:{}", tar_ip, IPMSG_DEFAULT_PORT);
                socket_clone.send_to(::util::utf8_to_gb18030(packet.to_string().as_ref()).as_slice(), addr.as_str());
            });
        }
    });
}


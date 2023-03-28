use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::thread;
use std::net::UdpSocket;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::prelude::*;
use log::{info, trace, warn};
use crate::model::{self, Packet};
use crate::constant::{self, IPMSG_SENDMSG, IPMSG_FILEATTACHOPT, IPMSG_DEFAULT_PORT, IPMSG_BR_ENTRY, IPMSG_BROADCASTOPT};

pub fn create_sendmsg(context :String, files: Vec<model::FileInfo>, tar_ip: String) -> (Packet, Option<model::ShareInfo>){
    let commond = if files.len() > 0 { IPMSG_SENDMSG|IPMSG_FILEATTACHOPT } else { IPMSG_SENDMSG };//如果有文件，需要扩展文件
    let share_info = if files.len() > 0 {
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
    for (i, file) in files.iter().enumerate() {
        additional.push_str(file.to_fileinfo_msg().as_str());
        additional.push('\u{7}');
    }
    let mut context1: String = context.to_owned();
    context1.push('\u{0}');
    context1.push_str(additional.as_str());
    context1.push('\u{0}');
    let packet = Packet::new(commond, Some(context1));
    info!("send message {:?}", packet);
    return (packet, share_info);
}

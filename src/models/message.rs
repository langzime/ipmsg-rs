use crate::constants::protocol::{IPMSG_FILEATTACHOPT, IPMSG_SENDMSG};
use crate::models::model::{self, Packet};
use time::OffsetDateTime;
use tracing::{info, instrument};

#[instrument]
pub fn create_sendmsg(context: String, file_opt: Option<model::FileInfo>, tar_ip: String) -> (Packet, Option<model::ShareInfo>) {
    let commond = if file_opt.is_some() {
        IPMSG_SENDMSG | IPMSG_FILEATTACHOPT
    } else {
        IPMSG_SENDMSG
    }; //如果有文件，需要扩展文件
    let share_info = if let Some(f) = file_opt.clone() {
        Some(model::ShareInfo {
            packet_no: OffsetDateTime::now_utc().unix_timestamp(),
            host: tar_ip.clone(),
            host_cnt: 1,
            file_info: vec![f.clone()],
            file_cnt: 1,
            attach_time: OffsetDateTime::now_utc(),
        })
    } else {
        None
    };

    let mut additional = String::new();
    if let Some(f) = file_opt {
        additional.push_str(f.to_fileinfo_msg().as_str());
        additional.push('\u{7}');
    }
    let mut context1: String = context.to_owned();
    context1.push('\u{0}');
    context1.push_str(additional.as_str());
    context1.push('\u{0}');
    let packet = Packet::new(commond, Some(context1));
    info!("send message {:?}", packet);
    (packet, share_info)
}

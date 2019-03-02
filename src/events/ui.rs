use crate::model::{User, ReceivedPacketInner, Packet, ReceivedSimpleFileInfo, FileInfo, ShareInfo};

pub enum UiEvent {
    UpdateUserListFooterStatus(String),//create_or_open_chat
    OpenOrReOpenChatWindow {
        name: String,
        ip: String
    },
    UserListRemoveOne(String),
    UserListAddOne(User),
    CloseChatWindow(String),
    OpenOrReOpenChatWindow1 { name: String, ip: String, packet: Option<Packet>},
    DisplaySelfSendMsgInHis{to_ip: String, context: String, files: Option<ShareInfo>},
    DisplayReceivedMsgInHis{from_ip: String, name: String, context: String, files: Vec<ReceivedSimpleFileInfo> },
    RemoveInReceivedList {packet_id: u32, file_id: u32, download_ip: String },
}
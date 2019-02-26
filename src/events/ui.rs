use crate::model::{User, ReceivedPacketInner, Packet, ReceivedSimpleFileInfo, FileInfo};

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
    DisplaySelfSendMsgInHis{to_ip: String, context: String, files: Vec<FileInfo>},
    DisplayReceivedMsgInHis{from_ip: String, name: String, context: String, files: Vec<ReceivedSimpleFileInfo> }
}
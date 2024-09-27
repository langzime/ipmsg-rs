use crate::models::model::{Packet, ReceivedPacketInner, ReceivedSimpleFileInfo, ShareInfo, User};
use std::path::PathBuf;

#[derive(Debug)]
pub enum UiEvent {
    UpdateUserListFooterStatus(String), //create_or_open_chat
    OpenOrReOpenChatWindow {
        name: String,
        ip: String,
    },
    UserListRemoveOne(String),
    UserListAddOne(User),
    CloseChatWindow(String),
    OpenOrReOpenChatWindow1 {
        name: String,
        ip: String,
        packet: Option<Packet>,
    },
    DisplaySelfSendMsgInHis {
        to_ip: String,
        context: String,
        files: Option<ShareInfo>,
    },
    //接受到消息
    DisplayReceivedMsgInHis {
        from_ip: String,
        name: String,
        context: String,
        files: Vec<ReceivedSimpleFileInfo>,
    },
    RemoveInReceivedList {
        packet_id: u32,
        file_id: u32,
        download_ip: String,
    },
    Quit,
}

pub enum ModelEvent {
    UserListSelected(String),
    UserListDoubleClicked {
        name: String,
        ip: String,
    },
    ReceivedPacket {
        packet: Packet,
    },
    BroadcastEntry(Packet),
    RecMsgReply {
        packet: Packet,
        from_ip: String,
    },
    BroadcastExit(String),
    RecOnlineMsgReply {
        packet: Packet,
        from_user: User,
    },
    ClickChatWindowCloseBtn {
        from_ip: String,
    },
    NotifyOnline {
        user: User,
    },
    ReceivedMsg {
        msg: ReceivedPacketInner,
    },
    SendOneMsg {
        to_ip: String,
        packet: Packet,
        context: String,
        files: Option<ShareInfo>,
    },
    DownloadIsBusy {
        file: ReceivedSimpleFileInfo,
    },
    PutDownloadTaskInPool {
        file: ReceivedSimpleFileInfo,
        save_base_path: PathBuf,
        download_ip: String,
    },
    RemoveDownloadTaskInPool {
        packet_id: u32,
        file_id: u32,
        download_ip: String,
    },
    PutInTcpFilePool(),
}

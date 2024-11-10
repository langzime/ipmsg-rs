use crate::constants::protocol::{self, IPMSG_VERSION};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use time::OffsetDateTime;

///
/// 数据包格式
#[derive(Clone, Debug)]
pub struct Packet {
    ///版本好 标准协议为1
    pub ver: String,
    ///数据包编号
    pub packet_no: String,
    ///发送者的昵称
    pub sender_name: String,
    ///发送者的主机名
    pub sender_host: String,
    ///命令字
    pub command_no: u32,
    ///附加数据
    pub additional_section: Option<String>,
    ///发送者ip
    pub ip: String,
}

type ExtStr = String;

trait ExtMsg {
    fn to_ext_msg() -> ExtStr;
}

#[derive(Default)]
pub struct PacketBuilder {
    ///版本好 标准协议为1
    pub ver: String,
    ///数据包编号
    pub packet_no: String,
    ///发送者的昵称
    pub sender_name: String,
    ///发送者的主机名
    pub sender_host: String,
    ///命令字
    pub command_no: u32,
    ///扩展命令
    pub ext_commands: Vec<u32>,
}

impl PacketBuilder {
    ///命令
    fn command(command_no: u32) -> PacketBuilder {
        let mut packet_builder: PacketBuilder = Default::default();
        packet_builder.ver = format!("{}", IPMSG_VERSION);
        packet_builder.packet_no = format!("{}", OffsetDateTime::now_utc().unix_timestamp());
        packet_builder.sender_name = protocol::HOST_NAME.clone();
        packet_builder.sender_host = protocol::HOST_NAME.clone();
        packet_builder.command_no = command_no;
        packet_builder
    }
    ///扩展命令
    fn command_opt(mut self, ext_command_no: u32) -> PacketBuilder {
        self.ext_commands.push(ext_command_no);
        self
    }

    /*fn finish(&self) -> Packet {

    }*/
}

impl Packet {
    ///new packet
    pub fn new(command_no: u32, additional_section: Option<String>) -> Packet {
        let timestamp = OffsetDateTime::now_utc().unix_timestamp();
        Packet {
            ver: format!("{}", IPMSG_VERSION),
            packet_no: format!("{}", timestamp),
            sender_name: protocol::HOST_NAME.clone(),
            sender_host: protocol::get_local_ip().to_string(),
            command_no,
            additional_section,
            ip: "".to_owned(),
        }
    }

    /// from attrs 生成packet
    pub fn from<S>(ver: S, packet_no: S, sender_name: S, sender_host: S, command_no: u32, additional_section: Option<String>) -> Packet
    where
        S: Into<String>,
    {
        Packet {
            ver: ver.into(),
            packet_no: packet_no.into(),
            sender_name: sender_name.into(),
            sender_host: sender_host.into(),
            command_no,
            additional_section,
            ip: "".to_owned(),
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = if let Some(ref ext_str) = self.additional_section {
            format!(
                "{}:{}:{}:{}:{}:{}",
                self.ver, self.packet_no, self.sender_name, self.sender_host, self.command_no, ext_str
            )
        } else {
            format!(
                "{}:{}:{}:{}:{}:{}",
                self.ver, self.packet_no, self.sender_name, self.sender_host, self.command_no, ""
            )
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub host: String,
    pub ip: String,
    pub group: String,
}

impl User {
    pub fn new<S: Into<String>>(name: S, host: S, ip: S, group: S) -> User {
        User {
            name: name.into(),
            host: host.into(),
            ip: ip.into(),
            group: group.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operate {
    ADD,
    REMOVE,
}

#[derive(Clone, Debug)]
pub struct OperUser {
    pub user: User,
    pub oper: Operate,
}

impl OperUser {
    pub fn new(user: User, oper: Operate) -> OperUser {
        OperUser { user: user, oper: oper }
    }
}

#[derive(Clone, Debug)]
pub struct ShareInfo {
    //包编号
    pub packet_no: i64,
    // 要发送的目的机器列表
    pub host: String,
    // 要发送的目的机器个数
    pub host_cnt: u32,
    //transStat
    // 要传输的文件信息
    pub file_info: Vec<FileInfo>,
    // 要传输的文件个数
    pub file_cnt: u32,
    //文件添加时间
    pub attach_time: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct FileInfo {
    //要传输文件id
    pub file_id: i64,
    //文件名
    pub file_name: PathBuf,
    pub name: String,
    //文件的属性，如是文件或者文件夹，只读等
    pub attr: u8, // 1 普通文件 2 文件夹
    //文件大小
    pub size: u64,
    //文件最后一次修改时间
    pub mtime: OffsetDateTime,
    //文件最后一次访问时间
    pub atime: OffsetDateTime,
    //文件创建时间
    pub crtime: OffsetDateTime,
}

impl FileInfo {
    pub fn try_get<T: AsRef<Path>>(path: T) -> Result<FileInfo> {
        let name = path.as_ref().file_name().unwrap().to_str().unwrap();
        let metadata: Metadata = fs::metadata(path.as_ref())?;
        let size = metadata.len();
        let attr = if metadata.is_file() {
            protocol::IPMSG_FILE_REGULAR
        } else if metadata.is_dir() {
            protocol::IPMSG_FILE_DIR
        } else {
            return Err(anyhow!("选择文件失败，请选择文件或者文件夹"));
        };
        let file_info = FileInfo {
            file_id: OffsetDateTime::now_utc().unix_timestamp(),
            file_name: path.as_ref().to_path_buf().clone(),
            name: name.to_string().clone(),
            attr: attr as u8,
            size,
            mtime: OffsetDateTime::from(metadata.modified()?),
            atime: OffsetDateTime::from(metadata.accessed()?),
            crtime: OffsetDateTime::from(metadata.created()?),
        };
        Ok(file_info)
    }
    pub fn to_fileinfo_msg(&self) -> String {
        self.file_name
            .as_path()
            .file_name()
            .and_then(|name| name.to_str())
            .map(|file_name| {
                format!(
                    "{}:{}:{:x}:{:x}:{}:",
                    self.file_id,
                    file_name,
                    self.size,
                    self.mtime.unix_timestamp(),
                    self.attr
                )
            })
            .unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceivedSimpleFileInfo {
    //要传输文件id
    pub file_id: u32,
    pub packet_id: u32,
    pub name: String,
    pub attr: u8, // 1 普通文件 2 文件夹
    pub size: u64,
    pub mtime: i64,
}

#[derive(Clone, Debug)]
pub struct ReceivedPacketInner {
    ///发送者ip
    pub ip: String,
    ///原始packet
    pub packet: Option<Packet>,
    ///文件列表
    pub opt_files: Option<Vec<ReceivedSimpleFileInfo>>,
}

impl ReceivedPacketInner {
    pub fn new<S: Into<String>>(ip: S) -> ReceivedPacketInner {
        ReceivedPacketInner {
            ip: ip.into(),
            packet: None,
            opt_files: None,
        }
    }

    pub fn packet(mut self, packet: Packet) -> ReceivedPacketInner {
        self.packet = Some(packet);
        self
    }

    pub fn opt_files(mut self, opt_files: Vec<ReceivedSimpleFileInfo>) -> ReceivedPacketInner {
        self.opt_files = Some(opt_files);
        self
    }

    pub fn option_opt_files(mut self, opt_files: Option<Vec<ReceivedSimpleFileInfo>>) -> ReceivedPacketInner {
        self.opt_files = opt_files;
        self
    }
}

pub struct ErrMsg {
    pub msg: String,
    pub fatal: bool,
}

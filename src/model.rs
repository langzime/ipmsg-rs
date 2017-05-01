use std::net::TcpStream;
use chrono::prelude::*;
use constant::{self, IPMSG_VERSION};
use std::path::{PathBuf, Path};

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
        let local: DateTime<Local> = Local::now();
        let mut packet_builder: PacketBuilder = Default::default();
        packet_builder.ver = format!("{}", IPMSG_VERSION);
        packet_builder.packet_no = format!("{}", local.timestamp());
        packet_builder.sender_name = constant::hostname.clone();
        packet_builder.sender_host = constant::hostname.clone();
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
        let local: DateTime<Local> = Local::now();
        Packet {
            ver: format!("{}", IPMSG_VERSION),
            packet_no: format!("{}", local.timestamp()),
            sender_name: constant::hostname.clone(),
            sender_host: constant::hostname.clone(),
            command_no: command_no,
            additional_section: additional_section,
            ip: "".to_owned(),
        }
    }

    /// from attrs 生成packet
    pub fn from<S>(ver: S,
                packet_no: S,
                sender_name: S,
                sender_host: S,
                command_no: u32,
                additional_section: Option<String>) -> Packet where S: Into<String> {
        Packet {
            ver: ver.into(),
            packet_no: packet_no.into(),
            sender_name: sender_name.into(),
            sender_host: sender_host.into(),
            command_no: command_no,
            additional_section: additional_section,
            ip: "".to_owned(),
        }
    }
}

impl ToString for Packet {
    fn to_string(&self) -> String {
        if let Some(ref ext_str) = self.additional_section {
            format!("{}:{}:{}:{}:{}:{}",
                    self.ver,
                    self.packet_no,
                    self.sender_name,
                    self.sender_host,
                    self.command_no,
                    ext_str)
        }else {
            format!("{}:{}:{}:{}:{}:{}",
                    self.ver,
                    self.packet_no,
                    self.sender_name,
                    self.sender_host,
                    self.command_no,
                    "")
        }
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
    pub fn new<S: Into<String>>(name :S, host :S, ip :S, group :S) -> User {
        User{
            name: name.into(),
            host: host.into(),
            ip: ip.into(),
            group: group.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operate {
    ADD, REMOVE
}

#[derive(Clone, Debug)]
pub struct OperUser{
    pub user :User,
    pub oper: Operate,
}

impl OperUser {
    pub fn new(user: User, oper :Operate) -> OperUser{
        OperUser{
            user: user,
            oper: oper,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShareInfo {
    //包编号
    pub packet_no: u32,
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
    pub attach_time: NaiveTime,
}

#[derive(Clone, Debug)]
pub struct FileInfo {
    //要传输文件id
    pub file_id: u32,
    //文件名
    pub file_name: PathBuf,
    //文件的属性，如是文件或者文件夹，只读等
    pub attr: u8,// 1 普通文件 2 文件夹
    //文件大小
    pub size: u64,
    //文件最后一次修改时间
    pub mtime: NaiveTime,
    //文件最后一次访问时间
    pub atime: NaiveTime,
    //文件创建时间
    pub crtime: NaiveTime,
    pub is_selected: bool,
}

impl FileInfo {
    pub fn to_fileinfo_msg(&self) -> String {
        let file_name = self.file_name.as_path().file_name().unwrap().to_str().unwrap();
        format!("{}:{}:{:x}:{:x}:{}:", self.file_id, file_name, self.size, self.mtime.second(), self.attr)
    }
}


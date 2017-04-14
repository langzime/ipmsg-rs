use std::net::TcpStream;
use chrono::prelude::*;
use constant;

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

impl Packet {

    ///new packet
    pub fn new(command_no: u32, additional_section: Option<String>) -> Packet {
        let local: DateTime<Local> = Local::now();
        let hostname = ::hostname::get_hostname().unwrap();
        Packet {
            ver: format!("{}", constant::IPMSG_VERSION),
            packet_no: format!("{}", local.timestamp()),
            sender_name: hostname.clone(),
            sender_host: hostname.clone(),
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
        let hostname = ::hostname::get_hostname().unwrap();
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
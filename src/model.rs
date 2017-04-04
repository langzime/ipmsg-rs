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
    pub fn from(ver: String, packet_no: String, sender_name: String, sender_host: String, command_no: u32, additional_section: Option<String>) -> Packet {
        Packet {
            ver: ver,
            packet_no: packet_no,
            sender_name: sender_name,
            sender_host: sender_host,
            command_no: command_no,
            additional_section: additional_section,
            ip: "".to_owned(),
        }
    }
}

impl ToString for Packet {
    fn to_string(&self) -> String {
        let ext = &self.additional_section;
        //ext.as_ref().unwrap_or(&hostname)
        let hostname = ::hostname::get_hostname().unwrap();
        format!("{}:{}:{}:{}:{}:{}",
                self.ver,
                self.packet_no,
                self.sender_name,
                self.sender_host,
                self.command_no,
                ext.as_ref().unwrap_or(&"".to_owned()))
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
    pub fn new(name :String, host :String, ip :String, group :String) -> User {
        User{
            name: name,
            host: host,
            ip: ip,
            group: group,
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
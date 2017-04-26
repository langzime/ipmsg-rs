/*  header  */
///飞鸽版本
pub const IPMSG_VERSION: u32 = 0x0001;
///默认端口号
pub const IPMSG_DEFAULT_PORT: u32 = 0x0979;

/*  command  */
/// 无操作
pub const IPMSG_NOOPERATION: u32 = 0x00000000;
///上线（开始于广播此命令
pub const IPMSG_BR_ENTRY: u32 = 0x00000001;
///下线（结束于广播此命令）
pub const IPMSG_BR_EXIT: u32 = 0x00000002;
///通报新上线
pub const IPMSG_ANSENTRY: u32 = 0x00000003;
///更改为离开状态
pub const IPMSG_BR_ABSENCE: u32 = 0x00000004;


///搜寻有效的主机用户
pub const IPMSG_BR_ISGETLIST: u32 = 0x00000010;
///主机列表发送通知
pub const IPMSG_OKGETLIST: u32 = 0x00000011;
//主机列表发送请求
pub const IPMSG_GETLIST: u32 = 0x00000012;
///主机列表发送
pub const IPMSG_ANSLIST: u32 = 0x00000013;

pub const IPMSG_BR_ISGETLIST2: u32 = 0x00000018;


///消息发送
pub const IPMSG_SENDMSG: u32 = 0x00000020;
///消息收到确认
pub const IPMSG_RECVMSG: u32 = 0x00000021;
///消息打开通知
pub const IPMSG_READMSG: u32 = 0x00000030;
///消息丢弃通知
pub const IPMSG_DELMSG: u32 = 0x00000031;

///消息打开确认通知（version-8中添加）
pub const IPMSG_ANSREADMSG: u32 = 0x00000032;
///获得IPMSG版本信息
pub const IPMSG_GETINFO: u32 = 0x00000040;
///发送IPMSG版本信息
pub const IPMSG_SENDINFO: u32 = 0x00000041;

///获得缺席信息
pub const IPMSG_GETABSENCEINFO: u32 = 0x00000050;
///发送缺席信息
pub const IPMSG_SENDABSENCEINFO: u32 = 0x00000051;

///文件传输请求
pub const IPMSG_GETFILEDATA: u32 = 0x00000060;
///丢弃附加文件
pub const IPMSG_RELEASEFILES: u32 = 0x00000061;
///附着统计文件请求
pub const IPMSG_GETDIRFILES: u32 = 0x00000062;

///获得RSA公钥
pub const IPMSG_GETPUBKEY: u32 = 0x00000072;
///应答RSA公钥
pub const IPMSG_ANSPUBKEY: u32 = 0x00000073;

/* file types for fileattach command */
pub const IPMSG_FILE_REGULAR: u32 = 0x00000001;
pub const IPMSG_FILE_DIR: u32 = 0x00000002;
pub const IPMSG_FILE_RETPARENT: u32 = 0x00000003;// return parent directory
pub const IPMSG_FILE_SYMLINK: u32 = 0x00000004;
pub const IPMSG_FILE_CDEV: u32 = 0x00000005;// for UNIX
pub const IPMSG_FILE_BDEV: u32 = 0x00000006;// for UNIX
pub const IPMSG_FILE_FIFO: u32 = 0x00000007;// for UNIX
pub const IPMSG_FILE_RESFORK: u32 = 0x00000010;// for mac

/* file attribute options for fileattach command */
pub const IPMSG_FILE_RONLYOPT: u32 = 0x00000100;
pub const IPMSG_FILE_HIDDENOPT: u32 = 0x00001000;
pub const IPMSG_FILE_EXHIDDENOPT: u32 = 0x00002000;// for MacOS X
pub const IPMSG_FILE_ARCHIVEOPT: u32 = 0x00004000;
pub const IPMSG_FILE_SYSTEMOPT: u32 = 0x00008000;

/* extend attribute types for fileattach command */
pub const IPMSG_FILE_CREATETIME: u32 = 0x00000016;
pub const IPMSG_FILE_MTIME: u32 = 0x00000014;

pub const FILELIST_SEPARATOR: char = '\u{7}';
pub const HOSTLIST_SEPARATOR: char = '\u{7}';

/* option or all command */
///存在/缺席模式（成员识别命令中使用）
pub const IPMSG_ABSENCEOPT: u32 = 0x00000100;
///服务器模式（预留）
pub const IPMSG_SERVEROPT: u32 = 0x00000200;
///发送单个成员识别命令
pub const IPMSG_DIALUPOPT: u32 = 0x00010000;
///附件
pub const IPMSG_FILEATTACHOPT: u32 = 0x00200000;
///密码
pub const IPMSG_ENCRYPTOPT: u32 = 0x00400000;
///全部使用utf-8
pub const IPMSG_UTF8OPT: u32 = 0x00800000;
///兼容utf-8
pub const IPMSG_CAPUTF8OPT: u32 = 0x01000000;
///加密的附件信息
pub const IPMSG_ENCEXTMSGOPT: u32 = 0x04000000;
///支持图像
pub const IPMSG_CLIPBOARDOPT: u32 = 0x08000000;
pub const IPMSG_CAPFILEENC_OBSLT: u32 = 0x00001000;
pub const IPMSG_CAPFILEENCOPT: u32 = 0x00040000;

/* option for sendmsg command */
///需要回信确认
pub const IPMSG_SENDCHECKOPT: u32 = 0x00000100;
//密封消息
pub const IPMSG_SECRETOPT: u32 = 0x00000200;
///广播（报告）
pub const IPMSG_BROADCASTOPT: u32 = 0x00000400;
///组播（多选）
pub const IPMSG_MULTICASTOPT: u32 = 0x00000800;
///自动应答
pub const IPMSG_AUTORETOPT: u32 = 0x00002000;
///重试标志（搜索HOSTLIST时使用）
pub const IPMSG_RETRYOPT: u32 = 0x00004000;
///挂锁
pub const IPMSG_PASSWORDOPT: u32 = 0x00008000;
///不保留日志
pub const IPMSG_NOLOGOPT: u32 = 0x00020000;
///通知BR_ENTRY以外的成员
pub const IPMSG_NOADDLISTOPT: u32 = 0x00080000;
///密封消息确认
pub const IPMSG_READCHECKOPT: u32 = 0x00100000;
pub const IPMSG_SECRETEXOPT: u32 = IPMSG_READCHECKOPT|IPMSG_SECRETOPT;

pub const IPMSG_LIMITED_BROADCAST: &'static str = "255.255.255.255";

pub fn get_mode(command: u32) -> u32 {
    command & 0x000000ff
}

pub fn get_opt(command: u32) -> u32 {
    command & 0xffffff00
}

extern crate hostname as host_name;
extern crate local_ip;

use std::net::IpAddr;

///得到本地ip
pub fn get_local_ip() -> Option<IpAddr> {
    local_ip::get()
}

///得到主机名
pub fn get_host_name() -> Option<String> {
    host_name::get_hostname()
}

lazy_static! {
    pub static ref hostname: String = get_host_name().unwrap();
    pub static ref localip: String = get_local_ip().unwrap().to_string();
    pub static ref addr: String = format!("{}{}", "0.0.0.0:", IPMSG_DEFAULT_PORT);
}



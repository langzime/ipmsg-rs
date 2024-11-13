use crate::constants::protocol;
use crate::constants::protocol::{HOST_NAME, IPMSG_PACKET_DELIMITER, LOCAL_IP, REPARENT_PATH};
use crate::core::{GLOBLE_RECEIVER, GLOBLE_SENDER};
use crate::models::event::{ModelEvent, TcpEvent, UdpEvent, UiEvent};
use crate::models::message::create_sendmsg;
use crate::models::model::{Packet, ReceivedPacketInner, ReceivedSimpleFileInfo, ShareInfo, User};
use crate::store::logic::insert_message;
use crate::store::models::NewMessage;
use crate::util::packet_parser;
use crate::{constants, util, IpmsgUI};
use anyhow::Result;
use combine::Parser;
use crossbeam_channel::SendError;
use encoding::all::GB18030;
use encoding::{DecoderTrap, Encoding};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use slint::Weak;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{fs, select};

pub struct UdpWorker {
    pub channel: UnboundedSender<UdpEvent>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl UdpWorker {
    pub fn new(ui_event_sender: UnboundedSender<UiEvent>) -> Self {
        let (channel, r) = mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({ move || tokio::runtime::Runtime::new().unwrap().block_on(udp_loop(r, ui_event_sender)).unwrap() });
        UdpWorker { worker_thread, channel }
    }

    pub fn join(self) -> Result<()> {
        let _ = self.channel.send(UdpEvent::Quit);
        self.worker_thread.join().unwrap();
        Ok(())
    }
}

async fn udp_loop(mut r: UnboundedReceiver<UdpEvent>, ui_event_sender: UnboundedSender<UiEvent>) -> Result<()> {
    let socket = match UdpSocket::bind(protocol::ADDR.as_str()).await {
        Ok(s) => {
            info!("udp server start listening! {:?}", protocol::ADDR.as_str());
            s
        }
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let soc_r = Arc::new(socket);
    let soc_w = soc_r.clone();
    let mut buf = [0; 2048];
    loop {
        select! {
            res = soc_r.recv_from(&mut buf) => {
                if let Ok((len, addr)) = res {
                    debug!("{:?} bytes received from {:?}", len, addr);
                    let (bytes, addr) = (buf[..len].to_vec(), addr);
                    if let Ok(receive_str) = GB18030.decode(&bytes, DecoderTrap::Strict) {
                        info!("receive raw message -> {:?} from ip -> {:?}", receive_str, addr.ip());
                        if let Ok((mut packet, _)) = packet_parser().parse(receive_str.as_str()) {
                            packet.ip = addr.ip().to_string();
                            model_packet_dispatcher(packet, ui_event_sender.clone())?;
                        } else {
                            error!("packet parser fail!");
                        }
                    } else {
                        error!("decode raw bytes fail!");
                    }
                }
            }
            udp_res = r.recv() => {
                if let Some(event) = udp_res {
                    match event {
                        UdpEvent::Quit => {
                            break;
                        }
                        UdpEvent::Bytes(bytes) => {
                            match soc_w.send(&*bytes).await {
                                Ok(_size) => {}
                                Err(e) => {
                                    error!("soc_w send failed!! {e}");
                                }
                            };
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn model_packet_dispatcher(packet: Packet, ui_event_sender: UnboundedSender<UiEvent>) -> Result<()> {
    let mut extstr = String::new();
    if let Some(ref additional_section) = packet.additional_section {
        extstr = additional_section.to_owned();
    }
    let opt = protocol::get_opt(packet.command_no);
    let cmd = protocol::get_mode(packet.command_no);

    if opt & protocol::IPMSG_SENDCHECKOPT != 0 {
        let recvmsg = Packet::new(protocol::IPMSG_RECVMSG, Some(packet.packet_no.to_string()));
        GLOBLE_SENDER.send(ModelEvent::RecMsgReply {
            packet: recvmsg,
            from_ip: packet.ip.to_owned(),
        })?;
    }
    if cmd == protocol::IPMSG_BR_EXIT {
        //收到下线通知消息
        GLOBLE_SENDER.send(ModelEvent::BroadcastExit(packet.sender_host.to_owned()))?;
    } else if cmd == protocol::IPMSG_BR_ENTRY {
        //收到上线通知消息
        ///扩展段 用户名|用户组
        let ext_vec = extstr.splitn(2, |c| c == ':').collect::<Vec<&str>>();
        let ansentry_packet = Packet::new(protocol::IPMSG_ANSENTRY, None);

        let group_name = if ext_vec.len() > 2 { ext_vec[1].to_owned() } else { "".to_owned() };
        let user_name = if ext_vec.len() > 1 && !ext_vec[0].is_empty() {
            ext_vec[0].to_owned()
        } else {
            packet.sender_name.clone()
        };

        let user = User::new(user_name, packet.sender_host.to_owned(), packet.ip.to_owned(), group_name);
        info!("{user:?}");
        GLOBLE_SENDER.send(ModelEvent::RecOnlineMsgReply {
            packet: ansentry_packet,
            from_user: user,
        })?;
    } else if cmd == protocol::IPMSG_ANSENTRY {
        //通报新上线
        let user = User::new(
            packet.sender_name.to_owned(),
            packet.sender_host.to_owned(),
            packet.ip.to_owned(),
            "".to_owned(),
        );
        GLOBLE_SENDER.send(ModelEvent::NotifyOnline { user })?;
    } else if cmd == protocol::IPMSG_SENDMSG {
        //收到发送的消息
        //文字消息|文件扩展段
        let ext_vec = extstr.split('\0').collect::<Vec<&str>>();
        if opt & protocol::IPMSG_SECRETOPT != 0 {
            //是否是密封消息
            info!("i am secret message !");
        }
        //文字消息内容|文件扩展
        let mut files_opt: Option<Vec<ReceivedSimpleFileInfo>> = None;
        if opt & protocol::IPMSG_FILEATTACHOPT != 0 {
            if ext_vec.len() > 1 {
                let files_str = ext_vec[1];
                info!("i have file attachment {:?}", files_str);
                let files = files_str
                    .split(protocol::FILELIST_SEPARATOR)
                    .into_iter()
                    .filter(|x: &&str| !x.is_empty())
                    .collect::<Vec<&str>>();
                let mut simple_file_infos = Vec::new();
                for file_str in files {
                    let file_attr = file_str
                        .splitn(6, |c| c == ':')
                        .into_iter()
                        .filter(|x: &&str| !x.is_empty())
                        .collect::<Vec<&str>>();
                    if file_attr.len() >= 5 {
                        let file_id = file_attr[0].parse::<u32>().unwrap();
                        let file_name = file_attr[1];
                        let size = u64::from_str_radix(file_attr[2], 16).unwrap(); //大小
                        let mmtime = file_attr[3]; //修改时间
                        let mut mmtime_num = i64::from_str_radix(mmtime, 16).unwrap(); //时间戳
                        if mmtime_num >= 10000000000 {
                            mmtime_num = mmtime_num / 1000;
                        }
                        let file_attr = file_attr[4].parse::<u32>()?; //文件属性
                        if file_attr == protocol::IPMSG_FILE_REGULAR {
                            info!("i am ipmsg_file_regular");
                        } else if file_attr == protocol::IPMSG_FILE_DIR {
                            info!("i am ipmsg_file_dir");
                        } else {
                            panic!("no no type")
                        }
                        let simple_file_info = ReceivedSimpleFileInfo {
                            file_id,
                            packet_id: packet.packet_no.parse::<u32>()?,
                            name: file_name.to_owned(),
                            attr: file_attr as u8,
                            size,
                            mtime: mmtime_num,
                        };
                        simple_file_infos.push(simple_file_info);
                    }
                }
                if simple_file_infos.len() > 0 {
                    files_opt = Some(simple_file_infos);
                }
            };
        }
        let packet_clone = packet.clone();
        let received_packet_inner = ReceivedPacketInner::new(packet.ip.to_owned()).packet(packet_clone).option_opt_files(files_opt);
        GLOBLE_SENDER.send(ModelEvent::ReceivedMsg { msg: received_packet_inner })?;
    } else if cmd == protocol::IPMSG_NOOPERATION {
        info!("i am IPMSG_NOOPERATION");
    } else if cmd == protocol::IPMSG_BR_ABSENCE {
        info!("i am IPMSG_BR_ABSENCE");
    } else {
    }
    Ok(())
}

///分享的文件列表
pub static FILE_LIST: Lazy<Arc<Mutex<Vec<ShareInfo>>>> = Lazy::new(|| return Default::default());
pub struct TcpWorker {
    pub channel: UnboundedSender<TcpEvent>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl TcpWorker {
    pub fn new() -> Self {
        let (channel, r) = mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({ move || tokio::runtime::Runtime::new().unwrap().block_on(tcp_loop(r)).unwrap() });
        TcpWorker { worker_thread, channel }
    }

    pub fn join(self) -> Result<()> {
        let _ = self.channel.send(TcpEvent::Quit);
        self.worker_thread.join().unwrap();
        Ok(())
    }
}

async fn tcp_loop(mut r: UnboundedReceiver<TcpEvent>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    for (mut stream, addr) in listener.accept().await {
        tokio::spawn(async move {
            let mut buf = [0; 2048];
            let byte_size = stream.read(&mut buf[..]).await.unwrap();
            let tmp_str = GB18030.decode(&buf[0..byte_size], DecoderTrap::Strict).unwrap();
            info!("file_processer receive raw str {:?}", tmp_str);
            let result = packet_parser().parse(tmp_str.as_str());
            match result {
                Ok((mut packet, _)) => {
                    packet.ip = addr.ip().to_string();
                    let cmd = protocol::get_mode(packet.command_no);
                    if packet.additional_section.is_some() {
                        if cmd == protocol::IPMSG_GETFILEDATA {
                            //文件请求
                            //FileServer::process_file(&pool_tmp, &mut stream_echo, packet.additional_section.unwrap())
                        } else if cmd == protocol::IPMSG_GETDIRFILES {
                            //FileServer::process_dir(pool_tmp, stream_echo, packet.additional_section.unwrap())
                        } else {
                            info!("Invalid packet tcp file cmd {:?} !", tmp_str);
                        }
                    } else {
                        info!("Invalid packet additional_section is none {:?} !", tmp_str);
                    }
                }
                Err(_) => {
                    info!("Invalid packet tcp file cmd {:?} !", tmp_str);
                }
            }
        });
    }
    Ok(())
}

async fn process_file(pool_tmp: &Arc<Mutex<Vec<ShareInfo>>>, stream_echo: &mut TcpStream, ext_str: String) -> Result<()> {
    let file_attr = ext_str
        .splitn(4, |c| c == ':')
        .into_iter()
        .filter(|x: &&str| !x.is_empty())
        .collect::<Vec<&str>>();
    info!("file packet parse {:?}", file_attr);
    if file_attr.len() >= 3 {
        let packet_id = i64::from_str_radix(file_attr[0], 16)? as u32;
        let file_id = i64::from_str_radix(file_attr[1], 16)?;
        let offset = file_attr[2].parse::<u32>()?;
        let mut search_result: Option<ShareInfo> = None;
        {
            let search = pool_tmp.lock()?;
            let ref vec: Vec<ShareInfo> = *search;
            let result = vec.iter().find(|ref s| s.packet_no == packet_id as i64);
            search_result = result.cloned();
        }
        if let Some(result_share_file) = search_result {
            let file_info = result_share_file.file_info.iter().find(|f| f.file_id == file_id);
            if let Some(file_info) = file_info {
                let mut f: File = File::open(&file_info.file_name)?;
                let mut buf = [0; 1024];
                let mut buffer = BufWriter::new(stream_echo);
                while let Ok(bytes_read) = f.read(&mut buf).await {
                    if bytes_read == 0 {
                        break;
                    }
                    buffer.write(&buf[..bytes_read]).await?;
                    buffer.flush().await?;
                }
            }
        }
    }
    Ok(())
}

async fn process_dir(pool_tmp: Arc<Mutex<Vec<ShareInfo>>>, mut stream_echo: TcpStream, ext_str: String) -> Result<()> {
    let file_attr = ext_str
        .splitn(3, |c| c == ':')
        .into_iter()
        .filter(|x: &&str| !x.is_empty())
        .collect::<Vec<&str>>();
    info!("file dir packet parse {:?}", file_attr);
    if file_attr.len() >= 2 {
        let packet_id = i64::from_str_radix(file_attr[0], 16)?;
        let file_id = i64::from_str_radix(file_attr[1], 16)?;
        let mut search_result: Option<ShareInfo> = Option::None;
        {
            let search = pool_tmp.lock()?;
            let ref vec: Vec<ShareInfo> = *search;
            let result = vec.iter().find(|ref s| s.packet_no == packet_id);
            search_result = result.cloned();
        }
        if let Some(result_share_file) = search_result {
            let file_info = result_share_file.file_info.iter().find(|ref f| f.file_id == file_id);
            if let Some(file_info) = file_info {
                let ref root_path: PathBuf = file_info.file_name;
                let mut buffer = BufWriter::new(stream_echo);
                send_dir(root_path, &mut buffer).await?;
            }
        }
    }
    Ok(())
}

pub async fn send_dir(root_path: &PathBuf, mut buffer: &mut BufWriter<TcpStream>) -> Result<()> {
    buffer.write(util::utf8_to_gb18030(&make_header(&root_path, false)).as_slice()).await?;
    debug!("{:?}", make_header(&root_path, false));
    if root_path.is_dir() {
        let mut entries = fs::read_dir(".").await?;
        while let Some(entry) = entries.next_entry().await? {
            let sub = entry.path();
            if sub.is_file() {
                let header = make_header(&sub, false).await?;
                buffer.write(util::utf8_to_gb18030(&header).as_slice()).await?;
                info!("{:?}", header);
                let mut buf = [0; 1024];
                let mut f: File = File::open(&sub).await.unwrap();
                while let Ok(bytes_read) = f.read(&mut buf).await {
                    if bytes_read == 0 {
                        break;
                    }
                    buffer.write(&buf[..bytes_read]).await.unwrap();
                    buffer.flush().await.unwrap();
                }
            } else {
                Box::pin(send_dir(&sub, &mut buffer)).await?;
            }
        }
    }
    let ret_parent = crate::core::fileserver::make_header(root_path, true);
    buffer.write(ret_parent.as_bytes()).unwrap();
    debug!("{ret_parent:?}");
    Ok(())
}

///
/// 转换报文
pub async fn make_header(path: &PathBuf, ret_parent: bool) -> Result<String> {
    let file_name;
    let file_attr;
    let file_size;
    let mut header = String::new();
    header.push(IPMSG_PACKET_DELIMITER);
    if ret_parent {
        file_attr = protocol::IPMSG_FILE_RETPARENT;
        let tmp_file_name = format!("{}", REPARENT_PATH);
        header.push_str(tmp_file_name.as_str()); //filename
        file_size = 0;
    } else {
        let path_metadata = fs::metadata(&path).await?;
        file_size = path_metadata.len();
        file_name = path.file_name().unwrap().to_str().unwrap();
        header.push_str(file_name);
        if path_metadata.is_dir() {
            file_attr = protocol::IPMSG_FILE_DIR;
        } else {
            file_attr = protocol::IPMSG_FILE_REGULAR;
        }
        path_metadata.created();
        let _ = path_metadata.modified();
    }

    header.push(IPMSG_PACKET_DELIMITER);
    header.push_str(format!("{:x}", file_size).as_str()); //filesize//
    header.push(IPMSG_PACKET_DELIMITER);
    header.push_str(format!("{:x}", file_attr).as_str()); //fileattr
    let timestamp_now = OffsetDateTime::now_utc().unix_timestamp();
    header.push_str(
        format!(
            ":{:x}={:x}:{:x}={:x}:",
            protocol::IPMSG_FILE_CREATETIME,
            timestamp_now,
            protocol::IPMSG_FILE_MTIME,
            timestamp_now
        )
        .as_str(),
    ); //
    let mut length = util::utf8_to_gb18030(&header).len();
    length = length + format!("{:0>4x}", length).len();
    header.insert_str(0, format!("{:0>4x}", length).as_str());
    Ok(header)
}

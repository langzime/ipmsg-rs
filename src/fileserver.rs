
use std::sync::{Mutex, Arc};
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::{Read, Write, BufWriter};
use std::path::PathBuf;
use std::fs::{self, File, Metadata, ReadDir};
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::prelude::*;
use log::{info, trace, warn, debug};
use combine::parser::Parser;
use crate::model::{FileInfo, Packet, ShareInfo};
use crate::{constant, util};
use crate::constant::REPARENT_PATH;
use crate::util::packet_parser;

#[derive(Clone, Debug)]
pub struct FileServer {
    pub file_pool: Arc<Mutex<Vec<ShareInfo>>>
}

impl FileServer {

    pub fn new(file_pool: Arc<Mutex<Vec<ShareInfo>>>) -> FileServer {
        FileServer{
            file_pool
        }
    }

    pub fn run(&self) {
        let pool_tmp = self.file_pool.clone();
        thread::spawn(move || {
            let tcp_listener: TcpListener = TcpListener::bind(constant::ADDR.as_str()).unwrap();
            let pool_tmp = pool_tmp.clone();
            info!("tcp server start listening! {:?}", constant::ADDR.as_str());
            for stream in tcp_listener.incoming() {
                let base_stream = stream.unwrap().try_clone().unwrap();
                //let search_arc = search_arc_tmp.clone();
                let pool_tmp = pool_tmp.clone();
                let src = base_stream.peer_addr().unwrap();
                thread::spawn(move || {
                    let mut stream_echo = base_stream;
                    let mut buf = [0; 2048];
                    let byte_size = stream_echo.read(&mut buf[..]).unwrap();
                    let mut tmp_buf = &buf[0..byte_size];
                    let tmp_str = GB18030.decode(&tmp_buf, DecoderTrap::Strict).unwrap();
                    info!("file_processer receive raw str {:?}", tmp_str);
                    let result = packet_parser().parse(tmp_str.as_str());
                    match result {
                        Ok((mut packet, _)) => {
                            packet.ip = src.ip().to_string();
                            let cmd = constant::get_mode(packet.command_no);
                            if packet.additional_section.is_some() {
                                if cmd == constant::IPMSG_GETFILEDATA {
                                        //文件请求
                                    FileServer::process_file(&pool_tmp, &mut stream_echo, packet.additional_section.unwrap())
                                }else if cmd == constant::IPMSG_GETDIRFILES {
                                    FileServer::process_dir(pool_tmp, stream_echo, packet.additional_section.unwrap())
                                }else {
                                    info!("Invalid packet tcp file cmd {:?} !", tmp_str);
                                }
                            }else{
                                info!("Invalid packet additional_section is none {:?} !", tmp_str);
                            }
                        }
                        Err(_) => {
                            info!("Invalid packet tcp file cmd {:?} !", tmp_str);
                        }
                    }
                });
            }
        });
    }

    fn process_dir(pool_tmp: Arc<Mutex<Vec<ShareInfo>>>, mut stream_echo: TcpStream, ext_str: String) -> () {
        let file_attr = ext_str.splitn(3, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
        info!("file dir packet parse {:?}", file_attr);
        if file_attr.len() >= 2 {
            let packet_id = i64::from_str_radix(file_attr[0], 16).unwrap() as u32;
            let file_id = i64::from_str_radix(file_attr[1], 16).unwrap();
            let mut search_result: Option<ShareInfo> = Option::None;
            {
                let search = pool_tmp.lock().unwrap();
                let ref vec: Vec<ShareInfo> = *search;
                let result = vec.iter().find(|ref s| s.packet_no == packet_id);
                search_result = result.cloned();
            }
            if let Some(result_share_file) = search_result {
                let file_info = result_share_file.file_info.iter().find(|ref f| f.file_id == file_id as u32);
                if let Some(file_info) = file_info {
                    let ref root_path: PathBuf = file_info.file_name;
                    let mut buffer = BufWriter::new(stream_echo.try_clone().unwrap());
                    send_dir(root_path, &mut buffer);
                }
            }
        }
    }

    fn process_file(pool_tmp: &Arc<Mutex<Vec<ShareInfo>>>, mut stream_echo: &mut TcpStream, ext_str: String) -> () {
        let file_attr = ext_str.splitn(4, |c| c == ':').into_iter().filter(|x: &&str| !x.is_empty()).collect::<Vec<&str>>();
        info!("file packet parse {:?}", file_attr);
        if file_attr.len() >= 3 {
            let packet_id = i64::from_str_radix(file_attr[0], 16).unwrap() as u32;
            let file_id = i64::from_str_radix(file_attr[1], 16).unwrap();
            let offset = file_attr[2].parse::<u32>().unwrap();
            let mut search_result: Option<ShareInfo> = Option::None;
            {
                let search = pool_tmp.lock().unwrap();
                let ref vec: Vec<ShareInfo> = *search;
                let result = vec.iter().find(|ref s| s.packet_no == packet_id);
                search_result = result.cloned();
            }
            if let Some(result_share_file) = search_result {
                let file_info = result_share_file.file_info.iter().find(|f| f.file_id == file_id as u32);
                if let Some(file_info) = file_info {
                    let mut f: File = File::open(&file_info.file_name).unwrap();
                    let mut buf = [0; 1024];
                    let mut buffer = BufWriter::new(stream_echo);
                    while let Ok(bytes_read) = f.read(&mut buf) {
                        if bytes_read == 0 { break; }
                        buffer.write(&buf[..bytes_read]).unwrap();
                        buffer.flush().unwrap();
                    }
                }
            }
        }
    }
}

//send dir
pub fn send_dir(root_path: &PathBuf, mut buffer : & mut BufWriter<TcpStream>) {
    buffer.write(util::utf8_to_gb18030(&make_header(&root_path, false)).as_slice()).unwrap();//root dir
    debug!("{:?}", make_header(&root_path, false));
    if root_path.is_dir() {
        for sub_path in fs::read_dir(root_path).unwrap() {
            let sub = &sub_path.unwrap().path();
            if sub.is_file() {
                let header = make_header(sub, false);
                buffer.write(util::utf8_to_gb18030(&header).as_slice()).unwrap();
                info!("{:?}", header);
                let mut buf = [0; 1024];
                let mut f: File = File::open(sub).unwrap();
                while let Ok(bytes_read) = f.read(&mut buf) {
                    if bytes_read == 0 { break; }
                    buffer.write(&buf[..bytes_read]).unwrap();
                    buffer.flush().unwrap();
                }
            }else {
                send_dir(sub, &mut buffer);
            }
        }
    }
    let ret_parent = make_header(root_path, true);
    buffer.write(ret_parent.as_bytes()).unwrap();
    debug!("{ret_parent:?}");
}

///
/// 转换报文
pub fn make_header(path: &PathBuf, ret_parent: bool) -> String {
    let file_name;
    let file_attr;
    let file_size;
    if ret_parent {
        file_attr = constant::IPMSG_FILE_RETPARENT;
        file_name = REPARENT_PATH;
        file_size = 0;
    }else{
        let path_metadata: Metadata = fs::metadata(&path).unwrap();
        file_size = path_metadata.len();
        if path_metadata.is_dir() {
            file_attr = constant::IPMSG_FILE_DIR;
        } else {
            file_attr = constant::IPMSG_FILE_REGULAR;
        }
    }
    let file_name: &str = &path.file_name().unwrap().to_str().unwrap();
    let mut header = String::new();
    header.push_str(":");
    header.push_str(file_name);//filename
    header.push_str(":");
    header.push_str(format!("{:x}", file_size).as_str());//filesize//
    header.push_str(":");
    header.push_str(format!("{:x}", file_attr).as_str());//fileattr
    let timestamp_now = Local::now().timestamp();
    header.push_str(format!(":{:x}={:x}:{:x}={:x}:", constant::IPMSG_FILE_CREATETIME, timestamp_now, constant::IPMSG_FILE_MTIME, timestamp_now).as_str());//
    let mut length = util::utf8_to_gb18030(&header).len();
    length = length + format!("{:0>4x}", length).len();
    header.insert_str(0, format!("{:0>4x}", length).as_str());
    header
}
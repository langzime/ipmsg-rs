use std::io::prelude::*;
use std::net::TcpStream;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::thread;
use std::fmt::{self, Display};
use std::error::Error;
use std::fs::{self, File, ReadDir};
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use log::{info, trace, warn, debug};
use anyhow::{Result, anyhow};
use crate::constant::{self, IPMSG_SENDMSG, IPMSG_GETFILEDATA, IPMSG_GETDIRFILES, IPMSG_FILE_DIR, IPMSG_FILE_REGULAR, IPMSG_FILE_RETPARENT, IPMSG_PACKET_DELIMITER};
use crate::events::model::ModelEvent;
use crate::GLOBLE_SENDER;
use crate::model::Packet;
use crate::model::{FileInfo, ReceivedSimpleFileInfo};

#[derive(Clone, Debug)]
pub struct ManagerPool {
    pub file_pool: Arc<Mutex<HashMap<u32, PoolFile>>>,
}

impl ManagerPool {

    pub fn new(file_pool: Arc<Mutex<HashMap< u32, PoolFile>>>) -> ManagerPool {
        ManagerPool { file_pool}
    }

    pub fn run(mut self, file_info: ReceivedSimpleFileInfo, save_path: PathBuf, download_ip: String) {
        let tmp = self.clone();
        {
            let mut lock = tmp.file_pool.lock().unwrap();
            let file = lock.get(&file_info.file_id);
            if let Some(p_file) = file {
                if p_file.status == 1 {
                    //下载中
                    GLOBLE_SENDER.send(ModelEvent::DownloadIsBusy{ file: file_info }).expect("send DownloadIsBusy fail!");
                    return;
                }
            }else {
                lock.insert(file_info.file_id, PoolFile { status: 1, file_info: file_info.clone() });
            }
        }
        let tmp = self.clone();
        thread::spawn(move || {
            let download_url = format!("{}:{}", download_ip, constant::IPMSG_DEFAULT_PORT);
            let is_ok = download(download_url, save_path, file_info.clone()).is_ok();
            {
                let mut lock = tmp.file_pool.lock().unwrap();
                let mut file = lock.get(&file_info.file_id);
                if let Some(p_file) = file.take() {
                    if is_ok {
                        lock.remove(&file_info.file_id);
                        GLOBLE_SENDER.send(ModelEvent::RemoveDownloadTaskInPool{ packet_id: file_info.packet_id, file_id: file_info.file_id, download_ip }).expect("send RemoveDownloadTaskInPool fail!");
                    }else{
                        let mut tmp_file = p_file.clone();
                        tmp_file.status = 0;
                        lock.insert(tmp_file.file_info.file_id, tmp_file);
                    }

                }
            }

        });
    }

}

#[derive(Clone, Debug)]
pub struct PoolFile {
    pub status: u8, //0 初始 1 下载中
    pub file_info: ReceivedSimpleFileInfo,
}

pub fn download<A: ToSocketAddrs, S: AsRef<Path>>(addr: A, to_path: S, r_file: ReceivedSimpleFileInfo) -> Result<()> {
    info!("start download file");
    let file_type = r_file.attr as u32;
    let mut stream = TcpStream::connect(addr)?;
    let packet = Packet::new(IPMSG_SENDMSG| if file_type == IPMSG_FILE_DIR { IPMSG_GETDIRFILES } else { IPMSG_GETFILEDATA }, Some(format!("{:x}:{:x}:0:\u{0}", r_file.packet_id, r_file.file_id)));
    stream.write(packet.to_string().as_bytes())?;
    debug!("filetype {}", file_type);
    if file_type == IPMSG_FILE_REGULAR {
        let mut file_location = to_path.as_ref().to_path_buf();
        file_location.push(r_file.name);
        let file_size = r_file.size;
        let mut buffer = BufReader::new(stream);
        read_bytes_to_file(&mut buffer, file_size, &file_location)?;
    }else if file_type == IPMSG_FILE_DIR {
        let mut next_path = to_path.as_ref().to_path_buf();
        let mut buffer = BufReader::new(stream);
        while let Ok(Some(header_size_str)) = read_delimiter(&mut buffer) {
            let header_size = u64::from_str_radix(&header_size_str, 16)?;
            info!("header_size {:?}", header_size);
            let header_context_str = read_bytes(&mut buffer, (header_size - 1 - header_size_str.as_bytes().len() as u64))?;//-1是减去的那个冒号
            let v: Vec<&str> = header_context_str.splitn(4, |c| c == ':').collect();
            let file_name = v[0];
            let file_size = u64::from_str_radix(v[1], 16)?;
            let file_attr = u32::from_str_radix(v[2], 16)?;
            let opt = constant::get_opt(file_attr);
            let cmd = constant::get_mode(file_attr);
            info!("header context {:?}", v);
            if cmd == IPMSG_FILE_DIR {
                next_path.push(file_name);
                if !next_path.exists() {
                    fs::create_dir(&next_path)?;
                }
                info!("crate dir{:?}", next_path);
            }else if cmd == IPMSG_FILE_REGULAR {
                next_path.push(file_name);
                info!("crate file{:?}", next_path);
                read_bytes_to_file(&mut buffer, file_size, &next_path)?;
                next_path.pop();
            }else if cmd == IPMSG_FILE_RETPARENT  {
                next_path.pop();
                info!("back to parent {:?}", next_path);
            }else {

            }
        }
    }
    info!("download end!");
    Ok(())
}

fn read_delimiter(mut stream : & mut BufReader<TcpStream>) -> Result<Option<String>> {
    let mut s_buffer = Vec::new();
    let len = stream.read_until(u8::try_from(IPMSG_PACKET_DELIMITER)?, &mut s_buffer)?;
    if len != 0usize {
        if len > 200 {
            return Err(anyhow!("read_delimiter error!"));
        }else {
            s_buffer.pop();
            Ok(Some(String::from_utf8(s_buffer).unwrap()))
        }
    }else {
        Ok(None)
    }
}

fn read_bytes(mut stream : & mut BufReader<TcpStream>, len: u64) -> Result<String> {
    let mut s_buffer = Vec::new();
    let mut handler = stream.take(len);
    handler.read_to_end(&mut s_buffer)?;
    Ok(GB18030.decode(s_buffer.as_slice(), DecoderTrap::Ignore).map_err(|e| anyhow!("{:?}", e))?)
}

fn read_bytes_to_file(mut stream : & mut BufReader<TcpStream>, len: u64, file_path: &PathBuf) -> Result<()> {
    let mut f: File = File::create(file_path).unwrap();
    info!("file len {:?}", len);
    let mut handler = stream.take(len as u64);
    let mut buf = [0; 1024 * 4];
    while let Ok(bytes_read) = handler.read(&mut buf) {
        if bytes_read == 0 { break; }
        f.write(&buf[..bytes_read])?;
    }
    Ok(())
}

///
/// unkown filesize can use follow
/// 可以不需要文件长度的读
///
fn read_bytes_to_file_unsize(mut stream : & mut BufReader<TcpStream>, file_path: &PathBuf) -> Result<()> {
    let mut file: File = File::create(file_path).unwrap();
    loop {
        let mut buffer = [0; 2048];
        let num = stream.read(&mut buffer[..])?;
        if num == 0 {
            break;
        }
        file.write(&buffer[0..num])?;
    }
    Ok(())
}
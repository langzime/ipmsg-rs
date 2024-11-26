use crate::constants::protocol::{
    self, IPMSG_FILE_DIR, IPMSG_FILE_REGULAR, IPMSG_FILE_RETPARENT, IPMSG_GETDIRFILES, IPMSG_GETFILEDATA, IPMSG_PACKET_DELIMITER, IPMSG_SENDMSG,
};
use crate::core::GLOBLE_SENDER;
use crate::models::event::ModelEvent;
use crate::models::model::{Packet, ReceivedSimpleFileInfo, ShareInfo};
use anyhow::{anyhow, Result};
use encoding::all::GB18030;
use encoding::{DecoderTrap, Encoding};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::{debug, info};

pub static DOWNLOAD_TASK_LIST: Lazy<Arc<Mutex<HashMap<u32, PoolFile>>>> = Lazy::new(|| return Default::default());

#[derive(Clone, Debug)]
pub struct PoolFile {
    pub status: u8, //0 初始 1 下载中
    pub file_info: ReceivedSimpleFileInfo,
}

#[derive(Clone, Debug)]
pub struct DownloadTaskPool;

impl DownloadTaskPool {
    pub fn add_task(file_info: ReceivedSimpleFileInfo, save_path: PathBuf, download_ip: String) {
        {
            let mut lock = DOWNLOAD_TASK_LIST.lock().unwrap();
            let file = lock.get(&file_info.file_id);
            if let Some(p_file) = file {
                if p_file.status == 1 {
                    //下载中
                    GLOBLE_SENDER
                        .send(ModelEvent::DownloadIsBusy { file: file_info })
                        .expect("send DownloadIsBusy fail!");
                    return;
                }
            } else {
                lock.insert(
                    file_info.file_id,
                    PoolFile {
                        status: 1,
                        file_info: file_info.clone(),
                    },
                );
            }
        }
        tokio::spawn(async move {
            let download_url = format!("{}:{}", download_ip, protocol::IPMSG_DEFAULT_PORT);
            let is_ok = download(download_url, save_path, file_info.clone()).await.is_ok();
            {
                let mut lock = DOWNLOAD_TASK_LIST.lock().unwrap();
                let mut file = lock.get(&file_info.file_id);
                if let Some(p_file) = file.take() {
                    if is_ok {
                        lock.remove(&file_info.file_id);
                        GLOBLE_SENDER
                            .send(ModelEvent::RemoveDownloadTaskInPool {
                                packet_id: file_info.packet_id,
                                file_id: file_info.file_id,
                                download_ip,
                            })
                            .expect("send RemoveDownloadTaskInPool fail!");
                    } else {
                        let mut tmp_file = p_file.clone();
                        tmp_file.status = 0;
                        lock.insert(tmp_file.file_info.file_id, tmp_file);
                    }
                }
            }
        });
    }
}

pub async fn download<A: ToSocketAddrs, S: AsRef<Path>>(addr: A, to_path: S, r_file: ReceivedSimpleFileInfo) -> Result<()> {
    info!("start download file");
    let file_type = r_file.attr as u32;
    let mut stream = TcpStream::connect(addr).await?;
    let packet = Packet::new(
        IPMSG_SENDMSG | if file_type == IPMSG_FILE_DIR { IPMSG_GETDIRFILES } else { IPMSG_GETFILEDATA },
        Some(format!("{:x}:{:x}:0:\u{0}", r_file.packet_id, r_file.file_id)),
    );
    stream.write(packet.to_string().as_bytes()).await?;
    debug!("filetype {}", file_type);
    if file_type == IPMSG_FILE_REGULAR {
        let mut file_location = to_path.as_ref().to_path_buf();
        file_location.push(r_file.name);
        let file_size = r_file.size;
        let mut buffer = BufReader::new(stream);
        read_bytes_to_file(&mut buffer, file_size, &file_location).await?;
    } else if file_type == IPMSG_FILE_DIR {
        let mut next_path = to_path.as_ref().to_path_buf();
        let mut buffer = BufReader::new(stream);
        while let Ok(Some(header_size_str)) = read_delimiter(&mut buffer).await {
            let header_size = u64::from_str_radix(&header_size_str, 16)?;
            info!("header_size {:?}", header_size);
            let header_context_str = read_bytes(&mut buffer, (header_size - 1 - header_size_str.as_bytes().len() as u64)).await?; //-1是减去的那个冒号
            let v: Vec<&str> = header_context_str.splitn(4, |c| c == ':').collect();
            let file_name = v[0];
            let file_size = u64::from_str_radix(v[1], 16)?;
            let file_attr = u32::from_str_radix(v[2], 16)?;
            let opt = protocol::get_opt(file_attr);
            let cmd = protocol::get_mode(file_attr);
            info!("header context {:?}", v);
            if cmd == IPMSG_FILE_DIR {
                next_path.push(file_name);
                if !next_path.exists() {
                    fs::create_dir(&next_path).await?;
                }
                info!("crate dir{:?}", next_path);
            } else if cmd == IPMSG_FILE_REGULAR {
                next_path.push(file_name);
                info!("crate file{:?}", next_path);
                read_bytes_to_file(&mut buffer, file_size, &next_path).await?;
                next_path.pop();
            } else if cmd == IPMSG_FILE_RETPARENT {
                next_path.pop();
                info!("back to parent {:?}", next_path);
            } else {
            }
        }
    }
    info!("download end!");
    Ok(())
}

async fn read_delimiter(mut stream: &mut BufReader<TcpStream>) -> Result<Option<String>> {
    let mut s_buffer = Vec::new();
    let len = stream.read_until(u8::try_from(IPMSG_PACKET_DELIMITER)?, &mut s_buffer).await?;
    if len != 0usize {
        if len > 200 {
            return Err(anyhow!("read_delimiter error!"));
        } else {
            s_buffer.pop();
            Ok(Some(String::from_utf8(s_buffer).unwrap()))
        }
    } else {
        Ok(None)
    }
}

async fn read_bytes(mut stream: &mut BufReader<TcpStream>, len: u64) -> Result<String> {
    let mut s_buffer = Vec::new();
    let mut handler = stream.take(len);
    handler.read_to_end(&mut s_buffer).await?;
    Ok(GB18030.decode(s_buffer.as_slice(), DecoderTrap::Ignore).map_err(|e| anyhow!("{:?}", e))?)
}

async fn read_bytes_to_file(mut stream: &mut BufReader<TcpStream>, len: u64, file_path: &PathBuf) -> Result<()> {
    let mut f = File::create(file_path).await.unwrap();
    info!("file len {:?}", len);
    let mut handler = stream.take(len);
    let mut buf = [0; 1024 * 4];
    while let Ok(bytes_read) = handler.read(&mut buf).await {
        if bytes_read == 0 {
            break;
        }
        f.write(&buf[..bytes_read]).await?;
    }
    Ok(())
}

///
/// unkown filesize can use follow
/// 可以不需要文件长度的读
///
async fn read_bytes_to_file_unsize(mut stream: &mut BufReader<TcpStream>, file_path: &PathBuf) -> Result<()> {
    let mut file: File = File::create(file_path).await?;
    loop {
        let mut buffer = [0; 2048];
        let num = stream.read(&mut buffer[..]).await?;
        if num == 0 {
            break;
        }
        file.write(&buffer[0..num]).await?;
    }
    Ok(())
}

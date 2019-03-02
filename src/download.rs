use std::io::prelude::*;
use std::net::TcpStream;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::thread;
use std::fmt::{self, Display};
use std::error::Error;
use std::fs::{self, File, Metadata, ReadDir};
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use crate::constant::{self, IPMSG_SENDMSG, IPMSG_GETFILEDATA, IPMSG_GETDIRFILES, IPMSG_FILE_DIR, IPMSG_FILE_REGULAR, IPMSG_FILE_RETPARENT};
use crate::events::model::ModelEvent;
use crate::model::Packet;
use crate::model::{FileInfo, ReceivedSimpleFileInfo};

#[derive(Debug)]
pub enum DownLoadError {
    IoError(io::Error),
    InValidType,
    ReaDelimiterErr,
}

impl Display for DownLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DownLoadError::IoError(ref err) => err.fmt(f),
            DownLoadError::InValidType => write!(f, "InValidType"),
            DownLoadError::ReaDelimiterErr => write!(f, "ReaDelimiterErr")
        }
    }
}

impl Error for DownLoadError {
    fn description(&self) -> &str {
        match *self {
            DownLoadError::IoError(ref err) => err.description(),
            DownLoadError::InValidType => "InValidType",
            DownLoadError::ReaDelimiterErr => "DownLoadError"
        } }
    fn cause(&self) -> Option<&Error> {
        match *self {
            DownLoadError::IoError(ref err) => Some(err),
            DownLoadError::InValidType => None,
            DownLoadError::ReaDelimiterErr => None
        }
    }
}

#[derive(Clone, Debug)]
pub struct ManagerPool {
    pub file_pool: Arc<Mutex<HashMap<u32, PoolFile>>>,
    pub model_sender: crossbeam_channel::Sender<ModelEvent>
}

impl ManagerPool {

    pub fn new(file_pool: Arc<Mutex<HashMap< u32, PoolFile>>>, model_event_sender: crossbeam_channel::Sender<ModelEvent>) -> ManagerPool {
        ManagerPool { file_pool, model_sender: model_event_sender }
    }

    pub fn run(mut self, file_info: ReceivedSimpleFileInfo, save_path: PathBuf, download_ip: String) {
        let p1 = self.clone();
        {
            let mut lock = p1.file_pool.lock().unwrap();
            let sender = p1.model_sender;
            let file = (*lock).get(&file_info.file_id);
            if let Some(p_file) = file {
                if p_file.status == 1 {
                    //下载中
                    sender.send(ModelEvent::DownloadIsBusy{ file: file_info });
                    return;
                }
            }else {
                (*lock).insert(file_info.file_id, PoolFile { status: 1, file_info: file_info.clone() });
            }
        }
        let p2 = self.clone();
        thread::spawn(move || {
            let download_url = format!("{}:{}", download_ip, constant::IPMSG_DEFAULT_PORT);
            let is_ok = download(download_url, save_path, file_info.packet_id, file_info.file_id, file_info.clone().name, file_info.clone().attr as u32).is_ok();
            {
                let sender = p2.model_sender;
                let mut lock = p2.file_pool.lock().unwrap();
                let mut file = (*lock).get(&file_info.file_id);
                if let Some(p_file) = file.take() {
                    if is_ok {
                        (*lock).remove(&file_info.file_id);
                        sender.send(ModelEvent::RemoveDownloadTaskInPool{ packet_id: file_info.packet_id, file_id: file_info.file_id, download_ip });
                    }else{
                        let mut tmp_file = p_file.clone();
                        tmp_file.status = 0;
                        (*lock).insert(tmp_file.file_info.file_id, tmp_file);
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

impl From<io::Error> for DownLoadError {
    fn from(err: io::Error) -> DownLoadError {
        DownLoadError::IoError(err)
    }
}

pub fn download<A: ToSocketAddrs, S: AsRef<Path>>(addr: A, to_path: S, packet_id: u32, file_id: u32, name: String, file_type: u32) -> Result<(), DownLoadError> {
    info!("start download file");
    let mut stream = TcpStream::connect(addr)?;
    let path: &Path = to_path.as_ref();
    let metadata: Metadata = fs::metadata(path)?;
    let packet = Packet::new(IPMSG_SENDMSG| if file_type == IPMSG_FILE_DIR { IPMSG_GETDIRFILES } else { IPMSG_GETFILEDATA }, Some(format!("{:x}:{:x}:0:\u{0}", packet_id, file_id)));
    stream.write(packet.to_string().as_bytes())?;
    debug!("filetype {}", file_type);
    if file_type == IPMSG_FILE_REGULAR {
        let mut file_location = path.to_path_buf();
        file_location.push(name);
        let mut file = File::create(&file_location)?;
        loop {
            let mut buffer = [0; 2048];
            let num = stream.read(&mut buffer[..])?;
            if num == 0 {
                break;
            }
            file.write(&buffer[0..num])?;
        }
    }else if file_type == IPMSG_FILE_DIR {
        let mut base_file_location = path.to_path_buf();
        let mut buffer = BufReader::new(stream);
        let mut path_infos = PathInfos{
            inner: buffer,
            next_path: base_file_location,
        };
        for path_info in path_infos {

        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct PathInfos {
    inner: BufReader<TcpStream>,
    next_path: PathBuf,
}

impl Iterator for PathInfos {
    type Item = Result<(), DownLoadError>;

    fn next(&mut self) -> Option<Result<(), DownLoadError>> {
        let ref mut stream = self.inner;
        let ref mut next_path = self.next_path.clone();
        match read_delimiter(stream) {
            Ok(option)  => match option {
                Some(header_size_str) => {
                    let header_size = u64::from_str_radix(&header_size_str, 16).unwrap();
                    info!("header_size {:?}", header_size);
                    let header_context_str = read_bytes(stream, (header_size - 1 - header_size_str.as_bytes().len() as u64));//-1是减去的那个冒号
                    let v: Vec<&str> = header_context_str.splitn(4, |c| c == ':').collect();
                    let file_name = v[0];
                    let file_size = u64::from_str_radix(v[1], 16).unwrap();
                    let file_attr = u32::from_str_radix(v[2], 16).unwrap();
                    let opt = constant::get_opt(file_attr);
                    let cmd = constant::get_mode(file_attr);
                    info!("header context {:?}", v);
                    if cmd == IPMSG_FILE_DIR {
                        next_path.push(file_name);
                        if !next_path.exists() {
                            fs::create_dir(&next_path).unwrap();
                        }
                        info!("crate dir{:?}", next_path);
                    }else if cmd == IPMSG_FILE_REGULAR {
                        next_path.push(file_name);
                        info!("crate file{:?}", next_path);
                        read_bytes_to_file(stream, file_size, &next_path);
                        next_path.pop();
                    }else if cmd == IPMSG_FILE_RETPARENT  {
                        info!("back to parent");
                        next_path.pop();
                    }else {

                    }
                    self.next_path = next_path.to_path_buf();
                    Some(Ok(()))
                },
                None => None,
            },
            Err(x) => Some(Err(x)),
        }
    }
}

fn read_delimiter(mut stream : & mut BufReader<TcpStream>) -> Result<Option<String>, DownLoadError> {
    let mut s_buffer = Vec::new();
    let len = stream.read_until(b':', &mut s_buffer)?;
    if len != 0usize {
        if len > 200 {
            Err(DownLoadError::ReaDelimiterErr)
        }else {
            s_buffer.pop();
            Ok(Some(String::from_utf8(s_buffer).unwrap()))
        }
    }else {
        Ok(None)
    }
}

fn read_bytes(mut stream : & mut BufReader<TcpStream>, len: u64) -> String {
    let mut s_buffer = Vec::new();
    let mut handler = stream.take(len);
    handler.read_to_end(&mut s_buffer);
    GB18030.decode(s_buffer.as_slice(), DecoderTrap::Ignore).unwrap()
}

fn read_bytes_to_file(mut stream : & mut BufReader<TcpStream>, len: u64, file_path: &PathBuf) {
    let mut f: File = File::create(file_path).unwrap();
    info!("file len {:?}", len);
    let mut handler = stream.take(len as u64);
    let mut buf = [0; 1024 * 4];
    while let Ok(bytes_read) = handler.read(&mut buf) {
        info!("file in ...");
        if bytes_read == 0 { break; }
        f.write(&buf[..bytes_read]);
    }
}

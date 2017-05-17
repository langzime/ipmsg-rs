use std::io::prelude::*;
use std::net::TcpStream;
use std::io::Result;
use std::io::Error;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::fs::{self, File, Metadata, ReadDir};
use std::net::ToSocketAddrs;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use constant::{IPMSG_SENDMSG, IPMSG_GETFILEDATA, IPMSG_GETDIRFILES, IPMSG_FILE_DIR, IPMSG_FILE_REGULAR, IPMSG_FILE_RETPARENT};
use model::Packet;

pub fn download<A: ToSocketAddrs, S: AsRef<Path>>(addr: A, to_path: S, packet_id: u32, file_id: u32, name: String, file_type: u32) -> Result<()> {
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
        //header-size:filename:file-size:fileattr:contents-data
        let mut base_file_location = path.to_path_buf();
        base_file_location.push(name);
        if !base_file_location.exists() {
            fs::create_dir(&base_file_location)?;
        }
        let root_path = base_file_location;
        let mut buffer = BufReader::new(stream);
        download_file(&mut buffer, &root_path);
    }
    Ok(())
}

fn download_file<S>(mut stream : & mut BufReader<TcpStream>, next_base_path: S) -> Result<()> where S: AsRef<Path> {
    if let Some(header_size_str) = read_delimiter(stream){
        let mut next_path: PathBuf = next_base_path.as_ref().to_path_buf();
        let header_size = u64::from_str_radix(&header_size_str, 16).unwrap();
        info!("header_size {:?}", header_size);
        let header_context_str = read_bytes(&mut stream, (header_size - 1 - header_size_str.as_bytes().len() as u64));//-1是减去的那个冒号
        let v: Vec<&str> = header_context_str.splitn(4, |c| c == ':').collect();
        let file_name = v[0];
        let file_size = u64::from_str_radix(v[1], 16).unwrap();;
        let file_attr = v[2].parse::<u32>().unwrap();
        info!("header context {:?}", v);
        if file_attr == IPMSG_FILE_DIR {
            next_path.push(file_name);
            if !next_path.exists() {
                fs::create_dir(&next_path)?;
            }
            info!("crate dir{:?}", next_path);
            download_file(&mut stream, next_path)?;
        }else if file_attr == IPMSG_FILE_REGULAR {
            let tmp_path = next_path.clone();
            info!("base dir {:?}", tmp_path);
            next_path.push(file_name);
            //create path
            read_bytes_to_file(&mut stream, file_size, &next_path);
            //传入下一个的是目录，可能得在去掉文件名，在往下传
            download_file(&mut stream, tmp_path)?;
        }else if file_attr == IPMSG_FILE_RETPARENT  {
            //root 从哪里读取
            info!("back to parent");
            next_path.pop();
            download_file(&mut stream, next_path)?;
        }
    }
    Ok(())
}

fn read_delimiter(mut stream : & mut BufReader<TcpStream>) -> Option<String> {
    let mut s_buffer = Vec::new();
    if let Ok(buffer) = stream.read_until(b':', &mut s_buffer) {
        if buffer != 0usize {
            s_buffer.pop();
            Some(GB18030.decode(s_buffer.as_slice(), DecoderTrap::Ignore).unwrap())
        }else {
            None
        }
    }else {
        None
    }
}

fn read_bytes(mut stream : & mut BufReader<TcpStream>, len: u64) -> String {
    let mut s_buffer = Vec::new();
    let mut handler = stream.take(len);
    handler.read_to_end(&mut s_buffer);
    GB18030.decode(s_buffer.as_slice(), DecoderTrap::Ignore).unwrap()
}

fn read_bytes_to_file<S>(mut stream : & mut BufReader<TcpStream>, len: u64, file_path: S) where S: AsRef<Path> {
    let mut f: File = File::create(&file_path).unwrap();
    let mut handler = stream.take(len as u64);
    let mut buf = [0; 1024 * 4];
    while let Ok(bytes_read) = handler.read(&mut buf) {
        if bytes_read == 0 { break; }
        f.write(&buf[..bytes_read]).unwrap();
    }
}

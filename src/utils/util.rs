use crate::models::model::Packet;
use combine::error::ParseError;
use combine::{many, many1, satisfy, token, Parser, Stream};
use encoding::all::GB18030;
use encoding::{EncoderTrap, Encoding};
use std::path::PathBuf;
use tracing::instrument;

pub fn utf8_to_gb18030(ori_str: &str) -> Vec<u8> {
    GB18030.encode(&ori_str, EncoderTrap::Strict).unwrap()
}

#[instrument]
pub fn packet_parser<Input>() -> impl Parser<Input, Output = Packet>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(satisfy(|c| c != ':')),
        token(':'),
        many1(satisfy(|c| c != ':')),
        token(':'),
        many1(satisfy(|c| c != ':')),
        token(':'),
        many1(satisfy(|c| c != ':')),
        token(':'),
        many1(satisfy(|c| c != ':')),
        token(':'),
        many(satisfy(|c| true)),
    )
        .map(
            |(verson, _, send_temp, _, hostname, _, host, _, cmd, _, ext): (String, _, String, _, String, _, String, _, String, _, String)| {
                let add_ext = if ext.is_empty() { None } else { Some(ext) };
                Packet::from(verson, send_temp, hostname, host, cmd.parse::<u32>().unwrap(), add_ext)
            },
        )
}

pub fn get_config_dir() -> PathBuf {
    if let Some(mut tmp) = dirs::config_dir() {
        tmp = tmp.join("ipmsg-rs");
        if tmp.exists() == false {
            std::fs::create_dir_all(&tmp).expect("create dir failed");
        }
        tmp
    } else {
        panic!("获取操作系统配置目录失败！");
    }
}

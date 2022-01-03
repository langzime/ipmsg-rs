use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::Utc;
use chrono::DateTime;
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};
use chrono::TimeZone;
use combine::error::ParseError;
use combine::{many1, many, Parser, Stream, sep_by, token, skip_many, skip_many1, satisfy, choice, optional, any};
use combine::parser::range::{take_while, take_while1, take_until_range};
use combine::parser::char::{letter, space, digit, char};
use crate::model::Packet;

pub fn utf8_to_gb18030<'a>(ori_str :&'a str) -> Vec<u8> {
    GB18030.encode(&ori_str, EncoderTrap::Strict).unwrap()
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Utc> {
    let dur = t.duration_since(UNIX_EPOCH).unwrap();
    Utc.timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}

pub fn packet_parser<Input>() -> impl Parser<Input, Output=Packet>
    where
        Input: Stream<Token=char>,
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
    ).map(|(verson, _, send_temp, _, hostname, _, host, _, cmd, _, ext): (String, _, String, _, String, _, String, _, String, _, String)| {
        let add_ext = if ext.is_empty() {
            None
        }else{
            Some(ext)
        };
        Packet::from(verson, send_temp, hostname, host, cmd.parse::<u32>().unwrap(), add_ext)
    })
}
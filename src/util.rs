use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::Utc;
use chrono::DateTime;
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};
use chrono::TimeZone;
use combine::error::ParseError;
use combine::{many1, many, Parser, Stream, sep_by, token, skip_many, skip_many1, satisfy, choice, optional, any};
use combine::range::{take_while, take_while1, take_until_range};
use combine::char::{letter, space, digit, char};
use crate::model::Packet;

pub fn utf8_to_gb18030<'a>(ori_str :&'a str) -> Vec<u8> {
    GB18030.encode(&ori_str, EncoderTrap::Strict).unwrap()
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Utc> {
    let dur = t.duration_since(UNIX_EPOCH).unwrap();
    Utc.timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}

#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn packet_parser<I>() -> impl Parser<Input=I, Output=Packet>
    where
        I: Stream<Item=char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
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
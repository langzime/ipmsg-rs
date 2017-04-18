use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;
use chrono::UTC;
use chrono::DateTime;
use std::time::UNIX_EPOCH;
use std::time::{Duration, SystemTime};
use chrono::TimeZone;

pub fn utf8_to_gb18030<'a>(ori_str :&'a str) -> Vec<u8> {
    GB18030.encode(&ori_str, EncoderTrap::Strict).unwrap()
}

pub fn system_time_to_date_time(t: SystemTime) -> DateTime<UTC> {
    let dur = t.duration_since(UNIX_EPOCH).unwrap();
    UTC.timestamp(dur.as_secs() as i64, dur.subsec_nanos())
}
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::GB18030;

pub fn utf8_to_gb18030<'a>(ori_str :&'a str) -> Vec<u8> {
    GB18030.encode(&ori_str, EncoderTrap::Strict).unwrap()
}
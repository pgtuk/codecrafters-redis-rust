use std::io::Cursor;

use super::frame::Frame;

pub fn make_frame(input: &[u8]) -> Frame {
    let mut cursor = Cursor::new(&input[..]);

    Frame::parse(&mut cursor).unwrap()
}
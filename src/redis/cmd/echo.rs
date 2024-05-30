use anyhow::Result;
use bytes::Bytes;

use super::{
    Frame,
    Parser
};

#[derive(Debug, PartialEq)]
pub struct Echo {
    msg: Bytes,
}

impl Echo {
    pub fn new(msg: Bytes) -> Echo {
        Echo { msg }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Echo> {
        match parser.next_string() {
            Ok(msg) => Ok(Echo::new(msg.into())),
            Err(e) => Err(e.into())
        }
    }

    pub fn apply(self) -> Frame {
        self.to_frame()
    }

    fn to_frame (self) -> Frame {
        Frame::Bulk(self.msg)
    }
}
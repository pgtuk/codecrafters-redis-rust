use anyhow::Result;
use bytes::Bytes;

use crate::redis::{
    cmd::ClientCmd,
    utils::Named,
};

use super::{Frame, Parser};

#[derive(Debug, PartialEq, Clone)]
pub struct Echo {
    msg: Bytes,
}

impl Named for Echo {
    const NAME: &'static str = "ECHO";
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

    pub fn apply(&self) -> Frame {
        Frame::Bulk(self.msg.clone())
    }
}

impl ClientCmd for Echo {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Echo::NAME.into()));
        frame.add(Frame::Bulk(self.msg.clone()));

        frame
    }
}
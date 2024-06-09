use anyhow::Result;

use crate::redis::{
    frame::Frame,
    parser::{
        Parser, 
        ParserError,
    },
};

#[derive(Debug, PartialEq)]
pub struct Ping {
    msg: Option<String>,
}

impl Ping {
    pub fn new(msg: Option<String>) -> Ping {
        Ping { msg }
    }

    pub fn parse_args(parser: &mut Parser) -> Result<Ping> {
        match parser.next_string() {
            Ok(args) => Ok(Ping::new(Some(args))),
            Err(ParserError::EndOfStream) => Ok(Ping::new(None)),
            Err(e) => Err(e.into()),
        }
    }

    pub fn apply(self) -> Frame {
        match self.msg {
            None => Frame::Simple("PONG".to_string()),
            Some(msg) => Frame::Bulk(msg.into()),
        }
    }

    pub fn to_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk("PING".into()));
        if let Some(msg) = self.msg {
            frame.add(Frame::Bulk(msg.into()))
        }

        frame
    }
}

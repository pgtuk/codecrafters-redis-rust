use anyhow::Result;

use crate::redis::{
    frame::Frame,
    parser::{
        Parser, 
        ParserError,
    },
    utils::Named,
};
use crate::redis::connection::Connection;

use super::client_cmd::ClientCmd;

#[derive(Debug, PartialEq)]
pub struct Ping {
    msg: Option<String>,
}

impl Named for Ping {
    const NAME: &'static str = "PING";
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

    pub async fn apply(self, conn: &mut Connection) -> Result<()> {
        let frame = match self.msg {
            None => Frame::Simple("PONG".to_string()),
            Some(msg) => Frame::Bulk(msg.into()),
        };

        conn.write_frame(&frame).await?;

        Ok(())
    }
}

impl ClientCmd for Ping {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Ping::NAME.into()));
        if let Some(msg) = &self.msg {
            frame.add(Frame::Bulk(msg.clone().into()))
        }

        frame
    }
}

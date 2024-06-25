use anyhow::Result;

use crate::redis::{
    db::Db,
    frame::Frame,
    parser::Parser,
};
use crate::redis::cmd::ClientCmd;
use crate::redis::connection::Connection;
use crate::redis::utils::Named;

#[derive(Debug, PartialEq)]
pub struct Get {
    key: String,
}

impl Named for Get {
    const NAME: &'static str = "GET";
}

impl Get {
    pub fn new(key: String) -> Get {
        Get { key }
    }
 
    pub fn parse_args(parser: &mut Parser) -> Result<Get> {
        match parser.next_string() {
            Ok(key) => Ok(Get::new(key)),
            Err(e) => Err(e.into())
        }
    }

    pub async fn apply(self, conn: &mut Connection, db: &mut Db) -> Result<()> {
        let frame = match db.get(&self.key) {
            Some(data) => Frame::Bulk(data),
            None => Frame::Null,
        };

        conn.write_frame(&frame).await?;

        Ok(())
    }
}

impl ClientCmd for Get {
    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        frame.add(Frame::Bulk(Get::NAME.into()));
        frame.add(Frame::Bulk(self.key.clone().into()));

        frame
    }
}

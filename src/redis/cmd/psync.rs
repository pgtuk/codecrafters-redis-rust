use anyhow::Result;
use base64::prelude::*;

use crate::redis::{
    frame::Frame,
    parser::Parser,
    ServerInfo, utils::Named,
};
use crate::redis::connection::Connection;

use super::ClientCmd;

const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

#[derive(Debug, PartialEq)]
pub struct Psync {
    pub replication_id: String,
    pub offset: i32,
}

impl Named for Psync {
    const NAME: &'static str = "PSYNC";
}

impl Psync {
    pub fn parse_args(_: &mut Parser) -> Result<Psync> {
        // TODO implement
        Ok(Psync { replication_id: "replication_id".to_string(), offset: 1 })
    }

    pub async fn apply(self, conn: &mut Connection, info: &ServerInfo) -> Result<()> {
        let frame = Frame::Simple(format!("FULLRESYNC {} 0", info.replinfo.repl_id));

        conn.write_frame(&frame).await?;
        conn.write_rdb(&Psync::build_rdb_frame()).await?;

        Ok(())
    }

    fn build_rdb_frame() -> Vec<u8> {
        BASE64_STANDARD.decode(EMPTY_RDB).unwrap()
    }
}

impl Default for Psync {
    fn default() -> Self {
        Psync { replication_id: "?".to_string(), offset: -1 }
    }
}

impl ClientCmd for Psync {

    fn to_frame(&self) -> Frame {
        let mut frame = Frame::array();

        let items: Vec<String> = vec![
            Psync::NAME.to_string(), 
            self.replication_id.clone(), 
            self.offset.to_string(),
        ];

        for item in items {
            frame.add(Frame::Bulk(item.into()))
        }
   
        frame
    }
}
